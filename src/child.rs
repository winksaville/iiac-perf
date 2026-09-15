//! One bench per process: the parent's spawn of a child and the child's side of it.
//!
//! A process start re-rolls where a bench's rings and stacks land in memory, and that placement
//! sets the bench's level, so a bench sharing a process with the benches before it inherits their
//! placement. Every bench runs in a child of its own instead.
//!
//! - The parent resolves every run knob once, starts the sleep inhibit and the clock pin once,
//!   and for each bench writes a [`Spec`] to a file and runs `current_exe()` with the hidden
//!   `--child-spec PATH`. It waits in the kernel while the child measures, so it adds no thread of
//!   its own to the run, the `suggest-freq` sampler bug in notes/bugs.md being the warning.
//! - The child reads the spec, pins its main thread to the pool's first CPU, calibrates the tick
//!   rate, runs the one bench, and prints only that bench's report, the parent having printed the
//!   banner, `Setup:`, and `Config:`.
//! - A spec carries resolved values, never flags, so a child loads no config file, and the
//!   record's `config` is the parent's, the one the report printed.

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::bands::BandLabels;
use crate::harness::RunCfg;
use crate::record::{RecordConfig, Recorder};

/// Everything a child needs to run one bench as the parent resolved it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spec {
    /// The registered bench name, exact.
    pub bench: String,
    /// [`RunCfg::target_seconds`].
    pub target_seconds: f64,
    /// [`RunCfg::samples_override`].
    pub samples_override: Option<u64>,
    /// [`RunCfg::inner_override`].
    pub inner_override: Option<u64>,
    /// [`RunCfg::pin_cpus`], resolved from any profile name.
    pub pin_cpus: Vec<usize>,
    /// [`RunCfg::report_ticks`].
    pub report_ticks: bool,
    /// [`RunCfg::seam_probes`].
    pub seam_probes: bool,
    /// [`RunCfg::band_labels`], as its lowercase name.
    pub band_labels: String,
    /// [`RunCfg::decimals`].
    pub decimals: usize,
    /// [`RunCfg::settle_time_s`]. Every child is a fresh process, so every child pays it.
    pub settle_time_s: f64,
    /// [`RunCfg::warm_cap_s`].
    pub warm_cap_s: f64,
    /// [`RunCfg::blocks`].
    pub blocks: u64,
    /// [`RunCfg::block_sleep_s`].
    pub block_sleep_s: (f64, f64),
    /// [`RunCfg::block_warmup_s`].
    pub block_warmup_s: f64,
    /// Where and how the child records, when `--record` was given.
    pub record: Option<RecordSpec>,
}

/// The `--record` sink as a child rebuilds it: the path made absolute, the tags verbatim, and
/// the parent's resolved config.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordSpec {
    /// The `--record` path, absolute.
    pub path: PathBuf,
    /// The `--tag` list, verbatim.
    pub tags: Vec<String>,
    /// The run's config as the parent's `Config:` list resolved it.
    pub config: RecordConfig,
}

impl Spec {
    /// The spec for running `bench` under `cfg`, recording through `record` when set.
    pub fn new(bench: &str, cfg: &RunCfg, record: Option<RecordSpec>) -> Spec {
        Spec {
            bench: bench.to_string(),
            target_seconds: cfg.target_seconds,
            samples_override: cfg.samples_override,
            inner_override: cfg.inner_override,
            pin_cpus: cfg.pin_cpus.to_vec(),
            report_ticks: cfg.report_ticks,
            seam_probes: cfg.seam_probes,
            band_labels: cfg.band_labels.as_str().to_string(),
            decimals: cfg.decimals,
            settle_time_s: cfg.settle_time_s,
            warm_cap_s: cfg.warm_cap_s,
            blocks: cfg.blocks,
            block_sleep_s: cfg.block_sleep_s,
            block_warmup_s: cfg.block_warmup_s,
            record,
        }
    }
}

/// A private directory for one invocation's spec files, under the system temp directory and
/// named by the parent's pid, removed by [`ScratchDir`]'s drop.
pub struct ScratchDir(PathBuf);

