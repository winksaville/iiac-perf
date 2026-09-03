//! Two-threaded `crossbeam_queue::SegQueue` round-trip bench, spin
//! waits.

use std::hint::black_box;
use std::sync::Arc;
use std::thread;

use crossbeam_queue::SegQueue;

use crate::harness::{self, Bench, RunCfg};
use crate::pin;
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "cb-seg-2t";

/// Shutdown sentinel sent instead of a counter value; the echo
/// worker exits on receipt without replying. The counter skips it.
const STOP: u64 = u64::MAX;

/// Main → worker → main round-trip over two `SegQueue`s. The queue
/// has no blocking API, so both ends spin on `pop`, the same wait
/// policy as `mpsc-2t-spin` and the zcr 2t benches, and those are
/// its peers rather than the parking `mpsc-2t` / `cb-chan-2t`.
///
/// - Capability class: `SegQueue` is MPMC. What zc-ring-x1's
///   segmented SPSC promises is less, and this is the row it lands
///   against.
/// - Shutdown: `Drop` pushes [`STOP`]; the worker exits on receipt.
pub struct CbSeg2Thread {
    req: Arc<SegQueue<u64>>,
    resp: Arc<SegQueue<u64>>,
    worker: Option<thread::JoinHandle<()>>,
    counter: u64,
}

impl CbSeg2Thread {
    /// Spawn the spinning echo worker over two fresh queues,
    /// optionally pinning it to `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let req = Arc::new(SegQueue::<u64>::new());
        let resp = Arc::new(SegQueue::<u64>::new());
        let worker = {
            let req = Arc::clone(&req);
            let resp = Arc::clone(&resp);
            thread::spawn(move || {
                pin::pin_current(worker_cpu);
                loop {
                    let v = loop {
                        if let Some(v) = req.pop() {
                            break v;
                        }
                        core::hint::spin_loop();
                    };
                    if v == STOP {
                        break;
                    }
                    resp.push(v);
                }
            })
        };
        Self {
            req,
            resp,
            worker: Some(worker),
            counter: 0,
        }
    }
}

impl Bench for CbSeg2Thread {
    fn name(&self) -> &str {
        "cb-seg-2t: crossbeam_queue::SegQueue round-trip (2 threads, spin)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        if self.counter == STOP {
            self.counter = 1;
        }
        self.req.push(self.counter);
        loop {
            if let Some(v) = self.resp.pop() {
                return black_box(v);
            }
            core::hint::spin_loop();
        }
    }
}

impl Drop for CbSeg2Thread {
    /// Push [`STOP`] and join the worker.
    fn drop(&mut self) {
        self.req.push(STOP);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = CbSeg2Thread::new(cfg.cpu_for(1));
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
}
