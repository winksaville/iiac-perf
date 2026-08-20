//! Generic bench driver: the [`Bench`] trait, the warm loop, adaptive outer/inner loop sizing,
//! and the batch/block pipeline that turns a workload into a [`RunOutput`].
//!
//! Measuring only. Rendering that `RunOutput` into text is [`crate::report`], which this module
//! never calls: the value is the seam, and it flows one way.

use std::hint::black_box;

use hdrhistogram::Histogram;

use crate::bands::BandLabels;
use crate::dither::Dither;

const FRAMING_DOMINATION_RATIO: f64 = 10.0;
const MAX_INNER: u64 = 1_000;

/// Minimum bench steps in one warmup pass, so a pass's per-step
/// cost (the sizing input) is never one sample.
///
/// - Small on purpose: for cheap steps the wall minimum ([`WARM_PASS_MIN_SECONDS`]) dominates
///   anyway, and a large count would hold a genuinely slow bench (ms-scale steps) far past the
///   cap before its first pass ended.
const WARM_PASS_MIN_STEPS: u64 = 8;

/// Minimum wall seconds in one warmup pass: with [`WARM_PASS_MIN_STEPS`], "whichever is larger"
/// sets the pass length, so a pass is a real burst of load rather than a blink.
const WARM_PASS_MIN_SECONDS: f64 = 0.001;

/// Ceiling on the step chunk between elapsed checks inside an adaptive warmup pass. The chunk
/// starts at 1 and doubles, so a cheap bench amortizes `Instant::now` out of its measured pass
/// cost while a slow bench still gets a deadline check every step (the estimate-phase hang this
/// replaces, bugs.md #1).
const WARM_STEP_CHUNK_MAX: u64 = 64;

/// Minimum probes in the warm exit window: the split detector wants 4 points a side.
const WARM_WINDOW_MIN_PROBES: usize = 8;

/// Minimum wall span of the warm exit window (seconds).
///
/// - A window can be far shorter than what it certifies: 16 probes of the retired fixed warmup
///   spanned ~17 us against a transition arriving at ~800 ms, so agreement alone certifies
///   nothing. The span makes the window a statement about held time.
/// - 50 ms is governor-transition scale (single P-state moves land in tens of ms), not
///   full-ramp scale: the exit rule already guarantees the window sits after any movement it
///   can see, so unlike the retired fixed 300 ms tail it does not need to be long enough to
///   *contain* a ramp. What a timing window cannot see at any length (a steady dwell below the
///   top) is the clock rung's job.
const WARM_WINDOW_MIN_SECONDS: f64 = 0.05;

/// Default hard cap on the per-run warm stretch (seconds): the `--warm-cap` / `warm_cap`
/// default ([`RunCfg::warm_cap_s`]). Hitting the cap is reported, never silently absorbed: the
/// window's actual grade stands (the "run started unstable" signal), or the run is labelled
/// uncertified when no window ever formed ([`WarmExit`]). The cap also deadlines every
/// adaptive pass, so a pathologically slow bench exits with a diagnosis instead of hanging in
/// an open step loop.
///
/// - 1.5 s rather than governor scale (the 0.4 s it started at): the 3900X's relaxation
///   re-ramp measured ~0.7 to 1.4 s on 2026-08-02's acceptance runs, and a cap below it turns
///   an absorbable ramp into a "not settled" report. A settled box exits in ~50 ms regardless;
///   the cap prices only the disturbed case.
pub const DEFAULT_WARM_CAP_S: f64 = 1.5;

/// Timer pairs per *timed group* inside a micro-probe.
///
/// - The probe's unit of measurement is a group, not a single
///   pair: one `Instant` pair brackets [`PROBE_GROUP_PAIRS`]
///   pairs and the total is divided down to a per-pair value.
/// - **Why group at all:** the timer reads integer nanoseconds,
///   so a single ~25 ns pair is quantized to ~4% — coarser than
///   the frequency-ramp movement the environment grade exists to
///   see (~9% on the 3900X). A 64-pair group totals ~1.6 µs, so
///   the same 1 ns quantum is ~0.06% of the value, and the
///   per-pair figure lands on a ~15 ps lattice.
/// - Grouping costs the *census* its sensitivity, so the census
///   is counted per pair instead — see [`Prober::probe`].
const PROBE_GROUP_PAIRS: usize = 64;

/// Timed groups per micro-probe — the population its floor and
/// spread quantiles are taken over.
///
/// - 128 groups of [`PROBE_GROUP_PAIRS`] is ~8,192 pairs, about
///   256 µs. Sized against the batch seam it runs in, which
///   already costs 1-2 ms (a `select_nth_unstable` plus 65,536
///   histogram records), so a probe adds a fraction of a gap
///   that already exists rather than a new one.
const PROBE_GROUPS: usize = 128;

/// Initial capacity of the warmup probe series: a settled box exits with
/// ~[`WARM_WINDOW_MIN_SECONDS`] of ~1 ms passes, so ~64 probes; a first run's process warm adds
/// ~150.
const WARM_PROBES_CAPACITY: usize = 64;

/// Default wall seconds the **first** run in a process spends
/// stepping the bench before any samples are recorded: the
/// `--settle-time` / `settle_time` default.
///
/// - **The first bench of every process reports numbers ~8.6%
///   slow otherwise.** Measured on a 7600x 2026-07-29: `min-now`
///   reads 17.6 ns as the process's first bench against 16.2 ns
///   once warm, and its env bench stretch grades `step 11.05% F`
///   while benches
///   2-17 of the same process grade env-clean. The P-state boost
///   is machine state, so every later bench inherits it. The
///   warm belongs to the process, not to each run.
/// - 1.5 s rather than 1.0: the ramp measured at ~150-200 ms but
///   the 3900X's relaxation lands later, and the exit window
///   ([`WARM_WINDOW_MIN_SECONDS`]) needs settled time behind it.
/// - Cost is ~2% of an `all -d 5` sweep (~85 s to ~86.5 s), paid
///   once, against a wrong histogram on whichever bench ran
///   first.
pub const DEFAULT_SETTLE_TIME_S: f64 = 1.5;

/// Wall seconds of stepping between micro-probes during the
/// process warm: ~150 probes over [`DEFAULT_SETTLE_TIME_S`], so
/// the series resolves a transition to ~10 ms and the exit
/// window holds several probes even before any per-run pass.
const PROCESS_WARM_PROBE_GAP_S: f64 = 0.01;

/// Steps between elapsed-time checks during the process warm, so
/// a cheap bench doesn't spend the warm inside `Instant::now`.
const PROCESS_WARM_STEP_CHUNK: usize = 64;

/// Quantile defining a micro-probe's floor, matching
/// [`BATCH_FLOOR_Q`]'s reasoning: the left edge is sparse, a
/// tenth of the population is not.
const ENV_FLOOR_Q: f64 = 0.10;

/// Upper quantile paired with [`ENV_FLOOR_Q`] to measure a
/// probe's spread — how wide the bulk of the distribution sits
/// above its own floor.
const ENV_SPREAD_Q: f64 = 0.90;

/// Census threshold for a micro-probe group, the batch census
/// rule ([`BATCH_OVER_MULT`]) rebased on the probe's scale: the
/// additive term is 5 ns rather than 50, because a per-pair
/// value is ~25 ns and a 50 ns floor would never be crossed.
const ENV_OVER_ADD_PS: u64 = 5_000;

/// Batch buffer capacity in samples — the pipeline's memory
/// bound (512 KiB of u64 ps values). Fast benches fill it in
/// ~15–40 ms and flush full; slow benches flush earlier on
/// [`BATCH_TARGET_SECONDS`].
pub(crate) const BATCH_SAMPLES: usize = 65_536;

/// Time-based batch flush (seconds): a partial batch flushes
/// once it spans this long, so slow benches still get a usable
/// time axis (drift/burst localization) from few samples.
pub(crate) const BATCH_TARGET_SECONDS: f64 = 0.05;

/// Push-count mask between time checks in
/// [`BatchPipeline::push`] — one `Instant::now` per 1024
/// samples keeps the check cost off the per-sample path.
const BATCH_CHECK_MASK: usize = 1023;

/// Quantile defining a batch's *robust* floor, the statistic the
/// gauge's drift/step signals read.
///
/// - The raw min is too sparse to grade movement: measured on a
///   quiet 3900X at inner=10 (100 ps lattice), adjacent batch
///   minima flipped between 22.0 and 23.0 ns — a 4.5% "step" on
///   a run with no state change, which alone would have graded
///   every quiet run F.
/// - The same batches' p10 sat on 23.0 ns run-wide and moved
///   only when the machine did. The left edge of the
///   distribution is sparse; a tenth of 65,536 samples is not.
pub(crate) const BATCH_FLOOR_Q: f64 = 0.10;

/// Census threshold: a batch sample is "over floor" above
/// `max(BATCH_OVER_MULT x floor, floor + BATCH_OVER_ADD_PS)`,
/// applied per batch against the batch's own [`BATCH_FLOOR_Q`]
/// floor.
///
/// - Measured against the raw min instead, the census was
///   meaningless on any bench with a low tail: mpsc-2t batches
///   whose min landed on a 0.9 µs fast path (against a 6.5 µs
///   floor) counted 99.9% of their samples "over floor", and the
///   ones whose min landed normally counted 1%.
const BATCH_OVER_MULT: f64 = 1.5;

/// Additive part of the census threshold (50 ns in ps).
const BATCH_OVER_ADD_PS: u64 = 50_000;

