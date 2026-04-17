//! Two-threaded `std::sync::mpsc` round-trip bench.

use std::hint::black_box;
use std::sync::mpsc;
use std::thread;

use crate::harness::{self, Bench, RunCfg};
use crate::pin;

/// Registry name used on the CLI.
pub const NAME: &str = "mpsc-2t";

/// Main → worker → main round-trip over two `std::sync::mpsc`
/// channels. Measures wake/cross-core cost when the worker parks,
/// or the spin-spin fast path when both ends stay hot on the same
/// CCX.
pub struct StdMpsc2Thread {
    req_tx: mpsc::Sender<u64>,
    resp_rx: mpsc::Receiver<u64>,
    worker: Option<thread::JoinHandle<()>>,
    counter: u64,
}

impl StdMpsc2Thread {
    /// Spawn the echo worker, optionally pinning it to `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_tx, req_rx) = mpsc::channel::<u64>();
        let (resp_tx, resp_rx) = mpsc::channel::<u64>();
        let worker = thread::spawn(move || {
            pin::pin_current(worker_cpu);
            while let Ok(v) = req_rx.recv() {
                if resp_tx.send(v).is_err() {
                    break;
                }
            }
        });
        Self {
            req_tx,
            resp_rx,
            worker: Some(worker),
            counter: 0,
        }
    }
}

impl Bench for StdMpsc2Thread {
    fn name(&self) -> &str {
        "std::sync::mpsc round-trip (2 threads)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        self.req_tx.send(self.counter).unwrap();
        let v = self.resp_rx.recv().unwrap();
        black_box(v)
    }
}

impl Drop for StdMpsc2Thread {
    fn drop(&mut self) {
        // Replace req_tx with a dummy so we can drop the real one;
        // worker's recv() then returns Err and the worker exits.
        // Done this way (instead of Option<Sender>) to keep step() branch-free.
        let (dummy_tx, _) = mpsc::channel();
        drop(std::mem::replace(&mut self.req_tx, dummy_tx));
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = StdMpsc2Thread::new(cfg.core_for(1));
    let (hist, outer, inner, duration_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_report(bench.name(), outer, inner, duration_s, &hist, cfg.overhead);
}
