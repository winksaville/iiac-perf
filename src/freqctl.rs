//! Explicit CPU-frequency control: the `read-freq` / `pin-freq` / `restore-freq` command words
//! and the `--pin-freq` run guard.
//!
//! [`crate::freq`] reads and never writes. This module is the one place the harness mutates the
//! box, and the reasons mutation used to be banned outright (root, global, persistent) survive
//! as its design constraints:
//!
//! - mutation happens only on an explicit command word or flag, never as a side effect
//! - a pinned run restores on every catchable exit path: normal return and panic via the
//!   guard's `Drop`, SIGINT and SIGTERM via a pre-rendered async-signal-safe handler
//! - restore converges to a steady state the user *declared* in the config's `[freq]` section,
//!   never to a remembered one. A remembered state ratchets on back-to-back runs (run 2
//!   "restores" run 1's pin), and transient state files are a demonstrated failure mode in this
//!   repo. Declared-state convergence is idempotent from any starting point, which is also the
//!   whole recovery story after SIGKILL or power loss: run `restore-freq`, nothing saved to
//!   find
//!
//! Writes are staged as ordered **plans** of (path, token) pairs because cpufreq rejects
//! `min > max`: every clamp move goes floor-first (min to the hardware floor, then max, then
//! min), so each intermediate write is valid from any starting state. A pin is `min = max` at
//! the target with boost off, and the load-independent default target is the base clock
//! ([`freq::base_clock`]), the manufacturer's guaranteed all-core sustained frequency.

use std::ffi::CString;
use std::sync::OnceLock;

use crate::config::FreqConfig;
use crate::freq;

/// An ordered list of sysfs writes: (absolute path, token). Order is load-bearing, see the
/// module doc's floor-first rule.
type Plan = Vec<(String, String)>;

/// The kernel's single boost knob when the driver has no per-CPU `boost` files.
const GLOBAL_BOOST: &str = "/sys/devices/system/cpu/cpufreq/boost";

/// Path of one per-CPU cpufreq knob.
fn knob(cpu: usize, name: &str) -> String {
    format!("/sys/devices/system/cpu/cpu{cpu}/cpufreq/{name}")
}

/// Which boost control the box exposes, decided once at capability read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoostKnob {
    /// No boost control at all (rpi5-20cd).
    None,
    /// Per-CPU `cpufreq/boost` files.
    PerCpu,
    /// The single global `cpufreq/boost` file.
    Global,
}

/// One CPU's write targets and validation data.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CpuCaps {
    /// The logical CPU number.
    cpu: usize,
    /// `cpuinfo_min_freq` (kHz): the hardware floor, and the staging value for clamp moves.
    hw_min_khz: u64,
    /// `cpuinfo_max_freq` (kHz): the hardware ceiling.
    hw_max_khz: u64,
    /// Discrete `scaling_available_frequencies` (kHz) where the driver lists them, `None` on
    /// continuous-range drivers (amd-pstate, intel_pstate).
    avail_khz: Option<Vec<u64>>,
}

/// What the box exposes, read once per command: write targets plus the data that validates a
/// declared state against reality.
#[derive(Debug, Clone, PartialEq, Eq)]
struct BoxCaps {
    /// Per-CPU capabilities, ascending by CPU.
    cpus: Vec<CpuCaps>,
    /// Whether `energy_performance_preference` exists (amd-pstate-epp yes, rpi5-20cd no).
    has_epp: bool,
    /// `scaling_available_governors`, when readable, for validating the declared governor.
    governors: Option<Vec<String>>,
    /// `energy_performance_available_preferences`, when readable.
    epp_prefs: Option<Vec<String>>,
    /// Which boost control exists.
    boost: BoostKnob,
}

/// Read the box's capabilities from sysfs. Errors when there is no cpufreq to control (a VM,
/// typically), naming what is missing.
fn read_caps() -> Result<BoxCaps, String> {
    let cpu_ids = freq::cpus();
    let Some(&first) = cpu_ids.first() else {
        return Err("no CPUs under /sys/devices/system/cpu".to_string());
    };
    let mut cpus = Vec::with_capacity(cpu_ids.len());
    for cpu in cpu_ids {
        let (Some(hw_min_khz), Some(hw_max_khz)) = (
            freq::read_khz(cpu, "cpuinfo_min_freq"),
            freq::read_khz(cpu, "cpuinfo_max_freq"),
        ) else {
            return Err(format!(
                "cpu{cpu} exposes no cpufreq range (cpuinfo_min_freq/cpuinfo_max_freq): \
                 nothing to control"
            ));
        };
        cpus.push(CpuCaps {
            cpu,
            hw_min_khz,
            hw_max_khz,
            avail_khz: freq::available_khz(cpu),
        });
    }
    let boost = if freq::read_token(first, "boost").is_some() {
        BoostKnob::PerCpu
    } else if std::path::Path::new(GLOBAL_BOOST).exists() {
        BoostKnob::Global
    } else {
        BoostKnob::None
    };
    Ok(BoxCaps {
        cpus,
        has_epp: freq::read_token(first, "energy_performance_preference").is_some(),
        governors: token_list(first, "scaling_available_governors"),
        epp_prefs: token_list(first, "energy_performance_available_preferences"),
        boost,
    })
}

/// One whitespace-separated token-list file for `cpu`, `None` when absent.
fn token_list(cpu: usize, name: &str) -> Option<Vec<String>> {
    let text = freq::read_token(cpu, name)?;
    Some(text.split_whitespace().map(str::to_string).collect())
}

