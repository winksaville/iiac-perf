//! `minstant::Instant::now()` call-cost bench.

use std::hint::black_box;

use crate::harness::{self, Bench, RunCfg};

/// Registry name used on the CLI.
pub const NAME: &str = "min-now";

/// Cost of a single `minstant::Instant::now()` call (rdtsc-based on
/// x86_64 with invariant TSC).
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

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = MinstantNow;
    let (hist, outer, inner, duration_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_report(bench.name(), outer, inner, duration_s, &hist, cfg.overhead);
}
