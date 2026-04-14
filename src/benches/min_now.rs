use std::hint::black_box;

use crate::harness::{self, Bench};
use crate::overhead::Overhead;

pub const NAME: &str = "min-now";

pub struct MinstantNow;

impl Bench for MinstantNow {
    fn name(&self) -> &str {
        "minstant::Instant::now()"
    }

    fn step(&mut self) -> u64 {
        black_box(minstant::Instant::now());
        1
    }
}

pub fn run(iterations: u64, overhead: &Overhead) {
    let mut bench = MinstantNow;
    let hist = harness::run_bench(&mut bench, iterations);
    harness::print_histogram(bench.name(), iterations, &hist, overhead);
}
