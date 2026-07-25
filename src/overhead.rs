//! Apparatus-overhead calibration, dithered and amortized.
//!
//! Three constants, from three measurement passes:
//!
//! - **Dithered two-point fit** (mean of the fastest window
//!   means at `N_LOW` / `N_HIGH`, random sub-quantum phase
//!   dither per sample): slope → `loop_per_iter_ns`, intercept →
//!   `frame_sample_ns`, the in-interval timer-pair slice a
//!   recorded sample actually contains. Both are subtracted from
//!   reported values.
//! - **Window pass** and **loop-only pass** (min over amortized
//!   window means at `N_LOW`, identical but for the per-sample
//!   timer pair): their difference is `frame_call_ns`, the full
//!   call-to-call cost of taking a sample — sizes the experiment,
//!   never subtracted (most of it sits outside recorded
//!   intervals).
//!
//! All three passes drive the same [`run_inner`], so the loop
//! term cancels exactly in the `frame_call_ns` subtraction
//! instead of being estimated from the dithered slope at a
//! different call site.
//!
//! Dither makes the ~10 ns clock quantum a zero-mean error that
//! averages away in means (validated on r5-7600x: frame_sample
//! repeats to ±0.06 ns within a frequency regime). See
//! notes/design.md#dithering-random-phase-injection and
//! notes/design.md#timer-overhead-in-interval-vs-call-to-call.

use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::harness::Bench;

/// Calibration warmup iterations — long enough for CPU frequency
/// boost to ramp before the first measurement.
pub const CAL_WARMUP: u64 = 100_000;

/// Inner-loop count for the low-N calibration point.
pub const N_LOW: u64 = 100;

/// Inner-loop count for the high-N calibration point. A wide spread
/// (`N_HIGH / (N_HIGH - N_LOW) ≈ 1.01`) keeps noise amplification on
/// the fitted intercept small.
pub const N_HIGH: u64 = 10_000;

/// Samples per window in the call-to-call pass at [`N_LOW`]. The
/// window's ±1-quantum error divides by this: ~0.001 ns at a
/// ~10 ns quantum.
pub const W_LOW_SAMPLES: u64 = 10_000;

/// Windows in the call-to-call pass; the minimum is kept. The min
/// sheds windows inflated by preemption or a frequency dip while
/// staying amortized (each candidate is already a window mean).
pub const W_LOW_WINDOWS: u64 = 100;

/// Dither span in neutral spin iterations (~0.4-0.5 ns each →
/// ~26-32 ns, spanning ~3 clock quanta). A random 0..span delay
/// before each sample randomizes its phase on the ~10 ns clock
/// lattice, making the quantization error zero-mean (see
/// notes/design.md#dithering-random-phase-injection).
pub const DITHER_SPAN: u64 = 64;

/// Dithered-fit windows per point (each yields one window mean;
/// the spread across windows is the dispersion signal).
///
/// - Raised from 20 with the per-window sample counts halved, so
///   the total sample budget is unchanged. More windows sharpen
///   the dispersion diagnostics (spread, median-window) at no
///   cost.
pub const DITHER_WINDOWS: u64 = 40;

/// Dithered-fit samples per window at [`N_LOW`].
pub const DITHER_LOW_SAMPLES: u64 = 2_500;

/// Dithered-fit samples per window at [`N_HIGH`] (samples are
/// ~100× longer, so fewer keep the wall cost comparable).
pub const DITHER_HIGH_SAMPLES: u64 = 250;

/// Fastest window means averaged for the `fast` diagnostic — the
/// lower 10% of [`DITHER_WINDOWS`].
///
/// - **Not** the production estimator: window-level tail
///   selection assumes interference is sporadic enough that some
///   window escapes it. Under a continuous competitor no
///   `N_HIGH` window does (measured: fastest window 142,644 ns
///   against a 76,573 ns sample floor), so this reads as
///   contaminated as the means. Kept as a logged comparison
///   because its *divergence* from the sample quantiles is a
///   contamination signal. See
///   notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit.
pub const DITHER_FAST_WINDOWS: u64 = 4;