/// The declared steady state, validated against the box: what `restore-freq` converges to.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Steady {
    /// The declared governor token.
    governor: String,
    /// The declared EPP token, present exactly when the box has EPP.
    epp: Option<String>,
    /// The declared boost switch, present exactly when the box has a boost knob.
    boost: Option<bool>,
    /// Declared lower clamp (kHz), `None` meaning each CPU's hardware floor.
    min_khz: Option<u64>,
    /// Declared upper clamp (kHz), `None` meaning each CPU's hardware ceiling.
    max_khz: Option<u64>,
}

/// The refusal printed when no `[freq]` table is declared: pinning without a declared way home
/// is exactly the failure mode this design exists against.
fn no_steady_state() -> String {
    "no [freq] steady state is declared, so a pin would have no way home.\n\
     Add a [freq] section to the config, usually ~/.config/iiac-perf/config.md (in a toml \
     fence).\n\
     `iiac-perf read-freq --as-config` prints the current state in that form, ready to paste.\n\
     Under sudo, $HOME may be root's. The project-local ./iiac-perf.md works there too."
        .to_string()
}

/// Validate the declared `[freq]` table against the box's capabilities.
///
/// The box-dependent completeness rule lives here: a knob the box exposes must be declared
/// (restoring around it would silently leave a pin's residue), and a knob the box lacks must
/// not be (the declaration would be fiction).
fn resolve_steady(cfg: Option<&FreqConfig>, caps: &BoxCaps) -> Result<Steady, String> {
    let Some(cfg) = cfg else {
        return Err(no_steady_state());
    };
    if let Some(governors) = &caps.governors
        && !governors.contains(&cfg.governor)
    {
        return Err(format!(
            "freq.governor {:?} is not available on this box (available: {})",
            cfg.governor,
            governors.join(", ")
        ));
    }
    match (caps.has_epp, &cfg.epp) {
        (true, None) => {
            let hint = match &caps.epp_prefs {
                Some(p) => format!(" (available: {})", p.join(", ")),
                None => String::new(),
            };
            return Err(format!(
                "this box exposes energy_performance_preference: declare freq.epp{hint}"
            ));
        }
        (false, Some(_)) => {
            return Err("freq.epp is declared but this box exposes no \
                        energy_performance_preference"
                .to_string());
        }
        (true, Some(epp)) => {
            if let Some(prefs) = &caps.epp_prefs
                && !prefs.contains(epp)
            {
                return Err(format!(
                    "freq.epp {:?} is not available on this box (available: {})",
                    epp,
                    prefs.join(", ")
                ));
            }
        }
        (false, None) => {}
    }
    match (caps.boost, cfg.boost) {
        (BoostKnob::None, Some(_)) => {
            return Err("freq.boost is declared but this box exposes no boost control".to_string());
        }
        (BoostKnob::PerCpu | BoostKnob::Global, None) => {
            return Err("this box exposes a boost control: declare freq.boost".to_string());
        }
        _ => {}
    }
    let min_khz = cfg.min_mhz.map(|m| m * 1000);
    let max_khz = cfg.max_mhz.map(|m| m * 1000);
    for c in &caps.cpus {
        for (name, khz) in [("min_mhz", min_khz), ("max_mhz", max_khz)] {
            if let Some(khz) = khz
                && !(c.hw_min_khz..=c.hw_max_khz).contains(&khz)
            {
                return Err(format!(
                    "freq.{name} {} MHz is outside cpu{}'s range {}-{} MHz",
                    khz / 1000,
                    c.cpu,
                    c.hw_min_khz / 1000,
                    c.hw_max_khz / 1000
                ));
            }
        }
    }
    Ok(Steady {
        governor: cfg.governor.clone(),
        epp: cfg.epp.clone(),
        boost: cfg.boost,
        min_khz,
        max_khz,
    })
}

/// The boost writes for `enabled`, empty when the box has no knob.
fn boost_writes(caps: &BoxCaps, enabled: bool) -> Plan {
    let token = if enabled { "1" } else { "0" };
    match caps.boost {
        BoostKnob::None => Vec::new(),
        BoostKnob::Global => vec![(GLOBAL_BOOST.to_string(), token.to_string())],
        BoostKnob::PerCpu => caps
            .cpus
            .iter()
            .map(|c| (knob(c.cpu, "boost"), token.to_string()))
            .collect(),
    }
}

/// The clamp writes taking one CPU from any state to `[min_khz, max_khz]`, floor-first so every
/// intermediate write is valid (see the module doc).
fn clamp_writes(c: &CpuCaps, min_khz: u64, max_khz: u64) -> Plan {
    vec![
        (knob(c.cpu, "scaling_min_freq"), c.hw_min_khz.to_string()),
        (knob(c.cpu, "scaling_max_freq"), max_khz.to_string()),
        (knob(c.cpu, "scaling_min_freq"), min_khz.to_string()),
    ]
}

/// The plan converging the box to `steady`: governor and EPP first (amd-pstate ties the legal
/// EPP values to the governor), then boost, then the clamps.
fn restore_plan(steady: &Steady, caps: &BoxCaps) -> Plan {
    let mut plan = Plan::new();
    for c in &caps.cpus {
        plan.push((knob(c.cpu, "scaling_governor"), steady.governor.clone()));
        if let Some(epp) = &steady.epp {
            plan.push((knob(c.cpu, "energy_performance_preference"), epp.clone()));
        }
    }
    if let Some(enabled) = steady.boost {
        plan.extend(boost_writes(caps, enabled));
    }
    for c in &caps.cpus {
        let min_khz = match steady.min_khz {
            Some(k) => k,
            None => c.hw_min_khz,
        };
        let max_khz = match steady.max_khz {
            Some(k) => k,
            None => c.hw_max_khz,
        };
        plan.extend(clamp_writes(c, min_khz, max_khz));
    }
    plan
}

