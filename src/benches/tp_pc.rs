//! TProbe variant of `producer-consumer`: a dedicated producer
//! thread and a dedicated consumer thread trade messages over
//! two `std::sync::mpsc` channels. Each actor measures its own
//! full loop iteration via [`TProbe::start`] / [`TProbe::end`],
//! which read hardware ticks directly and defer delta / histogram
//! ingestion to `report()` — skipping the tick→ns conversion on
//! the hot path.
//!
//! Run back-to-back with `producer-consumer` to see whether
//! dropping the tick→ns conversion trims the per-sample framing.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

use crate::harness::RunCfg;
use crate::pin;
use crate::tprobe::TProbe;

/// Registry name used on the CLI.
pub const NAME: &str = "tp-pc";

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let (req_tx, req_rx) = mpsc::channel::<u64>();
    let (resp_tx, resp_rx) = mpsc::channel::<u64>();
    let shutdown = Arc::new(AtomicBool::new(false));

    let producer_cpu = cfg.core_for(0);
    let consumer_cpu = cfg.core_for(1);

    let producer_shutdown = shutdown.clone();
    let producer = thread::spawn(move || {
        pin::pin_current(producer_cpu);
        let mut probe = TProbe::new("producer loop");
        let mut counter: u64 = 0;
        while !producer_shutdown.load(Ordering::Relaxed) {
            let id = probe.start(0);
            counter = counter.wrapping_add(1);
            if req_tx.send(counter).is_err() {
                break;
            }
            if resp_rx.recv().is_err() {
                break;
            }
            probe.end(id);
        }
        probe
    });

    let consumer = thread::spawn(move || {
        pin::pin_current(consumer_cpu);
        let mut probe = TProbe::new("consumer loop");
        loop {
            let id = probe.start(0);
            let v = match req_rx.recv() {
                Ok(v) => v,
                Err(_) => break,
            };
            if resp_tx.send(v).is_err() {
                break;
            }
            probe.end(id);
        }
        probe
    });

    thread::sleep(Duration::from_secs_f64(cfg.target_seconds));
    shutdown.store(true, Ordering::Relaxed);

    let mut producer_probe = producer.join().expect("producer panicked");
    let mut consumer_probe = consumer.join().expect("consumer panicked");

    println!(
        "tp-pc (2 threads, TProbe tick-only) [duration={:.1}s]:",
        cfg.target_seconds
    );
    producer_probe.report(cfg.report_ticks);
    consumer_probe.report(cfg.report_ticks);
}
