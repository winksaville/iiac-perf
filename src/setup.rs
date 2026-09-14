//! The `setup` command word: make a host ready for iiac-perf.
//!
//! A host was ready only after hand work, its `[freq]` steady state written into the XDG config
//! with the clamp limits a restore needs, and the limits are what a hand forgets (the 7600x's
//! declaration omitted them, and a restore there fell to 427 MHz). `setup` writes the declaration
//! from the live state instead.
//!
//! - **Print by default, write with `--apply`**: the plain command shows exactly what `--apply`
//!   would write and where, so the file is reviewed before it exists.
//! - **Never overwrite**: a missing file is created, a file without `[freq]` gains one at its
//!   end, and a file that already declares `[freq]` is left alone and checked, since it may hold
//!   profiles and knobs nobody wants regenerated.
//! - **Checked before written**: the new text must parse and pass the same steady-state checks
//!   every pin and restore applies, so a pinned clamp at setup time refuses rather than writing a
//!   pin as the steady state.
//! - **An existing declaration is checked against the live state too**: one can fit the hardware
//!   range and still be another host's, as the 7600x's once was, so `setup` names every declared
//!   value the host does not run at.
//! - **Run as the user**: the config belongs under the user's home, and under sudo `$HOME` may
//!   be root's.
//! - **Permissions by ownership, once**: a udev rule hands the user the cpufreq files a pin or
//!   restore writes and `/dev/cpu_dma_latency`, so those commands run without sudo. `--apply`
//!   calls sudo once to install the rule and take ownership now, and `--uninstall --apply` removes
//!   the rule and gives the files back to root. Ownership rather than a POSIX ACL, since we think
//!   sysfs does not reliably support ACLs, while chown on its files is what udev rules already do.

use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::config;
use crate::freqctl;

/// What `setup` does with the XDG config file.
#[derive(Debug, PartialEq)]
enum ConfigPlan {
    /// No file exists: create it with this text.
    Create { path: PathBuf, text: String },
    /// The file exists without `[freq]`: append this text to it, `whole` being the file after.
    Append {
        path: PathBuf,
        text: String,
        whole: String,
    },
    /// The file already declares `[freq]`: leave it, and check what it declares.
    Declared {
        path: PathBuf,
        config: Box<config::Config>,
    },
}

/// Where the permissions rule lives.
const RULE_PATH: &str = "/etc/udev/rules.d/70-iiac-perf.rules";

/// The wake-latency clamp device the permissions also hand over, for the pin-idle knob.
const DMA_LATENCY: &str = "/dev/cpu_dma_latency";

/// The `setup` command: print the plan, and carry it out with `apply`. `uninstall` plans the
/// permissions' removal instead and leaves the config alone. Exit 0 when the host is ready (or
/// would be, printing), 1 when a step failed or a declaration does not pass, 2 on a refusal to
/// run.
pub fn run(apply: bool, uninstall: bool) -> i32 {
    if is_root() {
        eprintln!(
            "error: setup: run it as your user, not under sudo: the config belongs under your \
             home, $HOME under sudo may be root's, and setup calls sudo itself for the \
             permissions"
        );
        return 2;
    }
    let user = match std::env::var("USER") {
        Ok(u) if plain_account_name(&u) => u,
        Ok(u) => {
            eprintln!(
                "error: setup: USER {u:?} is not a plain account name: it is written into a udev \
                 rule and a root script"
            );
            return 2;
        }
        Err(_) => {
            eprintln!("error: setup: USER is unset: cannot name whose the permissions are");
            return 2;
        }
    };
    let ok = if uninstall {
        permissions_step(apply, &user, true)
    } else {
        // Both steps run even when the first fails, so one invocation reports everything.
        let config_ok = config_step(apply);
        println!();
        let permissions_ok = permissions_step(apply, &user, false);
        config_ok && permissions_ok
    };
    if ok { 0 } else { 1 }
}

/// Whether `name` is safe to write unquoted into a udev rule and a shell script: letters, digits,
/// `_`, `.`, and `-`, not leading with `-`.
fn plain_account_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('-')
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

/// Whether the process runs as root.
fn is_root() -> bool {
    // SAFETY: geteuid takes no arguments, cannot fail, and touches no memory.
    unsafe { libc::geteuid() == 0 }
}