/// The plan pinning every CPU at `khz`: boost off, then `min = max = khz`. Errors when the
/// target does not fit a CPU, naming that CPU's valid values.
fn pin_plan(khz: u64, caps: &BoxCaps) -> Result<Plan, String> {
    for c in &caps.cpus {
        if let Some(avail) = &c.avail_khz {
            if !avail.contains(&khz) {
                let mhz: Vec<String> = avail.iter().map(|k| (k / 1000).to_string()).collect();
                return Err(format!(
                    "{} MHz is not one of cpu{}'s discrete frequencies ({} MHz)",
                    khz / 1000,
                    c.cpu,
                    mhz.join(", ")
                ));
            }
        } else if !(c.hw_min_khz..=c.hw_max_khz).contains(&khz) {
            return Err(format!(
                "{} MHz is outside cpu{}'s range {}-{} MHz",
                khz / 1000,
                c.cpu,
                c.hw_min_khz / 1000,
                c.hw_max_khz / 1000
            ));
        }
    }
    let mut plan = boost_writes(caps, false);
    for c in &caps.cpus {
        plan.extend(clamp_writes(c, khz, khz));
    }
    Ok(plan)
}

/// Resolve the pin target: the command line, else the config's `pin_mhz`, else the discovered
/// base clock. Returns the kHz value and a human phrase saying where it came from.
fn resolve_pin(cfg: Option<&FreqConfig>, cli_mhz: Option<u64>) -> Result<(u64, String), String> {
    if let Some(mhz) = cli_mhz {
        if mhz == 0 {
            return Err("0 is not a frequency".to_string());
        }
        return Ok((mhz * 1000, "given on the command line".to_string()));
    }
    if let Some(mhz) = cfg.and_then(|f| f.pin_mhz) {
        return Ok((mhz * 1000, "config pin_mhz".to_string()));
    }
    match freq::base_clock() {
        Some(b) => Ok((b.khz, format!("base clock from {}", b.source))),
        None => Err(
            "no pin target: none given, no [freq] pin_mhz configured, and no base \
                     clock discoverable on this box"
                .to_string(),
        ),
    }
}

