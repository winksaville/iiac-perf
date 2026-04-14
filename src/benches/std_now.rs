use std::hint::black_box;

use crate::harness::{self, Bench};
use crate::overhead::Overhead;

pub const NAME: &str = "std-now";

pub struct StdInstantNow;

impl Bench for StdInstantNow {
    fn name(&self) -> &str {
        "std::time::Instant::now()"
    }

    fn step(&mut self) -> u64 {
        black_box(std::time::Instant::now());
        1
    }
}

pub fn run(iterations: u64, overhead: &Overhead) {
    let mut bench = StdInstantNow;
    let hist = harness::run_bench(&mut bench, iterations);
    harness::print_histogram(bench.name(), iterations, &hist, overhead);
}
