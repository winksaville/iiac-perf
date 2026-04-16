use std::hint::black_box;

use crate::harness::{self, Bench, RunCfg};

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

pub fn run(cfg: &RunCfg) {
    let mut bench = MinstantNow;
    let (hist, outer, inner, duration_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_histogram(bench.name(), outer, inner, duration_s, &hist, cfg.overhead);
}