/// Execute a plan in order, stopping at the first failure. A permission failure names the fix
/// plainly instead of half-working: these writes need root.
fn apply(plan: &Plan) -> Result<(), String> {
    for (path, token) in plan {
        if let Err(e) = std::fs::write(path, token) {
            let hint = if e.kind() == std::io::ErrorKind::PermissionDenied {
                " (writing cpufreq needs root: rerun under sudo)"
            } else {
                ""
            };
            return Err(format!("writing {token:?} to {path}: {e}{hint}"));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The state display: read-freq's lines and the --as-config form.
// ---------------------------------------------------------------------------

/// One CPU's displayable policy state, `None` fields simply omitted from the line.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CpuState {
    /// `scaling_governor`.
    governor: Option<String>,
    /// `energy_performance_preference`.
    epp: Option<String>,
    /// The boost token (`1` / `0`), per-CPU or global.
    boost: Option<String>,
    /// `scaling_min_freq` (kHz).
    min_khz: Option<u64>,
    /// `scaling_max_freq` (kHz).
    max_khz: Option<u64>,
}

/// Read one CPU's displayable state.
fn cpu_state(cpu: usize) -> CpuState {
    let boost = match freq::read_token(cpu, "boost") {
        Some(t) => Some(t),
        None => std::fs::read_to_string(GLOBAL_BOOST)
            .ok()
            .map(|s| s.trim().to_string()),
    };
    CpuState {
        governor: freq::read_token(cpu, "scaling_governor"),
        epp: freq::read_token(cpu, "energy_performance_preference"),
        boost,
        min_khz: freq::read_khz(cpu, "scaling_min_freq"),
        max_khz: freq::read_khz(cpu, "scaling_max_freq"),
    }
}

/// kHz as a `GHz` display value with two decimals (`3800000` -> `3.80`).
fn ghz(khz: u64) -> String {
    format!("{:.2}", khz as f64 / 1e6)
}

/// A compressed CPU list: `[0,1,2,3,8]` -> `0-3,8`.
fn cpu_list(cpus: &[usize]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < cpus.len() {
        let start = cpus[i];
        let mut end = start;
        while i + 1 < cpus.len() && cpus[i + 1] == end + 1 {
            i += 1;
            end = cpus[i];
        }
        parts.push(if start == end {
            start.to_string()
        } else {
            format!("{start}-{end}")
        });
        i += 1;
    }
    parts.join(",")
}

/// Render one policy group's state line. `prefix` carries the `cpuN-M: ` label when the box has
/// more than one group, and the base-clock suffix rides every line so a cut single line still
/// carries it.
fn state_line(state: &CpuState, cur: Option<(&'static str, u64)>, prefix: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(g) = &state.governor {
        parts.push(format!("governor={g}"));
    }
    if let Some(e) = &state.epp {
        parts.push(format!("epp={e}"));
    }
    if let Some(b) = &state.boost {
        let word = match b.as_str() {
            "1" => "on",
            "0" => "off",
            other => other,
        };
        parts.push(format!("boost={word}"));
    }
    if let (Some(min), Some(max)) = (state.min_khz, state.max_khz) {
        parts.push(format!("clamp={}-{}GHz", ghz(min), ghz(max)));
    }
    if let Some((label, khz)) = cur {
        parts.push(format!("{label}={}GHz", ghz(khz)));
    }
    if let Some(b) = freq::base_clock() {
        parts.push(format!("base={}GHz({})", ghz(b.khz), b.source));
    }
    if parts.is_empty() {
        parts.push("cpufreq not exposed".to_string());
    }
    format!("{prefix}{}", parts.join(" "))
}

/// The current frequency for a group's first CPU: the delivered average where the driver
/// reports one (`avg=`), else the driver's `scaling_cur_freq` (`cur=`, which some drivers
/// derive and some report as the requested value, so the label says which file it is).
fn cur_freq(cpu: usize) -> Option<(&'static str, u64)> {
    if let Some(khz) = freq::read_khz(cpu, "cpuinfo_avg_freq") {
        return Some(("avg", khz));
    }
    freq::read_khz(cpu, "scaling_cur_freq").map(|khz| ("cur", khz))
}

/// The box's state lines, one per policy group: CPUs sharing an identical (governor, EPP,
/// boost, clamp) tuple collapse into one line, and a single-group box (the common case) prints
/// exactly one line with no CPU prefix, the shape a prompt or status bar wants.
fn state_lines() -> Vec<String> {
    let mut groups: std::collections::BTreeMap<CpuState, Vec<usize>> = Default::default();
    for cpu in freq::cpus() {
        groups.entry(cpu_state(cpu)).or_default().push(cpu);
    }
    if groups.is_empty() {
        return vec!["cpufreq not exposed".to_string()];
    }
    let one = groups.len() == 1;
    groups
        .iter()
        .map(|(state, cpus)| {
            let prefix = if one {
                String::new()
            } else {
                format!("cpu{}: ", cpu_list(cpus))
            };
            state_line(state, cpus.first().and_then(|&c| cur_freq(c)), &prefix)
        })
        .collect()
}

/// Print the current state as a `[freq]` config section, ready to paste into a `toml` fence:
/// the answer to pin-freq's refusal when no steady state is declared.
fn print_as_config() -> i32 {
    let cpus = freq::cpus();
    let Some(&first) = cpus.first() else {
        eprintln!("error: no CPUs under /sys/devices/system/cpu");
        return 1;
    };
    let states: Vec<CpuState> = cpus.iter().map(|&c| cpu_state(c)).collect();
    let state = &states[0];
    let Some(governor) = &state.governor else {
        eprintln!("error: cpufreq exposes no scaling_governor: nothing to declare");
        return 1;
    };
    println!("[freq]");
    if states.iter().any(|s| s != state) {
        println!("# cpu policy groups disagree: these are cpu{first}'s values");
    }
    println!("governor = {governor:?}");
    if let Some(epp) = &state.epp {
        println!("epp = {epp:?}");
    }
    match state.boost.as_deref() {
        Some("1") => println!("boost = true"),
        Some("0") => println!("boost = false"),
        Some(other) => println!("# boost token {other:?} unrecognized: declare boost by hand"),
        None => {}
    }
    if let (Some(min), Some(max)) = (
        freq::read_khz(first, "cpuinfo_min_freq"),
        freq::read_khz(first, "cpuinfo_max_freq"),
    ) {
        println!(
            "# min_mhz / max_mhz omitted: the hardware range ({}-{} MHz)",
            min / 1000,
            max / 1000
        );
    }
    match freq::base_clock() {
        Some(b) => println!(
            "# pin_mhz omitted: the base clock ({} MHz from {})",
            b.khz / 1000,
            b.source
        ),
        None => println!("# pin_mhz: no base clock discoverable, declare one before pinning"),
    }
    0
}

// ---------------------------------------------------------------------------
// The command words.
// ---------------------------------------------------------------------------

/// The `read-freq` command: print the clock state (no root needed), or with `as_config` the
/// paste-ready `[freq]` section.
pub fn cmd_read_freq(as_config: bool) -> i32 {
    if as_config {
        return print_as_config();
    }
    for line in state_lines() {
        println!("{line}");
    }
    0
}

/// The `pin-freq` command: hold the clock at the resolved target until `restore-freq`. Refuses
/// without a declared steady state, because a pin with no declared way home is how a box gets
/// stranded.
pub fn cmd_pin_freq(cfg: Option<&FreqConfig>, cli_mhz: Option<u64>) -> i32 {
    let plan = read_caps().and_then(|caps| {
        resolve_steady(cfg, &caps)?;
        let (khz, source) = resolve_pin(cfg, cli_mhz)?;
        Ok((pin_plan(khz, &caps)?, khz, source))
    });
    let (plan, khz, source) = match plan {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: pin-freq: {e}");
            return 2;
        }
    };
    if let Err(e) = apply(&plan) {
        eprintln!("error: pin-freq: {e}");
        return 1;
    }
    println!(
        "pinned at {} MHz ({source}): min = max, boost off",
        khz / 1000
    );
    for line in state_lines() {
        println!("{line}");
    }
    println!("restore with: iiac-perf restore-freq");
    0
}

/// The `restore-freq` command: converge the box to the declared steady state, from any starting
/// point, an unclean death's residue included.
pub fn cmd_restore_freq(cfg: Option<&FreqConfig>) -> i32 {
    let plan = read_caps().and_then(|caps| {
        let steady = resolve_steady(cfg, &caps)?;
        Ok(restore_plan(&steady, &caps))
    });
    let plan = match plan {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: restore-freq: {e}");
            return 2;
        }
    };
    if let Err(e) = apply(&plan) {
        eprintln!("error: restore-freq: {e}");
        return 1;
    }
    println!("restored the declared [freq] steady state");
    for line in state_lines() {
        println!("{line}");
    }
    0
}

