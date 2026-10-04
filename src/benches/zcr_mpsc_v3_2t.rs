//! Two-threaded zc-ring-x1 mpsc v3 round-trip bench, policy
//! (`send`) API, spin waits, the attachable segmented ring over a
//! pool, generic over the ring's segment mode and wake.

use std::hint::black_box;
use std::thread;

use zc_ring_x1::mpsc::v3::{Mode, MpscConsumer, MpscProducer, Multi};
use zc_ring_x1::policy;
use zc_ring_x1::wake::{NoWake, Wake};

use crate::benches::zcr_common::{Msg, SEGMENTS, STOP, leak_mpsc_v3_ring, round_trip_switches};
use crate::harness::{self, Bench, RunCfg};
use crate::pin;
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "zcr-mpsc-v3-2t";

/// Segment switches of the four ends of the two rings.
pub struct Switches {
    /// The request ring, producer then consumer.
    pub req: (u64, u64),
    /// The response ring, producer then consumer.
    pub resp: (u64, u64),
}

/// Main to worker to main round-trip over two zc-ring-x1 v3 MPSC
/// rings, one producer per ring, the shape of `zcr-mpsc-v2-2t`
/// over the attachable ring, so the pair at one placement is the
/// cross-core handoff of the two rings side by side.
///
/// - Wait policy: a `spin_loop` hint per failed attempt on both
///   the send and receive sides, so over a wake that can sleep
///   nobody does, and what `W` adds is its checks on the message
///   path.
/// - Switches: one message in flight means the consumer keeps up
///   and neither ring leaves its first segment. The worker hands
///   its two ends' counts back at shutdown, and the four are the
///   run's counters, expected zero.
/// - Shutdown: `Drop` sends the [`STOP`] sentinel, and the worker
///   exits on receipt without replying.
pub struct ZcrMpscV3TwoThread<M: Mode, W: Wake + 'static> {
    req_tx: MpscProducer<'static, M, W>,
    resp_rx: MpscConsumer<'static, M, W>,
    worker: Option<thread::JoinHandle<(u64, u64)>>,
    counter: u64,
    title: &'static str,
}

impl<M: Mode, W: Wake + 'static> ZcrMpscV3TwoThread<M, W> {
    /// Spawn the spinning echo worker over two fresh leaked v3
    /// MPSC rings of `segments` segments each, optionally pinning
    /// it to `worker_cpu`, reporting under `title`.
    pub fn new(segments: u32, title: &'static str, worker_cpu: Option<usize>) -> Self {
        let (req_tx, mut req_rx) = leak_mpsc_v3_ring::<M, W>(segments);
        let (resp_tx, resp_rx) = leak_mpsc_v3_ring::<M, W>(segments);
        let worker = thread::spawn(move || {
            pin::pin_current(worker_cpu);
            loop {
                let v = {
                    let slot = req_rx
                        .reserve_slot_with::<Msg>(policy::spin)
                        // OK: the spin policy returns true forever,
                        // so the reserve never gives up.
                        .expect("spin policy never gives up");
                    let v = *slot;
                    slot.release();
                    v
                };
                if v == STOP {
                    break;
                }
                resp_tx
                    .send::<Msg>(policy::spin, |m| *m = v)
                    // OK: as above, the policy never gives up.
                    .expect("spin policy never gives up");
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
            .send::<Msg>(policy::spin, |m| *m = STOP)
            // OK: the spin policy returns true forever, so the send
            // never gives up.
            .expect("spin policy never gives up");
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

impl<M: Mode, W: Wake + 'static> Bench for ZcrMpscV3TwoThread<M, W> {
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
            .send::<Msg>(policy::spin, |m| *m = c)
            // OK: the spin policy returns true forever, so the send
            // never gives up.
            .expect("spin policy never gives up");
        let slot = self
            .resp_rx
            .reserve_slot_with::<Msg>(policy::spin)
            // OK: as above, the policy never gives up.
            .expect("spin policy never gives up");
        let v = *slot;
        slot.release();
        black_box(v)
    }
}

impl<M: Mode, W: Wake + 'static> Drop for ZcrMpscV3TwoThread<M, W> {
    /// Stop the worker if [`shutdown`](Self::shutdown) has not.
    fn drop(&mut self) {
        if self.worker.is_some() {
            self.shutdown();
        }
    }
}

/// Run the bench over rings of mode `M`, wake `W`, and `segments`
/// segments, reporting and recording under `name`.
pub fn run_as<M: Mode, W: Wake + 'static>(
    name: &str,
    title: &'static str,
    segments: u32,
    cfg: &RunCfg,
) {
    let mut bench = ZcrMpscV3TwoThread::<M, W>::new(segments, title, cfg.cpu_for(1));
    let mut out = harness::run_adaptive(&mut bench, cfg);
    let s = bench.shutdown();
    out.counters = round_trip_switches(s.req, s.resp);
    report::print_report(bench.name(), &out, cfg);
    record::append(name, &out, cfg);
}

/// Registry entry point: `Multi` over [`SEGMENTS`] segments with
/// no wake, v2's ring in v3.
pub fn run(cfg: &RunCfg) {
    run_as::<Multi, NoWake>(
        NAME,
        "zcr-mpsc-v3-2t: zc-ring-x1 mpsc v3 send round-trip (2 threads, spin)",
        SEGMENTS,
        cfg,
    );
}
