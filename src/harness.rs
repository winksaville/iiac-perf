//! Generic bench driver: the [`Bench`] trait, adaptive outer/inner
//! loop sizing, and the band-histogram report.

use std::hint::black_box;

use hdrhistogram::Histogram;

use crate::bands::{self, BandLabels};
use crate::overhead::{Dither, Overhead};

const WARMUP: u64 = 10_000;
const ESTIMATE_STEPS: u64 = 1_000;
const ESTIMATE_SAMPLES: usize = 5;
const FRAMING_DOMINATION_RATIO: f64 = 10.0;
const MAX_INNER: u64 = 1_000;

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

/// Micro-probes taken during warmup, spread evenly across it —
/// the leading stretch of the environment series, and the only
/// stretch measured before the bench starts running (see
/// [`crate::gauge::EnvGrade`]). The rest of the series is
/// sampled at batch seams across the whole run.
const WARMUP_PROBES: usize = 16;

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
const BATCH_SAMPLES: usize = 65_536;

/// Time-based batch flush (seconds): a partial batch flushes
/// once it spans this long, so slow benches still get a usable
/// time axis (drift/burst localization) from few samples.
const BATCH_TARGET_SECONDS: f64 = 0.05;

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
const BATCH_FLOOR_Q: f64 = 0.10;

/// Census threshold: a batch sample is "over floor" above
/// `max(BATCH_OVER_MULT x floor, floor + BATCH_OVER_ADD_PS)` —
/// the calibration disturbed-sample definition, applied per batch
/// against the batch's own [`BATCH_FLOOR_Q`] floor.
///
/// - Measured against the raw min instead, the census was
///   meaningless on any bench with a low tail: mpsc-2t batches
///   whose min landed on a 0.9 µs fast path (against a 6.5 µs
///   floor) counted 99.9% of their samples "over floor", and the
///   ones whose min landed normally counted 1%.
const BATCH_OVER_MULT: f64 = 1.5;

/// Additive part of the census threshold (50 ns in ps).
const BATCH_OVER_ADD_PS: u64 = 50_000;

/// Sleep-separated blocks: random sleep bounds (ms) between
/// blocks. Randomized so block boundaries don't phase-lock with
/// kernel ticks or workload periodicity; long enough to let the
/// scheduler / frequency state re-roll.
const BLOCK_SLEEP_MS_MIN: u64 = 1;
const BLOCK_SLEEP_MS_MAX: u64 = 10;

/// Unrecorded post-wake warm-up per block (seconds) — each wake
/// pays a frequency ramp + cache refill (the measured
/// cold-calibration effect), which must not leak into the block's
/// samples.
const BLOCK_WARMUP_SECONDS: f64 = 0.002;

/// Histogram value bounds: 1 ps to 60 s at 3 sig figs. Values
/// are recorded in **picoseconds** — the timer reads integer ns,
/// but dividing a sample by `inner` in ps keeps the true sub-ns
/// per-call precision that ns recording truncated (a 4.7 ns call
/// no longer rounds to 5). The high bound is a sane-world
/// ceiling for one recorded sample, not a technical limit —
/// [`record_sample`] clamps above it and [`warn_invalid`] flags
/// the run.
const HIST_LOW_PS: u64 = 1;
const HIST_HIGH_PS: u64 = 60_000_000_000_000;

/// Picoseconds per nanosecond: recorded values are ps, display
/// is ns.
const PS_PER_NS: f64 = 1000.0;

/// Label-column width for the two gauge blocks, wide enough for
/// the longest label (`env worst case:`) plus a gap. Shared so
/// the environment and run rows line up under each other.
const GAUGE_LABEL_W: usize = 21;

/// `CLOCK_BOOTTIME` minus `CLOCK_MONOTONIC` elapsed divergence
/// (seconds) at or above which [`warn_invalid`] reports that the
/// system suspended during the run.
const SUSPEND_WARN_S: f64 = 1.0;

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
    /// Calibrated apparatus overhead (framing + loop/iter) used to
    /// compute the `adjusted` mean columns in the report.
    pub overhead: &'a Overhead,
    /// Wall-clock seconds budget for time-based runs. Ignored when
    /// `outer_override` is set.
    pub target_seconds: f64,
    /// Force a fixed outer-loop count, bypassing the time budget.
    pub outer_override: Option<u64>,
    /// Force a fixed inner-loop count, bypassing the
    /// overhead-dominated auto-sizing.
    pub inner_override: Option<u64>,
    /// Core pool for thread pinning. Indexed positionally with
    /// wrap-around via [`core_for`][RunCfg::core_for]; empty means
    /// no pinning.
    pub pin_cores: &'a [usize],
    /// When set, [`crate::tprobe::TProbe::report`] emits raw TSC
    /// ticks instead of nanoseconds. Plumbed from the `-t/--ticks`
    /// CLI flag.
    pub report_ticks: bool,
    /// Sample the environment at every batch seam, so the
    /// environment grade spans the whole run. Cleared by
    /// `--no-env-probe`, which leaves only the warmup probes.
    /// Plumbed from the CLI.
    pub seam_probes: bool,
    /// Band-label style for [`print_report`] histogram rows.
    /// Plumbed from the `--band-labels` CLI flag.
    pub band_labels: BandLabels,
    /// Decimal digits on [`print_report`] time columns. Plumbed
    /// from the `--decimals` CLI flag (default 1; 0 restores
    /// integers; 3 is the ps recording floor).
    pub decimals: usize,
    /// Split the run into this many sleep-separated blocks and
    /// report block-replication stats (mean ± 95% CI, LSC).
    /// Plumbed from the `--blocks` CLI flag; `None` = single
    /// continuous run. See
    /// notes/design.md#within-invocation-replication-sleep-separated-blocks.
    pub blocks: Option<u64>,
}

