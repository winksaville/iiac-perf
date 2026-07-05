//! Two-threaded zc-ring-x1 round-trip bench, easy
//! (`reserve_slot_spin`) API tier.

use std::hint::black_box;
use std::thread;

use zc_ring_x1::{Consumer, Producer};

use crate::benches::zcr_common::{Msg, STOP, leak_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::pin;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-spin-2t";

/// Main → worker → main round-trip over two zc-ring-x1 rings as
/// `zcr-raw-2t`, but both ends wait inside `reserve_slot_spin` —
/// the easy tier with the spin policy baked in.
///
/// - Same effective wait policy as `zcr-raw-2t` / `zcr-with-2t`
///   (spin_loop hint per failed attempt), so any delta between
///   the three is pure API-tier cost under cross-core traffic.
/// - Shutdown: `Drop` sends the [`STOP`] sentinel; the worker
///   exits on receipt without replying. `reserve_slot_spin`
///   never gives up, so the sentinel (not a closure returning
///   `false`) is the only way to unblock a spinning consumer.
pub struct ZcrSpin2Thread {
    req_tx: Producer<'static>,
    resp_rx: Consumer<'static>,
    worker: Option<thread::JoinHandle<()>>,
    counter: u64,
}

impl ZcrSpin2Thread {
    /// Spawn the spinning echo worker over two fresh leaked
    /// rings, optionally pinning it to `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_tx, mut req_rx) = leak_ring();
        let (mut resp_tx, resp_rx) = leak_ring();
        let worker = thread::spawn(move || {
            pin::pin_current(worker_cpu);
            loop {
                let slot = req_rx.reserve_slot_spin::<Msg>();
                let v = *slot;
                slot.release();
                if v == STOP {
                    break;
                }
                let mut slot = resp_tx.reserve_slot_spin::<Msg>();
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

impl Bench for ZcrSpin2Thread {
    fn name(&self) -> &str {
        "zcr-spin-2t: zc-ring-x1 reserve_slot_spin round-trip (2 threads, spin)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        if self.counter == STOP {
            self.counter = 1;
        }
        let mut slot = self.req_tx.reserve_slot_spin::<Msg>();
        *slot = self.counter;
        slot.commit();
        let slot = self.resp_rx.reserve_slot_spin::<Msg>();
        let v = *slot;
        slot.release();
        black_box(v)
    }
}

impl Drop for ZcrSpin2Thread {
    /// Send [`STOP`] and join the worker.
    fn drop(&mut self) {
        let mut slot = self.req_tx.reserve_slot_spin::<Msg>();
        *slot = STOP;
        slot.commit();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = ZcrSpin2Thread::new(cfg.core_for(1));
    let (hist, outer, inner, duration_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_report(bench.name(), outer, inner, duration_s, &hist, cfg.overhead);
}
