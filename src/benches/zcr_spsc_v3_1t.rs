//! Single-threaded zc-ring-x1 spsc v3 round-trip bench, closure
//! (`reserve_slot_with`) API, the segmented ring over a pool.

use std::hint::black_box;

use zc_ring_x1::spsc::v3::{Consumer, Producer};

use crate::benches::zcr_common::{Msg, leak_v3_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-spsc-v3-1t";

/// Same-thread round-trip through the v3 ring's
/// `reserve_slot_with` on both ends, the shape of `zcr-spsc-v2-1t`
/// over the segmented ring.
///
/// - One message in flight, so the consumer keeps up and the ring
///   lives in its first segment: the measurement is v2's fast path
///   plus whatever v3's look-ahead and its wider seq word cost
///   when no switch happens. The switch counts are printed after
///   the report and should read zero.
pub struct ZcrSpscV3OneThread {
    producer: Producer<'static>,
    consumer: Consumer<'static>,
    counter: u64,
}

impl ZcrSpscV3OneThread {
    /// Construct the bench over one fresh leaked v3 ring.
    pub fn new() -> Self {
        let (producer, consumer) = leak_v3_ring();
        Self {
            producer,
            consumer,
            counter: 0,
        }
    }

    /// Segment switches each end made, producer then consumer.
    pub fn switches(&self) -> (u64, u64) {
        (self.producer.switches(), self.consumer.switches())
    }
}

impl Bench for ZcrSpscV3OneThread {
    fn name(&self) -> &str {
        "zcr-spsc-v3-1t: zc-ring-x1 spsc v3 reserve_slot_with round-trip (1 thread)"
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
    let mut bench = ZcrSpscV3OneThread::new();
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
    let (p, c) = bench.switches();
    println!("segment switches: producer {p}, consumer {c}");
}
