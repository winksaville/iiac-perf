//! The starting config: one template, every key commented out, and the commands that write it.
//!
//! The template is `iiac-perf.example.md`, compiled in, so the file in the repo and the file an
//! installed binary writes cannot differ. It holds every key at its built-in default, or at a
//! sample value where a key has none, each one commented out, so a fresh file changes nothing
//! until a line is uncommented.
//!
//! - `init-config [PATH]` prints the template, or writes it to PATH, never over a file. A
//!   `.toml` PATH gets headings and keys without the prose, which as comments buries the keys.
//! - `init-config --from OLD [PATH]` brings a file up to date by writing a fresh one: the
//!   template with every key OLD sets uncommented at OLD's value. A key OLD lacks arrives with
//!   the template, and a key the loader no longer knows fails OLD's parse by name. OLD is never
//!   written, and its author's own prose is what the new file loses.
//! - Run flags on an `init-config` line set their keys in the new file, so a command line that
//!   worked becomes a file. Under them is what that line would have run on: the run keys the
//!   host's XDG and local files set, layered as the loader layers them, or `--from`'s or
//!   `--config NAME`'s file in their place. So the file is the run the line makes here. The
//!   host's `[freq]` and `[profiles]` are not copied: a run under the new file still gets them
//!   from the host's files, and a copied clamp is wrong on the next host.
//! - `init-config --from-record FILE [PATH]` starts from a record's `config.run`, the run as it
//!   resolved, one invocation's when FILE holds several, `--series ID` choosing. What was true of
//!   the record's host alone is named the way this host names it or left out with a comment: a
//!   pool of cpu numbers becomes this host's profile for the record's placement, and a clock in
//!   MHz and a record path are left out. Notes beside the file say where this host differs.
//! - `update-config FILE` is the same fill written back over FILE: its own values, the line's
//!   over them, the old file kept as `FILE.bak` only when `--backup` asks.
//! - `setup-freq` creates a missing XDG file from the same template, the live `[freq]` filled in.
//!
//! The template's one rule makes this mechanical: inside a `toml` fence a `#` with no space
//! after it is a commented-out key or table header, `#blocks = 100`, and one with a space is a
//! comment. A table has a fence to itself after every top-level key.

use std::io::Write;
use std::path::Path;

use crate::config;

/// The template, in the markdown carrier.
pub const TEMPLATE: &str = include_str!("../iiac-perf.example.md");

/// A template fence line as a key line: `#blocks = 100` is `blocks = 100`. A `#` with a space
/// after it is a comment, never a key, which is how a reader tells the two apart too.
fn uncommented(line: &str) -> Option<&str> {
    let body = line.strip_prefix('#')?;
    if body.starts_with(' ') || body.is_empty() {
        None
    } else {
        Some(body)
    }
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
/// `[freq]` sample: the live state's lines, the header first, as `setup-freq` has them. An error
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

/// What heads the TOML carrier in place of the prose it leaves out.
const TOML_HEAD: &str = "\
# iiac-perf config: every key, commented out at its default, or at a sample where it has none.
# Uncomment a line to set it. `iiac-perf init-config` prints this file with the prose that
# explains each key, and docs/config.md is the reference.
";

/// A markdown config as plain TOML: the fences' lines as they are, each section's heading as a
/// ruled comment, and the prose left out. As comments the prose and the commented keys both
/// begin `# `, and a set key is lost among them, where markdown's fences keep the two apart.
fn to_toml(md: &str) -> String {
    let mut out = TOML_HEAD.to_string();
    let mut in_fence = false;
    for line in md.lines() {
        if line.starts_with("```") {
            // A section's fences run together: the blank lines are the headings'.
            in_fence = !in_fence && line.starts_with("```toml");
        } else if in_fence {
            out.push_str(line);
            out.push('\n');
        } else if let Some(heading) = line.strip_prefix("## ") {
            out.push_str(&format!("\n# ---- {heading} ----\n\n"));
        }
    }
    out
}

/// Where an `init-config` line's starting values come from.
pub enum Start<'a> {
    /// The host's files, as a plain run layers them.
    Host,
    /// `--from OLD`: this file.
    From(&'a Path),
    /// `--config NAME`: the file the search finds.
    Named(&'a Path),
    /// `--from-record FILE`: a record's run config, `--series ID` choosing the invocation.
    Record(&'a Path, Option<&'a str>),
}

/// What `init-config` does when its PATH already exists.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Existing {
    /// Refuse, the file left as it is.
    Refuse,
    /// `--overwrite`: replace it, keeping nothing.
    Overwrite,
    /// `--backup`: replace it, the old file kept as `PATH.bak`.
    Backup,
}

/// The `init-config` command: print the starting config, or write it to `path`, with the
/// `start` file's values set and `line`'s, the run flags' keys, over them. A `.toml` path gets
/// the TOML carrier, anything else the markdown one. An existing file's own values are not
/// kept, which is `update-config`'s job.
pub fn run(path: Option<&Path>, start: Start, line: toml::Table, existing: Existing) -> i32 {
    let (mut text, taken) = match starting_text(start, line) {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("error: init-config: {e}");
            return 2;
        }
    };
    let Some(path) = path else {
        // The file is the output, so what it took goes beside it, not into it.
        for note in taken {
            eprintln!("init-config: {note}");
        }
        print!("{text}");
        return 0;
    };
    for note in taken {
        println!("init-config: {note}");
    }
    text = in_carrier(path, text);
    let written = match existing {
        Existing::Refuse => write_new(path, &text).map(|()| Vec::new()),
        Existing::Overwrite => replace(path, &text, false),
        Existing::Backup => replace(path, &text, true),
    };
    match written {
        Ok(notes) => {
            for note in notes {
                println!("init-config: {note}");
            }
            println!("init-config: wrote {}", path.display());
            0
        }
        Err(e) => {
            eprintln!("error: init-config: {e}");
            1
        }
    }
}

