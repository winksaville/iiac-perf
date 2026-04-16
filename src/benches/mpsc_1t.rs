use std::hint::black_box;
use std::sync::mpsc;

use crate::harness::{self, Bench, RunCfg};

pub const NAME: &str = "mpsc-1t";

pub struct StdMpscRoundTrip {
    tx: mpsc::Sender<u64>,
    rx: mpsc::Receiver<u64>,
    counter: u64,
}

impl StdMpscRoundTrip {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self { tx, rx, counter: 0 }
    }
}

impl Bench for StdMpscRoundTrip {
    fn name(&self) -> &str {
        "std::sync::mpsc round-trip (1 thread)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        self.tx.send(self.counter).unwrap();
        let v = self.rx.recv().unwrap();
        black_box(v)
    }
}

pub fn run(cfg: &RunCfg) {
    let mut bench = StdMpscRoundTrip::new();
    let (hist, outer, inner, duration_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_histogram(bench.name(), outer, inner, duration_s, &hist, cfg.overhead);
}
