//! Generic bench driver: the [`Bench`] trait, adaptive outer/inner
//! loop sizing, and the band-histogram report.

use std::hint::black_box;

use hdrhistogram::Histogram;

use crate::bands::{self, BandLabels};
use crate::overhead::Overhead;

const WARMUP: u64 = 10_000;
const ESTIMATE_STEPS: u64 = 1_000;
const ESTIMATE_SAMPLES: usize = 5;
const FRAMING_DOMINATION_RATIO: f64 = 10.0;
const MAX_INNER: u64 = 1_000;

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
    /// Band-label style for [`print_report`] histogram rows.
    /// Plumbed from the `--band-labels` CLI flag.
    pub band_labels: BandLabels,
    /// Decimal digits on [`print_report`] time columns. Plumbed
    /// from the `--decimals` CLI flag (default 1; 0 restores
    /// integers; 3 is the ps recording floor).
    pub decimals: usize,
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

/// Fresh histogram over `[HIST_LOW_PS, HIST_HIGH_PS]` at 3 sig
/// figs, resize disabled — out-of-range samples clamp (see
/// [`record_sample`]) rather than grow the histogram.
fn new_hist() -> Histogram<u64> {
    Histogram::<u64>::new_with_bounds(HIST_LOW_PS, HIST_HIGH_PS, 3).unwrap() // OK: constant bounds
}

/// Time one sample (`inner` back-to-back calls), divide down to a
/// per-call value in **picoseconds**, and record it, clamping at
/// the histogram bounds — a suspend-inflated or wedged sample
/// must not panic a long run ([`warn_invalid`] flags it instead).
fn record_sample<B: Bench>(bench: &mut B, inner: u64, hist: &mut Histogram<u64>) {
    let start = minstant::Instant::now();
    for _ in 0..inner {
        black_box(bench.step());
    }
    let elapsed_ps = start.elapsed().as_nanos().saturating_mul(1000);
    hist.saturating_record(round_elapsed_ps(elapsed_ps, inner));
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
/// row (empty bands are skipped). Label style comes from
/// `cfg.band_labels` and is recorded as `labels=` in the header
/// metadata so saved outputs are self-describing. The `adjusted`
/// columns subtract per-call apparatus overhead
/// (`cfg.overhead.per_call_ns(inner)`); the untrimmed `stdev` is the
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
    cfg: &RunCfg,
    suspended_s: f64,
) {
    // Header line: bench name + logfmt-style metadata. `adj` is the
    // apparatus overhead subtracted from each sample downstream.
    let adj = cfg.overhead.per_call_ns(inner);
    let total = outer * inner;
    println!(
        "{name} [duration={:.1}s outer={} inner={} calls={} adj/call={}ns labels={}]:",
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
        let idx = bounds[1..]
            .iter()
            .position(|b| mid_rank < b.pct)
            .unwrap_or(n_bands - 1);
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
            let idx = bounds[1..]
                .iter()
                .position(|b| mid_rank < b.pct)
                .unwrap_or(n_bands - 1);
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
}
