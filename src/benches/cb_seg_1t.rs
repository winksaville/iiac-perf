//! Single-threaded `crossbeam_queue::SegQueue` round-trip bench.

use std::hint::black_box;

use crossbeam_queue::SegQueue;

use crate::harness::{self, Bench, RunCfg};
use crate::record;
use crate::report;

/// Registry name used on the CLI.
pub const NAME: &str = "cb-seg-1t";

/// Same-thread push-then-pop through one `SegQueue`, the
/// ecosystem's unbounded segmented queue. Measures the queue's own
/// overhead with no scheduler interaction: one message in flight,
/// so the steady state stays inside one segment and the allocator
/// is never on the path.
///
/// - Capability class: `SegQueue` is MPMC, the closest structural
///   peer to a segmented SPSC over a pool. A queue that promises
///   less is allowed to be faster, so read this row beside the MPSC
///   and SPSC rows with that in mind.
pub struct CbSegRoundTrip {
    queue: SegQueue<u64>,
    counter: u64,
}

impl CbSegRoundTrip {
    /// Construct the bench with a fresh queue.
    pub fn new() -> Self {
        Self {
            queue: SegQueue::new(),
            counter: 0,
        }
    }
}

impl Bench for CbSegRoundTrip {
    fn name(&self) -> &str {
        "cb-seg-1t: crossbeam_queue::SegQueue round-trip (1 thread)"
    }

    fn step(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        self.queue.push(self.counter);
        // OK: the pop follows its push on the same thread, and no
        // other thread touches the queue, so it is never empty here.
        let v = self.queue.pop().unwrap();
        black_box(v)
    }
}

/// Registry entry point.
pub fn run(cfg: &RunCfg) {
    let mut bench = CbSegRoundTrip::new();
    let out = harness::run_adaptive(&mut bench, cfg);
    report::print_report(bench.name(), &out, cfg);
    record::append(NAME, &out, cfg);
}
