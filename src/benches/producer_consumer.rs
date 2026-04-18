//! Free-form probe-only bench: a dedicated producer thread and a
//! dedicated consumer thread trade messages over two
//! `std::sync::mpsc` channels. Each actor measures its own full
//! loop iteration (send+recv on the producer, recv+send on the
//! consumer) via a single [`Probe`]. No outer `Bench`-trait
//! histogram — the application drives itself and probes are the
//! only measurement channel.
//!
//! Written as a UX experiment in the probe-only style: main
//! orchestrates, producer produces, consumer consumes. Compare
//! with `probe-mpsc-2t`, which embeds probes inside the
//! step-driven `Bench` trait.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

use crate::harness::RunCfg;
use crate::pin;
use crate::probe::Probe;

/// Registry name used on the CLI.
pub const NAME: &str = "producer-consumer";

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
        let mut probe = Probe::new("producer loop");
        let mut counter: u64 = 0;
        while !producer_shutdown.load(Ordering::Relaxed) {
            let s = minstant::Instant::now();
            counter = counter.wrapping_add(1);
            if req_tx.send(counter).is_err() {
                break;
            }
            if resp_rx.recv().is_err() {
                break;
            }
            probe.record(s.elapsed().as_nanos() as u64);
        }
        probe
    });

    let consumer = thread::spawn(move || {
        pin::pin_current(consumer_cpu);
        let mut probe = Probe::new("consumer loop");
        loop {
            let s = minstant::Instant::now();
            let v = match req_rx.recv() {
                Ok(v) => v,
                Err(_) => break,
            };
            if resp_tx.send(v).is_err() {
                break;
            }
            probe.record(s.elapsed().as_nanos() as u64);
        }
        probe
    });

    thread::sleep(Duration::from_secs_f64(cfg.target_seconds));
    shutdown.store(true, Ordering::Relaxed);

    let producer_probe = producer.join().expect("producer panicked");
    let consumer_probe = consumer.join().expect("consumer panicked");

    println!(
        "producer-consumer (2 threads, probe-only) [duration={:.1}s]:",
        cfg.target_seconds
    );
    producer_probe.report();
    consumer_probe.report();
}