impl RunCfg<'_> {
    /// CPU id for the bench's `thread_idx`-th thread, using
    /// wrap-around over the pool. Returns `None` when the pool is
    /// empty so callers can treat unpinned and pinned runs uniformly.
    pub fn core_for(&self, thread_idx: usize) -> Option<usize> {
        if self.pin_cores.is_empty() {
            None
        } else {
            Some(self.pin_cores[thread_idx % self.pin_cores.len()])
        }
    }
}

/// Block-replication statistics from a sleep-separated run —
/// each block is a mini-run (own sleep re-roll + warm-up), so the
/// spread of block means yields an honest-per-invocation CI and
/// LSC. See
/// notes/design.md#within-invocation-replication-sleep-separated-blocks.
#[derive(Debug)]
pub struct BlockStats {
    /// Number of blocks (Y).
    pub blocks: u64,
    /// Mean of the per-block means, ns.
    pub mean_ns: f64,
    /// 95% confidence half-width on `mean_ns`:
    /// `t(0.975, Y-1) * s / sqrt(Y)`, ns.
    pub ci95_ns: f64,
    /// Least significant change vs an equal-Y run of another
    /// implementation: `t(0.975, 2Y-2) * s * sqrt(2/Y)`, ns.
    pub lsc_ns: f64,
}

impl BlockStats {
    /// Fit from per-block means (ns). Caller guarantees
    /// `means.len() >= 2` (the CLI enforces `--blocks 2..`).
    fn from_means(means: &[f64]) -> BlockStats {
        let y = means.len() as f64;
        let mean = means.iter().sum::<f64>() / y;
        let var = means.iter().map(|m| (m - mean) * (m - mean)).sum::<f64>() / (y - 1.0);
        let s = var.sqrt();
        let yy = means.len() as u64;
        BlockStats {
            blocks: yy,
            mean_ns: mean,
            ci95_ns: t975(yy - 1) * s / y.sqrt(),
            lsc_ns: t975(2 * yy - 2) * s * (2.0 / y).sqrt(),
        }
    }
}

/// Two-sided 95% Student-t quantile (`t(0.975, df)`), table for
/// df ≤ 30, then the conservative 2.0 (the true value falls from
/// 2.042 toward the normal 1.96).
fn t975(df: u64) -> f64 {
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
/// histogram plus the metadata [`print_report`] needs and the
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
    /// [`ClockPair`]); [`print_report`] flags poisoned stats
    /// when non-trivial.
    pub suspended_s: f64,
    /// Block-replication stats — `Some` only for `--blocks`
    /// runs.
    pub block_stats: Option<BlockStats>,
    /// Time-ordered per-batch summaries from the pipeline.
    pub batches: Vec<BatchSummary>,
    /// Time-ordered micro-probe summaries from warmup — the
    /// environment grade's input, measured before the workload's
    /// character enters the numbers.
    pub probes: Vec<ProbeSummary>,
}

