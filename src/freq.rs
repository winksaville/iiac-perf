//! cpufreq sysfs reads, in two jobs: what the CPU is doing *now*, and what it has been *told to
//! do*. Never writes: reading is diagnosis, while setting a governor needs root and is global
//! and persistent, so a benchmark that mutated one would outlive itself.
//!
//! - **Delivered frequency** ([`avg_freq`], [`max_freq`]) feeds the warm loop's clock gate:
//!   `cpuinfo_avg_freq` where the driver provides it (amd-pstate), nothing otherwise.
//!   `scaling_cur_freq` is deliberately not a fallback: some drivers report the *requested*
//!   rather than the delivered frequency, and a gate fed requested values would certify a dwell
//!   as the top. Read the honest file where it exists and fall back to timing-only everywhere
//!   else.
//! - **Policy** ([`policy`], [`Policy`], [`PolicyField`]) is the standing configuration: which
//!   driver, governor, EPP and boost setting the box is running under. It is provenance, not
//!   measurement, and it exists because no report before 0.25.0 recorded it: the same bench
//!   measured 8.9% apart under `powersave` and `performance` on a 3900X, so any A/B spanning a
//!   policy change read an environmental delta as a code change.

/// One delivered-frequency sample: which logical CPU it was read on, and the kHz value.
#[derive(Debug, Clone, Copy)]
pub struct FreqSample {
    /// Logical CPU the calling thread was on at the read.
    pub cpu: usize,
    /// Delivered frequency (kHz) as the driver reports it.
    pub khz: u64,
}

/// Sample the calling thread's current CPU and its delivered frequency; `None` when the driver
/// does not expose `cpuinfo_avg_freq` (see the module doc) or the read fails.
pub fn avg_freq() -> Option<FreqSample> {
    let cpu = current_cpu()?;
    let khz = read_khz(cpu, "cpuinfo_avg_freq")?;
    Some(FreqSample { cpu, khz })
}

/// The hardware maximum (kHz) for `cpu`, for the reported ratio; `None` when unavailable.
pub fn max_freq(cpu: usize) -> Option<u64> {
    read_khz(cpu, "cpuinfo_max_freq")
}

/// Relative band the delivered clock must hold across a window's samples to count as stable
/// under load ([`clock_stable`]).
///
/// - Stability, never a fraction of `cpuinfo_max_freq`: a max-fraction threshold would need
///   tuning per box (96.1% sustained on the 3900X against 99.7% on the 7600x) and a
///   thermally-limited laptop plateaus lower still while that plateau is its honest clock.
/// - One percent, the same scale as the timing signals' A cutoffs: the measured dwell-to-top
///   step is +12.4%, an order of magnitude above the band.
const FREQ_STABLE_TOL: f64 = 0.01;

/// Whether the delivered clock held still across a window's samples: the gate the warm exit
/// and the settle scan share, so "settled" means the same thing at both sites.
///
/// - Anything short of clean same-CPU readings falls back to timing-only (`true`): a missing
///   `cpuinfo_avg_freq` (the read is amd-pstate-specific) or a mid-window migration of an
///   unpinned main means there is no honest per-core series to gate on.
pub(crate) fn clock_stable(samples: &[Option<FreqSample>]) -> bool {
    let mut min = u64::MAX;
    let mut max = 0u64;
    let mut cpu: Option<usize> = None;
    for s in samples {
        let Some(f) = s else { return true };
        match cpu {
            None => cpu = Some(f.cpu),
            Some(c) if c != f.cpu => return true,
            Some(_) => {}
        }
        min = min.min(f.khz);
        max = max.max(f.khz);
    }
    if max == 0 {
        return true;
    }
    (max - min) as f64 / max as f64 <= FREQ_STABLE_TOL
}

/// The calling thread's current logical CPU (`sched_getcpu`), `None` on syscall failure.
fn current_cpu() -> Option<usize> {
    let cpu = unsafe { libc::sched_getcpu() };
    usize::try_from(cpu).ok()
}

