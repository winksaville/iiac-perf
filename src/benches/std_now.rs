//! `std::time::Instant::now()` call-cost bench.

use std::hint::black_box;

use crate::harness::{self, Bench, RunCfg};
use crate::record;
use crate::report;

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
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
}
