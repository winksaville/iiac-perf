# Chores 05

Continuation of [chores-04](chores-04.md). Records landed work;
conventions in
[agent-data/notes.md](../../agent-data/notes.md#chores-conventions)
and [cycle-protocol.md](../cycle-protocol.md#chores-sections).

## Table of Contents

- [feat: grade the run from raw batches](#feat-grade-the-run-from-raw-batches)
- [docs: adopt universal AGENTS from vc-x1-template](#docs-adopt-universal-agents-from-vc-x1-template)
- [feat: compact the grade block into labelled columns](#feat-compact-the-grade-block-into-labelled-columns)
- [docs: explain the grade columns and the blocks/batches nesting](#docs-explain-the-grade-columns-and-the-blocksbatches-nesting)
- [docs: typeable punctuation only](#docs-typeable-punctuation-only)
- [docs: record the dynamic-warmup and placement-tracking designs](#docs-record-the-dynamic-warmup-and-placement-tracking-designs)

## feat: grade the run from raw batches

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
- [[5]] 0.23.0-4 `feat: environment grade across the run` —
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
- [[6]] 0.23.0-5 `feat: split warmup and run environment
  stretches` — the environment series is graded as two
  stretches instead of one: the warmup tail and the probes
  taken while the bench ran, printed as `env warmup:` and
  `env run:` with the composite the worse of the two. Warmup
  is scored on its trailing 8 probes, because absorbing a ramp
  is what warmup is *for*. See
  [Two stretches, one series](#two-stretches-one-series)
- [[7]] 0.23.0-6 `feat: qualify-environment subcommand` —
  `iiac-perf qualify-environment` respawns this binary
  `--runs` times at `--gap`, collects each run's environment
  grade, prints the table and a verdict, exiting nonzero when
  the machine does not qualify.
  `tests/qualify_environment.rs` shrinks to invoking it and
  asserting on the exit status. See
  [Naming: qualify-environment](#naming-qualify-environment)
  and
  [A selftest with a command line](#a-selftest-with-a-command-line)
- [[8]] 0.23.0-7 `refactor: drop overhead calibration`
  - `overhead.rs` and the `calibrate` command are gone, along
    with the three constants, the `adjusted` report column, the
    `adj/call=` header field, and the six-signal calibration
    grade. Reported values are raw. The startup banner becomes
    `Setup:`, listing the tick rate and the pin, sleep-inhibit
    and config state that remain. See
    [What replaced subtraction](#what-replaced-subtraction) and
    [Dither was not overhead machinery](#dither-was-not-overhead-machinery)
- [[9]] 0.23.0-8 `feat: warm the box once per process`
  - the first run in a process warms the box for
    `--settle-time` seconds (default 1.5, config `settle_time`,
    0 skips it) before recording anything, probing every 10 ms
    into the same warmup series it already builds. Later runs
    in the process inherit the machine state it wins, so the
    cost is paid once.
  - the warmup grade's window becomes the last 300 ms of the
    stretch rather than its last 8 probes, and the row leads
    with settle time (`settled 0.86s` / `not settled`), which
    `qualify-environment` reads into a `settle` column and a
    median line. See
    [Settle time is not a grade](#settle-time-is-not-a-grade)
- [[10]] 0.23.0 `feat: grade the run from raw batches`
  - close-out bookkeeping: version-of-record to 0.23.0, the
    cycle's `## In Progress` block retired into this section,
    its `## Done` entry written, and `notes/README.md` brought
    up to date with the commands and flags the cycle changed
  
### Settle time is not a grade

Warmup that deliberately spans the box coming up to speed
stops *reporting* that it did: the tail window sits after the
ramp, so the letter goes back to A and the run reads as though
nothing happened. Settle time is what replaces the letter — how
long the box took to reach the state it then measured in.

The definition, in one sentence: when the floor entered, and
stayed inside, ±1% of the level warmup ended at. The mechanics
matter mostly where they were got wrong twice, both times by
disagreeing with the letter printed beside them:

- A **suffix-median** rule read `settled 0.00s` on stretches
  whose own `drift` graded F. A minority of moved probes never
  shifts a median, so the rule reported a box that had visibly
  moved as having never left.
- A **per-probe** rule then read "never settled" beside
  `drift 0.00% A` on the 3900X, whose floor flickers across a
  1% band constantly while every median the grade takes stays
  put.

What settles it is asking the question of the same statistic
the grade uses: a running window median, 8 probes (the step
detector's minimum span) or a quarter of the series when it is
shorter — a quarter being what `drift` compares. On the 16-probe
stretch of a non-first bench, an 8-probe window is half the
series and swallowed an excursion `drift` graded F, which is
what forced the quarter clamp.

The third correction came from a user run rather than a test:
`qualify-environment -d=0 --settle-time=5` on the 3900X reported
settle times of 3.5-5.0 s, against ~1.0 s medians at the 1.5 s
default. The statistic was tracking the *budget*. It is the last
excursion's end, and a box that never really stops moving has its
last excursion near the end of warmup however long warmup is; the
distance between a run reading `5.01s` and one reading `not` was
a single probe. The rule now requires the settled state to hold
through the graded tail window, which is the span the letter is
scored over — so "settled at T" means the window the grade looked
at contained no movement, and anything later is `not settled`.

Two things the number does not say, both easy to read into it:
it is measured against where *this* warmup ended, so it never
says which state was the right one; and it is biased early by up
to one window, since the first window that reads settled
straddles the last of the ramp.

It is **reported, never judged**. A box still moving when warmup
ends already shows up as a `drift`/`step` D/F on the warmup
stretch, so a settle threshold in the `qualify-environment`
verdict would restate an existing criterion. What the column
adds is the *size* — how much `--settle-time` this box wants —
which is the input the "Dynamic warmup" Todo turns into
a stopping rule.

We think the reason a warm cannot be a fixed constant is that
the number is per-box: measured 2026-07-30 on the 3900X, the
default 1.5 s absorbs the startup ramp (warmup grades A, settle
landing 0.09–1.36 s across runs), but the box's ~10% bistable
shift then recurs mid-bench. Raising the warm to 3.5 s did not
help — 2 of 4 runs still moved, at 4.4 s and 4.6 s, one of them
inside the warmup tail. That is what kept the composite at
worst-of-two: the plan had the warmup stretch stop counting
because "a transition inside warmup is warmup working", which is
true of a ramp and false of a relapse.

### What replaced subtraction

Nothing replaced it, which is the point. The adjustment
estimated a quantity that is not well defined at this scale:
additivity is an approximation on a superscalar CPU, the
constants moved ~10% with frequency regime, and in the
same-harness A/B this tool exists for the correction appears on
both sides and cancels. Removing it costs a column and buys back
the ~1 s calibration that ran before every invocation.

What manages the apparatus cost now is sizing rather than
subtraction. A micro-probe (landed at -1) times back-to-back
timer pairs and `pick_inner` chooses `inner` so framing is a
small fraction of the workload's per-call cost. The residue is
small and, more usefully, common to both sides of a comparison.
That is a weaker claim than the old column made, and a true one.

The banner rename follows from what the block still holds. With
the constants and the grade gone it lists the tick rate, the warm
pin, the bench pin, sleep inhibit and config: provenance for the
numbers below it, not measurement. `Calibration:` would have been
a heading naming the one thing the block no longer does.

### Dither was not overhead machinery

`Dither` lived in `overhead.rs` and would have died with it, but
the harness uses it in four places, including `rand_u64` for
block sleep lengths, and the sample-seam dither is a measurement
property the README documents: a random sub-quantum spin outside
the timed interval makes the clock-quantum error zero-mean
instead of a coherent bias.

So this rung is an extraction as well as a deletion. `Dither`,
its `XorShift64` source and `DITHER_SPAN` move to `src/dither.rs`
with the calibration-specific statistics left behind. We think
the original placement was an accident of history rather than a
design: dither was built for calibration first and reused at the
seam later, so it never moved out.

### The warm pin's remaining job

Pinning main before the tick-rate read is what survives of the
calibration pin, and it is now close to ceremony. Measured on the
3900X, the read is insensitive to placement: 3.792891 ticks/ns
pinned to CPU0 against 3.792888 on CPU11 and CPU23, an ~8e-7
spread. It is a ratio of TSC ticks to monotonic nanoseconds over
~10 ms, so an interruption inflates both sides and cancels, and
`constant_tsc`/`nonstop_tsc` mean there is no per-core rate to
discover.

That matters because CPU0 is measurably the kernel's busiest
core here, taking 4-10x the `LOC`, `CAL`, `RES` and `TLB` counts
of CPU11 or CPU23 with no `irqbalance` running. Pinning a
measurement there would be a poor default; pinning a
cancellation-immune ratio there is merely pointless. The pin and
`--no-pin-cal` stay for now because the "Dynamic warmup"
Todo turns warmup into a real timing phase that will want a pin
back, and the placement question is recorded there rather than
churned twice.

### The 7600x stopped passing, and the grade is why

The rung's plan said to re-check the -6 selftest's verdict
thresholds against a genuinely cold first run once calibration
was gone. Measured 2026-07-29 on the 7600x: the box stops
qualifying, and chasing why turned up a defect in the grade
rather than a fact about the machine.

```
  run   warmup  env-run  worst   mean
  1     A       F        F       17.6 ns
  2     F       A        F       16.2 ns
  ...
  median environment grade: F
  verdict: NOT QUALIFIED
```

Before this rung the same box graded ten straight A's. Those A's
were an artifact of this tool: calibration spun ~1 s on core 0
before every invocation, which carried the box past a transition
it would otherwise meet mid-measurement. Deleting calibration
removes that accidental pre-warm, which is what the rung
predicted. So far so expected.

**The machine is fine, though.** `/proc/loadavg` reads 0.00,
`interference` reads 0.00% and `spread` 0.48%, and sampling
`cpuinfo_avg_freq` on the pinned core through a run shows exactly
one discrete event:

```
  0.05-0.75s   4841 MHz     dwelling at an intermediate P-state
  0.80s        4980 MHz
  0.85-2.40s   5440 MHz     +12.4%, the reported ~11% step
```

Not a ramp from idle: the box begins at 89% of its 5457 MHz max,
holds there for ~0.75 s of sustained load, then steps to 99.7%.
Pre-warm core 0 for 1.5 s and run again and every signal on both
grades reads A, `drift` and `step` at 0.00%. The F is the step
landing inside the window, nothing else.

**So `env warmup: A` is the bug.** Warmup is scored on its
trailing probes to answer "did it end settled", and it answered
yes while the box sat at 4841 MHz. A dwell *is* steady, so a
timing-only test cannot separate "settled at the top" from
"dwelling one P-state below it". That is the failure mode
[The clock as a warmup criterion](#the-clock-as-a-warmup-criterion-design-unmeasured)
described as a design concern and marked unmeasured; it is
measured now. Two consequences:

- the warmup stretch can issue a **vacuous A** over a window far
  shorter than the phenomenon it is certifying. For `min-now` the
  16 warmup probes span ~17 us against a transition that arrives
  at ~800 ms
- `qualify-environment`'s verdict is therefore not usable as a
  gate on this hardware class. It will read NOT QUALIFIED on any
  amd-pstate-epp box that dwells and then boosts, which is to say
  on a healthy quiet machine. Recorded against the
  "Dynamic warmup" Todo, which owns both the exit
  condition and the clock reading that separates a dwell from the
  top

What the run grade reports remains true and useful: something
moved 11% at 1.04 s, and the environment series agrees at 1.06 s,
so it was the box and not the workload. The two-series
attribution works. What overreaches is turning that into a
verdict on whether the machine is fit to measure on, when the
honest reading is that the harness started measuring too early.

### Naming: qualify-environment

The command was built as `settle` and renamed before it
shipped. `settle` describes the *phenomenon* — the box settles
into a state — but as a command word its imperative reads with
the wrong subject: `calibrate` works because the tool
calibrates, while the tool does not settle, the machine does.

`qualify` puts the action back on the tool, in the sense
equipment qualification uses. The object is spelled out because
`qualify` alone leaves "qualify what?" open, and because naming
the object leaves room for a sibling — `qualify-bench` (does
this workload give repeatable numbers?) is a plausible later
command that a bare `qualify` would have foreclosed.
"Environment" rather than "box" because that is the printed
vocabulary: the banner says `environment`, the rows say `env
warmup:` / `env run:`, the type is `EnvGrade`.

"Settle" survives where it was always right — as prose for the
phenomenon, and in *settle time*, the number the dynamic-warmup
Todo will report.

What the command runs is, in metrology terms, a repeatability
study of the apparatus plus the machine — the discipline that
names its instrument a *gauge*, which is what this project
already calls its grading module.

### A selftest with a command line

The test asks one question — *is this machine fit to measure
on?* — and until now it could only be asked by
`cargo test --ignored` with env vars that only that file
understood. The logic moves into the binary as a
`qualify-environment` command word, and the test becomes a
wrapper that asserts on its exit status.

What the move buys:

- **Real flags with real `--help`.** `SETTLE_N` / `SETTLE_GAP`
  / `SETTLE_PRINT_ONLY` become `--runs`, `--gap`,
  `--print-only`, plus `-d` for each child's duration and
  `--pin` passed through. Discoverable rather than
  archaeological.
- **Runnable by hand on any box**, including one with no Rust
  toolchain — `scp` the binary and run it, which is how the
  7600x and the Dell get characterized.
- **The test still exists**, still `#[ignore]`d, and still
  prints the table so a failure explains itself; it just no
  longer owns the logic.

Design points:

- **The observable is the environment grade**, migrated from
  the `calibrate` environment letter that -7 deletes. This is
  a test of the box, so it reads the workload-independent
  measurement. Both -5 stretches show in the table: `warmup`
  is the box's own settling behaviour across respawns, which
  is what a qualification test is asking about, and `env run`
  says
  whether it then held.
- **Respawn rather than loop in-process.** A fresh process per
  run is what terminal use looks like, and in-process repeats
  would share warmed state — the very thing under test. The
  children get `--no-inhibit` because the parent already holds
  the sleep lock and a re-exec per run would cost more than
  the run.
- **`min-now` is the child workload.** The box is the subject,
  so the leanest bench is right; it also measures nearly what
  the probe measures, which keeps the two grades
  commensurable.
- **The verdict is grades, not values**: median environment
  grade at B or better, and no run whose `drift` or `step`
  reached D/F in either stretch. Those two are the transition
  detectors — a D/F there is a state change landing inside a
  measurement window. `spread` and `interference` wobble is
  ambient contamination and does not fail a run. The table
  carries a `mean` column anyway, because a two-state box is
  visible at a glance in it.
- **An unknown letter scores worst**, so a parse miss can
  never flatter a run.
- **Run the test with `--release`.** `cargo test` builds a
  debug binary, and each child then spends ~20 s in
  unoptimized calibration and warmup against ~2 s optimized —
  200 s for the default ten runs, measured. It is also the
  less representative measurement, since the child's own
  phases are what provoke the box's state change and should
  run at the speed a real run does.

First numbers on the 3900X (four pinned runs, `-d 1`): warmup
letters A/D/A/A with an `env run` C, verdict NOT QUALIFIED. That is the
correct answer today — the box is the one that shows the
relaxation and the dynamic-warmup fix is a separate Todo, so a
PASS here would mean the test had stopped working. Unpinned,
the `mean` column showed 25.0 / 22.4 / 24.4 ns across three
runs, the two states plainly visible.

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

This window is also the seam with the "Dynamic warmup"
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
  -7.
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
and the one -7 removes an answer to.

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
[qualify_environment.rs](../../tests/qualify_environment.rs) runs, whose
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
([qualify_environment.rs](../../tests/qualify_environment.rs)) showing up
in the run rather than in calibration. Chasing which bench the
selftest should read to see that reliably is what exposed the
deeper problem: any answer would have been a workload chosen to
approximate a workload-independent question. The -4 environment
grade answers it directly, from the warmup micro-probe, so the
selftest reads that and needs no bench at all.
The thresholds were left where they are
rather than widened to absorb it: the box really is bistable, and
the dynamic-warmup Todo is the fix.

### Outcome

The cycle set out to stop subtracting an ill-defined overhead and
to grade the run rather than the room. Both landed, and the
report changed shape in the process.

- **Reported values are raw.** `overhead.rs`, the constants, the
  adjusted column and the `calibrate` command are gone, and
  nothing estimates a quantity that cancels in same-harness A/B.
- **Two grades, from two populations.** `run:` scores the
  batches the run itself produced; `env` scores the box from
  micro-probes that never touch the bench. An F on one beside an
  A on the other is a true and useful statement, which the single
  calibration letter could not make.
- **The box has a selftest.** `qualify-environment` respawns this
  binary, reads the environment stretches, and exits nonzero when
  the machine is not fit to measure on. It needs no toolchain on
  the box under test.
- **Startup is warmed once per process**, so the first bench no
  longer reports a cold machine's numbers, and the warmup row
  reports how long settling took.

What the cycle deliberately did not fix, each with an owner:

- **The 3900X's bistable floor.** Measured 2026-07-30: a 1.5 s
  warm absorbs the startup ramp, but the ~10% shift recurs at
  arbitrary times, and a 3.5 s warm still left 2 of 4 runs
  moving. No warm length reaches it; replication (`--blocks`) is
  the answer, and the box correctly fails `qualify-environment`.
- **A convergence rule for warmup.** The "Dynamic warmup"
  Todo owns it, and inherits the probe series and the settle
  statistic this cycle built as its input.
- **The report's grade block is verbose**: 85 grade lines on an
  `all` sweep. The columns Todo owns the reformat.
- **The selftest still runs a bench** it does not need, since
  every number in its table comes from the probe series. Todo #4
  owns removing it.

### Post-facto trapezoid rewrite (2026-07-31)

An experiment run after 0.23.1 landed: reshape this cycle's
published linear run into the trapezoid (merge non-ff) form in
place, to learn whether the shape can be adopted post facto.
One operation did it:

- `jj rebase -s snwmxmsnnywl -d yzvlvtkuplul -d ykvsnkysxulz
  --ignore-immutable`: the close-out change itself becomes the
  merge (first parent the 0.22.0 close-out, second parent the
  -8 rung tip), and 0.23.1 follows as its descendant. The
  rungs are not rewritten; they simply become the side leg.

Verified before pushing:

- the merge's tree is byte-identical to the original
  close-out (`git diff` empty): the graph changed shape, the
  content did not
- `git log --first-parent` reads one close-out per cycle
  (0.23.1 -> 0.23.0 -> 0.22.0)
- change IDs and `ochid:` trailers survived on both rewritten
  commits, so every cross-repo link stays valid and the bot
  repo needs no change at all

Costs, both instances of the backfill timing rule:

- the close-out's recorded SHA went stale (615646dd14cb
  became 797766b1d708); [[10]] is re-recorded in this same
  edit
- publishing required a remote rewrite of main, so any other
  clone reconciles with a force fetch

Takeaway: ochids ride chids, which rebases preserve, so the
dual-repo linkage is rebase-proof; the fragile references are
recorded SHAs, and the timing rule already localizes those to
one definition per commit.

## docs: adopt universal AGENTS from vc-x1-template

- [[11]] 0.23.1 docs: adopt universal AGENTS from vc-x1-template

Adopted the restructured universal bot instructions as a
single-commit cycle. The work that produced the adoption base
happened outside this repo, in the new vc-x1-template repo
created the same day; details live there. It was unpublished at
the time of writing, so no commit ref yet; backfillable here
once it lands.

- adoption base: the frozen snapshot
  `agents-protocol/AGENTS-vc-x1-f5-20260730-snapshot/` in
  vc-x1-template, the new template + coordination repo (init
  payload split into `work/` + `work.claude/` so init copies
  only payload; discussion artifacts and the adoption registry
  in `agents-protocol/`; per-member mailboxes in `messages/`)
- the pin set (AGENTS.md, CLAUDE.md, agent-data/) stays
  byte-identical to the adoption base; everything
  project-local moved to the new `custom.md`, including the
  dogfood log where this cycle's process findings landed
- snapshot-side amendments (rule 0 "read custom.md first",
  hard-rules-first ordering, generic pinned-to lines, and the
  as-built ladder form this very section uses) were authored
  in the snapshot because vc-x1's session was live; sync to
  vc-x1 is pending via its mailbox in vc-x1-template

## feat: compact the grade block into labelled columns

- [[12]] 0.23.2 feat: compact the grade block into labelled columns

The 0.23.2 single-commit cycle (renumbered from 0.24.0 by the
0.23.4 cycle), from the Todo of the same aim
(target shape decided 2026-07-29). The report's five grade lines
become one header over three rows: `env warmup`, `env bench`
(the stretch formerly called `run`, ending the collision with
the `run` grade), and `run all`.

- **a header labels every column**, so signal names stop
  repeating on every row; on an `all` sweep the block drops
  from 85 lines to 68
- **one header over all rows**, with a plain `-` where a signal
  does not apply to a row: the env/run signal mapping made
  visible (env has `spread` where run has `bursts`)
- **the composite becomes a leading `worst` column**; the
  `env worst case:` and `run worst case:` lines are gone. The
  worse-of-two-stretches env composite is now computed by its
  one consumer, `qualify-environment`
- **settle time becomes a `settle` column** (warmup row only).
  The Todo's target shape predated settle time landing at
  0.23.0-8, so the column is this cycle's one addition to it
- **the parser moved with the format**: `qualify.rs` splits
  rows on the two-plus-space cell gap the right-aligned
  columns guarantee, reads `worst` positionally, and takes
  drift/step letters from their cells; its tests now feed
  columnar rows
- **precision is settled and documented**: percentages keep
  their own fixed two decimals (bursts zero), the step
  timestamp its two (batches cannot locate a step finer than
  10 ms), `ticks/ns` its six; README's `--decimals` entry now
  states the flag covers exactly the band table's time columns

## docs: explain the grade columns and the blocks/batches nesting

- [[13]] 0.23.3 docs: explain the grade columns and the blocks/batches nesting

The 0.23.3 single-commit cycle (renumbered from 0.24.1 by the
0.23.4 cycle): make the report decodable
without assembling the answer from 160 lines of prose. Grown
from a 2026-08-01 session reading `zcr-with-2t` pinning
experiments, where the grade block and the blocks/batches
relationship both needed explaining in conversation.

- **column reference in README**: a compact per-column list
  directly under the grade-block example in "The two grades",
  one bullet per column stating which rows it applies to and
  what it measures; the existing sections keep the depth
- **blocks nest above batches**, stated at both places a
  reader looks: the README `--blocks` bullet and the `--blocks`
  help text. Batches are the grade block's contiguous
  time-series grain; blocks are the CI's sleep-separated
  replication grain; each block is a contiguous stretch of
  whole batches
- **a blank line before the grade block** separates it from
  the summary rows, on every report
- rides along: the "Topology-aware pinning and lCPU
  terminology" Todo entry (the session's pinning-experiment
  findings: SMT ~35 ns, same-CCX ~133 ns, cross-CCX ~633 ns on
  the 3900X), and the punctuation-sweep pickup moved to
  `## In Progress` as the next cycle

## docs: typeable punctuation only

- [[14]] 0.23.4 docs: typeable punctuation only

The 0.23.4 single-commit cycle landing the parked
`punctuation-sweep` branch (change `qymovnlz`, parked
pre-0.23.1): the typeable-punctuation conversions for
README.md, TODO.md and notes/cycle-protocol.md, folded onto
current `main` and completed against the text written since
the branch parked.

- **the fold**: `jj new main` + `jj squash --from qymovnlz`,
  then 44 conflict regions resolved by one rule: the
  destination's (main's) semantics win, the branch's
  punctuation conversion is re-applied where the sentence
  survived
- **AGENTS.md hunks dropped whole** (`jj restore --from main`):
  0.23.1 replaced the file with the pinned universal core,
  already typeable, whose rule 8 is the sweep's rule
- **the sweep was then completed, not just merged**: text
  written after the branch parked carried ~46 new banned
  characters (19 em dashes in README.md alone); all converted,
  each em dash by the structural decision prose.md requires.
  The one survivor is README's transcribed report banner
  (quoted tool output keeps its characters), which exposes
  that the *binary* prints an em dash in its banner, a
  src-side conversion for a future functional cycle
- **the branch's two appended Todo entries died in review**:
  "retire `Commits:`" landed at 0.23.1 (as-built ladder form),
  and "absorb versioning.md" targeted the pre-restructure
  files; its one load-bearing bullet, the minor-vs-patch
  advancement convention, moved to custom.md instead of a
  re-scoped entry
- **the cycle also renumbered published history**: 0.24.0 and
  0.24.1 became 0.23.2 and 0.23.3 under the scope-based
  advancement rule adopted mid-cycle (custom.md, "Version
  advancement is scope-based"): both were presentation and
  docs, contents-not-shape, so patches. Executed as a jj
  rewrite of the two commits (version-of-record and
  description, ochid trailers hand-copied) plus a force-push;
  refs [[12]] and [[13]] re-backfilled to the rewritten SHAs;
  the old-to-new mapping lives in the dogfood log because the
  bot repo's transcripts keep the old banners
- rides along: custom.md dogfood entry for the
  `vc-x1 push --body` leading-hyphen failure hit while pushing
  0.23.3, the advancement-convention record it references, and
  the "Always work on a topic bookmark" Todo the rewrite
  motivated

## docs: record the dynamic-warmup and placement-tracking designs

- [[N]] 0.23.5 docs: record the dynamic-warmup and placement-tracking designs

The 0.23.5 single-commit cycle, banking two design decisions
from the 2026-08-01 session before they evaporate; notes and
doc comments only.

- **"Dynamic startup warmup" renamed "Dynamic warmup"**, 11
  mentions across TODO.md, src/harness.rs, src/gauge.rs,
  tests/qualify_environment.rs and chores-05.md: the entry's
  scope is the warmup *mechanism* becoming condition-driven,
  not just the startup instance
- **end state recorded in the Todo: one parameterized warm
  loop.** The harness has three warms (per-run
  `warmup_and_probe`, once-per-process `process_warm`,
  `run_blocked`'s 2 ms block warm); the first two already share
  one probe series, prober and time origin, and the entry now
  commits to all three becoming exit-condition policies over a
  single mechanism rather than growing a fourth variant
- **placement tracking added to the topology Todo.** The
  pinning experiment made placement the dominant factor (4-18x)
  while the harness can only control it, not observe it. The
  design: cooperative knowledge (the `--pin` pool) layered over
  observational knowledge (a `/proc/self/task/` sweep at batch
  seams, children recursively; CPU-time deltas identify active
  threads with no bench cooperation). Sampled truth at the step
  detector's own granularity, so batches get placement-class
  labels (attributed steps) and unpinned `--blocks` runs
  stratify by class instead of smearing

# References

[1]: https://github.com/winksaville/iiac-perf/commit/621c5c97dbe1 "621c5c97dbe1418fdcb99db6080eecde40891491"
[2]: https://github.com/winksaville/iiac-perf/commit/769067779b20 "769067779b205d60d34961c841df671e0aefe0d9"
[3]: https://github.com/winksaville/iiac-perf/commit/f53644288058 "f53644288058d66350da3553eb2759e270b3d80a"
[4]: https://github.com/winksaville/iiac-perf/commit/4ce786ff7168 "4ce786ff7168efd8dc84c0afee4bbcdb71220a5a"
[5]: https://github.com/winksaville/iiac-perf/commit/8b58eac90202 "8b58eac90202d234558bc968b8c4de5660249961"
[6]: https://github.com/winksaville/iiac-perf/commit/44803acb3230 "44803acb323061b6d69ed9707f9d0d47f901e54d"
[7]: https://github.com/winksaville/iiac-perf/commit/e1e1a710aa1c "e1e1a710aa1c7e93381c469d46cea6bf9d00b1ad"
[8]: https://github.com/winksaville/iiac-perf/commit/197ddd48ed3f "197ddd48ed3f21664c133fbedecbad70d6d7ef14"
[9]: https://github.com/winksaville/iiac-perf/commit/b0437d08ad2e "b0437d08ad2e4e15a10dd17b11a0f1af959208b7"
[10]: https://github.com/winksaville/iiac-perf/commit/797766b1d708 "797766b1d708c3dba21f2f01a81c1590ab8dec0e"
[11]: https://github.com/winksaville/iiac-perf/commit/7f284cee8e5a "7f284cee8e5af27d780783f37f2fd3e6313d12ec"
[12]: https://github.com/winksaville/iiac-perf/commit/5a5a6bf779fc "5a5a6bf779fc6bf84502a50a3cd999cb86b3b5cc"
[13]: https://github.com/winksaville/iiac-perf/commit/43de4cc0e2b9 "43de4cc0e2b91639e2c69a0724bc5d891d5f018b"
[14]: https://github.com/winksaville/iiac-perf/commit/90fa62ef92ab "90fa62ef92aba0e97497f87aa418b23e7d7c2bfc"