/// The config step: plan, check, print, and with `apply` write. Returns whether the host's
/// declaration is (or, printing, would be) one every pin and restore accepts.
fn config_step(apply: bool) -> bool {
    let path = match config::xdg_target() {
        Ok(Some(p)) => p,
        Ok(None) => {
            eprintln!("error: setup: neither XDG_CONFIG_HOME nor HOME is set: no config home");
            return false;
        }
        Err(e) => {
            eprintln!("error: setup: {e}");
            return false;
        }
    };
    let section = match freqctl::freq_section() {
        Ok(lines) => lines,
        Err(e) => {
            eprintln!("error: setup: {e}");
            return false;
        }
    };
    let existing = match std::fs::read_to_string(&path) {
        Ok(text) => Some(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            eprintln!("error: setup: reading {}: {e}", path.display());
            return false;
        }
    };
    let plan = match plan_config(&path, existing.as_deref(), &section) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: setup: {e}");
            return false;
        }
    };
    match plan {
        ConfigPlan::Declared { path, config } => {
            println!(
                "config: {} already declares [freq], left as is",
                path.display()
            );
            match freqctl::check_steady(config.freq.as_ref()) {
                Ok(()) => {
                    println!("config: its [freq] passes every pin and restore check");
                    match &config.freq {
                        Some(freq) => live_step(freq, &section),
                        None => true,
                    }
                }
                Err(e) => {
                    eprintln!("error: setup: its [freq] does not pass: {e}");
                    eprintln!("The live state as a [freq] section, to merge by hand:");
                    for line in &section {
                        eprintln!("  {line}");
                    }
                    false
                }
            }
        }
        ConfigPlan::Create { path, text } => write_step(apply, &path, &text, &text, false),
        ConfigPlan::Append { path, text, whole } => write_step(apply, &path, &text, &whole, true),
    }
}

/// Compare an existing declaration with the live state, since a declaration can pass every
/// range check and still name a state the host never runs at. Returns whether they agree, or
/// could not be compared because the clamp is pinned.
fn live_step(freq: &config::FreqConfig, section: &[String]) -> bool {
    let check = match freqctl::live_check(freq) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: setup: {e}");
            return false;
        }
    };
    if check.mismatches.is_empty() {
        if check.pinned {
            println!("config: its [freq] matches the live state, a pin (min = max) the host holds");
        } else {
            println!("config: its [freq] matches the live state");
        }
        return true;
    }
    eprintln!("error: setup: its [freq] is not the state this host runs at:");
    for m in &check.mismatches {
        eprintln!("  {m}");
    }
    if check.pinned {
        eprintln!(
            "The host's clamp is min = max now: if a pin is still running, rerun setup after it \
             restores."
        );
    }
    eprintln!(
        "A restore would move the host to the declared state. If that state is wrong, remove \
         the [freq] table and rerun setup --apply, which writes the live state below. If it is \
         right, restore-freq moves the host to it."
    );
    for line in section {
        eprintln!("  {line}");
    }
    false
}

/// Check, print, and with `apply` write a planned `text`, `whole` being the file as it would
/// read afterwards. Returns whether the declaration passes (and, applying, was written).
fn write_step(apply: bool, path: &Path, text: &str, whole: &str, appending: bool) -> bool {
    let checked =
        config::parse_text(path, whole).and_then(|c| freqctl::check_steady(c.freq.as_ref()));
    if let Err(e) = checked {
        eprintln!("error: setup: the live state does not make a declaration: {e}");
        return false;
    }
    let verb = match (apply, appending) {
        (false, false) => "would create",
        (false, true) => "would append to",
        (true, false) => "creating",
        (true, true) => "appending to",
    };
    println!("config: {verb} {}:", path.display());
    println!();
    print!("{text}");
    println!();
    if !apply {
        println!("config: rerun with --apply to write it");
        return true;
    }
    match write_config(path, text, appending) {
        Ok(()) => {
            println!("config: wrote {}", path.display());
            true
        }
        Err(e) => {
            eprintln!("error: setup: {e}");
            false
        }
    }
}