// ---------------------------------------------------------------------------
// The suggest-freq command.
// ---------------------------------------------------------------------------

/// Continuous-driver descent step: 100 MHz per candidate.
const SUGGEST_STEP_KHZ: u64 = 100_000;

/// Cap on candidates tried: 12 covers 1.1 GHz below the ceiling at the step above, and a box
/// holding nothing in that span wants a human look, not a longer descent.
const SUGGEST_MAX_CANDIDATES: usize = 12;

/// Sampler cadence while a candidate runs (ms), the batch-seam scale.
const SUGGEST_SAMPLE_MS: u64 = 50;

/// The candidate ladder, descending from max-with-boost-off: a discrete driver's available
/// list, or [`SUGGEST_STEP_KHZ`] steps down from the base clock (the boost-off ceiling) on a
/// continuous-range driver.
fn suggest_candidates(caps: &BoxCaps, base_khz: Option<u64>) -> Result<Vec<u64>, String> {
    let Some(c0) = caps.cpus.first() else {
        return Err("no CPUs under /sys/devices/system/cpu".to_string());
    };
    if let Some(avail) = &c0.avail_khz {
        let mut v: Vec<u64> = avail.clone();
        v.sort_unstable();
        v.dedup();
        v.reverse();
        v.truncate(SUGGEST_MAX_CANDIDATES);
        return Ok(v);
    }
    let Some(base) = base_khz else {
        return Err(
            "no base clock discoverable, so the boost-off ceiling is unknown on this \
             continuous-range driver"
                .to_string(),
        );
    };
    let top = base.min(c0.hw_max_khz);
    let mut v = Vec::new();
    let mut k = top;
    while k >= c0.hw_min_khz && v.len() < SUGGEST_MAX_CANDIDATES {
        v.push(k);
        match k.checked_sub(SUGGEST_STEP_KHZ) {
            Some(n) => k = n,
            None => break,
        }
    }
    Ok(v)
}

/// One candidate's sampled-clock verdict: the delivered series against the pin target.
#[derive(Debug, PartialEq, Eq)]
struct Hold {
    /// Samples collected while the bench ran.
    n: usize,
    /// Slowest delivered sample (kHz).
    min_khz: u64,
    /// Median delivered sample (kHz).
    med_khz: u64,
    /// Fastest delivered sample (kHz).
    max_khz: u64,
    /// Whether the candidate held: series stable within
    /// [`freq::FREQ_STABLE_TOL`] and its median on the target.
    held: bool,
}

/// Assess a sampled series against the pin target. `None` when nothing was sampled, which is
/// never a hold: an unverifiable claim must fail rather than grade well.
fn assess(series: &[u64], target_khz: u64) -> Option<Hold> {
    if series.is_empty() {
        return None;
    }
    let mut v = series.to_vec();
    v.sort_unstable();
    let (min_khz, med_khz, max_khz) = (v[0], v[v.len() / 2], v[v.len() - 1]);
    let stable = (max_khz - min_khz) as f64 / max_khz as f64 <= freq::FREQ_STABLE_TOL;
    let on_target =
        med_khz.abs_diff(target_khz) as f64 / target_khz as f64 <= freq::FREQ_STABLE_TOL;
    Some(Hold {
        n: series.len(),
        min_khz,
        med_khz,
        max_khz,
        held: stable && on_target,
    })
}

/// Sample `cpu`'s delivered clock every [`SUGGEST_SAMPLE_MS`] while `f` runs, returning the
/// series.
fn sample_while<F: FnOnce()>(cpu: usize, f: F) -> Vec<u64> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    let stop = Arc::new(AtomicBool::new(false));
    let sampler_stop = Arc::clone(&stop);
    let sampler = std::thread::spawn(move || {
        let mut series = Vec::new();
        while !sampler_stop.load(Ordering::Relaxed) {
            if let Some((_, khz)) = cur_freq(cpu) {
                series.push(khz);
            }
            std::thread::sleep(std::time::Duration::from_millis(SUGGEST_SAMPLE_MS));
        }
        series
    });
    f();
    stop.store(true, Ordering::Relaxed);
    sampler.join().unwrap_or_default() // OK: a panicked sampler yields an empty series, which assess() treats as not held
}

/// Restore-on-drop for the descent: the declared steady state, whatever candidate was live.
struct SuggestGuard {
    /// The restore plan `Drop` executes.
    restore: Plan,
}

impl Drop for SuggestGuard {
    /// Converge back to the declared steady state, pointing at `restore-freq` on failure.
    fn drop(&mut self) {
        if let Err(e) = apply(&self.restore) {
            eprintln!("warning: freq restore failed: {e}. Run `iiac-perf restore-freq`.");
        }
    }
}