/// Histogram value bounds: 1 ps to 60 s at 3 sig figs. Values
/// are recorded in **picoseconds** — the timer reads integer ns,
/// but dividing a sample by `inner` in ps keeps the true sub-ns
/// per-call precision that ns recording truncated (a 4.7 ns call
/// no longer rounds to 5). The high bound is a sane-world
/// ceiling for one recorded sample, not a technical limit —
/// [`record_sample`] clamps above it and [`crate::report::warn_invalid`] flags
/// the run.
const HIST_LOW_PS: u64 = 1;
pub const HIST_HIGH_PS: u64 = 60_000_000_000_000;

/// Picoseconds per nanosecond: recorded values are ps, display
/// is ns.
pub const PS_PER_NS: f64 = 1000.0;

/// A benchmark workload: a named operation that `step()` performs
/// in a tight loop for sub-µs latency measurement. Implementors own
/// any setup state (channels, spawned threads, counters). `step()`
/// returns a value so the caller can `black_box` it against dead-code
/// elimination.
pub trait Bench {
    /// Human-readable name used in the report header.
    fn name(&self) -> &str;

    /// Run one unit of work. Return any value derived from the work
    /// to defeat DCE — the caller black-boxes it.
    fn step(&mut self) -> u64;
}

/// Runtime configuration for one [`run_adaptive`] call.
#[derive(Debug)]
pub struct RunCfg<'a> {
    /// Wall-clock seconds budget for time-based runs. Ignored when
    /// `outer_override` is set.
    pub target_seconds: f64,
    /// Force a fixed outer-loop count, bypassing the time budget.
    pub outer_override: Option<u64>,
    /// Force a fixed inner-loop count, bypassing the
    /// micro-probe-driven auto-sizing.
    pub inner_override: Option<u64>,
    /// CPU pool for thread pinning. Indexed positionally with
    /// wrap-around via [`cpu_for`][RunCfg::cpu_for]; empty means
    /// no pinning.
    pub pin_cpus: &'a [usize],
    /// When set, [`crate::tprobe::TProbe::report`] emits raw TSC
    /// ticks instead of nanoseconds. Plumbed from the `-t/--ticks`
    /// CLI flag.
    pub report_ticks: bool,
    /// Sample the environment at every batch seam, so the
    /// environment grade spans the whole run. Cleared by
    /// `--no-env-probe`, which leaves only the warmup probes.
    /// Plumbed from the CLI.
    pub seam_probes: bool,
    /// Band-label style for [`crate::report::print_report`] histogram rows.
    /// Plumbed from the `--band-labels` CLI flag.
    pub band_labels: BandLabels,
    /// Decimal digits on [`crate::report::print_report`] time columns. Plumbed
    /// from the `--decimals` CLI flag (default 1; 0 restores
    /// integers; 3 is the ps recording floor).
    pub decimals: usize,
    /// Seconds the first run in the process spends warming the
    /// box before it records anything; zero skips the warm. Later
    /// runs in the same process inherit the machine state it won,
    /// so the cost is paid once. Plumbed from `--settle-time` /
    /// the `settle_time` config key, defaulting to
    /// [`DEFAULT_SETTLE_TIME_S`].
    pub settle_time_s: f64,
    /// Hard cap on the per-run warm-until-stable stretch (seconds); hitting it is reported
    /// ([`WarmExit`]), never silently absorbed. Plumbed from `--warm-cap` / the `warm_cap`
    /// config key, defaulting to [`DEFAULT_WARM_CAP_S`]. Zero caps immediately, which is how a
    /// run measures what the warm is worth.
    pub warm_cap_s: f64,
    /// Split the run into this many measurement blocks and report
    /// block stats (mean, and CI95 / LSC when the blocks
    /// replicate). Plumbed from the `--blocks` CLI flag; `None` =
    /// single continuous run. See
    /// notes/design.md#within-invocation-replication-sleep-separated-blocks.
    pub blocks: Option<u64>,
    /// Sleep between blocks, `(min_s, max_s)` seconds, re-rolled
    /// uniformly per block when the ends differ. Zero (the
    /// default) never sleeps: the blocks are then partitions of
    /// one continuous run, not replicates, and [`BlockStats`]
    /// withholds CI95 / LSC. Plumbed from `--block-sleep` / the
    /// `block_sleep` config key.
    pub block_sleep_s: (f64, f64),
    /// Unrecorded post-wake warmup per block, seconds. Keeps the
    /// frequency ramp and cache refill out of the samples; zero
    /// (the default) records from the first post-wake call, which
    /// is how cold-wake behavior is seen. Plumbed from
    /// `--block-warmup` / the `block_warmup` config key.
    pub block_warmup_s: f64,
    /// The per-run record sink, when `--record` was given:
    /// [`crate::record::append`] writes one NDJSON object per
    /// finished harness run through it. `None` records nothing.
    pub record: Option<&'a crate::record::Recorder>,
}

impl RunCfg<'_> {
    /// CPU id for the bench's `thread_idx`-th thread, using
    /// wrap-around over the pool. Returns `None` when the pool is
    /// empty so callers can treat unpinned and pinned runs uniformly.
    pub fn cpu_for(&self, thread_idx: usize) -> Option<usize> {
        if self.pin_cpus.is_empty() {
            None
        } else {
            Some(self.pin_cpus[thread_idx % self.pin_cpus.len()])
        }
    }
}

/// Per-block statistics from a `--blocks` run. With a nonzero
/// block sleep each block is a mini-run (own sleep re-roll +
/// warm-up), so the spread of block means yields an
/// honest-per-invocation CI and LSC. With no sleep the blocks are
/// partitions of one continuous run — they share the run's state,
/// cannot replicate it, and CI95 / LSC are withheld rather than
/// printed as a fiction. See
/// notes/design.md#within-invocation-replication-sleep-separated-blocks.
#[derive(Debug)]
pub struct BlockStats {
    /// Number of blocks (Y).
    pub blocks: u64,
    /// Mean of the per-block means, ns.
    pub mean_ns: f64,
    /// 95% confidence half-width on `mean_ns`:
    /// `t(0.975, Y-1) * s / sqrt(Y)`, ns. `None` when the blocks
    /// are sleepless partitions rather than replicates.
    pub ci95_ns: Option<f64>,
    /// Least significant change vs an equal-Y run of another
    /// implementation: `t(0.975, 2Y-2) * s * sqrt(2/Y)`, ns.
    /// `None` when the blocks are sleepless partitions.
    pub lsc_ns: Option<f64>,
    /// The per-block means themselves (ns), in run order: the
    /// record's series. An aggregate cannot be decomposed, and
    /// the series is what lets within-run scatter be compared
    /// against across-run scatter. Kept whole: the CLI caps
    /// `--blocks` at 1,000, the record's keep-every-mean bound.
    pub means_ns: Vec<f64>,
}

impl BlockStats {
    /// Fit from per-block means (ns), which the stats keep.
    /// `replicated` says whether a nonzero sleep separated the
    /// blocks; without one the t-formulas' independence premise is
    /// false, so CI95 / LSC stay `None`. Caller guarantees
    /// `means.len() >= 2` (the CLI enforces `--blocks 2..`).
    fn from_means(means: Vec<f64>, replicated: bool) -> BlockStats {
        let y = means.len() as f64;
        let mean = means.iter().sum::<f64>() / y;
        let var = means.iter().map(|m| (m - mean) * (m - mean)).sum::<f64>() / (y - 1.0);
        let s = var.sqrt();
        let yy = means.len() as u64;
        let (ci95_ns, lsc_ns) = if replicated {
            (
                Some(t975(yy - 1) * s / y.sqrt()),
                Some(t975(2 * yy - 2) * s * (2.0 / y).sqrt()),
            )
        } else {
            (None, None)
        };
        BlockStats {
            blocks: yy,
            mean_ns: mean,
            ci95_ns,
            lsc_ns,
            means_ns: means,
        }
    }
}

/// Two-sided 95% Student-t quantile (`t(0.975, df)`), table for
/// df ≤ 30, then the conservative 2.0 (the true value falls from
/// 2.042 toward the normal 1.96). Shared with
/// [`crate::resolution`], which applies the same LSC formula per
/// aggregation level.
pub(crate) fn t975(df: u64) -> f64 {
    const TABLE: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
        2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
        2.052, 2.048, 2.045, 2.042,
    ];
    match df {
        0 => f64::INFINITY,
        1..=30 => TABLE[(df - 1) as usize],
        _ => 2.0,
    }
}

