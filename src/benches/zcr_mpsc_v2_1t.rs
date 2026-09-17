//! Single-threaded zc-ring-x1 mpsc v2 round-trip bench, closure
//! (`send_with`) API, the segmented ring over a pool.

use std::hint::black_box;

use zc_ring_x1::mpsc::v2::{MpscConsumer, MpscProducer};

use crate::benches::zcr_common::{Msg, leak_mpsc_v2_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-mpsc-v2-1t";

/// Same-thread round-trip sending through the v2 MPSC ring's
/// `send_with` and receiving through its consumer guard, the
/// shape of `zcr-mpsc-v1-1t` over the segmented ring.
///
/// - One message in flight, so the consumer keeps up and the ring
///   lives in its first segment: the measurement is v1's fast
///   path, one claim CAS and the in-slot seq publish, plus
///   whatever v2's segment bookkeeping costs when no switch
///   happens. The switch counts print after the report and should
///   read zero.
pub struct ZcrMpscV2OneThread {
    producer: MpscProducer<'static>,
    consumer: MpscConsumer<'static>,
    counter: u64,
}

impl ZcrMpscV2OneThread {
    /// Construct the bench over one fresh leaked v2 MPSC ring.
    pub fn new() -> Self {
        let (producer, consumer) = leak_mpsc_v2_ring();
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

impl Bench for ZcrMpscV2OneThread {
    fn name(&self) -> &str {
        "zcr-mpsc-v2-1t: zc-ring-x1 mpsc v2 send_with round-trip (1 thread)"
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
            // OK: the closure returns true forever, so the send
            // never gives up.
            .expect("spin closure never gives up");
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
    let mut bench = ZcrMpscV2OneThread::new();
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
    let (p, c) = bench.switches();
    println!("segment switches: producer {p}, consumer {c}");
}
