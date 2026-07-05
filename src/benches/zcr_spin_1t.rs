//! Single-threaded zc-ring-x1 round-trip bench, easy
//! (`reserve_slot_spin`) API tier.

use std::hint::black_box;

use zc_ring_x1::{Consumer, Producer};

use crate::benches::zcr_common::{Msg, leak_ring};
use crate::harness::{self, Bench, RunCfg};

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-spin-1t";

/// Same-thread round-trip as `zcr-raw-1t`, but reserving through
/// `reserve_slot_spin` — the easy tier with the spin policy baked
/// in and no `Result` to handle.
///
/// - The spin never engages here (one message in flight, never
///   full/empty), so any delta vs `zcr-raw-1t` / `zcr-with-1t`
///   is the cost of the built-in-policy wrapper's fast path.
pub struct ZcrSpin1Thread {
    producer: Producer<'static>,
    consumer: Consumer<'static>,
    counter: u64,
}

impl ZcrSpin1Thread {
    /// Construct the bench over one fresh leaked ring.
    pub fn new() -> Self {
        let (producer, consumer) = leak_ring();
        Self {
            producer,
            consumer,
            counter: 0,
        }
    }
}

impl Bench for ZcrSpin1Thread {
    fn name(&self) -> &str {
        "zcr-spin-1t: zc-ring-x1 reserve_slot_spin round-trip (1 thread)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        let mut slot = self.producer.reserve_slot_spin::<Msg>();
        *slot = self.counter;
        slot.commit();
        let slot = self.consumer.reserve_slot_spin::<Msg>();
        let v = *slot;
        slot.release();
        black_box(v)
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = ZcrSpin1Thread::new();
    let (hist, outer, inner, duration_s, suspended_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_report(
        bench.name(),
        outer,
        inner,
        duration_s,
        &hist,
        cfg.overhead,
        suspended_s,
    );
}