/// Everything a finished [`run_adaptive`] run produced — the
/// histogram plus the metadata [`crate::report::print_report`] needs and the
/// time-ordered [`BatchSummary`] series the gauge reads.
#[derive(Debug)]
pub struct RunOutput {
    /// Per-call values (ps) of every sample.
    pub hist: Histogram<u64>,
    /// Samples taken (outer-loop count).
    pub outer: u64,
    /// Calls per sample (inner-loop count).
    pub inner: u64,
    /// Measured wall time, seconds.
    pub duration_s: f64,
    /// Seconds the system spent suspended during the run (see
    /// [`ClockPair`]); [`crate::report::print_report`] flags poisoned stats
    /// when non-trivial.
    pub suspended_s: f64,
    /// Block-replication stats — `Some` only for `--blocks`
    /// runs.
    pub block_stats: Option<BlockStats>,
    /// Time-ordered per-batch summaries from the pipeline.
    pub batches: Vec<BatchSummary>,
    /// Time-ordered micro-probe summaries — the environment
    /// grade's input. One series, two stretches: see
    /// [`RunOutput::warmup_probes`].
    pub probes: Vec<ProbeSummary>,
    /// How many leading [`RunOutput::probes`] came from warmup.
    /// Splits the series into the stretch measured before the
    /// bench ran and the stretch measured alongside it — graded
    /// separately, because a ramp warmup absorbed is not a fault
    /// and blending the two invents a step at the boundary.
    pub warmup_probes: usize,
    /// How the warm stretch ended: the warm-until-stable exit verdict the report prints
    /// beside the warmup grade.
    pub warm_exit: WarmExit,
    /// Probe count of the warm exit window, the graded tail of the warmup stretch (see
    /// [`crate::report::env_stretches`]).
    pub warm_tail: usize,
    /// Delivered-clock summary at warm end, when the driver exposes one: reported, never
    /// graded.
    pub warm_clock: Option<WarmClock>,
    /// The clock's journey through the warm stretch ([`crate::gauge::settle`]), computed
    /// while the warmup's clock series was alive since the series is not carried: what the
    /// report's settle cell prints. Read, never scored, like the clock
    /// summary above.
    pub warm_settle: Option<crate::gauge::Settle>,
    /// The warmup clock series compressed for the `-v` profile line
    /// ([`crate::gauge::clock_profile`]), carried because the series itself is not: extremes
    /// plus the tick-per-step shape the stability gate saw. `None` when the clock was
    /// unreadable throughout.
    pub warm_clock_profile: Option<crate::gauge::ClockProfile>,
    /// Wall seconds this run spent warming in total (the process warm when this run ran it,
    /// plus the capped per-run stretch): the header bracket's `warm=used/budget` numerator.
    pub warm_used_s: f64,
    /// This run's total warm allowance (the settle budget when this run ran the process warm,
    /// plus the cap): the `warm=used/budget` denominator.
    pub warm_budget_s: f64,
    /// Wall-clock instant the measured stretch began (warmup excluded): the record's `t_start`
    /// and its dir-mode filename stamp, captured here because only the harness knows when
    /// measuring started.
    pub wall_start: std::time::SystemTime,
    /// One delivered-clock sample per batch seam, so the clock series spans the bench rather
    /// than stopping at warmup's end (which is what hid a mid-bench climb from every report).
    /// Empty when the driver exposes no `cpuinfo_avg_freq`.
    pub seam_clock: Vec<SeamClock>,
    /// The run's resolution claim, fit from the batch series
    /// ([`crate::resolution`]): the variance-curve drift floor
    /// that replaced the within-run LSC as the headline. `None`
    /// below two usable batches.
    pub resolution: Option<crate::resolution::Resolution>,
}

/// One delivered-clock read at a batch seam: the run-phase counterpart of the warmup's clock
/// series, and what makes per-run frequency min/max/median fall out of a record. Under a pinned
/// clock the series collapsing to the pin is the pin's own verification.
#[derive(Debug, Clone, Copy)]
pub struct SeamClock {
    /// Sample time, integer nanoseconds from the run's time origin (the warmup start,
    /// [`BatchSummary::t_start_s`]'s axis at 1e9 scale). The raw elapsed read, kept raw so the
    /// record stores what was measured and seconds stay derivable.
    pub t_ns: u64,
    /// Logical CPU the read landed on.
    pub cpu: usize,
    /// Delivered frequency (kHz) as the driver reports it.
    pub khz: u64,
}

/// Drive `bench` against `cfg` and return a [`RunOutput`].
///
/// After warming until stable (see [`warmup_and_probe`]), `inner` is auto-sized so apparatus
/// framing doesn't dominate (skipped when `cfg.inner_override` is set). The outer loop runs
/// either for `cfg.outer_override` iterations or until `cfg.target_seconds` elapses, as one
/// continuous run or split into `cfg.blocks` measurement blocks (`block_stats` is `Some`
/// only then). Samples flow through the [`BatchPipeline`], so the output carries the run's time
/// axis as per-batch summaries alongside the histogram.
pub fn run_adaptive<B: Bench>(bench: &mut B, cfg: &RunCfg) -> RunOutput {
    let warmed = warmup_and_probe(bench, cfg.settle_time_s, cfg.warm_cap_s);

    // The last warmup probe is the most-warmed one, so sizing reads a post-warmup
    // frame by construction; the step cost is the exit window's best pass
    // ([`Warmed::step_cost_ns`]).
    let frame_ns = match warmed.probes.last() {
        Some(p) => (p.floor_q_ps as f64 / PS_PER_NS).max(1.0),
        None => 1.0,
    };
    let inner = cfg
        .inner_override
        .unwrap_or_else(|| pick_inner(warmed.step_cost_ns, frame_ns));

    let Warmed {
        origin,
        probes: warm_probes,
        prober,
        exit: warm_exit,
        tail: warm_tail,
        clock: warm_clock,
        settle: warm_settle,
        clock_profile: warm_clock_profile,
        used_s: warm_used_s,
        budget_s: warm_budget_s,
        ..
    } = warmed;
    let warmup_probes = warm_probes.len();
    let mut pipeline = BatchPipeline::new(origin, prober, warm_probes, cfg.seam_probes);
    let wall_start = std::time::SystemTime::now();
    let clocks = ClockPair::now();
    let (block_stats, duration_s) = match cfg.blocks {
        Some(blocks) => {
            let (duration_s, stats) = run_blocked(bench, &mut pipeline, blocks, inner, cfg);
            (Some(stats), duration_s)
        }
        None => match cfg.outer_override {
            Some(outer) => (None, run_counted(bench, &mut pipeline, outer, inner)),
            None => (
                None,
                run_timed(bench, &mut pipeline, cfg.target_seconds, inner),
            ),
        },
    };
    let (hist, batches, probes, seam_clock) = pipeline.finish();
    let outer = match cfg.outer_override {
        Some(outer) if cfg.blocks.is_none() => outer,
        _ => hist.len(),
    };
    let resolution = crate::resolution::from_batches(&batches);
    RunOutput {
        hist,
        outer,
        inner,
        duration_s,
        suspended_s: clocks.suspended_s(),
        block_stats,
        batches,
        probes,
        warmup_probes,
        warm_exit,
        warm_tail,
        warm_clock,
        warm_settle,
        warm_clock_profile,
        warm_used_s,
        warm_budget_s,
        wall_start,
        seam_clock,
        resolution,
    }
}

/// Run `blocks` measurement blocks: before each, sleep a uniform
/// draw from `sleep_s` (re-rolls scheduler / frequency / mode-mix
/// state; skipped at zero) and step unrecorded for `warmup_s`
/// (post-wake ramp; skipped at zero), then measure the block's
/// share of the budget (`outer / blocks` samples, or
/// `target_seconds / blocks`). All samples land in one histogram;
/// per-block means feed [`BlockStats`], which withholds CI95 /
/// LSC when the sleep is zero (partitions, not replicates). The
/// returned duration is wall time including sleeps and warm-ups.
fn run_blocked<B: Bench>(
    bench: &mut B,
    pipeline: &mut BatchPipeline,
    blocks: u64,
    inner: u64,
    cfg: &RunCfg,
) -> (f64, BlockStats) {
    let outer_override = cfg.outer_override;
    let target_seconds = cfg.target_seconds;
    let (sleep_s, warmup_s) = (cfg.block_sleep_s, cfg.block_warmup_s);
    let mut dither = Dither::new();
    let mut means: Vec<f64> = Vec::with_capacity(blocks as usize);
    let run_start = std::time::Instant::now();
    for b in 0..blocks {
        let (lo, hi) = sleep_s;
        if hi > 0.0 {
            let frac = dither.rand_u64() as f64 / u64::MAX as f64;
            std::thread::sleep(std::time::Duration::from_secs_f64(lo + frac * (hi - lo)));
        }
        if warmup_s > 0.0 {
            warm_loop(
                bench,
                WarmPass::Seconds {
                    s: warmup_s,
                    chunk: 1,
                },
                None,
                |_, n| n >= 1,
            );
        }
        // Align batch boundaries to blocks: the flush moves the
        // batch clock past the sleep + warmup gap, so no batch
        // spans time the bench wasn't running.
        pipeline.flush();

        let mut sum_ps: u128 = 0;
        let mut n: u64 = 0;
        match outer_override {
            Some(outer) => {
                // Distribute the remainder over the first blocks.
                let count = outer / blocks + u64::from(b < outer % blocks);
                for _ in 0..count {
                    sum_ps += u128::from(record_sample(bench, inner, pipeline, &mut dither));
                    n += 1;
                }
            }
            None => {
                let budget = target_seconds / blocks as f64;
                let block_start = std::time::Instant::now();
                loop {
                    sum_ps += u128::from(record_sample(bench, inner, pipeline, &mut dither));
                    n += 1;
                    if block_start.elapsed().as_secs_f64() >= budget {
                        break;
                    }
                }
            }
        }
        pipeline.flush();
        if n > 0 {
            means.push(sum_ps as f64 / n as f64 / PS_PER_NS);
        }
    }
    let duration_s = run_start.elapsed().as_nanos() as f64 / 1e9;
    let stats = BlockStats::from_means(means, sleep_s.1 > 0.0);
    (duration_s, stats)
}