impl ScratchDir {
    /// Create the directory.
    pub fn new() -> Result<ScratchDir, String> {
        let dir = std::env::temp_dir().join(format!("{}-{}", crate::BIN_NAME, std::process::id()));
        std::fs::create_dir_all(&dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
        Ok(ScratchDir(dir))
    }

    /// The directory's path.
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchDir {
    /// Remove the directory and whatever a child left in it. A failure leaves a small directory
    /// in the temp area, which is not worth failing a finished run over.
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_dir_all(&self.0) {
            log::warn!("removing {}: {e}", self.0.display());
        }
    }
}

/// Run `spec` in a child of `exe`, its spec file named by `index` inside `dir`, waiting for it
/// to exit. The child inherits stdout and stderr, so its report streams as it prints. `verbose`
/// passes `-v` on.
pub fn spawn(
    exe: &Path,
    dir: &Path,
    index: usize,
    spec: &Spec,
    verbose: bool,
) -> Result<(), String> {
    let path = dir.join(format!("child-{index}.json"));
    let text = serde_json::to_string(spec).map_err(|e| format!("serializing the spec: {e}"))?;
    std::fs::write(&path, text).map_err(|e| format!("writing {}: {e}", path.display()))?;
    let mut cmd = Command::new(exe);
    cmd.arg("--child-spec").arg(&path);
    if verbose {
        cmd.arg("-v");
    }
    let status = cmd
        .status()
        .map_err(|e| format!("{}: spawning {}: {e}", spec.bench, exe.display()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{}: the child process {status}", spec.bench))
    }
}

/// The child's side: read the spec at `spec_path`, run its bench, and return the exit code.
pub fn child_main(spec_path: &Path) -> i32 {
    match run_spec(spec_path) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("error: child: {e}");
            2
        }
    }
}

/// Read, check, and run one spec: pin main to the pool's first CPU, warm the tick-rate
/// calibration before anything measures, build the record sink, and run the bench.
fn run_spec(spec_path: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(spec_path)
        .map_err(|e| format!("reading {}: {e}", spec_path.display()))?;
    let spec: Spec =
        serde_json::from_str(&text).map_err(|e| format!("parsing {}: {e}", spec_path.display()))?;
    let run = crate::benches::find(&spec.bench)
        .ok_or_else(|| format!("no bench is named '{}'", spec.bench))?;
    let band_labels =
        BandLabels::from_str(&spec.band_labels, false).map_err(|e| format!("band_labels: {e}"))?;
    if let Some(&cpu) = spec.pin_cpus.first() {
        crate::pin::pin_current(Some(cpu));
    }
    crate::ticks::ticks_per_ns();
    let recorder = match &spec.record {
        None => None,
        Some(r) => Some(Recorder::new(&r.path, &r.tags, r.config.clone())?),
    };
    let cfg = RunCfg {
        target_seconds: spec.target_seconds,
        samples_override: spec.samples_override,
        inner_override: spec.inner_override,
        pin_cpus: &spec.pin_cpus,
        report_ticks: spec.report_ticks,
        seam_probes: spec.seam_probes,
        band_labels,
        decimals: spec.decimals,
        settle_time_s: spec.settle_time_s,
        warm_cap_s: spec.warm_cap_s,
        blocks: spec.blocks,
        block_sleep_s: spec.block_sleep_s,
        block_warmup_s: spec.block_warmup_s,
        record: recorder.as_ref(),
    };
    run(&cfg);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A run configuration with every field off its zero value, so a dropped field shows.
    fn cfg(pins: &[usize]) -> RunCfg<'_> {
        RunCfg {
            target_seconds: 2.5,
            samples_override: Some(1000),
            inner_override: Some(7),
            pin_cpus: pins,
            report_ticks: true,
            seam_probes: false,
            band_labels: BandLabels::Frac,
            decimals: 3,
            settle_time_s: 0.5,
            warm_cap_s: 0.25,
            blocks: 40,
            block_sleep_s: (0.001, 0.01),
            block_warmup_s: 0.002,
            record: None,
        }
    }

    #[test]
    fn a_spec_carries_every_knob_through_json() {
        let pins = [3, 5];
        let record = RecordSpec {
            path: PathBuf::from("/tmp/records/"),
            tags: vec!["series=a".to_string()],
            config: RecordConfig::new(&[], &[]),
        };
        let spec = Spec::new("min-now", &cfg(&pins), Some(record));
        let back: Spec = serde_json::from_str(&serde_json::to_string(&spec).unwrap()).unwrap();
        assert_eq!(back, spec);
        assert_eq!(back.pin_cpus, [3, 5]);
        assert_eq!(back.band_labels, "frac");
        assert_eq!(
            BandLabels::from_str(&back.band_labels, false),
            Ok(BandLabels::Frac)
        );
    }

    #[test]
    fn a_missing_spec_or_unknown_bench_is_an_error_not_a_panic() {
        assert_eq!(child_main(Path::new("/nonexistent/iiac-perf-spec.json")), 2);
        let dir = std::env::temp_dir().join(format!("iiac-perf-child-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("spec.json");
        let spec = Spec::new("no-such-bench", &cfg(&[]), None);
        std::fs::write(&path, serde_json::to_string(&spec).unwrap()).unwrap();
        assert_eq!(child_main(&path), 2);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
