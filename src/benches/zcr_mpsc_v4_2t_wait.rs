//! Two-threaded zc-ring-x1 mpsc v4 round-trip benches whose
//! receivers wait by a timed form with a limit, where the twins in
//! `zcr_mpsc_v4_2t` spin without one: a sleep, a spin and then a
//! sleep, and a timed spin.
//!
//! Each registry name is
//! `zcr-mpsc-v4-<threads>-<mode>-<wait choice>[-spnt<spin>][-slpt<sleep>]`.
//! A name states what the ring is, used or not, since that costs
//! either way, then what a receiver does, a time of zero left out:
//!
//! - Wait choice: the ring's, `so` for `SpinOnly` and `slp-futex`
//!   for `Sleep<Futex>`, the choice and then the waiter.
//! - Spin time, `spnt`: how long a receiver spins at an empty ring
//!   before it sleeps, `1us` a microsecond, zc-ring-x1's
//!   `spin_time`. A name without one sleeps at once.
//! - Sleep time, `slpt`: how long it then sleeps, `fe` being
//!   forever, a sleep until a producer wakes it, zc-ring-x1's
//!   `sleep_time`. A name without one does not sleep.
//! - The two times are one receive call's. A receive that gives
//!   up, as a spin with no sleep after it does, is made again, so
//!   `spnt1us` alone is a spin without end that reads the clock
//!   and returns every microsecond.

use std::hint::black_box;
use std::thread;

use zc_ring_x1::mpsc::v4::{Mode, MpscConsumer, MpscProducer, Single};
use zc_ring_x1::wake::{Futex, Sleep, Sleeps, SpinOnly, Spins, Waits};
use zc_ring_x1::{Ticks, microsecs_to_ticks};

use crate::benches::zcr_common::{Msg, STOP, leak_mpsc_v4_ring, round_trip_switches};
use crate::benches::zcr_mpsc_v4_2t::Switches;
use crate::harness::{self, Bench, RunCfg};
use crate::pin;
use crate::record;
use crate::report;

/// Registry name of the `Single` ring over `Sleep<Futex>` whose
/// receivers sleep at once, with no spin, until woken.
pub const NAME_SLEEP: &str = "zcr-mpsc-v4-2t-single-slp-futex-slptfe";

/// Registry name of the `Single` ring over `Sleep<Futex>` whose
/// receivers spin for a microsecond, then sleep until woken.
pub const NAME_SPIN_SLEEP: &str = "zcr-mpsc-v4-2t-single-slp-futex-spnt1us-slptfe";

/// Registry name of the `Single` ring over `SpinOnly` whose
/// receivers spin for a microsecond at a time.
pub const NAME_TIMED_SPIN: &str = "zcr-mpsc-v4-2t-single-so-spnt1us";