/// Drive `bench` against `cfg` and return a [`RunOutput`].
///
/// After a fixed warmup, `inner` is auto-sized so apparatus framing
/// doesn't dominate (skipped when `cfg.inner_override` is set). The
/// outer loop runs either for `cfg.outer_override` iterations or
/// until `cfg.target_seconds` elapses — as one continuous run, or
/// split into `cfg.blocks` sleep-separated blocks (`block_stats`
/// is `Some` only then). Samples flow through the
/// [`BatchPipeline`], so the output carries the run's time axis
/// as per-batch summaries alongside the histogram.
pub fn run_adaptive<B: Bench>(bench: &mut B, cfg: &RunCfg) -> RunOutput {
    let (origin, warm_probes, prober) = warmup_and_probe(bench);

    let step_cost_ns = estimate_step_cost(bench);
    // The last warmup probe is the most-warmed one, so sizing
    // reads a post-warmup frame by construction.
    let frame_ns = match warm_probes.last() {
        Some(p) => (p.floor_q_ps as f64 / PS_PER_NS).max(1.0),
        None => 1.0,
    };
    let inner = cfg
        .inner_override
        .unwrap_or_else(|| pick_inner(step_cost_ns, frame_ns));

    let mut pipeline = BatchPipeline::new(origin, prober, warm_probes, cfg.seam_probes);
    let clocks = ClockPair::now();
    let (block_stats, duration_s) = match cfg.blocks {
        Some(blocks) => {
            let (duration_s, stats) = run_blocked(
                bench,
                &mut pipeline,
                blocks,
                cfg.outer_override,
                cfg.target_seconds,
                inner,
            );
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
    let (hist, batches, probes) = pipeline.finish();
    let outer = match cfg.outer_override {
        Some(outer) if cfg.blocks.is_none() => outer,
        _ => hist.len(),
    };
    RunOutput {
        hist,
        outer,
        inner,
        duration_s,
        suspended_s: clocks.suspended_s(),
        block_stats,
        batches,
        probes,
    }
}

/// Run `blocks` sleep-separated blocks: each block sleeps a
/// random [`BLOCK_SLEEP_MS_MIN`]..=[`BLOCK_SLEEP_MS_MAX`] ms
/// (re-rolls scheduler / frequency / mode-mix state), steps
/// unrecorded for [`BLOCK_WARMUP_SECONDS`] (post-wake ramp), then
/// measures its share of the budget (`outer / blocks` samples, or
/// `target_seconds / blocks`). All samples land in one histogram;
/// per-block means feed [`BlockStats`]. The returned duration is
/// wall time including sleeps and warm-ups.
fn run_blocked<B: Bench>(
    bench: &mut B,
    pipeline: &mut BatchPipeline,
    blocks: u64,
    outer_override: Option<u64>,
    target_seconds: f64,
    inner: u64,
) -> (f64, BlockStats) {
    let mut dither = Dither::new();
    let mut means: Vec<f64> = Vec::with_capacity(blocks as usize);
    let run_start = std::time::Instant::now();
    for b in 0..blocks {
        let ms =
            BLOCK_SLEEP_MS_MIN + dither.rand_u64() % (BLOCK_SLEEP_MS_MAX - BLOCK_SLEEP_MS_MIN + 1);
        std::thread::sleep(std::time::Duration::from_millis(ms));

        let warm_start = std::time::Instant::now();
        while warm_start.elapsed().as_secs_f64() < BLOCK_WARMUP_SECONDS {
            black_box(bench.step());
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
    let stats = BlockStats::from_means(&means);
    (duration_s, stats)
}

fn estimate_step_cost<B: Bench>(bench: &mut B) -> f64 {
    let mut samples: Vec<f64> = (0..ESTIMATE_SAMPLES)
        .map(|_| {
            let start = std::time::Instant::now();
            for _ in 0..ESTIMATE_STEPS {
                black_box(bench.step());
            }
            let total = start.elapsed().as_nanos() as u64;
            total as f64 / ESTIMATE_STEPS as f64
        })
        .collect();
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    samples[ESTIMATE_SAMPLES / 2]
}

/// Summary of one micro-probe — the environment's time axis, the
/// warmup-side counterpart to [`BatchSummary`].
///
/// - The probe measures the apparatus alone (timer pairs), never
///   the bench, so every field describes the *box* rather than
///   the workload. That is what makes it gradeable as an
///   environment certificate: see [`crate::gauge::EnvGrade`].
/// - Values are per-pair picoseconds, each the mean of one
///   [`MICRO_PROBE_GROUP`]-sized timed group.
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
    // OK: the quantiles' sample size, for the 0.23.0-5 settle
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
    // OK: the probe's central value, for the 0.23.0-5 settle
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

/// Warm the bench and measure the box at the same time: run the
/// [`WARMUP`] steps in [`WARMUP_PROBES`] chunks with a micro-probe
/// after each.
///
/// Returns the origin the whole run's timestamps are measured
/// from, the warmup probe series, and the prober to keep sampling
/// with at batch seams.
///
/// - The probes interleave with warmup rather than preceding it,
///   so this stretch spans the window where a frequency governor
///   ramps, and the *last* probe — the sizing input — is taken
///   after the full warmup.
/// - This is the only workload-independent stretch of the series:
///   nothing but the warmup steps has run yet. Once the bench is
///   running, seam probes share the box with it (on a 2t bench,
///   with its worker thread), which is a truer picture of the
///   environment the run actually had and a slightly less pure
///   measure of the machine alone.
/// - Warmup step count is unchanged; these probes add ~4 ms. The
///   "Dynamic startup warmup" Todo replaces the fixed count with
///   warm-until-stable and will consume this series as its
///   convergence input.
fn warmup_and_probe<B: Bench>(bench: &mut B) -> (std::time::Instant, Vec<ProbeSummary>, Prober) {
    let origin = std::time::Instant::now();
    let mut prober = Prober::new();
    let chunk = (WARMUP / WARMUP_PROBES as u64).max(1);
    let mut probes = Vec::with_capacity(WARMUP_PROBES);
    for _ in 0..WARMUP_PROBES {
        for _ in 0..chunk {
            black_box(bench.step());
        }
        probes.push(prober.probe(origin));
    }
    (origin, probes, prober)
}

/// Size `inner` so per-sample apparatus cost is dominated by
/// workload: `inner ≈ RATIO * frame / step`.
///
/// - `frame_ns` comes from [`micro_probe_frame_ns`] — an
///   order-of-magnitude sizing input, not a calibrated
///   constant; the ratio and [`MAX_INNER`] clamp absorb its
///   imprecision.
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
    // OK: bounds the batch for the 0.23.0-4 settle selftest's
    // per-batch table; the gauge locates events by `t_start_s`.
    pub t_end_s: f64,
    /// Samples in the batch.
    pub count: u64,
    /// Minimum per-call value (ps) — the batch's fastest sample.
    #[allow(dead_code)]
    // OK: the batch's extreme record, for the 0.23.0-4 settle
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
    // the 0.23.0-4 settle selftest; no gauge signal reads it.
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
    /// summaries, and the environment probe series.
    fn finish(mut self) -> (Histogram<u64>, Vec<BatchSummary>, Vec<ProbeSummary>) {
        self.flush();
        (self.hist, self.summaries, self.probes)
    }
}

/// Time one sample (`inner` back-to-back calls), divide down to a
/// per-call value in **picoseconds**, and record it, clamping at
/// the histogram bounds — a suspend-inflated or wedged sample
/// must not panic a long run ([`warn_invalid`] flags it instead).
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

/// Print `WARNING` lines when the finished run's tail-sensitive
/// stats — `max` and the untrimmed mean/stdev — are poisoned:
///
/// - the system suspended during the run (clock divergence — a
///   mid-sample suspend inflates that sample by the sleep gap,
///   even under the histogram bound);
/// - one or more samples clamped at [`HIST_HIGH_PS`] (a wedged or
///   suspend-inflated sample with no detected suspend).
///
/// A few inflated samples out of millions land in the extreme
/// tail band: percentile boundaries and the trimmed non-tail
/// stats are unaffected, so the flag names what died rather than
/// condemning the whole report. Called at the end of
/// [`print_report`] so the flag is the last thing in the bench's
/// report, where it can't scroll out of mind. Prints one
/// `WARNING {name}:` header with each finding indented below it,
/// keeping the findings visible next to the long bench name.
///
/// Warnings are for stats that are *invalid* — poisoned by a
/// suspend or a clamp. The run gauge never routes here: its
/// signals describe the run truthfully, and much of what they
/// describe belongs to the workload rather than the machine.
fn warn_invalid(name: &str, hist: &Histogram<u64>, suspended_s: f64) {
    let mut findings: Vec<String> = Vec::new();
    if suspended_s >= SUSPEND_WARN_S {
        findings.push(format!(
            "system suspended ~{suspended_s:.1}s during the run; max/mean/stdev poisoned"
        ));
    }
    if !hist.is_empty() && hist.max() >= HIST_HIGH_PS {
        findings.push(format!(
            "sample(s) clamped at the {}s histogram bound; max/mean/stdev poisoned",
            HIST_HIGH_PS / 1_000_000_000_000
        ));
    }
    if !findings.is_empty() {
        println!("WARNING {name}:");
        for finding in &findings {
            println!("  {finding}");
        }
    }
}

/// Format an integer with thousands separators, e.g.
/// `12345` → `"12,345"`.
pub fn fmt_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format a float with `decimals` fractional digits and thousands
/// separators on the integer part.
pub fn fmt_commas_f64(n: f64, decimals: usize) -> String {
    let s = format!("{n:.decimals$}");
    let (sign, body) = match s.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", s.as_str()),
    };
    let (int_part, frac_part) = match body.find('.') {
        Some(i) => (&body[..i], &body[i..]),
        None => (body, ""),
    };
    let int_num: u64 = int_part.parse().unwrap_or(0);
    format!("{sign}{}{frac_part}", fmt_commas(int_num))
}

/// Index of the band containing `mid_rank`, over the full boundary
/// ladder `bounds` (`n_bands = bounds.len() - 1`).
///
/// - Bands are **right-closed** `(lower, upper]`: a rank exactly on a
///   boundary falls in the band that boundary *caps* — a single
///   sample's mid-rank of 0.5 lands in `p50`, not `p60`. This matches
///   the upper-boundary row labels and the CDF reading of a
///   percentile (value at or below which that fraction of samples
///   falls). See [`print_report`] for the convention and references.
/// - `mid_rank` is the Hazen plotting position `(i - 0.5) / n` of the
///   sample's rank, computed by the caller.
fn band_index(mid_rank: f64, bounds: &[bands::Boundary]) -> usize {
    let n_bands = bounds.len() - 1;
    bounds[1..]
        .iter()
        .position(|b| mid_rank <= b.pct)
        .unwrap_or(n_bands - 1)
}

/// Build the trimmed-stat range label from the populated bands
/// below the n2 ≡ p99 tail cut.
///
/// - Names the first..last populated band in `band_count[..trim_bands]`
///   by its **upper** boundary (`bounds[i + 1]`), matching the row
///   labels — so the label tracks the real extent of the trimmed
///   data rather than asserting a `min` row (never printed — rows use
///   upper boundaries) or an `n2` band that can be empty.
/// - Collapses to a single name when one band holds all the trimmed
///   data (`p60`, not `p60..p60`).
/// - Empty string when no trimmed band is populated — only with no
///   samples at all, where the caller's `trim` is `None` and the
///   label goes unused.
fn trim_range_label(
    bounds: &[bands::Boundary],
    band_count: &[u64],
    trim_bands: usize,
    style: BandLabels,
) -> String {
    let first = (0..trim_bands).find(|&i| band_count[i] > 0);
    let last = (0..trim_bands).rev().find(|&i| band_count[i] > 0);
    match (first, last) {
        (Some(f), Some(l)) if f == l => bounds[f + 1].trim_name(style).to_string(),
        (Some(f), Some(l)) => format!(
            "{}..{}",
            bounds[f + 1].trim_name(style),
            bounds[l + 1].trim_name(style),
        ),
        _ => String::new(),
    }
}

/// Print the full bench report: header line (logfmt-style metadata),
/// per-band histogram, whole-histogram mean/stdev, and trimmed
/// mean/stdev (every band below the n2 ≡ p99 tail cut). The trimmed
/// rows are labeled by the span of populated non-tail bands (e.g.
/// `mean z4..n2`), so `min` — never a row — is not asserted and an
/// empty n2 band is not named; see the label derivation below.
///
/// Each histogram row is one band, labeled by its **upper**
/// boundary — deciles in the body (`p10` … `p90`), nines/zeros in
/// the tails (`zK`/`nK` = fraction 10^-K of samples below/above
/// the boundary) — the lower boundary being the previous printed
/// row (empty bands are skipped). Bands are **right-closed**
/// `(lower, upper]` (see [`band_index`]): a sample whose rank lands
/// exactly on a boundary counts in the band that boundary caps, so a
/// lone median sample reads `p50`, not `p60` — matching the
/// upper-boundary label and the CDF definition of a percentile. This
/// is `pandas.cut`'s `right=True` convention; the rank is the Hazen
/// plotting position `(i - 0.5) / n`. Label style comes from
/// `cfg.band_labels` and is recorded as `labels=` in the header
/// metadata so saved outputs are self-describing. The `adjusted`
/// columns subtract per-call apparatus overhead
/// (`cfg.overhead.adjust_per_call_ns(inner)` — the amortized loop
/// cost plus the dithered in-interval framing slice amortized by
/// `inner`); the untrimmed `stdev` is the
/// hdrhistogram-native stdev, which includes the ms-scale outliers
/// in the tail band. Ends with `WARNING` lines flagging poisoned
/// stats when they apply — `suspended_s` comes from
/// [`run_adaptive`] (see [`warn_invalid`]).
pub fn print_report(name: &str, out: &RunOutput, cfg: &RunCfg) {
    let hist = &out.hist;
    let outer = out.outer;
    let inner = out.inner;
    let duration_s = out.duration_s;
    let suspended_s = out.suspended_s;
    let block_stats = out.block_stats.as_ref();
    // Header line: bench name + logfmt-style metadata. `adj` is the
    // apparatus overhead subtracted from each sample downstream.
    let adj = cfg.overhead.adjust_per_call_ns(inner);
    let total = outer * inner;
    let blocks_meta = match block_stats {
        Some(b) => format!(" blocks={}", b.blocks),
        None => String::new(),
    };
    let batches_meta = format!(" batches={}", out.batches.len());
    println!(
        "{name} [duration={:.1}s outer={} inner={} calls={} adj/call={}ns{blocks_meta}{batches_meta} labels={}]:",
        duration_s,
        fmt_commas(outer),
        inner,
        fmt_commas(total),
        fmt_commas_f64(adj, 2),
        cfg.band_labels.as_str(),
    );

    let bounds = bands::boundaries();

    // Trim anchor: bands at or above the n2 (p99) boundary are
    // the "tail" — excluded from the trimmed stats no matter how
    // many finer tail bands subdivide them.
    #[allow(clippy::unwrap_used)]
    // OK: boundaries() always emits n2 (N_DEPTH >= 2)
    let trim_bands = bounds.iter().position(|b| b.zpn == "n2").unwrap();

    let n_bands = bounds.len() - 1;
    let sample_count = hist.len();

    // Accumulate per-band stats by walking recorded histogram buckets.
    // Each bucket is assigned to the band containing its midpoint rank.
    let mut band_first = vec![u64::MAX; n_bands];
    let mut band_last = vec![0u64; n_bands];
    let mut band_count = vec![0u64; n_bands];
    let mut band_sum = vec![0u128; n_bands];

    let mut cumulative = 0u64;
    for iv in hist.iter_recorded() {
        let value = iv.value_iterated_to();
        let count = iv.count_at_value();
        let mid_rank = (cumulative as f64 + count as f64 / 2.0) / sample_count as f64;
        let idx = band_index(mid_rank, &bounds);
        band_first[idx] = band_first[idx].min(value);
        band_last[idx] = band_last[idx].max(value);
        band_count[idx] += count;
        band_sum[idx] += value as u128 * count as u128;
        cumulative += count;
    }

    // Trimmed-stat range label, derived from the populated bands.
    let trim_range = trim_range_label(&bounds, &band_count, trim_bands, cfg.band_labels);
    let mean_trim_label = format!("mean {trim_range}");
    let stdev_trim_label = format!("stdev {trim_range}");

    // Build rendered rows: (label, first, last, range, count, mean, adj_mean).
    struct BandRow {
        label: String,
        first: String,
        last: String,
        range: String,
        count: String,
        mean: String,
        adj_mean: String,
    }

    let mut rows: Vec<BandRow> = Vec::new();
    for i in 0..n_bands {
        if band_count[i] == 0 {
            continue;
        }
        let mean_ns = band_sum[i] as f64 / band_count[i] as f64 / PS_PER_NS;
        let adj_mean = (mean_ns - adj).max(0.0);
        rows.push(BandRow {
            label: bounds[i + 1].label(cfg.band_labels),
            first: fmt_commas_f64(band_first[i] as f64 / PS_PER_NS, cfg.decimals),
            last: fmt_commas_f64(band_last[i] as f64 / PS_PER_NS, cfg.decimals),
            range: fmt_commas_f64(
                (band_last[i] - band_first[i] + 1) as f64 / PS_PER_NS,
                cfg.decimals,
            ),
            count: fmt_commas(band_count[i]),
            mean: fmt_commas_f64(mean_ns, cfg.decimals),
            adj_mean: fmt_commas_f64(adj_mean, cfg.decimals),
        });
    }

    // Whole-histogram and trimmed (every band below the n2 ≡ p99
    // tail cut) summary values, rendered before the width
    // pass so the widths account for them — the untrimmed stdev
    // is often wider than any band mean and would otherwise
    // overflow its column, shifting its line right.
    let hist_mean = hist.mean() / PS_PER_NS;
    let hist_adj = (hist_mean - adj).max(0.0);
    let hist_mean_s = fmt_commas_f64(hist_mean, cfg.decimals);
    let hist_adj_s = fmt_commas_f64(hist_adj, cfg.decimals);
    let hist_stdev_s = fmt_commas_f64(hist.stdev() / PS_PER_NS, cfg.decimals);

    let trim_count: u64 = band_count[..trim_bands].iter().sum();
    let trim = if trim_count > 0 {
        let trim_sum: u128 = band_sum[..trim_bands].iter().sum();
        let trim_mean = trim_sum as f64 / trim_count as f64 / PS_PER_NS;
        let trim_adj = (trim_mean - adj).max(0.0);

        // Variance: walk histogram buckets, include only non-tail bands.
        let mut trim_var_sum = 0.0f64;
        let mut trim_var_count = 0u64;
        let mut cum = 0u64;
        for iv in hist.iter_recorded() {
            let value = iv.value_iterated_to();
            let count = iv.count_at_value();
            let mid_rank = (cum as f64 + count as f64 / 2.0) / sample_count as f64;
            let idx = band_index(mid_rank, &bounds);
            if idx < trim_bands {
                let diff = value as f64 / PS_PER_NS - trim_mean;
                trim_var_sum += diff * diff * count as f64;
                trim_var_count += count;
            }
            cum += count;
        }
        let trim_stdev = if trim_var_count > 1 {
            (trim_var_sum / trim_var_count as f64).sqrt()
        } else {
            0.0
        };

        Some((
            fmt_commas_f64(trim_mean, cfg.decimals),
            fmt_commas_f64(trim_adj, cfg.decimals),
            fmt_commas_f64(trim_stdev, cfg.decimals),
        ))
    } else {
        None
    };

    // Block-replication summary strings, rendered before the
    // width pass like the other summary lines.
    let block_strs = block_stats.map(|b| {
        (
            fmt_commas_f64(b.mean_ns, cfg.decimals),
            fmt_commas_f64(b.ci95_ns, cfg.decimals),
            fmt_commas_f64(b.lsc_ns, cfg.decimals),
        )
    });

    // Column widths from rendered strings — band rows and the
    // summary lines that print in the mean/adjusted columns.
    let label_w = rows
        .iter()
        .map(|r| r.label.len())
        .max()
        .unwrap_or(0)
        .max(stdev_trim_label.len());
    let first_w = rows.iter().map(|r| r.first.len()).max().unwrap_or(0);
    let last_w = rows.iter().map(|r| r.last.len()).max().unwrap_or(0);
    let range_w = rows.iter().map(|r| r.range.len()).max().unwrap_or(0);
    let count_w = rows.iter().map(|r| r.count.len()).max().unwrap_or(0);
    let mean_w = rows
        .iter()
        .map(|r| r.mean.len())
        .chain([hist_mean_s.len(), hist_stdev_s.len()])
        .chain(trim.iter().flat_map(|(m, _, s)| [m.len(), s.len()]))
        .chain(
            block_strs
                .iter()
                .flat_map(|(m, c, l)| [m.len(), c.len(), l.len()]),
        )
        .max()
        .unwrap_or(0);
    // The `adjusted` header (8 chars) spans GAP + adj_w + ` ns`
    // = 7 + adj_w columns; the floor of 5 keeps 4 spaces between
    // the mean and adjusted headers when the adjusted values are
    // narrow.
    let adj_w = rows
        .iter()
        .map(|r| r.adj_mean.len())
        .chain([hist_adj_s.len()])
        .chain(trim.iter().map(|(_, a, _)| a.len()))
        .max()
        .unwrap_or(0)
        .max(5);

    const INDENT: &str = "  ";
    const GAP: &str = "    ";

    // Header row. Each label right-justifies to the last
    // character of its column's ` ns` unit; `count` is unitless
    // and right-justifies to its digits.
    const UNIT: usize = " ns".len();
    let first_col = INDENT.len() + label_w + 1 + first_w + UNIT;
    let last_gap = GAP.len() + last_w + UNIT;
    let range_gap = GAP.len() + range_w + UNIT;
    let count_gap = GAP.len() + count_w;
    let mean_gap = GAP.len() + mean_w + UNIT;
    let adj_gap = GAP.len() + adj_w + UNIT;
    println!(
        "{:>first_col$}{:>last_gap$}{:>range_gap$}{:>count_gap$}{:>mean_gap$}{:>adj_gap$}",
        "first", "last", "range", "count", "mean", "adjusted",
    );

    for r in &rows {
        println!(
            "{INDENT}{:<label_w$} {:>first_w$} ns{GAP}{:>last_w$} ns{GAP}{:>range_w$} ns{GAP}{:>count_w$}{GAP}{:>mean_w$} ns{GAP}{:>adj_w$} ns",
            r.label, r.first, r.last, r.range, r.count, r.mean, r.adj_mean,
        );
    }

    // Whole-histogram summary. Aligned to mean/adjusted columns.
    let skip = first_w
        + " ns".len()
        + GAP.len()
        + last_w
        + " ns".len()
        + GAP.len()
        + range_w
        + " ns".len()
        + GAP.len()
        + count_w;
    println!(
        "{INDENT}{:<label_w$} {:>skip$}{GAP}{hist_mean_s:>mean_w$} ns{GAP}{hist_adj_s:>adj_w$} ns",
        "mean", "",
    );
    println!(
        "{INDENT}{:<label_w$} {:>skip$}{GAP}{hist_stdev_s:>mean_w$} ns",
        "stdev", "",
    );

    if let Some((trim_mean_s, trim_adj_s, trim_stdev_s)) = &trim {
        println!(
            "{INDENT}{:<label_w$} {:>skip$}{GAP}{trim_mean_s:>mean_w$} ns{GAP}{trim_adj_s:>adj_w$} ns",
            mean_trim_label, "",
        );
        println!(
            "{INDENT}{:<label_w$} {:>skip$}{GAP}{trim_stdev_s:>mean_w$} ns",
            stdev_trim_label, "",
        );
    }
    if let Some((block_mean_s, block_ci_s, block_lsc_s)) = &block_strs {
        println!(
            "{INDENT}{:<label_w$} {:>skip$}{GAP}{block_mean_s:>mean_w$} ns",
            "mean blocks", "",
        );
        println!(
            "{INDENT}{:<label_w$} {:>skip$}{GAP}{block_ci_s:>mean_w$} ns",
            "CI95", "",
        );
        println!(
            "{INDENT}{:<label_w$} {:>skip$}{GAP}{block_lsc_s:>mean_w$} ns",
            "LSC", "",
        );
    }
    // Environment gauge: the letter grading the *box*, from the
    // warmup probe series — printed above the run gauge because
    // it is measured first and reads as the run's preconditions.
    if let Some(g) = crate::gauge::EnvGrade::from_probes(&out.probes) {
        let [sl_spread, sl_int, sl_drift, sl_step] = g.signal_letters();
        let step_at = if g.step_frac > 0.0 {
            format!(" @{:.3}s", g.step_at_s)
        } else {
            String::new()
        };
        let gw = GAUGE_LABEL_W;
        println!(
            "{INDENT}{:<gw$}spread {:.2}% {sl_spread}, interference {:.2}% {sl_int}, \
             drift {:.2}% {sl_drift}, step {:.2}%{step_at} {sl_step}",
            "env:",
            g.spread_frac * 100.0,
            g.interference_frac * 100.0,
            g.drift_frac * 100.0,
            g.step_frac * 100.0,
        );
        println!("{INDENT}{:<gw$}{}", "env worst case:", g.letter);
    }
    // Run gauge: the letter grading *these* numbers, from the
    // run's own batches, with every signal's own letter beside
    // its value. Reported, never warned on — see [`crate::gauge`].
    if let Some(g) = crate::gauge::RunGrade::from_batches(&out.batches) {
        let [sl_int, sl_burst, sl_drift, sl_step] = g.signal_letters();
        let step_at = if g.step_frac > 0.0 {
            format!(" @{:.1}s", g.step_at_s)
        } else {
            String::new()
        };
        // The composite prints on its own labelled line under the
        // signals it came from, so "worst signal wins" is visible
        // rather than needing the docs.
        let gw = GAUGE_LABEL_W;
        println!(
            "{INDENT}{:<gw$}interference {:.2}% {sl_int}, bursts {:.0}% {sl_burst}, \
             drift {:.2}% {sl_drift}, step {:.2}%{step_at} {sl_step}",
            "run:",
            g.interference_frac * 100.0,
            g.burst_frac * 100.0,
            g.drift_frac * 100.0,
            g.step_frac * 100.0,
        );
        println!("{INDENT}{:<gw$}{}", "run worst case:", g.letter);
    }
    warn_invalid(name, hist, suspended_s);
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let (hist, batches, probes) = p.finish();
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
        let (hist, batches, _) = p.finish();
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
        let (hist, batches, probes) = p.finish();
        assert_eq!(hist.len(), 1);
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].count, 1);
        // The empty flush moved the clock without probing: a
        // probe belongs to a batch, and there was no batch.
        assert_eq!(probes.len(), 1);
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

    /// A `band_count` vec (len = n_bands) with the given band
    /// indices marked populated.
    fn counts(n_bands: usize, populated: &[usize]) -> Vec<u64> {
        let mut c = vec![0u64; n_bands];
        for &i in populated {
            c[i] = 1;
        }
        c
    }

    #[test]
    fn trim_range_label_spans_populated_bands() {
        let bounds = bands::boundaries();
        let n_bands = bounds.len() - 1;
        // OK: boundaries() always emits n2 (N_DEPTH >= 2)
        let trim_bands = bounds.iter().position(|b| b.zpn == "n2").unwrap();

        // Full range: first band (label z4) through the n2 band.
        let c = counts(n_bands, &[0, 5, trim_bands - 1]);
        assert_eq!(
            trim_range_label(&bounds, &c, trim_bands, BandLabels::Zpn),
            "z4..n2"
        );
        assert_eq!(
            trim_range_label(&bounds, &c, trim_bands, BandLabels::Both),
            "z4..n2"
        );
        assert_eq!(
            trim_range_label(&bounds, &c, trim_bands, BandLabels::Frac),
            "0.000_1..0.99"
        );

        // n2 band empty: upper end is the last populated band (p90).
        let c = counts(n_bands, &[0, 11]);
        assert_eq!(
            trim_range_label(&bounds, &c, trim_bands, BandLabels::Zpn),
            "z4..p90"
        );

        // One populated band collapses to a single name.
        let c = counts(n_bands, &[8]);
        assert_eq!(
            trim_range_label(&bounds, &c, trim_bands, BandLabels::Zpn),
            "p60"
        );

        // No populated trimmed band yields an empty label (unused).
        let c = counts(n_bands, &[]);
        assert_eq!(
            trim_range_label(&bounds, &c, trim_bands, BandLabels::Zpn),
            ""
        );
    }

    #[test]
    fn band_index_right_closed_on_boundary() {
        let bounds = bands::boundaries();
        // Label of the band `mid_rank` falls in (bands labeled by
        // upper boundary → bounds[idx + 1]).
        let label = |r: f64| bounds[band_index(r, &bounds) + 1].zpn.as_str();

        // Right-closed: a rank exactly on a boundary lands in the band
        // that boundary caps, not the next one up.
        assert_eq!(label(0.5), "p50"); // single-sample mid-rank
        assert_eq!(label(0.4), "p40"); // exactly the p40 boundary
        assert_eq!(label(0.99), "n2"); // exactly p99 → last non-tail band
        assert_eq!(label(0.01), "z2"); // exactly the z2 boundary

        // Strictly-interior ranks are unaffected by the closed end.
        assert_eq!(label(0.45), "p50");
        assert_eq!(label(0.55), "p60");
        assert_eq!(label(0.05), "p10");
    }
}
