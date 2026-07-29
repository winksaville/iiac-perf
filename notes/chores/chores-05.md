# Chores 05

Continuation of [chores-04](chores-04.md). Records landed work;
conventions in [AGENTS.md](../../AGENTS.md#chores-conventions) and
[cycle-protocol.md](../cycle-protocol.md#chores-sections).

## feat: grade the run from raw batches

Commits: [[1]],[[2]],[[3]],[[4]]

Decided in
[Replanning II](chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run):
the overhead subtraction estimates an ill-defined quantity —
additivity is an approximation on a superscalar CPU, the
constants moved ~10% with frequency regimes, and the correction
cancels in same-harness A/B anyway — while the calibration-time
grade certified a ~1 s window *before* the run: the room, not
the exam. This cycle removes the adjustment machinery and moves
grading onto the run's own time-ordered batch data.

### As-built ladder

- [[1]] 0.23.0-0 `chore: open raw-batch grading cycle`
- [[2]] 0.23.0-1 `feat: micro-probe inner-loop sizing` —
  `pick_inner`'s frame input now comes from a ~1 ms
  micro-probe (low quantile over back-to-back timer pairs)
  instead of `cfg.overhead.frame_call_ns`; sizing no longer
  depends on startup calibration. Also lands
  `tests/settle_anomaly.rs`, the `#[ignore]`d settle-anomaly
  acceptance test for the dynamic-warmup Todo — captured
  while the failing baseline (calibrate letter) still exists
  — and per-signal letters on the environment line
  (`CalGrade::signal_letters`), a display shakedown for the
  -4 gauge: every composite letter now names its cause
- [[3]] 0.23.0-2 `feat: time-ordered batch pipeline` —
  samples flow through `BatchPipeline` (raw buffer, flush on
  65,536 samples or 50 ms, whichever first): per-batch
  summaries (floor, mean, max, census over-floor count,
  timestamps) for the -3 gauge, then bulk-record into the
  histogram, buffer reused; memory bounded. Block mode
  flushes at block boundaries so no batch spans a sleep gap.
  Run outputs folded into `RunOutput` (retiring
  print_report's too-many-arguments debt); header gains
  `batches=N`
- [[4]] 0.23.0-3 `feat: batch-based run gauge` — `gauge.rs`
  grades a run from its batch series and prints a `run` row
  with the letter and all four signals, each carrying its own
  letter, with the composite on its own `overall worst case:`
  line beneath them. Landed beside the calibration grade so the
  two can be read against each other; four signals to
  calibration's six, see
  [Six calibration signals, four run signals](#six-calibration-signals-four-run-signals).
  It reports and never warns — see
  [Grade the run, judge the box](#grade-the-run-judge-the-box).
  Every signal's *definition* changed under measurement — see
  [Signals the data rejected](#signals-the-data-rejected)
- [[N]] 0.23.0-4 `feat: environment grade across the run` —
  `EnvGrade` grades the **box** from a series of micro-probes
  and prints as its own `env` row above the run row. The
  probes measure the apparatus alone, so the letter carries no
  workload character — the separation the run grade's
  "reports, never warns" stance asked for. The single sizing
  probe becomes a series: 16 probes interleaved with warmup
  (the last still feeding `pick_inner`), then one at every
  batch seam, so the series spans the whole run on the batch
  series' own time axis. `--no-env-probe` limits it to the
  warmup stretch. See
  [Grading the box without the bench](#grading-the-box-without-the-bench),
  [Why the probe times groups](#why-the-probe-times-groups),
  [Warmup was the wrong window](#warmup-was-the-wrong-window),
  and
  [What the probe cannot see](#what-the-probe-cannot-see).
  `overall worst case:` is renamed `run worst case:` now that
  two grades print

- [[N]] 0.23.0-5 `feat: split warmup and run environment
  stretches` — the environment series is graded as two
  stretches instead of one: the warmup tail and the probes
  taken while the bench ran, printed as `env warmup:` and
  `env run:` with the composite the worse of the two. Warmup
  is scored on its trailing 8 probes, because absorbing a ramp
  is what warmup is *for*. See
  [Two stretches, one series](#two-stretches-one-series)

### Two stretches, one series

The -4 series ran from warmup through the end of the run and
was graded whole. That is wrong in a way that only shows on a
cold box, which is to say the way -7 is about to make normal.

Warmup exists to absorb the frequency ramp. Grading the whole
series therefore faults warmup for succeeding: a stretch that
starts at 30 ns and settles at 24 ns before the bench begins
reads as a ~20% `step` at the warmup/run boundary, even though
nothing went wrong and the run itself never moved. Today
`calibrate`'s ~1 s of spinning hides this by pre-warming the
box five times over the ~150-200 ms the ramp takes; -7 deletes
that, and the fake step appears.

The fix is to score the stretches apart:

- **`env warmup`** — the *trailing* [`WARMUP_TAIL_PROBES`] (8)
  probes of the warmup stretch. The question is "did it end
  settled", not "was it steady throughout"; the tail is the
  only part that answers it. Eight is the smallest window the
  split detector works in, needing four points a side.
- **`env run`** — the seam probes, whole. "Did it stay
  settled."
- **Composite** — the worse of the two. Starting a measurement
  on a box that had not settled is a real environment problem,
  so warmup counts; the split is there so a reader can see
  which stretch earned the letter.

The claim is unit-tested from both sides:
`warmup_ramp_does_not_fault_a_clean_run` builds a ramping
warmup followed by a flat run, asserts the *blended* grade
invents a step over 10%, then asserts both split stretches
grade A.

This window is also the seam with the "Dynamic startup warmup"
Todo, which turns it into the exit condition — warm until the
trailing window grades A — at which point the stopping rule
and the warmup letter are one computation rather than two
things that have to agree.

### Warmup was the wrong window

The rung was planned as "environment grade from warmup", and
built that way first. On a quiet 3900X it read A beside a run
grade of A — which prompted the right question, because that
box is known bistable. Three reasons, in ascending order of
how much they matter:

- `calibrate` still runs ~1 s of hard spinning before the
  bench loop (`main.rs`), so by the time warmup probes run the
  governor is already at the top. The grade could not see a
  cold-start ramp because none was left. This one dissolves at
  -6.
- The window was ~17 ms — 16 probes of ~1 ms plus 10,000
  min-now steps. Against governor ramps of tens to hundreds of
  ms that is short.
- Most of all it was the *wrong* window. Grading the first
  17 ms and captioning a 5 s run with the result is this
  cycle's own critique of the calibration grade — certifying
  the room, not the exam — reproduced in miniature.

The fix reuses the batch pipeline's existing shape rather than
inventing one: `flush()` already stops the bench to summarize
a batch, so a probe in that seam costs a fraction of a gap
that exists regardless (the seam already runs a
`select_nth_unstable` over up to 65,536 values plus 65,536
histogram records, ~1-2 ms). Each probe shrank to 128 groups,
~256 µs, since ~120 of them now cover a 5 s run instead of 16
covering its prelude.

The payoff is more than a longer window: every batch now
carries a bench floor and a box floor stamped at the same
instant on the same clock, so movement can be *attributed*.
The first run after the change caught the 3900X mid-shift —
`env step 9.88% @2.138s` beside `run step 9.98% @2.1s`. Same
instant, same magnitude: the box moved and the bench floor
moved with it. An `env` A beside a `run` D would have said the
opposite, that the workload did it. Neither grade can make
that call alone.

### What the probe cannot see

The probe measures the box only while the measuring thread is
running, so time spent descheduled is largely invisible to it
— a ~256 µs probe usually fits inside one scheduling quantum
and is never preempted at all. Measured on the 3900X with the
bench pinned to core 0:

- One spinner `taskset` to the same core: `spread` 0.31% ->
  2.02% (B), `drift` A -> D, `step` A -> D, but
  `interference` only 0.01% -> 0.04%.
- Three spinners on the same core: every signal back to A. Not
  a quieter box — a bench thread that holds its quantum, so
  each probe completes cleanly and the contention lands
  between probes.
- Six unpinned spinners: nothing at all, because with 24 CPUs
  the scheduler left core 0 alone. Affinity does not reserve a
  core, but an idle machine effectively does.

So this grade detects frequency and state movement well and
preemption poorly, and `interference` catches only the rare
large intrusion that happens to land inside a probe. Its
thresholds are set one order of magnitude above the measured
quiet baseline because there is no good upper anchor to
calibrate against; `spread`, `drift` and `step` carry the
detection. We think closing the preemption gap needs a
different instrument — something that reads a counter of
involuntary context switches rather than sampling elapsed
time — which is a separate question from this rung.

### The seam probe's cost, measured

Probing at seams perturbs the bench. Interleaved A/B on the
3900X, `-d 3`, pinned, trimmed mean (`--no-env-probe` off vs
on), five pairs each:

- `zcr-with-2t` (spinning, two threads): 133.18 ns off vs
  134.32 ns on — **+1.14 ns, +0.86%**, and on was slower in
  all five pairs, so it is a bias rather than noise.
- `min-now` (one thread): 22.0 ns off vs 22.1 ns on, at the
  edge of resolution.

We think the difference between the two is the worker thread:
it keeps spinning on an empty ring through the probe, so the
first handoff after the probe starts from a different queue
state. A single-threaded bench just pauses.

Default is **on** anyway. The bias is common-mode in the
same-harness A/B this harness exists for, 0.86% is small
against what the grade catches (a 9% state shift, invisible
without it), and `--no-env-probe` is there for anyone who
wants absolute numbers untouched. Two of the five
`--no-env-probe` `min-now` runs happened to land in the
3900X's slow state at 24 ns against the others' 22 ns — a 9%
excursion that the flag they were using would have hidden.

### The clock behind the anomaly

The 3900X's "bistable" ~9% shift and the 7600x's calibration
`F` have one mechanism, and it is the core clock. Both boxes
run `amd-pstate-epp`, which exposes delivered frequency
through unprivileged sysfs
(`cpufreq/cpuinfo_avg_freq`, APERF/MPERF-derived), so this is
measured rather than inferred.

**Measured, 2026-07-28.** Sampling `cpu0`'s frequency while a
pinned `min-now` ran:

- **3900X** — clock climbed to 4.44-4.52 GHz under sustained
  load against a 4.674 GHz max. In the run where the
  transition landed inside the measurement window, the climb
  was **+9.75%** and the grades read `env step 8.99% @1.003s`
  beside `run step 9.07% @0.4s`. A clock change and two
  independently-measured steps, agreeing within a percent.
- **7600x** — climbed to 5.440 GHz against a 5.457 GHz max,
  **+12.37%**, and then held to within 0.002% for six
  seconds. The same invocation's calibration reported
  "independent slope estimates differ **12.4%**" and
  "constants differ **12.5%** between attempts". The
  calibration WARNINGs were reading the clock ramp to two
  significant figures.
- **Ramp shape** — at 50 ms resolution the climb takes
  ~150-200 ms, beginning when calibration pins core 0 and
  starts hammering it, roughly 1.3 s after launch (process
  startup plus the `systemd-inhibit` re-exec).

**What this does not say.** Whether the transition lands
inside a measurement window depends on the box's recent load
history, not on a schedule: a later 3900X run had the climb
complete before warmup and graded `run` A on every signal. So
the mechanism is established; its *timing* remains
duty-cycle-dependent, which is the behaviour the earlier notes
described without a cause.

**A correction, and a caveat for any future reader.** An
earlier draft of this section reported both boxes "idling" at
~88% of max. That was an artifact of the measurement:
APERF/MPERF only accumulate in C0, so reading a core's
frequency wakes it, and polling at 4-20 Hz held the observed
core at ~4.09 GHz. A single cold read after 6 s quiet, taken
from a reader pinned to another core, gives 3.588 GHz, and a
genuinely quiet 3900X sits at 1.746 GHz — exactly
`amd_pstate_lowest_nonlinear_freq`, and the value that appears
in the decay tail after a run ends. The floor is 563 MHz. The
quiet-to-sustained span is therefore a factor of 2.6, not 9%.

The lesson generalizes to any frequency reading this tool
might take: reading the core it is *already running on* is the
sound case, because that core is in C0 by construction.
Polling some other core and calling the result idle is not a
measurement.

### The clock as a warmup criterion (design, unmeasured)

The obvious use is the warmup exit condition, and it solves a
problem no timing-based criterion can. A steadiness test
cannot distinguish "settled at the top" from "dwelling at an
intermediate P-state", because a dwell is *steady*. A clock
reading distinguishes them directly.

We think the right gate is **clock stability under load**, not
a fraction of max:

- A threshold on `cpuinfo_max_freq` would need tuning between
  the two boxes' sustained fractions (96.1% on the 3900X,
  99.7% on the 7600x) against their pre-load readings, and a
  thermally-limited laptop would plateau lower still and never
  reach it — while that plateau *is* its honest sustained
  clock.
- Stability under load needs no tuned constant. Warmup is
  itself sustained load, so the climb happens during warmup if
  warmup lasts long enough; the ~200 ms measured here is the
  scale.
- Report the ratio for information; do not grade on it. The
  letter should stay a statement about measured time, with the
  clock explaining it.

Portability caveat: `cpuinfo_avg_freq` is amd-pstate-specific,
and some drivers' `scaling_cur_freq` returns the requested
rather than the delivered frequency. Any use must be optional
— read it where present, fall back to timing-only otherwise.
An Intel laptop is the next box to check, and the interesting
one: a thermally-limited machine should show the opposite
shape, starting fast and decaying under sustained load, which
is the direction neither desktop can produce.

### Grading the box without the bench

The run grade describes the run it printed, and a run's
steadiness is largely its workload's character. That is the
right answer for a histogram's caption and the wrong one for
the question "is this machine fit to measure on" — the
question the retired `calibrate` grade was really answering,
and the one -6 removes an answer to.

Warmup is the only window a run has where no workload has
entered the numbers yet, so that is where the environment
certificate is measured. The probe times timer pairs and
nothing else, which is what makes the resulting letter a
statement about the machine.

Signals against the run grade's four:

- `interference`, `drift`, `step` — the same questions with
  the same definitions, over a different population. `drift`
  is the one the cycle needs most: it is the frequency-ramp
  detector the 7600x F diagnosis and the 3900X bistability
  both call for.
- `spread` — new here: how wide a probe's bulk sits above its
  own floor. There is no run-side analog worth having,
  because a bench's spread is mostly its workload (park /
  unpark bimodality is a fact about `mpsc`, not the box)
  while a timer pair has no character of its own.
- `bursts` — dropped. The contamination it would count is
  already counted by `interference`, and a hot-probe fraction
  adds nothing a spread and a census do not already say.

Thresholds are separate from the run grade's even where the
numbers currently match, because the two grade different
populations and should be free to diverge as measurement
says they should — as `interference`'s since have (see
[What the probe cannot see](#what-the-probe-cannot-see)).

One caveat on "without the bench", added once probes moved to
the seams: the *warmup* stretch is measured with nothing but
warmup steps behind it, but a seam probe shares the box with
the bench — on a 2t bench, with its still-spinning worker.
That is a truer picture of the environment the run actually
had and a slightly less pure measure of the machine alone.
The warmup stretch remains the clean one.

### Why the probe times groups

The probe's unit of measurement changed from a single timer
pair to a *group* of 64 pairs, timed by one outer `Instant`
pair and divided down.

The reason is resolution. The timer reads integer
nanoseconds, so a single ~25 ns pair is quantized to ~4% —
coarser than the ~9% bistable shift the environment grade
exists to see, which would have left `spread` reporting
quantization and `drift` unable to resolve anything smaller
than a P-state jump. A 64-pair group totals ~1.6 µs, so the
same 1 ns quantum is ~0.06% of the value and the per-pair
figure lands on a ~15 ps lattice. Measured on the 3900X the
difference is visible directly: `spread` reads ~0.35% on a
quiet box, well clear of a floor that single-pair timing
would have pinned near 4%.

The sizing input shifts only in spirit — from "the quietest
single pair" to "the mean pair of the quietest group". Both
are an uncontended frame cost, which is all `pick_inner`
documents itself as needing.

Grouping does cost the *census* its meaning, which is why the
census is counted per pair instead. One 3 µs intrusion is one
disturbed pair in 8,192 (0.012%), but it lifts its whole
group's mean past the cut and reads as one group in 128
(0.78%) — a 64x over-count, and the mirror of the same
problem in the other direction, since an intrusion under
~800 ns vanishes into a group mean entirely. Each pair's own
reading is already in hand inside the group loop, and a
census threshold sits far above the 1 ns quantum, so counting
pairs costs nothing and asks the same question the run
grade's census asks of a sample.

### Six calibration signals, four run signals

Both grades score each signal 0–4 by counting how many of its
four ascending cutoffs it crosses, and take the composite as the
maximum — the worst signal wins outright. The run grade carries
four of calibration's six:

- `disturbed` → `interference`, the census rate, rebased on the
  batch's own floor.
- `dirty windows` → `bursts`, window becoming batch.
- `drift` → `drift`, unchanged in spirit.
- `repeat` → `step`. The one substantive translation: `repeat`
  compares constants between two clean calibration attempts, a
  transition detector at attempt-to-attempt scale where `drift`
  is the same detector inside one window (the 2026-07-27
  settle-test observation). A single run has no second attempt,
  so the equivalent question within one run is whether the floor
  shifted partway through — the split detector.
- `resid` and `cross` have **no run-side analog** and none was
  invented. Both grade how well a *fit* holds: the worst residual
  of a ladder point against the Theil-Sen line, and the loop-only
  slope against the dithered two-point fit. They exist because
  calibration fits a line through a multi-N ladder. A bench run
  fits nothing, so a run-side version would have required
  inventing the fit first.

The composite prints on its own labelled line beneath the signals
rather than as a letter in front of them:

```
  run:                 interference 0.02% A, bursts 18% A, drift 9.09% D, step 13.05% @1.0s F
  overall worst case:  F
```

The shape makes the rule self-evident — the overall letter is
always one of the letters directly above it, and a reader can see
which signal earned it without consulting anything. The -4
environment grade adopts the same shape.

### Grade the run, judge the box

The gauge first printed `WARNING` lines for any signal at D or
worse, inheriting the calibration grade's shape. Two boxes' worth
of data says that was the wrong shape, for a reason that goes to
what this application is for.

The purpose is measuring performance differences as code changes,
and comparing benches against each other. Guidance on how quiet
the box is comes along as a side benefit. So the report's job is
a histogram faithful to what was measured, and a warning is a
claim that something is *wrong* with it.

A run's steadiness is mostly the workload's character. A
multi-threaded bench carries OS involvement in its own numbers —
scheduling, placement, park/unpark — and a blocking round-trip is
genuinely less steady than a spinning one. `mpsc-2t` reading F
while `mpsc-2t-spin` reads A, same box and same second, is a true
description of two different workloads, not a fault in either.
Warning about it would train the reader to ignore the letter on
exactly the benches where it carries the most information.

So the run grade reports: the letter and its four signals, no
`WARNING`, no advice, no cause named. `warn_invalid` keeps its
original job — stats *invalidated* by a suspend or a histogram
clamp, which really are broken.

That leaves the environment question, which the run grade cannot
answer: a signal computed from measurement-phase samples mixes
box and workload, and no post-hoc filter separates them. The
separator is the *instrument*, not the window: the micro-probe
touches no bench code, so it measures the box wherever it runs.
The environment grade is taken from a probe series — through
warmup and then at every batch seam — gets its own signals and
letter, and prints beside the run grade. That grade is a verdict
on the box rather than on the bench, which makes it the one that
could reasonably warn later.
It became rung -4, ahead of the selftest and the `calibrate`
deletion, because it is the certificate `calibrate` currently
provides and build-then-demolish says the replacement lands
first.

A discriminator for telling a machine transition from a bimodal
workload was drafted as a Todo and then dropped: it existed to
decide what a warning should *say*, and with no warning to write
it had no job left. The -4 seam probes turn out to supply one
anyway, for free — the two series share a time axis, so a `step`
in both at the same instant is the machine and a `step` in only
the run is the workload (see
[Warmup was the wrong window](#warmup-was-the-wrong-window)).

### Signals the data rejected

Three of the four signals were first written as the obvious
relocation of a calibration self-check and had to be redefined
once run data went through them. The pattern each time: a
statistic that is sound on the calibration's tight synthetic
loop is meaningless on a real bench's distribution.

- **Floor movement, min vs quantile.** The first cut compared
  adjacent batches' raw minima. On a quiet 3900X at inner=10
  those minima flipped between 22.0 and 23.0 ns batch to batch —
  a 4.5% "transition" on a run with no state change, grading
  every quiet run D/F. The left edge of the distribution is
  sparse; the p10 of 65,536 samples is not, and it sat on one
  value run-wide. `BatchSummary` now carries both: `floor_ps`
  for the record, `floor_q_ps` for every judgment.
- **Transition detection, adjacent pairs vs split points.** Even
  on the robust floor, an adjacent-pair maximum fires on one hot
  batch out of a hundred — which is a burst, not a transition.
  The detector now scores every interior split of the run on the
  medians of its two sides, so a transient moves nothing, and
  ranks candidates by change x split balance so the reported
  time lands at the transition rather than at the first of the
  ties that plateau around it.
- **Census cut, min vs quantile (again).** The per-batch
  over-floor cut (`max(1.5x floor, floor + 50 ns)`) was built on
  the raw min. On mpsc-2t, whose distribution has a fast path
  near 0.9 µs against a 6.5 µs floor, batches whose min happened
  to land on it counted 99.9% of their samples "over floor" and
  the rest counted 1%. Rebuilt on `floor_q_ps` it reads a few
  percent, as intended.
- **Burst reference, quietest vs typical.** A batch was "hot"
  above the *quietest* batch's mean — an extreme, against which
  any bench with real spread reads ~100% hot (mpsc-2t: 98%).
  Against the run's median batch mean the same run reads 33%.

### Two boxes, two failure modes

The thresholds were checked on both machines. The design
constraint they answer to: a quiet release-build box should
essentially never leave A/B, or a false alarm every third run
destroys trust in the warning.

On the **quiet 7600x** (2026-07-28, built and run on the box, not
a copied binary), `iiac-perf all -d 2` graded A on 12 of 16
benches — every single-threaded bench and every spinning one.
Ten back-to-back `min-now -d 1` runs at zero gap graded **A ten
times**, `drift` and `step` both 0.00% on each, `interference`
steady at 0.02–0.03%. That is the same cadence
[settle_anomaly.rs](../../tests/settle_anomaly.rs) runs, whose
observable is the *calibrate* letter — it read nine A and one B
(`repeat ±0.29 ns`) on the same box in the same session, so the
gauge is at least as clean as the check it will replace.

The exceptions were the **blocking** mpsc round-trips, `mpsc-2t`
and `probe-mpsc-2t`, at `step` F (12–17% floor shift), plus
`ice-ps-2t` and `zcr-mpsc-2t` at C. Pinning to a core pair did
not clear them — six pinned runs still produced three F's —
while `mpsc-2t-spin`, the same round-trip spinning instead of
parking, graded A. We think the blocking path's floor is
genuinely bimodal (a hot handoff versus an actual park/unpark),
which is a property of the bench, not the box.

That pair is what settled the reports-never-warns rule above.
The warnings had asserted "the machine changed state mid-run" —
wrong on exactly these benches, where the movement is the
workload's. Two intermediate versions were tried and discarded
first: deleting the cause outright (which left a reader with a
percentage and nothing to do with it), then hedging the cause and
naming a check. Both still framed the letter as a fault. It
isn't one, and the fix was to stop warning rather than to keep
rewording the warning.

### What the gauge says about the 3900X

Here the movement *was* the machine: unpinned 2-thread benches
graded D on floor movement and moved to A/C when pinned to a core
pair, with nothing else changed. We think that is placement
instability being reported honestly — an unpinned 2-thread run on
a 3900X mixes core placements, and the floor moves when it does.
The contrast with the 7600x's blocking-mpsc F's, which pinning
did *not* clear, is what makes the two cases distinguishable at
all.

On the same box, min-now runs grade A on some invocations and D
(step ~8.7%, a 22 → 24 ns floor shift) on others, at both 1 s and
3 s budgets. That is the settle anomaly
([settle_anomaly.rs](../../tests/settle_anomaly.rs)) showing up
in the run rather than in calibration. Chasing which bench the
selftest should read to see that reliably is what exposed the
deeper problem: any answer would have been a workload chosen to
approximate a workload-independent question. The -4 environment
grade answers it directly, from the warmup micro-probe, so the
selftest reads that and needs no bench at all.
The thresholds were left where they are
rather than widened to absorb it: the box really is bistable, and
the dynamic-warmup Todo is the fix.

# References

[1]: https://github.com/winksaville/iiac-perf/commit/621c5c97dbe1 "621c5c97dbe1418fdcb99db6080eecde40891491"
[2]: https://github.com/winksaville/iiac-perf/commit/769067779b20 "769067779b205d60d34961c841df671e0aefe0d9"
[3]: https://github.com/winksaville/iiac-perf/commit/f53644288058 "f53644288058d66350da3553eb2759e270b3d80a"
[4]: https://github.com/winksaville/iiac-perf/commit/4ce786ff7168 "4ce786ff7168efd8dc84c0afee4bbcdb71220a5a"
