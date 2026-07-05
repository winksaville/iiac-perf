//! Generic bench driver: the [`Bench`] trait, adaptive outer/inner
//! loop sizing, and the band-histogram report.

use std::hint::black_box;

use hdrhistogram::Histogram;

use crate::overhead::Overhead;

const WARMUP: u64 = 10_000;
const ESTIMATE_STEPS: u64 = 1_000;
const ESTIMATE_SAMPLES: usize = 5;
const FRAMING_DOMINATION_RATIO: f64 = 10.0;
const MAX_INNER: u64 = 1_000;

/// Histogram value bounds: 1 ns to 60 s at 3 sig figs. The high
/// bound is a sane-world ceiling for one recorded sample, not a
/// technical limit — [`record_sample`] clamps above it and
/// [`warn_invalid`] flags the run.
const HIST_LOW_NS: u64 = 1;
const HIST_HIGH_NS: u64 = 60_000_000_000;

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

/// Drive `bench` against `cfg` and return
/// `(histogram, outer, inner, duration_s, suspended_s)`.
///
/// After a fixed warmup, `inner` is auto-sized so apparatus framing
/// doesn't dominate (skipped when `cfg.inner_override` is set). The
/// outer loop runs either for `cfg.outer_override` iterations or
/// until `cfg.target_seconds` elapses. `suspended_s` is the time
/// the system spent suspended during the measured run (see
/// [`ClockPair`]); pass it to [`print_report`], which flags the
/// poisoned stats when it is non-trivial.
pub fn run_adaptive<B: Bench>(bench: &mut B, cfg: &RunCfg) -> (Histogram<u64>, u64, u64, f64, f64) {
    for _ in 0..WARMUP {
        black_box(bench.step());
    }

    let step_cost_ns = estimate_step_cost(bench);
    let framing_ns = cfg.overhead.framing_per_sample_ns.max(1.0);
    let inner = cfg
        .inner_override
        .unwrap_or_else(|| pick_inner(step_cost_ns, framing_ns));

    let clocks = ClockPair::now();
    let (hist, outer, duration_s) = match cfg.outer_override {
        Some(outer) => {
            let (hist, duration_s) = run_counted(bench, outer, inner);
            (hist, outer, duration_s)
        }
        None => {
            let (hist, duration_s) = run_timed(bench, cfg.target_seconds, inner);
            let outer = hist.len();
            (hist, outer, duration_s)
        }
    };
    (hist, outer, inner, duration_s, clocks.suspended_s())
}

