# Todo

This file uses [Prose form](AGENTS.md#prose-form). It
contains near term tasks with a short description and
uses links or reference links for more details.

## In Progress

When a `## Todo` item is picked up, its text moves here: the
problem overview and its list of things to do. That is followed
by the "plan" — a bulleted list of the development "ladder":
   - [[N]] 0.xx.y-0 blah (done)
   - [[N]] 0.xx.y-1 blah blah (current)
   - [[N]] 0.xx.y-2 blah blah blah
   - [[N]] 0.xx.y close-out and validation

**feat: grade the run from raw batches**

The 0.23.0 cycle, decided in
[Replanning II](notes/chores/chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run):
the overhead subtraction estimates an ill-defined quantity and
cancels in same-harness A/B anyway, while the calibration-time
grade certified the room, not the exam. Drop the adjustment
machinery; grade the run from its own time-ordered batches.

- [[67]] 0.23.0-0 `chore: open raw-batch grading cycle`
  (done)
- [[68]] 0.23.0-1 `feat: micro-probe inner-loop sizing`
  (done) —
  `pick_inner` inputs from a ~1 ms micro-probe (low quantile
  over back-to-back timer pairs), never printed; unhooks
  sizing from `cfg.overhead`
- [[69]] 0.23.0-2 `feat: time-ordered batch pipeline`
  (done) —
  samples land in a raw batch buffer; per-batch summaries
  (floor, mean, census counts) taken as batches fill, then
  bulk-record into the histogram; bounded memory
- [[70]] 0.23.0-3 `feat: batch-based run gauge` (done) — relocate
  the -6 grade machinery (signals, thresholds, letter) onto
  batch summaries: drift from floor movement, bursts localized
  in time, interference rate from census counts (absorbs the
  crossover entry's rate analysis); print each signal's own
  letter beside its value (the scores already exist — the
  composite is their worst, and showing them makes every letter
  self-explaining). Lands while calibrate still exists, so the
  two grades can be sanity-checked against each other
  - **reports, never warns** (decided 2026-07-28): the report's
    job is a histogram faithful to what was measured, and a
    run's steadiness is largely the workload's character — a
    blocking round-trip is genuinely less steady than a
    spinning one. The letter is that fact, not a fault
- [[72]] 0.23.0-4 `feat: environment grade across the run`
  (done) — the box's own grade, with its own signals and
  letter, printed beside the run grade. Micro-probes time
  timer pairs and never touch the bench, so no workload
  character enters the letter; this is the environment
  certificate that must exist before -6 deletes `calibrate`
  (build-then-demolish)
  - retitled from "from warmup": a warmup-only window is
    ~17 ms against the run's seconds, and graded A on a 3900X
    that was demonstrably moving. Probes now also run at every
    batch seam — a seam already costs 1-2 ms, so a ~256 us
    probe is a fraction of a gap that exists anyway — and the
    series shares the batch series' time axis, which lets
    movement be *attributed*: a step in both at one instant is
    the machine, a step in only the run is the workload
  - `--no-env-probe` limits probing to warmup; default is on,
    the seam bias measured at +0.86% on a spinning 2t bench
    and ~0 on 1t
  - coordinate with the "Dynamic startup warmup" Todo, which
    rewrites the same phase and now inherits this series as
    its convergence input; warnings stay off for now but this
    grade — a verdict on the box, not the bench — is the one
    that could earn them [[71]]
- [[73]] 0.23.0-5 `feat: split warmup and run environment
  stretches` (done) — promoted out of -7's precondition
  list to its own rung: the qualification selftest's observable *is*
  the environment grade, so restructuring it first means
  building that selftest once instead of twice
  - grade the probe series as two stretches — the warmup
    tail and the probes taken while the bench ran — rather
    than one blended series. `env warmup:` and `env run:`
    rows, composite is the worse of the two, so a reader sees
    which stretch earned the letter
  - warmup is scored on its **trailing 8 probes**, not the
    whole stretch: absorbing a ramp is what warmup is for, so
    the question is "did it end settled", not "was it steady
    throughout". This window is what the "Dynamic startup
    warmup" Todo turns into the exit condition
  - the failure being prevented is a fake step at the
    boundary: blended, a cold warmup followed by a hot run
    reads as a large step that nothing did wrong. Unit-tested
    both ways — the test asserts the blended series *would*
    have invented a >10% step [[71]]
- [[74]] 0.23.0-6 `feat: qualify-environment subcommand`
  (done)
  - `iiac-perf qualify-environment` respawns this binary
    `--runs` times at `--gap`, collects each run's environment
    grade, prints the table and a verdict, exits nonzero on
    FAIL. `tests/qualify_environment.rs` shrinks to invoking it
    and asserting on the exit status
  - the env knobs became real flags with real `--help`
    (`--runs`, `--gap`, `--print-only`, `-d` per child,
    `--pin` passed through), so the selftest is runnable by
    hand on any box — including one with no Rust toolchain,
    which is how the 7600x and the Dell get characterized
  - reads the environment grade, not the run grade: it is a
    test of the box. Both -5 stretches show — `env warmup` is
    the settling behaviour across respawns, `env run` whether
    it then held
  - verdict is grades, not values: median >= B and no run
    with `drift`/`step` at D/F in either stretch. Those two
    are the transition detectors; `spread`/`interference`
    wobble is ambient contamination and does not fail a run.
    The `mean` column is there because a two-state box is
    visible at a glance in it
  - FAILs on the 3900X today, correctly: that box shows the
    relaxation and the fix is the "Dynamic startup warmup"
    Todo, so a PASS would mean the test had stopped working
- [[76]] 0.23.0-7 `refactor: drop overhead calibration`
  (done)
  - delete overhead.rs, the constants block, adjusted columns,
    the `calibrate` command; raw values only; one README
    sentence on apparatus framing. Deletion last: every
    capability has a living replacement before its old home
    goes (build-then-demolish, decided 2026-07-28)
  - what makes this safe on a cold box landed at -5.
    Calibration spins ~1 s on core 0 before any bench and the
    clock ramp measured at -4 takes ~150-200 ms, so
    calibration covers it five times over — an accidental
    pre-warm this rung removes. After deletion warmup sees the
    whole climb, which the split stretches absorb correctly
    [[71]]
  - re-check the -6 selftest's verdict thresholds against a
    genuinely cold first run once calibration is gone: its
    inputs move even though its observable does not
- [[N]] 0.23.0-8 `feat: warm the box once per process`
  (done)
  - **why it is not optional: the first bench of every process
    reports numbers ~8.6% slow on a 7600x.** `min-now` reads
    17.6 ns as bench 1 against 16.2 ns once warm. A wrong
    histogram, not just a wrong letter, landing on whichever
    bench runs first. Diagnosis in
    [chores-05.md](notes/chores/chores-05.md#the-7600x-stopped-passing-and-the-grade-is-why)
  - **warm once per process, not per bench.** Measured
    2026-07-29, `all -d 2` on the 7600x in one process: bench 1
    (`min-now`) grades `env run step 11.05% F`, benches 2-17 all
    grade env-clean. The P-state boost is machine state, so every
    later bench inherits it. That puts the cost at ~2% of an
    `all -d 5` run (~85 s to ~86.5 s), not ~30%
  - the changes as built:
    - a process-level warm before the first bench, `--settle-time`
      seconds (default 1.5, config `settle_time`, 0 skips it),
      stepping the bench and probing every 10 ms into the same
      warmup series. Made a knob rather than a constant because
      the right value is per-box and unknown
    - **a time-based tail window** (last ~300 ms) instead of the
      last 8 of 16 probes, so the graded window stops depending
      on how many probes were taken. With the current geometry
      the tail is warmup's second half, so a 1 s warmup still
      straddles a transition at ~0.8 s and grades F
    - **the composite stays worst-of-two**, against the plan.
      "A transition inside warmup is warmup working" holds for a
      *ramp*, which the tail window now excludes anyway; it does
      not hold for the 3900X's relapses, which land inside the
      tail and are real. Dropping the warmup stretch would have
      hidden exactly those. Measured 2026-07-30
    - **report settle time** on the warmup line, and as a column
      in `qualify-environment`. Without it, absorbing the ramp
      sends the selftest back to all-A, which is the artifact
      0.23.0-7 just diagnosed; the useful observable becomes how
      long the box took to settle, as the "Dynamic startup
      warmup" Todo already anticipated. Reported, never judged —
      a box still moving at warmup's end is already a warmup
      drift/step D/F, so a settle threshold would only restate it
    - a stretch still moving at its end reads `not settled`, not
      a time near the end of warmup — which is the same string a
      box that settled late would print
  - leaves that Todo holding the real convergence criterion and
    the clock-reading gate. This rung is the correctness stopgap,
    not the design
- [[N]] 0.23.0 `feat: grade the run from raw batches` —
  close-out and validation

## Todo

Entries are in **strict priority rank** — #1 highest,
descending. Reprioritize by moving an entry, then
`vc-x1 fix-todo --no-dry-run TODO.md` to renumber.
The numbers are positional rank, not stable IDs — to refer
to a Todo, name it by its **title** (a greppable mention;
a numbered list item has no anchor to link to), not its
number. Long-tail entries
live in [todo-backlog.md](notes/todo-backlog.md). Use the
[Prose Form in AGENTS.md](AGENTS.md#prose-form); deeper
detail goes in `notes/chores/chores-NN.md` design
subsections (link via `[N]` ref).

1. Compact the report's grade block into labelled columns.
   Today each report ends with `env warmup:`, `env run:`,
   `env worst case:`, `run:` and `run worst case:`, three of
   which carry a single letter, and every signal repeats its own
   name on every row; on `iiac-perf all` that is 85 grade lines
   across 17 benches. Target shape (decided 2026-07-29):
   ```
     grade  phase     worst   spread   bursts  interference    drift             step
     env    warmup        A  0.30% A        -       0.01% A  0.00% A          0.00% A
     env    bench         C  0.33% A        -       0.02% A  0.00% A    4.20% @2.97s C
     run    all           D        -    33% B       2.59% B  2.93% C    9.37% @1.90s D
   ```
   - **a header labels every column**, the technique the band
     table already uses for `first|last|range|count|mean`. Signal
     names stop repeating on every row, so the columns get
     narrower
   - **one header over all rows**, with a blank where a signal
     does not apply to that grade. The blanks are not filler:
     they are the signal mapping made visible, which README
     currently spends a paragraph explaining (env has `spread`
     where run has `bursts`, and neither has the other's).
     Cheaper and clearer than a header per grade
   - a blank cell is a **plain hyphen** `-`, or `n/a` if a hyphen
     reads as a minus sign next to the percentages. Never an em
     dash: see
     [Typeable punctuation only](AGENTS.md#typeable-punctuation-only),
     and note the sweep converted exactly this kind of table cell
     in README
   - **`grade` and `phase` as two label columns**, which is what
     stops "warmup, run, run" from needing a footnote. `grade`
     names the subject, `phase` the slice of time, so two `env`
     rows read as one grade measured per phase and the single
     `run` row as a grade taken whole. It also removes a genuine
     collision: today `run` means a time window in `env run:` and
     a subject in `run:`, on adjacent lines
   - **rename the env stretch `run` to `bench`**, so the phases
     are `warmup` and `bench`. That pair describes a run's two
     halves better than `warmup`/`run` did, and it leaves the
     word `run` meaning exactly one thing
   - cheaper variant if two label columns are too much: keep
     single labels as `env warmup` / `env bench` / `run`. Removes
     the collision but does not explain why `env` has two rows
   - the composite is a leading `worst` column rather than a
     comma member: with names in the header every signal reads
     *value letter*, and a composite has no value, so as a list
     member it would look like a signal whose measurement went
     missing
   - `env worst case:` goes away. It was the worse of the two
     stretch letters, which are both visible above it, so the
     reader derives it the way they would anyway. `run worst
     case:` goes away outright: one row, one letter
   - the win is width and repetition rather than height: 5 lines
     in, 4 out (one header plus three rows)
   - **rows stop being self-describing**, which is the cost being
     accepted here. A row means nothing without its header, and
     this repo quotes single grade lines: `chores-05.md` has
     "`env step 9.88% @2.138s` beside `run step 9.98% @2.1s`",
     and README quotes several more. Future quotes have to name
     the signal in prose once the name leaves the row
   - `step` is the wide column, since it alone carries a
     timestamp (`4.20% @2.97s C` against `0.30% A`). Either give
     it a fixed width or move the timestamp elsewhere
   - **settle the block's precision, which today ignores
     `--decimals` and is internally inconsistent.**
     - the two grades printing the step timestamp at different
       precision was fixed in 0.23.0-7: `step_at_suffix` in
       `harness.rs` now owns the format for both, at two decimals
       (10 ms), because batches flush at ~15-50 ms so neither
       series locates a step finer than that
     - percentages hardcode `{:.2}%`, `bursts` `{:.0}%`. Piping
       `--decimals` into them looks consistent but is wrong: the
       flag's rationale is the ns recording floor, which does not
       transfer to a ratio, and `--decimals 0` would render
       `spread 0% A` and destroy the column's signal. Give the
       percentages their own fixed precision
     - the timestamp is a genuine time, in seconds, so it has a
       claim on the flag. But at the default of 1 it would read
       `@2.0s`
       and stops locating a shift usefully. Pick one: its own
       precision, or an explicitly documented scope for
       `--decimals`
     - `ticks/ns` in the `Setup:` block hardcodes `{:.6}`,
       inherited from the deleted `print_raw_calibration`. Another
       ratio, same question
     - whichever way it lands, README's `--decimals` entry needs
       to say what the flag covers rather than "the report's time
       columns", which is already ambiguous about the grade block
   - **this changes a parser.** `src/qualify.rs` reads its
     verdict input with `strip_prefix("env worst case:")` (near
     line 172) and must instead take the worse of the two stretch
     letters. Its `parse_stretch` tests assume the inline comma
     list and need rewriting for positional columns
   - keeps the -3 property that made the old shape worth having:
     the composite sits beside its causes, so it names its own
     cause without a lookup
   - coordinate with the "Machine-readable report output" entry:
     a `--format json` consumer wants a named field for the
     composite regardless of the text form, and that is the
     better home for whether an env-level composite should exist
     at all
2. Land the parked `punctuation-sweep` branch as 0.23.1. The
   work is done and committed on the `punctuation-sweep` bookmark
   (change `qymovnlz`), a sibling of `main`, holding the
   `Typeable punctuation only` rule plus 405 em dash conversions
   and 47 arrow/ellipsis/en-dash conversions across AGENTS.md,
   TODO.md, notes/cycle-protocol.md and README.md.
   - **it is a parking commit, not a publishable one.** Fold it
     back into a working copy before pushing:
     `jj new main`, then
     `jj squash --from qymovnlz --into @`. Pushing it as-is trips
     the bug at
     [cycle-protocol.md](notes/cycle-protocol.md#recovery):
     `vc-x1 push` creates the commit from pending changes and
     would mint a stamped empty duplicate on top
   - expect a `TODO.md` conflict. 0.23.0-7 renumbered every entry
     (`fix-todo`, when the grade-block entry went to #1) and the
     branch also edited numbered entries' first lines. Resolution
     is mechanical: take main's numbering, re-add the branch's two
     appended entries (absorb versioning.md, retire `Commits:`)
   - also expect ~10 conflicting lines in README.md where
     0.23.0-7 rewrote calibration prose the branch had converted,
     and drop the branch's `## In Progress` ladder hunks outright:
     close-out deletes that block
   - `Cargo.toml` bumps to 0.23.1 at that point, not before. A
     docs-only cycle takes a patch bump; see entry on absorbing
     versioning.md for where that convention is recorded
   - the branch's `chores-05.md` edit (dropping the `Commits:`
     line) is unrelated to punctuation and rides along; see the
     entry retiring `Commits:` for the rest of that work
3. Dynamic startup warmup — replace the fixed
   `WARMUP = 10_000` step count in `harness.rs` with
   warm-until-stable: a fixed count's wall-clock scales with
   step cost, so the fastest benches warm ~10 us against
   frequency-governor ramps of tens-to-hundreds of ms and
   `pick_inner` sizes mid-ramp (the 7600x F diagnosis,
   [Replanning II](notes/chores/chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run);
   the retired -6 rung's design, revived for the run itself)
   - terminology: the warmup unit is a **warmup pass** (the
     0.22.0-4 "loop-only passes" sense) — a short, unrecorded,
     timed burst of bench steps yielding one floor. Not a
     "probe": `TProbe` is the measurement instrument, and the
     "micro-probe" is the 0.23.0 cycle's ~1 ms timer-pair
     frame measurement
   - fuse with `pick_inner` sizing: the warmup pass *is* the
     step-cost sizing pass, repeated back to back until K
     consecutive floors agree within tolerance; the final
     agreeing floor is the sizing input, so sizing is
     post-ramp by construction and convergence is tested on
     the number actually consumed (the micro-probe supplies
     the frame input, run after convergence)
   - K defaults to 3: two agreeing passes can certify a dwell
     at an intermediate P-state (the staircase ramp), not the
     top
   - **trailing-window grade = exit condition** (design,
     2026-07-28): rather than K agreeing floors, grade a
     *sliding window* over the -4 warmup probe series and warm
     until the trailing window reads A, or the cap. The exit
     condition and the warmup letter become one computation:
     exit on A means the run started post-ramp by
     construction, and hitting the cap reports whatever the
     window actually scored — the "run started unstable"
     signal, not a silent proceed. Window length takes the
     same "minimum count or minimum wall time, whichever is
     larger" shape as pass length (count because the split
     detector needs 4 points a side; wall time because the
     ramp is a ms-scale phenomenon). Signals: spread, drift,
     step — `interference` is the weak one [[71]]
   - **read the clock, not just the timing** (design,
     unmeasured): steadiness cannot tell "settled at the top"
     from "dwelling at an intermediate P-state", because a
     dwell *is* steady. Delivered frequency is an unprivileged
     sysfs read on both AMD boxes
     (`cpufreq/cpuinfo_avg_freq`) and the ramp is ~150-200 ms
     (measured at -4). Gate on **clock stability under load**,
     never on a fraction of `cpuinfo_max_freq`: a threshold
     would need tuning between 96.1% (3900X) and 99.7%
     (7600x) sustained, and a thermally-limited laptop
     plateaus lower still while that plateau is its honest
     clock. Optional by construction — `cpuinfo_avg_freq` is
     amd-pstate-specific and some drivers' `scaling_cur_freq`
     reports requested rather than delivered — so read where
     present, fall back to timing-only. Report the ratio, do
     not grade on it [[71]]
   - report shape: normal output carries one warmup line
     (letter plus **settle time**, a number this project does
     not have yet and a real machine characteristic); `-v`
     shows the complete warmup picture, the per-probe table
     with the ramp's shape. The qualification selftest reading a
     table of settle times across respawns is a better
     observable than a table of blended letters
   - convergence is agreement, not direction — the 3900X is
     bistable (2026-07-27 `calibrate` runs): sustained rapid
     repetition climbs it into a fast state (~0.445 ns/iter,
     drift bracket caught 48.6 -> 45.3 mid-window, D/F),
     low-duty isolated runs sit at ~0.489 (B), ~9% apart, and
     transitions straddle windows in *both* directions (the
     climb under quick cadence; the settle-back after load,
     which outlasted an 8 s gap once the box had been hot);
     fixed warmup absorbs only transitions shorter than
     itself, whatever their sign
   - floors, not means, so one preemption doesn't fake
     (in)stability; warm box exits near-immediately
   - **decide where warmup runs, and whether it pins at all.**
     Measured on the 3900X 2026-07-29, cumulative per-CPU kernel
     interrupts: CPU0 takes 4.1M `LOC` against 0.65M on CPU11 and
     0.54M on CPU23, 6.2M `CAL` against ~0.6M, plus 4-6x the
     `RES` and 3x the `TLB` of either. No `irqbalance` running.
     CPU0 is the boot CPU and the `nohz_full` housekeeping CPU,
     so this is expected rather than a quirk of this box. Some of
     CPU0's `LOC` share is self-inflicted, since every run
     pinned main there and spun ~1 s until 0.23.0-7.
     - it did not matter for the only consumer left after
       0.23.0-7. The tick-rate read measured 3.792891 pinned to
       CPU0 against 3.792888 on CPU11 and CPU23, an ~8e-7
       spread. That read is a ratio of TSC ticks to monotonic ns
       over ~10 ms, so an interruption inflates both sides and
       cancels, and `constant_tsc`/`nonstop_tsc` mean there is no
       per-core rate to find. So the current default is harmless,
       not correct
     - it will matter here, where warmup becomes a real timing
       phase converging on frequency state
     - **not** "use the last core": on hybrid Intel parts the
       high-numbered CPUs are usually E-cores, so that rule
       silently picks the slowest core there. A middle core is
       arbitrary, which is the same folklore-over-measurement
       move 0.23.0 removed elsewhere. The principled version is
       "not the boot CPU, and a full-frequency core", which needs
       topology awareness rather than a new constant
     - the sharper question is whether to pin at all. Pinning
       main to the kernel's busiest core for 10 ms and restoring
       afterwards is ceremony now; deleting it would take
       `--no-pin-cal` with it. Left in place at 0.23.0-7 because
       this entry will want a pin back, so it should be justified
       or deleted here rather than churned twice
   - slow steps (`inner` -> 1 territory): pass length adapts
     — minimum step count or minimum wall time, whichever is
     larger — so a floor is never one sample; the cap exit
     then distinguishes "floors disagreed" (unstable, gauge
     signal) from "too slow to certify K passes" (proceed,
     label the run uncertified — at `inner = 1` sizing can't
     be wrong and framing is negligible, so the stakes are
     low there)
   - hard cap at governor scale (a few hundred ms); hitting it
     unconverged is a gauge signal ("run started unstable"),
     not a silent proceed — and the cap doubles as the
     estimate-phase deadline the `--pin` guard entry (below)
     wants, so slow and non-converging benches share one
     diagnostic exit
   - differential start-vs-end QC: repeat the same pass at
     run end and compare floors (never absolute, never
     subtracted); the N-sweep slope/intercept decomposition
     stays an idea unless batch data shows frame shifts need
     separating from per-iteration shifts
   - acceptance test — `tests/qualify_environment.rs` (landed
     0.23.0-1, simplified to one loop 2026-07-28):
     `#[ignore]`d integration test spawning the real binary
     per run (`CARGO_BIN_EXE`), one loop of 10 back-to-back
     runs — the loop's own load provokes the transition,
     whichever run straddles it lights up drift/repeat, and
     later runs ride the state the early runs forced
     (involuntary warmup — the service the fix makes
     deliberate). Verdict: median ≥ B and zero runs with
     drift/repeat at D/F — cause-aware via the per-signal
     letters, so ambient contamination (disturbed/dirty,
     e.g. a concurrent build) and the resid machine trait
     can't flake it. Reproduced failing 2026-07-27 on the
     3900X (repeat F on the climb run). `IIAC_PERF_BIN` pins a
     saved failing build; the observable (calibrate letter)
     migrated to the 0.23.0-4 environment grade when calibrate
     died. Part of this cycle's close-out validation
   - **a trailing-window A can be vacuous, measured 2026-07-29 on
     the 7600x.** The window certifies "did it end settled" and
     said yes while the box dwelled one P-state below the top,
     because a dwell is steady. Then it stepped +12.4% at ~0.8 s,
     inside the run. Sampling `cpuinfo_avg_freq` on the pinned
     core: 4841 MHz held for ~0.75 s, then 5440 MHz against a
     5457 MHz max. Pre-warm core 0 for 1.5 s and every signal on
     both grades reads A, so the machine is fit and the harness
     simply started measuring too early
     - this is the case the "read the clock, not just the timing"
       bullet below predicted and called unmeasured. It is
       measured now, and it is the strongest argument for making
       the clock reading part of the exit condition rather than
       an optional extra
     - a window can also be far shorter than what it certifies:
       `min-now`'s 16 warmup probes span ~17 us against a
       transition arriving at ~800 ms. Whatever the exit rule, it
       needs a minimum wall-clock span, not just agreement
     - **`qualify-environment`'s verdict is not usable as a gate
       until this lands.** It reads NOT QUALIFIED on any
       amd-pstate-epp box that dwells then boosts, which is to
       say on a healthy idle machine. Fixing the exit condition
       fixes the selftest at the same time, since its observable
       is this grade. Detail in
       [chores-05.md](notes/chores/chores-05.md#the-7600x-stopped-passing-and-the-grade-is-why)
4. Qualify the environment without a bench.
   `qualify-environment` respawns children running `min-now`,
   but every number in its table comes from the micro-probe
   series, which never touches the bench. The bench is there
   only to give the warm something to do and to produce a
   report to parse — so the selftest inherits a workload's
   character it does not want, and the parent parses prose
   (see the machine-readable-output entry below, which this
   would make moot for the selftest).
   - measure the probe series directly: warm and probe with no
     bench registered, grade the stretches, done. The `mean`
     column becomes the probe's own floor, which is the
     quantity the grade is computed from rather than a second
     measurement of nearly the same thing
   - **the warm's character is the open question.** A
     probe-driven warm is light; on hardware where a heavy
     workload drives a different clock/power state (AVX
     offsets), a light warm would qualify the box for work it
     will not do. Moot on the 3900X and 7600x, where `min-now`
     *is* essentially the probe, so decide it with a
     measurement on a box where it isn't
   - **respawn or loop** is a second question, not this one:
     respawning resets process-local state (address space,
     caches, allocator) and loops do not, but neither resets
     the machine's P-state — what re-rolls that is the gap and
     the duty cycle. If the answer is loop, the results stay
     structured data and never become text
   - coordinate with the "Dynamic startup warmup" Todo, which
     owns the convergence rule this would warm by, and with
     the grade-block columns entry, which reformats the table
     this prints [[75]]
5. Guard `--pin` pools smaller than the bench's thread
   placements, and deadline the estimate phase — `zcr-mpsc-2t
   --pin 8` put both spinning software threads on one logical
   CPU and appeared hung until ^C (2026-07-26, bug #1 in
   [bugs.md](notes/bugs.md#bugs))
   - track `core_for` requests in `RunCfg` (max `thread_idx`
     asked for); refuse the run when placements exceed unique
     CPUs in the pool — placement only goes through `core_for`
     when pinning is active, so the guard covers every path,
     and no pinning means the scheduler separates the spinners
     itself
   - wall-clock deadline on the open-loop 5x1,000-step
     estimate phase so *any* pathologically slow bench aborts
     with a diagnostic naming per-step cost and pinning,
     instead of hanging
6. Move the batch seam's work off the measuring thread, using
   the FastForward-style SPSC ring — the batch flush stops the
   bench for ~1-2 ms (a `select_nth_unstable` over up to
   65,536 values plus 65,536 histogram records) every 50 ms,
   so ~2-4% of a run is spent at seams. Hand the filled buffer
   to a consumer thread that sorts, summarizes and records
   while the producer fills a second one; the seam drops to a
   pointer swap
   - the payload is one word, a buffer offset — the exact
     shape `ffq` is built for, and the project dogfooding the
     queue it benchmarks
   - double-buffered: at ~1-2 ms of work per 50 ms batch the
     consumer runs ~30x faster than it needs to, so two
     buffers never back up
   - honest cost: the consumer's cross-core traffic runs
     *during* measurement, trading a gap on the hot core for
     background L3 pressure. Measure it the way the -4 seam
     probe was measured (interleaved A/B, pinned, trimmed
     mean) rather than assuming
   - blocked on the ring existing — see the
     "FastForward-style SPSC ring" entry, currently on the
     `ffq-spsc-notes` bookmark rather than `main`
7. Tighten thread/CPU terminology across docs and doc
   comments: "software thread" for what `thread::spawn`
   makes, "logical CPU" (hardware thread) for what `--pin`
   selects and the OS schedules onto, "physical core" for the
   engine SMT siblings share — bare "core"/"CPU"/"thread"
   only where context disambiguates
   - spin-wait bench docs state the precondition: each
     spinning software thread needs its own logical CPU
   - `--pin` help/README say slots are logical CPU ids
8. Rebase `web-claude-tweaks` onto post-0.22.0 `main` —
   rewrites an already-published bookmark (needs approval)
   and its arbitrary `0.21.0-b` version needs replacing;
   owed from the 0.22.0 close-out plan
9. Unit scaling in report columns (`us`/`ms`) — per-row
   auto-scale so columns stay eyeball-comparable (bands are
   monotonic, so a row's first/last/mean share a magnitude),
   or `--units ns|auto` for script-stable output; needs
   `--decimals` landed first (`3.18 ms` vs `3 ms`); candidate
   `-4` for the report-options cycle.
10. Machine-readable report output (`--format json`, or
    key=value lines to stay dependency-light) — design once
    the batch gauge lands (0.23.0-4) so the schema covers the
    surviving surface: report stats, gauge signals, letter.
    Consumers: `tests/qualify_environment.rs` (drops its
    brittle-but-loud line parsing), placement-map validation
    runs, cross-run comparison scripts. Kin to the
    unit-scaling entry's `--units ns` script-stable concern
    (above) — one flag family.
11. Trimmed core stats: `mean/stdev p10-p90` report row,
    additional to (never replacing) `mean` / `mean min-p99`;
    trim bounds possibly configurable (`--trim p10:p90`?) —
    the full mean wobbles ~±1.4% with the run's mode mix while
    the core plateau is ~±0.2% stable, so the trimmed row is
    the run-to-run comparable number. Boundary sensitivity
    (see [[57]]): window edges in the mode-mix smear inherit
    its wobble (p50-p60 ±0.05% vs p40-p50 ~1%), so also
    consider a dominant-*mode* statistic (peak-density region,
    bottom-count-independent) [[57]]
12. Find and label the interference crossover — the band where
    the tail stops measuring the code and starts measuring the
    machine. Not to hide it: to *name* it, because that is the
    signal TProbe exists to surface (the OS swapping, a drive
    stalling, anything not caused by the code under test).
    - Locate it from the data rather than fixing it at a
      percentile: the giveaway is the band `range` exploding
      (min-now 0.21.0, 3900X: `n3` range 3.0 ns -> `n4` range
      200.4 ns), not a chosen p99.
    - The crossover moves with the bench. A counting argument
      places it: interference arrives at a *rate*, so it can
      only contaminate so many samples. That run's `n2` held
      838,635 of 8,059,469 samples over 5 s = ~168,000/s, and
      nothing in the OS runs at that rate, so `n2` is code. The
      `n4`+ bands total ~1,500/s — timer-interrupt territory.
    - So report the above-crossover count as an **interference
      rate**, and consider surfacing whether the run was quiet
      enough to trust. Calibration wants exactly this signal
      (see [[61]]); a contaminated run is currently only
      detectable by squinting at band ranges.
    - Superseded pointer: the 0.22.0-5 calibration-time grade
      certified the ~1 s window before the run;
      [Replanning II](notes/chores/chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run)
      moves grading onto the run itself. This entry's crossover
      and rate analysis is absorbed by Todo #1's batch design,
      which supplies the time axis the histogram lacks.
    - Pairs with the trimmed-core-stats entry above: that one
      needs a defensible upper bound, and this is how to find
      one per run instead of hardcoding p99.
13. Upstream the ladder commit-ref convention to
    `../vc-template-x1`: In Progress ladder rungs (and the
    chores As-built rungs) carry a prepended `[[N]]`
    commit-ref placeholder, backfilled as each commit
    becomes permanent — template's cycle-protocol.md,
    AGENTS.md, and TODO.md example need the shape; that
    repo has its own approval/push flow
14. Investigate: suspend gap missing from samples. A 0.13.5
    `--no-inhibit` suspend test detected ~1.2 s suspended inside
    the measured window but the max sample was only 4.0 ms,
    while the 0.13.1 test (8.4 s gap) showed the expected 10.4 s
    max sample. We think minstant's TSC may halt across some
    suspends and count through others. Repeat the test comparing
    detected gap vs max sample; if the TSC halts, per-sample
    timing silently loses suspend time — document either way.
15. CLAUDE.md governance model (design cogitation) [20]
16. Revisit probe adjustment under the in-interval vs
    call-to-call split: probes take one call per sample
    (inner=1), so the in-interval timer slice is unamortized
    and unmeasurable — an `adjusted` column can subtract
    nothing defensible; maybe state a bound instead
    [analysis](notes/design.md#timer-overhead-in-interval-vs-call-to-call)
17. Convert `harness` / `Bench` to probe-based measurement. Will
    likely need inner-loop support on `Probe` (batch N calls per
    sample; report divides by N and accounts for per-sample
    framing) so very-small workloads can still amortize timer
    overhead the way `run_adaptive` does today.
18. Rename app
19. Design an app to measure IIAC perforanace written in Rust[1]
20. `ice-ps-2t-wait` — iceoryx2 pub/sub with blocking waits via
    `Listener`/`Notifier` events; completes the {transport} ×
    {wait policy} matrix cell that compares against `mpsc-2t`
21. Switch ice benches to the loan-based zero-copy send path
    (`loan_uninit` + `send`) — the API a perf-sensitive user would
    use, and closer to iceoryx2's own benchmark method
22. Payload-size sweep for the round-trip benches (8 B / 8 KiB /
    1 MiB) — makes iceoryx2's size-independent latency vs channel
    copy cost visible in our own tables
23. `crossbeam-1t` / `crossbeam-2t` — `crossbeam-channel` directly
    (compare to mpsc-1t/2t which use crossbeam under the std API)
24. `tokio-mpsc-1t` / `tokio-mpsc-2t` — `tokio::sync::mpsc` round-trip
    inside a Tokio runtime (async overhead)
25. `flume-1t` / `flume-2t` — `flume` MPMC channel
26. Function-call baselines: direct call vs `Box<dyn Trait>` vs
    `async fn` (poll-once) — anchors the channel/serde numbers
    against the cheapest possible "send a value then receive it" path
27. When the second channel impl lands, extract shared message types
    + round-trip helpers into `src/benches/common.rs` (deferred from 0.2.0)
28. Additional thread control (count, per-thread pin lists, NUMA) —
    shape once a concrete bench needs it
29. Rename crate `iiac-perf` → general-purpose name (breaking; deferred)

## Ideas

Longer-range thoughts, not yet ranked work. `-` bullets, no
numbering; promote into `## Todo` when one becomes actionable.

- Per-bench dependency isolation — motivated by dep provenance:
  the deps are the thing being measured, so a dep bump (e.g.
  iceoryx2 0.9.2 → 0.9.3) legitimately moves that bench's
  numbers and shouldn't ride in silently. Options considered
  (2026-07-08):
  - Caveat first: a Cargo **workspace shares one Cargo.lock**
    across members — it scopes deps per package (ice benches
    alone pay for iceoryx2; faster `-p` builds; harness/probes
    become a library crate) but does *not* give per-bench lock
    isolation, and it splits the single CLI into many binaries.
  - Targeted updates (`cargo update -p <crate>`, never bare
    `cargo update`) — ~90% of the provenance benefit at zero
    structure cost; adoptable immediately as discipline.
  - Feature gates (`--features ice`) — solves build weight in
    the current single package, not lock isolation.
  - Truly standalone crates (own Cargo.lock each) — the only
    real per-bench dep isolation; maximum maintenance, and cuts
    against "same harness, same build" A/B comparability.
  - Current lean: targeted-update discipline now; feature gates
    or workspace only when bench families multiply.
- clap CompleteEnv dynamic completion (the `unstable-dynamic`
  feature): clap's native runtime completer (`COMPLETE=bash
  iiac-perf`) would give bash die-hards a compact column view
  without carapace; revisit if clap stabilizes it.
- Stability selftest mode (2026-07-27) — grade the
  environment more thoroughly than a single run's gauge: a
  product subcommand that respawns its own binary
  (`current_exe()`) N times at configurable cadences and
  reports cross-run gauge agreement ("is this box currently
  trustworthy for A/B?"). Precedent in-product: the
  calibration repeat self-check and `--blocks` both already
  validate by orchestrated repetition; this is the next ring
  out. Subsumes `tests/qualify_environment.rs`'s orchestration —
  the test reduces to asserting on the verdict, and its
  env-var knobs become clap flags. Concrete motivation
  (2026-07-27): the qualification test can't run on the 7600x,
  which has only the installed binary — environment
  qualification shouldn't require a source tree.
  **Promoted 2026-07-28**: the minimal version is the
  0.23.0-6 ladder rung (`qualify-environment subcommand`); what
  remains here for later is the fuller mode — cadence
  sweeps, richer cross-run reporting.
- Tick-phase avoidance (2026-07-27): the scheduler tick is
  periodic per-CPU (~300/s at CONFIG_HZ=300) and a tick hit is
  an unmistakable outlier, so predict the next tick from
  detected hits and pause measuring ~30 us around it — ~1%
  duty cost, no governor exposure with governor+EPP
  `performance`. Doesn't improve the bulk stats (tick hits
  are already detected and trimmed); buys a cleaner
  above-crossover tail on unmodified machines, so rarer
  aperiodic events (device IRQs, SMIs, code slow paths)
  become visible over the periodic contaminant. Check the
  interaction with dither (anti-phase scheduling must not
  introduce a systematic phase bias), and compare against
  `nohz_full`/`isolcpus` isolation — which abolishes the tick
  on a dedicated core and strictly dominates when a reboot is
  allowed — before building.

## Bugs

_See [bugs.md](notes/bugs.md)._

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](notes/done.md) to keep this file small.

- fix: calibration robust to codegen and noise [[66]] —
  the 0.22.0 cycle; validation pass recorded in
  [placement-map.md](notes/placement-map.md)

# References

[1]: /README.md#Design-010
[20]: /notes/chores/chores-02.md#claudemd-governance-model-071
[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[66]: /notes/chores/chores-04.md#fix-calibration-robust-to-codegen-and-noise
[67]: https://github.com/winksaville/iiac-perf/commit/621c5c97dbe1 "621c5c97dbe1418fdcb99db6080eecde40891491"
[68]: https://github.com/winksaville/iiac-perf/commit/769067779b20 "769067779b205d60d34961c841df671e0aefe0d9"
[69]: https://github.com/winksaville/iiac-perf/commit/f53644288058 "f53644288058d66350da3553eb2759e270b3d80a"
[70]: https://github.com/winksaville/iiac-perf/commit/4ce786ff7168 "4ce786ff7168efd8dc84c0afee4bbcdb71220a5a"
[71]: /notes/chores/chores-05.md#the-clock-behind-the-anomaly
[72]: https://github.com/winksaville/iiac-perf/commit/8b58eac90202 "8b58eac90202d234558bc968b8c4de5660249961"
[73]: https://github.com/winksaville/iiac-perf/commit/44803acb3230 "44803acb323061b6d69ed9707f9d0d47f901e54d"
[74]: https://github.com/winksaville/iiac-perf/commit/e1e1a710aa1c "e1e1a710aa1c7e93381c469d46cea6bf9d00b1ad"
[75]: /notes/chores/chores-05.md#settle-time-is-not-a-grade
[76]: https://github.com/winksaville/iiac-perf/commit/197ddd48ed3f "197ddd48ed3f21664c133fbedecbad70d6d7ef14"
