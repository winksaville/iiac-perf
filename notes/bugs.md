# Known bugs

This file uses [Prose form](../agent-data/prose.md#prose-form). It
lists known defects we're aware of but haven't scheduled a fix for.
Each entry describes what goes wrong, when, and the cost of
the failure. Entries are numbered (`1.` `2.` …) the same way
as `## Todo` in `../TODO.md`; run
`vc-x1 fix-todo --no-dry-run notes/bugs.md` to renumber after
insert / delete / reorder.

## Bugs

1. 2t benches accept a 1-CPU pin pool and livelock through spin
   handoffs. `core_for` wraps the pin pool (`src/harness.rs`),
   so e.g. `iiac-perf zcr-mpsc-2t --pin 8` pins both the main
   thread and the spinning echo worker to core 8. Neither side
   yields, so every handoff waits for an involuntary preemption
   (milliseconds instead of ~200 ns) — the 5×1,000-step cost
   estimate alone takes minutes and the run appears hung.
   Observed on 3900x 2026-07-26; `--pin 8,9` behaves normally.
   Cost: an apparent hang the user must ^C, with no hint that
   the pinning was the cause. The bug requires pinning — an
   unpinned run lets the scheduler separate the threads and
   behaves normally. Fix direction:
   - Track `core_for` requests in `RunCfg` (max `thread_idx`
     asked for): thread placement only goes through `core_for`
     when pinning is active, so refusing a pool with fewer
     unique CPUs than requested placements covers every path
     to this bug — no per-bench thread-count declaration
     needed.
   - Independently, put a wall-clock deadline on the open-loop
     5×1,000-step estimate phase so *any* pathologically slow
     bench aborts with a diagnostic instead of hanging.
   - Update 2026-08-02 (0.24.0): the deadline half is fixed structurally: the estimate phase is
     gone, and every warmup pass is deadlined by the warm cap (`--warm-cap`, default 1.5 s), so
     the hang shrinks to a bounded wait ending in an "uncertified" report. The pool-size guard
     half remains open (the run still livelocks through the measurement itself).
   - Update 2026-09-02 (0.27.0): `cb-seg-2t` joins the spinning 2t set this covers; `cb-chan-2t`
     parks and does not.

2. `suggest-freq` perturbs the run it measures. Its descent
   wraps each candidate's bench in `sample_while`
   (`src/freqctl.rs:699`), which spawns an unpinned thread
   that wakes about 20 times a second to read
   `scaling_cur_freq`. An ordinary run has no such thread.
   Measured on 7600x 2026-09-04 with `zcr-mpsc-2t
   --blocks=100 --inner 100 -d 60 --pin-cpus 1,2`, three
   interleaved reps per arm, every run graded A:
   `--pin-freq=4701` read 74.43 / 74.15 / 74.41 ns while
   `suggest-freq` at that same 4701 read 62.51 / 62.82 /
   62.89, an 18.5% gap. Both reported `4.67->4.67GHz 99%`
   and both verified "Delivered 4.66-4.67 GHz". The gap is
   real work rather than a display artifact: `outer` was
   7.93M against 9.40M in the same 60.8 s, a ratio of 1.185
   matching the latency ratio exactly. We think the waker
   holds the package out of deep idle and the fabric clock
   with it, which is what a cross-core round trip rides.
   Cost: a `suggest-freq` bench report cannot be compared
   against any ordinary run's, and nothing on the report
   says so. Fix direction:
   - Reuse the run's own delivered-clock series instead of
     sampling separately. Every run already records one
     (`clock_t_ns` / `clock_cpu` / `clock_khz` in the
     record, and the settle cell is built from it), and that
     path demonstrably does not perturb, since the plain
     `--pin-freq` run carries it and droops anyway.
   - Failing that, pin the sampler outside the bench's CPU
     set, and say on the report that a descent's numbers are
     measured under a sampler.
   - Sampling inline on the measuring thread works only at
     block granularity. `outer` runs to millions, so a
     per-sample sysfs read would cost more than it measures,
     while the block gap is already a non-measuring window.
   - Whichever way it lands, the fixed `suggest-freq` will
     report *slower* numbers than it does today, which is
     correct: it should measure what an ordinary run gets.

# References
