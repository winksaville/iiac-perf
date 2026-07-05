//! Single-threaded zc-ring-x1 round-trip bench, closure
//! (`reserve_slot_with`) API tier.

use std::hint::black_box;

use zc_ring_x1::{Consumer, Producer};

use crate::benches::zcr_common::{Msg, leak_ring};
use crate::harness::{self, Bench, RunCfg};

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-with-1t";

/// Same-thread round-trip as `zcr-raw-1t`, but reserving through
/// `reserve_slot_with` with an app-supplied spin closure.
///
/// - The closure never runs here (one message in flight, never
///   full/empty), so any delta vs `zcr-raw-1t` is the cost of
///   the `_with` wrapper's fast path — which zc-ring-x1's docs
///   claim does exactly the loads `reserve_slot` does.
pub struct ZcrWith1Thread {
    producer: Producer<'static>,
    consumer: Consumer<'static>,
    counter: u64,
}

impl ZcrWith1Thread {
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

impl Bench for ZcrWith1Thread {
    fn name(&self) -> &str {
        "zcr-with-1t: zc-ring-x1 reserve_slot_with round-trip (1 thread)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        let mut slot = self
            .producer
            .reserve_slot_with::<Msg>(|_| {
                core::hint::spin_loop();
                true
            })
            .expect("spin closure never gives up");
        *slot = self.counter;
        slot.commit();
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
    let mut bench = ZcrWith1Thread::new();
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
