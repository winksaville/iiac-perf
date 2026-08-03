# Chores 06

Continuation of [chores-05](chores-05.md). Records landed work;
conventions in
[agent-data/notes.md](../../agent-data/notes.md#chores-conventions)
and [cycle-protocol.md](../cycle-protocol.md#chores-sections).

## Table of Contents

- [feat: dynamic warmup](#feat-dynamic-warmup)

## feat: dynamic warmup

- [[N]] 0.24.0-0 chore: open the dynamic-warmup cycle
- [[N]] 0.24.0-1 refactor: one parameterized warm loop
- [[N]] 0.24.0-2 feat: warm until the trailing window grades A
- [[N]] 0.24.0-3 feat: warm where the bench runs

The 0.24.0 cycle: replace the fixed `WARMUP = 10_000` step count
in `harness.rs` with warm-until-stable. A fixed count's
wall-clock scales with step cost, so the fastest benches warm
~10 us against frequency-governor ramps of tens-to-hundreds of
ms and `pick_inner` sizes mid-ramp (the 7600x F diagnosis,
[Replanning II](chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run)),
and a timing-only "did it end settled" test can issue a vacuous
A while the box dwells one P-state below the top (measured
2026-07-29 [[1]]). The design accumulated in the Todo entry over
2026-07-27 to 2026-08-01; the subsections below are its record,
plus the decisions made at pickup (2026-08-02). First cycle run
on a topic bookmark (`dynamic-warmup`).

### One warm loop, three policies

The end state, decided 2026-08-01: one parameterized warm loop.
Step the bench, probe periodically, stop when the exit condition
holds. The harness's three warms become policies over that one
mechanism:

- the per-run warmup exits when the trailing window grades A,
  or at the cap
- the process warm (`process_warm`) exits on the `--settle-time`
  budget
- the block warm (`run_blocked`'s 2 ms spin) is the same loop
  with a fixed-time exit and probing disabled

`process_warm` and `warmup_and_probe` already share one probe
series, prober and time origin; this completes the fusion
instead of adding a fourth variant.

Terminology: the warmup unit is a **warmup pass** (the 0.22.0-4
"loop-only passes" sense), a short, unrecorded, timed burst of
bench steps yielding one floor. Not a "probe": `TProbe` is the
measurement instrument, and the "micro-probe" is the 0.23.0
cycle's ~1 ms timer-pair frame measurement.

### The exit condition: grade the trailing window

Rather than K agreeing floors, grade a *sliding window* over
the warmup probe series and warm until the trailing window
reads A, or the cap (design 2026-07-28). The exit condition and
the warmup letter become one computation:

- exit on A means the run started post-ramp by construction
- hitting the cap reports whatever the window actually scored,
  the "run started unstable" signal, not a silent proceed
- window length takes the same "minimum count or minimum wall
  time, whichever is larger" shape as pass length: count
  because the split detector needs 4 points a side, wall time
  because the ramp is a ms-scale phenomenon
- signals: spread, drift, step; `interference` is the weak one
  [[2]]
- the minimum wall span is load-bearing: a window can be far
  shorter than what it certifies (`min-now`'s 16 warmup probes
  span ~17 us against a transition arriving at ~800 ms), so
  agreement alone certifies nothing

Floors, not means, so one preemption doesn't fake
(in)stability; a warm box exits near-immediately.

Convergence is agreement, not direction. The 3900X is bistable
(2026-07-27 `calibrate` runs): sustained rapid repetition
climbs it into a fast state (~0.445 ns/iter), low-duty isolated
runs sit at ~0.489 (B), ~9% apart, and transitions straddle
windows in *both* directions; a fixed warmup absorbs only
transitions shorter than itself, whatever their sign.

The hard cap sits at governor scale (a few hundred ms; exact
constant measured on both boxes during the rung). The cap
doubles as the estimate-phase deadline the `--pin` guard Todo
wants, so slow and non-converging benches share one diagnostic
exit.

### Sizing fusion

The warmup pass *is* the step-cost sizing pass: the converged
floor is the sizing input, so sizing is post-ramp by
construction and convergence is tested on the number actually
consumed. The `estimate_step_cost` phase folds into the warm
loop; the micro-probe supplies the frame input, run after
convergence.

Slow steps (`inner` -> 1 territory): pass length adapts
(minimum step count or minimum wall time, whichever is larger)
so a floor is never one sample. The cap exit then distinguishes
"floors disagreed" (unstable, gauge signal) from "too slow to
certify" (proceed, label the run uncertified: at `inner = 1`
sizing can't be wrong and framing is negligible, so the stakes
are low there).

### Read the clock, not just the timing

Steadiness cannot tell "settled at the top" from "dwelling at
an intermediate P-state", because a dwell *is* steady. Measured
2026-07-29 on the 7600x [[1]]: the trailing window graded A
while the box held 4841 MHz for ~0.75 s, then stepped +12.4%
inside the run. This was the strongest argument for making the
clock reading part of the exit condition rather than an
optional extra, and the rung is in scope (decided 2026-08-02).

- delivered frequency is an unprivileged sysfs read on both AMD
  boxes (`cpufreq/cpuinfo_avg_freq`); the ramp is ~150-200 ms
- gate on **clock stability under load**, never on a fraction
  of `cpuinfo_max_freq`: a threshold would need tuning between
  96.1% (3900X) and 99.7% (7600x) sustained, and a
  thermally-limited laptop plateaus lower still while that
  plateau is its honest clock
- optional by construction: `cpuinfo_avg_freq` is
  amd-pstate-specific and some drivers' `scaling_cur_freq`
  reports requested rather than delivered, so read where
  present and fall back to timing-only
- report the ratio, do not grade on it

`qualify-environment`'s verdict is not usable as a gate until
this cycle lands: it reads NOT QUALIFIED on any amd-pstate-epp
box that dwells then boosts, which is to say on a healthy idle
machine [[1]]. Fixing the exit condition fixes the selftest at
the same time, since its observable is this grade.

### Placement: warm where the bench runs

Decided 2026-08-02: the warm follows the bench's pin. Warm on
`pin[0]` when `--pin` is set, else wherever the scheduler has
main (a busy thread stays put, and the warm state lands on the
core that measures). The CPU0-default tick-rate warm pin and
`--no-pin-cal` are deleted rather than justified.

- CPU0 is measurably the kernel's busiest core on the 3900X
  (2026-07-29, cumulative per-CPU interrupts: 4.1M `LOC`
  against ~0.6M on CPU11/CPU23, 6.2M `CAL` against ~0.6M, 4-6x
  the `RES`, 3x the `TLB`; no `irqbalance`). CPU0 is the boot
  CPU and the `nohz_full` housekeeping CPU, so this is expected
  rather than a quirk of this box
- the pin did not matter for the tick-rate read (a ratio of TSC
  ticks to monotonic ns over ~10 ms: interruptions inflate both
  sides and cancel, ~8e-7 spread across cores), so the current
  default was harmless, not correct; it would have mattered
  here, where warmup becomes a real timing phase converging on
  per-core frequency state
- the rejected alternative, a topology-aware "not the boot CPU,
  and a full-frequency core" pin, adds machinery this cycle
  does not need; it stays with the topology Todo. "Use the last
  core" folklore was already rejected there (hybrid parts put
  E-cores at high indices)
- as built at -3: main pins to `pin[0]` (and stays; it is
  thread 0 of every bench) only when `--pin` is given;
  `--no-pin-cal` and `pin.rs`'s save/restore pair are deleted;
  the Setup cell is renamed `warm pin` -> `main pin`, naming
  main's placement for warm and run both

### Report shape

Normal output carries one warmup line: letter plus **settle
time**, a real machine characteristic. `-v` shows the complete
warmup picture, the per-probe table with the ramp's shape. The
qualification selftest reading a table of settle times across
respawns is a better observable than a table of blended
letters.

### Acceptance test

`tests/qualify_environment.rs` (landed 0.23.0-1, simplified to
one loop 2026-07-28): `#[ignore]`d integration test spawning
the real binary per run (`CARGO_BIN_EXE`), one loop of 10
back-to-back runs. The loop's own load provokes the transition;
verdict: median >= B and zero runs with drift/repeat at D/F.
Reproduced failing 2026-07-27 on the 3900X (repeat F on the
climb run); `IIAC_PERF_BIN` pins a saved failing build. Part of
this cycle's close-out validation.

### As built at -2: one window, and what it showed

The -2 rung's as-built decisions, where they refine the design
above:

- **the exit window replaced the fixed 300 ms tail**
  (`WARMUP_TAIL_SECONDS` deleted): the graded warmup tail is now
  exactly the window the exit condition tested
  (`RunOutput::warm_tail`), so the printed letter is the letter
  the exit saw. The long tail's job (catch a ramp inside a fixed
  budget) is gone because the exit keeps warming until the
  window is clean
- provisional constants, sized on the 3900X and flagged for the
  7600x pass: pass minimums 8 steps / 1 ms, window minimums 8
  probes / 50 ms, cap 400 ms. 50 ms is governor-transition
  scale, not full-ramp scale; the dwell a timing window cannot
  see at any length is the clock rung's job
- the warm stretch's cost moved from ~4 ms fixed to ~51 ms
  settled (window span + probe overhead); the exit is condition
  driven, so a disturbed box pays up to the cap instead
- the settle cell now answers by exit verdict: a settled exit
  reports gauge::settle's time, a cap exit prints "not settled"
  (the exit's own finding), a window that never formed prints
  "uncertified" (parsed as blank by qualify's `parse_settle`)
- sizing reads the exit window's best pass (min per-step cost),
  and the estimate phase is deleted; the cap deadlines every
  adaptive pass, which also retires the estimate-phase hang
  (bugs.md #1's deadline half; the pool-size guard half remains
  open)
- observed on the 3900X `all` sweep: two benches printed
  `A` + `not settled`, a window that grades A while an 8-median
  excursion left the 1% settle band inside it (the bistable
  flicker at grade-invisible scale). Truthful but odd on one
  line; review whether settle's band should align with the
  window grade's thresholds

### Deferred: start-vs-end differential QC

Repeat the warmup pass at run end and compare floors (never
absolute, never subtracted) as a "did the box shift" check.
Deferred at pickup (2026-08-02): the run already carries seam
probes across its whole span and the run grade's drift/step
signals answer the same question; revisit if batch data shows
frame shifts need separating from per-iteration shifts. The
N-sweep slope/intercept decomposition likewise stays an idea.

# References

[1]: /notes/chores/chores-05.md#the-7600x-stopped-passing-and-the-grade-is-why
[2]: /notes/chores/chores-05.md#the-clock-behind-the-anomaly
