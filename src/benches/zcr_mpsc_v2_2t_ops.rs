//! Known-cost variants of `zcr-mpsc-v2-2t`: its round trip with one
//! operation added, a positive control for `analyze --compare`.
//!
//! - `-nop` adds a call that does nothing, which moves the bench to
//!   other code and costs a call.
//! - `-store-seqcst` adds a SeqCst store, a full barrier, `xchg` on
//!   x86.
//!
//! A copy of [`super::zcr_mpsc_v2_2t`] rather than a change to it, so
//! the base stays the bench every record of it measured, and each
//! variant compares against it unchanged.

use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use zc_ring_x1::mpsc::v2::{MpscConsumer, MpscProducer};

use crate::benches::zcr_common::{Msg, STOP, leak_mpsc_v2_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::pin;
use crate::record;
use crate::report;

/// The variant that adds a call doing nothing to each round trip.
pub const NAME_NOP: &str = "zcr-mpsc-v2-2t-nop";

/// The variant that adds a SeqCst store to each round trip.
pub const NAME_STORE_SEQCST: &str = "zcr-mpsc-v2-2t-store-seqcst";

/// The extra operation per round trip, a const generic so each variant is its own code: a call
/// to [`nop`].
const NOP: u8 = 1;
/// A call to [`store_seqcst`].
const STORE_SEQCST: u8 = 2;

/// A cell on a cache line of its own, which only the main thread touches, so the variants'
/// op is uncontended and measures the instruction, not coherence traffic.
#[repr(align(128))]
struct Line(AtomicUsize);

/// The `-nop` variant's op: nothing, kept a real call by `inline(never)` and its arguments kept
/// live by `black_box`, so the variant pays the call and no more.
#[inline(never)]
fn nop(cell: &AtomicUsize, v: u64) {
    black_box((cell, v));
}

/// The `-store-seqcst` variant's op: a SeqCst store, a full barrier, `xchg` on x86, kept a call
/// of its own by `inline(never)` so its cost is the store and the same call `-nop` pays.
#[inline(never)]
fn store_seqcst(cell: &AtomicUsize, v: u64) {
    cell.store(v as usize, Ordering::SeqCst);
}

/// Segment switches of the four ends of the two rings.
pub struct Switches {
    /// The request ring, producer then consumer.
    pub req: (u64, u64),
    /// The response ring, producer then consumer.
    pub resp: (u64, u64),
}

/// `zcr-mpsc-v2-2t`'s round trip with the operation `EXTRA` added
/// on the main thread while the request is in flight: main to
/// worker to main over two zc-ring-x1 v2 MPSC rings, one producer
/// per ring.
///
/// - Wait policy: a `spin_loop` hint per failed attempt on both
///   the send and receive sides.
/// - Switches: one message in flight means the consumer keeps up
///   and neither ring leaves its first segment. The worker hands
///   its two ends' counts back at shutdown, and the four are
///   printed after the report, expected zero.
/// - Shutdown: `Drop` sends the [`STOP`] sentinel, and the worker
///   exits on receipt without replying.
pub struct ZcrMpscV2Ops<const EXTRA: u8> {
    req_tx: MpscProducer<'static>,
    resp_rx: MpscConsumer<'static>,
    worker: Option<thread::JoinHandle<(u64, u64)>>,
    counter: u64,
    /// The variants' cell.
    cell: Box<Line>,
}

impl<const EXTRA: u8> ZcrMpscV2Ops<EXTRA> {
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
            cell: Box::new(Line(AtomicUsize::new(0))),
        }
    }
}

impl<const EXTRA: u8> Bench for ZcrMpscV2Ops<EXTRA> {
    fn name(&self) -> &str {
        match EXTRA {
            NOP => {
                "zcr-mpsc-v2-2t-nop: zc-ring-x1 mpsc v2 send_with round-trip (2 threads, spin), plus a call that does nothing"
            }
            STORE_SEQCST => {
                "zcr-mpsc-v2-2t-store-seqcst: zc-ring-x1 mpsc v2 send_with round-trip (2 threads, spin), plus a SeqCst store"
            }
            _ => "zcr-mpsc-v2-2t-ops: an unregistered variant",
        }
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
        // The variants' op, while the request is in flight. A constant match, so the base
        // compiles to none of it.
        match EXTRA {
            NOP => nop(black_box(&self.cell.0), c),
            STORE_SEQCST => store_seqcst(black_box(&self.cell.0), c),
            _ => {}
        }
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

impl<const EXTRA: u8> ZcrMpscV2Ops<EXTRA> {
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

impl<const EXTRA: u8> Drop for ZcrMpscV2Ops<EXTRA> {
    /// Stop the worker if [`shutdown`](Self::shutdown) has not.
    fn drop(&mut self) {
        if self.worker.is_some() {
            self.shutdown();
        }
    }
}

/// Registry entry point of `-nop`.
pub fn run_nop(cfg: &RunCfg) {
    run_as::<NOP>(NAME_NOP, cfg);
}

/// Registry entry point of `-store-seqcst`.
pub fn run_store_seqcst(cfg: &RunCfg) {
    run_as::<STORE_SEQCST>(NAME_STORE_SEQCST, cfg);
}

/// Run the variant `EXTRA`, recorded as `name`.
fn run_as<const EXTRA: u8>(name: &str, cfg: &RunCfg) {
    let mut bench = ZcrMpscV2Ops::<EXTRA>::new(cfg.cpu_for(1));
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(name, &out, cfg);
    let s = bench.shutdown();
    println!(
        "segment switches: request ring producer {}, consumer {}; response ring producer {}, consumer {}",
        s.req.0, s.req.1, s.resp.0, s.resp.1
    );
}