/// The permissions step: show the rule and the files, and with `apply` install (or, with
/// `uninstall`, remove) them through one sudo. Returns whether the step is done or, printing,
/// shown.
fn permissions_step(apply: bool, user: &str, uninstall: bool) -> bool {
    let mut files = crate::freqctl::written_paths();
    if files.is_empty() {
        eprintln!("error: setup: this box exposes no cpufreq files to hand over");
        return false;
    }
    if Path::new(DMA_LATENCY).exists() {
        files.push(DMA_LATENCY.to_string());
    }
    let targets = chown_targets(&files);
    let rule = rule_text(user);
    let installed = std::fs::read_to_string(RULE_PATH).is_ok_and(|t| t == rule);
    let uid = file_uid_of_all(&files);
    let owned = uid.is_some_and(|u| u == own_uid());
    if uninstall {
        if !installed && !Path::new(RULE_PATH).exists() && uid == Some(0) {
            println!("permissions: not installed, nothing to remove");
            return true;
        }
        println!(
            "permissions: {} {RULE_PATH} and give {} files back to root:",
            if apply { "removing" } else { "would remove" },
            files.len()
        );
        return run_script(apply, &uninstall_script(&targets));
    }
    if installed && owned {
        println!("permissions: {RULE_PATH} is installed and {user} owns every file, nothing to do");
        return true;
    }
    println!(
        "permissions: {} {RULE_PATH} and hand {user} {} files:",
        if apply { "installing" } else { "would install" },
        files.len()
    );
    run_script(apply, &apply_script(user, &rule, &targets))
}

/// The files as the scripts name them: each per-CPU knob once, as a `cpu[0-9]*` glob the root
/// shell expands, and any other file verbatim, so the script stays a few lines on a 24-CPU box.
fn chown_targets(files: &[String]) -> Vec<String> {
    let mut targets: Vec<String> = Vec::new();
    for f in files {
        let target = match per_cpu_knob(f) {
            Some(name) => format!("/sys/devices/system/cpu/cpu[0-9]*/cpufreq/{name}"),
            None => f.clone(),
        };
        if !targets.contains(&target) {
            targets.push(target);
        }
    }
    targets
}

/// The knob name of a per-CPU cpufreq path (`.../cpu12/cpufreq/boost` -> `boost`), `None` for
/// anything else.
fn per_cpu_knob(path: &str) -> Option<&str> {
    let rest = path.strip_prefix("/sys/devices/system/cpu/cpu")?;
    let (id, knob) = rest.split_once("/cpufreq/")?;
    if !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()) {
        Some(knob)
    } else {
        None
    }
}

/// Print the root script, and with `apply` run it through `sudo sh -c`, the one password
/// prompt. Returns whether it was shown or ran cleanly.
fn run_script(apply: bool, script: &str) -> bool {
    println!("  as root:");
    for line in script.lines() {
        println!("    {line}");
    }
    println!();
    if !apply {
        println!("permissions: rerun with --apply to run it (one sudo)");
        return true;
    }
    match std::process::Command::new("sudo")
        .arg("sh")
        .arg("-c")
        .arg(script)
        .status()
    {
        Ok(status) if status.success() => {
            println!("permissions: done");
            true
        }
        Ok(status) => {
            eprintln!("error: setup: the root script failed ({status})");
            false
        }
        Err(e) => {
            eprintln!("error: setup: running sudo: {e}");
            false
        }
    }
}

/// The effective uid of this process.
fn own_uid() -> u32 {
    // SAFETY: geteuid takes no arguments, cannot fail, and touches no memory.
    unsafe { libc::geteuid() }
}

/// The owner every file shares, `None` when they differ or one is unreadable.
fn file_uid_of_all(files: &[String]) -> Option<u32> {
    let mut shared = None;
    for f in files {
        let uid = std::fs::metadata(f).ok()?.uid();
        match shared {
            None => shared = Some(uid),
            Some(u) if u == uid => {}
            Some(_) => return None,
        }
    }
    shared
}

/// The udev rule handing `user` the cpufreq files and the latency clamp on every boot and CPU
/// hotplug. One `RUN` per file so no shell quoting passes through udev, a missing file (a box
/// without per-CPU boost) failing that one `chown` harmlessly.
fn rule_text(user: &str) -> String {
    let mut out = format!(
        "# iiac-perf setup: {user} sets the CPU clock and the wake-latency clamp without sudo.\n\
         # Written by `iiac-perf setup --apply`, removed by `iiac-perf setup --uninstall --apply`.\n"
    );
    for name in crate::freqctl::WRITTEN_KNOBS {
        out.push_str(&format!(
            "SUBSYSTEM==\"cpu\", ACTION==\"add\", RUN+=\"/usr/bin/chown {user} /sys%p/cpufreq/{name}\"\n"
        ));
    }
    out.push_str(&format!(
        "SUBSYSTEM==\"cpu\", KERNEL==\"cpu0\", ACTION==\"add\", RUN+=\"/usr/bin/chown {user} {}\"\n",
        crate::freqctl::GLOBAL_BOOST_PATH
    ));
    out.push_str(&format!("KERNEL==\"cpu_dma_latency\", OWNER=\"{user}\"\n"));
    out
}

