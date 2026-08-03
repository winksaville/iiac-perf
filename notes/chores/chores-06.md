# Chores 06

Continuation of [chores-05](chores-05.md). Records landed work; conventions in
[agent-data/notes.md](../../agent-data/notes.md#chores-conventions) and
[cycle-protocol.md](../cycle-protocol.md#chores-sections).

## Table of Contents

- [feat: dynamic warmup](#feat-dynamic-warmup)

## feat: dynamic warmup

- [[N]] 0.24.0-0 feat: dynamic warmup opening
- [[N]] 0.24.0-1 refactor: one parameterized warm loop
- [[N]] 0.24.0-2 feat: warm until the trailing window grades A
- [[N]] 0.24.0-3 feat: warm where the bench runs
- [[N]] 0.24.0-4 feat: read the clock during warmup
- [[N]] 0.24.0-5 feat: settle follows the warm window grade
- [[N]] 0.24.0-6 feat: configurable warm cap
- [[N]] 0.24.0 feat: dynamic warmup

The 0.24.0 cycle: replace the fixed `WARMUP = 10_000` step count in `harness.rs` with
warm-until-stable. A fixed count's wall-clock scales with step cost, so the fastest benches warm
~10 us against frequency-governor ramps of tens-to-hundreds of ms and `pick_inner` sizes mid-ramp
(the 7600x F diagnosis,
[Replanning II](chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run)), and a timing-only
"did it end settled" test can issue a vacuous A while the box dwells one P-state below the top
(measured 2026-07-29 [[1]]). The design accumulated in the Todo entry over 2026-07-27 to
2026-08-01; the subsections below are its record, plus the decisions made at pickup (2026-08-02).
First cycle run on a topic bookmark (`dynamic-warmup`).

### One warm loop, three policies

The end state, decided 2026-08-01: one parameterized warm loop. Step the bench, probe
periodically, stop when the exit condition holds. The harness's three warms become policies over
that one mechanism:

- the per-run warmup exits when the trailing window grades A, or at the cap
- the process warm (`process_warm`) exits on the `--settle-time` budget
- the block warm (`run_blocked`'s 2 ms spin) is the same loop with a fixed-time exit and probing
  disabled

`process_warm` and `warmup_and_probe` already share one probe series, prober and time origin;
this completes the fusion instead of adding a fourth variant.

Terminology: the warmup unit is a **warmup pass** (the 0.22.0-4 "loop-only passes" sense), a
short, unrecorded, timed burst of bench steps yielding one floor. Not a "probe": `TProbe` is the
measurement instrument, and the "micro-probe" is the 0.23.0 cycle's ~1 ms timer-pair frame
measurement.

### The exit condition: grade the trailing window

Rather than K agreeing floors, grade a *sliding window* over the warmup probe series and warm
until the trailing window reads A, or the cap (design 2026-07-28). The exit condition and the
warmup letter become one computation:

- exit on A means the run started post-ramp by construction
- hitting the cap reports whatever the window actually scored, the "run started unstable" signal,
  not a silent proceed
- window length takes the same "minimum count or minimum wall time, whichever is larger" shape as
  pass length: count because the split detector needs 4 points a side, wall time because the ramp
  is a ms-scale phenomenon
- signals: spread, drift, step; `interference` is the weak one [[2]]
- the minimum wall span is load-bearing: a window can be far shorter than what it certifies
  (`min-now`'s 16 warmup probes span ~17 us against a transition arriving at ~800 ms), so
  agreement alone certifies nothing

Floors, not means, so one preemption doesn't fake (in)stability; a warm box exits
near-immediately.

Convergence is agreement, not direction. The 3900X is bistable (2026-07-27 `calibrate` runs):
sustained rapid repetition climbs it into a fast state (~0.445 ns/iter), low-duty isolated runs
sit at ~0.489 (B), ~9% apart, and transitions straddle windows in *both* directions; a fixed
warmup absorbs only transitions shorter than itself, whatever their sign.

The hard cap sits at governor scale (a few hundred ms; exact constant measured on both boxes
during the rung). The cap doubles as the estimate-phase deadline the `--pin` guard Todo wants, so
slow and non-converging benches share one diagnostic exit.

### Sizing fusion

The warmup pass *is* the step-cost sizing pass: the converged floor is the sizing input, so
sizing is post-ramp by construction and convergence is tested on the number actually consumed.
The `estimate_step_cost` phase folds into the warm loop; the micro-probe supplies the frame
input, run after convergence.

Slow steps (`inner` -> 1 territory): pass length adapts (minimum step count or minimum wall time,
whichever is larger) so a floor is never one sample. The cap exit then distinguishes "floors
disagreed" (unstable, gauge signal) from "too slow to certify" (proceed, label the run
uncertified: at `inner = 1` sizing can't be wrong and framing is negligible, so the stakes are
low there).

### Read the clock, not just the timing

Steadiness cannot tell "settled at the top" from "dwelling at an intermediate P-state", because a
dwell *is* steady. Measured 2026-07-29 on the 7600x [[1]]: the trailing window graded A while the
box held 4841 MHz for ~0.75 s, then stepped +12.4% inside the run. This was the strongest
argument for making the clock reading part of the exit condition rather than an optional extra,
and the rung is in scope (decided 2026-08-02).

- delivered frequency is an unprivileged sysfs read on both AMD boxes
  (`cpufreq/cpuinfo_avg_freq`); the ramp is ~150-200 ms
- gate on **clock stability under load**, never on a fraction of `cpuinfo_max_freq`: a threshold
  would need tuning between 96.1% (3900X) and 99.7% (7600x) sustained, and a thermally-limited
  laptop plateaus lower still while that plateau is its honest clock
- optional by construction: `cpuinfo_avg_freq` is amd-pstate-specific and some drivers'
  `scaling_cur_freq` reports requested rather than delivered, so read where present and fall back
  to timing-only
- report the ratio, do not grade on it

`qualify-environment`'s verdict is not usable as a gate until this cycle lands: it reads NOT
QUALIFIED on any amd-pstate-epp box that dwells then boosts, which is to say on a healthy idle
machine [[1]]. Fixing the exit condition fixes the selftest at the same time, since its
observable is this grade.

As built at -4: `src/freq.rs` reads `cpuinfo_avg_freq` on the calling thread's current CPU
(`sched_getcpu`), one sample per warmup probe, kept parallel to the probe series across the
process-warm handoff.

- the exit gains a second gate: timing-A *and* clock held within 1% (`FREQ_STABLE_TOL`) across
  the exit window; a timing-steady window with a moving clock classifies Unstable (the dwell
  case, unit-tested against the 7600x numbers)
- anything short of clean same-CPU readings falls back to timing-only: file absent, read failure,
  or an unpinned main migrating mid-window (samples carry their CPU id)
- the ratio prints on the `-v` warmup summary line (`clock 4093/4674 MHz (87.6%)` measured on the
  3900X, whose honest sustained clock is ~87% of `cpuinfo_max_freq`: live confirmation that a
  fraction-of-max threshold would misfire and stability-under-load is the right gate)
- review point: the ratio is `-v`-only for now; the design said "report it", and the normal
  grade block's columns are parsed positionally by qualify, so adding it there was deferred to
  review

### Placement: warm where the bench runs

Decided 2026-08-02: the warm follows the bench's pin. Warm on `pin[0]` when `--pin` is set, else
wherever the scheduler has main (a busy thread stays put, and the warm state lands on the core
that measures). The CPU0-default tick-rate warm pin and `--no-pin-cal` are deleted rather than
justified.

- CPU0 is measurably the kernel's busiest core on the 3900X (2026-07-29, cumulative per-CPU
  interrupts: 4.1M `LOC` against ~0.6M on CPU11/CPU23, 6.2M `CAL` against ~0.6M, 4-6x the `RES`,
  3x the `TLB`; no `irqbalance`). CPU0 is the boot CPU and the `nohz_full` housekeeping CPU, so
  this is expected rather than a quirk of this box
- the pin did not matter for the tick-rate read (a ratio of TSC ticks to monotonic ns over
  ~10 ms: interruptions inflate both sides and cancel, ~8e-7 spread across cores), so the current
  default was harmless, not correct; it would have mattered here, where warmup becomes a real
  timing phase converging on per-core frequency state
- the rejected alternative, a topology-aware "not the boot CPU, and a full-frequency core" pin,
  adds machinery this cycle does not need; it stays with the topology Todo. "Use the last core"
  folklore was already rejected there (hybrid parts put E-cores at high indices)
- as built at -3: main pins to `pin[0]` (and stays; it is thread 0 of every bench) only when
  `--pin` is given; `--no-pin-cal` and `pin.rs`'s save/restore pair are deleted; the Setup cell
  is renamed `warm pin` -> `main pin`, naming main's placement for warm and run both

### Report shape

Normal output carries one warmup line: letter plus **settle time**, a real machine
characteristic. `-v` shows the complete warmup picture, the per-probe table with the ramp's
shape. The qualification selftest reading a table of settle times across respawns is a better
observable than a table of blended letters.

### Acceptance test

`tests/qualify_environment.rs` (landed 0.23.0-1, simplified to one loop 2026-07-28):
`#[ignore]`d integration test spawning the real binary per run (`CARGO_BIN_EXE`), one loop of 10
back-to-back runs. The loop's own load provokes the transition; verdict: median >= B and zero
runs with drift/repeat at D/F. Reproduced failing 2026-07-27 on the 3900X (repeat F on the climb
run); `IIAC_PERF_BIN` pins a saved failing build. Part of this cycle's close-out validation.

### As built at -2: one window, and what it showed

The -2 rung's as-built decisions, where they refine the design above:

- **the exit window replaced the fixed 300 ms tail** (`WARMUP_TAIL_SECONDS` deleted): the graded
  warmup tail is now exactly the window the exit condition tested (`RunOutput::warm_tail`), so
  the printed letter is the letter the exit saw. The long tail's job (catch a ramp inside a fixed
  budget) is gone because the exit keeps warming until the window is clean
- provisional constants, sized on the 3900X and flagged for the 7600x pass: pass minimums 8
  steps / 1 ms, window minimums 8 probes / 50 ms, cap 400 ms. 50 ms is governor-transition
  scale, not full-ramp scale; the dwell a timing window cannot see at any length is the clock
  rung's job
- the warm stretch's cost moved from ~4 ms fixed to ~51 ms settled (window span + probe
  overhead); the exit is condition driven, so a disturbed box pays up to the cap instead
- the settle cell now answers by exit verdict: a settled exit reports gauge::settle's time, a cap
  exit prints "not settled" (the exit's own finding), a window that never formed prints
  "uncertified" (parsed as blank by qualify's `parse_settle`)
- sizing reads the exit window's best pass (min per-step cost), and the estimate phase is
  deleted; the cap deadlines every adaptive pass, which also retires the estimate-phase hang
  (bugs.md #1's deadline half; the pool-size guard half remains open)
- observed on the 3900X `all` sweep: two benches printed `A` + `not settled`, a window that
  grades A while an 8-median excursion left the 1% settle band inside it (the bistable flicker at
  grade-invisible scale). Truthful but odd on one line; review whether settle's band should align
  with the window grade's thresholds
- resolved at -5 (wink, 2026-08-02): aligned. Settle became the earliest suffix of the warm
  stretch that grades A, scanned front-to-back and never shorter than the exit window, so the
  letter and the settle cell are one computation and cannot disagree; `SETTLE_TOL`,
  `SETTLE_WINDOW` and the forward-median machinery (`window_floors`) are deleted. A settled
  exit always finds a time ("not settled" reaches the report only on a cap exit), and the
  post-change sweep read A rows with settle 0.01 to 0.44 s and no contradictions

### Acceptance run after -4 (3900X, 2026-08-02)

`qualify_environment` run once with all four rungs in, as review data for close-out (the 7600x,
the box the dwell was measured on, still needs its pass):

- the warmup column is what the cycle promised: 9 A + 1 B, no vacuous letters, settle times
  honest (0.01 s warm-box, 0.74 to 1.41 s when the box had relaxed between runs, two runs "not
  settled" at the cap while the box was still flickering)
- the verdict still reads NOT QUALIFIED, now for a run-side reason: mid-run transitions (env
  bench drift/step D/F on 3 of 10) from the bistable trait, which warmup cannot prevent and the
  report truthfully attributes
- close-out questions this raises: is the verdict's "transition-degraded" rule right for a box
  whose trait this is, and should the 400 ms cap sit above the 3900X's ~1 s relaxation re-ramp
  (runs that settled at 1.2 s did so inside the respawned process warm, not the capped per-run
  stretch)
- resolved at -6 (wink, 2026-08-02): the cap default rises to 1.5 s and becomes `--warm-cap` /
  config `warm_cap` (CLI > config > built-in, the `--settle-time` pattern; zero or more, 0 caps
  immediately). The post-change `all` sweep on the 3900X read zero "not settled" rows where the
  0.4 s cap produced two: the relaxation re-ramp is now absorbable. With `--warm-cap 0
  --settle-time 0` no warmup probes exist at all, so the warmup row is absent (no certificate),
  which is distinct from "uncertified" (probes exist, no valid window). The verdict-rule
  question stays open for the qualification redesign
- also at -6, warm visibility (wink, same day): the Setup banner gains a `warm budget` cell
  (`settle 1.5s once + cap 1.5s per run`, the resolved budgets), and each report's header
  bracket gains `warm=used/budget`, this run's total warm spend over its total allowance
  (first run `warm=1.51/3.0s`, settle + cap; later runs `warm=0.13/1.5s`, cap alone). Setup
  prints before any run, so it carries only the budgets. First cut showed the capped stretch
  alone and read `warm=0.00/1.5s` on every settled-by-process-warm first run (wink caught it):
  truthful about the cap, useless about the cost

The 7600x pass (wink, same day, installed binary):

- `min-now` reads straight A's with settle 0.77 s: the warm loop rode through the 4841 MHz dwell
  and exited after the ~0.8 s boost
- trimmed stdev 0.1 ns; the vacuous-A defect is closed on the box it was measured on
- the cross-respawn `qualify-environment` verdict there is still to be run

Duty cycle selects the bistable state (wink, same day, 3900X unpinned):

- plain `-d 5` and `--blocks 100` climb into the fast state (~21.8 ns) and grade F when the flip
  lands mid-run
- `--blocks 1000` (5 ms bursts between 1-10 ms sleeps) holds the slow state (24.0 ns) for 13 s
  and reads straight A's with CI95 0.0 ns
- grade A certifies internal consistency of the state the run held, not a canonical number; A/B
  wants matched duty cycle
- feeds the "Report interpretation guide" Todo's worked examples
- strengthens the seam-clock idea: sample `cpuinfo_avg_freq` at batch seams so a mid-run step
  gets a "clock moved" attribution

The constant-clock control (wink, same day, 7600x, `--decimals 3`): at a held clock, mode does
not move the number, and block count is a tradeoff, not a dial to max out.

- plain vs `--blocks 2`: trimmed means identical to the third decimal (16.196 ns both), full
  means identical (16.236 ns), band values byte-identical; the 3900X mode divergence was pure
  DVFS state
- `--blocks 2` read CI95 0.003 / LSC 0.001 ns, but from one degree of freedom (t = 12.7, a
  single pair of block means): real agreement, fragile interval; quote LSCs from tens of blocks
- `--blocks 1000` on the same box: mean +1.6% (16.503), trimmed stdev 0.110 -> 0.342 ns, LSC
  8x larger (0.008 ns). We think ~5 ms blocks sit close to every wake, so C-state exit residue
  the 2 ms block warm does not fully re-establish contributes proportionally more and
  between-block variance rises
- the ordering is bench-shape-dependent (wink, same day, 7600x `zcr-with-2t`, a spin-partner
  bench): blocks 2 and 20 agree to 0.002 ns (110.835 / 110.837 trimmed) while 1000 blocks' LSC
  0.022 beats 20's 0.083, the reverse of `min-now`. We think the spinning worker rides through
  main's sleeps, so the box never idles between blocks; wake residue shrinks and replication
  (df 999) wins. The +~1% mean shift at 1000 blocks persists
- the sleep budget selects the state too (wink-requested experiment, same day, 3900X, source
  patched to a fixed 0.5 ms sleep, 1000 blocks): ~67% duty (5 ms measure / ~2.5 ms gap) landed
  in the unstable middle: the box straddled both states (band mass split across 21.8 and
  24.0 ns), a 9.3% step at 5.6 s graded D on both series, and LSC 0.143 ns, 6x worse than the
  1-10 ms sleeps' 0.023. The 1-10 ms budget (~40% duty) holds the slow state; sustained load
  holds the fast one; between them is the flip zone. A `--block-sleep` knob would make this
  explorable without patching source (idea)
  - for change detection, tens of blocks: replication df with per-block cleanliness, the
    tightest defensible LSC; even 1000 blocks' 0.008 ns is 0.05% of the mean
  - for representativeness, high block counts are *more* real, not noisier: real IPC usage is
    bursty (wake, exchange, go quiet), so the +1.6% and the wider tail are the delivered cost
    of a deployment-like duty cycle, which the hot loop's floor number never shows
  - blocks mode still shields the coldest part by design: the 2 ms post-wake warm is
    unrecorded, so true first-call-after-sleep cost never lands in the histogram. A cold-start
    mode that records or separately reports post-wake samples is a natural extension (idea,
    2026-08-02)

### Outcome

What the cycle set out to fix is fixed, measured on the box that motivated it:

- the 7600x reads straight A's with settle 0.77 s: the warm loop rides through the 4841 MHz
  dwell and exits after the ~0.8 s boost the old fixed warmup measured straight through. The
  vacuous-A defect cannot recur by construction: the exit condition, the printed letter, and
  the settle time are one computation over one window
- the warmup certificate is complete on the 3900X too: the close-out acceptance run reads
  10/10 warmup A, settle 0.01 to 1.48 s, zero "not settled" (the 1.5 s cap absorbs every
  relaxation re-ramp the 0.4 s cap truncated). What remains is the mid-run bistable flip (4 of
  10 runs, env-bench D), which no warmup can prevent and the run grade truthfully attributes;
  its consequences moved to Todos: qualification-as-evidence, seam-clock attribution, blocks as
  the first-class mode
- the day's measurement session (duty cycle selects the state; constant-clock control; block
  count picks the question; the 0.5 ms flip zone) is recorded above and feeds the "Report
  interpretation guide" Todo, ranked #1 at close-out
- grew beyond the planned four rungs by two, both wink-driven same-day: settle/grade alignment
  (-5) and the configurable cap with warm visibility (-6)

### Deferred: start-vs-end differential QC

Repeat the warmup pass at run end and compare floors (never absolute, never subtracted) as a "did
the box shift" check. Deferred at pickup (2026-08-02): the run already carries seam probes across
its whole span and the run grade's drift/step signals answer the same question; revisit if batch
data shows frame shifts need separating from per-iteration shifts. The N-sweep slope/intercept
decomposition likewise stays an idea.

# References

[1]: /notes/chores/chores-05.md#the-7600x-stopped-passing-and-the-grade-is-why
[2]: /notes/chores/chores-05.md#the-clock-behind-the-anomaly
