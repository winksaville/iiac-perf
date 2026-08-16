//! Per-run NDJSON records: the `--record` side channel that outlives the session.
//!
//! The report prints and is gone, so no run's numbers survive the terminal that showed them.
//! This module appends one self-describing JSON object per finished harness run, alongside the
//! unchanged display, so a run can be re-analysed without its session.
//!
//! - **One object per bench result**, not per process: `all` emits one record per bench, each
//!   carrying the host / policy / clock stamp of its own run.
//! - **NDJSON, one object per line**: `jq -s .` makes an array on demand, an interrupted run
//!   still parses, and per-run files concatenate with `cat`.
//! - **The open is append-and-create, never truncate**, in both path modes: the no-truncate
//!   invariant is what protects existing evidence, whoever chose the file name.
//! - **The record documents its own fields**: [`FIELD_DOCS`] is the dictionary the
//!   `describe-record` command prints, and a test fails on any record key with no entry.
//! - **Absent is not zero**: every field a box may not expose serializes as `null` rather than
//!   a default, so the key set is identical in every record.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::freq::{self, PolicyField};
use crate::gauge::Settle;
use crate::harness::{PS_PER_NS, RunCfg, RunOutput, WarmExit};

/// Layout version stamped into every record, bumped on any change to a field's name, unit, or
/// meaning, so a dictionary printed by today's binary can be checked against a record written
/// by an older one.
pub const SCHEMA_VERSION: u32 = 1;

/// The fixed quantile ladder (percentages), identical in every record. Fixed rather than the
/// report's populated bands, whose labels move with the data: a constant ladder is what makes
/// an A/B estimator question answerable after the fact, cheap now and impossible to backfill.
pub const QUANTILE_PCTS: [f64; 13] = [
    0.01, 0.1, 1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0, 99.9, 99.99,
];

/// Where records go: resolved once from the `--record` path's shape.
#[derive(Debug, PartialEq, Eq)]
enum Target {
    /// One file per run inside this directory, named `<ts>-<host>-<bench>.ndjson`, so a rerun
    /// can never clobber a run's evidence (a fixed name is exactly what killed the powersave
    /// series).
    Dir(PathBuf),
    /// Every record appends to this one file.
    File(PathBuf),
}

/// The resolved `--record` sink: target, verbatim tags, and the host stamp, built once at
/// startup so a bad path fails before any bench spends minutes measuring.
#[derive(Debug)]
pub struct Recorder {
    target: Target,
    tags: BTreeMap<String, String>,
    host: String,
}

