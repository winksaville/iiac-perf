//! Two-threaded `std::sync::mpsc` round-trip bench, spin waits.

use std::hint::black_box;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;

use crate::harness::{self, Bench, RunCfg};
use crate::pin;

/// Registry name used on the CLI.
pub const NAME: &str = "mpsc-2t-spin";

/// Main → worker → main round-trip over two `std::sync::mpsc`
/// channels where both ends spin on `try_recv` instead of parking
/// in `recv`. Same wait policy as the `ice-*-2t` benches
/// (non-blocking receive + `spin_loop`), so `mpsc-2t-spin` vs
/// `ice-ps-2t` is a pure transport comparison — in-process channel
/// queue vs shared-memory queue — while `mpsc-2t` vs this bench
/// isolates the park/wake cost of blocking `recv`.
pub struct StdMpsc2ThreadSpin {
    req_tx: mpsc::Sender<u64>,
    resp_rx: mpsc::Receiver<u64>,
    worker: Option<thread::JoinHandle<()>>,
    counter: u64,
}

impl StdMpsc2ThreadSpin {
    /// Spawn the spinning echo worker, optionally pinning it to
    /// `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_tx, req_rx) = mpsc::channel::<u64>();
        let (resp_tx, resp_rx) = mpsc::channel::<u64>();
        let worker = thread::spawn(move || {
            pin::pin_current(worker_cpu);
            loop {
                match req_rx.try_recv() {
                    Ok(v) => {
                        if resp_tx.send(v).is_err() {
                            break;
                        }
                    }
                    Err(TryRecvError::Empty) => core::hint::spin_loop(),
                    Err(TryRecvError::Disconnected) => break,
                }
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

impl Bench for StdMpsc2ThreadSpin {
    fn name(&self) -> &str {
        "mpsc-2t-spin: std::sync::mpsc round-trip (2 threads, spin)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        self.req_tx.send(self.counter).unwrap();
        loop {
            match self.resp_rx.try_recv() {
                Ok(v) => return black_box(v),
                Err(TryRecvError::Empty) => core::hint::spin_loop(),
                Err(TryRecvError::Disconnected) => panic!("worker gone"),
            }
        }
    }
}

impl Drop for StdMpsc2ThreadSpin {
    fn drop(&mut self) {
        // Replace req_tx with a dummy so we can drop the real one;
        // worker's try_recv() then returns Disconnected and the
        // worker exits. Same shape as mpsc-2t's Drop.
        let (dummy_tx, _) = mpsc::channel();
        drop(std::mem::replace(&mut self.req_tx, dummy_tx));
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = StdMpsc2ThreadSpin::new(cfg.core_for(1));
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