/// Summary of one micro-probe — the environment's time axis, the
/// warmup-side counterpart to [`BatchSummary`].
///
/// - The probe measures the apparatus alone (timer pairs), never
///   the bench, so every field describes the *box* rather than
///   the workload. That is what makes it gradeable as an
///   environment certificate: see [`crate::gauge::EnvGrade`].
/// - Values are per-pair picoseconds, each the mean of one
///   [`PROBE_GROUP_PAIRS`]-sized timed group.
/// - `t_start_s` is seconds from the *warmup* start, a different
///   clock from [`BatchSummary::t_start_s`]'s run start — the
///   two series describe adjacent phases, not one timeline.
#[derive(Debug)]
pub struct ProbeSummary {
    /// Probe start, seconds from the run's time origin.
    pub t_start_s: f64,
    /// Timed groups in the probe ([`PROBE_GROUPS`]) — the
    /// population behind `floor_q_ps` and `spread_q_ps`.
    #[allow(dead_code)]
    // OK: the quantiles' sample size, for the qualify-environment
    // selftest's table; the grade reads the quantiles themselves,
    // and its census population is `pairs`.
    pub groups: u64,
    /// Robust floor: the [`ENV_FLOOR_Q`] quantile of the probe's
    /// per-pair values (ps). The sizing input, and what the
    /// environment grade's drift and step signals track.
    pub floor_q_ps: u64,
    /// The [`ENV_SPREAD_Q`] quantile of the same values (ps) —
    /// with the floor, the probe's spread.
    pub spread_q_ps: u64,
    /// Mean per-pair value (ps).
    #[allow(dead_code)]
    // OK: the probe's central value, for the qualify-environment
    // selftest's table; no environment-grade signal reads it —
    // `bursts`, the run grade's only mean-based signal, has no
    // environment analog (see [`crate::gauge::EnvGrade`]).
    pub mean_ps: f64,
    /// Individual timer pairs in the probe — the census
    /// population, [`PROBE_GROUPS`] x [`PROBE_GROUP_PAIRS`].
    pub pairs: u64,
    /// Census: individual pairs above
    /// `max(BATCH_OVER_MULT x floor, floor + ENV_OVER_ADD_PS)`.
    pub over_pairs: u64,
}

/// Reusable scratch for the micro-probe, so a probe at every
/// batch seam allocates nothing.
struct Prober {
    /// Per-group mean pair cost (ps), sorted in place.
    groups: Vec<u64>,
    /// Every individual pair's own reading (ns) — the census
    /// population.
    pairs: Vec<u32>,
}

impl Prober {
    /// Prober with both buffers sized for one probe.
    fn new() -> Self {
        Self {
            groups: Vec::with_capacity(PROBE_GROUPS),
            pairs: Vec::with_capacity(PROBE_GROUPS * PROBE_GROUP_PAIRS),
        }
    }

    /// Run one ~256 µs micro-probe: time [`PROBE_GROUPS`] groups
    /// of [`PROBE_GROUP_PAIRS`] back-to-back timer pairs (empty
    /// timed intervals) and summarize the per-pair distribution.
    ///
    /// - `since` anchors [`ProbeSummary::t_start_s`], so every
    ///   probe in a run shares one time origin.
    /// - The floor is a low quantile, not the minimum: it rejects
    ///   one-sided interference (preemption only ever inflates a
    ///   group) while staying off the exact edge.
    /// - **Two resolutions on purpose.** Floor, spread, drift and
    ///   step read *group* means, where the timer's 1 ns quantum
    ///   is ~0.06% of a ~1.6 µs group rather than ~4% of a single
    ///   pair. The census instead counts *individual* pairs,
    ///   because a group mean hides anything smaller than
    ///   ~800 ns — an intrusion has to survive being averaged
    ///   over 64 pairs to register. A census threshold sits far
    ///   above the 1 ns quantum, so counting pairs costs the
    ///   census nothing and each pair's reading is already in
    ///   hand.
    fn probe(&mut self, since: std::time::Instant) -> ProbeSummary {
        let t_start_s = since.elapsed().as_nanos() as f64 / 1e9;
        self.groups.clear();
        self.pairs.clear();
        for _ in 0..PROBE_GROUPS {
            let group_start = std::time::Instant::now();
            for _ in 0..PROBE_GROUP_PAIRS {
                let start = std::time::Instant::now();
                self.pairs.push(start.elapsed().as_nanos() as u32);
            }
            let total_ns = group_start.elapsed().as_nanos() as f64;
            self.groups
                .push((total_ns * PS_PER_NS / PROBE_GROUP_PAIRS as f64) as u64);
        }
        self.groups.sort_unstable();

        let floor_q_ps = quantile_at(&self.groups, ENV_FLOOR_Q).max(1);
        let spread_q_ps = quantile_at(&self.groups, ENV_SPREAD_Q);
        let sum: u128 = self.groups.iter().map(|&g| u128::from(g)).sum();
        let mean_ps = sum as f64 / self.groups.len() as f64;

        let cut_ps = over_floor_cut(floor_q_ps, ENV_OVER_ADD_PS);
        let cut_ns = cut_ps as f64 / PS_PER_NS;
        let over_pairs = self
            .pairs
            .iter()
            .filter(|&&p| f64::from(p) > cut_ns)
            .count() as u64;

        ProbeSummary {
            t_start_s,
            groups: self.groups.len() as u64,
            floor_q_ps,
            spread_q_ps,
            mean_ps,
            pairs: self.pairs.len() as u64,
            over_pairs,
        }
    }
}

/// Value at quantile `q` of an already-sorted slice, clamped to
/// the last element. Empty input yields 0.
fn quantile_at(sorted: &[u64], q: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64 * q) as usize).min(sorted.len() - 1);
    sorted[idx]
}

/// Census cut for a floor: `max(BATCH_OVER_MULT x floor, floor + add)`
/// — the multiplicative rule with an additive guard so a very
/// small floor doesn't make every sample "over".
fn over_floor_cut(floor_ps: u64, add_ps: u64) -> u64 {
    ((floor_ps as f64 * BATCH_OVER_MULT) as u64).max(floor_ps + add_ps)
}

/// True exactly once per process, for the first run to ask:
/// the [`process_warm`] gate.
///
/// - The state is the warm itself: the P-state boost it wins is
///   machine state that every later run in the process inherits,
///   so a second warm would pay 1.5 s to re-establish what is
///   already true.
fn claim_process_warm() -> bool {
    static WARMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    !WARMED.swap(true, std::sync::atomic::Ordering::Relaxed)
}

/// One warmup pass's length: how long the warm loop steps the bench before it probes and
/// re-tests its exit condition.
///
/// - The two shapes mirror the harness's warms: the process and block warms step wall time,
///   checking elapsed every `chunk` steps so a cheap bench doesn't spend the pass inside
///   `Instant::now`; the per-run warmup adapts, so a pass is never one sample and never an
///   unchecked open loop.
enum WarmPass {
    /// Wall seconds per pass, elapsed checked every `chunk` steps.
    Seconds { s: f64, chunk: usize },
    /// At least `min_steps` and at least `min_s` seconds, whichever is larger, cut short at
    /// `deadline` (the warm cap), so a pathologically slow bench exits mid-pass with a
    /// diagnosis rather than hanging (bugs.md #1). The elapsed-check chunk starts at 1 and
    /// doubles to [`WARM_STEP_CHUNK_MAX`], so a cheap bench amortizes the checks out of its
    /// measured pass cost while a slow bench is checked every step.
    Adaptive {
        min_steps: u64,
        min_s: f64,
        deadline: std::time::Instant,
    },
}

/// The probe series a warm stretch appends to: the prober, the series' time origin, and the
/// vec probes land in.
struct WarmSeries<'a> {
    prober: &'a mut Prober,
    origin: std::time::Instant,
    probes: &'a mut Vec<ProbeSummary>,
}

/// The one warm loop every warm in the harness is a policy over: step `bench` in warmup passes,
/// probe after each pass when a series is given, and stop when `done` says so.
///
/// - `done` is tested before each pass with the probe series so far and the number of passes
///   completed, so a fixed-count policy counts, a budget policy reads its own clock, and the
///   warm-until-stable policy grades the series' trailing window.
/// - Probing after the pass rather than before means the first probe already has a pass of warm
///   behind it.
/// - Returns each pass's per-step wall cost (ns): the sizing input. When a series is given the
///   vec is parallel to the probes this call appended, one cost per probe.
fn warm_loop<B: Bench>(
    bench: &mut B,
    pass: WarmPass,
    mut series: Option<WarmSeries<'_>>,
    mut done: impl FnMut(&[ProbeSummary], u64) -> bool,
) -> Vec<f64> {
    let mut costs: Vec<f64> = Vec::new();
    let mut passes: u64 = 0;
    loop {
        let view: &[ProbeSummary] = series.as_ref().map_or(&[], |ws| ws.probes.as_slice());
        if done(view, passes) {
            break;
        }
        let pass_start = std::time::Instant::now();
        let mut steps: u64 = 0;
        match pass {
            WarmPass::Seconds { s, chunk } => {
                while pass_start.elapsed().as_secs_f64() < s {
                    for _ in 0..chunk {
                        black_box(bench.step());
                    }
                    steps += chunk as u64;
                }
            }
            WarmPass::Adaptive {
                min_steps,
                min_s,
                deadline,
            } => {
                let mut chunk: u64 = 1;
                loop {
                    for _ in 0..chunk {
                        black_box(bench.step());
                    }
                    steps += chunk;
                    let now = std::time::Instant::now();
                    if now >= deadline {
                        break;
                    }
                    let elapsed_s = (now - pass_start).as_secs_f64();
                    if steps >= min_steps && elapsed_s >= min_s {
                        break;
                    }
                    // Still early in the pass: this bench is cheap, so widen the
                    // chunk to keep the check cost out of the measured pass.
                    if chunk < WARM_STEP_CHUNK_MAX && elapsed_s < min_s / 4.0 {
                        chunk *= 2;
                    }
                }
            }
        }
        if steps > 0 {
            costs.push(pass_start.elapsed().as_nanos() as f64 / steps as f64);
        }
        if let Some(ws) = series.as_mut() {
            ws.probes.push(ws.prober.probe(ws.origin));
        }
        passes += 1;
    }
    costs
}