fn estimate_step_cost<B: Bench>(bench: &mut B) -> f64 {
    let mut samples: Vec<f64> = (0..ESTIMATE_SAMPLES)
        .map(|_| {
            let start = minstant::Instant::now();
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

fn pick_inner(step_cost_ns: f64, framing_ns: f64) -> u64 {
    let target = (FRAMING_DOMINATION_RATIO * framing_ns / step_cost_ns).ceil() as u64;
    target.clamp(1, MAX_INNER)
}

fn run_counted<B: Bench>(bench: &mut B, outer: u64, inner: u64) -> (Histogram<u64>, f64) {
    let mut hist = new_hist();
    let run_start = minstant::Instant::now();
    for _ in 0..outer {
        record_sample(bench, inner, &mut hist);
    }
    let duration_s = run_start.elapsed().as_nanos() as f64 / 1e9;
    (hist, duration_s)
}

fn run_timed<B: Bench>(bench: &mut B, target_seconds: f64, inner: u64) -> (Histogram<u64>, f64) {
    let mut hist = new_hist();
    let target_ns = (target_seconds * 1e9) as u128;
    let run_start = minstant::Instant::now();
    loop {
        record_sample(bench, inner, &mut hist);
        if run_start.elapsed().as_nanos() >= target_ns {
            break;
        }
    }
    let duration_s = run_start.elapsed().as_nanos() as f64 / 1e9;
    (hist, duration_s)
}

/// Fresh histogram over `[HIST_LOW_NS, HIST_HIGH_NS]` at 3 sig
/// figs, resize disabled — out-of-range samples clamp (see
/// [`record_sample`]) rather than grow the histogram.
fn new_hist() -> Histogram<u64> {
    Histogram::<u64>::new_with_bounds(HIST_LOW_NS, HIST_HIGH_NS, 3).unwrap() // OK: constant bounds
}

/// Time one sample (`inner` back-to-back calls), divide down to a
/// per-call value, and record it, clamping at the histogram
/// bounds — a suspend-inflated or wedged sample must not panic a
/// long run ([`warn_invalid`] flags it instead).
fn record_sample<B: Bench>(bench: &mut B, inner: u64, hist: &mut Histogram<u64>) {
    let start = minstant::Instant::now();
    for _ in 0..inner {
        black_box(bench.step());
    }
    let elapsed_ns = start.elapsed().as_nanos() as u64;
    hist.saturating_record(round_elapsed(elapsed_ns, inner));
}

fn round_elapsed(elapsed_ns: u64, inner: u64) -> u64 {
    (elapsed_ns + inner / 2) / inner
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
/// - one or more samples clamped at [`HIST_HIGH_NS`] (a wedged or
///   suspend-inflated sample with no detected suspend).
///
/// A few inflated samples out of millions land in the extreme
/// tail band: percentile boundaries and the trimmed `min-p99`
/// stats are unaffected, so the flag names what died rather than
/// condemning the whole report. Called at the end of
/// [`print_report`] so the flag is the last thing in the bench's
/// report, where it can't scroll out of mind. Prints one
/// `WARNING {name}:` header with each finding indented below it,
/// keeping the findings visible next to the long bench name.
fn warn_invalid(name: &str, hist: &Histogram<u64>, suspended_s: f64) {
    let mut findings: Vec<String> = Vec::new();
    if suspended_s >= SUSPEND_WARN_S {
        findings.push(format!(
            "system suspended ~{suspended_s:.1}s during the run; max/mean/stdev poisoned"
        ));
    }
    if !hist.is_empty() && hist.max() >= HIST_HIGH_NS {
        findings.push(format!(
            "sample(s) clamped at the {}s histogram bound; max/mean/stdev poisoned",
            HIST_HIGH_NS / 1_000_000_000
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

/// Print the full bench report: header line (logfmt-style metadata),
/// per-band histogram (min→p1, p1→p10, … p99→max), whole-histogram
/// mean/stdev, and trimmed mean/stdev (min–p99, excluding the p99→max
/// tail). The `adjusted` columns subtract per-call apparatus overhead
/// (`overhead.per_call_ns(inner)`); the untrimmed `stdev` is the
/// hdrhistogram-native stdev, which includes the ms-scale outliers
/// in the tail band. Ends with `WARNING` lines flagging poisoned
/// stats when they apply — `suspended_s` comes from
/// [`run_adaptive`] (see [`warn_invalid`]).
pub fn print_report(
    name: &str,
    outer: u64,
    inner: u64,
    duration_s: f64,
    hist: &Histogram<u64>,
    overhead: &Overhead,
    suspended_s: f64,
) {
    // Header line: bench name + logfmt-style metadata. `adj` is the
    // apparatus overhead subtracted from each sample downstream.
    let adj = overhead.per_call_ns(inner);
    let total = outer * inner;
    println!(
        "{name} [duration={:.1}s outer={} inner={} calls={} adj/call={}ns]:",
        duration_s,
        fmt_commas(outer),
        inner,
        fmt_commas(total),
        fmt_commas_f64(adj, 2),
    );

    // Band boundaries defined by percentiles. Each consecutive pair
    // forms one band; we iterate the histogram to compute per-band
    // stats (first, last, count, mean).
    let boundary_pcts: &[f64] = &[
        0.0, 0.01, 0.10, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70, 0.80, 0.90, 0.99, 1.0,
    ];
    let boundary_names: &[&str] = &[
        "min", "p1", "p10", "p20", "p30", "p40", "p50", "p60", "p70", "p80", "p90", "p99", "max",
    ];

    let n_bands = boundary_pcts.len() - 1;
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
        let idx = boundary_pcts[1..]
            .iter()
            .position(|&b| mid_rank < b)
            .unwrap_or(n_bands - 1);
        band_first[idx] = band_first[idx].min(value);
        band_last[idx] = band_last[idx].max(value);
        band_count[idx] += count;
        band_sum[idx] += value as u128 * count as u128;
        cumulative += count;
    }

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
        let mean_val = band_sum[i] as f64 / band_count[i] as f64;
        let adj_mean = (mean_val - adj).max(0.0);
        rows.push(BandRow {
            label: format!("{}-{}", boundary_names[i], boundary_names[i + 1]),
            first: fmt_commas(band_first[i]),
            last: fmt_commas(band_last[i]),
            range: fmt_commas(band_last[i] - band_first[i] + 1),
            count: fmt_commas(band_count[i]),
            mean: fmt_commas_f64(mean_val, 0),
            adj_mean: fmt_commas_f64(adj_mean, 0),
        });
    }

    // Column widths from rendered strings.
    let label_w = rows
        .iter()
        .map(|r| r.label.len())
        .max()
        .unwrap_or(0)
        .max("stdev min-p99".len());
    let first_w = rows.iter().map(|r| r.first.len()).max().unwrap_or(0);
    let last_w = rows.iter().map(|r| r.last.len()).max().unwrap_or(0);
    let range_w = rows.iter().map(|r| r.range.len()).max().unwrap_or(0);
    let count_w = rows.iter().map(|r| r.count.len()).max().unwrap_or(0);
    let mean_w = rows.iter().map(|r| r.mean.len()).max().unwrap_or(0);
    let adj_w = rows.iter().map(|r| r.adj_mean.len()).max().unwrap_or(0);

    const INDENT: &str = "  ";
    const GAP: &str = "    ";

    // Header row.
    let first_col = INDENT.len() + label_w + 1 + first_w;
    let last_gap = " ns".len() + GAP.len() + last_w;
    let range_gap = " ns".len() + GAP.len() + range_w;
    let count_gap = " ns".len() + GAP.len() + count_w;
    let mean_gap = GAP.len() + mean_w;
    let adj_gap = " ns".len() + GAP.len() + adj_w;
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
    let hist_mean = hist.mean();
    let hist_adj = (hist_mean - adj).max(0.0);
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
        "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} ns{GAP}{:>adj_w$} ns",
        "mean",
        "",
        fmt_commas_f64(hist_mean, 0),
        fmt_commas_f64(hist_adj, 0),
    );
    println!(
        "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} ns",
        "stdev",
        "",
        fmt_commas_f64(hist.stdev(), 0),
    );

    // Trimmed mean/stdev (min–p99, excluding p99-max band).
    let trim_count: u64 = band_count[..n_bands - 1].iter().sum();
    if trim_count > 0 {
        let trim_sum: u128 = band_sum[..n_bands - 1].iter().sum();
        let trim_mean = trim_sum as f64 / trim_count as f64;
        let trim_adj = (trim_mean - adj).max(0.0);

        // Variance: walk histogram buckets, include only non-tail bands.
        let mut trim_var_sum = 0.0f64;
        let mut trim_var_count = 0u64;
        let mut cum = 0u64;
        for iv in hist.iter_recorded() {
            let value = iv.value_iterated_to();
            let count = iv.count_at_value();
            let mid_rank = (cum as f64 + count as f64 / 2.0) / sample_count as f64;
            let idx = boundary_pcts[1..]
                .iter()
                .position(|&b| mid_rank < b)
                .unwrap_or(n_bands - 1);
            if idx < n_bands - 1 {
                let diff = value as f64 - trim_mean;
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

        println!(
            "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} ns{GAP}{:>adj_w$} ns",
            "mean min-p99",
            "",
            fmt_commas_f64(trim_mean, 0),
            fmt_commas_f64(trim_adj, 0),
        );
        println!(
            "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} ns",
            "stdev min-p99",
            "",
            fmt_commas_f64(trim_stdev, 0),
        );
    }
    warn_invalid(name, hist, suspended_s);
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_pair_no_suspend_gap() {
        let clocks = ClockPair::now();
        let gap = clocks.suspended_s();
        assert!(gap.abs() < 0.5, "unexpected clock divergence: {gap}");
    }

    #[test]
    fn saturating_record_clamps_above_bound() {
        let mut hist = new_hist();
        hist.saturating_record(HIST_HIGH_NS * 2);
        assert_eq!(hist.len(), 1);
        assert!(hist.max() >= HIST_HIGH_NS);
    }
}
