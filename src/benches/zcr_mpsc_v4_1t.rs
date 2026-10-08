//! Single-threaded zc-ring-x1 mpsc v4 round-trip bench, the timed
//! spin forms (`send_spin`, `recv_spin`) with no time limit, the
//! attachable segmented ring over a pool, generic over the ring's
//! segment mode and its choice of how its endpoints wait.
//!
//! Each registry name is
//! `zcr-mpsc-v4-<threads>-<mode>-<wait choice>[-spnt<spin>][-slpt<sleep>]`.
//! A name states what the ring is, used or not, since that costs
//! either way, then what a receiver does, a time of zero left out:
//!
//! - Mode: `single`, or `multi-<n>seg` over `n` segments.
//! - Wait choice: the ring's, `so` for `SpinOnly` and `sos-futex`
//!   for `SpinOrSleep<Futex>`, the choice by its initials and then
//!   the waiter. A bench and its twin over the other choice differ
//!   in this field alone.
//! - Spin time, `spnt`: how long a receiver spins at an empty ring
//!   before it sleeps, `fe` being forever. zc-ring-x1's
//!   `spin_time`.
//! - Sleep time, `slpt`: how long it then sleeps, zc-ring-x1's
//!   `sleep_time`. No bench here sleeps, so none names one.

use std::hint::black_box;

use zc_ring_x1::Ticks;
use zc_ring_x1::mpsc::v4::{Mode, MpscConsumer, MpscProducer, Multi, Single};
use zc_ring_x1::wake::{Futex, SpinOnly, SpinOrSleep, Spins};

use crate::benches::zcr_common::{Msg, SEGMENTS, leak_mpsc_v4_ring, ring_switches};
use crate::harness::{self, Bench, RunCfg};
use crate::record;
use crate::report;

/// Registry name of the `Multi` ring over [`SEGMENTS`] segments, v3's
/// first bench in v4.
pub const NAME: &str = "zcr-mpsc-v4-1t-multi-2seg-so-spntfe";

/// Registry name of the `Multi` ring over one segment.
pub const NAME_1SEG: &str = "zcr-mpsc-v4-1t-multi-1seg-so-spntfe";

/// Registry name of the `Single` ring.
pub const NAME_SINGLE: &str = "zcr-mpsc-v4-1t-single-so-spntfe";

/// Registry name of [`NAME`]'s ring over `SpinOrSleep<Futex>`.
pub const NAME_SOS: &str = "zcr-mpsc-v4-1t-multi-2seg-sos-futex-spntfe";

/// Registry name of [`NAME_1SEG`]'s ring over `SpinOrSleep<Futex>`.
pub const NAME_1SEG_SOS: &str = "zcr-mpsc-v4-1t-multi-1seg-sos-futex-spntfe";

/// Registry name of [`NAME_SINGLE`]'s ring over `SpinOrSleep<Futex>`.
pub const NAME_SINGLE_SOS: &str = "zcr-mpsc-v4-1t-single-sos-futex-spntfe";

/// Same-thread round-trip sending through the v4 MPSC ring's
/// `send_spin` and receiving through its `recv_spin`, the shape of
/// `zcr-mpsc-v3-1t` over v4's ring.
///
/// - One message in flight, so the consumer keeps up and the ring
///   lives in its first segment: the measurement is v3's fast path
///   plus whatever v4's closure receive, its control block, mode
///   `M`, and wait choice `W` cost on it. The switch counts are
///   the run's counters and should read zero.
/// - Each spin is given `Ticks::FOREVER`, so neither side reads a
///   clock or gives up, and over a choice that can sleep nobody
///   does: what `W` adds is its checks for a sleeper.
pub struct ZcrMpscV4OneThread<M: Mode, W: Spins> {
    producer: MpscProducer<'static, M, W>,
    consumer: MpscConsumer<'static, M, W>,
    counter: u64,
    title: &'static str,
}