/// Candidate discard fractions for the low-quantile estimator:
/// drop this fraction of the fastest samples, then take the
/// minimum of what remains (so `0.0` is the strict minimum).
///
/// - Scheduler interference is one-sided, so the *low* tail is
///   the uncontaminated part of the distribution — but its very
///   bottom is where samples that rounded down on the ~10 ns
///   clock lattice collect. Discarding a slice sheds those
///   without giving up the robustness.
/// - The aim is a *stable* estimate, not an unbiased one: a
///   small repeatable bias is subtracted consistently, whereas
///   contamination is neither small nor repeatable.
pub const DITHER_LOW_Q: [f64; 4] = [0.0, 0.001, 0.01, 0.05];

/// Index into [`DITHER_LOW_Q`] selecting the production fit's
/// discard fraction.
pub const DITHER_PROD_Q: usize = 2;

/// Xorshift64* PRNG for dither lengths. No external dep; phase
/// randomization needs rough uniformity, not statistical rigor.
struct XorShift64(u64);

impl XorShift64 {
    /// Next pseudo-random u64.
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// Sub-quantum phase dither: a random 0..[`DITHER_SPAN`] neutral
/// spin, run *outside* the timed interval before each sample.
/// Used by calibration and by the harness sample seam.
pub struct Dither(XorShift64);

impl Dither {
    /// New dither source, seeded from wall-clock nanos (any
    /// per-invocation variation suffices for phase dither).
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64 | 1)
            .unwrap_or(0x9E37_79B9_7F4A_7C15); // OK: fixed fallback seed still dithers
        Dither(XorShift64(seed))
    }

    /// Spin a random 0..[`DITHER_SPAN`] iterations to re-roll the
    /// next sample's phase on the clock lattice.
    #[inline]
    pub fn spin(&mut self) {
        let r = self.0.next() % DITHER_SPAN;
        for _ in 0..r {
            black_box(1u64);
        }
    }

    /// Next raw pseudo-random u64, for callers needing coarser
    /// randomness (e.g. the harness's block sleep lengths).
    pub fn rand_u64(&mut self) -> u64 {
        self.0.next()
    }
}

impl Default for Dither {
    /// Same as [`Dither::new`].
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregates of one dithered measurement point (all ns):
/// linear statistics that keep the dither win, plus min for
/// reference against the lattice floor.
#[derive(Debug)]
pub struct DitherPoint {
    /// Full mean over all samples — unbiased under dither but
    /// absorbs interrupt spikes.
    pub mean_ns: f64,
    /// Mean of samples ≤ p99 — sheds the top 1% of samples.
    /// Retained for comparison and regime fingerprinting; too
    /// little trim to survive contention, so no longer the fit
    /// input.
    pub mean_p99_ns: f64,
    /// Mean of the fastest [`DITHER_FAST_WINDOWS`] window means.
    /// A diagnostic, not the fit input — see
    /// [`DITHER_FAST_WINDOWS`].
    pub mean_fast_ns: f64,
    /// Low quantiles, one per [`DITHER_LOW_Q`] entry: the
    /// smallest sample remaining after discarding that fraction
    /// of the fastest. Index into it with the same position.
    pub low_q_ns: [f64; DITHER_LOW_Q.len()],
    /// Median of per-window means — robust to a bad window
    /// without snapping (window means are not lattice-valued).
    pub median_window_ns: f64,
    /// Spread (max − min) of the window means — a dispersion
    /// signal for a CI and for regime-shift detection.
    pub window_spread_ns: f64,
    /// Minimum sample — the lattice floor, for comparison.
    pub min_ns: u64,
}

/// Apparatus-overhead model fitted by [`calibrate`].
#[derive(Debug)]
pub struct Overhead {
    /// Call-to-call cost of taking one sample (full timer-pair
    /// apparatus cost, clock-read latencies included), in ns —
    /// the window pass minus the loop-only pass. Sizes the
    /// experiment ([`crate::harness`]'s `pick_inner`); most of it
    /// sits *outside* recorded intervals, so it is never
    /// subtracted from reported values. Clamped at 0, with a
    /// warning, if the two passes disagree.
    pub frame_call_ns: f64,
    /// In-interval timer-pair slice a recorded sample contains,
    /// in ns — the dithered fit's intercept. Subtracted per
    /// sample (amortized by `inner`); see
    /// [`Overhead::adjust_per_call_ns`]. Clamped at 0, with a
    /// warning, when the fit returns a negative intercept.
    pub frame_sample_ns: f64,
    /// Per-inner-iteration loop overhead (branch + `black_box`),
    /// in ns — the dithered fit's slope. Subtracted per call.
    /// Also the frequency-regime fingerprint (repeats to 5
    /// significant figures within a regime).
    pub loop_per_iter_ns: f64,
    /// Raw call-to-call window minimum at [`N_LOW`] (ns).
    /// Preserved for `-v` logging and cache provenance.
    pub cal_w_low_ns: f64,
    /// Raw loop-only window minimum at [`N_LOW`] (ns) — the same
    /// pass without the per-sample timer pair. `cal_w_low_ns`
    /// minus this is [`Overhead::frame_call_ns`].
    pub cal_l_low_ns: f64,
    /// Raw dithered point at [`N_LOW`].
    pub cal_d_low: DitherPoint,
    /// Raw dithered point at [`N_HIGH`].
    pub cal_d_high: DitherPoint,
    /// Wall-clock duration of the full calibration run.
    pub cal_duration: Duration,
}

impl Overhead {
    /// Per-call apparatus overhead subtracted from reported values,
    /// in ns: the amortized loop cost plus the in-interval framing
    /// slice amortized by `inner`.
    ///
    /// - `frame_sample_ns` is the dithered-fit intercept — the
    ///   slice of timer cost recorded intervals actually contain
    ///   (±~0.1 ns run-to-run within a frequency regime), not the
    ///   call-to-call cost, most of which falls outside them.
    pub fn adjust_per_call_ns(&self, inner: u64) -> f64 {
        self.frame_sample_ns / inner as f64 + self.loop_per_iter_ns
    }
}

struct EmptyBench;

impl Bench for EmptyBench {
    fn name(&self) -> &str {
        "empty"
    }

