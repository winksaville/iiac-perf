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