/// One cpufreq sysfs value (kHz) for `cpu`, `None` when the file is absent or unparsable.
fn read_khz(cpu: usize, name: &str) -> Option<u64> {
    let path = format!("/sys/devices/system/cpu/cpu{cpu}/cpufreq/{name}");
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// One policy value, plus whether every CPU that exposes it agrees.
///
/// Paired with the `Option` in [`Policy`], this encodes three states a bare string cannot:
///
/// - **absent** (`None`): the knob does not exist on this box. rpi5-20cd exposes no
///   `energy_performance_preference` at all, and a printed default there would describe a
///   policy the machine does not have.
/// - **present and uniform**: the box has one policy, the ordinary case.
/// - **present and split**: cpufreq exposes these files per *policy group*, so nothing stops
///   `cpu0` reading `powersave` while `cpu8` reads `performance`. One CPU's token printed as
///   the box's policy would be a quiet lie in the block that exists to be provenance.
///
/// The token is kept **raw**: never an enum, never prettied. A governor some future kernel adds
/// should reach a record under its real name rather than as `Unknown`, and a record should say
/// what the machine said. Prettying belongs at the print site, which is all `boost_word` does
/// when it turns `1` into `enabled`.
///
/// Three consumers, one shape, which is why the flag travels with the value rather than being
/// resolved at the read:
///
/// - the `Setup:` block prints it, marking a split box `(mixed across CPUs)` so the reader is
///   never handed one CPU's token as the machine's answer;
/// - the per-run record serializes both parts, so a policy that was split when the numbers were
///   taken can still be seen years later;
/// - `qualify-environment` reads it as a fitness precondition, where a split box is a finding in
///   its own right and not a value to average away.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PolicyField {
    /// The first CPU's raw sysfs token.
    pub value: String,
    /// True when every CPU exposing this file reports the same token.
    pub uniform: bool,
}

/// The box's cpufreq policy: the knobs that decide how the CPU picks a frequency.
///
/// Every field is an `Option` because absence is a real answer, not a default to invent:
/// rpi5-20cd's cpufreq exposes no `energy_performance_preference` at all, and a printed
/// `performance` there would describe a policy the box does not have.
///
/// - `boost` falls back to the global `cpufreq/boost` when no per-CPU file exists. Intel's
///   inverted `intel_pstate/no_turbo` is deliberately not read: no box in this project's fleet
///   runs it, and a silent sign error is worse than an absent row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// `scaling_driver`, e.g. `amd-pstate-epp`.
    pub driver: Option<PolicyField>,
    /// `scaling_governor`, e.g. `powersave`.
    pub governor: Option<PolicyField>,
    /// `energy_performance_preference`, absent outside EPP-capable drivers.
    pub epp: Option<PolicyField>,
    /// `boost` as its raw `1` / `0` token.
    pub boost: Option<PolicyField>,
    /// `scaling_min_freq` (kHz) as its raw token: the governor's lower clamp. With the upper
    /// clamp and boost it is what makes a pinned run verifiable from its record: a pin sets
    /// min = max, and a record carrying the clamp shows it.
    pub scaling_min_freq: Option<PolicyField>,
    /// `scaling_max_freq` (kHz) as its raw token: the governor's upper clamp.
    pub scaling_max_freq: Option<PolicyField>,
}

/// Read the box's cpufreq policy; each field independently `None` when its file is absent.
pub fn policy() -> Policy {
    let cpus = cpus();
    Policy {
        driver: read_field(&cpus, "scaling_driver"),
        governor: read_field(&cpus, "scaling_governor"),
        epp: read_field(&cpus, "energy_performance_preference"),
        boost: read_boost(&cpus),
        scaling_min_freq: read_field(&cpus, "scaling_min_freq"),
        scaling_max_freq: read_field(&cpus, "scaling_max_freq"),
    }
}

/// Logical CPUs sysfs knows about, ascending; empty when the directory is unreadable.
fn cpus() -> Vec<usize> {
    let Ok(dir) = std::fs::read_dir("/sys/devices/system/cpu") else {
        return Vec::new();
    };
    let mut cpus: Vec<usize> = dir
        .flatten()
        .filter_map(|e| {
            let name = e.file_name();
            let name = name.to_str()?;
            name.strip_prefix("cpu")?.parse().ok()
        })
        .collect();
    cpus.sort_unstable();
    cpus
}

/// One policy field across every CPU: the first reading plus whether the rest agree.
fn read_field(cpus: &[usize], name: &str) -> Option<PolicyField> {
    let mut values = cpus.iter().filter_map(|&cpu| read_token(cpu, name));
    let value = values.next()?;
    let uniform = values.all(|v| v == value);
    Some(PolicyField { value, uniform })
}

/// `boost` per CPU, falling back to the single global knob (uniform by construction).
fn read_boost(cpus: &[usize]) -> Option<PolicyField> {
    if let Some(field) = read_field(cpus, "boost") {
        return Some(field);
    }
    let raw = std::fs::read_to_string("/sys/devices/system/cpu/cpufreq/boost").ok()?;
    Some(PolicyField {
        value: raw.trim().to_string(),
        uniform: true,
    })
}

/// One cpufreq sysfs value for `cpu` as a trimmed token, `None` when the file is absent.
fn read_token(cpu: usize, name: &str) -> Option<String> {
    let path = format!("/sys/devices/system/cpu/cpu{cpu}/cpufreq/{name}");
    Some(std::fs::read_to_string(path).ok()?.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_cpu_resolves() {
        assert!(current_cpu().is_some());
    }

    #[test]
    fn absent_file_reads_none() {
        assert_eq!(read_khz(usize::MAX, "cpuinfo_avg_freq"), None);
    }
}