/// The `suggest-freq <bench>` command: find the highest boost-off frequency the box *holds*
/// under the real bench's load and schedule, and end with the config line to paste.
///
/// - the load is the bench itself, driven through the normal run path with the session's full
///   run configuration, because the held frequency is thermal and duty-cycle dependent: 2t on
///   SMT siblings heats differently than 1t on one CPU
/// - each candidate pins (min = max, boost off), runs the bench once, and a sampler thread
///   reads the delivered clock every [`SUGGEST_SAMPLE_MS`]; the candidate holds when the
///   series is stable and its median sits on the target ([`assess`])
/// - the declared steady state restores on every exit path a pinned run restores on: drop,
///   panic, SIGINT/SIGTERM
pub fn cmd_suggest_freq(
    cfg: Option<&FreqConfig>,
    bench_name: &str,
    run: crate::benches::RunFn,
    run_cfg: &crate::harness::RunCfg,
) -> i32 {
    let prep = read_caps().and_then(|caps| {
        let steady = resolve_steady(cfg, &caps)?;
        let restore = restore_plan(&steady, &caps);
        let candidates = suggest_candidates(&caps, freq::base_clock().map(|b| b.khz))?;
        Ok((caps, restore, candidates))
    });
    let (caps, restore, candidates) = match prep {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: suggest-freq: {e}");
            return 2;
        }
    };
    let sample_cpu = run_cfg.pin_cpus.first().copied().unwrap_or(0); // OK: unpinned samples CPU 0, every clamp is box-wide
    if cur_freq(sample_cpu).is_none() {
        eprintln!(
            "error: suggest-freq: cpu{sample_cpu} exposes no readable frequency \
             (cpuinfo_avg_freq or scaling_cur_freq): a hold cannot be verified"
        );
        return 2;
    }
    let Some(&top) = candidates.first() else {
        eprintln!("error: suggest-freq: the candidate ladder is empty");
        return 2;
    };
    arm_signal_restore(&restore);
    let _guard = SuggestGuard {
        restore: restore.clone(),
    };
    println!(
        "suggest-freq: {bench_name} for {:.1} s per candidate, descending from {} MHz \
         (boost off), verifying on cpu{sample_cpu}\n",
        run_cfg.target_seconds,
        top / 1000,
    );
    for &khz in &candidates {
        let plan = match pin_plan(khz, &caps) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: suggest-freq: {e}");
                return 1;
            }
        };
        if let Err(e) = apply(&plan) {
            eprintln!("error: suggest-freq: {e}");
            return 1;
        }
        println!("candidate {} MHz:", khz / 1000);
        let series = sample_while(sample_cpu, || run(run_cfg));
        match assess(&series, khz) {
            None => println!(
                "candidate {} MHz: no clock samples, not held (unverifiable is a failure)",
                khz / 1000
            ),
            Some(h) if h.held => {
                println!(
                    "candidate {} MHz: held. Delivered {}-{} GHz, median {}, {} samples",
                    khz / 1000,
                    ghz(h.min_khz),
                    ghz(h.max_khz),
                    ghz(h.med_khz),
                    h.n
                );
                println!(
                    "\nsuggestion: the highest held pin is {} MHz, measured under \
                     `{bench_name}` at {:.1} s. A different bench, duration, or pin layout \
                     can hold a different clock. Paste into the config's [freq] section:",
                    khz / 1000,
                    run_cfg.target_seconds,
                );
                println!("pin_mhz = {}", khz / 1000);
                return 0;
            }
            Some(h) => println!(
                "candidate {} MHz: not held. Delivered {}-{} GHz, median {}, {} samples",
                khz / 1000,
                ghz(h.min_khz),
                ghz(h.max_khz),
                ghz(h.med_khz),
                h.n
            ),
        }
        println!();
    }
    eprintln!(
        "suggest-freq: no candidate held within the {}-step descent. The box is throttling \
         below its ladder: check cooling, or pin by hand from read-freq's numbers",
        candidates.len()
    );
    1
}

// ---------------------------------------------------------------------------
// The --pin-freq run guard.
// ---------------------------------------------------------------------------

/// The restore plan pre-rendered for the signal handler: C paths and byte tokens, so the
/// handler allocates nothing. Set once when a [`RunPin`] engages.
static SIGNAL_PLAN: OnceLock<Vec<(CString, Vec<u8>)>> = OnceLock::new();

/// SIGINT/SIGTERM handler while a run pin is live: replay the restore plan with raw
/// open/write/close (the async-signal-safe subset, which is why `std::fs` is not used here),
/// then exit with the conventional 128+signal status.
extern "C" fn restore_on_signal(sig: libc::c_int) {
    if let Some(plan) = SIGNAL_PLAN.get() {
        for (path, token) in plan {
            unsafe {
                let fd = libc::open(path.as_ptr(), libc::O_WRONLY);
                if fd >= 0 {
                    libc::write(fd, token.as_ptr().cast(), token.len());
                    libc::close(fd);
                }
            }
        }
    }
    unsafe { libc::_exit(128 + sig) };
}

/// Arm [`restore_on_signal`] with the pre-rendered `plan` for SIGINT and SIGTERM.
fn arm_signal_restore(plan: &Plan) {
    let rendered: Vec<(CString, Vec<u8>)> = plan
        .iter()
        .filter_map(|(path, token)| {
            // Sysfs paths built here never contain a NUL, so this cannot skip in practice.
            let c = CString::new(path.as_str()).ok()?;
            Some((c, token.clone().into_bytes()))
        })
        .collect();
    if SIGNAL_PLAN.set(rendered).is_err() {
        return;
    }
    unsafe {
        let mut sa: libc::sigaction = std::mem::zeroed();
        sa.sa_sigaction = restore_on_signal as *const () as usize;
        libc::sigemptyset(&mut sa.sa_mask);
        libc::sigaction(libc::SIGINT, &sa, std::ptr::null_mut());
        libc::sigaction(libc::SIGTERM, &sa, std::ptr::null_mut());
    }
}

/// A live run pin: created before the warmup so the whole run executes at the pinned clock,
/// held to the end of `main`. Restores the declared steady state on drop (normal exit and
/// panic) and via [`restore_on_signal`] on SIGINT/SIGTERM.
pub struct RunPin {
    /// The pinned frequency (kHz), for the Setup block's row.
    pub khz: u64,
    /// Where the target came from, for the same row.
    pub source: String,
    /// The restore plan `Drop` executes.
    restore: Plan,
}

