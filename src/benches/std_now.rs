//! `std::time::Instant::now()` call-cost bench.

use std::hint::black_box;

use crate::harness::{self, Bench, RunCfg};

/// Registry name used on the CLI.
pub const NAME: &str = "std-now";

/// Cost of a single `std::time::Instant::now()` call (typically
/// `CLOCK_MONOTONIC` via the vDSO on Linux).
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

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = StdInstantNow;
    let (hist, outer, inner, duration_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_report(bench.name(), outer, inner, duration_s, &hist, cfg.overhead);
}
