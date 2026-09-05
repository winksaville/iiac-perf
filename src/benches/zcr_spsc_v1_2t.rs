//! Two-threaded zc-ring-x1 spsc v1 round-trip bench, closure
//! (`reserve_slot_with`) API, spin waits, the seam-word ring.

use std::hint::black_box;
use std::thread;

use zc_ring_x1::spsc::v1::{Consumer, Producer};

use crate::benches::zcr_common::{Msg, STOP, leak_v1_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::pin;
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-spsc-v1-2t";

/// Main to worker to main round-trip over two v1 rings, both
/// ends waiting inside `reserve_slot_with` with an app-supplied
/// spin closure, the shape of `zcr-spsc-v0-2t` over the seam-word
/// protocol.
///
/// - Wait policy: a `spin_loop` hint per failed attempt, so the
///   measurement is the seam-word handoff under real cross-core
///   traffic, where v1's design claim lives: each end polls the
///   slot's seq word and never the other end's index line.
/// - Shutdown: `Drop` sends the [`STOP`] sentinel, and the worker
///   exits on receipt without replying.
pub struct ZcrSpscV1TwoThread {
    req_tx: Producer<'static>,
    resp_rx: Consumer<'static>,
    worker: Option<thread::JoinHandle<()>>,
    counter: u64,
}

impl ZcrSpscV1TwoThread {
    /// Spawn the spinning echo worker over two fresh leaked v1
    /// rings, optionally pinning it to `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_tx, mut req_rx) = leak_v1_ring();
        let (mut resp_tx, resp_rx) = leak_v1_ring();
        let worker = thread::spawn(move || {
            pin::pin_current(worker_cpu);
            loop {
                let v = {
                    let slot = req_rx
                        .reserve_slot_with::<Msg>(|_| {
                            core::hint::spin_loop();
                            true
                        })
                        // OK: the closure returns true forever, so
                        // the reserve never gives up.
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
                    // OK: as above, the closure never gives up.
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

impl Bench for ZcrSpscV1TwoThread {
    fn name(&self) -> &str {
        "zcr-spsc-v1-2t: zc-ring-x1 spsc v1 reserve_slot_with round-trip (2 threads, spin)"
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
            // OK: the closure returns true forever, so the reserve
            // never gives up.
            .expect("spin closure never gives up");
        *slot = self.counter;
        slot.commit();
        let slot = self
            .resp_rx
            .reserve_slot_with::<Msg>(|_| {
                core::hint::spin_loop();
                true
            })
            // OK: as above, the closure never gives up.
            .expect("spin closure never gives up");
        let v = *slot;
        slot.release();
        black_box(v)
    }
}

impl Drop for ZcrSpscV1TwoThread {
    /// Send [`STOP`] and join the worker.
    fn drop(&mut self) {
        let mut slot = self
            .req_tx
            .reserve_slot_with::<Msg>(|_| {
                core::hint::spin_loop();
                true
            })
            // OK: the closure returns true forever, so the reserve
            // never gives up.
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
    let mut bench = ZcrSpscV1TwoThread::new(cfg.cpu_for(1));
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
}