/// How both ends of a ring over `W` wait, a bench's one difference
/// from another's.
///
/// - Each method returns only once its message is through, trying
///   again when a timed form gives up, so a bench never gives up
///   and no result is unwrapped.
/// - `Copy`, since the worker thread takes its own.
pub trait Waiting<M: Mode, W: Waits>: Copy + Send + 'static {
    /// Send `v`, waiting at a full ring.
    fn send(&self, producer: &MpscProducer<'static, M, W>, v: Msg);

    /// Receive the oldest message, waiting at an empty ring.
    fn recv(&self, consumer: &mut MpscConsumer<'static, M, W>) -> Msg;
}

/// Spin for `spin`, then sleep for up to `sleep`, by
/// `send_spin_sleep` and `recv_spin_sleep`, on a ring whose choice
/// can sleep.
#[derive(Clone, Copy)]
pub struct SpinSleep {
    /// How long to spin before sleeping, `Ticks::ZERO` sleeping at
    /// once.
    pub spin: Ticks,
    /// How long to sleep in all, `Ticks::FOREVER` until woken.
    pub sleep: Ticks,
}

impl<M: Mode, W: Sleeps> Waiting<M, W> for SpinSleep {
    fn send(&self, producer: &MpscProducer<'static, M, W>, v: Msg) {
        while producer
            .send_spin_sleep::<Msg>(self.spin, self.sleep, |m| *m = v)
            .is_err()
        {}
    }

    fn recv(&self, consumer: &mut MpscConsumer<'static, M, W>) -> Msg {
        loop {
            if let Ok(v) = consumer.recv_spin_sleep::<Msg, Msg>(self.spin, self.sleep, |m| *m) {
                return v;
            }
        }
    }
}

/// Spin for `give_up` at a time, by `send_spin` and `recv_spin`, on
/// a ring whose choice offers the spin alone.
#[derive(Clone, Copy)]
pub struct TimedSpin {
    /// How long one spin lasts before it gives up and is made
    /// again.
    pub give_up: Ticks,
}

impl<M: Mode, W: Spins> Waiting<M, W> for TimedSpin {
    fn send(&self, producer: &MpscProducer<'static, M, W>, v: Msg) {
        while producer.send_spin::<Msg>(self.give_up, |m| *m = v).is_err() {}
    }

    fn recv(&self, consumer: &mut MpscConsumer<'static, M, W>) -> Msg {
        loop {
            if let Ok(v) = consumer.recv_spin::<Msg, Msg>(self.give_up, |m| *m) {
                return v;
            }
        }
    }
}

/// Main to worker to main round-trip over two zc-ring-x1 v4 MPSC
/// rings, the shape of `zcr-mpsc-v4-2t`, with both ends of both
/// rings waiting as `P` says.
///
/// - Waits: with one message in flight a ring is never full, so
///   only the two receivers wait, the worker for a request and
///   main for its response, and a send's wait never runs.
/// - A receiver that sleeps is woken by the send that feeds it, so
///   a round trip of [`NAME_SLEEP`] holds two sleeps and two wakes,
///   each a system call.
/// - Switches: neither ring leaves its first segment, and the four
///   ends' counts are the run's counters, expected zero.
/// - Shutdown: `Drop` sends the [`STOP`] sentinel, which wakes a
///   sleeping worker as any message does, and the worker exits on
///   receipt without replying.
pub struct ZcrMpscV4TwoThreadWait<M: Mode, W: Waits + 'static, P: Waiting<M, W>> {
    req_tx: MpscProducer<'static, M, W>,
    resp_rx: MpscConsumer<'static, M, W>,
    worker: Option<thread::JoinHandle<(u64, u64)>>,
    counter: u64,
    waiting: P,
    title: &'static str,
}

impl<M: Mode, W: Waits + 'static, P: Waiting<M, W>> ZcrMpscV4TwoThreadWait<M, W, P> {
    /// Spawn the echo worker over two fresh leaked v4 MPSC rings of
    /// `segments` segments each, waiting as `waiting` says,
    /// optionally pinning it to `worker_cpu`, reporting under
    /// `title`.
    pub fn new(segments: u32, waiting: P, title: &'static str, worker_cpu: Option<usize>) -> Self {
        let (req_tx, mut req_rx) = leak_mpsc_v4_ring::<M, W>(segments);
        let (resp_tx, resp_rx) = leak_mpsc_v4_ring::<M, W>(segments);
        let worker = thread::spawn(move || {
            pin::pin_current(worker_cpu);
            loop {
                let v = waiting.recv(&mut req_rx);
                if v == STOP {
                    break;
                }
                waiting.send(&resp_tx, v);
            }
            (req_rx.switches(), resp_tx.switches())
        });
        Self {
            req_tx,
            resp_rx,
            worker: Some(worker),
            counter: 0,
            waiting,
            title,
        }
    }

    /// Send [`STOP`], join the worker, and return the four ends'
    /// switch counts, as `zcr-mpsc-v4-2t`'s does.
    pub fn shutdown(&mut self) -> Switches {
        self.waiting.send(&self.req_tx, STOP);
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

impl<M: Mode, W: Waits + 'static, P: Waiting<M, W>> Bench for ZcrMpscV4TwoThreadWait<M, W, P> {
    fn name(&self) -> &str {
        self.title
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        if self.counter == STOP {
            self.counter = 1;
        }
        self.waiting.send(&self.req_tx, self.counter);
        black_box(self.waiting.recv(&mut self.resp_rx))
    }
}

