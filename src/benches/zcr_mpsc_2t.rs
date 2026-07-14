//! Two-threaded zc-ring-x1 MPSC round-trip bench, closure
//! (`send_with`) API, spin waits.

use std::hint::black_box;
use std::thread;

use zc_ring_x1::{MpscConsumer, MpscProducer};

use crate::benches::zcr_common::{Msg, STOP, leak_mpsc_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::pin;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-mpsc-2t";

/// Main → worker → main round-trip over two zc-ring-x1 MPSC
/// rings — one producer per ring, so this is the "MPSC when
/// you don't need it" number against `zcr-with-2t`'s SPSC
/// pair at the same placement.
///
/// - Wait policy: a `spin_loop` hint per failed attempt on
///   both the send and receive sides.
/// - Shutdown: `Drop` sends the [`STOP`] sentinel; the worker
///   exits on receipt without replying.
pub struct ZcrMpsc2Thread {
    req_tx: MpscProducer<'static>,
    resp_rx: MpscConsumer<'static>,
    worker: Option<thread::JoinHandle<()>>,
    counter: u64,
}

impl ZcrMpsc2Thread {
    /// Spawn the spinning echo worker over two fresh leaked
    /// MPSC rings, optionally pinning it to `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_tx, mut req_rx) = leak_mpsc_ring();
        let (resp_tx, resp_rx) = leak_mpsc_ring();
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
                resp_tx
                    .send_with::<Msg>(
                        |_| {
                            core::hint::spin_loop();
                            true
                        },
                        |m| *m = v,
                    )
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

impl Bench for ZcrMpsc2Thread {
    fn name(&self) -> &str {
        "zcr-mpsc-2t: zc-ring-x1 mpsc send_with round-trip (2 threads, spin)"
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
            .expect("spin closure never gives up");
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

impl Drop for ZcrMpsc2Thread {
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
            .expect("spin closure never gives up");
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = ZcrMpsc2Thread::new(cfg.core_for(1));
    let (hist, outer, inner, duration_s, suspended_s, block_stats) =
        harness::run_adaptive(&mut bench, cfg);
    harness::print_report(
        bench.name(),
        outer,
        inner,
        duration_s,
        &hist,
        cfg,
        suspended_s,
        block_stats.as_ref(),
    );
}