/// One NDJSON record: everything a re-analysis needs without the session that produced it.
/// Field meanings live in [`FIELD_DOCS`], the single dictionary a test keeps honest.
#[derive(Serialize)]
struct Record<'a> {
    schema_version: u32,
    version: &'static str,
    t_start: String,
    utc_offset_s: Option<i64>,
    host: &'a str,
    pid: u32,
    run_index: u32,
    bench: &'a str,
    tags: &'a BTreeMap<String, String>,
    pin_cpus: &'a [usize],
    duration_s: f64,
    suspended_s: f64,
    warm_exit: &'static str,
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
    quantile_pcts: &'static [f64],
    quantile_ns: Vec<f64>,
    blocks: Option<u64>,
    block_mean_ns: Option<&'a [f64]>,
    block_ci95_ns: Option<f64>,
    block_lsc_ns: Option<f64>,
    clock_t_ns: Vec<u64>,
    clock_cpu: Vec<usize>,
    clock_khz: Vec<u64>,
    driver: Option<&'a PolicyField>,
    governor: Option<&'a PolicyField>,
    epp: Option<&'a PolicyField>,
    boost: Option<&'a PolicyField>,
    scaling_min_freq: Option<&'a PolicyField>,
    scaling_max_freq: Option<&'a PolicyField>,
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
        name: "host",
        unit: "-",
        meaning: "hostname of the box that ran the bench",
    },
    FieldDoc {
        name: "pid",
        unit: "-",
        meaning: "process id, which with run_index orders records sharing a timestamp",
    },
    FieldDoc {
        name: "run_index",
        unit: "-",
        meaning: "0-based index of this record within its process ('all' emits several per second)",
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
        name: "pin_cpus",
        unit: "-",
        meaning: "the --pin core pool the run's threads drew from, empty means unpinned",
    },
    FieldDoc {
        name: "duration_s",
        unit: "s",
        meaning: "measured wall time of the run",
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
        meaning: "timed samples recorded (the outer-loop count)",
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
        meaning: "whole-histogram per-call mean, tail included (see suspended_s for when it lies)",
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
        meaning: "sleep-separated block count, null for a single continuous run",
    },
    FieldDoc {
        name: "block_mean_ns",
        unit: "ns",
        meaning: "per-block mean series in run order, null when the run was not blocked",
    },
    FieldDoc {
        name: "block_ci95_ns",
        unit: "ns",
        meaning: "95% confidence half-width on the block-mean average, null when not blocked",
    },
    FieldDoc {
        name: "block_lsc_ns",
        unit: "ns",
        meaning: "least significant change vs an equal-blocks run, null when not blocked",
    },
    FieldDoc {
        name: "clock_t_ns",
        unit: "ns",
        meaning: "delivered-clock sample times at batch seams, raw integer ns from warmup start",
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
        "One NDJSON object per bench result (--record), schema_version {SCHEMA_VERSION}. Fields:\n"
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
}

/// Append `out`'s record through `cfg`'s sink, a no-op when `--record` was not given. A record
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
    /// Resolve the `--record` path and `--tag` list into a sink, validating both now so a bad
    /// argument fails before any bench runs. The path's shape picks the mode: a trailing `/` or
    /// an existing directory means one file per run in that directory (created if missing), and
    /// anything else means append to that one file.
    pub fn new(path: &Path, tags: &[String]) -> Result<Recorder, String> {
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
        let target = resolve_target(path);
        if let Target::Dir(dir) = &target {
            std::fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
        }
        Ok(Recorder {
            target,
            tags: tag_map,
            host: hostname(),
        })
    }

    /// Build and append one record. The open is append-and-create in both modes, never
    /// truncate.
    fn write(&self, bench: &str, out: &RunOutput, cfg: &RunCfg) -> Result<(), String> {
        let policy = freq::policy();
        let record = build_record(
            bench,
            out,
            cfg,
            &self.host,
            &self.tags,
            &policy,
            next_index(),
        );
        let line = serde_json::to_string(&record).map_err(|e| format!("serializing: {e}"))?;
        let path = match &self.target {
            Target::File(f) => f.clone(),
            Target::Dir(d) => d.join(format!(
                "{}-{}-{}.ndjson",
                basic_stamp(out.wall_start),
                sanitize(&self.host),
                sanitize(bench),
            )),
        };
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)
            .map_err(|e| format!("opening {}: {e}", path.display()))?;
        writeln!(file, "{line}").map_err(|e| format!("writing {}: {e}", path.display()))?;
        Ok(())
    }
}

/// Dir mode when the argument ends with `/` or names an existing directory, file mode
/// otherwise: the path's shape picks the mode.
fn resolve_target(path: &Path) -> Target {
    let shaped_dir = path.as_os_str().to_string_lossy().ends_with('/');
    if shaped_dir || path.is_dir() {
        Target::Dir(path.to_path_buf())
    } else {
        Target::File(path.to_path_buf())
    }
}