impl RunPin {
    /// Validate, pin, and arm the restores. On a pin that fails partway the restore plan runs
    /// best-effort before the error returns, so a half-applied pin does not outlive the error
    /// message.
    pub fn engage(cfg: Option<&FreqConfig>, cli_mhz: Option<u64>) -> Result<RunPin, String> {
        let caps = read_caps()?;
        let steady = resolve_steady(cfg, &caps)?;
        let restore = restore_plan(&steady, &caps);
        let (khz, source) = resolve_pin(cfg, cli_mhz)?;
        let pin = pin_plan(khz, &caps)?;
        if let Err(e) = apply(&pin) {
            apply(&restore).ok();
            return Err(e);
        }
        arm_signal_restore(&restore);
        Ok(RunPin {
            khz,
            source,
            restore,
        })
    }
}

impl Drop for RunPin {
    /// Restore the declared steady state, pointing at `restore-freq` if any write fails.
    fn drop(&mut self) {
        if let Err(e) = apply(&self.restore) {
            eprintln!("warning: freq restore failed: {e}. Run `iiac-perf restore-freq`.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A two-CPU box shaped like the 3900X: EPP, global boost, continuous range.
    fn amd_caps() -> BoxCaps {
        BoxCaps {
            cpus: vec![
                CpuCaps {
                    cpu: 0,
                    hw_min_khz: 550_000,
                    hw_max_khz: 3_800_000,
                    avail_khz: None,
                },
                CpuCaps {
                    cpu: 1,
                    hw_min_khz: 550_000,
                    hw_max_khz: 3_800_000,
                    avail_khz: None,
                },
            ],
            has_epp: true,
            governors: Some(vec!["performance".into(), "powersave".into()]),
            epp_prefs: Some(vec!["performance".into(), "balance_performance".into()]),
            boost: BoostKnob::Global,
        }
    }

    /// A one-CPU box shaped like the Pi: no EPP, no boost, discrete frequencies.
    fn pi_caps() -> BoxCaps {
        BoxCaps {
            cpus: vec![CpuCaps {
                cpu: 0,
                hw_min_khz: 1_500_000,
                hw_max_khz: 2_400_000,
                avail_khz: Some(vec![1_500_000, 2_400_000]),
            }],
            has_epp: false,
            governors: Some(vec!["ondemand".into(), "performance".into()]),
            epp_prefs: None,
            boost: BoostKnob::None,
        }
    }

    fn full_cfg() -> FreqConfig {
        FreqConfig {
            governor: "powersave".into(),
            epp: Some("balance_performance".into()),
            boost: Some(true),
            min_mhz: None,
            max_mhz: None,
            pin_mhz: None,
        }
    }

    #[test]
    fn steady_refuses_with_no_config() {
        let err = resolve_steady(None, &amd_caps()).unwrap_err();
        assert!(err.contains("no [freq] steady state"), "got: {err}");
        assert!(err.contains("read-freq --as-config"), "got: {err}");
    }

    #[test]
    fn steady_requires_the_knobs_the_box_has() {
        let mut cfg = full_cfg();
        cfg.epp = None;
        let err = resolve_steady(Some(&cfg), &amd_caps()).unwrap_err();
        assert!(err.contains("declare freq.epp"), "got: {err}");
        assert!(err.contains("balance_performance"), "got: {err}");

        let mut cfg = full_cfg();
        cfg.boost = None;
        let err = resolve_steady(Some(&cfg), &amd_caps()).unwrap_err();
        assert!(err.contains("declare freq.boost"), "got: {err}");
    }

    #[test]
    fn steady_rejects_knobs_the_box_lacks() {
        let mut cfg = full_cfg();
        cfg.governor = "ondemand".into();
        let err = resolve_steady(Some(&cfg), &pi_caps()).unwrap_err();
        assert!(err.contains("freq.epp is declared"), "got: {err}");

        let mut cfg = full_cfg();
        cfg.epp = None;
        cfg.governor = "ondemand".into();
        let err = resolve_steady(Some(&cfg), &pi_caps()).unwrap_err();
        assert!(err.contains("freq.boost is declared"), "got: {err}");
    }

    #[test]
    fn steady_validates_tokens_against_the_box() {
        let mut cfg = full_cfg();
        cfg.governor = "schedutil".into();
        let err = resolve_steady(Some(&cfg), &amd_caps()).unwrap_err();
        assert!(err.contains("schedutil"), "got: {err}");
        assert!(err.contains("performance, powersave"), "got: {err}");

        let mut cfg = full_cfg();
        cfg.epp = Some("power".into());
        let err = resolve_steady(Some(&cfg), &amd_caps()).unwrap_err();
        assert!(err.contains("freq.epp"), "got: {err}");
    }

    #[test]
    fn steady_validates_clamps_against_the_range() {
        let mut cfg = full_cfg();
        cfg.max_mhz = Some(5000);
        let err = resolve_steady(Some(&cfg), &amd_caps()).unwrap_err();
        assert!(err.contains("outside cpu0's range"), "got: {err}");
        assert!(err.contains("550-3800"), "got: {err}");
    }

    #[test]
    fn restore_plan_is_floor_first_and_complete() {
        let steady = resolve_steady(Some(&full_cfg()), &amd_caps()).unwrap();
        let plan = restore_plan(&steady, &amd_caps());
        let writes: Vec<(&str, &str)> =
            plan.iter().map(|(p, t)| (p.as_str(), t.as_str())).collect();
        // Governor and EPP lead, boost follows, then per-CPU staged clamps.
        assert_eq!(
            writes,
            vec![
                (
                    "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor",
                    "powersave"
                ),
                (
                    "/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_preference",
                    "balance_performance"
                ),
                (
                    "/sys/devices/system/cpu/cpu1/cpufreq/scaling_governor",
                    "powersave"
                ),
                (
                    "/sys/devices/system/cpu/cpu1/cpufreq/energy_performance_preference",
                    "balance_performance"
                ),
                ("/sys/devices/system/cpu/cpufreq/boost", "1"),
                (
                    "/sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq",
                    "550000"
                ),
                (
                    "/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq",
                    "3800000"
                ),
                (
                    "/sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq",
                    "550000"
                ),
                (
                    "/sys/devices/system/cpu/cpu1/cpufreq/scaling_min_freq",
                    "550000"
                ),
                (
                    "/sys/devices/system/cpu/cpu1/cpufreq/scaling_max_freq",
                    "3800000"
                ),
                (
                    "/sys/devices/system/cpu/cpu1/cpufreq/scaling_min_freq",
                    "550000"
                ),
            ]
        );
    }

    #[test]
    fn pin_plan_pins_min_max_and_boost_off() {
        let plan = pin_plan(3_800_000, &amd_caps()).unwrap();
        assert_eq!(
            plan[0],
            (
                "/sys/devices/system/cpu/cpufreq/boost".to_string(),
                "0".to_string()
            )
        );
        // Per CPU: staged floor, then max, then min, all at the pin.
        assert_eq!(plan[1].1, "550000");
        assert_eq!(plan[2].1, "3800000");
        assert_eq!(plan[3].1, "3800000");
        assert_eq!(plan.len(), 1 + 2 * 3);
    }

    #[test]
    fn pin_plan_validates_range_and_discrete_lists() {
        let err = pin_plan(5_000_000, &amd_caps()).unwrap_err();
        assert!(
            err.contains("outside cpu0's range 550-3800 MHz"),
            "got: {err}"
        );

        let err = pin_plan(2_000_000, &pi_caps()).unwrap_err();
        assert!(err.contains("discrete frequencies"), "got: {err}");
        assert!(err.contains("1500, 2400"), "got: {err}");

        assert!(pin_plan(2_400_000, &pi_caps()).is_ok());
    }

    #[test]
    fn resolve_pin_precedence_is_cli_config_base() {
        let mut cfg = full_cfg();
        cfg.pin_mhz = Some(3600);
        let (khz, source) = resolve_pin(Some(&cfg), Some(3000)).unwrap();
        assert_eq!(khz, 3_000_000);
        assert!(source.contains("command line"));
        let (khz, source) = resolve_pin(Some(&cfg), None).unwrap();
        assert_eq!(khz, 3_600_000);
        assert!(source.contains("pin_mhz"));
        assert!(resolve_pin(None, Some(0)).is_err());
    }

    #[test]
    fn suggest_candidates_descend_the_discrete_list() {
        let v = suggest_candidates(&pi_caps(), None).unwrap();
        assert_eq!(v, vec![2_400_000, 1_500_000]);
    }

    #[test]
    fn suggest_candidates_step_down_from_the_base_clock() {
        let v = suggest_candidates(&amd_caps(), Some(3_800_000)).unwrap();
        assert_eq!(v[0], 3_800_000);
        assert_eq!(v[1], 3_700_000);
        assert_eq!(v.len(), SUGGEST_MAX_CANDIDATES);
        assert_eq!(*v.last().unwrap(), 2_700_000);
    }

    #[test]
    fn suggest_candidates_need_a_base_on_continuous_drivers() {
        assert!(suggest_candidates(&amd_caps(), None).is_err());
    }

    #[test]
    fn assess_holds_only_a_stable_on_target_series() {
        // Stable on target: held.
        let h = assess(&[3_790_000, 3_800_000, 3_800_000], 3_800_000).unwrap();
        assert!(h.held);
        // Stable but sagged below the target: not held.
        let h = assess(&[3_500_000, 3_500_000, 3_510_000], 3_800_000).unwrap();
        assert!(!h.held);
        // Median on target but unstable across the run: not held.
        let h = assess(&[3_400_000, 3_800_000, 3_800_000], 3_800_000).unwrap();
        assert!(!h.held);
        // Nothing sampled: unverifiable, never held.
        assert!(assess(&[], 3_800_000).is_none());
    }

    #[test]
    fn cpu_list_compresses_runs() {
        assert_eq!(cpu_list(&[0, 1, 2, 3, 8]), "0-3,8");
        assert_eq!(cpu_list(&[5]), "5");
        assert_eq!(cpu_list(&[0, 2, 4]), "0,2,4");
    }

    #[test]
    fn ghz_formats_two_decimals() {
        assert_eq!(ghz(3_800_000), "3.80");
        assert_eq!(ghz(550_000), "0.55");
    }

    #[test]
    fn state_line_translates_boost_and_omits_absent_fields() {
        let state = CpuState {
            governor: Some("powersave".into()),
            epp: None,
            boost: Some("1".into()),
            min_khz: Some(550_000),
            max_khz: Some(3_800_000),
        };
        let line = state_line(&state, Some(("avg", 3_790_000)), "");
        assert!(line.starts_with("governor=powersave boost=on clamp=0.55-3.80GHz avg=3.79GHz"));
        assert!(!line.contains("epp="));
    }
}
