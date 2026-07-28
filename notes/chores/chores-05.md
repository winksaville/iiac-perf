# Chores 05

Continuation of [chores-04](chores-04.md). Records landed work;
conventions in [AGENTS.md](../../AGENTS.md#chores-conventions) and
[cycle-protocol.md](../cycle-protocol.md#chores-sections).

## feat: grade the run from raw batches

Commits: [[1]],[[2]],[[3]]

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
- [[N]] 0.23.0-3 `feat: batch-based run gauge` — `gauge.rs`
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
box and workload, and no post-hoc filter separates them. Warmup
is the one workload-independent window — the -1 micro-probe
touches no bench code — so the environment grade is measured
there, gets its own signals and letter, and prints beside the run
grade. That grade is a verdict on the box rather than on the
bench, which makes it the one that could reasonably warn later.
It became rung -4, ahead of the selftest and the `calibrate`
deletion, because it is the certificate `calibrate` currently
provides and build-then-demolish says the replacement lands
first.

A discriminator for telling a machine transition from a bimodal
workload was drafted as a Todo and then dropped: it existed to
decide what a warning should *say*, and with no warning to write
and the environment question moved to warmup, it had no job left.

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
