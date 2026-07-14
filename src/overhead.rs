//! Apparatus-overhead calibration, fully amortized.
//!
//! Two amortized window measurements at two `inner` sizes give a
//! two-point fit whose every input has quantization error q/M
//! instead of the full ~10 ns clock quantum:
//!
//! - slope → `loop_per_iter_ns`, the per-iteration loop cost —
//!   the only constant *subtracted* from reported values.
//! - intercept → `frame_call_ns`, the call-to-call cost of taking
//!   one sample — drives experiment sizing, never subtraction.
//!
//! The in-interval slice of the timer-pair cost (what lands inside
//! a recorded sample) is deliberately not estimated: it cannot be
//! amortized by construction, so any estimate carries the full
//! quantum of error. It stays in reported values as a small bounded
//! residue. See
//! notes/design.md#timer-overhead-in-interval-vs-call-to-call.

use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::harness::Bench;

/// Calibration warmup iterations — long enough for CPU frequency
/// boost to ramp before the first window measurement.
pub const CAL_WARMUP: u64 = 100_000;

/// Inner-loop count for the low-N calibration point.
pub const N_LOW: u64 = 100;

/// Inner-loop count for the high-N calibration point. A wide spread
/// (`N_HIGH / (N_HIGH - N_LOW) ≈ 1.01`) keeps noise amplification on
/// the fitted intercept small.
pub const N_HIGH: u64 = 10_000;

/// Samples per window at the low point. The window's ±1-quantum
/// error divides by this: ~0.001 ns at a ~10 ns quantum.
pub const W_LOW_SAMPLES: u64 = 10_000;

/// Windows measured at the low point; the minimum is kept. The min
/// sheds windows inflated by preemption or a frequency dip while
/// staying amortized (each candidate is already a window mean).
pub const W_LOW_WINDOWS: u64 = 100;

/// Samples per window at the high point (samples are ~100× longer
/// there, so fewer are needed for the same absolute q/M error).
pub const W_HIGH_SAMPLES: u64 = 1_000;

/// Windows measured at the high point; the minimum is kept.
pub const W_HIGH_WINDOWS: u64 = 20;

/// Apparatus-overhead model fitted by [`calibrate`].
#[derive(Debug)]
pub struct Overhead {
    /// Call-to-call cost of taking one sample (full timer-pair
    /// apparatus cost, clock-read latencies included), in ns.
    /// Sizes the experiment ([`crate::harness`]'s `pick_inner`);
    /// most of it sits *outside* recorded intervals, so it is
    /// never subtracted from reported values.
    pub frame_call_ns: f64,
    /// Per-inner-iteration loop overhead (branch + `black_box`),
    /// in ns — the amortized slope, and the per-call subtraction
    /// constant (see [`Overhead::adjust_per_call_ns`]).
    pub loop_per_iter_ns: f64,
    /// Raw amortized per-sample window minimum at [`N_LOW`] (ns).
    /// Preserved for `-v` logging and cache provenance.
    pub cal_w_low_ns: f64,
    /// Raw amortized per-sample window minimum at [`N_HIGH`] (ns).
    pub cal_w_high_ns: f64,
    /// Wall-clock duration of the full calibration run.
    pub cal_duration: Duration,
}

