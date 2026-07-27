# Bugs

This file uses [Prose form](../AGENTS.md#prose-form). It
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

# References
