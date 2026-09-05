//! Single-threaded zc-ring-x1 spsc v1 round-trip bench, closure
//! (`reserve_slot_with`) API, the seam-word ring.

use std::hint::black_box;

use zc_ring_x1::spsc::v1::{Consumer, Producer};

use crate::benches::zcr_common::{Msg, leak_v1_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-v1-1t";

/// Same-thread round-trip through the v1 ring's
/// `reserve_slot_with` on both ends, the shape of `zcr-with-1t`
/// over the seam-word protocol.
///
/// - The wait closures never run here (one message in flight,
///   never full or empty), so the measurement is v1's
///   uncontended fast path: a seq load and a seq store per end,
///   with each end's index line private to it, against
///   `zcr-with-1t`'s v0 pair that reads the other end's index.
pub struct ZcrV1OneThread {
    producer: Producer<'static>,
    consumer: Consumer<'static>,
    counter: u64,
}

impl ZcrV1OneThread {
    /// Construct the bench over one fresh leaked v1 ring.
    pub fn new() -> Self {
        let (producer, consumer) = leak_v1_ring();
        Self {
            producer,
            consumer,
            counter: 0,
        }
    }
}

impl Bench for ZcrV1OneThread {
    fn name(&self) -> &str {
        "zcr-v1-1t: zc-ring-x1 spsc v1 reserve_slot_with round-trip (1 thread)"
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
    let mut bench = ZcrV1OneThread::new();
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
}
