//! Single-threaded zc-ring-x1 spsc v0 round-trip bench, closure
//! (`reserve_slot_with`) API, the index-line ring.

use std::hint::black_box;

use zc_ring_x1::{Consumer, Producer};

use crate::benches::zcr_common::{Msg, leak_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-spsc-v0-1t";

/// Same-thread round-trip reserving through `reserve_slot_with`
/// with an app-supplied spin closure.
///
/// - The closure never runs here (one message in flight, never
///   full or empty), so the measurement is the cost of the `_with`
///   wrapper's fast path, a single claim with no contention.
pub struct ZcrSpscV0OneThread {
    producer: Producer<'static>,
    consumer: Consumer<'static>,
    counter: u64,
}

impl ZcrSpscV0OneThread {
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

impl Bench for ZcrSpscV0OneThread {
    fn name(&self) -> &str {
        "zcr-spsc-v0-1t: zc-ring-x1 spsc v0 reserve_slot_with round-trip (1 thread)"
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
    let mut bench = ZcrSpscV0OneThread::new();
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
}
