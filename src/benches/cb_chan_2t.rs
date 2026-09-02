//! Two-threaded `crossbeam_channel::unbounded` round-trip bench.

use std::hint::black_box;
use std::thread;

use crossbeam_channel::{Receiver, Sender, unbounded};

use crate::harness::{self, Bench, RunCfg};
use crate::pin;
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "cb-chan-2t";

/// Main → worker → main round-trip over two unbounded crossbeam
/// channels, the same shape and wait policy as `mpsc-2t`: both
/// ends block in `recv`, so the number is the park/wake cost when
/// the worker sleeps, or the spin-spin fast path when both ends
/// stay hot. `mpsc-2t` against this bench is the std wrapper's
/// cost over the same crossbeam code.
///
/// - Capability class: the channel is MPMC. Its spinning peers are
///   `mpsc-2t-spin` and the zcr 2t rows, not this bench's blocking
///   twin.
pub struct CbChan2Thread {
    req_tx: Sender<u64>,
    resp_rx: Receiver<u64>,
    worker: Option<thread::JoinHandle<()>>,
    counter: u64,
}

impl CbChan2Thread {
    /// Spawn the echo worker, optionally pinning it to `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_tx, req_rx) = unbounded::<u64>();
        let (resp_tx, resp_rx) = unbounded::<u64>();
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

impl Bench for CbChan2Thread {
    fn name(&self) -> &str {
        "cb-chan-2t: crossbeam_channel::unbounded round-trip (2 threads)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        // OK: an unbounded send fails only when the worker has
        // dropped `req_rx`, which it does only after `Drop` here
        // disconnects it.
        self.req_tx.send(self.counter).unwrap();
        // OK: `recv` fails only when the worker is gone, and the
        // worker replies to every request until `Drop`.
        let v = self.resp_rx.recv().unwrap();
        black_box(v)
    }
}

impl Drop for CbChan2Thread {
    fn drop(&mut self) {
        // Replace req_tx with a dummy so we can drop the real one;
        // the worker's recv() then returns Err and the worker
        // exits. Same shape as mpsc-2t's Drop, keeping step()
        // branch-free.
        let (dummy_tx, _) = unbounded();
        drop(std::mem::replace(&mut self.req_tx, dummy_tx));
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = CbChan2Thread::new(cfg.cpu_for(1));
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
}
