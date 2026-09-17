//! The starting config: one template, every key commented out, and the commands that write it.
//!
//! The template is `iiac-perf.example.md`, compiled in, so the file in the repo and the file an
//! installed binary writes cannot differ. It holds every key at its built-in default, or at a
//! sample value where a key has none, each one commented out, so a fresh file changes nothing
//! until a line is uncommented.
//!
//! - `init-config [PATH]` prints the template, or writes it to PATH, never over a file.
//! - `init-config --from OLD [PATH]` brings a file up to date by writing a fresh one: the
//!   template with every key OLD sets uncommented at OLD's value. A key OLD lacks arrives with
//!   the template, and a key the loader no longer knows fails OLD's parse by name. OLD is never
//!   written, and its author's own prose is what the new file loses.
//! - `setup` creates a missing XDG file from the same template, the live `[freq]` filled in.
//!
//! The template's one rule makes this mechanical: inside a `toml` fence every `#` line is a
//! key or a table header, and a table has a fence to itself after every top-level key.

use std::io::Write;
use std::path::Path;

use crate::config;

/// The template, in the markdown carrier.
pub const TEMPLATE: &str = include_str!("../iiac-perf.example.md");

/// A template fence line as a key line: `# blocks = 100` is `blocks = 100`.
fn uncommented(line: &str) -> Option<&str> {
    line.strip_prefix("# ")
}

/// The table a header line opens: `[freq]` is `freq`.
fn table_name(line: &str) -> Option<&str> {
    line.strip_prefix('[')?.strip_suffix(']')
}

/// The key a top-level key line sets: `blocks = 100` is `blocks`.
fn key_name(line: &str) -> Option<&str> {
    let (key, _) = line.split_once('=')?;
    Some(key.trim())
}

