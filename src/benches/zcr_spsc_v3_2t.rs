//! Two-threaded zc-ring-x1 spsc v3 round-trip bench, closure
//! (`reserve_slot_with`) API, spin waits, the segmented ring over
//! a pool.

use std::hint::black_box;
use std::thread;

use zc_ring_x1::spsc::v3::{Consumer, Producer};

use crate::benches::zcr_common::{Msg, STOP, leak_v3_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::pin;
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-spsc-v3-2t";

/// Main to worker to main round-trip over two v3 rings, both
/// ends waiting inside `reserve_slot_with` with an app-supplied
/// spin closure, the shape of `zcr-spsc-v2-2t` over the segmented
/// ring.
///
/// - Wait policy: a `spin_loop` hint per failed attempt, so the
///   measurement is the in-slot handoff under real cross-core
///   traffic, one line per handoff as in v2, plus whatever v3's
///   look-ahead costs when no switch happens.
/// - Switches: one message in flight means the consumer keeps up
///   and neither ring leaves its first segment. The worker hands
///   its two ends' counts back at shutdown, and the four are
///   printed after the report, expected zero.
/// - Shutdown: `Drop` sends the [`STOP`] sentinel, and the worker
///   exits on receipt without replying.
pub struct ZcrSpscV3TwoThread {
    req_tx: Producer<'static>,
    resp_rx: Consumer<'static>,
    worker: Option<thread::JoinHandle<(u64, u64)>>,
    counter: u64,
}

/// Segment switches of the four ends of the two rings.
pub struct Switches {
    /// The request ring, producer then consumer.
    pub req: (u64, u64),
    /// The response ring, producer then consumer.
    pub resp: (u64, u64),
}

impl ZcrSpscV3TwoThread {
    /// Spawn the spinning echo worker over two fresh leaked v3
    /// rings, optionally pinning it to `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_tx, mut req_rx) = leak_v3_ring();
        let (mut resp_tx, resp_rx) = leak_v3_ring();
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
            (req_rx.switches(), resp_tx.switches())
        });
        Self {
            req_tx,
            resp_rx,
            worker: Some(worker),
            counter: 0,
        }
    }

    /// Send [`STOP`], join the worker, and return the four ends'
    /// switch counts. A second call returns zeros for the worker's
    /// ends, since it has gone.
    pub fn shutdown(&mut self) -> Switches {
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
        let (req_rx, resp_tx) = match self.worker.take() {
            Some(worker) => worker.join().unwrap_or((0, 0)), // OK: a panicked worker has no counts, and the bench already printed its report
            None => (0, 0),
        };
        Switches {
            req: (self.req_tx.switches(), req_rx),
            resp: (resp_tx, self.resp_rx.switches()),
        }
    }
}

impl Bench for ZcrSpscV3TwoThread {
    fn name(&self) -> &str {
        "zcr-spsc-v3-2t: zc-ring-x1 spsc v3 reserve_slot_with round-trip (2 threads, spin)"
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

impl Drop for ZcrSpscV3TwoThread {
    /// Stop the worker if [`shutdown`](Self::shutdown) has not.
    fn drop(&mut self) {
        if self.worker.is_some() {
            self.shutdown();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = ZcrSpscV3TwoThread::new(cfg.cpu_for(1));
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
    let s = bench.shutdown();
    println!(
        "segment switches: request ring producer {}, consumer {}; response ring producer {}, consumer {}",
        s.req.0, s.req.1, s.resp.0, s.resp.1
    );
}
