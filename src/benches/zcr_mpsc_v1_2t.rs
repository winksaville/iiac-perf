//! Two-threaded zc-ring-x1 mpsc v1 round-trip bench, closure
//! (`send_with`) API, spin waits, the equality-seq ring.

use std::hint::black_box;
use std::thread;

use zc_ring_x1::mpsc::v1::{MpscConsumer, MpscProducer};

use crate::benches::zcr_common::{Msg, STOP, leak_mpsc_v1_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::pin;
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-mpsc-v1-2t";

/// Main to worker to main round-trip over two zc-ring-x1 v1 MPSC
/// rings, one producer per ring, the shape of `zcr-mpsc-v0-2t`
/// over the equality-seq protocol, so the pair at one placement
/// is the cross-core handoff of the two rings side by side. The
/// prediction on record in zc-ring-x1 is v0's cost at every depth
/// above 1.
///
/// - Wait policy: a `spin_loop` hint per failed attempt on both
///   the send and receive sides.
/// - Shutdown: `Drop` sends the [`STOP`] sentinel, and the worker
///   exits on receipt without replying.
pub struct ZcrMpscV1TwoThread {
    req_tx: MpscProducer<'static>,
    resp_rx: MpscConsumer<'static>,
    worker: Option<thread::JoinHandle<()>>,
    counter: u64,
}

impl ZcrMpscV1TwoThread {
    /// Spawn the spinning echo worker over two fresh leaked v1
    /// MPSC rings, optionally pinning it to `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_tx, mut req_rx) = leak_mpsc_v1_ring();
        let (resp_tx, resp_rx) = leak_mpsc_v1_ring();
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
                resp_tx
                    .send_with::<Msg>(
                        |_| {
                            core::hint::spin_loop();
                            true
                        },
                        |m| *m = v,
                    )
                    // OK: as above, the closure never gives up.
                    .expect("spin closure never gives up");
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

impl Bench for ZcrMpscV1TwoThread {
    fn name(&self) -> &str {
        "zcr-mpsc-v1-2t: zc-ring-x1 mpsc v1 send_with round-trip (2 threads, spin)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        if self.counter == STOP {
            self.counter = 1;
        }
        let c = self.counter;
        self.req_tx
            .send_with::<Msg>(
                |_| {
                    core::hint::spin_loop();
                    true
                },
                |m| *m = c,
            )
            // OK: the closure returns true forever, so the send
            // never gives up.
            .expect("spin closure never gives up");
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

impl Drop for ZcrMpscV1TwoThread {
    /// Send [`STOP`] and join the worker.
    fn drop(&mut self) {
        self.req_tx
            .send_with::<Msg>(
                |_| {
                    core::hint::spin_loop();
                    true
                },
                |m| *m = STOP,
            )
            // OK: the closure returns true forever, so the send
            // never gives up.
            .expect("spin closure never gives up");
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = ZcrMpscV1TwoThread::new(cfg.cpu_for(1));
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
}
