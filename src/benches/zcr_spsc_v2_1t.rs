//! Single-threaded zc-ring-x1 spsc v2 round-trip bench, closure
//! (`reserve_slot_with`) API, the in-slot seq ring.

use std::hint::black_box;

use zc_ring_x1::spsc::v2::{Consumer, Producer};

use crate::benches::zcr_common::{Msg, leak_v2_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-spsc-v2-1t";

/// Same-thread round-trip through the v2 ring's
/// `reserve_slot_with` on both ends, the shape of `zcr-spsc-v1-1t`
/// over the in-slot seq protocol.
///
/// - The wait closures never run here (one message in flight,
///   never full or empty), so the measurement is v2's
///   uncontended fast path: v1's seq load and seq store per end,
///   with the seq word now in the slot's own line rather than a
///   seq array, so each end touches one line per message.
pub struct ZcrSpscV2OneThread {
    producer: Producer<'static>,
    consumer: Consumer<'static>,
    counter: u64,
}

impl ZcrSpscV2OneThread {
    /// Construct the bench over one fresh leaked v2 ring.
    pub fn new() -> Self {
        let (producer, consumer) = leak_v2_ring();
        Self {
            producer,
            consumer,
            counter: 0,
        }
    }
}

impl Bench for ZcrSpscV2OneThread {
    fn name(&self) -> &str {
        "zcr-spsc-v2-1t: zc-ring-x1 spsc v2 reserve_slot_with round-trip (1 thread)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        let mut slot = self
            .producer
            .reserve_slot_with::<Msg>(|_| {
                core::hint::spin_loop();
                true
            })
            // OK: the closure returns true forever, so the reserve
            // never gives up and the Err arm is unreachable.
            .expect("spin closure never gives up");
        *slot = self.counter;
        slot.commit();
        let slot = self
            .consumer
            .reserve_slot_with::<Msg>(|_| {
                core::hint::spin_loop();
                true
            })
            // OK: as above, the closure never gives up.
            .expect("spin closure never gives up");
        let v = *slot;
        slot.release();
        black_box(v)
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = ZcrSpscV2OneThread::new();
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
}
