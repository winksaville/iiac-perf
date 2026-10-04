//! Single-threaded zc-ring-x1 mpsc v3 round-trip bench, policy
//! (`send`) API, the attachable segmented ring over a pool, generic
//! over the ring's segment mode and wake.

use std::hint::black_box;

use zc_ring_x1::mpsc::v3::{Mode, MpscConsumer, MpscProducer, Multi, Single};
use zc_ring_x1::policy;
use zc_ring_x1::wake::{NoWake, Wake};

use crate::benches::zcr_common::{Msg, SEGMENTS, leak_mpsc_v3_ring, ring_switches};
use crate::harness::{self, Bench, RunCfg};
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-mpsc-v3-1t";

/// Registry name of the `Multi` ring over one segment.
pub const NAME_1SEG: &str = "zcr-mpsc-v3-1t-1seg";

/// Registry name of the `Single` ring.
pub const NAME_SINGLE: &str = "zcr-mpsc-v3-1t-single";

/// Same-thread round-trip sending through the v3 MPSC ring's
/// `send` and receiving through its consumer guard, the shape of
/// `zcr-mpsc-v2-1t` over the attachable ring.
///
/// - One message in flight, so the consumer keeps up and the ring
///   lives in its first segment: the measurement is v2's fast
///   path plus whatever v3's control block, counted roles, mode `M`,
///   and wake `W` cost on it. The switch counts are the run's
///   counters and should read zero.
/// - Every wait spins, so over a wake that can sleep nobody does,
///   and what `W` adds is its checks on the message path.
pub struct ZcrMpscV3OneThread<M: Mode, W: Wake> {
    producer: MpscProducer<'static, M, W>,
    consumer: MpscConsumer<'static, M, W>,
    counter: u64,
    title: &'static str,
}

impl<M: Mode, W: Wake> ZcrMpscV3OneThread<M, W> {
    /// Construct the bench over one fresh leaked v3 MPSC ring of
    /// `segments` segments, reporting under `title`.
    pub fn new(segments: u32, title: &'static str) -> Self {
        let (producer, consumer) = leak_mpsc_v3_ring::<M, W>(segments);
        Self {
            producer,
            consumer,
            counter: 0,
            title,
        }
    }

    /// Segment switches each end made, producer then consumer.
    pub fn switches(&self) -> (u64, u64) {
        (self.producer.switches(), self.consumer.switches())
    }
}

impl<M: Mode, W: Wake> Bench for ZcrMpscV3OneThread<M, W> {
    fn name(&self) -> &str {
        self.title
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        let c = self.counter;
        self.producer
            .send::<Msg>(policy::spin, |m| *m = c)
            // OK: the spin policy returns true forever, so the send
            // never gives up.
            .expect("spin policy never gives up");
        let slot = self
            .consumer
            .reserve_slot_with::<Msg>(policy::spin)
            // OK: as above, the policy never gives up.
            .expect("spin policy never gives up");
        let v = *slot;
        slot.release();
        black_box(v)
    }
}

/// Run the bench over a ring of mode `M`, wake `W`, and `segments`
/// segments, reporting and recording under `name`.
pub fn run_as<M: Mode, W: Wake>(name: &str, title: &'static str, segments: u32, cfg: &RunCfg) {
    let mut bench = ZcrMpscV3OneThread::<M, W>::new(segments, title);
    let mut out = harness::run_adaptive(&mut bench, cfg);
    out.counters = ring_switches(bench.switches());
    report::print_report(bench.name(), &out, cfg);
    record::append(name, &out, cfg);
}

/// Registry entry point: `Multi` over [`SEGMENTS`] segments with
/// no wake, v2's ring in v3.
pub fn run(cfg: &RunCfg) {
    run_as::<Multi, NoWake>(
        NAME,
        "zcr-mpsc-v3-1t: zc-ring-x1 mpsc v3 send round-trip (1 thread)",
        SEGMENTS,
        cfg,
    );
}

/// Registry entry point: `Multi` over one segment with no wake,
/// `Single`'s geometry in the mode that can switch, so against
/// [`run_single`] the difference is the mode alone.
pub fn run_1seg(cfg: &RunCfg) {
    run_as::<Multi, NoWake>(
        NAME_1SEG,
        "zcr-mpsc-v3-1t-1seg: zc-ring-x1 mpsc v3 send round-trip, Multi over 1 segment (1 thread)",
        1,
        cfg,
    );
}

/// Registry entry point: `Single`, one segment and no switch
/// compiled in, with no wake.
pub fn run_single(cfg: &RunCfg) {
    run_as::<Single, NoWake>(
        NAME_SINGLE,
        "zcr-mpsc-v3-1t-single: zc-ring-x1 mpsc v3 send round-trip, Single (1 thread)",
        1,
        cfg,
    );
}