/// The root script `--apply` runs: install the rule, reload udev, and hand over the files now,
/// since the rule acts only on the next boot or hotplug.
fn apply_script(user: &str, rule: &str, targets: &[String]) -> String {
    let mut out = format!(
        "set -e\ncat > {RULE_PATH} <<'IIAC_PERF_RULE'\n{rule}IIAC_PERF_RULE\nudevadm control --reload\n"
    );
    for t in targets {
        out.push_str(&format!("chown {user} {t}\n"));
    }
    out
}

/// The root script `--uninstall --apply` runs: remove the rule, reload udev, and give the files
/// back to root.
fn uninstall_script(targets: &[String]) -> String {
    let mut out = format!("set -e\nrm -f {RULE_PATH}\nudevadm control --reload\n");
    for t in targets {
        out.push_str(&format!("chown root {t}\n"));
    }
    out
}

/// Decide what to do with the config at `path`, whose current text is `existing` (`None` when
/// no file exists), given the live `[freq]` section lines. Pure, so the decision is tested
/// without a filesystem.
fn plan_config(
    path: &Path,
    existing: Option<&str>,
    section: &[String],
) -> Result<ConfigPlan, String> {
    let md = path.extension().is_some_and(|e| e == "md");
    let Some(text) = existing else {
        let body = if md {
            format!("# iiac-perf config\n\n{}", md_section(section))
        } else {
            toml_section(section)
        };
        return Ok(ConfigPlan::Create {
            path: path.to_path_buf(),
            text: body,
        });
    };
    let parsed = config::parse_text(path, text)?;
    if parsed.freq.is_some() {
        return Ok(ConfigPlan::Declared {
            path: path.to_path_buf(),
            config: Box::new(parsed),
        });
    }
    let separator = if text.is_empty() || text.ends_with("\n\n") {
        ""
    } else if text.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    let addition = if md {
        md_section(section)
    } else {
        toml_section(section)
    };
    let addition = format!("{separator}{addition}");
    Ok(ConfigPlan::Append {
        path: path.to_path_buf(),
        whole: format!("{text}{addition}"),
        text: addition,
    })
}

/// The section as a markdown carrier's prose and `toml` fence. It goes last in the file, since
/// the fences concatenate in document order and a bare key after a table header would land in
/// that table.
fn md_section(section: &[String]) -> String {
    format!(
        "The `[freq]` table below is this host's clock steady state. `iiac-perf restore-freq` \
         sets the\nCPU's governor, EPP, boost, and clamp (`min_mhz` to `max_mhz`) to these \
         values, and every pin\nreturns to them on exit. `iiac-perf setup` wrote them from the \
         live state.\n\n```toml\n{}```\n",
        toml_section(section)
    )
}