/// How the per-run warm stretch ended: the exit condition's verdict, carried into the report so
/// the stopping rule and the printed warmup certificate are one computation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WarmExit {
    /// The trailing window graded A before the cap: the run started post-ramp by construction.
    Settled,
    /// The cap elapsed with a gradeable window that never read A (the "run started unstable"
    /// signal). The window's actual grade stands in the report.
    Unstable,
    /// The cap elapsed before a gradeable window existed (a slow bench): the run proceeds
    /// uncertified. At the step costs that get here `inner` is 1 and framing is negligible, so
    /// the sizing stakes are low.
    Uncertified,
}

/// The warm exit window: the smallest trailing slice of the probe series holding at least
/// [`WARM_WINDOW_MIN_PROBES`] probes and spanning at least [`WARM_WINDOW_MIN_SECONDS`]. `None`
/// while the series cannot yet satisfy both minimums.
///
/// - This window is both the exit condition's input and the stretch the reported warmup grade
///   is computed over ([`crate::report::env_stretches`]), so exiting on A and printing A are the same
///   computation.
fn warm_window(probes: &[ProbeSummary]) -> Option<&[ProbeSummary]> {
    if probes.len() < WARM_WINDOW_MIN_PROBES {
        return None;
    }
    let end_t = probes.last()?.t_start_s;
    let cut = end_t - WARM_WINDOW_MIN_SECONDS;
    // First probe at or past the span cutoff; the window must start strictly
    // before it to span the minimum.
    let past_cut = probes.partition_point(|p| p.t_start_s < cut);
    if past_cut == 0 {
        return None;
    }
    let start = (past_cut - 1).min(probes.len() - WARM_WINDOW_MIN_PROBES);
    Some(&probes[start..])
}

/// Whether a probe window grades A on the environment signals: the warm-until-stable exit test.
fn window_grades_a(window: &[ProbeSummary]) -> bool {
    crate::gauge::EnvGrade::from_probes(window).is_some_and(|g| g.letter == 'A')
}

/// Classify how a warm stretch ended and the graded tail's probe count, from the final probe
/// series and the clock series sampled alongside it: the exit verdict [`warmup_and_probe`]
/// records and the report prints.
///
/// - A window that grades A on timing but whose clock still moved is [`WarmExit::Unstable`]:
///   steady is not settled when the box is mid-climb (the measured 7600x dwell).
fn classify_warm(
    probes: &[ProbeSummary],
    clock: &[Option<crate::freq::FreqSample>],
) -> (WarmExit, usize) {
    match warm_window(probes) {
        Some(w) => {
            let clock_w = &clock[clock.len().saturating_sub(w.len())..];
            let exit = if window_grades_a(w) && crate::freq::clock_stable(clock_w) {
                WarmExit::Settled
            } else {
                WarmExit::Unstable
            };
            (exit, w.len())
        }
        None => (WarmExit::Uncertified, probes.len()),
    }
}

/// Step `bench` for `settle_time_s` seconds, appending a micro-probe every
/// [`PROCESS_WARM_PROBE_GAP_S`]: the process warm, run once before the first bench of the
/// process.
///
/// - `settle_time_s` is the `--settle-time` budget ([`DEFAULT_SETTLE_TIME_S`] when unset); zero
///   skips the warm entirely, which is how a run measures what the warm is worth.
/// - Warming with the bench's own steps rather than a synthetic spin means the box is driven by
///   the work about to be measured, and costs the run nothing extra: these steps also warm the
///   bench's caches and branch predictors, which the warmup passes that follow were already
///   doing.
/// - The probes (and clock samples) land in the same warmup series as the ones that follow, on
///   the same time origin, so the ramp they span is what [`crate::gauge::settle`] reads and
///   what the exit window ([`warm_window`]) sits after.
fn process_warm<B: Bench>(
    bench: &mut B,
    prober: &mut Prober,
    origin: std::time::Instant,
    probes: &mut Vec<ProbeSummary>,
    clock: &mut Vec<Option<crate::freq::FreqSample>>,
    settle_time_s: f64,
) -> Vec<f64> {
    warm_loop(
        bench,
        WarmPass::Seconds {
            s: PROCESS_WARM_PROBE_GAP_S,
            chunk: PROCESS_WARM_STEP_CHUNK,
        },
        Some(WarmSeries {
            prober,
            origin,
            probes,
        }),
        |view, _| {
            sample_clock(clock, view.len());
            origin.elapsed().as_secs_f64() >= settle_time_s
        },
    )
}

/// Keep the clock series parallel to the probe series: one delivered-frequency sample per
/// probe, taken at the exit check right after the probe lands. Padding by length (rather than
/// pushing blindly) keeps the alignment across the process warm / per-run handoff.
fn sample_clock(clock: &mut Vec<Option<crate::freq::FreqSample>>, probes_len: usize) {
    while clock.len() < probes_len {
        clock.push(crate::freq::avg_freq());
    }
}

/// Everything the warm phase hands the run: the time origin, the probe series and prober, the
/// exit verdict with its window, and the sizing input the passes measured.
struct Warmed {
    /// The origin every timestamp in the run is measured from.
    origin: std::time::Instant,
    /// The warmup probe series (process warm + per-run passes).
    probes: Vec<ProbeSummary>,
    /// The prober, to keep sampling with at batch seams.
    prober: Prober,
    /// How the warm stretch ended.
    exit: WarmExit,
    /// Probe count of the exit window: the stretch the warmup grade is computed over. The
    /// whole series when no window formed ([`WarmExit::Uncertified`]).
    tail: usize,
    /// Per-step cost (ns): the minimum over the exit window's passes, so sizing reads a
    /// post-ramp number by construction.
    step_cost_ns: f64,
    /// Delivered-clock summary at warm end, when readable.
    clock: Option<WarmClock>,
    /// The clock's journey through the warm stretch, scored while the clock series is
    /// alive: [`crate::gauge::settle`]'s verdict, carried because the series itself is not.
    settle: Option<crate::gauge::Settle>,
    /// The series' `-v` profile ([`crate::gauge::clock_profile`]), compressed at the same
    /// moment and for the same reason.
    clock_profile: Option<crate::gauge::ClockProfile>,
    /// Wall seconds this run spent warming in total: the process
    /// warm when this run ran it, plus the capped per-run
    /// stretch.
    used_s: f64,
    /// This run's total warm allowance: the settle budget when
    /// this run ran the process warm, plus the cap.
    budget_s: f64,
}

/// Delivered-clock summary of the warm stretch's end, reported for information and never
/// graded: the letter stays a statement about measured time, with the clock explaining it.
#[derive(Debug, Clone, Copy)]
pub struct WarmClock {
    /// Delivered MHz at the last warm sample.
    pub end_mhz: f64,
    /// The core's hardware maximum (MHz), when exposed.
    pub max_mhz: Option<f64>,
}

