use std::hint::black_box;

use crate::harness::{self, Bench, RunCfg};

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

pub fn run(cfg: &RunCfg) {
    let mut bench = StdInstantNow;
    let (hist, iterations, inner) = harness::run_adaptive(&mut bench, cfg);
    harness::print_histogram(bench.name(), iterations, inner, &hist, cfg.overhead);
}
