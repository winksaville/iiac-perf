//! Single-threaded zc-ring-x1 mpsc v1 round-trip bench, closure
//! (`send_with`) API, the equality-seq ring.

use std::hint::black_box;

use zc_ring_x1::mpsc::v1::{MpscConsumer, MpscProducer};

use crate::benches::zcr_common::{Msg, leak_mpsc_v1_ring};
use crate::harness::{self, Bench, RunCfg};
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-mpsc-v1-1t";

/// Same-thread round-trip sending through the v1 MPSC ring's
/// `send_with` and receiving through its consumer guard, the
/// shape of `zcr-mpsc-v0-1t` over the equality-seq protocol.
///
/// - The wait closures never run here (one message in flight,
///   never full or empty), so the measurement is v1's
///   uncontended fast path, one claim CAS plus the per-slot seq
///   publish, where v1 checks the seq by equality against
///   `pos + M + 1` and v0 by a signed diff against `pos + 1`.
///   The prediction on record in zc-ring-x1 is v0's cost at every
///   depth above 1.
pub struct ZcrMpscV1OneThread {
    producer: MpscProducer<'static>,
    consumer: MpscConsumer<'static>,
    counter: u64,
}

impl ZcrMpscV1OneThread {
    /// Construct the bench over one fresh leaked v1 MPSC ring.
    pub fn new() -> Self {
        let (producer, consumer) = leak_mpsc_v1_ring();
        Self {
            producer,
            consumer,
            counter: 0,
        }
    }
}

impl Bench for ZcrMpscV1OneThread {
    fn name(&self) -> &str {
        "zcr-mpsc-v1-1t: zc-ring-x1 mpsc v1 send_with round-trip (1 thread)"
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
    let mut bench = ZcrMpscV1OneThread::new();
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
}