/// Warm the bench until the box reads settled, measuring it the whole way: adaptive warmup
/// passes with a micro-probe after each, exiting when the trailing window grades A, or at the
/// cap.
///
/// - The exit condition and the reported warmup grade are one computation: [`warm_window`]
///   picks the trailing window, [`window_grades_a`] tests it, and the same window is the graded
///   tail in the report ([`crate::report::env_stretches`]). Hitting the cap ([`RunCfg::warm_cap_s`]) reports what the
///   window actually scored ([`WarmExit`]), never a silent proceed.
/// - The warmup pass is also the sizing pass: each pass's per-step cost is measured, and the
///   minimum over the exit window is the [`pick_inner`] step-cost input, so sizing is post-ramp
///   by construction and convergence is tested on the number actually consumed. The retired
///   estimate phase's open 1,000-step loop is gone with it (bugs.md #1); the cap deadlines
///   every pass.
/// - Floors, not means, drive the exit (the window grades probe floors), so one preemption
///   doesn't fake (in)stability; a warm box exits as soon as the window minimums are met.
/// - This is the only workload-independent stretch of the series: nothing but the warmup steps
///   has run yet. Once the bench is running, seam probes share the box with it (on a 2t bench,
///   with its worker thread), which is a truer picture of the environment the run actually had
///   and a slightly less pure measure of the machine alone.
/// - The **first** run in the process prepends `settle_time_s` of [`process_warm`] to this
///   stretch, so its series spans the box coming up to speed and every later run inherits the
///   state that won. Its probes join the same series, so a settled process warm can satisfy the
///   exit with few or no per-run passes.
fn warmup_and_probe<B: Bench>(bench: &mut B, settle_time_s: f64, warm_cap_s: f64) -> Warmed {
    let origin = std::time::Instant::now();
    let mut prober = Prober::new();
    let mut probes = Vec::with_capacity(WARM_PROBES_CAPACITY);
    let mut clock: Vec<Option<crate::freq::FreqSample>> = Vec::new();
    let ran_process_warm = settle_time_s > 0.0 && claim_process_warm();
    let mut costs = if ran_process_warm {
        process_warm(
            bench,
            &mut prober,
            origin,
            &mut probes,
            &mut clock,
            settle_time_s,
        )
    } else {
        Vec::new()
    };
    let stretch_start = std::time::Instant::now();
    let deadline = stretch_start + std::time::Duration::from_secs_f64(warm_cap_s);
    let run_costs = warm_loop(
        bench,
        WarmPass::Adaptive {
            min_steps: WARM_PASS_MIN_STEPS,
            min_s: WARM_PASS_MIN_SECONDS,
            deadline,
        },
        Some(WarmSeries {
            prober: &mut prober,
            origin,
            probes: &mut probes,
        }),
        |view, _| {
            sample_clock(&mut clock, view.len());
            let settled = warm_window(view).is_some_and(|w| {
                window_grades_a(w) && crate::freq::clock_stable(&clock[clock.len() - w.len()..])
            });
            settled || std::time::Instant::now() >= deadline
        },
    );
    let used_s = origin.elapsed().as_secs_f64();
    let budget_s = if ran_process_warm { settle_time_s } else { 0.0 } + warm_cap_s;
    costs.extend(run_costs);
    let (exit, tail) = classify_warm(&probes, &clock);
    // `costs` is parallel to `probes` (one pass per probe), so the window's
    // passes are its last `tail` entries.
    let step_cost_ns = costs
        .iter()
        .rev()
        .take(tail.max(1))
        .copied()
        .fold(f64::INFINITY, f64::min);
    // Defensive: a zero-pass exit needs process-warm probes, so `costs` should never
    // be empty here; 1 ns makes pick_inner frame-dominated, the conservative
    // direction, if it is.
    let step_cost_ns = if step_cost_ns.is_finite() {
        step_cost_ns
    } else {
        1.0
    };
    let clock_summary = clock.iter().rev().flatten().next().map(|f| WarmClock {
        end_mhz: f.khz as f64 / 1000.0,
        max_mhz: crate::freq::max_freq(f.cpu).map(|khz| khz as f64 / 1000.0),
    });
    let settle = crate::gauge::settle(&probes, &clock, tail);
    let clock_profile = crate::gauge::clock_profile(&clock);
    Warmed {
        origin,
        probes,
        prober,
        exit,
        tail,
        step_cost_ns,
        clock: clock_summary,
        settle,
        clock_profile,
        used_s,
        budget_s,
    }
}

/// Size `inner` so per-sample apparatus cost is dominated by workload:
/// `inner ~= RATIO * frame / step`.
///
/// - `frame_ns` is the last warmup probe's floor and `step_cost_ns` the exit window's best
///   pass ([`Warmed::step_cost_ns`]): order-of-magnitude sizing inputs rather than measured
///   constants; the ratio and [`MAX_INNER`] clamp absorb their imprecision.
fn pick_inner(step_cost_ns: f64, frame_ns: f64) -> u64 {
    let target = (FRAMING_DOMINATION_RATIO * frame_ns / step_cost_ns).ceil() as u64;
    target.clamp(1, MAX_INNER)
}

/// Run a fixed `outer` count of samples, seam-dithered (see
/// [`record_sample`]), through the batch pipeline.
fn run_counted<B: Bench>(
    bench: &mut B,
    pipeline: &mut BatchPipeline,
    outer: u64,
    inner: u64,
) -> f64 {
    let mut dither = Dither::new();
    let run_start = std::time::Instant::now();
    for _ in 0..outer {
        record_sample(bench, inner, pipeline, &mut dither);
    }
    run_start.elapsed().as_nanos() as f64 / 1e9
}

/// Run samples until `target_seconds` elapses, seam-dithered (see
/// [`record_sample`]), through the batch pipeline.
fn run_timed<B: Bench>(
    bench: &mut B,
    pipeline: &mut BatchPipeline,
    target_seconds: f64,
    inner: u64,
) -> f64 {
    let mut dither = Dither::new();
    let target_ns = (target_seconds * 1e9) as u128;
    let run_start = std::time::Instant::now();
    loop {
        record_sample(bench, inner, pipeline, &mut dither);
        if run_start.elapsed().as_nanos() >= target_ns {
            break;
        }
    }
    run_start.elapsed().as_nanos() as f64 / 1e9
}

/// Fresh histogram over `[HIST_LOW_PS, HIST_HIGH_PS]` at 3 sig
/// figs, resize disabled — out-of-range samples clamp (see
/// [`record_sample`]) rather than grow the histogram.
fn new_hist() -> Histogram<u64> {
    Histogram::<u64>::new_with_bounds(HIST_LOW_PS, HIST_HIGH_PS, 3).unwrap() // OK: constant bounds
}

/// Summary of one time-ordered batch of samples — the run's
/// time axis, which the histogram destroys. Feeds the batch
/// gauge (drift from floor movement, bursts localized to their
/// batch, interference rate from census counts).
#[derive(Debug)]
pub struct BatchSummary {
    /// Batch start, seconds from run start.
    pub t_start_s: f64,
    /// Batch end (flush time), seconds from run start.
    #[allow(dead_code)]
    // OK: bounds the batch for the qualify-environment selftest's
    // per-batch table; the gauge locates events by `t_start_s`.
    pub t_end_s: f64,
    /// Samples in the batch.
    pub count: u64,
    /// Minimum per-call value (ps) — the batch's fastest sample.
    #[allow(dead_code)]
    // OK: the batch's extreme record, for the qualify-environment
    // selftest's table; the gauge grades movement on the robust
    // `floor_q_ps` instead (see [`BATCH_FLOOR_Q`]).
    pub floor_ps: u64,
    /// Robust floor: the [`BATCH_FLOOR_Q`] quantile of the
    /// batch's per-call values (ps). What the gauge's drift and
    /// step signals track.
    pub floor_q_ps: u64,
    /// Mean per-call value (ps).
    pub mean_ps: f64,
    /// Maximum per-call value (ps).
    #[allow(dead_code)]
    // OK: the run's worst excursion, localized to its batch — for
    // the qualify-environment selftest; no gauge signal reads it.
    pub max_ps: u64,
    /// Census: samples above
    /// `max(BATCH_OVER_MULT x floor, floor + BATCH_OVER_ADD_PS)`.
    pub over_floor: u64,
}

/// Time-ordered batch pipeline: samples land in a raw buffer;
/// a full (or time-expired) batch is summarized for the gauge
/// and bulk-recorded into the histogram, and the buffer is
/// reused. Memory stays bounded at one buffer plus the small
/// per-batch summaries.
struct BatchPipeline {
    buf: Vec<u64>,
    hist: Histogram<u64>,
    summaries: Vec<BatchSummary>,
    run_start: std::time::Instant,
    batch_start_s: f64,
    /// Micro-probe scratch, run once per non-empty flush.
    prober: Prober,
    /// The environment series: warmup probes, then one per batch
    /// seam. Shares [`BatchSummary`]'s time origin, so the two
    /// series line up sample for sample on one axis.
    probes: Vec<ProbeSummary>,
    /// Whether to probe at each seam (`--no-env-probe` clears
    /// it, leaving the warmup stretch alone).
    seam_probes: bool,
    /// One delivered-clock sample per non-empty flush, on the same
    /// time axis as the summaries. Never gated by `seam_probes`:
    /// a single sysfs read is orders cheaper than the ~256 us
    /// micro-probe that gate exists for.
    seam_clock: Vec<SeamClock>,
}

impl BatchPipeline {
    /// Pipeline continuing an in-progress run: `origin` is the
    /// timestamp origin (the warmup start, so batches and probes
    /// share one clock), and `probes` the warmup stretch of the
    /// environment series that `prober` keeps extending.
    fn new(
        origin: std::time::Instant,
        prober: Prober,
        probes: Vec<ProbeSummary>,
        seam_probes: bool,
    ) -> Self {
        let batch_start_s = origin.elapsed().as_nanos() as f64 / 1e9;
        Self {
            buf: Vec::with_capacity(BATCH_SAMPLES),
            hist: new_hist(),
            summaries: Vec::new(),
            run_start: origin,
            batch_start_s,
            prober,
            probes,
            seam_probes,
            seam_clock: Vec::new(),
        }
    }

    /// Seconds since the pipeline was created.
    fn elapsed_s(&self) -> f64 {
        self.run_start.elapsed().as_nanos() as f64 / 1e9
    }

    /// Append one per-call sample (ps); flushes when the buffer
    /// fills, or on a 1024-sample cadence when the batch has
    /// spanned [`BATCH_TARGET_SECONDS`].
    fn push(&mut self, per_call_ps: u64) {
        self.buf.push(per_call_ps);
        let len = self.buf.len();
        if len >= BATCH_SAMPLES
            || (len & BATCH_CHECK_MASK == 0
                && self.elapsed_s() - self.batch_start_s >= BATCH_TARGET_SECONDS)
        {
            self.flush();
        }
    }