/// The next record's 0-based index within this process: what separates the several records one
/// second of `all` can emit, alongside `pid`.
fn next_index() -> u32 {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Assemble the record from a finished run. Pure with respect to its inputs (the policy and
/// index are passed in), so the dictionary test can drive it without touching sysfs or the
/// process counter.
fn build_record<'a>(
    bench: &'a str,
    out: &'a RunOutput,
    cfg: &'a RunCfg,
    host: &'a str,
    tags: &'a BTreeMap<String, String>,
    policy: &'a freq::Policy,
    run_index: u32,
) -> Record<'a> {
    let (settle_s, settle_ghz) = match out.warm_settle {
        Some(Settle::At { t_s, ghz }) => (Some(t_s), ghz),
        Some(Settle::Never) | None => (None, None),
    };
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
        version: env!("CARGO_PKG_VERSION"),
        t_start: rfc3339_millis(out.wall_start),
        utc_offset_s: utc_offset_s(out.wall_start),
        host,
        pid: std::process::id(),
        run_index,
        bench,
        tags,
        pin_cpus: cfg.pin_cores,
        duration_s: out.duration_s,
        suspended_s: out.suspended_s,
        warm_exit: match out.warm_exit {
            WarmExit::Settled => "settled",
            WarmExit::Unstable => "unstable",
            WarmExit::Uncertified => "uncertified",
        },
        warm_used_s: out.warm_used_s,
        warm_budget_s: out.warm_budget_s,
        settle_s,
        settle_ghz,
        samples: out.outer,
        inner: out.inner,
        calls: out.outer * out.inner,
        min_ns: out.hist.min() as f64 / PS_PER_NS,
        mean_ns: out.hist.mean() / PS_PER_NS,
        stdev_ns: out.hist.stdev() / PS_PER_NS,
        max_ns: out.hist.max() as f64 / PS_PER_NS,
        quantile_pcts: &QUANTILE_PCTS,
        quantile_ns: QUANTILE_PCTS
            .iter()
            .map(|pct| out.hist.value_at_quantile(pct / 100.0) as f64 / PS_PER_NS)
            .collect(),
        blocks: out.block_stats.as_ref().map(|b| b.blocks),
        block_mean_ns: out.block_stats.as_ref().map(|b| b.means_ns.as_slice()),
        block_ci95_ns: out.block_stats.as_ref().map(|b| b.ci95_ns),
        block_lsc_ns: out.block_stats.as_ref().map(|b| b.lsc_ns),
        clock_t_ns,
        clock_cpu,
        clock_khz,
        driver: policy.driver.as_ref(),
        governor: policy.governor.as_ref(),
        epp: policy.epp.as_ref(),
        boost: policy.boost.as_ref(),
        scaling_min_freq: policy.scaling_min_freq.as_ref(),
        scaling_max_freq: policy.scaling_max_freq.as_ref(),
    }
}

/// The writing box's hostname via `gethostname`, `unknown-host` when the syscall fails.
fn hostname() -> String {
    let mut buf = [0u8; 256];
    // SAFETY: gethostname writes at most buf.len() bytes into buf.
    let rc = unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) };
    if rc != 0 {
        return "unknown-host".to_string();
    }
    let end = match buf.iter().position(|&b| b == 0) {
        Some(e) => e,
        // A name that filled the buffer arrives untruncated-looking either way: take it whole.
        None => buf.len(),
    };
    String::from_utf8_lossy(&buf[..end]).into_owned()
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

