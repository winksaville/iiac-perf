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
   the workload's, and it sets the **quantum**, the smallest value
   step a sample can express.
3. **Block** (`--blocks N`): consecutive samples, with an
   optional sleep and unrecorded warmup in front, every block
   sized to one sample count from the warmup's typical step
   cost, and stopped early at twice its share of the budget
   when the bench runs slower than that (the report says how
   many). Blocks are the run's **time axis** and its
   **replication axis** at once. The grade block's drift, step,
   bursts, and interference signals, the delivered-clock
   series, and the `resolution` row are computed over the
   block series, and `mean` is its count-weighted average. The
   default `--block-sleep` of 1-10 ms makes each block a
   mini-run separated by a state-re-rolling sleep, and the
   spread of block means yields `CI95 blocks` and `LSC blocks`.
   Every run has blocks, 100 by default, so a five-second run's
   blocks are about 50 ms. With `--block-sleep 0`, blocks are
   partitions of one continuous run and `CI95 blocks` and
   `LSC blocks` print `-`, and below eight blocks the stats that
   need more print `-` as well.
4. **Run**: one child process. Every bench runs `--runs` of them
   back to back, 5 by default, and a process start re-rolls where
   the bench's memory lands, which sets its level. Run-to-run
   scatter is *larger* than anything a single run can see, since
   every block of a run shares that one draw, which is why the
   `resolution` row exists within a run and why a bench's
   `CI95 runs` and `LSC runs` are computed over its run means
   ([A bench's runs](#a-benchs-runs)).
5. **Series**: one invocation's runs, which a record names in
   its `series` field. Nothing in a report is computed across
   invocations: that comparison is the reader's, by the method in
   [Comparing two implementations](#comparing-two-implementations).

So: `calls = samples x inner`, the histogram's population is
`samples`, blocks partition those samples in time, and replicate
the run when a sleep separates them, and runs replicate the
bench.

## The header bracket

```
minstant::Instant::now() [duration=5.6s measured=5.0s warm=1.50/3.0s samples=12,605,498 inner=21 calls=264,715,458 blocks=100 labels=both]:
```

- `duration`: wall time of the run, block sleeps and warmups
  included.
- `measured`: seconds spent inside blocks recording samples, the
  part of `duration` the `-d` budget buys. With the default
  sleep, `duration` runs about half a second longer at 100
  blocks. A `measured` well short of `-d` means the time cap
  cut blocks, and the report's note says how many.
- `warm=used/budget`: wall seconds spent warming over the
  allowance. The first run of a process carries the settle
  budget plus the per-run cap, and every run is the first of its
  own process now, so every run carries both.
  See [Settle time](#settle-time).
- `samples`: samples recorded, the histogram's population.
- `inner`: calls per sample. The recorded value is the mean of
  this many back-to-back calls.
- `calls`: `samples x inner`, bench operations measured in total.
- `blocks`: the block count, on every run.
- `labels`: the active `--band-labels` style, so a saved report
  is self-describing.

Reports saved before blocks became the time axis, some of the
worked examples below among them, carry a `batches=` token
here instead, the count of the retired time-axis chunks.

## The Setup banner

Every run opens with a `Setup:` block: the TSC tick rate, the
box's power policy (cpufreq driver, governor, EPP, boost), the
pinning plan (`main pin` / `bench pin`), the frequency pin when
`--pin-freq` is live, and the sleep-inhibit state. It is
provenance for the numbers below it, not measurement: no report
before the policy lines existed could distinguish an 8.9%
governor delta from a code change, which is why they print on
every run.

A `Config:` list follows it: the config files loaded, then every
run parameter with its resolved value and its source, `(default)`,
the file that set it, or the flag. A file or flag restating the
built-in is marked `same as default`, since removing it would
change nothing. The `freq` line is the declared `[freq]` steady
state and its file, or `none declared`: no run reads it unless it
pins, but every pin and restore returns to it, and a project-local
table replaces the XDG one whole. The block knobs print zeros included, an invisible
sleep shaping results being the failure mode they replaced.

```
Config:
  files             iiac-perf.md
  duration          300 ms   (-d)
  pin_cpus          2,3      (--pin-cpus)
  freq              powersave, EPP balance_performance, boost on, clamp 1745-4673 MHz  (~/.config/iiac-perf/config.md)
  blocks            10       (--blocks)
  block_sleep       1-10 ms  (iiac-perf.md, same as default)
  block_warmup      2 ms     (iiac-perf.md)
  settle_time       1.5 s    (default)
  ...
```

The apparatus cost that used to be measured and subtracted here
is now handled by construction instead. A micro-probe times
back-to-back timer pairs at startup and sizes the inner loop so
framing is a small fraction of the workload's per-call cost. The
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
`bench pin` (per-bench thread pool) separately, the pool's
placement closing its line, and the `Config:` list the warm
budget, `settle_time` (once per process)
and `warm_cap` (per run). Each run's report bracket then carries
its own `warm=used/budget` spend, and since every bench runs in a
process of its own, every run's budget includes the settle time.

The placement is what the pool's CPUs share, judged from the
first CPU's sysfs topology, in the form zc-ring-x1's measurement
tools print: `core` when the pool is one CPU, `SMT` when every CPU
is on its core, `CCX` when every CPU is on its L3, and `x-CCX`
when one is not. The runs header carries the same, `at 11,23
SMT`, so a summary places itself. A host whose sysfs has no cache
index 3 labels `SMT` and `x-CCX` and never `CCX`, and one whose
sibling list cannot be read prints the CPUs alone.

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
nines = −log₁₀(1−x)). `zK` is this project's mirror of it for the
fast tail (the underlying concept is the
[survival function](https://en.wikipedia.org/wiki/Survival_function)
/ CCDF tail fraction). The slow tail subdivides down to `n10`, the
fast tail only to `z4`, since a latency distribution is floored below
(nothing beats the fast path) and open above. A band only prints
when it has samples, so deep tail rows appear as run length earns
them (populating `n10` takes ~1e10 calls). Each row shows first,
last, range (`last - first + 1`), count, and mean.
The trimmed `mean`/`stdev` rows exclude every band at or above
`n2` (p99), and their label names the populated non-tail span (e.g.
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
  CI95 blocks      0.4   ns
  LSC blocks       0.5   ns
```

- **mean / stdev**: whole-histogram, tail included. One ms-scale
  outlier moves them, which is what the trimmed pair is for.
- **mean X..Y / stdev X..Y**: the same statistics over the
  populated non-tail span only (everything below `n2` = p99).
  The representative central tendency and spread. Prefer these
  for comparisons.
- **quantum**: the smallest per-call step this run could
  express: one timer tick divided by `inner`. It says whether
  the rows above describe the workload or the clock lattice. A
  `range 0.0 ns` band beside a coarse quantum is lattice, not
  uniformity.
- **resolution**: the smallest delta this run can honestly
  claim to distinguish, printed on **every** run. Fit from the
  block means: aggregate them in groups of 1, 2, 4, ... and
  watch whether variance keeps falling as `1/n`. Where it stops
  falling is drift the run cannot average away, and the worst
  level is the claim (Allan deviation's move). A change smaller
  than `resolution` is *not shown* by this run, however
  convincing the means look.
- **CI95 blocks / LSC blocks**: the 95% confidence half-width on
  `mean`, the block means' count-weighted average, and the least
  significant change against an equal-blocks run of something
  else, both over the run's blocks, so within one process. They
  print `-` when
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

## A bench's runs

Every bench runs `--runs` times, 5 by default, each run a child
process of its own and a bench's runs back to back. With one run
the report above is the whole output. With several, each child's
report is set aside (`-v` keeps it) and the bench prints a line per
run as it finishes, then its summary. From the 3900X, a busy desktop,
2026-09-15, `zcr-mpsc-v1-2t --runs 5 -d 1 --run-sleep 250ms-750ms`,
unpinned:

```
zcr-mpsc-v1-2t: 5 runs, each in a fresh process, unpinned

  run       pid            mean    stdev blocks      resolution            clock
    1       623      410.4 ns         79.7 ns         42.7 ns      4.22-4.52 GHz
    2       625      108.9 ns          6.2 ns          4.8 ns      3.61-4.54 GHz
    3       627      110.9 ns          2.1 ns          1.2 ns      3.37-3.64 GHz
    4       629      103.0 ns          6.8 ns          5.5 ns      3.61-4.54 GHz
    5       631      113.6 ns         32.0 ns          9.1 ns      3.29-4.09 GHz

  mean       169.4 ns
  stdev      134.8 ns
  CI95 runs  167.3 ns
  LSC runs   196.6 ns
  clock      3.29-4.54 GHz
```

- **the run lines**: each run's `mean`, the child's `pid`, and
  three columns for spotting an odd run:
  - `stdev blocks`: the standard deviation of the run's block
    means, how far its blocks wandered around its mean.
  - `resolution`: the run's drift floor, the row its own report
    carries. Well above its neighbours' means the run moved while
    it measured.
  - `clock`: the delivered clock its measuring core read at the
    block seams, one number when it held within 1%, as a pinned
    clock's should, and the range when it moved.

  Above, run 1 is a run that moved: its mean is four times the
  others', and its `stdev blocks` and `resolution` sit several times
  above theirs. A run on another level reads differently, an off
  mean with blocks as tight as its neighbours', as the 7600x's pinned
  run at 76.2 ns among runs near 70 did. A run's own report, `--runs 1` or `-v`,
  still carries its `CI95 blocks` and `LSC blocks`.
- **mean / stdev**: the plain mean of the run means, each process
  one draw, and their run-to-run standard deviation.
- **CI95 runs / LSC runs**: the confidence half-width on that mean
  and the least significant change against an equal-runs bench,
  over the run means. A process start re-rolls where the bench's
  memory lands, and the blocks of one run cannot see that, so these
  are the first error bars that cover placement. Above, one moved
  run makes them wide, which is the honest answer for that stretch
  of a busy desktop.
- **trimmed mean / winsorized stdev / CI95 trimmed / LSC trimmed**:
  the same four numbers over a series with its top and bottom 20%
  of runs dropped, printed from five runs up, with a `trimmed` line
  naming which runs went. They answer a different question:
  - **the plain pair** says what a run of this bench costs on this
    host, a disturbed run included, since a disturbance the host
    really produces is part of what a run draws.
  - **the trimmed pair** says whether a change moved the bench,
    where a run the host disturbed is noise about the code.
  - **the 3900X, clock free, on a busy desktop**: four of twenty runs
    were disturbed, the worst at 772 ns against a bulk near 385. The
    plain pair read 431.4 ns +- 56.5, unusable for a code question,
    and the trimmed pair 385.5 ns +- 4.0. A pinned invocation minutes
    earlier read 383.7 +- 2.3 plain and 384.1 trimmed, so the trimmed
    pair agreed across the two while the plain pair did not.
  - **the cost**: on a clean series the trimmed bar is the wider one,
    since it is built from fewer runs. Ten quiet `min-now` runs read
    `CI95 runs` 0.2 ns and `CI95 trimmed` 0.4 ns. Read the plain pair
    on a quiet host and the trimmed pair when the runs say the host
    was not.
  - **why the stdev is winsorized while the mean is trimmed**:
    dropping the extremes is what keeps them out of the value, and
    the error of that value comes from a series where they are pulled
    in to the boundary instead, since their absence is itself
    uncertainty. The pairing is Yuen's, and mixing it the other way
    would understate the error.
- **clock**: the lowest and highest clock any run's measuring core
  read, one number when every run held the same clock. Unpinned, a
  shift between invocations that the clock line also shows is the
  host, not the code.
- **what they do not cover**: a bench's runs go back to back, so
  they share whatever state the host holds for that stretch, its
  clock above all. Where the clock drifts, `CI95 runs` is still a
  lower bound on what another invocation reads. On the 3900X, two
  unpinned invocations of `min-now` a minute apart read 22.8 and
  22.5 ns, each with `LSC runs` of 0.1 ns, while two pinned with
  `--pin-freq` both read 26.3 ns. Pin the clock whenever the
  comparison crosses invocations ([Comparing two
  implementations](#comparing-two-implementations)).
- **the ratio**: the run `stdev` against a run's `stdev blocks`
  divided by the square root of its block count says whether
  per-process state dominates. `stdev blocks` itself is the spread of
  block means, each far shorter than a run, so it is naturally
  larger than the run `stdev`, and only the scaled comparison is
  fair. On the pinned 7600x, `zcr-mpsc-v1-2t` runs whose blocks put
  each run's mean within about 0.1 ns scattered by a run stdev of
  0.6 ns, per-process state dominating about six times. With
  `min-now` the ratio ran the other way: runs agreed to 1 ps where
  their blocks predicted about 3.5, so there the blocks' spread was
  short-term noise that averages away. Five runs judge a spread only
  to a factor of 2 or 3, so a ratio is a lead, not a verdict.
- **the cost**: every run is a fresh process, sleeps its run sleep
  (1-2 s by default), and pays the settle warm
  ([Settle time](#settle-time)), so a bench takes about
  `runs x (run_sleep + settle_time + duration)` plus the block
  sleeps. `-D` divides its total over every run of every bench.
- **precision**: a mean and its stdev print at least as precisely
  as the claims beside them, whatever `--decimals` says, so a
  0.01 ns `LSC runs` is never compared against means rounded to
  0.1 ns.

A run's record names the invocation in `series` and its place
among the bench's runs in `run`, so a records directory groups
back into the runs that made each summary.

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
vary run-to-run and machine-to-machine, and the *shape* of the
differences is the useful signal.

Each row is one *populated* band (see the boundary ladder above), and
empty bands are skipped. Columns:

- **first / last**: the smallest and largest sample *values* in the
  band, and `first` of the top row is the fastest call observed.
- **range**: `last − first + 1`, the band's width.
- **count**: samples in the band.
- **mean**: the band's mean, raw. Nothing is subtracted (see
  [The Setup banner](#the-setup-banner)).

Below the bands, `mean` / `stdev` are whole-histogram, and the trimmed
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
convention. Computing's other default is left-closed `[lower, upper)`
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
exactly where values land (the exact `-d` is machine-dependent, so tune
it to the count you want, and there are no timing guarantees):

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

"Is B really faster than A, or is it noise?" The workflow, one
bench per invocation, run the same way for each implementation:

```
iiac-perf zcr-mpsc-v1-2t --pin-cpus 0,1 -d 2
```

The bench runs 5 times by default, each run a fresh process
([A bench's runs](#a-benchs-runs)), and every run's `-d 2` is
divided into 100 blocks separated by 1-10 ms sleeps. Always pin
(`--pin-cpus`): unpinned, the OS's thread placement is re-rolled
per *process* and adds its own spread to the runs. On the 3900X,
2026-09-15, it printed:

```
zcr-mpsc-v1-2t: 5 runs, each in a fresh process, unpinned

  run       pid            mean     CI95 blocks      LSC blocks
    1         7        111.2 ns          0.5 ns          0.7 ns
    2         9        137.4 ns          4.8 ns          6.8 ns
    3        11        135.9 ns          5.6 ns          7.9 ns
    4        13        137.9 ns          5.0 ns          7.0 ns
    5        15        137.0 ns          4.7 ns          6.7 ns

  mean       131.9 ns
  stdev       11.6 ns
  CI95 runs   14.4 ns
  LSC runs    16.9 ns
```

- **mean**: the bench's headline number, the plain mean of its
  run means.
- **CI95 runs**: 95% confidence interval (half-width) on that
  mean: "the true value is within +-14 ns of 132, as far as five
  fresh processes can tell."
- **LSC runs**: least significant change: run the *other*
  implementation the same way (same `-d`, same `--runs`, same
  knobs, same pin), and if the two `mean` values differ by more
  than roughly the larger of the two `LSC runs`, the difference is
  real at 95% confidence.
- **on a host that disturbs some runs**, compare `trimmed mean`
  against the larger `LSC trimmed` instead, and say so when
  reporting the result. The plain pair on such a host answers what a
  run costs there, not whether the code changed ([A bench's
  runs](#a-benchs-runs)).

Run 1 is the case this surface exists for: its blocks agreed to
0.5 ns at 111 ns, a claim the other four processes, all near
137 ns, contradict by 26 ns. A single process would have reported
either level with a tight error bar. Five runs make the bar wide
enough to cover both, and more runs narrow it as the square root
of their count while showing how often each level comes up.

A run's own report, `--runs 1` or `-v`, still carries
`resolution`, `CI95 blocks`, and `LSC blocks`: honest
*within-process* claims, and lower bounds on what a fresh process
shows, since per-process state survives every block sleep
(~0.6% residual even pinned on an idle Ryzen 5 7600X, and on the
7600x's `zcr-mpsc-v1-2t` runs in one process landing on levels from
60.4 to 71.9 ns while each claimed a `CI95` under 0.1 ns). Use them
to see whether a run held still, and the run rows to compare.

Caveat: a bench's error bars cover the stretch of time its runs
took. Two implementations measured in different stretches, in two
invocations, a rebuild apart, or even back to back in one bench
list, also differ by whatever the host drifted between those
stretches, and neither bench's `CI95 runs` contains that. The
clock is the drift measured so far: the 3900X's unpinned
invocations of one bench a minute apart differed by three times
their `LSC runs`, and pinned ones agreed ([A bench's
runs](#a-benchs-runs)). So compare with the clock pinned
(`--pin-freq`, or `pin_freq` in the directory's config), and on a
host that still drifts, repeat the comparison later and see
whether it holds, or alternate the two invocations by hand
(A, B, A, B) so a drift lands on both.
Method and worked numbers:
[Comparing implementations](../notes/design.md#comparing-implementations-least-significant-change),
[block validation](../notes/design.md#block-validation-results-0210-4-r5-7600x).

## The two grades

Every report ends with the grade block: one column header over
three rows, each graded A-F from its own data: two `env` rows
for the **box**, one `run` row for *that run*. A row's `worst`
column is its composite, printed beside the signals that earned
it, and a blank cell (`-`) means that signal does not apply to that
row, which is the env/run signal mapping made visible:

```
  grade  phase                        settle  worst     spread  bursts  interference      drift               step
  env    warmup   4.84->5.24GHz 49% +-0.0% A      A    0.47% A       -       0.00% A    0.00% A            0.00% A
  env    bench                             -      F    0.48% A       -       0.00% A   11.05% F    11.05% @1.06s F
  run    all                               -      F          -   37% B       0.04% A   10.49% F    10.49% @1.04s F
```

Column reference (each signal prints its own letter beside its
value, and the sections below carry the depth):

- `grade` / `phase`: row labels. The two `env` rows grade the
  box from micro-probes that never touch the bench (`warmup`:
  did it end settled, and `bench`: did it stay settled). The `run`
  row grades the numbers above it, from the run's own blocks.
- `settle`: warmup row only, and a graded signal like the
  rest: the clock's journey, the settled share of the warm,
  how still it held, and the share's letter. `00%` is
  never-settled, an F. See [Settle time](#settle-time).
- `worst`: the row's composite letter, its worst signal
  outright, and always one of the letters printed beside it. On
  the warmup row the settle letter counts as a signal, which
  is the one place the clock decides a grade.
- `spread`: env rows only. How wide a probe's bulk sits above
  its own floor. A timer pair has no workload character, so
  width means the box itself moved.
- `bursts`: run row only. The fraction of blocks whose mean
  sits above the run's median block: whether interference was
  localized in time or spread out.
- `interference`: samples that sat above their block's floor,
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
  the bench's first blocks where no timing detector sees it,
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

The warm is per **process**: the boost it wins is machine state
the process's later runs inherit. Without it the first bench of a
process reports a cold machine's numbers (measured at ~8.6% slow
on a 7600x, a wrong histogram rather than merely a wrong letter).
Every bench runs in a child process of its own, so every bench
pays the warm, 1.5 s on top of a `-d 5` bench's budget.

`--settle-time 0` skips the warm, which is how you measure what
it is worth on a given box. A box that reads `00%` at the
default wants more, though that is not always curable: on a
3900X the floor is bistable and moves at arbitrary times, so a
3.5 s warm still left runs moving mid-bench. Replication
(`--blocks`) is the answer there, not a longer warm.

The probes run through warmup and then in the seam at every block
boundary, so the series covers the whole run on the same time
axis as the blocks. `--no-env-probe` limits them to warmup (so
only the warmup row appears),
which costs the grade its span. It exists because seam probing
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

- `interference`: samples that sat above their block's floor, as
  a fraction of the run. How much other work leaked in.
- `bursts`: blocks whose mean sits above the run's median block.
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
anywhere makes the run F and no number of A's pulls it back.
`step` alone earns the F in the example above. That is why every
signal prints its own letter: a row's `worst` is always one of
the letters shown beside it.

The `env` rows work the same way, over their own four signals.
There were once six, when the grade was measured at startup from
a calibration fit: two of them scored how well that *fit* held
(the worst residual of a ladder point against the Theil-Sen line,
and the loop-only slope against a dithered two-point fit). A
bench run fits nothing, so those two have no run-side analog and
none was invented. The reasoning is recorded in
[chores-05.md](../notes/chores/chores-05.md#six-calibration-signals-four-run-signals).

Both floor signals compare medians, not extremes, so one hot
block is a burst rather than a shift.

**It reports. It does not warn.** A low letter is not a fault to
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
bench is meaningful, and comparing it across different benches is
not.

This is a different question from the `env` rows, which grade the
box rather than the run. Judging the box from a bench's own
samples is not possible after the fact, since they mix the two
inseparably. The `env` grade instead comes from micro-probes that
time timer pairs and never touch bench code, so no workload
character enters it.

## What to conclude: a worked example

A real session (3900X, 2026-08-19) that exercises most of the
report's surfaces. First, find the frequency the box holds under
the actual workload:

```
$ sudo iiac-perf suggest-freq zcr-mpsc-v0-2t --pin-cpus 0,1
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
  does not touch. We think that is the mechanism, and the pinned
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
same idea, and ours is `pin-freq` / `restore-freq`.

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

### A run's mean follows its clock, and the sleep before it does not matter

The clock experiment (2026-09-17, `min-now`, both hosts) asked why two unpinned 3900X invocations
read 22.8 and 22.5 ns while two pinned with `--pin-freq --run-sleep 1s` read 26.3 ns, the pin and
the sleep having changed together. It is one definition, [configs/clock-shift.md][clock-cfg], run
as four conditions by flag, thirty runs each per host, the conditions interleaved three times over.
The 240 records are in `records/clock-shift.jsonl`, a line a run, and
`python3 configs/clock-shift.py records/clock-shift.jsonl` prints every number here.

| host | condition | runs | mean ns | stdev | clock GHz | cycles a call |
|---|---|---|---|---|---|---|
| 3900X | pinned, sleep | 30 | 26.32 | 0.20 | 3.768 | 99.2 |
| 3900X | pinned, no sleep | 30 | 26.35 | 0.31 | 3.768 | 99.3 |
| 3900X | unpinned, sleep | 30 | 23.49 | 0.66 | 4.241 | 99.6 |
| 3900X | unpinned, no sleep | 30 | 23.31 | 0.86 | 4.300 | 100.1 |
| 7600X | pinned, sleep | 30 | 18.94 | 0.00 | 4.666 | 88.4 |
| 7600X | pinned, no sleep | 30 | 18.94 | 0.00 | 4.666 | 88.4 |
| 7600X | unpinned, sleep | 30 | 16.34 | 0.02 | 5.439 | 88.9 |
| 7600X | unpinned, no sleep | 30 | 16.34 | 0.02 | 5.439 | 88.9 |

The clock is the mean of the record's `clock_khz`, the delivered clock read at the block seams, and
cycles a call is `mean_ns` times that clock.

- **The mean follows the clock.** On the 3900X the unpinned runs' clocks ranged from 3.97 to
  4.53 GHz, and each run's mean tracks the inverse of its own clock with a correlation of +0.94
  across the sixty runs. Multiplying the clock back in takes the spread of those runs from 3.3% of
  the mean to 1.1% of the cycles. The cycles a call costs is the same in all four conditions,
  99 to 100, so the 26.3 ns against 23.4 ns is one workload at two clocks: the means differ by
  a ratio of 1.125 and the clocks by 1.133.
- **The 7600X says the same by a different route.** Its unpinned clock did not move, 0.03% across
  sixty runs, so there is no spread for a correlation to read. Pinned against unpinned, its means
  differ by 1.159 and its clocks by 1.166, and the cycles a call costs is 88 to 89 in both.
- **The sleep before a run moves nothing.** Sleep against no sleep is +0.19 ns unpinned and
  -0.03 ns pinned on the 3900X, inside 95% half-widths of 0.40 and 0.14 ns, and 0.00 ns both ways
  on the 7600X, inside 0.01 ns. So of the two things that changed together, the pin was the cause.
- **What is left over.** The mean ratio falls short of the clock ratio by about 0.7% on both
  hosts. We think part of a call does not scale with the core clock, or the seam readings sit
  slightly off the clock the samples ran at. The experiment cannot tell these apart.

What to take from it:

- An unpinned number carries its clock. Compare two unpinned runs by their `clock` cells before
  their means, and on a host whose clock wanders, as the 3900X's does by 13% from run to run here,
  compare cycles a call or pin the clock.
- A pinned number is slower because boost is off, not because the pin costs anything. It is the
  steadier ruler: the 3900X's run-to-run spread falls from 0.7 to 0.2 ns.
- The 7600X held one clock unpinned through every run, so on that host this workload's unpinned
  number is as steady as a pinned one. That is this workload at this duty cycle, not a property
  to assume of another.
- `run_sleep` can be 0 for `min-now` without changing the reading, which halves the wait between
  runs. A multi-threaded bench has not been tested.

[clock-cfg]: ../configs/clock-shift.md

## Label styles (`--band-labels`)

`--band-labels` selects the row-label vocabulary, and the trimmed
`mean`/`stdev` rows and the report header's `labels=` metadata
follow the same style. The trimmed label names the **populated**
non-tail span, and here `min` is never a row (no samples land in the
fast tail), so it reads `p50..n2`, not a fixed `min..n2`. Default
`both` prints the zpn name and its literal fraction side by side
(the juxtaposition teaches the zpn vocabulary):

```
$ iiac-perf min-now -d 1 --band-labels both
minstant::Instant::now() [duration=1.0s samples=1,539,764 inner=23 calls=35,414,572 batches=24 labels=both]:
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

`zpn` drops the fraction (names only), and `frac` drops the name
(fractions only, so the trimmed label reads `0.50..0.99`). Same
bench, separate runs, and only the leftmost column and the trim
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
| zcr-mpsc-v0-1t |     2.5 ns | MPSC  |       | zc-ring-x1 mpsc v0, 1 thread  |
| zcr-mpsc-v0-2t |    69.1 ns | MPSC  | spin  | zc-ring-x1 mpsc v0, 2 threads |
| zcr-mpsc-v1-1t |     5.0 ns | MPSC  |       | mpsc v1, 3900X run, see below |
| zcr-mpsc-v1-2t |    86.0 ns | MPSC  | spin  | mpsc v1, 3900X run, see below |
| zcr-spsc-v1-1t |     4.0 ns | SPSC  |       | spsc v1, 3900X run, see below |
| zcr-spsc-v1-2t |   109.6 ns | SPSC  | spin  | spsc v1, 3900X run, see below |
| zcr-spsc-v2-1t |     4.6 ns | SPSC  |       | spsc v2, 3900X run, see below |
| zcr-spsc-v2-2t |   110.8 ns | SPSC  | spin  | spsc v2, 3900X run, see below |
| zcr-spsc-v3-1t |    14.9 ns | SPSC  |       | spsc v3, 3900X run, see below |
| zcr-spsc-v3-2t |   105.5 ns | SPSC  | spin  | spsc v3, 3900X run, see below |
| zcr-mpsc-v2-1t |     8.6 ns | MPSC  |       | mpsc v2, 3900X run, see below |
| zcr-mpsc-v2-2t |   106.5 ns | MPSC  | spin  | mpsc v2, 3900X run, see below |

**The class column is the first thing to read across rows.** The
queues promise different things: crossbeam's channel and
`SegQueue` are MPMC, any number of producers and consumers. std's
channel and zc-ring-x1's mpsc rings are MPSC. The zc-ring-x1
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
at 120 ns and `zcr-spsc-v0-2t` at 124 ns, and zc-ring-x1's mpsc v0
ring at 69 ns is the fastest handoff in the table. We think the mpsc
ring's one shared hot word per slot beats the index cache lines
the others bounce, an exploration tracked in zc-ring-x1's todo.

**The spsc v1 and v2 rows and the mpsc v1 rows are guests from a
3900X run.** `zcr-spsc-v1-1t` and `zcr-spsc-v1-2t` measure
zc-ring-x1's seam-word SPSC v1, `zcr-spsc-v2-1t` and
`zcr-spsc-v2-2t` its in-slot seq SPSC v2, the bounded ring itself
rather than the segmented queue that will draw on it, and
`zcr-mpsc-v1-1t` and `zcr-mpsc-v1-2t` its equality-seq MPSC v1,
the v0 ring with seq checks that let its capacity run down to 1.
Each pair's peers are the rows under the same prefix one and two
versions back, and the class sentence applies. They were measured
on a 3900X at 0.28.9-2, and a table that mixes boxes reads as one
run, so the comparison to make is within their own runs: one
unpinned `iiac-perf-dev zcr`, five seconds per bench, and one
`--pin-cpus 0,1`, two cores of one CCX
([placement-map.md](../notes/placement-map.md)). The earlier
pairs, at 0.28.3-3 before v2 existed and at 0.28.7-1 before the
mpsc pair carried a version, are in this file's history.

| bench          | unpinned | pinned 0,1 |
|----------------|---------:|-----------:|
| zcr-spsc-v0-1t |   2.8 ns |     2.6 ns |
| zcr-mpsc-v0-1t |   4.9 ns |     4.8 ns |
| zcr-mpsc-v1-1t |   5.0 ns |     4.9 ns |
| zcr-spsc-v1-1t |   4.3 ns |     4.4 ns |
| zcr-spsc-v2-1t |   4.7 ns |     4.3 ns |
| zcr-spsc-v0-2t | 135.4 ns |   134.4 ns |
| zcr-mpsc-v0-2t |  95.7 ns |   106.8 ns |
| zcr-mpsc-v1-2t |  86.0 ns |   110.9 ns |
| zcr-spsc-v1-2t |  92.6 ns |    93.8 ns |
| zcr-spsc-v2-2t | 110.8 ns |   113.5 ns |

Same thread, the two mpsc rings are one number, 4.8 and 4.9 ns
pinned, which is what v1's design predicted and what zc-ring-x1's
own matrix found from depth 2 up: the equality checks and the
precomputed commit value cost nothing the signed diffs did not.
The spsc v1 and v2 rings sit between v0 and the mpsc pair, mpsc's
per-slot seq publish without its claim CAS, and pinned, v2 is the
faster of the two, its seq and its message on one line. Across
threads the pinned column is the one to read, since unpinned the
`zcr-mpsc-v0-2t` warmup never settled and the spsc v1 and v2
single-thread runs graded D on drift, and there the two mpsc
rings are 4 ns apart, 107 and 111, inside a run pair's noise at a
40 ns spread, so at this depth v1 costs what v0 costs. spsc v1 at
94 ns beats v0 by three tenths and the mpsc rings by an eighth,
while v2 at 114 ns sits with the mpsc rings, which v2's design
claim did not predict: one line crossing cores per handoff rather
than two. We think the round trip hides the claim: the consumer
spins on the seq word, and moving that word into the slot puts
the spin on the line the producer is filling, so the poll pulls
the line away mid-write where v1's consumer spins on a line the
producer touches once. This is the second run pair to show it, so
the gap is a lead for zc-ring-x1's own measurements to chase, not
yet a ranking. The grades say the rest: pinned, every two-thread
run's bench phase graded A, the first pair here where the pinned
column carries no F, and the one blemish is `zcr-spsc-v0-1t` at C
on step.

**The spsc v3 rows are guests from a later 3900X run**, three
runs of a second each at `--pin-cpus 0,1` at 0.28.15-1, the pair
`zcr-spsc-v2-1t` and `zcr-spsc-v2-2t` run beside them: 4.7 and
110.1 ns for v2 against 14.9 and 105.5 for v3, with `LSC runs`
of 0.4, 5.0, 0.4, and 0.6 ns. `zcr-spsc-v3-1t` and
`zcr-spsc-v3-2t` measure zc-ring-x1's segmented SPSC v3, a ring of
two segments of eight slots over a pool, in the same round-trip
shapes, and both print their segment switch counts after the
report, zero in every run here, so the ring never left its first
segment. Across threads v3 reads five nanoseconds under v2, the
difference outside both LSCs. Same thread it reads three times
v2, ten nanoseconds a round trip, and a rerun with one segment
read the same, so the cost is v3's per-message path with no switch
in it, not the second segment. We think it is what the round trip
across cores hides under the handoff, and it is a lead for
zc-ring-x1, since the design note claims v2's cost where the
consumer keeps up.

**The mpsc v2 rows are guests from the same kind of run**, three
runs of a second at `--pin-cpus 0,1` at 0.28.15-4, the pair
`zcr-mpsc-v1-1t` and `zcr-mpsc-v1-2t` beside them: 4.8 and 102.4
ns for v1 against 8.6 and 106.5 for v2, with `LSC runs` of 0.1,
4.8, 0.02, and 2.3 ns. `zcr-mpsc-v2-1t` and `zcr-mpsc-v2-2t`
measure zc-ring-x1's segmented MPSC v2, two segments of eight
slots over a pool, one producer per ring, and print their switch
counts after the report, zero here. Same thread v2 costs 1.8
times v1, four nanoseconds a round trip, outside both LSCs, and
across threads it reads four nanoseconds over v1, inside v1's LSC.
So both segmented rings cost more same thread where their design
notes claim their predecessor's cost, spsc v3 by ten nanoseconds
and mpsc v2 by four, and both hide it under the cross-core
handoff. The 7600X read the same shape at its SMT pair `5,11`
with the clock pinned at 4701 MHz (wink, 2026-09-16, five runs of
250 ms): spsc v3 at 12.2 ns same thread against v2's 2.15, nearly
six times, and mpsc v2 at 5.95, while across the siblings v3 read
56 ns against v2's 40 with its runs spread from 51 to 67, the one
placement so far where v3 loses across threads, and a pair that
wants more runs before it is a number. The closing's 3900X run
at `0,1`, five runs of two seconds, put spsc v3 at 109.1 ns
against v2's 108.9, inside a 10.9 ns `LSC runs` that one run at
122 set, and mpsc v2 at 106.4 against v1's 97.5, outside both
LSCs.

## Verbose output (`-v`)

`-v` prints the affinity lifecycle on stderr. Main pins only
when `--pin-cpus` is given (to the pool's first slot, where it warms
and measures), and otherwise every mask stays as the process
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

std::sync::mpsc round-trip (2 threads) [duration=3.0s samples=363,598 inner=1 calls=363,598 batches=55 labels=both]:
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

std::sync::mpsc round-trip (2 threads) [duration=3.0s samples=363,056 inner=1 calls=363,056 batches=55 labels=both]:
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
  bench pin         [0, 1] (2 slots, 2 unique CPUs, CCX)

std::sync::mpsc round-trip (2 threads) [duration=3.0s samples=417,477 inner=1 calls=417,477 batches=55 labels=both]:
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