    /// Summarize and bulk-record the current batch, then reset
    /// the buffer. No-op on an empty buffer except moving the
    /// batch clock (used at block boundaries so sleep gaps
    /// never span a batch).
    fn flush(&mut self) {
        let t_end_s = self.elapsed_s();
        if self.buf.is_empty() {
            self.batch_start_s = t_end_s;
            return;
        }
        // Robust floor first: the partial sort reorders the
        // buffer, which none of the passes below depend on.
        let q_idx = ((self.buf.len() as f64 * BATCH_FLOOR_Q) as usize).min(self.buf.len() - 1);
        let (_, &mut floor_q_ps, _) = self.buf.select_nth_unstable(q_idx);

        let mut floor_ps = u64::MAX;
        let mut max_ps = 0u64;
        let mut sum: u128 = 0;
        for &v in &self.buf {
            floor_ps = floor_ps.min(v);
            max_ps = max_ps.max(v);
            sum += u128::from(v);
        }
        let over_cut = ((floor_q_ps as f64 * BATCH_OVER_MULT) as u64)
            .max(floor_q_ps.saturating_add(BATCH_OVER_ADD_PS));
        let mut over_floor = 0u64;
        for &v in &self.buf {
            self.hist.saturating_record(v);
            if v > over_cut {
                over_floor += 1;
            }
        }
        self.summaries.push(BatchSummary {
            t_start_s: self.batch_start_s,
            t_end_s,
            count: self.buf.len() as u64,
            floor_ps,
            floor_q_ps,
            mean_ps: sum as f64 / self.buf.len() as f64,
            max_ps,
            over_floor,
        });
        self.buf.clear();
        // The seam's delivered-clock read, ahead of the probe so
        // it lands as close to the batch it caps as possible.
        if let Some(f) = crate::freq::avg_freq() {
            self.seam_clock.push(SeamClock {
                t_ns: self.run_start.elapsed().as_nanos() as u64,
                cpu: f.cpu,
                khz: f.khz,
            });
        }
        // Probe the box in the seam the summary already opened:
        // the bench is stopped either way, so the environment
        // series gets the run's whole time span for a fraction of
        // a gap that exists regardless.
        if self.seam_probes {
            self.probes.push(self.prober.probe(self.run_start));
        }
        self.batch_start_s = self.elapsed_s();
    }

    /// Flush the tail batch and yield the histogram, the batch
    /// summaries, the environment probe series, and the seam
    /// clock series.
    #[allow(clippy::type_complexity)]
    // OK: a one-consumer private seam; naming a struct for it would outlive its use.
    fn finish(
        mut self,
    ) -> (
        Histogram<u64>,
        Vec<BatchSummary>,
        Vec<ProbeSummary>,
        Vec<SeamClock>,
    ) {
        self.flush();
        (self.hist, self.summaries, self.probes, self.seam_clock)
    }
}

/// Time one sample (`inner` back-to-back calls), divide down to a
/// per-call value in **picoseconds**, and record it, clamping at
/// the histogram bounds — a suspend-inflated or wedged sample
/// must not panic a long run ([`crate::report::warn_invalid`] flags it instead).
///
/// - The seam dither (a random sub-quantum spin before the timer
///   pair, outside the timed interval) stops the run's aggregate
///   means from carrying a coherent ±quantum phase bias — up to
///   ~±2% on fast benches (see
///   notes/design.md#dithering-random-phase-injection).
fn record_sample<B: Bench>(
    bench: &mut B,
    inner: u64,
    pipeline: &mut BatchPipeline,
    dither: &mut Dither,
) -> u64 {
    dither.spin();
    let start = std::time::Instant::now();
    for _ in 0..inner {
        black_box(bench.step());
    }
    let elapsed_ps = start.elapsed().as_nanos().saturating_mul(1000);
    let per_call_ps = round_elapsed_ps(elapsed_ps, inner);
    pipeline.push(per_call_ps);
    per_call_ps
}

/// Per-call value: `elapsed_ps / inner`, rounded to nearest, in
/// u128 so an hours-long suspend-inflated sample can't overflow
/// the ×1000 ns→ps conversion; the cast clamps at u64::MAX and
/// `saturating_record` clamps again at the histogram bound.
fn round_elapsed_ps(elapsed_ps: u128, inner: u64) -> u64 {
    let inner = inner as u128;
    ((elapsed_ps + inner / 2) / inner).min(u64::MAX as u128) as u64
}

/// Paired run-start readings of `CLOCK_MONOTONIC` and
/// `CLOCK_BOOTTIME`, for detecting a system suspend that spanned
/// a measurement run.
///
/// - `CLOCK_MONOTONIC` freezes while the system is suspended;
///   `CLOCK_BOOTTIME` keeps counting — the divergence of the two
///   elapsed times is the time spent suspended.
/// - Uses std `Instant` (`CLOCK_MONOTONIC`), not `minstant`: we
///   think the TSC keeps counting across s2idle suspend, which is
///   exactly the clock behavior being detected.
struct ClockPair {
    mono: std::time::Instant,
    boot_ns: u64,
}

impl ClockPair {
    /// Capture both clocks now.
    fn now() -> Self {
        Self {
            mono: std::time::Instant::now(),
            boot_ns: boottime_ns(),
        }
    }

    /// Seconds the system spent suspended since [`now`][Self::now]:
    /// boottime elapsed minus monotonic elapsed (~0 when no
    /// suspend occurred).
    fn suspended_s(&self) -> f64 {
        let boot_s = (boottime_ns() - self.boot_ns) as f64 / 1e9;
        let mono_s = self.mono.elapsed().as_nanos() as f64 / 1e9;
        boot_s - mono_s
    }
}

