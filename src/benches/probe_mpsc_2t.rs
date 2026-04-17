//! Probed two-threaded `std::sync::mpsc` round-trip bench.
//!
//! Structurally mirrors [`mpsc_2t`][super::mpsc_2t] and adds one
//! [`Probe`] per thread, timing each `channel.send(...)` call.
//! Run back-to-back with `mpsc-2t` to quantify per-probe overhead
//! (see the 0.8.0-dev1 plan section in `notes/chores-02.md`).

use std::hint::black_box;
use std::mem;
use std::sync::mpsc;
use std::thread;

use crate::harness::{self, Bench, RunCfg};
use crate::pin;
use crate::probe::Probe;

/// Registry name used on the CLI.
pub const NAME: &str = "probe-mpsc-2t";

/// Probed variant of `mpsc-2t`. Each thread owns its own
/// [`Probe`]; the worker's probe travels back on shutdown via
/// `JoinHandle<Probe>`. Call [`finish`][Self::finish] once after
/// the benchmark loop to drain both probes and join the worker.
pub struct ProbedStdMpsc2Thread {
    req_tx: mpsc::Sender<u64>,
    resp_rx: mpsc::Receiver<u64>,
    worker: Option<thread::JoinHandle<Probe>>,
    main_probe: Probe,
    counter: u64,
}

impl ProbedStdMpsc2Thread {
    /// Spawn the echo worker, optionally pinning it to `worker_cpu`.
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_tx, req_rx) = mpsc::channel::<u64>();
        let (resp_tx, resp_rx) = mpsc::channel::<u64>();
        let worker = thread::spawn(move || {
            pin::pin_current(worker_cpu);
            let mut worker_probe = Probe::new("worker send");
            while let Ok(v) = req_rx.recv() {
                let s = minstant::Instant::now();
                if resp_tx.send(v).is_err() {
                    break;
                }
                worker_probe.record(s.elapsed().as_nanos() as u64);
            }
            worker_probe
        });
        Self {
            req_tx,
            resp_rx,
            worker: Some(worker),
            main_probe: Probe::new("main send"),
            counter: 0,
        }
    }

    /// Shut down the worker and return the two probes. Must be
    /// called exactly once, after the benchmark loop; `step()`
    /// will panic on subsequent calls because the sender is gone.
    pub fn finish(&mut self) -> (Probe, Probe) {
        let (dummy_tx, _) = mpsc::channel();
        drop(mem::replace(&mut self.req_tx, dummy_tx));
        let worker_probe = self
            .worker
            .take()
            .expect("finish called twice")
            .join()
            .expect("worker panicked");
        let main_probe = mem::replace(&mut self.main_probe, Probe::new(""));
        (main_probe, worker_probe)
    }
}

impl Bench for ProbedStdMpsc2Thread {
    fn name(&self) -> &str {
        "std::sync::mpsc round-trip (2 threads, probed)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        let s = minstant::Instant::now();
        self.req_tx.send(self.counter).unwrap();
        self.main_probe.record(s.elapsed().as_nanos() as u64);
        let v = self.resp_rx.recv().unwrap();
        black_box(v)
    }
}

impl Drop for ProbedStdMpsc2Thread {
    fn drop(&mut self) {
        // Panic-path safety net: if run() didn't reach finish(),
        // tear down the worker the same way mpsc_2t does.
        let (dummy_tx, _) = mpsc::channel();
        drop(mem::replace(&mut self.req_tx, dummy_tx));
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = ProbedStdMpsc2Thread::new(cfg.core_for(1));
    let (hist, outer, inner, duration_s) = harness::run_adaptive(&mut bench, cfg);
    let (main_probe, worker_probe) = bench.finish();
    harness::print_report(bench.name(), outer, inner, duration_s, &hist, cfg.overhead);
    main_probe.report();
    worker_probe.report();
}
