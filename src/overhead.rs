use std::hint::black_box;

use crate::harness::{Bench, CALIBRATION_INNER, WARMUP};

pub struct Overhead {
    pub per_sample_min_ns: u64,
    pub calls_per_sample: u64,
}

impl Overhead {
    pub fn per_call_ns(&self) -> f64 {
        self.per_sample_min_ns as f64 / self.calls_per_sample as f64
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

pub fn calibrate(samples: u64) -> Overhead {
    let mut bench = EmptyBench;

    for _ in 0..WARMUP {
        black_box(bench.step());
    }

    let mut min_ns = u64::MAX;
    for _ in 0..samples {
        let start = minstant::Instant::now();
        for _ in 0..CALIBRATION_INNER {
            black_box(bench.step());
        }
        let e = start.elapsed().as_nanos() as u64;
        if e < min_ns {
            min_ns = e;
        }
    }
    Overhead {
        per_sample_min_ns: min_ns,
        calls_per_sample: CALIBRATION_INNER,
    }
}
