use std::hint::black_box;
use std::sync::mpsc;
use std::thread;

use crate::harness::{self, Bench, RunCfg};
use crate::pin;

pub const NAME: &str = "mpsc-2t";

pub struct StdMpsc2Thread {
    req_tx: mpsc::Sender<u64>,
    resp_rx: mpsc::Receiver<u64>,
    worker: Option<thread::JoinHandle<()>>,
    counter: u64,
}

impl StdMpsc2Thread {
    pub fn new(worker_cpu: Option<usize>) -> Self {
        let (req_tx, req_rx) = mpsc::channel::<u64>();
        let (resp_tx, resp_rx) = mpsc::channel::<u64>();
        let worker = thread::spawn(move || {
            pin::pin_current(worker_cpu);
            while let Ok(v) = req_rx.recv() {
                if resp_tx.send(v).is_err() {
                    break;
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

impl Bench for StdMpsc2Thread {
    fn name(&self) -> &str {
        "std::sync::mpsc round-trip (2 threads)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        self.req_tx.send(self.counter).unwrap();
        let v = self.resp_rx.recv().unwrap();
        black_box(v)
    }
}

impl Drop for StdMpsc2Thread {
    fn drop(&mut self) {
        // Replace req_tx with a dummy so we can drop the real one;
        // worker's recv() then returns Err and the worker exits.
        // Done this way (instead of Option<Sender>) to keep step() branch-free.
        let (dummy_tx, _) = mpsc::channel();
        drop(std::mem::replace(&mut self.req_tx, dummy_tx));
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

pub fn run(cfg: &RunCfg) {
    let mut bench = StdMpsc2Thread::new(cfg.core_for(1));
    let (hist, outer, inner, duration_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_histogram(bench.name(), outer, inner, duration_s, &hist, cfg.overhead);
}
