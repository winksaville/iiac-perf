//! Single-threaded zc-ring-x1 MPSC round-trip bench, closure
//! (`send_with`) API.

use std::hint::black_box;

use zc_ring_x1::{MpscConsumer, MpscProducer};

use crate::benches::zcr_common::{Msg, leak_mpsc_ring};
use crate::harness::{self, Bench, RunCfg};

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-mpsc-1t";

/// Same-thread round-trip sending through the MPSC ring's
/// `send_with` and receiving through its consumer guard.
///
/// - The wait closures never run here (one message in flight,
///   never full/empty), so the measurement is the MPSC
///   protocol's uncontended fast path — one claim CAS plus the
///   per-slot seq publish, against `zcr-with-1t`'s
///   load/store-only SPSC pair.
pub struct ZcrMpsc1Thread {
    producer: MpscProducer<'static>,
    consumer: MpscConsumer<'static>,
    counter: u64,
}

impl ZcrMpsc1Thread {
    /// Construct the bench over one fresh leaked MPSC ring.
    pub fn new() -> Self {
        let (producer, consumer) = leak_mpsc_ring();
        Self {
            producer,
            consumer,
            counter: 0,
        }
    }
}

impl Bench for ZcrMpsc1Thread {
    fn name(&self) -> &str {
        "zcr-mpsc-1t: zc-ring-x1 mpsc send_with round-trip (1 thread)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        let c = self.counter;
        self.producer
            .send_with::<Msg>(
                |_| {
                    core::hint::spin_loop();
                    true
                },
                |m| *m = c,
            )
            .expect("spin closure never gives up");
        let slot = self
            .consumer
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

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = ZcrMpsc1Thread::new();
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