/// Current `CLOCK_BOOTTIME` reading in nanoseconds.
fn boottime_ns() -> u64 {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: clock_gettime only writes `ts`; CLOCK_BOOTTIME is
    // always valid on Linux.
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut ts) };
    assert_eq!(rc, 0, "clock_gettime(CLOCK_BOOTTIME) failed");
    ts.tv_sec as u64 * 1_000_000_000 + ts.tv_nsec as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::env_stretches;

    /// A pipeline with a fresh clock and an empty environment
    /// series — the shape `run_adaptive` builds after warmup.
    fn test_pipeline() -> BatchPipeline {
        BatchPipeline::new(std::time::Instant::now(), Prober::new(), Vec::new(), true)
    }

    #[test]
    fn batch_pipeline_flushes_full_batches() {
        let mut p = test_pipeline();
        let n = BATCH_SAMPLES * 2 + 100;
        for _ in 0..n {
            p.push(1_000);
        }
        let (hist, batches, probes, _) = p.finish();
        assert_eq!(probes.len(), batches.len(), "one probe per non-empty flush");
        assert_eq!(hist.len(), n as u64);
        assert!(
            batches.len() >= 3,
            "expected >= 3 batches, got {}",
            batches.len()
        );
        let total: u64 = batches.iter().map(|b| b.count).sum();
        assert_eq!(total, n as u64);
        for b in &batches {
            assert_eq!(b.floor_ps, 1_000);
            assert_eq!(b.max_ps, 1_000);
            assert!((b.mean_ps - 1_000.0).abs() < f64::EPSILON);
            assert_eq!(b.over_floor, 0);
            assert!(b.t_end_s >= b.t_start_s);
        }
    }

    #[test]
    fn batch_summary_census_counts_spikes() {
        let mut p = test_pipeline();
        // Floor 10 ns (10_000 ps); threshold is
        // max(1.5x, +50 ns) = 60_000 ps. One sample above it,
        // one between floor and threshold (not counted).
        for _ in 0..100 {
            p.push(10_000);
        }
        p.push(55_000);
        p.push(2_000_000);
        let (hist, batches, _, _) = p.finish();
        assert_eq!(hist.len(), 102);
        assert_eq!(batches.len(), 1);
        let b = &batches[0];
        assert_eq!(b.count, 102);
        assert_eq!(b.floor_ps, 10_000);
        assert_eq!(b.max_ps, 2_000_000);
        assert_eq!(b.over_floor, 1);
    }

    #[test]
    fn batch_flush_on_empty_moves_clock_only() {
        let mut p = test_pipeline();
        p.flush();
        p.push(1_000);
        let (hist, batches, probes, _) = p.finish();
        assert_eq!(hist.len(), 1);
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].count, 1);
        // The empty flush moved the clock without probing: a
        // probe belongs to a batch, and there was no batch.
        assert_eq!(probes.len(), 1);
    }

    /// One probe at `at` seconds with the given floor (ps); the
    /// upper quantile sits a flat 0.8% above it, so `spread`
    /// never drives these cases.
    fn probe(at: f64, floor_ps: u64) -> ProbeSummary {
        ProbeSummary {
            t_start_s: at,
            groups: PROBE_GROUPS as u64,
            floor_q_ps: floor_ps,
            spread_q_ps: floor_ps * 1008 / 1000,
            mean_ps: floor_ps as f64,
            pairs: (PROBE_GROUPS * PROBE_GROUP_PAIRS) as u64,
            over_pairs: 0,
        }
    }

    /// A process-warm-shaped warmup stretch: probes every 10 ms
    /// across [`DEFAULT_SETTLE_TIME_S`], ramping 30 -> 24 ns until
    /// `ramp_end_s` and holding 24 ns after.
    fn warm_stretch(ramp_end_s: f64) -> Vec<ProbeSummary> {
        let n = (DEFAULT_SETTLE_TIME_S / PROCESS_WARM_PROBE_GAP_S) as usize;
        (0..n)
            .map(|i| {
                let at = i as f64 * PROCESS_WARM_PROBE_GAP_S;
                let floor = if at < ramp_end_s {
                    30_000 - (6_000.0 * at / ramp_end_s) as u64
                } else {
                    24_000
                };
                probe(at, floor)
            })
            .collect()
    }

    #[test]
    fn warmup_ramp_does_not_fault_a_clean_run() {
        // The box comes up to speed 0.8 s into a 1.5 s warm; the
        // run then holds 24 ns for two seconds.
        let mut probes = warm_stretch(0.8);
        let warmup = probes.len();
        for i in 0..40 {
            probes.push(probe(DEFAULT_SETTLE_TIME_S + i as f64 * 0.05, 24_000));
        }

        // Blended, the boundary reads as a large step — the
        // failure the split stretches exist to prevent.
        let blended = crate::gauge::EnvGrade::from_probes(&probes).expect("graded");
        assert!(
            blended.step_frac > 0.10,
            "expected a blended series to invent a step, got {}",
            blended.step_frac
        );

        // Split, both stretches are flat: the exit window sits
        // wholly after the ramp and the run never moved.
        let tail_len = warm_window(&probes[..warmup]).expect("window forms").len();
        let (warm, tail, during) = env_stretches(&probes, warmup, tail_len);
        assert_eq!(warm.len(), warmup);
        let first_tail = tail.first().expect("tail is non-empty").t_start_s;
        assert!(
            first_tail > 0.8,
            "tail window straddles the ramp, starting at {first_tail}"
        );
        let warm_grade = crate::gauge::EnvGrade::from_probes(tail).expect("graded");
        let run_grade = crate::gauge::EnvGrade::from_probes(during).expect("graded");
        assert_eq!(warm_grade.letter, 'A');
        assert_eq!(run_grade.letter, 'A');
    }

    #[test]
    fn warm_window_enforces_both_minimums() {
        // Coarse cadence (10 ms): the count minimum rules, so the window is exactly
        // WARM_WINDOW_MIN_PROBES even though fewer probes would already span the minimum.
        let coarse: Vec<ProbeSummary> = (0..30)
            .map(|i| probe(i as f64 * PROCESS_WARM_PROBE_GAP_S, 24_000))
            .collect();
        let w = warm_window(&coarse).expect("window forms");
        assert_eq!(w.len(), WARM_WINDOW_MIN_PROBES);
        let span = w.last().expect("non-empty").t_start_s - w[0].t_start_s;
        assert!(span >= WARM_WINDOW_MIN_SECONDS, "span {span}s too short");

        // Fine cadence (1 ms): the span minimum rules, so the window holds many more
        // probes than the count minimum.
        let fine: Vec<ProbeSummary> = (0..100).map(|i| probe(i as f64 * 0.001, 24_000)).collect();
        let w = warm_window(&fine).expect("window forms");
        assert!(
            w.len() > WARM_WINDOW_MIN_PROBES,
            "span minimum should widen the window, got {} probes",
            w.len()
        );
        let span = w.last().expect("non-empty").t_start_s - w[0].t_start_s;
        assert!(span >= WARM_WINDOW_MIN_SECONDS, "span {span}s too short");

        // A series that cannot satisfy the span yields no window at all, never a
        // vacuous short one.
        let short: Vec<ProbeSummary> = (0..16).map(|i| probe(i as f64 * 0.001, 24_000)).collect();
        assert!(warm_window(&short).is_none());
    }

    #[test]
    fn settle_time_finds_the_ramp_end() {
        // The observable the letter no longer carries: warmup
        // absorbed the ramp, and this says how long that took.
        let probes = warm_stretch(0.8);
        let tail_len = warm_window(&probes).expect("window forms").len();
        let (warm, _, _) = env_stretches(&probes, probes.len(), tail_len);
        let settled = match crate::gauge::settle(warm, &[], tail_len).expect("graded") {
            crate::gauge::Settle::At { t_s, .. } => t_s,
            crate::gauge::Settle::Never { .. } => panic!("a warm ending flat should settle"),
        };
        // Within a window of the ramp's end, and biased early
        // rather than late: the forward window that first reads
        // settled straddles the last of the ramp, and the last
        // 1% of a ramp is inside the band anyway.
        assert!(
            settled <= 0.8 && settled > 0.8 - 0.1,
            "settled at {settled}s, wanted ~0.8s"
        );
    }

    #[test]
    fn a_warm_still_moving_at_the_cap_is_unstable() {
        // The box is still ramping when the cap runs out: the exit window never grades
        // A, so the exit verdict (not gauge::settle) reports "not settled". 100 ps per
        // probe keeps the window's drift above the A cutoff right through the end.
        let n = (DEFAULT_SETTLE_TIME_S / PROCESS_WARM_PROBE_GAP_S) as usize;
        let probes: Vec<ProbeSummary> = (0..n)
            .map(|i| probe(i as f64 * PROCESS_WARM_PROBE_GAP_S, 40_000 - i as u64 * 100))
            .collect();
        let (exit, _) = classify_warm(&probes, &[]);
        assert_eq!(exit, WarmExit::Unstable);
    }

    #[test]
    fn movement_inside_the_exit_window_blocks_the_exit() {
        // Settled for a second, then a step inside the exit window: the window's step
        // signal blocks the A, so a warm that just moved cannot exit settled (the
        // stopping rule and the letter are one computation).
        let n = (DEFAULT_SETTLE_TIME_S / PROCESS_WARM_PROBE_GAP_S) as usize;
        let late = n - 4;
        let probes: Vec<ProbeSummary> = (0..n)
            .map(|i| {
                let floor = if i < late { 24_000 } else { 26_000 };
                probe(i as f64 * PROCESS_WARM_PROBE_GAP_S, floor)
            })
            .collect();
        let window = warm_window(&probes).expect("window forms");
        assert!(window.len() > 4, "the step must land inside the window");
        assert!(!window_grades_a(window));
        let (exit, _) = classify_warm(&probes, &[]);
        assert_eq!(exit, WarmExit::Unstable);
    }

    #[test]
    fn a_steady_dwell_with_a_moving_clock_is_unstable() {
        // The measured 7600x case: timing dead flat (a dwell is steady) while the
        // delivered clock climbs +12% inside the window. Timing-only would exit
        // Settled; the clock gate holds the verdict at Unstable.
        let n = 30;
        let probes: Vec<ProbeSummary> = (0..n)
            .map(|i| probe(i as f64 * PROCESS_WARM_PROBE_GAP_S, 24_000))
            .collect();
        let clock: Vec<Option<crate::freq::FreqSample>> = (0..n)
            .map(|i| {
                Some(crate::freq::FreqSample {
                    cpu: 0,
                    khz: 4_841_000 + i as u64 * 20_000,
                })
            })
            .collect();
        let (timing_only, _) = classify_warm(&probes, &[]);
        assert_eq!(timing_only, WarmExit::Settled, "dwell fools timing alone");
        let (gated, _) = classify_warm(&probes, &clock);
        assert_eq!(gated, WarmExit::Unstable);

        // A flat clock on the same timing exits settled, and a mid-window migration
        // falls back to timing-only.
        let flat: Vec<Option<crate::freq::FreqSample>> = (0..n)
            .map(|_| {
                Some(crate::freq::FreqSample {
                    cpu: 0,
                    khz: 5_440_000,
                })
            })
            .collect();
        assert_eq!(classify_warm(&probes, &flat).0, WarmExit::Settled);
        // The migration must land inside the exit window (its last
        // WARM_WINDOW_MIN_PROBES probes) to be seen.
        let migrated: Vec<Option<crate::freq::FreqSample>> = (0..n)
            .map(|i| {
                Some(crate::freq::FreqSample {
                    cpu: usize::from(i >= n - 4),
                    khz: 4_841_000 + i as u64 * 20_000,
                })
            })
            .collect();
        assert_eq!(classify_warm(&probes, &migrated).0, WarmExit::Settled);
    }

    #[test]
    fn a_short_series_is_uncertified() {
        // The cap arrived before the window minimums were met (a slow bench): no
        // certificate, by construction.
        let probes: Vec<ProbeSummary> = (0..4)
            .map(|i| probe(i as f64 * PROCESS_WARM_PROBE_GAP_S, 24_000))
            .collect();
        let (exit, tail) = classify_warm(&probes, &[]);
        assert_eq!(exit, WarmExit::Uncertified);
        assert_eq!(tail, 4, "grades whatever it has");
    }

    #[test]
    fn a_box_that_starts_settled_settles_at_zero() {
        // A warm already settled from its first probe: the whole stretch is the graded
        // tail (an uncertified-style short series grades whatever it has) and settle
        // reads zero.
        let probes: Vec<ProbeSummary> = (0..16).map(|i| probe(i as f64 * 0.001, 24_000)).collect();
        let (warm, tail, _) = env_stretches(&probes, probes.len(), probes.len());
        assert_eq!(tail.len(), 16, "short stretch grades whole");
        assert_eq!(
            crate::gauge::settle(warm, &[], tail.len()),
            Some(crate::gauge::Settle::At {
                t_s: 0.0,
                settled_frac: 1.0,
                start_ghz: None,
                ghz: None,
                rating: None
            })
        );
    }

    #[test]
    fn clock_pair_no_suspend_gap() {
        let clocks = ClockPair::now();
        let gap = clocks.suspended_s();
        assert!(gap.abs() < 0.5, "unexpected clock divergence: {gap}");
    }

    #[test]
    fn saturating_record_clamps_above_bound() {
        let mut hist = new_hist();
        hist.saturating_record(HIST_HIGH_PS * 2);
        assert_eq!(hist.len(), 1);
        assert!(hist.max() >= HIST_HIGH_PS);
    }

    #[test]
    fn round_elapsed_ps_keeps_sub_ns_precision() {
        // 156 ns over 33 calls = 4.727 ns/call — recorded as
        // 4,727 ps instead of the 5 ns that ns-rounding gave.
        assert_eq!(round_elapsed_ps(156_000, 33), 4_727);
        // Saturates instead of overflowing on absurd inputs.
        assert_eq!(round_elapsed_ps(u128::MAX - 1, 1), u64::MAX);
    }
}