impl<M: Mode, W: Spins> ZcrMpscV4OneThread<M, W> {
    /// Construct the bench over one fresh leaked v4 MPSC ring of
    /// `segments` segments, reporting under `title`.
    pub fn new(segments: u32, title: &'static str) -> Self {
        let (producer, consumer) = leak_mpsc_v4_ring::<M, W>(segments);
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

impl<M: Mode, W: Spins> Bench for ZcrMpscV4OneThread<M, W> {
    fn name(&self) -> &str {
        self.title
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        let c = self.counter;
        self.producer
            .send_spin::<Msg>(Ticks::FOREVER, |m| *m = c)
            // OK: a spin of Ticks::FOREVER never gives up.
            .expect("a spin without end never gives up");
        let v = self
            .consumer
            .recv_spin::<Msg, Msg>(Ticks::FOREVER, |m| *m)
            // OK: as above, the spin never gives up.
            .expect("a spin without end never gives up");
        black_box(v)
    }
}

/// Run the bench over a ring of mode `M`, wait choice `W`, and
/// `segments` segments, reporting and recording under `name`.
pub fn run_as<M: Mode, W: Spins>(name: &str, title: &'static str, segments: u32, cfg: &RunCfg) {
    let mut bench = ZcrMpscV4OneThread::<M, W>::new(segments, title);
    let mut out = harness::run_adaptive(&mut bench, cfg);
    out.counters = ring_switches(bench.switches());
    report::print_report(bench.name(), &out, cfg);
    record::append(name, &out, cfg);
}

/// Registry entry point: `Multi` over [`SEGMENTS`] segments and
/// `SpinOnly`, v3's ring in v4.
pub fn run(cfg: &RunCfg) {
    run_as::<Multi, SpinOnly>(
        NAME,
        "zcr-mpsc-v4-1t-multi-2seg-so-spntfe: zc-ring-x1 mpsc v4 send round-trip (1 thread)",
        SEGMENTS,
        cfg,
    );
}

/// Registry entry point: `Multi` over one segment and `SpinOnly`,
/// `Single`'s geometry in the mode that can switch, so against
/// [`run_single`] the difference is the mode alone.
pub fn run_1seg(cfg: &RunCfg) {
    run_as::<Multi, SpinOnly>(
        NAME_1SEG,
        "zcr-mpsc-v4-1t-multi-1seg-so-spntfe: zc-ring-x1 mpsc v4 send round-trip, Multi over 1 segment (1 thread)",
        1,
        cfg,
    );
}

/// Registry entry point: `Single`, one segment and no switch
/// compiled in, over `SpinOnly`.
pub fn run_single(cfg: &RunCfg) {
    run_as::<Single, SpinOnly>(
        NAME_SINGLE,
        "zcr-mpsc-v4-1t-single-so-spntfe: zc-ring-x1 mpsc v4 send round-trip, Single (1 thread)",
        1,
        cfg,
    );
}

/// Registry entry point: [`run`]'s ring over `SpinOrSleep<Futex>`.
/// Nobody sleeps, every wait a spin, so against [`run`] the
/// difference is the checks for a sleeper on the message path.
pub fn run_sos(cfg: &RunCfg) {
    run_as::<Multi, SpinOrSleep<Futex>>(
        NAME_SOS,
        "zcr-mpsc-v4-1t-multi-2seg-sos-futex-spntfe: zc-ring-x1 mpsc v4 send round-trip, SpinOrSleep<Futex> (1 thread)",
        SEGMENTS,
        cfg,
    );
}

/// Registry entry point: [`run_1seg`]'s ring over
/// `SpinOrSleep<Futex>`.
pub fn run_1seg_sos(cfg: &RunCfg) {
    run_as::<Multi, SpinOrSleep<Futex>>(
        NAME_1SEG_SOS,
        "zcr-mpsc-v4-1t-multi-1seg-sos-futex-spntfe: zc-ring-x1 mpsc v4 send round-trip, Multi over 1 segment, SpinOrSleep<Futex> (1 thread)",
        1,
        cfg,
    );
}

/// Registry entry point: [`run_single`]'s ring over
/// `SpinOrSleep<Futex>`.
pub fn run_single_sos(cfg: &RunCfg) {
    run_as::<Single, SpinOrSleep<Futex>>(
        NAME_SINGLE_SOS,
        "zcr-mpsc-v4-1t-single-sos-futex-spntfe: zc-ring-x1 mpsc v4 send round-trip, Single, SpinOrSleep<Futex> (1 thread)",
        1,
        cfg,
    );
}
