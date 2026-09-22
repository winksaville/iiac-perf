//! Per-run JSONL records: the `--record-dir` / `--record-file` side channel that outlives the
//! session.
//!
//! The report prints and is gone, so no run's numbers survive the terminal that showed them.
//! This module appends one self-describing JSON object per finished harness run, alongside the
//! unchanged display, so a run can be re-analysed without its session.
//!
//! - **One object per bench result**, not per process: `all` emits one record per bench, each
//!   carrying the host / policy / clock stamp of its own run.
//! - **JSONL, one object per line**: `jq -s .` makes an array on demand, an interrupted run
//!   still parses, and files concatenate with `cat`.
//! - **A file is an invocation or more, never less**: a named file takes every record sent to
//!   it, and a directory gets one file per invocation, named by the series id its runs share.
//! - **The open is append-and-create, never truncate**, in both path modes: the no-truncate
//!   invariant is what protects existing evidence, whoever chose the file name.
//! - **The record documents its own fields**: [`FIELD_DOCS`] is the dictionary the
//!   `describe-record` command prints, and a test fails on any record key with no entry.
//! - **Absent is not zero**: every field a box may not expose serializes as `null` rather than
//!   a default, so the key set is identical in every record.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::freq::{self, PolicyField};
use crate::gauge::Settle;
use crate::harness::{BlockSummary, PS_PER_NS, RunCfg, RunOutput, WarmExit};
use crate::host::{self, Host};
use crate::run_config::{Param, Source};

/// Layout version stamped into every record, bumped on any change to a field's name, unit, or
/// meaning, so a dictionary printed by today's binary can be checked against a record written
/// by an older one. What each bump did is in [`SCHEMA_HISTORY`].
pub const SCHEMA_VERSION: u32 = 8;

/// What each schema bump changed, newest first, so a reader holding an older record knows
/// what its keys became. Printed by `describe-record` under the dictionary.
pub const SCHEMA_HISTORY: &[(u32, &str)] = &[
    (
        8,
        "config.run and pin_placement added: the run's keys as a config file spells them, the \
         files' layered with the line's flags over them, and the pool's placement label, so a \
         record reruns from its own config and another host can form its own pair, and series \
         is the UTC start to the millisecond, 20260921T154030.304Z, where it ended in the \
         parent's pid",
    ),
    (
        7,
        "series and run added: every bench runs in a child process, runs times, so a record \
         names the invocation it belongs to and its run among its bench's runs, and run_index \
         is 0 in every record a bench child writes",
    ),
    (
        6,
        "config added: the config files loaded, and every run parameter's value and source as \
         the report's Config: list prints them",
    ),
    (
        5,
        "batches became blocks: batch_mean_ns, batch_samples, batch_agg are block_mean_ns, \
         block_samples, block_agg, resolution_batches is resolution_blocks, blocks_cut and \
         measured_s are added, mean_ns is count-weighted over the blocks, and blocks, block_sleep_*, \
         block_warmup_s are never null since every run has blocks",
    ),
    (
        4,
        "host is a block of the box's identity, where it was the hostname",
    ),
    (
        3,
        "batch_mean_ns, batch_samples, batch_agg, and the resolution_* keys added",
    ),
    (
        2,
        "block_sleep_min_s, block_sleep_max_s, block_warmup_s added",
    ),
    (1, "the first record"),
];

/// The fixed quantile ladder (percentages), identical in every record. Fixed rather than the
/// report's populated bands, whose labels move with the data: a constant ladder is what makes
/// an A/B estimator question answerable after the fact, cheap now and impossible to backfill.
pub const QUANTILE_PCTS: [f64; 13] = [
    0.01, 0.1, 1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0, 99.9, 99.99,
];

/// Where records go, each mode named by its own flag, `--record-dir` or `--record-file`, since a
/// mode picked by a path's trailing `/` let `record = "smooth-records"` append every session to
/// one oddly named file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    /// One file per invocation inside this directory, named `<label>-<series>-<host>.jsonl`, so
    /// a rerun can never clobber an earlier one's evidence (a fixed name is exactly what killed
    /// the powersave series). Every run of the invocation appends to it: the runs are fresh
    /// processes and share the series id, so each arrives at the same name on its own.
    Dir(PathBuf),
    /// Every record appends to this one file.
    File(PathBuf),
}

impl Target {
    /// The target with its path made absolute, as given when that fails, which still resolves
    /// in a child, since a child inherits its parent's directory.
    pub fn absolute(&self) -> Target {
        let abs = |path: &Path| match std::path::absolute(path) {
            Ok(abs) => abs,
            Err(_) => path.to_path_buf(),
        };
        match self {
            Target::Dir(dir) => Target::Dir(abs(dir)),
            Target::File(file) => Target::File(abs(file)),
        }
    }

    /// The target as the `Config:` list prints it: the mode it took, then the path.
    pub fn describe(&self) -> String {
        match self {
            Target::Dir(dir) => format!(
                "a file per invocation in {}",
                crate::run_config::display_path(dir)
            ),
            Target::File(file) => format!("appended to {}", crate::run_config::display_path(file)),
        }
    }
}

/// The resolved record sink: targets, verbatim tags, and the host stamp, built once at
/// startup so a bad path fails before any bench spends minutes measuring. A bench child writes
/// the same record to two targets, its parent's result file and the record target.
#[derive(Debug)]
pub struct Recorder {
    targets: Vec<Target>,
    stamp: Stamp,
    /// This sink's own invocation id, naming a directory's file when no series was given, as
    /// when `suggest-freq` records in-process.
    own_id: String,
}

/// What a parent reads back from a child's record: the run's identity and the numbers the
/// across-process summary is built from.
#[derive(Debug, Clone, PartialEq)]
pub struct RunSummary {
    /// The bench's registered name.
    pub bench: String,
    /// The child's process id.
    pub pid: u32,
    /// The run's count-weighted mean over its blocks, ns.
    pub mean_ns: f64,
    /// The sample stdev of the run's block means, ns, `None` below two blocks.
    pub block_stdev_ns: Option<f64>,
    /// The run's resolution, the drift floor of its block curve, ns.
    pub resolution_ns: Option<f64>,
    /// The lowest and highest delivered clock the run's dominant core read at its block seams,
    /// GHz, `None` when the host exposes no readable clock.
    pub clock_ghz: Option<(f64, f64)>,
}

/// Read every record in a JSONL file as a [`RunSummary`], in file order. A missing file is an
/// empty list, since a probe bench records nothing.
pub fn read_summaries(path: &Path) -> Result<Vec<RunSummary>, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("reading {}: {e}", path.display())),
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let r: Record = serde_json::from_str(line)
                .map_err(|e| format!("parsing a record in {}: {e}", path.display()))?;
            let clock: Vec<Option<freq::FreqSample>> = r
                .clock_cpu
                .iter()
                .zip(&r.clock_khz)
                .map(|(&cpu, &khz)| Some(freq::FreqSample { cpu, khz }))
                .collect();
            Ok(RunSummary {
                bench: r.bench,
                pid: r.pid,
                mean_ns: r.mean_ns,
                block_stdev_ns: crate::series::Series::of(&r.block_mean_ns).map(|s| s.stdev),
                resolution_ns: r.resolution_ns,
                clock_ghz: crate::gauge::clock_profile(&clock).map(|p| (p.min_ghz, p.max_ghz)),
            })
        })
        .collect()
}

