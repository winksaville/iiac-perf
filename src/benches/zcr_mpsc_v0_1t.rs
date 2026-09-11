//! Single-threaded zc-ring-x1 mpsc v0 round-trip bench, closure
//! (`send_with`) API, the Vyukov-seq ring.

use std::hint::black_box;

use zc_ring_x1::mpsc::v0::{MpscConsumer, MpscProducer};

use crate::benches::zcr_common::{Msg, leak_mpsc_v0_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-mpsc-v0-1t";

/// Same-thread round-trip sending through the v0 MPSC ring's
/// `send_with` and receiving through its consumer guard.
///
/// - The wait closures never run here (one message in flight,
///   never full or empty), so the measurement is the MPSC
///   protocol's uncontended fast path, one claim CAS plus the
///   per-slot seq publish, against `zcr-spsc-v0-1t`'s
///   load/store-only SPSC pair.
pub struct ZcrMpscV0OneThread {
    producer: MpscProducer<'static>,
    consumer: MpscConsumer<'static>,
    counter: u64,
}

impl ZcrMpscV0OneThread {
    /// Construct the bench over one fresh leaked v0 MPSC ring.
    pub fn new() -> Self {
        let (producer, consumer) = leak_mpsc_v0_ring();
        Self {
            producer,
            consumer,
            counter: 0,
        }
    }
}

impl Bench for ZcrMpscV0OneThread {
    fn name(&self) -> &str {
        "zcr-mpsc-v0-1t: zc-ring-x1 mpsc v0 send_with round-trip (1 thread)"
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
    let mut bench = ZcrMpscV0OneThread::new();
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
}
