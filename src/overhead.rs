use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::harness::Bench;

pub const CAL_WARMUP: u64 = 100_000;
pub const CAL_SAMPLES: u64 = 100_000;
pub const N_LOW: u64 = 100;
pub const N_HIGH: u64 = 10_000;

#[derive(Debug)]
pub struct Overhead {
    pub framing_per_sample_ns: f64,
    pub loop_per_iter_ns: f64,
    pub cal_min_low_ns: u64,
    pub cal_min_high_ns: u64,
    pub cal_duration: Duration,
}

impl Overhead {
    pub fn per_call_ns(&self, inner: u64) -> f64 {
        self.framing_per_sample_ns / inner as f64 + self.loop_per_iter_ns
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

pub fn calibrate() -> Overhead {
    let mut bench = EmptyBench;
    let cal_start = Instant::now();
    for _ in 0..CAL_WARMUP {
        black_box(bench.step());
    }

    let min_low = measure(&mut bench, CAL_SAMPLES, N_LOW);
    let min_high = measure(&mut bench, CAL_SAMPLES, N_HIGH);

    // Two-point fit: per_sample = framing + inner * loop_per_iter.
    // Slope between (N_LOW, min_low) and (N_HIGH, min_high) gives loop_per_iter,
    // intercept gives framing. Cancels TSC pipelining of the framing pair.
    //
    // Noise amplification: d(framing)/d(min_low) = N_HIGH / (N_HIGH - N_LOW).
    // At 100 / 10_000 that's ~1.01, so ~10 ns of slop on min_low produces
    // only ~10 ns on framing. Previously 100 / 1_000 gave ~1.11, which was
    // the cause of the "exact 2×" framing anomaly pre-0.6.0.
    let loop_per_iter_ns = if min_high > min_low {
        (min_high - min_low) as f64 / (N_HIGH - N_LOW) as f64
    } else {
        0.0
    };
    let framing_per_sample_ns = (min_low as f64 - N_LOW as f64 * loop_per_iter_ns).max(0.0);

    Overhead {
        framing_per_sample_ns,
        loop_per_iter_ns,
        cal_min_low_ns: min_low,
        cal_min_high_ns: min_high,
        cal_duration: cal_start.elapsed(),
    }
}

fn measure(bench: &mut EmptyBench, samples: u64, inner: u64) -> u64 {
    let mut min_ns = u64::MAX;
    for _ in 0..samples {
        let start = minstant::Instant::now();
        for _ in 0..inner {
            black_box(bench.step());
        }
        let e = start.elapsed().as_nanos() as u64;
        if e < min_ns {
            min_ns = e;
        }
    }
    min_ns
}