/// The section as plain TOML lines.
fn toml_section(section: &[String]) -> String {
    let mut out = String::new();
    for line in section {
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Write the planned text: create the file (and its directory) when it is new, never truncating
/// one that appeared since the plan, or append to the existing file.
fn write_config(path: &Path, text: &str, appending: bool) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    }
    let mut options = std::fs::OpenOptions::new();
    if appending {
        options.append(true);
    } else {
        options.write(true).create_new(true);
    }
    let mut file = options
        .open(path)
        .map_err(|e| format!("opening {}: {e}", path.display()))?;
    file.write_all(text.as_bytes())
        .map_err(|e| format!("writing {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section() -> Vec<String> {
        [
            "[freq]",
            "governor = \"powersave\"",
            "boost = true",
            "min_mhz = 1745",
            "max_mhz = 4673",
        ]
        .iter()
        .map(|l| l.to_string())
        .collect()
    }

    #[test]
    fn a_missing_file_is_created_and_parses() {
        let path = Path::new("/home/u/.config/iiac-perf/config.md");
        let ConfigPlan::Create { text, .. } = plan_config(path, None, &section()).unwrap() else {
            panic!("expected Create");
        };
        assert!(text.starts_with("# iiac-perf config\n\n"), "got: {text}");
        let freq = config::parse_text(path, &text).unwrap().freq.unwrap();
        assert_eq!(freq.min_mhz, Some(1745));
        assert_eq!(freq.max_mhz, Some(4673));
    }

    #[test]
    fn a_file_without_freq_gains_it_at_its_end() {
        let path = Path::new("config.md");
        let existing = "Blocks.\n\n```toml\nblocks = 50\n```";
        let ConfigPlan::Append { text, .. } =
            plan_config(path, Some(existing), &section()).unwrap()
        else {
            panic!("expected Append");
        };
        assert!(
            text.starts_with("\n\nThe `[freq]` table below"),
            "got: {text}"
        );
        let merged = config::parse_text(path, &format!("{existing}{text}")).unwrap();
        assert_eq!(merged.blocks, Some(50));
        assert_eq!(merged.freq.unwrap().max_mhz, Some(4673));
    }

    #[test]
    fn a_toml_carrier_gains_plain_toml() {
        let path = Path::new("config.toml");
        let existing = "decimals = 2\n";
        let ConfigPlan::Append { text, .. } =
            plan_config(path, Some(existing), &section()).unwrap()
        else {
            panic!("expected Append");
        };
        assert_eq!(text, format!("\n{}", toml_section(&section())));
        let merged = config::parse_text(path, &format!("{existing}{text}")).unwrap();
        assert_eq!(merged.decimals, Some(2));
        assert!(merged.freq.is_some());
    }

    #[test]
    fn a_file_declaring_freq_is_left_alone() {
        let path = Path::new("config.toml");
        let existing = "[freq]\ngovernor = \"powersave\"\n";
        let ConfigPlan::Declared { config, .. } =
            plan_config(path, Some(existing), &section()).unwrap()
        else {
            panic!("expected Declared");
        };
        assert_eq!(config.freq.unwrap().governor, "powersave");
    }

    #[test]
    fn the_rule_hands_every_written_knob_to_the_user() {
        let rule = rule_text("wink");
        for name in crate::freqctl::WRITTEN_KNOBS {
            let line = format!(
                "SUBSYSTEM==\"cpu\", ACTION==\"add\", RUN+=\"/usr/bin/chown wink /sys%p/cpufreq/{name}\""
            );
            assert!(rule.contains(&line), "missing {name} in:\n{rule}");
        }
        assert!(rule.contains("/sys/devices/system/cpu/cpufreq/boost"));
        assert!(rule.contains("KERNEL==\"cpu_dma_latency\", OWNER=\"wink\""));
        // udev expands `$`, so the rule must carry none.
        assert!(!rule.contains('$'), "got:\n{rule}");
    }

    #[test]
    fn only_plain_account_names_reach_the_scripts() {
        assert!(plain_account_name("wink"));
        assert!(plain_account_name("a.b_c-1"));
        for bad in ["", "-rf", "a b", "a;rm", "a$b", "a\"b"] {
            assert!(!plain_account_name(bad), "{bad:?} passed");
        }
    }

    #[test]
    fn per_cpu_knobs_collapse_to_one_glob_each() {
        let files = [
            "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor".to_string(),
            "/sys/devices/system/cpu/cpu1/cpufreq/scaling_governor".to_string(),
            "/sys/devices/system/cpu/cpu12/cpufreq/boost".to_string(),
            "/sys/devices/system/cpu/cpufreq/boost".to_string(),
            "/dev/cpu_dma_latency".to_string(),
        ];
        assert_eq!(
            chown_targets(&files),
            [
                "/sys/devices/system/cpu/cpu[0-9]*/cpufreq/scaling_governor",
                "/sys/devices/system/cpu/cpu[0-9]*/cpufreq/boost",
                "/sys/devices/system/cpu/cpufreq/boost",
                "/dev/cpu_dma_latency",
            ]
        );
    }

    #[test]
    fn the_scripts_install_and_remove_the_same_files() {
        let files = [
            "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor".to_string(),
            "/dev/cpu_dma_latency".to_string(),
        ];
        let rule = rule_text("wink");
        let apply = apply_script("wink", &rule, &files);
        assert!(apply.starts_with(
            "set -e\ncat > /etc/udev/rules.d/70-iiac-perf.rules <<'IIAC_PERF_RULE'\n"
        ));
        assert!(apply.contains(&format!("{rule}IIAC_PERF_RULE\nudevadm control --reload\n")));
        assert!(apply.ends_with(
            "chown wink /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor\n\
             chown wink /dev/cpu_dma_latency\n"
        ));
        let remove = uninstall_script(&files);
        assert!(remove.starts_with("set -e\nrm -f /etc/udev/rules.d/70-iiac-perf.rules\n"));
        assert!(remove.ends_with("chown root /dev/cpu_dma_latency\n"));
    }

    #[test]
    fn a_malformed_file_is_an_error_not_an_append() {
        let path = Path::new("config.toml");
        assert!(plan_config(path, Some("bogus = 1\n"), &section()).is_err());
    }
}
