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
//! - **Run as the user**: the config belongs under the user's home, and under sudo `$HOME` may
//!   be root's.

use std::io::Write;
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

/// The `setup` command: print the plan, and carry it out with `apply`. Exit 0 when the host is
/// ready (or would be, printing), 1 when a step failed or a declaration does not pass, 2 on a
/// refusal to run.
pub fn run(apply: bool) -> i32 {
    if is_root() {
        eprintln!(
            "error: setup: run it as your user, not under sudo: the config belongs under your \
             home, and $HOME under sudo may be root's"
        );
        return 2;
    }
    if config_step(apply) { 0 } else { 1 }
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
                    true
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
        "This host's clock steady state: what `restore-freq` converges to and every pin \
         restores on exit.\n`setup` wrote it from the live state, `min_mhz` and `max_mhz` being \
         the clamp the host runs at.\n\n```toml\n{}```\n",
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
        assert!(text.starts_with("\n\nThis host's"), "got: {text}");
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
    fn a_malformed_file_is_an_error_not_an_append() {
        let path = Path::new("config.toml");
        assert!(plan_config(path, Some("bogus = 1\n"), &section()).is_err());
    }
}
