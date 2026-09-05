# Reading a report

What every surface of a run's output means and, above all, what
to conclude from it. The flags that shape a report are in
[usage.md](usage.md), the config file in [config.md](config.md).
The report is dense by design: every cell answers a question
someone actually had, and this guide is the decoder. If you read
nothing else, read
[The measurement hierarchy](#the-measurement-hierarchy) and
[What to conclude: a worked example](#what-to-conclude-a-worked-example).

## The measurement hierarchy

Every number in a report is computed at one of five levels.
Knowing which level a number lives at is most of reading it:

1. **Call**: one execution of the bench's operation (for
   `min-now`, one `Instant::now()`). Never timed individually.
2. **Sample**: the unit that is actually timed. One timer pair
   brackets `inner` back-to-back calls, and the reading divided
   by `inner` is the recorded per-call value (kept in
   picoseconds, so sub-ns precision survives). `inner` is
   auto-sized so the timer's own cost stays a small fraction of
   the workload's; it sets the **quantum**, the smallest value
   step a sample can express.
3. **Batch**: a pipeline chunk of consecutive samples, flushed
   at 65,536 samples or 0.05 s, whichever comes first. Batches
   are the run's **time axis**: the grade block's drift and
   step signals, the delivered-clock series, and the
   `resolution` row are all computed over batches.
4. **Block** (`--blocks N`): a division of the run's budget, the
   **replication axis**. With a nonzero `--block-sleep` each
   block is a mini-run separated by a state-re-rolling sleep,
   and the spread of block means yields CI95 and LSC. With no
   sleep, blocks are mere partitions and those rows print `-`.
5. **Run**: one process invocation. Run-to-run scatter is
   *larger* than anything a single run can see, which is why the
   `resolution` row exists and why decisions that matter want
   3-5 interleaved runs.

So: `calls = outer x inner`, the histogram's population is
`outer` samples, batches partition those samples in time, and
blocks (when asked for) partition the budget for replication.

## The header bracket

```
minstant::Instant::now() [duration=5.0s warm=1.50/3.0s outer=12,605,498 inner=21 calls=264,715,458 blocks=10 batches=193 labels=both]:
```

- `duration`: measured wall time of the run (block sleeps and
  warmups included, when present).
- `warm=used/budget`: wall seconds spent warming over the
  allowance. The first run of a process carries the settle
  budget plus the per-run cap; later runs carry the cap alone.
  See [Settle time](#settle-time).
- `outer`: samples recorded, the histogram's population.
- `inner`: calls per sample; the recorded value is the mean of
  this many back-to-back calls.
- `calls`: `outer x inner`, bench operations measured in total.
- `blocks`: only on `--blocks` runs, the block count.
- `batches`: how many time-axis chunks the pipeline flushed.
- `labels`: the active `--band-labels` style, so a saved report
  is self-describing.

## The Setup banner

Every run opens with a `Setup:` block: the TSC tick rate, the
box's power policy (cpufreq driver, governor, EPP, boost), the
pinning plan (`main pin` / `bench pin`), the frequency pin when
`--pin-freq` is live, the block knobs whenever blocks run
(`block sleep` / `block warmup`, zeros included, each naming its
consequence), the warm budget, the sleep-inhibit state, and
which config files were loaded. It is provenance for the numbers
below it, not measurement: no report before the policy lines
existed could distinguish an 8.9% governor delta from a code
change, which is why they print on every run.

The apparatus cost that used to be measured and subtracted here
is now handled by construction instead. A micro-probe times
back-to-back timer pairs at startup and sizes the inner loop so
framing is a small fraction of the workload's per-call cost; the
cost is never named as a number and never removed from a sample.
See
[in-interval vs call-to-call](../notes/design.md#timer-overhead-in-interval-vs-call-to-call)
for why the in-interval slice and the call-to-call cost are
different quantities, and why only the latter is worth measuring
for sizing.

A sub-quantum phase dither still runs between bench samples at
the seam, so a run's aggregate means do not inherit a coherent
bias from where samples happen to land on the clock lattice
([dithering](../notes/design.md#dithering-random-phase-injection)).
Per-call costs are machine- and frequency-regime-specific: see
[Frequency dependence](../notes/design.md#frequency-dependence-what-is-constant-what-is-not).
To decide whether a difference between two implementations is
real (and how many runs that takes), see
[Comparing implementations: LSC](../notes/design.md#comparing-implementations-least-significant-change).

Steadiness is graded per run rather than at startup, from the
run's own data, and prints at the foot of each report. See
[The run grade's signals](#the-run-grades-signals).

The `Setup:` banner reports the `main pin` (main's placement,
covering the warm loop and thread 0 of every bench) and
`bench pin` (per-bench thread pool) separately, plus the
`warm budget` (the once-per-process settle time and the per-run
cap); each run's report bracket then carries its own
`warm=used/budget` spend, where the first run's budget includes
the settle time and later runs' is the cap alone.

## The band table

Each bench prints a band-based histogram in nanoseconds. Each row
is one band, labeled by its **upper boundary**, the lower boundary
being the previous printed row. Bands are **right-closed**
`(lower, upper]` (like `pandas.cut`): a sample whose rank lands
exactly on a boundary counts in the band that boundary *caps*, so a
lone median sample reads `p50`, matching the upper-boundary label
and the CDF sense of a percentile. Labels are deciles in the body
(`p10` ... `p90`) and **nines/zeros** notation in both tails, where
`nK`/`zK`
mark the boundary with a fraction 10<sup>-K</sup> of samples above
(`n`) or below (`z`) it, so `n2` ≡ p99, `n3` ≡ p99.9, ... `n10`,
and `z2` ≡ p1, `z3` ≡ p0.1, `z4`. "K nines" is standard
engineering shorthand for proportions near one
([Nines (notation)](https://en.wikipedia.org/wiki/Nines_%28notation%29),
nines = −log₁₀(1−x)); `zK` is this project's mirror of it for the
fast tail (the underlying concept is the
[survival function](https://en.wikipedia.org/wiki/Survival_function)
/ CCDF tail fraction). The slow tail subdivides down to `n10`, the
fast tail only to `z4`, since a latency distribution is floored below
(nothing beats the fast path) and open above. A band only prints
when it has samples, so deep tail rows appear as run length earns
them (populating `n10` takes ~1e10 calls). Each row shows first,
last, range (`last - first + 1`), count, and mean.
The trimmed `mean`/`stdev` rows exclude every band at or above
`n2` (p99); their label names the populated non-tail span (e.g.
`mean z4..n2`, or `p20..n2` when the low tail is empty), so it
tracks the rows that are actually present rather than a fixed
`min..n2`: `min` is never a row (rows are named by their upper
boundary) and the `n2` band can itself be empty.

The full boundary ladder across its range (label styles per
`--band-labels`). The ladder is generated by
[`src/bands.rs`](../src/bands.rs), the single source of truth for
boundaries and labels, and this table is pinned by that module's
unit test, so code and docs can't silently drift:

| zpn       | frac              | ≡ percentile    | tail fraction |
|-----------|-------------------|-----------------|---------------|
| `z4`      | `0.000_1`         | p0.01           | 1e-4 below    |
| `z3`      | `0.001`           | p0.1            | 1e-3 below    |
| `z2`      | `0.01`            | p1              | 1e-2 below    |
| `p10`-`p90` | `0.10`-`0.90`   | deciles         | n/a           |
| `n2`      | `0.99`            | p99             | 1e-2 above    |
| `n3`      | `0.999`           | p99.9           | 1e-3 above    |
| `n4`      | `0.999_9`         | p99.99          | 1e-4 above    |
| `n5`      | `0.999_99`        | p99.999         | 1e-5 above    |
| `n6`      | `0.999_999`       | p99.9999        | 1e-6 above    |
| `n7`      | `0.999_999_9`     | p99.99999       | 1e-7 above    |
| `n8`      | `0.999_999_99`    | p99.999999      | 1e-8 above    |
| `n9`      | `0.999_999_999`   | p99.9999999     | 1e-9 above    |
| `n10`     | `0.999_999_999_9` | p99.99999999    | 1e-10 above   |

Every column is raw. The apparatus cost is managed by sizing
rather than by subtraction: a startup micro-probe times
back-to-back timer pairs and `inner` is chosen so framing is a
small fraction of the workload's per-call cost, which leaves a
residue small enough to ignore and, more to the point, common to
both sides of any same-harness comparison. A dither still runs
between bench samples at the seam, so aggregate means carry no
coherent phase bias. See
[design.md](../notes/design.md#dithering-random-phase-injection).

## The summary rows

Below the band table, fenced by blank lines, each row answers
one question about the whole run:

```
  mean           116.2   ns
  stdev           44.7   ns
  mean z4..n2    115.1   ns
  stdev z4..n2    13.7   ns
  quantum          0.044 ns
  resolution       0.17  ns
  mean blocks    115.9   ns
  CI95             0.4   ns
  LSC              0.5   ns
```

- **mean / stdev**: whole-histogram, tail included. One ms-scale
  outlier moves them, which is what the trimmed pair is for.
- **mean X..Y / stdev X..Y**: the same statistics over the
  populated non-tail span only (everything below `n2` = p99).
  The representative central tendency and spread; prefer these
  for comparisons.
- **quantum**: the smallest per-call step this run could
  express: one timer tick divided by `inner`. It says whether
  the rows above describe the workload or the clock lattice. A
  `range 0.0 ns` band beside a coarse quantum is lattice, not
  uniformity.
- **resolution**: the smallest delta this run can honestly
  claim to distinguish, printed on **every** run. Fit from the
  batch means: aggregate them in groups of 1, 2, 4, ... and
  watch whether variance keeps falling as `1/n`; where it stops
  falling is drift the run cannot average away, and the worst
  level is the claim (Allan deviation's move). A change smaller
  than `resolution` is *not shown* by this run, however
  convincing the means look.
- **mean blocks / CI95 / LSC**: only on `--blocks` runs. The
  mean of the block means; the 95% confidence half-width on it;
  and the least significant change against an equal-blocks run
  of something else. CI95 and LSC print `-` when
  `--block-sleep` is 0: sleepless blocks are partitions of one
  continuous run, and replication statistics built on them
  would be fiction. See
  [Comparing two implementations](#comparing-two-implementations).

Resolution, CI95, and LSC are *claims*, and a claim never prints
as a bare zero: the display extends its precision until the
leading digit shows (to at most 3 decimals, the recording floor)
and prints `<0.001 ns` below that. So `-` means "no claim
exists", `<0.001 ns` means "a claim too small to spell", and
they are different statements.

## Warnings

Runs inhibit system sleep by default (see `--no-inhibit` in
[usage.md](usage.md#flags)), so these mainly matter for
uninhibited runs. A report may end with `WARNING` lines (printed
last so they can't scroll out of mind) flagging that `max` and
the untrimmed mean/stdev are poisoned. The few inflated samples
land in the extreme tail band, so percentile boundaries, the
bands below the tail, and the trimmed `mean`/`stdev` rows remain
usable:

- **system suspended**: the run spanned a system suspend,
  detected by `CLOCK_BOOTTIME` vs `CLOCK_MONOTONIC` elapsed
  divergence. A mid-sample suspend inflates that one sample by
  the whole sleep gap.
- **sample(s) clamped**: a sample exceeded the histogram's 60 s
  bound and was recorded as 60 s instead of aborting the run
  (visible as a pileup at `max`).

## A report, walked through

Measurements below are on a Ryzen 9 3900X, idle desktop. Numbers
vary run-to-run and machine-to-machine; the *shape* of the
differences is the useful signal.

Each row is one *populated* band (see the boundary ladder above);
empty bands are skipped. Columns:

- **first / last**: the smallest and largest sample *values* in the
  band; `first` of the top row is the fastest call observed.
- **range**: `last − first + 1`, the band's width.
- **count**: samples in the band.
- **mean**: the band's mean, raw. Nothing is subtracted (see
  [The Setup banner](#the-setup-banner)).

Below the bands, `mean` / `stdev` are whole-histogram; the trimmed
`mean X..Y` / `stdev X..Y` drop the `≥ p99` tail so a few ms-scale
outliers don't poison them, and their label names the populated
non-tail span.

**How samples map to bands.** A sample's rank is its
[Hazen plotting position](https://splashback.io/2021/05/hazen-percentile/)
(Allen Hazen, 1914) `mid_rank = (i − 0.5) / n` (`i` = 1-based rank,
`n` = sample count). Bands are **right-closed** `(lower, upper]`, so the
`(` is *open* (excludes the lower boundary), the `]` is *closed*
(includes the upper), so a band holds the ranks
`band_lower < N ≤ band_upper`. A rank landing exactly on a boundary
therefore counts in the band that boundary *caps*. That's the
[`pandas.cut`](https://pandas.pydata.org/docs/reference/api/pandas.cut.html)
convention; computing's other default is left-closed `[lower, upper)`
([`numpy.histogram`](https://numpy.org/doc/stable/reference/generated/numpy.histogram.html),
language ranges,
[Dijkstra EWD831](https://www.cs.utexas.edu/~EWD/transcriptions/EWD08xx/EWD831.html)).
Right-closed matches this report's upper-boundary labels: "the `p50`
row" = samples *up to and including* the 50th percentile.

Ten distinct values (`n = 10`) spread one per band:

| value `i` | `mid_rank = (i−0.5)/10` | band  | interval `(lower, upper]`     |
|----------:|:-----------------------:|:------|:------------------------------|
| 1         | 0.05                    | `p10` | `(0.01, 0.10]` = `(z2, p10]`  |
| 2         | 0.15                    | `p20` | `(0.10, 0.20]`                |
| 3         | 0.25                    | `p30` | `(0.20, 0.30]`                |
| 4         | 0.35                    | `p40` | `(0.30, 0.40]`                |
| 5         | 0.45                    | `p50` | `(0.40, 0.50]`                |
| 6         | 0.55                    | `p60` | `(0.50, 0.60]`                |
| 7         | 0.65                    | `p70` | `(0.60, 0.70]`                |
| 8         | 0.75                    | `p80` | `(0.70, 0.80]`                |
| 9         | 0.85                    | `p90` | `(0.80, 0.90]`                |
| 10        | 0.95                    | `n2`  | `(0.90, 0.99]` = `(p90, n2]`  |

A **single sample** is the degenerate case (every percentile
collapses to that one value) and `mid_rank = (1 − 0.5)/1 = 0.5`
lands it in `p50` (since `0.40 < 0.50 ≤ 0.50`):

| `n` | `mid_rank` | band  |
|----:|:----------:|:------|
| 1   | 0.50       | `p50` |

**Investigating with `-d`.** Because membership is by rank, shrinking
the duration to force a known sample count is a handy way to watch
exactly where values land (the exact `-d` is machine-dependent; tune
it to the count you want; there are no timing guarantees):

```
$ iiac-perf zcr -d 0.000001        # a handful of samples
  p30 0.30       2.8 ns    2.8 ns    0.0 ns    2    2.8 ns      2.0 ns
  p70 0.70       3.0 ns    3.0 ns    0.0 ns    1    3.0 ns      2.3 ns
  p90 0.90       4.2 ns    4.2 ns    0.0 ns    1    4.2 ns      3.4 ns
  mean p30..p90                                     3.2 ns      2.4 ns

$ iiac-perf zcr -d 0.0000001       # one sample -> collapses to p50
  p50 0.50       6.3 ns    6.3 ns    0.0 ns    1    6.3 ns      5.5 ns
  mean p50                                          6.3 ns      5.5 ns
```

## Comparing two implementations

"Is B really faster than A, or is it noise?" The workflow:

```
iiac-perf mpsc-2t --pin-cpus 0,1 --blocks 10 --block-sleep 1-10ms --block-warmup 2ms -d 10
```

`--blocks 10 -d 10` divides the 10-second measuring budget
into **10 blocks of ~1 s each**: same total measurement, now
with an error bar, because `--block-sleep` makes each block a
mini-run (its sleep draw re-rolls scheduler/frequency state,
`--block-warmup` keeps the post-wake ramp out of the samples,
then the block measures its share of the budget). Both knobs
default to 0: sleepless blocks are partitions of one
continuous run, and CI95/LSC print `-` rather than a number
built on replication that never happened. Always pin
(`--pin-cpus`): unpinned, the OS's thread placement is re-rolled
per *process* and dominates run-to-run drift, which blocks
can't see. The report then ends with:

```
  resolution      12.41  ns
  mean blocks  4,745.953 ns
  CI95            16.115 ns
  LSC             21.169 ns
```

- **resolution**: printed on **every** run, blocks or not: the
  batch-curve drift floor, the smallest delta this run can
  honestly distinguish. Batch means are aggregated in groups
  of 1, 2, 4, ... and where their variance stops falling as
  `1/n` is drift the run cannot average away.
- **mean blocks**: the run's headline number: the mean of the
  10 block means.
- **CI95**: 95% confidence interval (half-width) on that
  mean: "the true value is within ±16 ns of 4,746, as far as
  this run can tell."
- **LSC**: least significant change: run the *other*
  implementation the same way (same `-d`, same `--blocks`,
  same knobs, same pin), and if the two `mean blocks` differ
  by more than roughly the larger of the two `LSC`s, the
  difference is real at 95% confidence.

Caveat: the block rows see *within-invocation* variation
only. Some per-process state survives even long sleeps
(measured ~0.6% residual drift even pinned, on an idle Ryzen
5 7600X), so treat `LSC` as a lower bound and `resolution` as
the honest single-run claim; for a decision that matters, run
each implementation 3-5 times interleaved (A,B,A,B,...) and
apply the same comparison to the per-run `mean blocks`
values. Method and worked numbers:
[Comparing implementations](../notes/design.md#comparing-implementations-least-significant-change),
[block validation](../notes/design.md#block-validation-results-0210-4-r5-7600x).

## The two grades

Every report ends with the grade block: one column header over
three rows, each graded A-F from its own data: two `env` rows
for the **box**, one `run` row for *that run*. A row's `worst`
column is its composite, printed beside the signals that earned
it; a blank cell (`-`) means that signal does not apply to that
row, which is the env/run signal mapping made visible:

```
  grade  phase                        settle  worst     spread  bursts  interference      drift               step
  env    warmup   4.84->5.24GHz 49% +-0.0% A      A    0.47% A       -       0.00% A    0.00% A            0.00% A
  env    bench                             -      F    0.48% A       -       0.00% A   11.05% F    11.05% @1.06s F
  run    all                               -      F          -   37% B       0.04% A   10.49% F    10.49% @1.04s F
```

Column reference (each signal prints its own letter beside its
value; the sections below carry the depth):

- `grade` / `phase`: row labels. The two `env` rows grade the
  box from micro-probes that never touch the bench (`warmup`:
  did it end settled; `bench`: did it stay settled). The `run`
  row grades the numbers above it, from the run's own batches.
- `settle`: warmup row only, and a graded signal like the
  rest: the clock's journey, the settled share of the warm,
  how still it held, and the share's letter. `00%` is
  never-settled, an F; see [Settle time](#settle-time).
- `worst`: the row's composite letter, its worst signal
  outright; always one of the letters printed beside it. On
  the warmup row the settle letter counts as a signal, which
  is the one place the clock decides a grade.
- `spread`: env rows only. How wide a probe's bulk sits above
  its own floor. A timer pair has no workload character, so
  width means the box itself moved.
- `bursts`: run row only. The fraction of batches whose mean
  sits above the run's median batch: whether interference was
  localized in time or spread out.
- `interference`: samples that sat above their batch's floor,
  as a fraction of the run: how much other work leaked in.
- `drift`: floor movement from the run's first quarter to its
  last: did the run finish where it started.
- `step`: the largest floor shift at any split of the run, and
  when (`10.49% @1.04s`): catches a shift-and-return that
  drift's endpoints miss.

The `env` rows are two phases of one probe series, scored
separately: `warmup` is the last 300 ms of the probes taken
before the bench ran ("did the box end settled"), `bench` the
probes taken alongside it ("did it stay settled"). They are
graded apart rather than as one series because absorbing a ramp
is exactly what warmup is *for*: blended, the boundary between a
cold warmup and a hot run reads as a large step that nothing
actually did wrong. The block prints no combined env letter:
each phase's `worst` is visible, and the worse of the two is
what `qualify-environment` computes for its verdict.

The rows answer different questions, and reading them together
says more than either alone. `run` describes the numbers above
it,
and a run's steadiness is largely its workload's character, so a
blocking round-trip reads worse than a spinning one, correctly.
`env` describes the machine: it comes from micro-probes that time
timer pairs and never touch the bench, so no workload character
enters it. An `env` A beside a `run` D means a bursty workload on
a quiet box. The same letter in both, at the same instant, means
the box moved and took the bench with it, as in the example
above: `min-now` on a 7600x, where the environment reports an
11.05% step at 1.06 s and the run reports a 10.49% step at
1.04 s. Same magnitude, same instant, from two series that share
a time axis but not an instrument. Neither grade could make that
call alone.

## Settle time

The warmup row's `settle` cell is the story of the CPU clock
during warmup, read left to right:

```
4.84->5.24GHz 49% +-0.0% A
```

- `4.84->5.24GHz`: the journey, the delivered clock at the
  first reading and the state it settled into (the median of
  the settled stretch). An arrow that goes nowhere
  (`4.09->4.09GHz`) is a box that was already at speed.
- `49%`: the settled share of the warm, how much of warmup was
  spent in the settled state, zero-padded to two digits so the
  column aligns. `100%` is a box that was ready all along, a
  small share is one that settled at the last moment (the
  floor is the exit window, at least 50 ms, as a share of the
  warm, since warmup exits the moment that window reads
  settled), and `00%` is reserved for never settled: a settled
  share always rounds up to at least `01%`.
- `+-0.0%`: how still "settled" is, the relative standard
  deviation of the clock across the settled stretch. A pinned
  clock certifies itself here, and a governor still wandering
  shows a fatter band.
- `A`: the share's letter, a graded signal like the rest: A at
  a quarter of the warm settled or more, B from 10%, C from
  5%, D below that, and F within 2% of never, never included.
  It folds into the warmup row's `worst`, the one place the
  clock decides a grade: a fast late ramp can finish inside
  the bench's first batches where no timing detector sees it,
  so a buzzer-beater settle reads D and a box that never
  settled reads F with the `00%` cell naming the cause.

`4.07->4.54GHz 00% F` is that last case: still moving when
warmup gave up, no share of the warm certified, and the numbers
that follow were measured on a moving clock. A cell with no GHz
(`49%` alone) appears only on a box whose driver exposes no
delivered clock: timing is all there is, and the letter grades
it without penalty. On a box with a readable clock, a settled
stretch the clock cannot verify (fewer than two readings from
the sampled core landed in it) does not certify at all and
reads `00% F`: an unverified claim must not outgrade a
verified bad one.

"Settled" means what the warm exit means: from some point on,
the probe timings grade A *and* the delivered clock held inside
1%. The cell reports the earliest such point: the journey ends
at it, the settled share runs from it to the end of warmup, and
the steadiness is measured across it. Every clock number reads the
single most-sampled CPU, because an unpinned run's sampler
rides the scheduler across cores and a mixed series rates
placement rather than the clock (measured on a 3900X: +-11.9%
mixed against +-0.2% for the same box filtered or pinned). It
never says which state was the *right* one: the journey is
measured against where this warmup ended, not any absolute
best speed.

`-v` adds a `clock:` line under the warmup probe table: the
journey, one tick per clock step (`^`/`v` a move beyond the 1%
band, `-` a hold inside it), and the series extremes. The
settled stretch reads all `-` by construction, so the settle
point is visible as the place the ticks go quiet.

The cell exists because warmup *absorbs* the box coming up to
speed rather than being graded on it: the first run of a
process spends `--settle-time` seconds (default 1.5) stepping
the bench before recording anything, so the letter answers "was
it settled when measurement started" and this cell answers "at
what state, and settled for how much of the warm".

The warm is per **process**, not per bench: the boost it wins is
machine state, so every later bench in the same process inherits
it. Without it the first bench of a process reports a cold
machine's numbers (measured at ~8.6% slow on a 7600x, a wrong
histogram rather than merely a wrong letter) while benches 2..N
read correctly. Cost is ~2% of an `all -d 5` sweep.

`--settle-time 0` skips the warm, which is how you measure what
it is worth on a given box. A box that reads `00%` at the
default wants more, though that is not always curable: on a
3900X the floor is bistable and moves at arbitrary times, so a
3.5 s warm still left runs moving mid-bench. Replication
(`--blocks`) is the answer there, not a longer warm.

The probes run through warmup and then in the seam at every batch
boundary, so the series covers the whole run on the same time
axis as the batches. `--no-env-probe` limits them to warmup (so
only the warmup row appears),
which costs the grade its span; it exists because seam probing
perturbs a spinning multi-threaded bench by ~0.9% (measured on
`zcr-spsc-v0-2t`), a bias that is common-mode in an A/B between two
benches but not in an absolute number.

The `env` signals differ slightly from the run's: `spread` (how
wide a probe's bulk sits above its own floor) replaces `bursts`,
because a bench's spread is mostly its workload while a timer
pair has no character of its own. Note that `env interference`
is the weakest of the four: a probe measures the box only while
the measuring thread is running, so preemption is largely
invisible to it and `spread`/`drift`/`step` carry the detection.

## The run grade's signals

- `interference`: samples that sat above their batch's floor, as
  a fraction of the run. How much other work leaked in.
- `bursts`: batches whose mean sits above the run's median batch.
  Whether that interference was localized in time or spread out.
- `drift`: floor movement from the run's first quarter to its
  last. Did the run finish where it started.
- `step`: the largest floor shift any split of the run divides,
  and when. Catches a shift that drift's endpoints miss: a run
  that moves and moves back reads low on `drift` and high on
  `step`.

**The overall letter is the worst signal, outright.** Each signal
scores 0-4 by counting how many of its four ascending cutoffs it
crosses: below all four is 0 = A, above all four is 4 = F (there
is no E). The composite is the maximum of those scores, so one F
anywhere makes the run F and no number of A's pulls it back;
`step` alone earns the F in the example above. That is why every
signal prints its own letter: a row's `worst` is always one of
the letters shown beside it.

The `env` rows work the same way, over their own four signals.
There were once six, when the grade was measured at startup from
a calibration fit: two of them scored how well that *fit* held
(the worst residual of a ladder point against the Theil-Sen line,
and the loop-only slope against a dithered two-point fit). A
bench run fits nothing, so those two have no run-side analog and
none was invented; the reasoning is recorded in
[chores-05.md](../notes/chores/chores-05.md#six-calibration-signals-four-run-signals).

Both floor signals compare medians, not extremes, so one hot
batch is a burst rather than a shift.

**It reports; it does not warn.** A low letter is not a fault to
fix. A run's steadiness is largely its workload's character: a
multi-threaded bench carries OS involvement in its own numbers
(scheduling, placement, park/unpark) so on a quiet box `mpsc-2t`
reads `step` F while `mpsc-2t-spin`, the same round-trip spinning
instead of parking, reads A. Both letters are true descriptions.
The report's job is a histogram faithful to what was measured,
and the grade is part of that description rather than a verdict
on it.

Where it earns its keep is the comparison you came for: before
trusting a delta between two runs, check that neither of them
straddled a shift. Comparing the letter between runs of the same
bench is meaningful; comparing it across different benches is
not.

This is a different question from the `env` rows, which grade the
box rather than the run. Judging the box from a bench's own
samples is not possible after the fact, since they mix the two
inseparably; the `env` grade instead comes from micro-probes that
time timer pairs and never touch bench code, so no workload
character enters it.

## What to conclude: a worked example

A real session (3900X, 2026-08-19) that exercises most of the
report's surfaces. First, find the frequency the box holds under
the actual workload:

```
$ sudo iiac-perf suggest-freq zcr-mpsc-2t --pin-cpus 0,1
...
candidate 3801 MHz: held. Delivered 3.77-3.77 GHz, median 3.77, 130 samples
suggestion: the highest held pin is 3801 MHz ...
pin_mhz = 3801
```

Two readings before moving on: the delivered clock is 3.77 GHz,
not 3.801: pinned at the ACPI nominal, amd-pstate delivers ~0.8%
under it, matching the box's `ticks/ns` (3.7928). The verdict's
1% tolerance absorbs that gap knowingly. And a hold is *per
schedule*: a different bench, duration, or pin layout may hold a
different clock.

Then the same bench at the pinned clock across three placements
(`--pin-freq=3801`, trimmed means and their resolutions):

| placement    | `--pin-cpus` | trimmed mean | resolution | grade notes            |
|--------------|--------------|-------------:|-----------:|-------------------------|
| SMT siblings | `0,12`       |      61.5 ns |    0.06 ns | all A                    |
| same CCX     | `0,1`        |     107.0 ns |    0.13 ns | all A                    |
| cross-CCX    | `0,6`        |     395.9 ns |    0.98 ns | run C (interference 10.85%) |

What the surfaces say, and what to conclude:

- **The settle cell** read `3.77->3.77GHz 99% +-0.0% A` on every
  run: an arrow that goes nowhere is a box already at speed, and
  a pinned clock certifies itself in the `+-0.0%`. The pin also
  verifies in the env `spread` staying at 0.03%.
- **The placements differ 1 : 1.7 : 6.4**, and every gap is far
  above every resolution: these differences are real, no second
  run needed for that conclusion.
- **The clock pin isolated a mechanism.** Unpinned boosted runs
  of the same placements (notes/placement-map.md) read 51.8 /
  98.3 / 401.7 ns. Down-clocking to 3.79 GHz slowed the near
  placements ~10-19% but moved cross-CCX barely at all: its cost
  lives in the fabric/IO-die clock domain, which the core pin
  does not touch. We think that is the mechanism; the pinned
  sweep is the isolating evidence.
- **The resolution row scaled with the placement** (0.06 to
  0.98 ns): the fabric route genuinely drifts more within a run,
  and the cross-CCX run's interference C says the same thing
  from a different instrument. A tuning campaign on that
  placement gets ~1 ns of single-run resolution, not 0.06.

### The two-regime workflow

Tune pinned, confirm unpinned:

- **Tuning runs** pin the clock (`--pin-freq`, target from
  `suggest-freq`) so the resolution shrinks until "did this
  tweak clear it" resolves in a run or two.
- **Reporting runs** keep the wandering clock, whose number is
  what the real world sees.

We think a pinned ranking can occasionally flip unpinned (boost
behavior interacts with how a workload holds cores), which is
why the confirm step exists. The pyperf tune/reset pair is the
same idea; ours is `pin-freq` / `restore-freq`.

### Duty cycle selects the state

An earlier 3900X lesson (2026-08-02) that reads wrong without
this guide: the same bench measured ~21.8 ns under sustained
load and 24.0 ns when run as 5 ms bursts between sleeps, both
grade A. The box is bistable, the duty cycle selects the state,
and **grade A certifies internal consistency of the state the
run held, not a canonical number**. So an A/B comparison wants a
matched duty cycle (same `-d`, same `--blocks`, same knobs) as
much as it wants a matched build, and a pinned clock
(`--pin-freq`) removes the state selection entirely.

## Label styles (`--band-labels`)

`--band-labels` selects the row-label vocabulary; the trimmed
`mean`/`stdev` rows and the report header's `labels=` metadata
follow the same style. The trimmed label names the **populated**
non-tail span, and here `min` is never a row (no samples land in the
fast tail), so it reads `p50..n2`, not a fixed `min..n2`. Default
`both` prints the zpn name and its literal fraction side by side
(the juxtaposition teaches the zpn vocabulary):

```
$ iiac-perf min-now -d 1 --band-labels both
minstant::Instant::now() [duration=1.0s outer=1,539,764 inner=23 calls=35,414,572 batches=24 labels=both]:
                       first          last         range        count          mean
  p50 0.50           24.0 ns       24.0 ns        0.0 ns    1,303,881       24.0 ns
  p90 0.90           24.0 ns       24.0 ns        0.0 ns       44,597       24.0 ns
  n2  0.99           24.4 ns       28.8 ns        4.4 ns      175,893       24.6 ns
  ...
  n7  0.999_999_9 2,170.9 ns    2,814.0 ns      643.1 ns            2    2,492.4 ns
  mean                                                                      24.2 ns
  stdev                                                                      7.9 ns
  mean p50..n2                                                              24.0 ns
  stdev p50..n2                                                              0.3 ns
  grade  phase        settle  worst     spread  bursts  interference      drift               step
  env    warmup        0.09s      A    0.30% A       -       0.00% A    0.00% A            0.00% A
  env    bench             -      A    0.30% A       -       0.01% A    0.00% A            0.00% A
  run    all               -      A          -    0% A       0.06% A    0.00% A            0.00% A
```

`zpn` drops the fraction (names only); `frac` drops the name
(fractions only, so the trimmed label reads `0.50..0.99`). Same
bench, separate runs; only the leftmost column and the trim
label change:

```
$ iiac-perf min-now -d 1 --band-labels zpn        $ iiac-perf min-now -d 1 --band-labels frac
  ... labels=zpn]:                                   ... labels=frac]:
  p50    ...                                         0.50      ...
  n2     ...                                         0.99      ...
  ...                                                ...
  mean p50..n2     24.0 ns                           mean 0.50..0.99     24.1 ns
  stdev p50..n2     0.3 ns                           stdev 0.50..0.99     0.3 ns
```

## `all` results (7600X, 0.27.0-5)

One `iiac-perf all --record` run on a headless 7600X, unpinned,
five seconds per bench, whole-run mean per bench from the
records. The three probe-only benches (`producer-consumer`,
`tp-pc`, `tp2-pc`) write no bench-level record and are not in
the table. Raw values, so each includes the apparatus cost
described in [The Setup banner](#the-setup-banner). Shapes, not
absolutes, and the earlier table (3900X, 0.23.0-7) is in this
file's history.

| bench          |       mean | class | wait  | note                          |
|----------------|-----------:|-------|-------|-------------------------------|
| min-now        |    16.2 ns |       |       | `minstant::Instant::now`      |
| std-now        |    16.2 ns |       |       | `std::time::Instant::now`     |
| mpsc-1t        |    12.5 ns | MPSC  |       | std channel, same thread      |
| mpsc-2t        | 4,868.1 ns | MPSC  | park  | blocking `recv`               |
| mpsc-2t-spin   |   120.2 ns | MPSC  | spin  | `try_recv` + `spin_loop`      |
| probe-mpsc-2t  | 5,145.9 ns | MPSC  | park  | `mpsc-2t` with probes         |
| cb-chan-1t     |     8.6 ns | MPMC  |       | crossbeam channel, same thread |
| cb-chan-2t     |   196.2 ns | MPMC  | park  | blocking `recv`, see below    |
| cb-seg-1t      |     8.0 ns | MPMC  |       | `SegQueue`, same thread       |
| cb-seg-2t      |   117.4 ns | MPMC  | spin  | `SegQueue`, spin on `pop`     |
| ice-ps-1t      |   164.6 ns |       |       | iceoryx2 pub/sub, 1 thread    |
| ice-ps-2t      |   449.0 ns |       | spin  | iceoryx2 pub/sub, 2 threads   |
| ice-rr-1t      |   474.5 ns |       |       | iceoryx2 req/res, 1 thread    |
| ice-rr-2t      |   684.3 ns |       | spin  | iceoryx2 req/res, 2 threads   |
| zcr-spsc-v0-1t |     1.9 ns | SPSC  |       | zc-ring-x1 spsc v0, 1 thread  |
| zcr-spsc-v0-2t |   123.5 ns | SPSC  | spin  | zc-ring-x1 spsc v0, 2 threads |
| zcr-mpsc-1t    |     2.5 ns | MPSC  |       | zc-ring-x1 mpsc, 1 thread     |
| zcr-mpsc-2t    |    69.1 ns | MPSC  | spin  | zc-ring-x1 mpsc, 2 threads    |

**The class column is the first thing to read across rows.** The
queues promise different things: crossbeam's channel and
`SegQueue` are MPMC, any number of producers and consumers; std's
channel and zc-ring-x1's mpsc ring are MPSC; the zc-ring-x1
spsc v0 ring is SPSC, one of each. A queue that promises less is
expected to be faster, since it has fewer writers to order, so an
SPSC row under an MPMC row is not the same contest won. What
zc-ring-x1 is building next, a segmented SPSC, lands against
`cb-seg-*`, the ecosystem's unbounded segmented queue and its
closest structural peer, and the class sentence applies there
too.

**The wait column splits the 2-thread rows more than the queue
does.** The parking rows (`mpsc-2t` and the probe family, both
blocking `recv`) sit near 5 µs while every spinning row is under
700 ns. `cb-chan-2t` is a parking row that mostly does not park:
crossbeam's `recv` spins briefly before it sleeps, so a
round-trip lands on the spin path or the park path by timing.
Its band table is bimodal, a third of the mass near 140 ns and a
fifth near 420 ns in a five-second run, and the interference
census reads that split as contamination and grades the run F.
The F is the wait policy, not the box, and the mean is a blend of
two paths. `mpsc-2t`, the same channel under std's wrapper, parks
almost every time, which is the 5 µs.

**Two readings the crossbeam rows give.** Same thread, the
channel and `SegQueue` cost about the same (8.6 and 8.0 ns) and
std's `mpsc` costs 12.5 ns over the same crossbeam code, so the
std wrapper is about 4 ns per round-trip. Across threads at the
same spin policy, `SegQueue` at 117 ns sits with `mpsc-2t-spin`
at 120 ns and `zcr-spsc-v0-2t` at 124 ns, and zc-ring-x1's mpsc ring
at 69 ns is the fastest handoff in the table. We think the mpsc
ring's one shared hot word per slot beats the index cache lines
the others bounce, an exploration tracked in zc-ring-x1's todo.

## Verbose output (`-v`)

`-v` prints the affinity lifecycle on stderr. Main pins only
when `--pin-cpus` is given (to the pool's first slot, where it warms
and measures); otherwise every mask stays as the process
launched.

```
$ iiac-perf mpsc-2t -d 3 -v
iiac-perf 0.23.0-7 — Rust latency microbenchmark harness

[INFO  iiac_perf] startup affinity: 0-23 (24 cpus)
[DEBUG iiac_perf] affinity for warm + run: 0-23 (24 cpus)
[DEBUG iiac_perf] ticks_per_ns: 3.792852
Setup:
  ticks/ns          3.792852
  main pin          none (scheduler placement)
  bench pin         none (unpinned)
  sleep inhibit     active (systemd-inhibit --what=sleep)
  config            none (built-in defaults)

std::sync::mpsc round-trip (2 threads) [duration=3.0s outer=363,598 inner=1 calls=363,598 batches=55 labels=both]:
                         first              last             range     count              mean
  z4  0.000_1         391.2 ns          401.2 ns           10.0 ns        15          400.1 ns
  z3  0.001           410.1 ns          411.1 ns            1.0 ns       409          410.9 ns
  z2  0.01            420.1 ns        6,361.1 ns        5,941.0 ns     3,215        1,133.2 ns
  p10 0.10          6,365.2 ns        6,656.0 ns          290.8 ns    35,233        6,596.7 ns
  ...
  p90 0.90          9,199.6 ns        9,412.6 ns          213.0 ns    35,618        9,298.0 ns
  n2  0.99          9,420.8 ns       11,403.3 ns        1,982.5 ns    32,814        9,793.5 ns
  n3  0.999        11,411.5 ns       16,662.5 ns        5,251.1 ns     3,272       13,153.5 ns
  n4  0.999_9      16,678.9 ns       91,160.6 ns       74,481.7 ns       329       25,497.9 ns
  n5  0.999_99     93,388.8 ns    1,265,631.2 ns    1,172,242.4 ns        32      383,158.3 ns
  n6  0.999_999 1,266,679.8 ns    1,782,579.2 ns      515,899.4 ns         4    1,443,364.9 ns
  mean                                                                              8,089.7 ns
  stdev                                                                             6,804.9 ns
  mean z4..n2                                                                       7,981.4 ns
  stdev z4..n2                                                                      1,296.2 ns
  grade  phase        settle  worst     spread  bursts  interference      drift               step
  env    warmup        0.84s      B    2.10% B       -       0.02% A    0.00% A            0.00% A
  env    bench             -      A    0.33% A       -       0.00% A    0.00% A            0.00% A
  run    all               -      F          -   36% B       5.04% C   10.67% F    25.20% @0.58s F
```

Notice `z4 first = 391 ns`, sub-µs. That's the
"both-ends-hot-and-spinning" fast path, where the scheduler has
co-located bench threads on the same CCX and neither has parked
in a futex. It survives because an unpinned run never pins main,
so the scheduler keeps its placement freedom.

## Default vs `--pin-cpus 0,1`

Default (unpinned bench): wide dispersion, but the fast path is
visible.

```
$ iiac-perf mpsc-2t -d 3
Setup:
  ...
  main pin          none (scheduler placement)
  bench pin         none (unpinned)

std::sync::mpsc round-trip (2 threads) [duration=3.0s outer=363,056 inner=1 calls=363,056 batches=55 labels=both]:
  z4  0.000_1         240.1 ns          400.1 ns          160.0 ns        29          374.3 ns
  ...
  n2  0.99          9,363.5 ns       11,255.8 ns        1,892.4 ns    32,539        9,738.6 ns
  n6  0.999_999 1,460,666.4 ns    1,771,044.9 ns      310,378.5 ns         4    1,566,048.3 ns
  mean                                                                              8,104.5 ns
  stdev                                                                             6,693.1 ns
  mean z4..n2                                                                       8,000.6 ns
  stdev z4..n2                                                                      1,312.5 ns
  grade  phase        settle  worst     spread  bursts  interference      drift               step
  env    warmup            -      A    0.33% A       -       0.01% A    0.00% A            0.00% A
  env    bench             -      F    0.33% A       -       0.01% A    0.00% A    12.62% @3.01s F
  run    all               -      F          -   40% B       3.19% B    0.58% A    12.19% @1.16s F
```

Pinned to two physical cores in the same CCX: tighter body, lower
mean.

```
$ iiac-perf mpsc-2t --pin-cpus 0,1 -d 3
Setup:
  ...
  main pin          core 0 (pool slot 0; warm + run)
  bench pin         [0, 1] (2 slots, 2 unique CPUs)

std::sync::mpsc round-trip (2 threads) [duration=3.0s outer=417,477 inner=1 calls=417,477 batches=55 labels=both]:
  z4  0.000_1         391.2 ns          470.0 ns           78.8 ns        42          421.2 ns
  ...
  n2  0.99          7,487.5 ns        9,027.6 ns        1,540.1 ns    37,406        7,864.7 ns
  n6  0.999_999 2,929,721.3 ns    3,066,036.2 ns      136,314.9 ns         4    2,988,441.6 ns
  mean                                                                              7,039.8 ns
  stdev                                                                            13,632.2 ns
  mean z4..n2                                                                       6,897.4 ns
  stdev z4..n2                                                                        511.3 ns
  grade  phase        settle  worst     spread  bursts  interference      drift               step
  env    warmup            -      A    0.26% A       -       0.01% A    0.00% A     0.53% @0.07s A
  env    bench             -      F    0.36% A       -       0.01% A   19.12% F    19.12% @1.49s F
  run    all               -      D          -   29% B       0.68% A    9.83% D     9.53% @0.78s D
```

Side-by-side (using the trimmed `z4..n2` rows, which exclude the
ms-scale OS-preemption outliers in the `n3`-`n6` tail bands):

| metric          | default    | `--pin-cpus 0,1` | Δ      |
|-----------------|-----------:|------------:|-------:|
| `mean z4..n2`   |   8,001 ns |    6,897 ns | −14 %  |
| `stdev z4..n2`  |   1,313 ns |      511 ns | −61 %  |
| `stdev` untrimmed |  6,693 ns |   13,632 ns | +104 % |

So `--pin-cpus 0,1` buys a tighter, lower-mean body at the cost of
being more exposed to a rare preemption: bound to one core, a
single outlier pushes the max to ms-scale, which is why the
untrimmed `stdev` moves the *wrong* way. Use the
`mean/stdev z4..n2` rows for representative central tendency and
spread.

Both runs kept the sub-µs `z4` fast path, where the scheduler has
co-located the threads and neither end has parked. Do not read
the `z4 first` difference between these two runs as an effect of
pinning: that column is the extreme of a sparse tail and moves
run to run by more than the gap between them.