/// What `init-config --from-record` reads of one record: the run's config and what the config
/// was a config of, the host and the pool.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedRun {
    /// The invocation's series id, `None` outside a bench child.
    pub series: Option<String>,
    /// The bench the record measured.
    pub bench: String,
    /// The `label` tag, when one was set.
    pub label: Option<String>,
    /// The run's keys as a config file spells them, [`RecordConfig::run`].
    pub run: toml::Table,
    /// The CPU pool the run drew from, empty when unpinned.
    pub pin_cpus: Vec<usize>,
    /// The pool's placement label, `SMT` and the like.
    pub pin_placement: Option<String>,
    /// The host that wrote the record.
    pub host: Host,
    /// The iiac-perf version that wrote it.
    pub version: String,
}

/// Read every record in a JSONL file as a [`RecordedRun`], in file order. A record before schema
/// 8 carries no `config.run`, so it is refused by its line, where a record that merely lacked the
/// field would read as an empty config.
pub fn read_runs(path: &Path) -> Result<Vec<RecordedRun>, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let mut runs = Vec::new();
    for (at, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let at = at + 1;
        let value: serde_json::Value =
            serde_json::from_str(line).map_err(|e| format!("{} line {at}: {e}", path.display()))?;
        // OK: a record with no schema_version reads as schema 0, too old to carry a config.
        let schema = value["schema_version"].as_u64().unwrap_or_default();
        if schema < 8 {
            return Err(format!(
                "{} line {at}: a schema {schema} record carries no config.run, which schema 8 \
                 added",
                path.display()
            ));
        }
        let r: Record = serde_json::from_value(value)
            .map_err(|e| format!("{} line {at}: {e}", path.display()))?;
        runs.push(RecordedRun {
            series: r.series,
            bench: r.bench,
            label: r.tags.get(LABEL_TAG).cloned(),
            run: r.config.run,
            pin_cpus: r.pin_cpus,
            pin_placement: r.pin_placement,
            host: r.host,
            version: r.version,
        });
    }
    if runs.is_empty() {
        return Err(format!("{} holds no record", path.display()));
    }
    Ok(runs)
}

/// What `analyze` reads of one record: the run's place, its tags, and the numbers the
/// cross-invocation statistics are built from.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalyzedRun {
    /// The invocation's series id.
    pub series: String,
    /// The run's 1-based number among its bench's runs.
    pub run: u64,
    /// The bench it measured.
    pub bench: String,
    /// The host that wrote it, by name.
    pub host: String,
    /// Its tags, verbatim.
    pub tags: BTreeMap<String, String>,
    /// The wall-clock UTC start of its measured stretch, RFC3339 to the millisecond.
    pub t_start: String,
    /// Every run parameter's value, as the `Config:` list printed it.
    pub params: BTreeMap<String, String>,
    /// The run's count-weighted mean, ns.
    pub mean_ns: f64,
    /// The run's block means, ns, in run order.
    pub block_mean_ns: Vec<f64>,
    /// The delivered clock at each block seam, kHz, empty when unreadable.
    pub clock_khz: Vec<u64>,
}

/// What reading skipped, so a count stands where a record did not.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Skipped {
    /// Lines that did not parse as a record of schema 6 or later, a broken last line from a
    /// crash mid-append among them.
    pub unreadable: usize,
    /// Records with no series, from before schema 7 or from a command that records in-process.
    pub no_series: usize,
}

/// Read every record in a JSONL file as an [`AnalyzedRun`], in file order, adding to `skipped`
/// what could not be one. Only a file that cannot be read is an error.
pub fn read_analyzed(path: &Path, skipped: &mut Skipped) -> Result<Vec<AnalyzedRun>, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let mut runs = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let Ok(r) = serde_json::from_str::<Record>(line) else {
            skipped.unreadable += 1;
            continue;
        };
        let (Some(series), Some(run)) = (r.series, r.run) else {
            skipped.no_series += 1;
            continue;
        };
        runs.push(AnalyzedRun {
            series,
            run,
            bench: r.bench,
            host: r.host.name,
            tags: r.tags,
            t_start: r.t_start,
            params: r
                .config
                .params
                .into_iter()
                .map(|(k, p)| (k, p.value))
                .collect(),
            mean_ns: r.mean_ns,
            block_mean_ns: r.block_mean_ns,
            clock_khz: r.clock_khz,
        });
    }
    Ok(runs)
}

/// What every record of one process carries unchanged: the host, the tags, the run's
/// configuration, and in a bench child the series and run it belongs to.
#[derive(Debug)]
struct Stamp {
    host: Host,
    tags: BTreeMap<String, String>,
    config: RecordConfig,
    series: Option<SeriesRun>,
}

/// Which invocation a record belongs to and which of its bench's runs it is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesRun {
    /// The invocation's id, shared by every run of every bench it spawned.
    pub id: String,
    /// The run's 1-based number among its bench's runs.
    pub run: u64,
}

/// A new invocation's series id: the UTC start to the millisecond, `20260921T154030.304Z`, fixed
/// length and sorting in time order. Two invocations share one only by starting in the same
/// millisecond on one host, a parallel launch, which is already a broken measurement, and the
/// append-only open loses nothing even then. A parent pid, which this replaced, only varied the
/// length.
pub fn new_series_id() -> String {
    basic_stamp(std::time::SystemTime::now())
}

/// The run's configuration as the record carries it: the files loaded and every run parameter.
/// Values and sources, not file hashes, since a hash moves with an edited comment while the
/// run it configures does not.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordConfig {
    /// Every config file loaded, in load order, the later winning.
    files: Vec<String>,
    /// Every run parameter by name, as the `Config:` list prints it.
    params: BTreeMap<String, RecordParam>,
    /// The run's keys as a config file spells them and the loader reads them: the files' run
    /// keys layered as `init-config` layers them, the line's flags over them. A key absent is
    /// the writing version's default. Empty in a record from before schema 8.
    #[serde(default)]
    run: toml::Table,
}

/// One run parameter in the record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RecordParam {
    /// The resolved value, rendered as the report prints it.
    value: String,
    /// `default`, the file's path as loaded, or the flag as typed.
    source: String,
    /// A file or flag set the value to what the default would have given.
    same_as_default: bool,
}

impl RecordConfig {
    /// The record's form of the loaded files and the resolved parameters, file paths made
    /// absolute, since the project-local file loads relative to a directory the record does not
    /// otherwise name.
    pub fn new(files: &[PathBuf], params: &[Param]) -> RecordConfig {
        let mut map = BTreeMap::new();
        for p in params {
            let source = match &p.source {
                Source::Default => "default".to_string(),
                Source::File(path) => absolute(path),
                Source::Flag(flag) => flag.clone(),
            };
            map.insert(
                p.key.to_string(),
                RecordParam {
                    value: p.value.clone(),
                    source,
                    same_as_default: p.same_as_default,
                },
            );
        }
        RecordConfig {
            files: files.iter().map(|f| absolute(f)).collect(),
            params: map,
            run: toml::Table::new(),
        }
    }

    /// The config with `run` as its loadable form, [`RecordConfig::run`].
    pub fn with_run(mut self, run: toml::Table) -> RecordConfig {
        self.run = run;
        self
    }
}