/// Put `text` at `file` whether or not one is there: the old file copied to `FILE.bak` first
/// when `backup` asks and there is one, the new text written beside it and renamed over it, so
/// a failure leaves `file` whole. Returns what to tell the user.
fn replace(file: &Path, text: &str, backup: bool) -> Result<Vec<String>, String> {
    let mut notes = Vec::new();
    if backup && file.exists() {
        let kept = beside(file, ".bak");
        std::fs::copy(file, &kept).map_err(|e| format!("copying to {}: {e}", kept.display()))?;
        notes.push(format!("kept the old file as {}", kept.display()));
    }
    let staged = beside(file, ".new");
    std::fs::write(&staged, text).map_err(|e| format!("writing {}: {e}", staged.display()))?;
    std::fs::rename(&staged, file).map_err(|e| format!("replacing {}: {e}", file.display()))?;
    Ok(notes)
}

/// The new file's text: the template with the start file's values and the line's set, checked
/// as a load would check it, so a bad flag value stops here rather than in the file's first run.
fn starting_text(start: Start, line: toml::Table) -> Result<(String, Vec<String>), String> {
    let (old, taken) = match start {
        Start::Host => {
            let (table, taken) = host_start(&config::host_files()?, &line)?;
            (Some(table), taken)
        }
        Start::From(path) => (Some(read_old(path)?), Vec::new()),
        Start::Named(name) => (Some(read_old(&config::find(name)?)?), Vec::new()),
        Start::Record(path, series) => {
            let runs = crate::record::read_runs(path)?;
            let run = chosen(&runs, series)?;
            let (config, _) = config::load(None)?;
            let adapted = adapt(run, &config.profiles, &crate::host::probe());
            let mut text = filled(Some(adapted.table), line.clone())?;
            // A key the line sets needs no word on why the record's was left out.
            for (key, note) in adapted.left_out {
                if !line.contains_key(&key) {
                    text = annotated(&text, &key, &note);
                }
            }
            return Ok((text, adapted.notes));
        }
    };
    Ok((filled(old, line)?, taken))
}

/// The one invocation a record file names: its only series, or the one `series` picks. Several
/// series and no pick is refused with each listed, since an experiment's file holds many and a
/// guess would start from the wrong one.
fn chosen<'a>(
    runs: &'a [crate::record::RecordedRun],
    series: Option<&str>,
) -> Result<&'a crate::record::RecordedRun, String> {
    if let Some(want) = series {
        return runs
            .iter()
            .find(|run| shown(run.series.as_deref(), "none") == want)
            .ok_or_else(|| format!("no record has series {want}\n{}", listing(runs)));
    }
    let first = &runs[0];
    if runs.iter().all(|run| run.series == first.series) {
        return Ok(first);
    }
    Err(format!(
        "the file holds several invocations: pick one with --series ID\n{}",
        listing(runs)
    ))
}