/// The template with `old`'s values set and, when given, `freq_section` in place of the
/// `[freq]` sample: the live state's lines, the header first, as `setup` has them. An error
/// when one of `old`'s tables will not write back as TOML, so a file never silently lacks it.
pub fn render(
    old: Option<&toml::Table>,
    freq_section: Option<&[String]>,
) -> Result<String, String> {
    let mut out = String::new();
    let mut in_fence = false;
    // Inside a table's fence whose lines were replaced: the template's samples are dropped.
    let mut dropping = false;
    // Inside a table's fence at all: its lines are the table's entries, not top-level keys.
    let mut in_table = false;
    for line in TEMPLATE.lines() {
        if !in_fence {
            in_fence = line.starts_with("```toml");
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if line.starts_with("```") {
            (in_fence, dropping, in_table) = (false, false, false);
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if dropping {
            continue;
        }
        let replaced = match uncommented(line) {
            Some(body) if table_name(body).is_some() => {
                in_table = true;
                table_lines(body, old, freq_section)?
            }
            Some(body) if !in_table => key_line(body, old),
            _ => None,
        };
        match replaced {
            Some(text) => {
                dropping = in_table;
                out.push_str(&text);
            }
            None => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    Ok(out)
}

/// The lines that replace a table's commented sample, `None` when nothing sets that table.
fn table_lines(
    header: &str,
    old: Option<&toml::Table>,
    freq_section: Option<&[String]>,
) -> Result<Option<String>, String> {
    let Some(name) = table_name(header) else {
        return Ok(None);
    };
    if name == "freq"
        && let Some(section) = freq_section
    {
        let mut text = String::new();
        for line in section {
            text.push_str(line);
            text.push('\n');
        }
        return Ok(Some(text));
    }
    let Some(value) = old.and_then(|t| t.get(name)) else {
        return Ok(None);
    };
    let mut table = toml::Table::new();
    table.insert(name.to_string(), value.clone());
    match toml::to_string(&table) {
        Ok(text) => Ok(Some(text)),
        Err(e) => Err(format!("[{name}] will not write back as TOML: {e}")),
    }
}

/// The line that replaces a commented top-level key, when `old` sets it.
fn key_line(body: &str, old: Option<&toml::Table>) -> Option<String> {
    let key = key_name(body)?;
    let value = old?.get(key)?;
    Some(format!("{key} = {value}\n"))
}

/// A markdown config as plain TOML: the fences' lines as they are and every other line a
/// comment, so the prose survives in the other carrier.
fn to_toml(md: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    for line in md.lines() {
        if line.starts_with("```") {
            in_fence = !in_fence && line.starts_with("```toml");
            continue;
        }
        if in_fence || line.is_empty() {
            out.push_str(line);
        } else {
            out.push_str("# ");
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// The `init-config` command: print the starting config, or write it to `path`, with `from`'s
/// values set when given. A `.toml` path gets the TOML carrier, anything else the markdown one.
pub fn run(path: Option<&Path>, from: Option<&Path>) -> i32 {
    let old = match from {
        None => None,
        Some(old_path) => match read_old(old_path) {
            Ok(table) => Some(table),
            Err(e) => {
                eprintln!("error: init-config: {e}");
                return 2;
            }
        },
    };
    let mut text = match render(old.as_ref(), None) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("error: init-config: {e}");
            return 1;
        }
    };
    let Some(path) = path else {
        print!("{text}");
        return 0;
    };
    if path.extension().is_some_and(|e| e == "toml") {
        text = to_toml(&text);
    }
    match write_new(path, &text) {
        Ok(()) => {
            println!("init-config: wrote {}", path.display());
            0
        }
        Err(e) => {
            eprintln!("error: init-config: {e}");
            1
        }
    }
}

/// Read and check the file whose values carry over. It goes through the loader's own checks
/// first, so a stale key or a bad value is reported as a load reports it.
fn read_old(path: &Path) -> Result<toml::Table, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    config::parse_text(path, &text)?;
    config::parse_table(path, &text)
}

/// Write `text` to a new file at `path`, never over one that exists.
fn write_new(path: &Path, text: &str) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => {
                format!("{} exists, and is left as it is", path.display())
            }
            _ => format!("opening {}: {e}", path.display()),
        })?;
    file.write_all(text.as_bytes())
        .map_err(|e| format!("writing {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, PinFreq};

    /// The template with every key line uncommented but `skip`, and but the bare `[freq]`
    /// header, which carries no values since a host's own are the only right ones.
    fn all_set(skip: &str) -> String {
        let mut out = String::new();
        let mut in_fence = false;
        for line in TEMPLATE.lines() {
            if line.starts_with("```") {
                in_fence = !in_fence && line.starts_with("```toml");
            }
            match uncommented(line) {
                Some(body) if in_fence && key_name(body) != Some(skip) && body != "[freq]" => {
                    out.push_str(body)
                }
                _ => out.push_str(line),
            }
            out.push('\n');
        }
        out
    }

    fn parse(text: &str) -> Config {
        config::parse_text(Path::new("config.md"), text).unwrap()
    }

    #[test]
    fn the_template_sets_nothing() {
        assert_eq!(parse(TEMPLATE), Config::default());
        assert_eq!(render(None, None).unwrap(), TEMPLATE);
    }

    /// The destructuring names every field, so a key added to the config fails to compile here
    /// until the template carries it.
    #[test]
    fn the_template_holds_every_key_at_its_default() {
        let Config {
            benches,
            duration,
            band_labels,
            decimals,
            settle_time,
            warm_cap,
            blocks,
            block_sleep,
            block_warmup,
            runs,
            run_sleep,
            pin_freq,
            total_duration,
            samples,
            inner,
            pin_cpus,
            record,
            env_probe,
            inhibit,
            ticks,
            verbose,
            tags,
            profiles,
            freq,
            sources: _,
        } = parse(&all_set("total_duration"));
        assert_eq!(duration, Some(crate::DEFAULT_DURATION));
        assert_eq!(band_labels, Some(crate::DEFAULT_BAND_LABELS));
        assert_eq!(decimals, Some(crate::DEFAULT_DECIMALS));
        assert_eq!(settle_time, Some(crate::harness::DEFAULT_SETTLE_TIME_S));
        assert_eq!(warm_cap, Some(crate::harness::DEFAULT_WARM_CAP_S));
        assert_eq!(blocks, Some(crate::harness::DEFAULT_BLOCKS));
        assert_eq!(block_sleep, Some(crate::harness::DEFAULT_BLOCK_SLEEP_S));
        assert_eq!(block_warmup, Some(0.0));
        assert_eq!(runs, Some(crate::DEFAULT_RUNS));
        assert_eq!(run_sleep, Some(crate::runs::DEFAULT_RUN_SLEEP_S));
        assert_eq!(env_probe, Some(true));
        assert_eq!(inhibit, Some(true));
        assert_eq!(ticks, Some(false));
        assert_eq!(verbose, Some(false));
        // No default, so a sample: present is what the template owes.
        assert!(benches.is_some());
        assert!(matches!(pin_freq, Some(PinFreq::MinMhz)));
        assert!(samples.is_some() && inner.is_some());
        assert!(pin_cpus.is_some() && record.is_some());
        assert!(!tags.is_empty() && !profiles.is_empty());
        // The header alone, for `setup` and `--from` to fill.
        assert!(freq.is_none() && TEMPLATE.contains("\n# [freq]\n"));
        // The other duration, which a file may not set beside `duration`.
        assert_eq!(total_duration, None);
        assert!(parse(&all_set("duration")).total_duration.is_some());
    }

    #[test]
    fn from_carries_a_files_own_values_and_nothing_else() {
        let old_text = "benches = [\"min-now\", \"std-now\"]\ntotal_duration = \"30s\"\n\
                        blocks = 10\nrun_sleep = \"0\"\npin_freq = 3801\nverbose = true\n\
                        [tags]\n\"odd key\" = \"a \\\"quoted\\\" value\"\n\
                        [profiles]\nsmt = \"0,12\"\n\
                        [freq]\ngovernor = \"powersave\"\nmin_mhz = 1745\nmax_mhz = 4673\n";
        let path = Path::new("old.toml");
        let old = config::parse_table(path, old_text).unwrap();
        let new = render(Some(&old), None).unwrap();
        assert_eq!(parse(&new), config::parse_text(path, old_text).unwrap());
        // A key the old file leaves alone stays commented, and the sample tables are gone.
        assert!(new.contains("\n# decimals = 1\n"), "got: {new}");
        assert!(new.contains("\nblocks = 10\n"), "got: {new}");
        assert!(!new.contains("ccd ="), "got: {new}");
        // The TOML carrier says the same.
        let as_toml = to_toml(&new);
        assert_eq!(
            config::parse_text(path, &as_toml).unwrap(),
            config::parse_text(path, old_text).unwrap()
        );
        assert!(as_toml.starts_with("# # iiac-perf config example\n"));
    }

    #[test]
    fn a_live_freq_section_replaces_the_sample() {
        let section = [
            "[freq]".to_string(),
            "governor = \"schedutil\"".to_string(),
            "# pin_mhz omitted: the base clock".to_string(),
        ];
        let text = render(None, Some(&section)).unwrap();
        let freq = parse(&text).freq.unwrap();
        assert_eq!(freq.governor, "schedutil");
        assert!(!text.contains("balance_performance"), "got: {text}");
    }

    #[test]
    fn an_existing_file_is_never_written_over() {
        let dir =
            std::env::temp_dir().join(format!("iiac-perf-init-config-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("iiac-perf.md");
        write_new(&path, "first\n").unwrap();
        let err = write_new(&path, "second\n").unwrap_err();
        assert!(err.contains("exists"), "unexpected error: {err}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "first\n");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