/// `path` made absolute against the current directory, as loaded when that fails.
fn absolute(path: &Path) -> String {
    match std::path::absolute(path) {
        Ok(abs) => abs.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

/// One JSONL record: everything a re-analysis needs without the session that produced it.
/// Field meanings live in [`FIELD_DOCS`], the single dictionary a test keeps honest.
///
/// Every field is owned, so the struct that writes a record is the struct that reads one back:
/// a borrowed field would save a clone per record and rule out `Deserialize`, since serde cannot
/// borrow a slice or a map from JSON.
#[derive(Serialize, Deserialize)]
struct Record {
    schema_version: u32,
    version: String,
    t_start: String,
    utc_offset_s: Option<i64>,
    host: Host,
    pid: u32,
    run_index: u32,
    series: Option<String>,
    run: Option<u64>,
    bench: String,
    tags: BTreeMap<String, String>,
    config: RecordConfig,
    pin_cpus: Vec<usize>,
    pin_placement: Option<String>,
    duration_s: f64,
    measured_s: f64,
    suspended_s: f64,
    warm_exit: String,
    warm_used_s: f64,
    warm_budget_s: f64,
    settle_s: Option<f64>,
    settle_ghz: Option<f64>,
    samples: u64,
    inner: u64,
    calls: u64,
    min_ns: f64,
    mean_ns: f64,
    stdev_ns: f64,
    max_ns: f64,
    quantile_pcts: Vec<f64>,
    quantile_ns: Vec<f64>,
    blocks: u64,
    blocks_cut: u64,
    block_sleep_min_s: f64,
    block_sleep_max_s: f64,
    block_warmup_s: f64,
    block_mean_ns: Vec<f64>,
    block_samples: Vec<u64>,
    block_agg: u64,
    block_ci95_ns: Option<f64>,
    block_lsc_ns: Option<f64>,
    resolution_ns: Option<f64>,
    resolution_blocks: Option<u64>,
    resolution_groups: Option<u64>,
    clock_t_ns: Vec<u64>,
    clock_cpu: Vec<usize>,
    clock_khz: Vec<u64>,
    driver: Option<PolicyField>,
    governor: Option<PolicyField>,
    epp: Option<PolicyField>,
    boost: Option<PolicyField>,
    scaling_min_freq: Option<PolicyField>,
    scaling_max_freq: Option<PolicyField>,
}

/// One field's dictionary entry: name, unit, one-line meaning.
pub struct FieldDoc {
    /// The record key, verbatim.
    pub name: &'static str,
    /// The value's unit, `-` for unitless.
    pub unit: &'static str,
    /// One line saying what the value means.
    pub meaning: &'static str,
}

/// The field dictionary `describe-record` prints: one entry per record key, in record order.
/// A test serializes a sample record and fails on any key missing here (or entry missing
/// there), so the dictionary cannot drift from the data.
pub const FIELD_DOCS: &[FieldDoc] = &[
    FieldDoc {
        name: "schema_version",
        unit: "-",
        meaning: "record layout version, checked against describe-record's own before trusting meanings",
    },
    FieldDoc {
        name: "version",
        unit: "-",
        meaning: "iiac-perf version that wrote the record",
    },
    FieldDoc {
        name: "t_start",
        unit: "RFC3339",
        meaning: "wall-clock UTC start of the measured stretch (warmup excluded), millisecond precision",
    },
    FieldDoc {
        name: "utc_offset_s",
        unit: "s",
        meaning: "the writing box's local-time offset from UTC, null when unresolvable",
    },
    FieldDoc {
        name: "host.name",
        unit: "-",
        meaning: "hostname of the box that ran the bench",
    },
    FieldDoc {
        name: "host.cpu_model",
        unit: "-",
        meaning: "the CPU's model name from /proc/cpuinfo, null when unreadable",
    },
    FieldDoc {
        name: "host.ram_bytes",
        unit: "B",
        meaning: "MemTotal from /proc/meminfo, what the kernel has, null when unreadable",
    },
    FieldDoc {
        name: "host.cache_line_bytes",
        unit: "B",
        meaning: "the L1 data cache's line size from sysfs, null when unreadable",
    },
    FieldDoc {
        name: "host.caches[].level",
        unit: "-",
        meaning: "one entry per sysfs cache index of CPU 0, in index order: its level (1, 2, 3)",
    },
    FieldDoc {
        name: "host.caches[].type",
        unit: "-",
        meaning: "the entry's kind: Data, Instruction, or Unified",
    },
    FieldDoc {
        name: "host.caches[].size_bytes",
        unit: "B",
        meaning: "the entry's size",
    },
    FieldDoc {
        name: "host.caches[].shared_cpus",
        unit: "-",
        meaning: "the entry's shared_cpu_list verbatim: L1's names the SMT siblings, L3's the CCX",
    },
    FieldDoc {
        name: "host.kernel",
        unit: "-",
        meaning: "the kernel release from uname, null when the call fails",
    },
    FieldDoc {
        name: "host.rustc",
        unit: "-",
        meaning: "the compiler that built the writing binary, baked in at build time",
    },
    FieldDoc {
        name: "pid",
        unit: "-",
        meaning: "process id, which with run_index orders records sharing a timestamp",
    },
    FieldDoc {
        name: "run_index",
        unit: "-",
        meaning: "0-based index of this record within its process, 0 in a bench child, which runs one bench",
    },
    FieldDoc {
        name: "series",
        unit: "-",
        meaning: "the invocation's id, its UTC start to the millisecond, shared by every run it spawned, null outside a bench child",
    },
    FieldDoc {
        name: "run",
        unit: "-",
        meaning: "1-based number of this run among its bench's runs in the series, null outside a bench child",
    },
    FieldDoc {
        name: "bench",
        unit: "-",
        meaning: "bench id as the CLI names it",
    },
    FieldDoc {
        name: "tags",
        unit: "-",
        meaning: "verbatim --tag key=value pairs, recorded and never interpreted",
    },
    FieldDoc {
        name: "config.files",
        unit: "-",
        meaning: "the config files loaded as absolute paths, in load order, the later winning, empty when none",
    },
    FieldDoc {
        name: "config.params",
        unit: "-",
        meaning: "every run parameter by name as {value, source, same_as_default}: the Config: list, freq the declared [freq] table, source default | a file | a flag",
    },
    FieldDoc {
        name: "config.run",
        unit: "-",
        meaning: "the run's keys as a config file spells them, the files' layered and the line's flags over them, a key absent being the version's default",
    },
    FieldDoc {
        name: "pin_cpus",
        unit: "-",
        meaning: "the --pin-cpus CPU pool the run's threads drew from, empty means unpinned",
    },
    FieldDoc {
        name: "pin_placement",
        unit: "-",
        meaning: "the pool's placement from its first CPU's topology: core | SMT | CCX | x-CCX, null when unpinned or unreadable",
    },
    FieldDoc {
        name: "duration_s",
        unit: "s",
        meaning: "wall time of the run, block sleeps and warmups included",
    },
    FieldDoc {
        name: "measured_s",
        unit: "s",
        meaning: "seconds inside blocks recording samples, duration_s less the sleeps and block warmups",
    },
    FieldDoc {
        name: "suspended_s",
        unit: "s",
        meaning: "seconds the system spent suspended mid-run, poisoning max/mean/stdev when non-trivial",
    },
    FieldDoc {
        name: "warm_exit",
        unit: "-",
        meaning: "how the warm stretch ended: settled | unstable | uncertified",
    },
    FieldDoc {
        name: "warm_used_s",
        unit: "s",
        meaning: "wall seconds spent warming (process warm when this run ran it, plus the capped stretch)",
    },
    FieldDoc {
        name: "warm_budget_s",
        unit: "s",
        meaning: "the run's total warm allowance (settle budget plus cap)",
    },
    FieldDoc {
        name: "settle_s",
        unit: "s",
        meaning: "when the warm stretch settled, from warmup start, null when it never did",
    },
    FieldDoc {
        name: "settle_ghz",
        unit: "GHz",
        meaning: "median delivered clock of the settled suffix, null when unreadable",
    },
    FieldDoc {
        name: "samples",
        unit: "-",
        meaning: "timed samples recorded, the header's samples=",
    },
    FieldDoc {
        name: "inner",
        unit: "-",
        meaning: "calls per sample: each sample records the mean of this many back-to-back calls",
    },
    FieldDoc {
        name: "calls",
        unit: "-",
        meaning: "samples x inner: bench steps measured in total",
    },
    FieldDoc {
        name: "min_ns",
        unit: "ns",
        meaning: "fastest per-call sample",
    },
    FieldDoc {
        name: "mean_ns",
        unit: "ns",
        meaning: "per-call mean, block_mean_ns weighted by block_samples and so exact, tail included (see suspended_s for when it lies)",
    },
    FieldDoc {
        name: "stdev_ns",
        unit: "ns",
        meaning: "whole-histogram per-call standard deviation, tail included",
    },
    FieldDoc {
        name: "max_ns",
        unit: "ns",
        meaning: "slowest per-call sample",
    },
    FieldDoc {
        name: "quantile_pcts",
        unit: "%",
        meaning: "the fixed quantile ladder, identical in every record",
    },
    FieldDoc {
        name: "quantile_ns",
        unit: "ns",
        meaning: "per-call value at each quantile_pcts entry, same order",
    },
    FieldDoc {
        name: "blocks",
        unit: "-",
        meaning: "measurement block count, every block sized to one sample count",
    },
    FieldDoc {
        name: "blocks_cut",
        unit: "-",
        meaning: "blocks the time cap (twice a block's budget share) ended short of their count, 0 when sizing held",
    },
    FieldDoc {
        name: "block_sleep_min_s",
        unit: "s",
        meaning: "lower bound of the per-block sleep span, 0 means sleepless partitions",
    },
    FieldDoc {
        name: "block_sleep_max_s",
        unit: "s",
        meaning: "upper bound of the per-block sleep span, drawn uniformly per block",
    },
    FieldDoc {
        name: "block_warmup_s",
        unit: "s",
        meaning: "unrecorded post-wake warmup per block, 0 records from the first post-wake call",
    },
    FieldDoc {
        name: "block_mean_ns",
        unit: "ns",
        meaning: "per-block mean series in run order, adjacent blocks count-weight merged past the point cap (see block_agg)",
    },
    FieldDoc {
        name: "block_samples",
        unit: "-",
        meaning: "samples behind each block_mean_ns point, same order",
    },
    FieldDoc {
        name: "block_agg",
        unit: "-",
        meaning: "blocks per recorded block point: 1 means verbatim, powers of 2 past the cap",
    },
    FieldDoc {
        name: "block_ci95_ns",
        unit: "ns",
        meaning: "95% confidence half-width on the block-mean average, null when sleepless blocks cannot replicate",
    },
    FieldDoc {
        name: "block_lsc_ns",
        unit: "ns",
        meaning: "least significant change vs an equal-blocks run, null when sleepless blocks cannot replicate",
    },
    FieldDoc {
        name: "resolution_ns",
        unit: "ns",
        meaning: "the resolution claim: the block-curve drift floor, the smallest delta the run honestly resolves, null below two blocks",
    },
    FieldDoc {
        name: "resolution_blocks",
        unit: "-",
        meaning: "blocks per group at the variance curve's floor level, null with resolution_ns",
    },
    FieldDoc {
        name: "resolution_groups",
        unit: "-",
        meaning: "groups at the floor level, the replication behind the claim, null with resolution_ns",
    },
    FieldDoc {
        name: "clock_t_ns",
        unit: "ns",
        meaning: "delivered-clock sample times at block seams, raw integer ns from warmup start",
    },
    FieldDoc {
        name: "clock_cpu",
        unit: "-",
        meaning: "logical CPU of each clock sample, same order as clock_t_ns",
    },
    FieldDoc {
        name: "clock_khz",
        unit: "kHz",
        meaning: "delivered frequency at each clock sample, all three arrays empty when unreadable",
    },
    FieldDoc {
        name: "driver",
        unit: "-",
        meaning: "cpufreq scaling_driver as {value, uniform}, null when not exposed",
    },
    FieldDoc {
        name: "governor",
        unit: "-",
        meaning: "cpufreq scaling_governor as {value, uniform}, uniform false means CPUs disagree",
    },
    FieldDoc {
        name: "epp",
        unit: "-",
        meaning: "energy_performance_preference as {value, uniform}, null outside EPP drivers",
    },
    FieldDoc {
        name: "boost",
        unit: "-",
        meaning: "boost as its raw 1/0 token in {value, uniform}, null when not exposed",
    },
    FieldDoc {
        name: "scaling_min_freq",
        unit: "kHz",
        meaning: "the governor's lower clamp as {value, uniform}, a pin reads min = max here",
    },
    FieldDoc {
        name: "scaling_max_freq",
        unit: "kHz",
        meaning: "the governor's upper clamp as {value, uniform}",
    },
];

/// Print the field dictionary: the `describe-record` command word. Documents the record's
/// *outputs*, where `--help` documents inputs.
pub fn describe() {
    println!(
        "One JSON object per line per bench run (--record-dir, --record-file), schema_version {SCHEMA_VERSION}. Fields:\n"
    );
    let name_w = FIELD_DOCS
        .iter()
        .map(|f| f.name.len())
        .fold("field".len(), usize::max);
    let unit_w = FIELD_DOCS
        .iter()
        .map(|f| f.unit.len())
        .fold("unit".len(), usize::max);
    println!("  {:<name_w$}  {:<unit_w$}  meaning", "field", "unit");
    for f in FIELD_DOCS {
        println!("  {:<name_w$}  {:<unit_w$}  {}", f.name, f.unit, f.meaning);
    }
    println!("\nSchema history, newest first:\n");
    for (v, what) in SCHEMA_HISTORY {
        println!("  {v}  {what}");
    }
}

/// Append `out`'s record through `cfg`'s sink, a no-op when no record target was given. A record
/// that fails to write dies loudly: the file is the run's evidence, and losing it silently is
/// the failure mode the flag exists to prevent.
pub fn append(bench: &str, out: &RunOutput, cfg: &RunCfg) {
    let Some(rec) = cfg.record else { return };
    if let Err(e) = rec.write(bench, out, cfg) {
        eprintln!("error: record: {e}");
        std::process::exit(1);
    }
}

impl Recorder {
    /// Resolve a record target and the `--tag` list into a sink, validating both now so a bad
    /// argument fails before any bench runs.
    pub fn new(target: Target, tags: &[String], config: RecordConfig) -> Result<Recorder, String> {
        let mut tag_map = BTreeMap::new();
        for tag in tags {
            let Some((k, v)) = tag.split_once('=') else {
                return Err(format!("--tag '{tag}' is not key=value"));
            };
            if k.is_empty() {
                return Err(format!("--tag '{tag}' has an empty key"));
            }
            tag_map.insert(k.to_string(), v.to_string());
        }
        let mut recorder = Recorder {
            targets: Vec::new(),
            stamp: Stamp {
                host: host::probe(),
                tags: tag_map,
                config,
                series: None,
            },
            own_id: new_series_id(),
        };
        recorder.add_target(target)?;
        Ok(recorder)
    }

    /// Stamp every later record with the series and run it belongs to.
    pub fn set_series(&mut self, series: SeriesRun) {
        self.stamp.series = Some(series);
    }

    /// Write every later record to `target` too. The directory a record lands in is created
    /// now, a file target's as a directory target's, since a missing one would otherwise fail
    /// only when the first run finished.
    pub fn add_target(&mut self, target: Target) -> Result<(), String> {
        let dir = match &target {
            Target::Dir(dir) => Some(dir.as_path()),
            Target::File(file) => file.parent().filter(|p| !p.as_os_str().is_empty()),
        };
        if let Some(dir) = dir {
            std::fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
        }
        self.targets.push(target);
        Ok(())
    }

    /// The file a directory target gets: the label, the invocation's series id, and the host,
    /// the label first since the name is the index and `ls` is the query. A file per run, which
    /// a name stamped from each run's own start gave, split one command's output across as many
    /// files as it had runs, and a reader opening one took a tenth of an invocation for the
    /// whole.
    fn dir_file_name(&self) -> String {
        let id = match &self.stamp.series {
            Some(series) => &series.id,
            None => &self.own_id,
        };
        format!(
            "{}-{}-{}.jsonl",
            sanitize(&self.label()),
            sanitize(id),
            sanitize(&self.stamp.host.name)
        )
    }

    /// The records' label: the `label` tag, which `--record-label` sets, or else the bench
    /// selector as typed, read from the `benches` parameter every record carries. So the name
    /// is built from what the data holds, and a renamed file still knows its label.
    fn label(&self) -> String {
        if let Some(label) = self.stamp.tags.get(LABEL_TAG) {
            return label.clone();
        }
        match self.stamp.config.params.get("benches") {
            Some(p) => default_label(&p.value),
            None => "benches".to_string(),
        }
    }

    /// Build one record and append it to every target. The open is append-and-create in both
    /// modes, never truncate.
    fn write(&self, bench: &str, out: &RunOutput, cfg: &RunCfg) -> Result<(), String> {
        let policy = freq::policy();
        let topology = cfg
            .pin_cpus
            .first()
            .and_then(|&cpu| crate::pin::sysfs_topology(cpu));
        let placement = crate::pin::placement_label(cfg.pin_cpus, &topology);
        let record = build_record(
            bench,
            out,
            cfg,
            &self.stamp,
            &policy,
            placement,
            next_index(),
        );
        let line = serde_json::to_string(&record).map_err(|e| format!("serializing: {e}"))?;
        for target in &self.targets {
            let path = match target {
                Target::File(f) => f.clone(),
                Target::Dir(d) => d.join(self.dir_file_name()),
            };
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)
                .map_err(|e| format!("opening {}: {e}", path.display()))?;
            writeln!(file, "{line}").map_err(|e| format!("writing {}: {e}", path.display()))?;
        }
        Ok(())
    }
}

/// The next record's 0-based index within this process: what separates the several records one
/// second of `all` can emit, alongside `pid`.
fn next_index() -> u32 {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Assemble the record from a finished run. Pure with respect to its inputs (the policy, the
/// placement, and the index are passed in), so the dictionary test can drive it without
/// touching sysfs or the process counter.
fn build_record(
    bench: &str,
    out: &RunOutput,
    cfg: &RunCfg,
    stamp: &Stamp,
    policy: &freq::Policy,
    placement: Option<&str>,
    run_index: u32,
) -> Record {
    let (settle_s, settle_ghz) = match out.warm_settle {
        Some(Settle::At { t_s, ghz, .. }) => (Some(t_s), ghz),
        Some(Settle::Never { .. }) | None => (None, None),
    };
    let (block_mean_ns, block_samples, block_agg) = block_series(&out.blocks);
    let mut clock_t_ns = Vec::with_capacity(out.seam_clock.len());
    let mut clock_cpu = Vec::with_capacity(out.seam_clock.len());
    let mut clock_khz = Vec::with_capacity(out.seam_clock.len());
    for s in &out.seam_clock {
        clock_t_ns.push(s.t_ns);
        clock_cpu.push(s.cpu);
        clock_khz.push(s.khz);
    }
    Record {
        schema_version: SCHEMA_VERSION,
        version: env!("CARGO_PKG_VERSION").to_string(),
        t_start: rfc3339_millis(out.wall_start),
        utc_offset_s: utc_offset_s(out.wall_start),
        host: stamp.host.clone(),
        pid: std::process::id(),
        run_index,
        series: stamp.series.as_ref().map(|s| s.id.clone()),
        run: stamp.series.as_ref().map(|s| s.run),
        bench: bench.to_string(),
        tags: stamp.tags.clone(),
        config: stamp.config.clone(),
        pin_cpus: cfg.pin_cpus.to_vec(),
        pin_placement: placement.map(str::to_string),
        duration_s: out.duration_s,
        measured_s: out.measured_s,
        suspended_s: out.suspended_s,
        warm_exit: match out.warm_exit {
            WarmExit::Settled => "settled",
            WarmExit::Unstable => "unstable",
            WarmExit::Uncertified => "uncertified",
        }
        .to_string(),
        warm_used_s: out.warm_used_s,
        warm_budget_s: out.warm_budget_s,
        settle_s,
        settle_ghz,
        samples: out.samples,
        inner: out.inner,
        calls: out.samples * out.inner,
        min_ns: out.hist.min() as f64 / PS_PER_NS,
        mean_ns: out.block_stats.mean_ns,
        stdev_ns: out.hist.stdev() / PS_PER_NS,
        max_ns: out.hist.max() as f64 / PS_PER_NS,
        quantile_pcts: QUANTILE_PCTS.to_vec(),
        quantile_ns: QUANTILE_PCTS
            .iter()
            .map(|pct| out.hist.value_at_quantile(pct / 100.0) as f64 / PS_PER_NS)
            .collect(),
        blocks: out.block_stats.blocks,
        blocks_cut: out.blocks_cut,
        block_sleep_min_s: cfg.block_sleep_s.0,
        block_sleep_max_s: cfg.block_sleep_s.1,
        block_warmup_s: cfg.block_warmup_s,
        block_mean_ns,
        block_samples,
        block_agg,
        block_ci95_ns: out.block_stats.ci95_ns,
        block_lsc_ns: out.block_stats.lsc_ns,
        resolution_ns: out.resolution.as_ref().map(|r| r.floor_ns),
        resolution_blocks: out.resolution.as_ref().map(|r| r.floor_group),
        resolution_groups: out.resolution.as_ref().map(|r| r.floor_groups),
        clock_t_ns,
        clock_cpu,
        clock_khz,
        driver: policy.driver.clone(),
        governor: policy.governor.clone(),
        epp: policy.epp.clone(),
        boost: policy.boost.clone(),
        scaling_min_freq: policy.scaling_min_freq.clone(),
        scaling_max_freq: policy.scaling_max_freq.clone(),
    }
}

/// Cap on recorded block-series points: past it adjacent blocks merge, so a record never
/// grows unbounded with `--blocks` and the resolution curve stays reproducible for group sizes
/// at or above the recorded aggregation.
const MAX_BLOCK_POINTS: usize = 1000;

/// The record's block-mean series: per-point mean (ns) and sample count, plus how many
/// blocks each point aggregates (1 = verbatim, powers of 2 past the cap). Zero-count blocks
/// are dropped, exactly as [`crate::resolution::from_blocks`] drops them.
fn block_series(blocks: &[BlockSummary]) -> (Vec<f64>, Vec<u64>, u64) {
    let usable: Vec<&BlockSummary> = blocks.iter().filter(|b| b.count > 0).collect();
    let mut agg: usize = 1;
    while usable.len().div_ceil(agg) > MAX_BLOCK_POINTS {
        agg *= 2;
    }
    let mut means = Vec::with_capacity(usable.len().div_ceil(agg));
    let mut counts = Vec::with_capacity(means.capacity());
    for chunk in usable.chunks(agg) {
        let count: u64 = chunk.iter().map(|b| b.count).sum();
        let sum: f64 = chunk.iter().map(|b| b.mean_ps * b.count as f64).sum();
        means.push(sum / count as f64 / PS_PER_NS);
        counts.push(count);
    }
    (means, counts, agg as u64)
}

/// The tag `--record-label NAME` sets, and the one a directory target's file name leads with.
pub const LABEL_TAG: &str = "label";

/// Most bench words a default label spells out: past it the list is its count, since a name
/// holding every bench of a long list is no longer read.
const LABEL_WORDS_MAX: usize = 3;

/// The label a selector gives, the `benches` parameter as the `Config:` list prints it: the
/// words joined by `_`, which no bench name holds, or `N-benches` past [`LABEL_WORDS_MAX`].
/// `all` stays `all`, being one word.
fn default_label(benches: &str) -> String {
    let words: Vec<&str> = benches
        .split(',')
        .map(str::trim)
        .filter(|w| !w.is_empty())
        .collect();
    match words.len() {
        0 => "benches".to_string(),
        n if n > LABEL_WORDS_MAX => format!("{n}-benches"),
        _ => words.join("_"),
    }
}

/// Keep a filename component to `[A-Za-z0-9._-]`, mapping anything else to `-`, so a hostname
/// or bench id can never smuggle a separator into the record path.
fn sanitize(part: &str) -> String {
    part.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Seconds and milliseconds since the Unix epoch. A pre-1970 clock (a broken box) reads as the
/// epoch itself: visibly wrong beats a panic mid-run.
fn epoch_parts(t: std::time::SystemTime) -> (i64, u32) {
    match t.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => (d.as_secs() as i64, d.subsec_millis()),
        Err(_) => (0, 0),
    }
}

/// Calendar date for a day count since 1970-01-01 (Howard Hinnant's `civil_from_days`), exact
/// over the whole Gregorian calendar.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// UTC calendar parts of a wall-clock instant: (year, month, day, hour, minute, second,
/// millisecond).
fn utc_parts(t: std::time::SystemTime) -> (i64, u32, u32, u32, u32, u32, u32) {
    let (secs, millis) = epoch_parts(t);
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400) as u32;
    let (y, mo, d) = civil_from_days(days);
    (y, mo, d, sod / 3_600, sod % 3_600 / 60, sod % 60, millis)
}

/// RFC3339 UTC with milliseconds (`2026-08-04T09:32:21.123Z`): the record's `t_start`. Millis
/// are not decoration, since `all` emits several records inside one second.
fn rfc3339_millis(t: std::time::SystemTime) -> String {
    let (y, mo, d, h, mi, s, ms) = utc_parts(t);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{ms:03}Z")
}

/// Basic ISO to the millisecond (`20260804T093221.123Z`) for series ids and dir-mode filenames:
/// no colons, and lexicographic order is chronological order.
fn basic_stamp(t: std::time::SystemTime) -> String {
    let (y, mo, d, h, mi, s, ms) = utc_parts(t);
    format!("{y:04}{mo:02}{d:02}T{h:02}{mi:02}{s:02}.{ms:03}Z")
}

/// The box's local-time offset from UTC at `t` (seconds east), via `localtime_r`, `None` when
/// the lookup fails. Its own field so the UTC `t_start` still reads in the box's local day.
fn utc_offset_s(t: std::time::SystemTime) -> Option<i64> {
    let (secs, _) = epoch_parts(t);
    let secs = secs as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    // SAFETY: localtime_r writes only `tm` and returns null on failure.
    let ok = unsafe { !libc::localtime_r(&secs, &mut tm).is_null() };
    ok.then_some(tm.tm_gmtoff as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bands::BandLabels;
    use crate::harness::{BlockStats, SeamClock};

    /// A populated sample run, block stats and seam clock included, so the serialized record
    /// exercises every key.
    fn sample_output() -> RunOutput {
        let mut hist =
            hdrhistogram::Histogram::<u64>::new_with_bounds(1, crate::harness::HIST_HIGH_PS, 3)
                .expect("constant bounds");
        for v in [20_000u64, 24_000, 30_000, 1_000_000] {
            hist.saturating_record(v);
        }
        RunOutput {
            hist,
            samples: 4,
            inner: 10,
            duration_s: 5.0,
            measured_s: 4.5,
            suspended_s: 0.0,
            block_stats: BlockStats {
                blocks: 2,
                mean_ns: 24.0,
                ci95_ns: Some(1.0),
                lsc_ns: Some(2.0),
            },
            blocks: [23_500.0, 24_500.0]
                .into_iter()
                .map(|mean_ps| BlockSummary {
                    t_start_s: 0.0,
                    t_end_s: 0.05,
                    count: 2,
                    floor_ps: 20_000,
                    floor_q_ps: 20_000,
                    mean_ps,
                    max_ps: 30_000,
                    over_floor: 0,
                })
                .collect(),
            blocks_cut: 1,
            probes: Vec::new(),
            warmup_probes: 0,
            warm_exit: WarmExit::Settled,
            warm_tail: 0,
            warm_clock: None,
            warm_settle: Some(Settle::At {
                t_s: 0.81,
                settled_frac: 0.47,
                start_ghz: Some(3.6),
                ghz: Some(4.35),
                rating: Some(0.001),
            }),
            warm_clock_profile: None,
            warm_used_s: 0.9,
            warm_budget_s: 3.0,
            wall_start: std::time::UNIX_EPOCH + std::time::Duration::from_millis(1_000_000_000_123),
            seam_clock: vec![SeamClock {
                t_ns: 1_500_000_000,
                cpu: 3,
                khz: 4_350_000,
            }],
            resolution: None,
        }
    }

    /// A `RunCfg` for record assembly, where only `pin_cpus` reaches the record.
    fn sample_cfg(pin: &[usize]) -> RunCfg<'_> {
        RunCfg {
            target_seconds: 5.0,
            samples_override: None,
            inner_override: None,
            pin_cpus: pin,
            report_ticks: false,
            seam_probes: true,
            band_labels: BandLabels::Both,
            decimals: 1,
            settle_time_s: 1.5,
            warm_cap_s: 1.5,
            blocks: 2,
            block_sleep_s: (0.001, 0.010),
            block_warmup_s: 0.002,
            record: None,
        }
    }

    /// Serialize the sample record to a JSON object.
    fn sample_value() -> serde_json::Value {
        let out = sample_output();
        let cfg = sample_cfg(&[0, 1]);
        let tags = BTreeMap::from([("series".to_string(), "t1".to_string())]);
        let policy = freq::Policy {
            driver: Some(PolicyField {
                value: "amd-pstate-epp".to_string(),
                uniform: true,
            }),
            governor: Some(PolicyField {
                value: "powersave".to_string(),
                uniform: false,
            }),
            epp: None,
            boost: Some(PolicyField {
                value: "1".to_string(),
                uniform: true,
            }),
            scaling_min_freq: Some(PolicyField {
                value: "550000".to_string(),
                uniform: true,
            }),
            scaling_max_freq: Some(PolicyField {
                value: "4672070".to_string(),
                uniform: true,
            }),
        };
        let host = Host {
            name: "3900x".to_string(),
            cpu_model: Some("AMD Ryzen 9 3900X 12-Core Processor".to_string()),
            ram_bytes: Some(32_767_688 * 1024),
            cache_line_bytes: Some(64),
            caches: vec![
                host::Cache {
                    level: 1,
                    kind: "Data".to_string(),
                    size_bytes: 32 * 1024,
                    shared_cpus: "0,12".to_string(),
                },
                host::Cache {
                    level: 3,
                    kind: "Unified".to_string(),
                    size_bytes: 16384 * 1024,
                    shared_cpus: "0-2,12-14".to_string(),
                },
            ],
            kernel: Some("7.2.3-arch1-2".to_string()),
            rustc: "rustc 1.98.0 (88d9e12ae 2026-08-18)".to_string(),
        };
        let config = RecordConfig::new(
            &[PathBuf::from("/work/iiac-perf.md")],
            &[
                Param::new(
                    "blocks",
                    "2".to_string(),
                    "100",
                    Source::Flag("--blocks".to_string()),
                ),
                Param::new(
                    "block_sleep",
                    "1-10 ms".to_string(),
                    "1-10 ms",
                    Source::File(PathBuf::from("/work/iiac-perf.md")),
                ),
            ],
        )
        .with_run(
            toml::from_str("blocks = 2\npin_cpus = \"smt\"\nblock_sleep = \"1-10ms\"\n")
                .expect("a run table"),
        );
        let stamp = Stamp {
            host,
            tags,
            config,
            series: Some(SeriesRun {
                id: "20260915T120000.123Z".to_string(),
                run: 3,
            }),
        };
        let record = build_record("min-now", &out, &cfg, &stamp, &policy, Some("SMT"), 7);
        serde_json::to_value(&record).expect("record serializes")
    }

    /// Whether `name`, a dotted path with `[]` marking an array of objects, resolves in
    /// `value`: `host.caches[].level` walks into `host`, then the first element of `caches`.
    fn path_exists(value: &serde_json::Value, name: &str) -> bool {
        let mut cur = value;
        for part in name.split('.') {
            let (key, indexed) = match part.strip_suffix("[]") {
                Some(k) => (k, true),
                None => (part, false),
            };
            let Some(next) = cur.get(key) else {
                return false;
            };
            cur = next;
            if indexed {
                let Some(first) = cur.as_array().and_then(|a| a.first()) else {
                    return false;
                };
                cur = first;
            }
        }
        true
    }

    #[test]
    fn every_record_key_is_documented_and_every_doc_names_a_key() {
        let value = sample_value();
        let obj = value.as_object().expect("record is an object");
        // A top-level key is documented by its own entry, or by entries under it, which is how
        // a nested block (`host.name`, `host.caches[].level`) is spelled. A key documented as
        // a whole (`tags`, `driver`) is never walked into.
        let undocumented: Vec<&String> = obj
            .keys()
            .filter(|k| {
                !FIELD_DOCS.iter().any(|f| {
                    f.name == k.as_str()
                        || f.name.starts_with(&format!("{k}."))
                        || f.name.starts_with(&format!("{k}[]"))
                })
            })
            .collect();
        assert!(
            undocumented.is_empty(),
            "undocumented keys: {undocumented:?}"
        );
        let stale: Vec<&str> = FIELD_DOCS
            .iter()
            .map(|f| f.name)
            .filter(|name| !path_exists(&value, name))
            .collect();
        assert!(stale.is_empty(), "docs naming no key: {stale:?}");
    }

    #[test]
    fn schema_history_heads_at_the_current_version() {
        assert_eq!(SCHEMA_HISTORY[0].0, SCHEMA_VERSION);
        let versions: Vec<u32> = SCHEMA_HISTORY.iter().map(|(v, _)| *v).collect();
        let expected: Vec<u32> = (1..=SCHEMA_VERSION).rev().collect();
        assert_eq!(versions, expected, "one entry per version, newest first");
    }

    #[test]
    fn record_round_trips_through_json() {
        let written = sample_value();
        let line = serde_json::to_string(&written).expect("record serializes");
        let read: Record = serde_json::from_str(&line).expect("record deserializes");
        let again = serde_json::to_value(&read).expect("record re-serializes");
        assert_eq!(written, again);
    }

    #[test]
    fn summaries_read_back_every_record_and_a_missing_file_is_empty() {
        let dir = std::env::temp_dir().join(format!("iiac-perf-summaries-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("result.jsonl");
        assert_eq!(read_summaries(&path).unwrap(), Vec::new());
        let line = serde_json::to_string(&sample_value()).unwrap();
        std::fs::write(&path, format!("{line}\n{line}\n")).unwrap();
        let got = read_summaries(&path).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].bench, "min-now");
        assert_eq!(got[0].mean_ns, sample_value()["mean_ns"].as_f64().unwrap());
        // Block means 23.5 and 24.5 ns, one seam clock read at 4.35 GHz on CPU 3.
        let stdev = got[0].block_stdev_ns.expect("two blocks spread");
        assert!((stdev - 0.5f64.sqrt()).abs() < 1e-9, "stdev {stdev}");
        assert_eq!(got[0].resolution_ns, None);
        assert_eq!(got[0].clock_ghz, Some((4.35, 4.35)));
        std::fs::write(&path, "not json\n").unwrap();
        assert!(read_summaries(&path).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_record_from_before_schema_8_still_reads() {
        let mut old = sample_value();
        old["config"]
            .as_object_mut()
            .expect("config is an object")
            .remove("run");
        old.as_object_mut()
            .expect("record is an object")
            .remove("pin_placement");
        let read: Record = serde_json::from_value(old).expect("a schema 7 record reads");
        assert!(read.config.run.is_empty());
        assert_eq!(read.pin_placement, None);
    }

    #[test]
    fn runs_read_back_and_a_record_before_schema_8_is_refused() {
        let dir = std::env::temp_dir().join(format!("iiac-perf-read-runs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("r.jsonl");
        let line = serde_json::to_string(&sample_value()).unwrap();
        std::fs::write(&path, format!("{line}\n\n{line}\n")).unwrap();
        let runs = read_runs(&path).unwrap();
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].series.as_deref(), Some("20260915T120000.123Z"));
        assert_eq!(runs[0].run["pin_cpus"].as_str(), Some("smt"));
        assert_eq!(runs[0].pin_placement.as_deref(), Some("SMT"));
        assert_eq!(runs[0].pin_cpus, [0, 1]);
        let mut old = sample_value();
        old["schema_version"] = serde_json::json!(7);
        std::fs::write(&path, serde_json::to_string(&old).unwrap()).unwrap();
        let err = read_runs(&path).unwrap_err();
        assert!(err.contains("line 1") && err.contains("schema 7"), "{err}");
        std::fs::write(&path, "").unwrap();
        assert!(read_runs(&path).unwrap_err().contains("no record"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn absent_fields_serialize_as_null_not_missing() {
        let value = sample_value();
        // `epp` was None in the sample policy: the key stays, its value is null, so the key
        // set is identical in every record.
        assert!(value.get("epp").is_some_and(serde_json::Value::is_null));
    }

    #[test]
    fn record_carries_the_fixed_ladder_and_matching_values() {
        let value = sample_value();
        let pcts = value["quantile_pcts"].as_array().expect("array");
        let vals = value["quantile_ns"].as_array().expect("array");
        assert_eq!(pcts.len(), QUANTILE_PCTS.len());
        assert_eq!(vals.len(), QUANTILE_PCTS.len());
        assert_eq!(pcts[6], serde_json::json!(50.0));
        // p50 of the 4-sample histogram: 24,000 ps = 24 ns (3-sig-fig bucketing may land
        // within the bucket's width).
        let p50 = vals[6].as_f64().expect("number");
        assert!((p50 - 24.0).abs() < 0.1, "p50 read {p50}");
    }

    #[test]
    fn record_stamps_provenance() {
        let value = sample_value();
        assert_eq!(value["schema_version"], serde_json::json!(SCHEMA_VERSION));
        assert_eq!(value["bench"], serde_json::json!("min-now"));
        assert_eq!(value["host"]["name"], serde_json::json!("3900x"));
        assert_eq!(
            value["host"]["caches"][1]["type"],
            serde_json::json!("Unified")
        );
        assert_eq!(
            value["host"]["caches"][1]["shared_cpus"],
            serde_json::json!("0-2,12-14")
        );
        assert_eq!(value["run_index"], serde_json::json!(7));
        assert_eq!(value["series"], serde_json::json!("20260915T120000.123Z"));
        assert_eq!(value["run"], serde_json::json!(3));
        assert_eq!(
            value["t_start"],
            serde_json::json!("2001-09-09T01:46:40.123Z")
        );
        assert_eq!(value["tags"]["series"], serde_json::json!("t1"));
        assert_eq!(
            value["config"]["files"],
            serde_json::json!(["/work/iiac-perf.md"])
        );
        assert_eq!(
            value["config"]["params"]["blocks"],
            serde_json::json!({"value": "2", "source": "--blocks", "same_as_default": false})
        );
        assert_eq!(
            value["config"]["params"]["block_sleep"],
            serde_json::json!({"value": "1-10 ms", "source": "/work/iiac-perf.md", "same_as_default": true})
        );
        assert_eq!(
            value["config"]["run"],
            serde_json::json!({"blocks": 2, "pin_cpus": "smt", "block_sleep": "1-10ms"})
        );
        assert_eq!(value["pin_placement"], serde_json::json!("SMT"));
        assert_eq!(value["governor"]["uniform"], serde_json::json!(false));
        assert_eq!(value["block_mean_ns"], serde_json::json!([23.5, 24.5]));
        assert_eq!(value["block_samples"], serde_json::json!([2, 2]));
        assert_eq!(value["block_agg"], serde_json::json!(1));
        assert_eq!(value["blocks_cut"], serde_json::json!(1));
        assert!(value["resolution_blocks"].is_null());
        assert_eq!(value["block_sleep_min_s"], serde_json::json!(0.001));
        assert_eq!(value["block_sleep_max_s"], serde_json::json!(0.010));
        assert_eq!(value["block_warmup_s"], serde_json::json!(0.002));
        assert_eq!(value["clock_t_ns"], serde_json::json!([1_500_000_000u64]));
        assert_eq!(value["clock_khz"], serde_json::json!([4_350_000u64]));
    }

    #[test]
    fn civil_from_days_matches_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // The billionth epoch second, a well-known landmark.
        assert_eq!(civil_from_days(1_000_000_000 / 86_400), (2001, 9, 9));
        // A leap day, and the day after.
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
        assert_eq!(civil_from_days(11_017), (2000, 3, 1));
    }

    #[test]
    fn timestamps_format_utc() {
        let t = std::time::UNIX_EPOCH + std::time::Duration::from_millis(1_000_000_000_123);
        assert_eq!(rfc3339_millis(t), "2001-09-09T01:46:40.123Z");
        assert_eq!(basic_stamp(t), "20010909T014640.123Z");
    }

    #[test]
    fn a_file_targets_directory_is_created() {
        let dir =
            std::env::temp_dir().join(format!("iiac-perf-record-parent-{}", std::process::id()));
        let file = dir.join("deeper").join("runs.jsonl");
        Recorder::new(Target::File(file.clone()), &[], RecordConfig::new(&[], &[])).unwrap();
        assert!(file.parent().unwrap().is_dir());
        assert!(!file.exists(), "the file itself waits for a record");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_directory_gets_one_file_per_invocation() {
        let dir = std::env::temp_dir().join("iiac-perf-record-dirfile");
        let config = || RecordConfig::new(&[], &[]);
        let series = |run| SeriesRun {
            id: "20260918T201112.456Z".to_string(),
            run,
        };
        // Two runs are two processes, so two sinks, and they share the invocation's id.
        let mut first = Recorder::new(Target::Dir(dir.clone()), &[], config()).unwrap();
        let mut second = Recorder::new(Target::Dir(dir.clone()), &[], config()).unwrap();
        first.set_series(series(1));
        second.set_series(series(2));
        assert_eq!(first.dir_file_name(), second.dir_file_name());
        let name = first.dir_file_name();
        // No benches parameter and no label tag, as in a bare sink.
        assert!(
            name.starts_with("benches-20260918T201112.456Z-"),
            "got: {name}"
        );
        assert!(name.ends_with(".jsonl"), "got: {name}");
        // Another invocation is another file, so a rerun cannot land on an earlier one's.
        let mut rerun = Recorder::new(Target::Dir(dir.clone()), &[], config()).unwrap();
        rerun.set_series(SeriesRun {
            id: "20260918T201500.789Z".to_string(),
            run: 1,
        });
        assert_ne!(rerun.dir_file_name(), name);
        // With no series, as when a command records in-process, the sink's own id names it.
        let alone = Recorder::new(Target::Dir(dir.clone()), &[], config()).unwrap();
        assert!(alone.dir_file_name().ends_with(".jsonl"));
        assert_ne!(alone.dir_file_name(), name);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_directory_files_name_leads_with_the_label() {
        let dir = std::env::temp_dir().join("iiac-perf-record-label");
        let benches = |value: &str| {
            RecordConfig::new(
                &[],
                &[Param::new(
                    "benches",
                    value.to_string(),
                    "none",
                    Source::Default,
                )],
            )
        };
        let named = |value: &str, tags: &[String]| {
            let mut rec = Recorder::new(Target::Dir(dir.clone()), tags, benches(value)).unwrap();
            rec.set_series(SeriesRun {
                id: "20260920T101010.042Z".to_string(),
                run: 1,
            });
            rec.dir_file_name()
        };
        assert!(named("ice-rr-2t", &[]).starts_with("ice-rr-2t-20260920T101010.042Z-"));
        assert!(named("all", &[]).starts_with("all-20260920T"));
        assert!(named("min-now, std-now", &[]).starts_with("min-now_std-now-20260920T"));
        assert!(named("a, b, c, d", &[]).starts_with("4-benches-20260920T"));
        // The label tag wins, sanitized as the host is.
        let tagged = named("all", &["label=pins run/2".to_string()]);
        assert!(tagged.starts_with("pins-run-2-20260920T"), "got: {tagged}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn tags_must_be_key_value() {
        let dir = std::env::temp_dir().join("iiac-perf-record-test");
        let config = || RecordConfig::new(&[], &[]);
        assert!(
            Recorder::new(Target::Dir(dir.clone()), &["novalue".to_string()], config()).is_err()
        );
        assert!(Recorder::new(Target::Dir(dir.clone()), &["=v".to_string()], config()).is_err());
        let rec = Recorder::new(Target::Dir(dir.clone()), &["k=v=w".to_string()], config())
            .expect("first '=' splits");
        assert_eq!(rec.stamp.tags.get("k").map(String::as_str), Some("v=w"));
    }

    #[test]
    fn block_series_merges_past_the_point_cap() {
        let blocks: Vec<BlockSummary> = (0..2500)
            .map(|_| BlockSummary {
                t_start_s: 0.0,
                t_end_s: 0.05,
                count: 1,
                floor_ps: 20_000,
                floor_q_ps: 20_000,
                mean_ps: 20_000.0,
                max_ps: 20_000,
                over_floor: 0,
            })
            .collect();
        let (means, counts, agg) = block_series(&blocks);
        assert_eq!(agg, 4);
        assert_eq!(means.len(), 625);
        assert_eq!(counts[0], 4);
        assert!((means[0] - 20.0).abs() < 1e-9);
    }

    #[test]
    fn sanitize_keeps_safe_chars_only() {
        assert_eq!(sanitize("rpi5-20cd"), "rpi5-20cd");
        assert_eq!(sanitize("a/b:c d"), "a-b-c-d");
    }
}