/// `value`, or `absent` in its place: how a note prints a field a record may lack.
fn shown<'a>(value: Option<&'a str>, absent: &'a str) -> &'a str {
    match value {
        Some(v) => v,
        None => absent,
    }
}

/// Each series in a record file, once, in file order: its id, label, and benches.
fn listing(runs: &[crate::record::RecordedRun]) -> String {
    let mut seen: Vec<(Option<&str>, Option<&str>, Vec<&str>)> = Vec::new();
    for run in runs {
        let series = run.series.as_deref();
        match seen.iter_mut().find(|(s, _, _)| *s == series) {
            Some((_, _, benches)) => {
                if !benches.contains(&run.bench.as_str()) {
                    benches.push(&run.bench);
                }
            }
            None => seen.push((series, run.label.as_deref(), vec![&run.bench])),
        }
    }
    seen.iter()
        .map(|(series, label, benches)| {
            format!(
                "  {}  label {}  benches {}",
                shown(*series, "none"),
                shown(*label, "-"),
                benches.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A record's run config made this host's: the keys to write, the keys left out with why, and
/// notes on where this host differs from the record's.
struct Adapted {
    table: toml::Table,
    left_out: Vec<(String, String)>,
    notes: Vec<String>,
}

/// The profile name each placement label is declared under, by the placements rule.
const PLACEMENT_PROFILES: [(&str, &str); 3] = [("SMT", "smt"), ("CCX", "ccx"), ("x-CCX", "x-ccx")];

/// `run`'s config with what was true of its host alone named for this one or left out: a pool of
/// cpu numbers becomes the profile `profiles` declares for the record's placement, a clock in MHz
/// and a record path are left out. Pure, the host's profiles and identity passed in.
fn adapt(
    run: &crate::record::RecordedRun,
    profiles: &std::collections::BTreeMap<String, String>,
    here: &crate::host::Host,
) -> Adapted {
    let mut table = run.run.clone();
    let mut left_out = Vec::new();
    let mut notes = vec![format!(
        "from the record: series {}, label {}, written by {} {} on {}",
        shown(run.series.as_deref(), "none"),
        shown(run.label.as_deref(), "-"),
        crate::BIN_NAME,
        run.version,
        run.host.name
    )];
    let host = &run.host.name;
    if let Some(spec) = table.get("pin_cpus").and_then(toml::Value::as_str)
        && spec.starts_with(|c: char| c.is_ascii_digit())
    {
        let spec = spec.to_string();
        let profile = run.pin_placement.as_deref().and_then(|placement| {
            PLACEMENT_PROFILES
                .iter()
                .find(|(label, _)| *label == placement)
                .map(|(_, name)| *name)
        });
        match profile {
            Some(name) if profiles.contains_key(name) => {
                table.insert(
                    "pin_cpus".to_string(),
                    toml::Value::String(name.to_string()),
                );
                notes.push(format!(
                    "pin_cpus: {host}'s {spec} was {}, so this host's {name} profile, {}",
                    shown(run.pin_placement.as_deref(), "-"),
                    profiles[name]
                ));
            }
            _ => {
                table.remove("pin_cpus");
                let placement = shown(run.pin_placement.as_deref(), "of unknown placement");
                let why = match profile {
                    Some(name) => format!("this host declares no {name} profile"),
                    None => "no profile names that placement".to_string(),
                };
                left_out.push((
                    "pin_cpus".to_string(),
                    format!("the record pinned {host}'s cpus {spec}, {placement}, and {why}"),
                ));
            }
        }
    }
    if let Some(mhz) = table.get("pin_freq").and_then(toml::Value::as_integer) {
        table.remove("pin_freq");
        left_out.push((
            "pin_freq".to_string(),
            format!(
                "the record pinned {host}'s clock at {mhz} MHz, and \"pin_mhz\" names this host's \
                 own"
            ),
        ));
    }
    for key in ["record_dir", "record_file"] {
        if let Some(path) = table.remove(key) {
            left_out.push((
                key.to_string(),
                format!("the record's {key} was {path} on {host}"),
            ));
        }
    }
    let differs = |what: &str, there: Option<&str>, ours: Option<&str>| {
        (there != ours).then(|| {
            format!(
                "{what} differs: {} on {host}, {} here",
                shown(there, "unknown"),
                shown(ours, "unknown")
            )
        })
    };
    notes.extend(
        [
            differs(
                "cpu",
                run.host.cpu_model.as_deref(),
                here.cpu_model.as_deref(),
            ),
            differs("kernel", run.host.kernel.as_deref(), here.kernel.as_deref()),
            differs("rustc", Some(&run.host.rustc), Some(&here.rustc)),
            differs(
                "version",
                Some(&run.version),
                Some(env!("CARGO_PKG_VERSION")),
            ),
        ]
        .into_iter()
        .flatten(),
    );
    Adapted {
        table,
        left_out,
        notes,
    }
}

/// `text` with `note` as a comment above the template's commented-out `key` line, so the file
/// says why a key the record set is not set here. A `# ` line inside a fence is a comment by the
/// template's rule, in either carrier.
fn annotated(text: &str, key: &str, note: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    let mut done = false;
    for line in text.lines() {
        if line.starts_with("```") {
            in_fence = !in_fence && line.starts_with("```toml");
        } else if in_fence && !done && uncommented(line).and_then(key_name) == Some(key) {
            for part in crate::wrap::wrap(note, 96).lines() {
                out.push_str("# ");
                out.push_str(part);
                out.push('\n');
            }
            done = true;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// The run keys `files` set, layered in their order, the later winning, and a line per file
/// naming what the new file takes from it, so nothing is inherited without a word. What `line`
/// sets is the line's, not a file's. `[freq]` and `[profiles]` are the host's and stay there.
fn host_start(
    files: &[std::path::PathBuf],
    line: &toml::Table,
) -> Result<(toml::Table, Vec<String>), String> {
    let mut tables = Vec::new();
    for file in files {
        let mut table = read_old(file)?;
        table.remove("freq");
        table.remove("profiles");
        // In a file "no" is no opinion, so a lower file's pin stands.
        if table.get("pin_freq").and_then(toml::Value::as_str) == Some("no") {
            table.remove("pin_freq");
        }
        tables.push(table);
    }
    let mut start = toml::Table::new();
    for table in &tables {
        start = merged(start, table.clone());
    }
    // A key is its last file's, unless the line sets it or clears it.
    let mut taken = Vec::new();
    for (at, (file, table)) in files.iter().zip(&tables).enumerate() {
        let keys: Vec<&str> = start
            .keys()
            .filter(|key| table.contains_key(*key))
            .filter(|key| {
                !tables[at + 1..]
                    .iter()
                    .any(|later| later.contains_key(*key))
            })
            .filter(|key| *key == "tags" || !line.contains_key(*key))
            .filter(|key| {
                !CHOICES.iter().any(|&(a, b)| {
                    (*key == a && line.contains_key(b)) || (*key == b && line.contains_key(a))
                })
            })
            .map(String::as_str)
            .collect();
        if !keys.is_empty() {
            taken.push(format!(
                "from {}: {}",
                crate::run_config::display_path(file),
                keys.join(", ")
            ));
        }
    }
    Ok((start, taken))
}

/// The run as a config file spells it, what a record carries: the run keys `files` set, layered
/// as [`host_start`] layers them, and `line`'s over them. `files` are those that gave the run its
/// keys, the named file alone under `--config NAME`.
pub fn run_table(files: &[std::path::PathBuf], line: &toml::Table) -> Result<toml::Table, String> {
    let (start, _) = host_start(files, line)?;
    let run = merged(start, line.clone());
    // Checked as a load checks it, so a record never carries a config that will not load.
    let text = toml::to_string(&run).map_err(|e| format!("writing the run's keys: {e}"))?;
    config::parse_text(Path::new("run.toml"), &text)?;
    Ok(run)
}

/// The template with `old`'s values and `line`'s over them, checked as a load checks it.
fn filled(old: Option<toml::Table>, line: toml::Table) -> Result<String, String> {
    let values = match (old, line.is_empty()) {
        (None, true) => None,
        (None, false) => Some(line),
        (Some(old), _) => Some(merged(old, line)),
    };
    let text = render(values.as_ref(), None)?;
    config::parse_text(Path::new("init-config.md"), &text)?;
    Ok(text)
}

/// The keys of one choice, of which a file sets one: the line's pick clears the file's other.
const CHOICES: [(&str, &str); 2] = [
    ("duration", "total_duration"),
    ("record_dir", "record_file"),
];

/// `line`'s keys over `old`'s: a key replaces, the line's tags join the file's, and the line's
/// choice of `duration` or `total_duration`, or of `record_dir` or `record_file`, clears the
/// file's other one.
fn merged(mut old: toml::Table, line: toml::Table) -> toml::Table {
    for (a, b) in CHOICES {
        if line.contains_key(a) {
            old.remove(b);
        }
        if line.contains_key(b) {
            old.remove(a);
        }
    }
    for (key, value) in line {
        match (old.get_mut(&key), value) {
            (Some(toml::Value::Table(have)), toml::Value::Table(more)) => have.extend(more),
            (_, value) => {
                old.insert(key, value);
            }
        }
    }
    old
}

/// `path` with `suffix` after its whole name: `queue.md` and `.bak` is `queue.md.bak`.
fn beside(path: &Path, suffix: &str) -> std::path::PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    name.into()
}

/// `text` in `path`'s carrier: TOML for a `.toml` path, markdown for anything else.
fn in_carrier(path: &Path, text: String) -> String {
    if path.extension().is_some_and(|e| e == "toml") {
        to_toml(&text)
    } else {
        text
    }
}

/// The `update-config` command: rewrite `file` as the template with its own values and
/// `line`'s over them. Everything is read, filled, and checked before `file` is touched, and
/// the new text is written beside it and renamed over it, so a failure leaves `file` whole.
pub fn update(file: &Path, backup: bool, line: toml::Table) -> i32 {
    match updated(file, backup, line) {
        Ok(notes) => {
            for note in notes {
                println!("update-config: {note}");
            }
            0
        }
        Err(e) => {
            eprintln!("error: update-config: {e}");
            2
        }
    }
}

/// [`update`]'s work, returning what to tell the user.
fn updated(file: &Path, backup: bool, line: toml::Table) -> Result<Vec<String>, String> {
    let before =
        std::fs::read_to_string(file).map_err(|e| format!("reading {}: {e}", file.display()))?;
    let old = read_old(file)?;
    // What the file would read as with nothing of its author's in it: when it already does,
    // the rewrite loses nothing.
    let plain = in_carrier(file, filled(Some(old.clone()), toml::Table::new())?);
    let text = in_carrier(file, filled(Some(old), line)?);
    let mut notes = replace(file, &text, backup)?;
    if !backup && before != plain {
        notes.push(format!(
            "{}'s own prose and comments are not carried over, and no --backup was asked",
            file.display()
        ));
    }
    notes.push(format!("wrote {}", file.display()));
    Ok(notes)
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
                format!(
                    "{} exists, and is left as it is: --backup replaces it and keeps {}, \
                     --overwrite replaces it and keeps nothing",
                    path.display(),
                    beside(path, ".bak").display()
                )
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

    /// The template with every key line uncommented but those in `skip`, and but the bare
    /// `[freq]` header, which carries no values since a host's own are the only right ones.
    fn all_set(skip: &[&str]) -> String {
        let mut out = String::new();
        let mut in_fence = false;
        for line in TEMPLATE.lines() {
            if line.starts_with("```") {
                in_fence = !in_fence && line.starts_with("```toml");
            }
            match uncommented(line) {
                Some(body)
                    if in_fence
                        && !key_name(body).is_some_and(|k| skip.contains(&k))
                        && body != "[freq]" =>
                {
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
            trim_runs,
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
        } = parse(&all_set(&["total_duration", "record_file"]));
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
        assert_eq!(trim_runs, Some(crate::series::Trim::DEFAULT));
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
        // The header alone, for `setup-freq` and `--from` to fill.
        assert!(freq.is_none() && TEMPLATE.contains("\n#[freq]\n"));
        // The other duration and the other record mode, which a file may not set beside the
        // first.
        assert_eq!(total_duration, None);
        assert!(
            parse(&all_set(&["duration", "record_file"]))
                .total_duration
                .is_some()
        );
        assert!(matches!(record, Some(crate::record::Target::Dir(_))));
        assert!(matches!(
            parse(&all_set(&["record_dir", "total_duration"])).record,
            Some(crate::record::Target::File(_))
        ));
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
        assert!(new.contains("\n#decimals = 1\n"), "got: {new}");
        assert!(new.contains("\nblocks = 10\n"), "got: {new}");
        assert!(!new.contains("ccd ="), "got: {new}");
        // The TOML carrier says the same.
        let as_toml = to_toml(&new);
        assert_eq!(
            config::parse_text(path, &as_toml).unwrap(),
            config::parse_text(path, old_text).unwrap()
        );
        assert!(as_toml.starts_with(TOML_HEAD));
        assert!(
            as_toml.contains("\n# ---- Blocks ----\n\nblocks = 10\n"),
            "got: {as_toml}"
        );
        // No prose: every comment is the head, a heading, or a key.
        assert!(!as_toml.contains("replicates"), "got: {as_toml}");
    }

    #[test]
    fn the_lines_values_go_over_the_files() {
        let old: toml::Table = toml::from_str(
            "total_duration = 60\nblocks = 100\n[tags]\nhost = \"a\"\nrun = \"1\"\n",
        )
        .unwrap();
        let line: toml::Table =
            toml::from_str("duration = 0.5\npin_freq = \"pin_mhz\"\n[tags]\nrun = \"2\"\n")
                .unwrap();
        let c = parse(&render(Some(&merged(old, line)), None).unwrap());
        assert_eq!((c.duration, c.total_duration), (Some(0.5), None));
        assert_eq!(c.blocks, Some(100));
        assert_eq!(c.pin_freq, Some(PinFreq::PinMhz));
        assert_eq!(
            (c.tags["host"].as_str(), c.tags["run"].as_str()),
            ("a", "2")
        );
        // The line alone, no file, and a bad value stopped before anything is written.
        let line: toml::Table = toml::from_str("blocks = 10\n").unwrap();
        assert_eq!(parse(&filled(None, line).unwrap()).blocks, Some(10));
        let bad: toml::Table = toml::from_str("run_sleep = \"soon\"\n").unwrap();
        assert!(filled(None, bad).unwrap_err().contains("run_sleep"));
        assert_eq!(filled(None, toml::Table::new()).unwrap(), TEMPLATE);
        // The record modes are one choice as the durations are.
        let old: toml::Table = toml::from_str("record_dir = \"runs\"\n").unwrap();
        let line: toml::Table = toml::from_str("record_file = \"r.jsonl\"\n").unwrap();
        let c = parse(&filled(Some(old), line).unwrap());
        assert_eq!(
            c.record,
            Some(crate::record::Target::File("r.jsonl".into()))
        );
    }

    /// A host shaped like the 3900X, `kernel` and `rustc` as given.
    fn host(name: &str, kernel: &str, rustc: &str) -> crate::host::Host {
        crate::host::Host {
            name: name.to_string(),
            cpu_model: Some("AMD Ryzen 9 3900X 12-Core Processor".to_string()),
            ram_bytes: None,
            cache_line_bytes: Some(64),
            caches: Vec::new(),
            kernel: Some(kernel.to_string()),
            rustc: rustc.to_string(),
        }
    }

    /// A recorded run of `series` whose config is `run`, pinned to `pins` at `placement`.
    fn recorded(
        series: &str,
        run: &str,
        pins: &[usize],
        placement: Option<&str>,
    ) -> crate::record::RecordedRun {
        crate::record::RecordedRun {
            series: Some(series.to_string()),
            bench: "ice-rr-2t".to_string(),
            label: None,
            run: toml::from_str(run).unwrap(),
            pin_cpus: pins.to_vec(),
            pin_placement: placement.map(str::to_string),
            host: host("3900x", "7.2.3", "rustc 1.98.0"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    #[test]
    fn a_record_file_of_several_invocations_needs_a_series() {
        let one = recorded("20260921T154030.304Z", "blocks = 10\n", &[], None);
        let two = recorded("20260921T160000.001Z", "blocks = 20\n", &[], None);
        let same = [one.clone(), one.clone()];
        assert_eq!(chosen(&same, None).unwrap(), &one);
        let both = [one.clone(), two.clone()];
        let err = chosen(&both, None).unwrap_err();
        assert!(
            err.contains("--series") && err.contains("20260921T160000.001Z"),
            "{err}"
        );
        assert_eq!(chosen(&both, Some("20260921T160000.001Z")).unwrap(), &two);
        assert!(chosen(&both, Some("nope")).unwrap_err().contains("nope"));
    }

    #[test]
    fn a_records_host_facts_are_named_for_this_host_or_left_out() {
        let profiles = std::collections::BTreeMap::from([
            ("smt".to_string(), "3,9".to_string()),
            ("ccx".to_string(), "3,2".to_string()),
        ]);
        let here = host("7600x", "7.2.6", "rustc 1.98.0");
        // A pool of numbers at a placement this host declares becomes its profile, and a clock
        // in MHz and a record path are left out, each with its reason.
        let run = recorded(
            "s",
            "blocks = 10\npin_cpus = \"11,23\"\npin_freq = 3801\nrecord_dir = \"runs\"\n",
            &[11, 23],
            Some("SMT"),
        );
        let a = adapt(&run, &profiles, &here);
        let want: toml::Table = toml::from_str("blocks = 10\npin_cpus = \"smt\"\n").unwrap();
        assert_eq!(a.table, want);
        let keys: Vec<&str> = a.left_out.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["pin_freq", "record_dir"]);
        assert!(a.left_out[0].1.contains("3801 MHz"), "{:?}", a.left_out);
        // Only what differs is noted: the kernel, not the cpu or the compiler.
        assert!(a.notes.iter().any(|n| n.starts_with("kernel differs")));
        assert!(
            !a.notes
                .iter()
                .any(|n| n.starts_with("cpu") || n.starts_with("rustc"))
        );
        // A placement this host has no profile for is left out, as is one with no name.
        let x = recorded("s", "pin_cpus = \"0,6\"\n", &[0, 6], Some("x-CCX"));
        let a = adapt(&x, &profiles, &here);
        assert!(a.table.is_empty());
        assert!(
            a.left_out[0].1.contains("no x-ccx profile"),
            "{:?}",
            a.left_out
        );
        // A profile name is already this host's way of saying it.
        let named = recorded(
            "s",
            "pin_cpus = \"smt\"\npin_freq = \"pin_mhz\"\n",
            &[11, 23],
            Some("SMT"),
        );
        let a = adapt(&named, &profiles, &here);
        assert_eq!(a.table, named.run);
        assert!(a.left_out.is_empty());
    }

    #[test]
    fn a_left_out_key_gets_its_reason_above_it_in_either_carrier() {
        let text = annotated(TEMPLATE, "pin_freq", "the record pinned 3801 MHz");
        let c = parse(&text);
        assert_eq!(c, parse(TEMPLATE), "a comment sets nothing");
        let at = text
            .find("# the record pinned 3801 MHz\n")
            .expect("the note is there");
        assert!(text[at..].lines().nth(1).unwrap().starts_with("#pin_freq"));
        assert!(to_toml(&text).contains("# the record pinned 3801 MHz\n"));
    }

    #[test]
    fn a_runs_table_is_its_files_keys_and_the_lines_over_them() {
        let dir = std::env::temp_dir().join(format!("iiac-perf-run-table-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("iiac-perf.toml");
        std::fs::write(
            &file,
            "blocks = 10\npin_cpus = \"smt\"\nrecord_dir = \"runs\"\n[profiles]\nsmt = \"0,12\"\n",
        )
        .unwrap();
        let line: toml::Table = toml::from_str("blocks = 20\nrecord_file = \"r.jsonl\"\n").unwrap();
        let run = run_table(&[file], &line).unwrap();
        let want: toml::Table =
            toml::from_str("blocks = 20\npin_cpus = \"smt\"\nrecord_file = \"r.jsonl\"\n").unwrap();
        assert_eq!(run, want);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_hosts_run_keys_are_the_start_and_its_facts_are_not() {
        let dir = std::env::temp_dir().join(format!("iiac-perf-host-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let xdg = dir.join("config.toml");
        let local = dir.join("iiac-perf.toml");
        std::fs::write(
            &xdg,
            "blocks = 50\nduration = 5\npin_freq = 3801\n[profiles]\nsmt = \"0,12\"\n\
             [freq]\ngovernor = \"powersave\"\n",
        )
        .unwrap();
        std::fs::write(
            &local,
            "blocks = 100\nblock_warmup = \"2ms\"\ntotal_duration = 60\npin_freq = \"no\"\n",
        )
        .unwrap();
        let line: toml::Table = toml::from_str("block_warmup = \"1ms\"\nruns = 2\n").unwrap();
        let (start, taken) = host_start(&[xdg, local], &line).unwrap();
        let c = parse(&filled(Some(start), line).unwrap());
        // The nearer file wins, its duration choice clears the other, and its "no" leaves the
        // lower file's pin standing, as the loader has them.
        assert_eq!(
            (c.blocks, c.duration, c.total_duration),
            (Some(100), None, Some(60.0))
        );
        assert_eq!(c.pin_freq, Some(PinFreq::Mhz(3801)));
        assert_eq!((c.block_warmup, c.runs), (Some(0.001), Some(2)));
        assert!(c.freq.is_none() && c.profiles.is_empty());
        // Each file is named with what came from it, and nothing the line or a nearer file set.
        assert_eq!(taken.len(), 2, "got: {taken:?}");
        assert!(
            taken[0].ends_with("config.toml: pin_freq"),
            "got: {taken:?}"
        );
        assert!(
            taken[1].ends_with("iiac-perf.toml: blocks, total_duration"),
            "got: {taken:?}"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn update_rewrites_in_place_and_keeps_a_backup_only_when_asked() {
        let dir = std::env::temp_dir().join(format!("iiac-perf-update-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("queue.toml");
        let mine = "# my note\nblocks = 100\nruns = 3\n";
        std::fs::write(&file, mine).unwrap();
        let line: toml::Table = toml::from_str("blocks = 20\n").unwrap();

        let notes = updated(&file, false, line.clone()).unwrap();
        assert!(notes[0].contains("not carried over"), "got: {notes:?}");
        assert_eq!(notes.len(), 2, "got: {notes:?}");
        let c = config::parse_text(&file, &std::fs::read_to_string(&file).unwrap()).unwrap();
        assert_eq!((c.blocks, c.runs), (Some(20), Some(3)));
        assert!(!beside(&file, ".bak").exists() && !beside(&file, ".new").exists());

        // A file that is already the template's own text loses nothing, so no note.
        let notes = updated(&file, false, toml::Table::new()).unwrap();
        assert_eq!(notes.len(), 1, "got: {notes:?}");

        let second = std::fs::read_to_string(&file).unwrap();
        updated(&file, true, line).unwrap();
        assert_eq!(
            std::fs::read_to_string(beside(&file, ".bak")).unwrap(),
            second
        );

        // A stale key, a bad value, and a missing file each leave the file as it was.
        std::fs::write(&file, "bogus = 1\n").unwrap();
        assert!(updated(&file, false, toml::Table::new()).is_err());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "bogus = 1\n");
        std::fs::write(&file, "blocks = 5\n").unwrap();
        let bad: toml::Table = toml::from_str("run_sleep = \"soon\"\n").unwrap();
        assert!(updated(&file, false, bad).is_err());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "blocks = 5\n");
        assert!(updated(&dir.join("absent.md"), false, toml::Table::new()).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
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
        // Asked to, it is replaced, and the old file kept only with a backup.
        assert!(replace(&path, "second\n", false).unwrap().is_empty());
        assert!(!beside(&path, ".bak").exists());
        assert_eq!(replace(&path, "third\n", true).unwrap().len(), 1);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "third\n");
        assert_eq!(
            std::fs::read_to_string(beside(&path, ".bak")).unwrap(),
            "second\n"
        );
        // A backup of nothing is nothing, not an error.
        let fresh = dir.join("fresh.md");
        assert!(replace(&fresh, "new\n", true).unwrap().is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
