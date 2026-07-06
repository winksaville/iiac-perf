//! Two-threaded zc-ring-x1 round-trip bench, raw API tier,
//! spin waits.

use std::hint::black_box;
use std::thread;

use zc_ring_x1::{Consumer, Empty, Full, Producer};

use crate::benches::zcr_common::{Msg, STOP, leak_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::pin;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-raw-2t";

/// Main → worker → main round-trip over two zc-ring-x1 rings
/// (req + resp), both ends using the raw `reserve_slot` tier
/// with hand-written spin loops on `Full`/`Empty`.
///
/// - Same wait policy as `mpsc-2t-spin` / `ice-*-2t`
///   (non-blocking attempt + `spin_loop`), so zcr-raw-2t vs
///   those is a pure transport comparison, while zcr-raw-2t vs
///   `zcr-with-2t` / `zcr-spin-2t` isolates API-tier cost under
///   real cross-core traffic.
/// - Shutdown: `Drop` sends the [`STOP`] sentinel; the worker
///   exits on receipt without replying.
pub struct ZcrRaw2Thread {
    req_tx: Producer<'static>,
    resp_rx: Consumer<'static>,
    worker: Option<thread::JoinHandle<()>>,
    counter: u64,
}

impl ZcrRaw2Thread {
    /// Spawn the spinning echo worker over two fresh leaked
    /// rings, optionally pinning it to `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_tx, mut req_rx) = leak_ring();
        let (mut resp_tx, resp_rx) = leak_ring();
        let worker = thread::spawn(move || {
            pin::pin_current(worker_cpu);
            loop {
                let v = loop {
                    match req_rx.reserve_slot::<Msg>() {
                        Ok(slot) => {
                            let v = *slot;
                            slot.release();
                            break v;
                        }
                        Err(Empty) => core::hint::spin_loop(),
                    }
                };
                if v == STOP {
                    break;
                }
                let mut slot = loop {
                    match resp_tx.reserve_slot::<Msg>() {
                        Ok(slot) => break slot,
                        Err(Full) => core::hint::spin_loop(),
                    }
                };
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

impl Bench for ZcrRaw2Thread {
    fn name(&self) -> &str {
        "zcr-raw-2t: zc-ring-x1 raw round-trip (2 threads, spin)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        if self.counter == STOP {
            self.counter = 1;
        }
        let mut slot = self
            .req_tx
            .reserve_slot::<Msg>()
            .expect("never full: one message in flight");
        *slot = self.counter;
        slot.commit();
        loop {
            match self.resp_rx.reserve_slot::<Msg>() {
                Ok(slot) => {
                    let v = *slot;
                    slot.release();
                    return black_box(v);
                }
                Err(Empty) => core::hint::spin_loop(),
            }
        }
    }
}

impl Drop for ZcrRaw2Thread {
    /// Send [`STOP`] and join the worker.
    fn drop(&mut self) {
        let mut slot = self
            .req_tx
            .reserve_slot::<Msg>()
            .expect("never full: nothing in flight at drop");
        *slot = STOP;
        slot.commit();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = ZcrRaw2Thread::new(cfg.core_for(1));
    let (hist, outer, inner, duration_s, suspended_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_report(
        bench.name(),
        outer,
        inner,
        duration_s,
        &hist,
        cfg,
        suspended_s,
    );
}
