//! Single-threaded `crossbeam_channel::unbounded` round-trip bench.

use std::hint::black_box;

use crossbeam_channel::{Receiver, Sender, unbounded};

use crate::harness::{self, Bench, RunCfg};
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "cb-chan-1t";

/// Same-thread send-then-receive through one unbounded crossbeam
/// channel. Measures the channel's own overhead with no scheduler
/// interaction, against `mpsc-1t`'s std wrapper over the same
/// crossbeam code (std's `mpsc` has been crossbeam underneath
/// since Rust 1.67).
///
/// - Capability class: the channel is MPMC. A queue that promises
///   less is expected to be faster, so read this row beside the
///   MPSC and SPSC rows with that in mind.
pub struct CbChanRoundTrip {
    tx: Sender<u64>,
    rx: Receiver<u64>,
    counter: u64,
}

impl CbChanRoundTrip {
    /// Construct the bench with a fresh unbounded channel.
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx, counter: 0 }
    }
}

impl Bench for CbChanRoundTrip {
    fn name(&self) -> &str {
        "cb-chan-1t: crossbeam_channel::unbounded round-trip (1 thread)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        // OK: an unbounded send fails only when every receiver is
        // gone, and `rx` lives in this struct.
        self.tx.send(self.counter).unwrap();
        // OK: `recv` fails only when the channel is empty and every
        // sender is gone, and `tx` just sent.
        let v = self.rx.recv().unwrap();
        black_box(v)
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = CbChanRoundTrip::new();
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
}
