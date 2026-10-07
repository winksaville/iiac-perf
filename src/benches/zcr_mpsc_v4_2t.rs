//! Two-threaded zc-ring-x1 mpsc v4 round-trip bench, the timed spin
//! forms (`send_spin`, `recv_spin`) with no time limit, the
//! attachable segmented ring over a pool, generic over the ring's
//! segment mode and its choice of how its endpoints wait.
//!
//! Each registry name is
//! `zcr-mpsc-v4-<threads>-<mode>-st<spin>-wt<wait>[-<wait choice>]`,
//! v3's form with the ring's choice of how its endpoints wait last:
//!
//! - Mode: `single`, or `multi-<n>seg` over `n` segments.
//! - Spin, `st`: how long a receiver spins at an empty ring before
//!   it waits, `fe` being forever.
//! - Wait, `wt`: how long it then waits, `none` being a receiver
//!   that never reaches a wait, as one spinning forever never does.
//! - Wait choice: the ring's, nothing for `SpinOnly` and
//!   `spinorsleep-futex` for `SpinOrSleep<Futex>`. A bench and its
//!   twin over the other choice differ in this field alone.

use std::hint::black_box;
use std::thread;

use zc_ring_x1::Ticks;
use zc_ring_x1::mpsc::v4::{Mode, MpscConsumer, MpscProducer, Multi, Single};
use zc_ring_x1::wake::{Futex, SpinOnly, SpinOrSleep, Spins};

use crate::benches::zcr_common::{Msg, SEGMENTS, STOP, leak_mpsc_v4_ring, round_trip_switches};
use crate::harness::{self, Bench, RunCfg};
use crate::pin;
use crate::record;
use crate::report;

/// Registry name of the `Multi` ring over [`SEGMENTS`] segments, v3's
/// first bench in v4.
pub const NAME: &str = "zcr-mpsc-v4-2t-multi-2seg-stfe-wtnone";

/// Registry name of the `Multi` ring over one segment.
pub const NAME_1SEG: &str = "zcr-mpsc-v4-2t-multi-1seg-stfe-wtnone";

/// Registry name of the `Single` ring.
pub const NAME_SINGLE: &str = "zcr-mpsc-v4-2t-single-stfe-wtnone";

/// Registry name of [`NAME`]'s ring over `SpinOrSleep<Futex>`.
pub const NAME_SOS: &str = "zcr-mpsc-v4-2t-multi-2seg-stfe-wtnone-spinorsleep-futex";

/// Registry name of [`NAME_1SEG`]'s ring over `SpinOrSleep<Futex>`.
pub const NAME_1SEG_SOS: &str = "zcr-mpsc-v4-2t-multi-1seg-stfe-wtnone-spinorsleep-futex";

/// Registry name of [`NAME_SINGLE`]'s ring over `SpinOrSleep<Futex>`.
pub const NAME_SINGLE_SOS: &str = "zcr-mpsc-v4-2t-single-stfe-wtnone-spinorsleep-futex";

/// Segment switches of the four ends of the two rings.
pub struct Switches {
    /// The request ring, producer then consumer.
    pub req: (u64, u64),
    /// The response ring, producer then consumer.
    pub resp: (u64, u64),
}

/// Main to worker to main round-trip over two zc-ring-x1 v4 MPSC
/// rings, one producer per ring, the shape of `zcr-mpsc-v3-2t`
/// over v4's ring, so the pair at one placement is the cross-core
/// handoff of the two rings side by side.
///
/// - Waits: `send_spin` and `recv_spin` with `Ticks::FOREVER` on
///   both sides, so nobody reads a clock, gives up, or sleeps, and
///   what `W` adds is its checks for a sleeper on the message
///   path.
/// - Switches: one message in flight means the consumer keeps up
///   and neither ring leaves its first segment. The worker hands
///   its two ends' counts back at shutdown, and the four are the
///   run's counters, expected zero.
/// - Shutdown: `Drop` sends the [`STOP`] sentinel, and the worker
///   exits on receipt without replying.
pub struct ZcrMpscV4TwoThread<M: Mode, W: Spins + 'static> {
    req_tx: MpscProducer<'static, M, W>,
    resp_rx: MpscConsumer<'static, M, W>,
    worker: Option<thread::JoinHandle<(u64, u64)>>,
    counter: u64,
    title: &'static str,
}