    fn step(&mut self) -> u64 {
        black_box(1)
    }
}

/// The inner loop, as one compiled copy shared by every
/// calibration pass.
///
/// - `#[inline(never)]` is load-bearing: `frame_call_ns` is a
///   difference of two passes, so any per-call-site codegen
///   difference in this loop lands in the result at full size.
///   One copy makes it cancel instead. See
///   notes/chores/chores-04.md#call-site-codegen-and-the-frame_call-subtraction.
/// - The call itself costs one call per *sample*, not per
///   iteration, so it does not enter `loop_per_iter_ns`; it is a
///   constant present in every pass, and cancels too.
#[inline(never)]
fn run_inner(bench: &mut EmptyBench, inner: u64) {
    for _ in 0..inner {
        black_box(bench.step());
    }
}

/// Run the dithered two-point calibration plus the call-to-call
/// window pass and return the fitted [`Overhead`]. Blocks for
/// ~150-200 ms on a typical modern x86; logs raw points and the
/// alternative fits at debug level.
pub fn calibrate() -> Overhead {
    let mut bench = EmptyBench;
    let mut dither = Dither::new();
    let cal_start = Instant::now();
    for _ in 0..CAL_WARMUP {
        black_box(bench.step());
    }

    let d_low = dither_measure(&mut dither, DITHER_WINDOWS, DITHER_LOW_SAMPLES, N_LOW);
    let d_high = dither_measure(&mut dither, DITHER_WINDOWS, DITHER_HIGH_SAMPLES, N_HIGH);
    log_dither_point("d_low", &d_low);
    log_dither_point("d_high", &d_high);
    log_alt_fits(&d_low, &d_high);

    // The production fit uses a low sample quantile at both
    // points: interference is one-sided and lands hardest on the
    // long N_HIGH samples, so only the low tail is uncontaminated.
    let (frame_raw, loop_per_iter_ns) = two_point_fit(
        d_low.low_q_ns[DITHER_PROD_Q],
        d_high.low_q_ns[DITHER_PROD_Q],
    );
    if frame_raw < 0.0 {
        log::warn!(
            "calibration: frame/sample fit intercept is negative ({frame_raw:.4} ns; \
             d_low_q={:.4}, d_high_q={:.4}, spreads {:.1}/{:.1}) — clamping to 0, \
             so per-sample overhead will be under-subtracted",
            d_low.low_q_ns[DITHER_PROD_Q],
            d_high.low_q_ns[DITHER_PROD_Q],
            d_low.window_spread_ns,
            d_high.window_spread_ns,
        );
    }
    let frame_sample_ns = frame_raw.max(0.0);

    // Call-to-call: two window passes at N_LOW differing only in
    // the per-sample timer pair, so the loop term cancels in the
    // subtraction rather than being estimated from the dithered
    // slope (a different call site, and in an unoptimized build a
    // ~12% different loop — see
    // notes/chores/chores-04.md#call-site-codegen-and-the-frame_call-subtraction).
    let w_low = measure_window(&mut bench, W_LOW_WINDOWS, W_LOW_SAMPLES, N_LOW);
    let l_low = measure_loop_only(&mut bench, W_LOW_WINDOWS, W_LOW_SAMPLES, N_LOW);
    let frame_call_raw = w_low - l_low;

    // Cross-check against the retired slope-based derivation. The
    // two agree when both passes measure the same loop; a large
    // gap is the signal that they don't.
    log::debug!(
        "frame_call: paired={frame_call_raw:.4} ns (w_low={w_low:.4}, l_low={l_low:.4}), \
         slope-based={:.4} ns",
        w_low - N_LOW as f64 * loop_per_iter_ns,
    );
    if frame_call_raw <= 0.0 {
        log::warn!(
            "calibration: frame/call came out non-positive ({frame_call_raw:.4} ns; \
             w_low={w_low:.4}, l_low={l_low:.4}) — the two window passes disagree, \
             so inner sizing falls back to a floor of 1 ns and reported values may \
             be mis-scaled"
        );
    }
    let frame_call_ns = frame_call_raw.max(0.0);

    Overhead {
        frame_call_ns,
        frame_sample_ns,
        loop_per_iter_ns,
        cal_w_low_ns: w_low,
        cal_l_low_ns: l_low,
        cal_d_low: d_low,
        cal_d_high: d_high,
        cal_duration: cal_start.elapsed(),
    }
}

/// Measure one dithered point: `windows` windows of `samples`
/// samples at the given `inner`, each sample preceded by a random
/// sub-quantum [`Dither::spin`] outside the timed interval.
fn dither_measure(dither: &mut Dither, windows: u64, samples: u64, inner: u64) -> DitherPoint {
    let mut bench = EmptyBench;
    let mut all: Vec<u64> = Vec::with_capacity((windows * samples) as usize);
    let mut window_means: Vec<f64> = Vec::with_capacity(windows as usize);
    for _ in 0..windows {
        let mut sum: u128 = 0;
        for _ in 0..samples {
            dither.spin();
            let start = Instant::now();
            run_inner(&mut bench, inner);
            let e = start.elapsed().as_nanos() as u64;
            sum += u128::from(e);
            all.push(e);
        }
        window_means.push(sum as f64 / samples as f64);
    }

    all.sort_unstable();
    let n = all.len();
    let mean_ns = all.iter().map(|&v| v as f64).sum::<f64>() / n as f64;
    let n99 = (n as f64 * 0.99).ceil() as usize;
    let mean_p99_ns = all[..n99].iter().map(|&v| v as f64).sum::<f64>() / n99 as f64;

    window_means.sort_unstable_by(|a, b| a.total_cmp(b));
    let median_window_ns = window_means[window_means.len() / 2];
    let window_spread_ns = window_means[window_means.len() - 1] - window_means[0];
    let fast = (DITHER_FAST_WINDOWS as usize).clamp(1, window_means.len());
    let mean_fast_ns = window_means[..fast].iter().sum::<f64>() / fast as f64;

    let mut low_q_ns = [0.0; DITHER_LOW_Q.len()];
    for (slot, frac) in low_q_ns.iter_mut().zip(DITHER_LOW_Q) {
        let idx = ((n as f64 * frac) as usize).min(n - 1);
        *slot = all[idx] as f64;
    }

    DitherPoint {
        mean_ns,
        mean_p99_ns,
        mean_fast_ns,
        low_q_ns,
        median_window_ns,
        window_spread_ns,
        min_ns: all[0],
    }
}

/// Amortized per-sample cost at a given `inner`: each window times
/// `samples` complete samples (timer pair around an `inner`-iteration
/// loop — the exact shape the harness takes one at a time) and
/// divides by the count, so quantization error is q/samples; the
/// minimum over `windows` windows is returned, in ns.
fn measure_window(bench: &mut EmptyBench, windows: u64, samples: u64, inner: u64) -> f64 {
    let mut min_ns = f64::INFINITY;
    for _ in 0..windows {
        let window = Instant::now();
        for _ in 0..samples {
            let start = Instant::now();
            run_inner(bench, inner);
            black_box(start.elapsed());
        }
        let per_sample = window.elapsed().as_nanos() as f64 / samples as f64;
        if per_sample < min_ns {
            min_ns = per_sample;
        }
    }
    min_ns
}

/// [`measure_window`] with the per-sample timer pair removed: the
/// loop's own cost per `inner` iterations, same window shape, same
/// min-over-windows estimator, same [`run_inner`].
///
/// - Paired with [`measure_window`], the difference is the
///   call-to-call framing cost with the loop term cancelled
///   exactly — no fitted slope, no second call site.
fn measure_loop_only(bench: &mut EmptyBench, windows: u64, samples: u64, inner: u64) -> f64 {
    let mut min_ns = f64::INFINITY;
    for _ in 0..windows {
        let window = Instant::now();
        for _ in 0..samples {
            run_inner(bench, inner);
        }
        let per_sample = window.elapsed().as_nanos() as f64 / samples as f64;
        if per_sample < min_ns {
            min_ns = per_sample;
        }
    }
    min_ns
}

/// Two-point linear fit over the [`N_LOW`] / [`N_HIGH`] pair,
/// shared by the production fit, the alternative-fit logging, and
/// the `calibrate` command's diagnostic output. Returns
/// `(intercept_ns, slope_ns)`; the slope clamps to 0 when the
/// points invert, and the intercept is left unclamped (a negative
/// value is itself a diagnostic signal).
pub fn two_point_fit(low_ns: f64, high_ns: f64) -> (f64, f64) {
    let slope = if high_ns > low_ns {
        (high_ns - low_ns) / (N_HIGH - N_LOW) as f64
    } else {
        0.0
    };
    (low_ns - N_LOW as f64 * slope, slope)
}

/// Debug-log one dithered point's aggregates.
fn log_dither_point(name: &str, p: &DitherPoint) {
    log::debug!(
        "dither {name}: mean={:.4} p99mean={:.4} fast={:.4} medwin={:.4} spread={:.4} min={} ns",
        p.mean_ns,
        p.mean_p99_ns,
        p.mean_fast_ns,
        p.median_window_ns,
        p.window_spread_ns,
        p.min_ns,
    );
}

/// Debug-log the alternative fits (full / p99 / fast / medwin) so
/// estimator agreement stays observable run to run; `fast` is the
/// production fit. Under contention the others go negative while
/// `fast` should not — that divergence is the diagnostic.
fn log_alt_fits(d_low: &DitherPoint, d_high: &DitherPoint) {
    for (kind, low, high) in fit_candidates(d_low, d_high) {
        let (intercept, slope) = two_point_fit(low, high);
        log::debug!(
            "dither fit({kind}): in-interval framing={intercept:.4} ns, loop_per_iter={slope:.6} ns"
        );
    }
}

/// Every aggregation the two points can be fitted through, as
/// `(label, low, high)` — the single list behind both the `-v`
/// debug log and the `calibrate` command's stdout block, so the
/// two can't drift.
///
/// - The means come first, then one entry per [`DITHER_LOW_Q`]
///   discard fraction. The production fit is the `lowq` entry at
///   [`DITHER_PROD_Q`].
pub fn fit_candidates(d_low: &DitherPoint, d_high: &DitherPoint) -> Vec<(String, f64, f64)> {
    let mut out = vec![
        ("full".to_string(), d_low.mean_ns, d_high.mean_ns),
        ("p99".to_string(), d_low.mean_p99_ns, d_high.mean_p99_ns),
        ("fast".to_string(), d_low.mean_fast_ns, d_high.mean_fast_ns),
        (
            "medwin".to_string(),
            d_low.median_window_ns,
            d_high.median_window_ns,
        ),
    ];
    for (i, frac) in DITHER_LOW_Q.iter().enumerate() {
        out.push((
            format!("lowq{:.1}%", frac * 100.0),
            d_low.low_q_ns[i],
            d_high.low_q_ns[i],
        ));
    }
    out
}