impl<M: Mode, W: Waits + 'static, P: Waiting<M, W>> Drop for ZcrMpscV4TwoThreadWait<M, W, P> {
    /// Stop the worker if [`shutdown`](Self::shutdown) has not.
    fn drop(&mut self) {
        if self.worker.is_some() {
            self.shutdown();
        }
    }
}

/// Run the bench over rings of mode `M` and wait choice `W`, one
/// segment each, waiting as `waiting` says, reporting and recording
/// under `name`.
pub fn run_as<M: Mode, W: Waits + 'static, P: Waiting<M, W>>(
    name: &str,
    title: &'static str,
    waiting: P,
    cfg: &RunCfg,
) {
    let mut bench = ZcrMpscV4TwoThreadWait::<M, W, P>::new(1, waiting, title, cfg.cpu_for(1));
    let mut out = harness::run_adaptive(&mut bench, cfg);
    let s = bench.shutdown();
    out.counters = round_trip_switches(s.req, s.resp);
    report::print_report(bench.name(), &out, cfg);
    record::append(name, &out, cfg);
}

/// Registry entry point: `Single` over `Sleep<Futex>`, each
/// receiver sleeping at an empty ring with no spin, until the send
/// that feeds it wakes it.
pub fn run_sleep(cfg: &RunCfg) {
    let waiting = SpinSleep {
        spin: Ticks::ZERO,
        sleep: Ticks::FOREVER,
    };
    run_as::<Single, Sleep<Futex>, _>(
        NAME_SLEEP,
        "zcr-mpsc-v4-2t-single-slp-futex-slptfe: zc-ring-x1 mpsc v4 send round-trip, Single, Sleep<Futex> (2 threads, sleep)",
        waiting,
        cfg,
    );
}

/// Registry entry point: `Single` over `Sleep<Futex>`, each
/// receiver spinning for a microsecond before it sleeps.
///
/// - A round trip is far under a microsecond, so main's response
///   arrives inside its spin. The worker's wait is for the next
///   request, which is as long as the harness takes between two
///   steps, so the worker sleeps whenever that passes a
///   microsecond, and the round trip after it pays a wake.
/// - Against [`run_sleep`] the difference is the sleeps and the
///   wakes a spin spares.
pub fn run_spin_sleep(cfg: &RunCfg) {
    let waiting = SpinSleep {
        spin: microsecs_to_ticks(1),
        sleep: Ticks::FOREVER,
    };
    run_as::<Single, Sleep<Futex>, _>(
        NAME_SPIN_SLEEP,
        "zcr-mpsc-v4-2t-single-slp-futex-spnt1us-slptfe: zc-ring-x1 mpsc v4 send round-trip, Single, Sleep<Futex> (2 threads, spin 1 us then sleep)",
        waiting,
        cfg,
    );
}

/// Registry entry point: `Single` over `SpinOnly`, each receiver
/// spinning for a microsecond at a time. Against the twin that
/// spins without a limit the difference is the clock a timed spin
/// reads at each empty look.
pub fn run_timed_spin(cfg: &RunCfg) {
    let waiting = TimedSpin {
        give_up: microsecs_to_ticks(1),
    };
    run_as::<Single, SpinOnly, _>(
        NAME_TIMED_SPIN,
        "zcr-mpsc-v4-2t-single-so-spnt1us: zc-ring-x1 mpsc v4 send round-trip, Single (2 threads, spin 1 us at a time)",
        waiting,
        cfg,
    );
}