impl Overhead {
    /// Per-call apparatus overhead subtracted from reported values,
    /// in ns: the amortized loop cost only.
    ///
    /// - The in-interval timer-pair slice is *not* subtracted — it
    ///   cannot be measured to better than the ~10 ns clock quantum,
    ///   and with `inner` sized from [`Overhead::frame_call_ns`] the
    ///   residue left in reported values is bounded by roughly
    ///   quantum / inner (~1-2% of step cost).
    pub fn adjust_per_call_ns(&self) -> f64 {
        self.loop_per_iter_ns
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

/// Run the amortized two-point calibration and return the fitted
/// [`Overhead`]. Blocks for ~200-300 ms on a typical modern x86.
pub fn calibrate() -> Overhead {
    let mut bench = EmptyBench;
    let cal_start = Instant::now();
    for _ in 0..CAL_WARMUP {
        black_box(bench.step());
    }

    let w_low = measure_window(&mut bench, W_LOW_WINDOWS, W_LOW_SAMPLES, N_LOW);
    let w_high = measure_window(&mut bench, W_HIGH_WINDOWS, W_HIGH_SAMPLES, N_HIGH);

    // Two-point fit: per_sample = frame_call + inner * loop_per_iter.
    // Both points are window-amortized (q/M quantization), so slope
    // and intercept are stable to ~±0.1% warm — the old min-based
    // fit's intercept inherited a full ~10 ns quantum and drew
    // 0-21 ns run to run (see
    // notes/design.md#calibration-accuracy-framing-quantization).
    let loop_per_iter_ns = if w_high > w_low {
        (w_high - w_low) / (N_HIGH - N_LOW) as f64
    } else {
        0.0
    };
    let frame_call_ns = (w_low - N_LOW as f64 * loop_per_iter_ns).max(0.0);

    Overhead {
        frame_call_ns,
        loop_per_iter_ns,
        cal_w_low_ns: w_low,
        cal_w_high_ns: w_high,
        cal_duration: cal_start.elapsed(),
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
            for _ in 0..inner {
                black_box(bench.step());
            }
            black_box(start.elapsed());
        }
        let per_sample = window.elapsed().as_nanos() as f64 / samples as f64;
        if per_sample < min_ns {
            min_ns = per_sample;
        }
    }
    min_ns
}

/// Dither span in empty-bench iterations (~0.4-0.5 ns each →
/// ~26-32 ns, spanning ~3 clock quanta). A random 0..span delay
/// before each sample randomizes its phase on the ~10 ns clock
/// lattice, making the quantization error zero-mean (see
/// notes/design.md#dithering-random-phase-injection).
pub const DITHER_SPAN: u64 = 64;

/// Dither-experiment windows per point (each yields one window
/// mean; the median across windows is the robust aggregate).
pub const DITHER_WINDOWS: u64 = 20;

/// Dither-experiment samples per window at `N_LOW`.
pub const DITHER_LOW_SAMPLES: u64 = 5_000;

/// Dither-experiment samples per window at `N_HIGH` (samples are
/// ~100× longer, so fewer keep the wall cost comparable).
pub const DITHER_HIGH_SAMPLES: u64 = 500;

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

/// Aggregates of one dithered measurement point (all ns):
/// linear statistics that keep the dither win, plus min for
/// reference against the lattice floor.
#[derive(Debug)]
pub struct DitherPoint {
    /// Full mean over all samples — unbiased under dither but
    /// absorbs interrupt spikes.
    pub mean_ns: f64,
    /// Mean of samples ≤ p99 — sheds one-sided interrupt
    /// contamination at a small estimable bias.
    pub mean_p99_ns: f64,
    /// Median of per-window means — robust to a bad window
    /// without snapping (window means are not lattice-valued).
    pub median_window_ns: f64,
    /// Spread (max − min) of the window means — a dispersion
    /// signal for a CI and for regime-shift detection.
    pub window_spread_ns: f64,
    /// Minimum sample — the lattice floor, for comparison.
    pub min_ns: u64,
}

/// Measure one dithered point: `windows` windows of `samples`
/// samples at the given `inner`, each sample preceded by a random
/// 0..[`DITHER_SPAN`]-iteration delay outside the timed interval.
fn dither_measure(rng: &mut XorShift64, windows: u64, samples: u64, inner: u64) -> DitherPoint {
    let mut bench = EmptyBench;
    let mut all: Vec<u64> = Vec::with_capacity((windows * samples) as usize);
    let mut window_means: Vec<f64> = Vec::with_capacity(windows as usize);
    for _ in 0..windows {
        let mut sum: u128 = 0;
        for _ in 0..samples {
            let r = rng.next() % DITHER_SPAN;
            for _ in 0..r {
                black_box(bench.step());
            }
            let start = Instant::now();
            for _ in 0..inner {
                black_box(bench.step());
            }
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

    DitherPoint {
        mean_ns,
        mean_p99_ns,
        median_window_ns,
        window_spread_ns,
        min_ns: all[0],
    }
}

/// Run the dithered two-point experiment and log its fits at
/// debug level. Validation vehicle for calibration v3: if the
/// in-interval intercept proves stable run-to-run, the dithered
/// fit becomes the calibration (with subtraction restored); see
/// the 0.21.0 In Progress plan and
/// notes/design.md#why-dither-works-and-which-statistics-keep-the-win.
///
/// - Costs ~70 ms; callers gate it (main runs it only when
///   debug logging is enabled).
pub fn dither_experiment() {
    // Seed from wall-clock nanos: any per-invocation variation
    // suffices for phase dither.
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64 | 1)
        .unwrap_or(0x9E37_79B9_7F4A_7C15); // OK: fixed fallback seed still dithers
    let mut rng = XorShift64(seed);

    let d_low = dither_measure(&mut rng, DITHER_WINDOWS, DITHER_LOW_SAMPLES, N_LOW);
    let d_high = dither_measure(&mut rng, DITHER_WINDOWS, DITHER_HIGH_SAMPLES, N_HIGH);

    log::debug!(
        "dither d_low:  mean={:.4} p99mean={:.4} medwin={:.4} spread={:.4} min={} ns",
        d_low.mean_ns,
        d_low.mean_p99_ns,
        d_low.median_window_ns,
        d_low.window_spread_ns,
        d_low.min_ns,
    );
    log::debug!(
        "dither d_high: mean={:.4} p99mean={:.4} medwin={:.4} spread={:.4} min={} ns",
        d_high.mean_ns,
        d_high.mean_p99_ns,
        d_high.median_window_ns,
        d_high.window_spread_ns,
        d_high.min_ns,
    );

    for (kind, low, high) in [
        ("full", d_low.mean_ns, d_high.mean_ns),
        ("p99", d_low.mean_p99_ns, d_high.mean_p99_ns),
        ("medwin", d_low.median_window_ns, d_high.median_window_ns),
    ] {
        let slope = if high > low {
            (high - low) / (N_HIGH - N_LOW) as f64
        } else {
            0.0
        };
        let intercept = low - N_LOW as f64 * slope;
        log::debug!(
            "dither fit({kind}): in-interval framing={intercept:.4} ns, loop_per_iter={slope:.6} ns"
        );
    }
}