/// Basic ISO to the second (`20260804T093221Z`) for dir-mode filenames: no colons, and
/// lexicographic order is chronological order.
fn basic_stamp(t: std::time::SystemTime) -> String {
    let (y, mo, d, h, mi, s, _) = utc_parts(t);
    format!("{y:04}{mo:02}{d:02}T{h:02}{mi:02}{s:02}Z")
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
            outer: 4,
            inner: 10,
            duration_s: 5.0,
            suspended_s: 0.0,
            block_stats: Some(BlockStats {
                blocks: 2,
                mean_ns: 24.0,
                ci95_ns: 1.0,
                lsc_ns: 2.0,
                means_ns: vec![23.5, 24.5],
            }),
            batches: Vec::new(),
            probes: Vec::new(),
            warmup_probes: 0,
            warm_exit: WarmExit::Settled,
            warm_tail: 0,
            warm_clock: None,
            warm_settle: Some(Settle::At {
                t_s: 0.81,
                ghz: Some(4.35),
            }),
            warm_used_s: 0.9,
            warm_budget_s: 3.0,
            wall_start: std::time::UNIX_EPOCH + std::time::Duration::from_millis(1_000_000_000_123),
            seam_clock: vec![SeamClock {
                t_ns: 1_500_000_000,
                cpu: 3,
                khz: 4_350_000,
            }],
        }
    }

    /// A `RunCfg` for record assembly, where only `pin_cores` reaches the record.
    fn sample_cfg(pin: &[usize]) -> RunCfg<'_> {
        RunCfg {
            target_seconds: 5.0,
            outer_override: None,
            inner_override: None,
            pin_cores: pin,
            report_ticks: false,
            seam_probes: true,
            band_labels: BandLabels::Both,
            decimals: 1,
            settle_time_s: 1.5,
            warm_cap_s: 1.5,
            blocks: Some(2),
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
        let record = build_record("min-now", &out, &cfg, "3900x", &tags, &policy, 7);
        serde_json::to_value(&record).expect("record serializes")
    }

    #[test]
    fn every_record_key_is_documented_and_every_doc_names_a_key() {
        let value = sample_value();
        let obj = value.as_object().expect("record is an object");
        let keys: std::collections::BTreeSet<&str> = obj.keys().map(String::as_str).collect();
        let docs: std::collections::BTreeSet<&str> = FIELD_DOCS.iter().map(|f| f.name).collect();
        let undocumented: Vec<&&str> = keys.difference(&docs).collect();
        assert!(
            undocumented.is_empty(),
            "undocumented keys: {undocumented:?}"
        );
        let stale: Vec<&&str> = docs.difference(&keys).collect();
        assert!(stale.is_empty(), "docs naming no key: {stale:?}");
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
        assert_eq!(value["host"], serde_json::json!("3900x"));
        assert_eq!(value["run_index"], serde_json::json!(7));
        assert_eq!(
            value["t_start"],
            serde_json::json!("2001-09-09T01:46:40.123Z")
        );
        assert_eq!(value["tags"]["series"], serde_json::json!("t1"));
        assert_eq!(value["governor"]["uniform"], serde_json::json!(false));
        assert_eq!(value["block_mean_ns"], serde_json::json!([23.5, 24.5]));
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
        assert_eq!(basic_stamp(t), "20010909T014640Z");
    }

    #[test]
    fn target_mode_follows_the_path_shape() {
        // A trailing slash is dir mode whether or not the directory exists yet.
        assert_eq!(
            resolve_target(Path::new("no/such/dir/")),
            Target::Dir(PathBuf::from("no/such/dir/"))
        );
        // No slash and no existing directory is file mode.
        assert_eq!(
            resolve_target(Path::new("no/such/file.ndjson")),
            Target::File(PathBuf::from("no/such/file.ndjson"))
        );
        // An existing directory is dir mode even without the slash.
        assert_eq!(
            resolve_target(Path::new("src")),
            Target::Dir(PathBuf::from("src"))
        );
    }

    #[test]
    fn tags_must_be_key_value() {
        let dir = std::env::temp_dir().join("iiac-perf-record-test");
        assert!(Recorder::new(&dir, &["novalue".to_string()]).is_err());
        assert!(Recorder::new(&dir, &["=v".to_string()]).is_err());
        let rec = Recorder::new(&dir, &["k=v=w".to_string()]).expect("first '=' splits");
        assert_eq!(rec.tags.get("k").map(String::as_str), Some("v=w"));
    }

    #[test]
    fn sanitize_keeps_safe_chars_only() {
        assert_eq!(sanitize("rpi5-20cd"), "rpi5-20cd");
        assert_eq!(sanitize("a/b:c d"), "a-b-c-d");
    }
}
