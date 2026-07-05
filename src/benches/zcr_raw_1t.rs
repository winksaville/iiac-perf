//! Single-threaded zc-ring-x1 round-trip bench, raw API tier.

use std::hint::black_box;

use zc_ring_x1::{Consumer, Producer};

use crate::benches::zcr_common::{Msg, leak_ring};
use crate::harness::{self, Bench, RunCfg};

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-raw-1t";

/// Same-thread reserve/commit then reserve/release through one
/// zc-ring-x1 ring, using the raw `reserve_slot` tier.
///
/// - The raw tier surfaces `Full`/`Empty` to the caller; neither
///   occurs here — exactly one message is in flight against a
///   capacity-8 ring — so this measures pure ring overhead with
///   no wait loop and no cross-core traffic.
/// - Baseline for `zcr-with-1t` / `zcr-spin-1t`: any delta
///   between the three is pure API-tier cost.
pub struct ZcrRaw1Thread {
    producer: Producer<'static>,
    consumer: Consumer<'static>,
    counter: u64,
}

impl ZcrRaw1Thread {
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

impl Bench for ZcrRaw1Thread {
    fn name(&self) -> &str {
        "zcr-raw-1t: zc-ring-x1 raw round-trip (1 thread)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        let mut slot = self
            .producer
            .reserve_slot::<Msg>()
            .expect("never full: one message in flight");
        *slot = self.counter;
        slot.commit();
        let slot = self
            .consumer
            .reserve_slot::<Msg>()
            .expect("never empty: message just committed");
        let v = *slot;
        slot.release();
        black_box(v)
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = ZcrRaw1Thread::new();
    let (hist, outer, inner, duration_s) = harness::run_adaptive(&mut bench, cfg);
    harness::print_report(bench.name(), outer, inner, duration_s, &hist, cfg.overhead);
}
