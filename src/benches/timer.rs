use std::hint::black_box;

use crate::harness::{self, Bench};
use crate::overhead::Overhead;

pub struct MinstantNow;

impl Bench for MinstantNow {
    fn name(&self) -> &str {
        "minstant::Instant::now()"
    }

    fn step(&mut self) -> u32 {
        black_box(minstant::Instant::now());
        1
    }
}

pub struct StdInstantNow;

impl Bench for StdInstantNow {
    fn name(&self) -> &str {
        "std::time::Instant::now()"
    }

    fn step(&mut self) -> u32 {
        black_box(std::time::Instant::now());
        1
    }
}

pub fn run(iterations: u64, overhead: &Overhead) {
    let mut minstant_bench = MinstantNow;
    let minstant_hist = harness::run_bench(&mut minstant_bench, iterations);
    harness::print_histogram(minstant_bench.name(), iterations, &minstant_hist, overhead);

    let mut std_bench = StdInstantNow;
    let std_hist = harness::run_bench(&mut std_bench, iterations);
    harness::print_histogram(std_bench.name(), iterations, &std_hist, overhead);
}
