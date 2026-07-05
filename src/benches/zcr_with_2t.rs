//! Two-threaded zc-ring-x1 round-trip bench, closure
//! (`reserve_slot_with`) API tier, spin waits.

use std::hint::black_box;
use std::thread;

use zc_ring_x1::{Consumer, Producer};

use crate::benches::zcr_common::{Msg, STOP, leak_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::pin;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-with-2t";

/// Main → worker → main round-trip over two zc-ring-x1 rings as
/// `zcr-raw-2t`, but both ends wait inside `reserve_slot_with`
/// with an app-supplied spin closure instead of a hand-written
/// retry loop around `reserve_slot`.
///
/// - Same wait policy as `zcr-raw-2t` (spin_loop hint per failed
///   attempt), so any delta between the two is the cost of the
///   `_with` wrapper under real cross-core traffic.
/// - Shutdown: `Drop` sends the [`STOP`] sentinel; the worker
///   exits on receipt without replying.
pub struct ZcrWith2Thread {
    req_tx: Producer<'static>,
    resp_rx: Consumer<'static>,
    worker: Option<thread::JoinHandle<()>>,
    counter: u64,
}

impl ZcrWith2Thread {
    /// Spawn the spinning echo worker over two fresh leaked
    /// rings, optionally pinning it to `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_tx, mut req_rx) = leak_ring();
        let (mut resp_tx, resp_rx) = leak_ring();
        let worker = thread::spawn(move || {
            pin::pin_current(worker_cpu);
            loop {
                let v = {
                    let slot = req_rx
                        .reserve_slot_with::<Msg>(|_| {
                            core::hint::spin_loop();
                            true
                        })
                        .expect("spin closure never gives up");
                    let v = *slot;
                    slot.release();
                    v
                };
                if v == STOP {
                    break;
                }
                let mut slot = resp_tx
                    .reserve_slot_with::<Msg>(|_| {
                        core::hint::spin_loop();
                        true
                    })
                    .expect("spin closure never gives up");
                *slot = v;
                slot.commit();
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

impl Bench for ZcrWith2Thread {
    fn name(&self) -> &str {
        "zcr-with-2t: zc-ring-x1 reserve_slot_with round-trip (2 threads, spin)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        if self.counter == STOP {
            self.counter = 1;
        }
        let mut slot = self
            .req_tx
            .reserve_slot_with::<Msg>(|_| {
                core::hint::spin_loop();
                true
            })
            .expect("spin closure never gives up");
        *slot = self.counter;
        slot.commit();
        let slot = self
            .resp_rx
            .reserve_slot_with::<Msg>(|_| {
                core::hint::spin_loop();
                true
            })
            .expect("spin closure never gives up");
        let v = *slot;
        slot.release();
        black_box(v)
    }
}

impl Drop for ZcrWith2Thread {
    /// Send [`STOP`] and join the worker.
    fn drop(&mut self) {
        let mut slot = self
            .req_tx
            .reserve_slot_with::<Msg>(|_| {
                core::hint::spin_loop();
                true
            })
            .expect("spin closure never gives up");
        *slot = STOP;
        slot.commit();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = ZcrWith2Thread::new(cfg.core_for(1));
    let (hist, outer, inner, duration_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_report(bench.name(), outer, inner, duration_s, &hist, cfg.overhead);
}