impl<M: Mode, W: Spins + 'static> ZcrMpscV4TwoThread<M, W> {
    /// Spawn the spinning echo worker over two fresh leaked v4
    /// MPSC rings of `segments` segments each, optionally pinning
    /// it to `worker_cpu`, reporting under `title`.
    pub fn new(segments: u32, title: &'static str, worker_cpu: Option<usize>) -> Self {
        let (req_tx, mut req_rx) = leak_mpsc_v4_ring::<M, W>(segments);
        let (resp_tx, resp_rx) = leak_mpsc_v4_ring::<M, W>(segments);
        let worker = thread::spawn(move || {
            pin::pin_current(worker_cpu);
            loop {
                let v = req_rx
                    .recv_spin::<Msg, Msg>(Ticks::FOREVER, |m| *m)
                    // OK: a spin of Ticks::FOREVER never gives up.
                    .expect("a spin without end never gives up");
                if v == STOP {
                    break;
                }
                resp_tx
                    .send_spin::<Msg>(Ticks::FOREVER, |m| *m = v)
                    // OK: as above, the spin never gives up.
                    .expect("a spin without end never gives up");
            }
            (req_rx.switches(), resp_tx.switches())
        });
        Self {
            req_tx,
            resp_rx,
            worker: Some(worker),
            counter: 0,
            title,
        }
    }

    /// Send [`STOP`], join the worker, and return the four ends'
    /// switch counts. A second call returns zeros for the worker's
    /// ends, since it has gone.
    pub fn shutdown(&mut self) -> Switches {
        self.req_tx
            .send_spin::<Msg>(Ticks::FOREVER, |m| *m = STOP)
            // OK: a spin of Ticks::FOREVER never gives up.
            .expect("a spin without end never gives up");
        let (req_rx, resp_tx) = match self.worker.take() {
            Some(worker) => worker.join().unwrap_or((0, 0)), // OK: a panicked worker has no counts, and the bench already printed its report
            None => (0, 0),
        };
        Switches {
            req: (self.req_tx.switches(), req_rx),
            resp: (resp_tx, self.resp_rx.switches()),
        }
    }
}

impl<M: Mode, W: Spins + 'static> Bench for ZcrMpscV4TwoThread<M, W> {
    fn name(&self) -> &str {
        self.title
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        if self.counter == STOP {
            self.counter = 1;
        }
        let c = self.counter;
        self.req_tx
            .send_spin::<Msg>(Ticks::FOREVER, |m| *m = c)
            // OK: a spin of Ticks::FOREVER never gives up.
            .expect("a spin without end never gives up");
        let v = self
            .resp_rx
            .recv_spin::<Msg, Msg>(Ticks::FOREVER, |m| *m)
            // OK: as above, the spin never gives up.
            .expect("a spin without end never gives up");
        black_box(v)
    }
}

impl<M: Mode, W: Spins + 'static> Drop for ZcrMpscV4TwoThread<M, W> {
    /// Stop the worker if [`shutdown`](Self::shutdown) has not.
    fn drop(&mut self) {
        if self.worker.is_some() {
            self.shutdown();
        }
    }
}

/// Run the bench over rings of mode `M`, wait choice `W`, and
/// `segments` segments, reporting and recording under `name`.
pub fn run_as<M: Mode, W: Spins + 'static>(
    name: &str,
    title: &'static str,
    segments: u32,
    cfg: &RunCfg,
) {
    let mut bench = ZcrMpscV4TwoThread::<M, W>::new(segments, title, cfg.cpu_for(1));
    let mut out = harness::run_adaptive(&mut bench, cfg);
    let s = bench.shutdown();
    out.counters = round_trip_switches(s.req, s.resp);
    report::print_report(bench.name(), &out, cfg);
    record::append(name, &out, cfg);
}

/// Registry entry point: `Multi` over [`SEGMENTS`] segments and
/// `SpinOnly`, v3's ring in v4.
pub fn run(cfg: &RunCfg) {
    run_as::<Multi, SpinOnly>(
        NAME,
        "zcr-mpsc-v4-2t-multi-2seg-stfe-wtnone: zc-ring-x1 mpsc v4 send round-trip (2 threads, spin)",
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
        "zcr-mpsc-v4-2t-multi-1seg-stfe-wtnone: zc-ring-x1 mpsc v4 send round-trip, Multi over 1 segment (2 threads, spin)",
        1,
        cfg,
    );
}

/// Registry entry point: `Single`, one segment and no switch
/// compiled in, over `SpinOnly`.
pub fn run_single(cfg: &RunCfg) {
    run_as::<Single, SpinOnly>(
        NAME_SINGLE,
        "zcr-mpsc-v4-2t-single-stfe-wtnone: zc-ring-x1 mpsc v4 send round-trip, Single (2 threads, spin)",
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
        "zcr-mpsc-v4-2t-multi-2seg-stfe-wtnone-spinorsleep-futex: zc-ring-x1 mpsc v4 send round-trip, SpinOrSleep<Futex> (2 threads, spin)",
        SEGMENTS,
        cfg,
    );
}

/// Registry entry point: [`run_1seg`]'s ring over
/// `SpinOrSleep<Futex>`.
pub fn run_1seg_sos(cfg: &RunCfg) {
    run_as::<Multi, SpinOrSleep<Futex>>(
        NAME_1SEG_SOS,
        "zcr-mpsc-v4-2t-multi-1seg-stfe-wtnone-spinorsleep-futex: zc-ring-x1 mpsc v4 send round-trip, Multi over 1 segment, SpinOrSleep<Futex> (2 threads, spin)",
        1,
        cfg,
    );
}

/// Registry entry point: [`run_single`]'s ring over
/// `SpinOrSleep<Futex>`.
pub fn run_single_sos(cfg: &RunCfg) {
    run_as::<Single, SpinOrSleep<Futex>>(
        NAME_SINGLE_SOS,
        "zcr-mpsc-v4-2t-single-stfe-wtnone-spinorsleep-futex: zc-ring-x1 mpsc v4 send round-trip, Single, SpinOrSleep<Futex> (2 threads, spin)",
        1,
        cfg,
    );
}
