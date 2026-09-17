//! Two-threaded zc-ring-x1 mpsc v2 round-trip bench, closure
//! (`send_with`) API, spin waits, the segmented ring over a pool.

use std::hint::black_box;
use std::thread;

use zc_ring_x1::mpsc::v2::{MpscConsumer, MpscProducer};

use crate::benches::zcr_common::{Msg, STOP, leak_mpsc_v2_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::pin;
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-mpsc-v2-2t";

/// Segment switches of the four ends of the two rings.
pub struct Switches {
    /// The request ring, producer then consumer.
    pub req: (u64, u64),
    /// The response ring, producer then consumer.
    pub resp: (u64, u64),
}

/// Main to worker to main round-trip over two zc-ring-x1 v2 MPSC
/// rings, one producer per ring, the shape of `zcr-mpsc-v1-2t`
/// over the segmented ring, so the pair at one placement is the
/// cross-core handoff of the two rings side by side.
///
/// - Wait policy: a `spin_loop` hint per failed attempt on both
///   the send and receive sides.
/// - Switches: one message in flight means the consumer keeps up
///   and neither ring leaves its first segment. The worker hands
///   its two ends' counts back at shutdown, and the four are
///   printed after the report, expected zero.
/// - Shutdown: `Drop` sends the [`STOP`] sentinel, and the worker
///   exits on receipt without replying.
pub struct ZcrMpscV2TwoThread {
    req_tx: MpscProducer<'static>,
    resp_rx: MpscConsumer<'static>,
    worker: Option<thread::JoinHandle<(u64, u64)>>,
    counter: u64,
}

impl ZcrMpscV2TwoThread {
    /// Spawn the spinning echo worker over two fresh leaked v1
    /// MPSC rings, optionally pinning it to `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_tx, mut req_rx) = leak_mpsc_v2_ring();
        let (resp_tx, resp_rx) = leak_mpsc_v2_ring();
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
            (req_rx.switches(), resp_tx.switches())
        });
        Self {
            req_tx,
            resp_rx,
            worker: Some(worker),
            counter: 0,
        }
    }
}

impl Bench for ZcrMpscV2TwoThread {
    fn name(&self) -> &str {
        "zcr-mpsc-v2-2t: zc-ring-x1 mpsc v2 send_with round-trip (2 threads, spin)"
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

impl ZcrMpscV2TwoThread {
    /// Send [`STOP`], join the worker, and return the four ends'
    /// switch counts. A second call returns zeros for the worker's
    /// ends, since it has gone.
    pub fn shutdown(&mut self) -> Switches {
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

impl Drop for ZcrMpscV2TwoThread {
    /// Stop the worker if [`shutdown`](Self::shutdown) has not.
    fn drop(&mut self) {
        if self.worker.is_some() {
            self.shutdown();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = ZcrMpscV2TwoThread::new(cfg.cpu_for(1));
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
    let s = bench.shutdown();
    println!(
        "segment switches: request ring producer {}, consumer {}; response ring producer {}, consumer {}",
        s.req.0, s.req.1, s.resp.0, s.resp.1
    );
}
