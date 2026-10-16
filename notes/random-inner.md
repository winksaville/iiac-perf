# Random inner removes the period of 4

Two invocations of one bench could disagree by more than their claims allowed, because the
harness sizes `inner` itself and a two-thread bench's time per step depends on the value it
picks: on the 7600X `ccx` pair `zcr-mpsc-v4-2t-single-sos-futex-spntfe` reads 71.1 ns a step at
an `inner` of 6 and 75.7 ns at 7. This file holds what was taken to understand that, from `feat:
fit, the cost of a step from samples of many lengths` (2026-10-08).

- A sweep, each run at one fixed `inner` and the runs stepping it across a range, shows the
  dependence has a period of 4.
- In runs where each sample picks its `inner` at random from a span the period disappears, so
  the result no longer hangs on one value. Random lengths change more than the repetition,
  though, and which change ends the period is not separated.
- They do not make every run agree: over 1-32 a run lands on one of two rates about 6 ns a step
  apart.
- `linear-regression`, `lr` for short and first named `fit`, reads both, and with the harness
  overhead of about 18 ns taken off gives a step's cost where a line holds.

Whether random `inner` makes separate invocations agree is not tested here.

## Terms

What is measured:

- **Step**: the operation a bench times. For a ring one message sent and received, for `min-now`
  one clock read.
- **Sample**: one timed stretch, a clock read before and after `inner` steps back to back. Its
  time, the sample time, is in ns.
- **`inner`**: the steps in one sample, also called its length. `--inner 7` fixes it for a run,
  `--inner 1-16` picks each sample's at random from the span, uniformly, and with neither the
  harness sizes it. "Fixed" and "random" in the tables say which.
- **Run**: one process measuring one bench over many samples, its result their mean.
- **Sweep**: a set of runs that steps a fixed `inner` across a range, each run at one value. The
  sweep here is 336 runs, two passes of three runs a bench at each value.
- **Pass**: one time through the list of `inner` values, `a` or `b`, the second the first
  reversed.
- **Time per step**: a sample's time divided by `inner`, in ns, what iiac-perf reports for a
  bench. It is a step's cost with the harness overhead shared out over the steps, `ns/op` in Go's
  and JMH's benchmarks. The reports and the other notes call it the level until a rename reaches
  them.
- **Harness overhead**: what a sample costs beyond its steps, the clock reads around them, about
  18 ns on the 7600X. It is the same whatever the bench, so it is measured once, by `min-now`.

What `linear-regression` reports, a line fitted through sample time against `inner`:

- **Slope**: what one more step adds to a sample, ns a step, the cost of a step where a line
  holds.
- **Intercept**: what a sample of no steps would take, ns. For `min-now` it is the harness
  overhead, and for a ring the overhead plus what the ring pays once a sample.
- **Residual**: how far a length's sample time sits from the line, ns.
- **Halves apart**: the line refitted over the lower and the upper half of the lengths, and the
  two slopes' difference as a percent of the whole line's slope.
- **A line holds**: the halves are within 1% of each other, so every step costs about the same
  whatever the length and the slope and intercept mean what they say. Otherwise no cost is
  named.
- **Period**: residuals that repeat with `inner` modulo some k, found in one pass and scored by
  how much of the other pass's scatter they account for. "None" is looked for and not found,
  and blank is not looked for, as where the lengths are not consecutive.

The benches, whose names are decoded in full in [The
names](mpsc-v4-mode-waits.md#the-names):

- `min-now`: a single clock read and no ring, one thread.
- The two-thread benches are `zcr-mpsc-v4-2t-single-` and a tail, written here by the tail.
  `2t` is a sender and a receiver on two threads, and `single` a `Single` ring.
  - `so-spntfe`: a ring that can only spin, `SpinOnly`, its receiver spinning forever at an
    empty ring. "The spin-only one" in the text.
  - `sos-futex-spntfe`: a ring that can spin or sleep on a futex, `SpinOrSleep<Futex>`, its
    receiver spinning forever and so never sleeping. "The ring that can sleep" in the text.
  - `slp-futex-slptfe`: a ring that sleeps on a futex, `Sleep<Futex>`, its receiver sleeping
    until woken at every empty look.
  - `so` and `sos-futex` are those first two where a column has no room.
- Placements: `ccx` and `smt`, the CPUs given in [The sweep](#the-sweep).

## Why it was taken

The disagreement was two overnight sessions on one binary, five invocations of ten runs and one
of fifty, against each other and against the session in
[mpsc-v4-mode-waits.md](mpsc-v4-mode-waits.md), on two-thread benches at cross-core placements.
Three probes on the 7600X `ccx` pair, on `sos-futex-spntfe`, found what moved the time per step.
Their records are not committed:

- The environment's size does not. Padded by 0 to 4096 bytes in eleven steps, ten runs each, 107
  of 110 runs read 75.5 to 75.9 ns.
- Address randomization does not. With it off, `setarch -R`, all 40 runs read 75.5 to 75.8 ns.
- `inner` does. In the fifty-run invocation every run at 71.1 ns had sized `inner` to 6 and every
  run at 75.7 ns to 7. Fixed by hand, three alternated passes of ten runs, `--inner` 5, 6, 7 and
  8 read 73.1, 70.6, 75.5 and 67.1 ns, 27 or more of each 30 runs within 0.6 ns.

## The sweep

- Source: commit `a946d7c1`, jj change `rrwqouqq`, a clean tree, `feat: mpsc bench names follow
  the ring's words`, the plain `iiac-perf` 0.28.23.
- Binary: `7715ab993ac3b78c`, built on the 3900X.
- Host: the 7600X, the clock pinned at 4701 MHz. `ccx` is CPUs 5 and 4, and `smt` for a
  one-thread bench is CPU 5.
- Two threads, on `ccx`, `--inner` 1 to 16: `zcr-mpsc-v4-2t-single-so-spntfe`,
  `zcr-mpsc-v4-2t-single-sos-futex-spntfe`, and `zcr-mpsc-v4-2t-single-slp-futex-slptfe`.
- One thread, on `smt`, `--inner` 20, 30, 40, 50, 60, 80, 100 and 120: `min-now`, a single clock
  read with no ring.
- Two passes, three runs of each bench at each `inner` in a pass, so six runs a point. A record's
  `tags.pass` is `a` or `b`.
- The order of `inner` in pass a is scrambled, and pass b runs it in reverse, so a drift over the
  11.5 minutes cannot pass for an effect of `inner`.
- Run: 2026-10-08T16:35Z to 16:47Z, `--run-sleep 100-200ms`.
- Records, every switch count in them zero:
  [records/inner-fixed-7600x.jsonl](../records/inner-fixed-7600x.jsonl), 336 runs. It was
  written as `inner-sweep-7600x.jsonl`, the name the loop below gives and each record carries.

The loop, pass b taking each list reversed:

```
T2="zcr-mpsc-v4-2t-single-so-spntfe zcr-mpsc-v4-2t-single-sos-futex-spntfe \
    zcr-mpsc-v4-2t-single-slp-futex-slptfe"
for i in 13 4 9 1 7 11 2 16 5 14 3 8 12 6 10 15; do
  iiac-perf run $T2 --pin-cpus ccx --runs 3 --run-sleep 100-200ms --inner $i \
    --tag pass=a --record-file inner-sweep-7600x.jsonl
done
for i in 60 20 120 40 100 30 80 50; do
  iiac-perf run min-now --pin-cpus smt --runs 3 --run-sleep 100-200ms --inner $i \
    --tag pass=a --record-file inner-sweep-7600x.jsonl
done
```

## One thread

`min-now`, the median of six runs. A sample's time is the time per step times `inner`. The
columns are the fixed `inner`, the time per step in ns, and the sample time in ns:

| `inner` | ns/step | sample ns |
|---|---|---|
| 20 | 19.190 | 383.8 |
| 30 | 18.885 | 566.6 |
| 40 | 18.729 | 749.2 |
| 50 | 18.635 | 931.7 |
| 60 | 18.572 | 1114.3 |
| 80 | 18.499 | 1479.9 |
| 100 | 18.449 | 1844.9 |
| 120 | 18.418 | 2210.2 |

The time per step falls as `inner` grows and never settles. A line through sample time against
`inner`, fitted by hand, has a slope of 18.264 ns a step in each pass and an intercept of 18.61
ns in pass a and 18.57 ns in pass b. So a sample costs about one clock read beyond its steps, and
the time per step carries that cost divided by `inner`.

## Two threads

`zcr-mpsc-v4-2t-single-*` on `ccx`, the median of six runs. The columns are the fixed `inner`
and each bench's time per step in ns, the bench named by its tail:

| `inner` | `so-spntfe` | `sos-futex-spntfe` | `slp-futex-slptfe` |
|---|---|---|---|
| 1 | 90.5 | 92.6 | 4868 |
| 2 | 75.3 | 77.9 | 4722 |
| 3 | 69.1 | 74.7 | 4667 |
| 4 | 68.8 | 70.1 | 4822 |
| 5 | 68.3 | 73.0 | 4449 |
| 6 | 70.1 | 70.6 | 4522 |
| 7 | 71.0 | 75.7 | 4378 |
| 8 | 70.6 | 66.9 | 4493 |
| 9 | 73.3 | 75.5 | 4434 |
| 10 | 73.1 | 72.0 | 4551 |
| 11 | 72.8 | 74.4 | 4510 |
| 12 | 72.1 | 69.2 | 4452 |
| 13 | 71.5 | 72.9 | 4530 |
| 14 | 70.8 | 71.4 | 4492 |
| 15 | 70.3 | 71.6 | 4521 |
| 16 | 69.4 | 67.9 | 4557 |

- At `inner` 1 the two spinning benches read 21 and 25 ns above their times per step at 16.
- `sos-futex-spntfe` is low at 4, 8, 12 and 16 and high at the odd values between, a period of 4.
  `so-spntfe` has none, only a slow wave, low at 5 and high at 9.
- Which of the two is faster depends on `inner`: at 8 the ring that can sleep reads 3.7 ns under
  the spin-only one, and at 7 it reads 4.7 ns over.
- The two passes agree. The same hand fit, a simple linear regression of sample time on `inner`,
  gives `so-spntfe` 70.48 and 70.46 ns a step and `sos-futex-spntfe` 69.61 and 69.54 ns a step.
- A line does not describe them, though, so those slopes are not the cost of a step. The slope
  moves with the range fitted where `min-now`'s stays at 18.26 ns over any part of its range.
  The columns are the `inner` values the line is fitted over, then for each of the two benches
  the slope in ns a step and the intercept in ns:

  | `inner` fitted | `so` slope ns | intercept ns | `sos-futex` slope ns | intercept ns |
  |---|---|---|---|---|
  | 1 to 16 | 70.5 | 6.5 | 69.6 | 18.6 |
  | 1 to 8 | 68.5 | 10.0 | 67.3 | 23.0 |
  | 5 to 12 | 75.7 | -33.7 | 70.8 | 10.9 |
  | 9 to 16 | 64.4 | 87.4 | 62.8 | 109.7 |

  What changing `inner` does to a two-thread bench is a systematic effect and not noise about a
  line, so the regression can give neither the sample's overhead nor the step's cost for them.
- `slp-futex-slptfe` sleeps at every look, and its passes differ by up to 10% at one `inner`, so
  this sweep says nothing of its shape.

## Random per sample

A run at a fixed `inner` repeats one burst length, so the sweep cannot say whether a two-thread
bench's cost follows the length or the repetition. `--inner 1-16` picks each sample's length at
random from the span, and the record holds the samples and their summed time at each length.

The timed code is the same both ways, one call timing a sample of a given length, but random
lengths change three things at once and these runs cannot tell them apart:

- the repetition: no sample follows one of its own length more often than chance
- the loop's exit: with a fixed count the CPU learns where the inner loop ends, and with a
  random one it guesses wrong about once a sample
- the process: a random run measures every length in one process, so its lengths share that
  process's layout and rate, where each fixed point is its own processes

- Source: commit `124244db`, jj change `vrxrlwwy`, a clean tree, `feat: inner drawn per sample
  from a range`, `iiac-perf-dev` 0.28.24-2, binary `f53a7f480a66675f`, built on the 3900X.
- The same host, placements, and benches as the sweep. Two passes of five runs, tagged `pass`,
  2026-10-08T17:30Z to 17:34Z, `--run-sleep 100-200ms`:
  - the three two-thread benches on `ccx` at `--inner 1-16` and at `--inner 1-32`
  - `min-now` on `smt` at `--inner 20-120`
  - the two spinning benches at `--inner 7` and at `--inner 8`, fixed, as a control on the
    binary
- Records, every switch count zero:
  [records/inner-random-7600x.jsonl](../records/inner-random-7600x.jsonl), 110 runs. It was
  written as `inner-drawn-7600x.jsonl`, the name each record carries.

The control reads as the landed binary did: `sos-futex-spntfe` 76.0 ns at 7 and 67.0 ns at 8,
`so-spntfe` 71.1 and 70.6 ns. So the new binary's fixed loop measures what the old one did.

## What linear-regression says

`iiac-perf-dev linear-regression records/inner-fixed-7600x.jsonl` and the same on
`records/inner-random-7600x.jsonl`. The command is a simple linear regression of sample time on
`inner`, by ordinary least squares. It refits each half of the lengths and names a cost only
where the two slopes agree within 1%, "a line holds". The columns are the bench, its `inner`
fixed at each value or random over the span, the slope in ns a step, the intercept in ns, the
halves apart in percent, the period found, and whether a line holds, each as
[Terms](#terms) says:

| bench | `inner` | slope | intercept | halves apart % | period | a line |
|---|---|---|---|---|---|---|
| `min-now` | fixed, 20 to 120 | 18.264 | 18.53 | 0.01 | | holds |
| `min-now` | random, 20-120 | 18.256 | 18.16 | 0.02 | none | holds |
| `so-spntfe` | fixed, 1 to 16 | 70.435 | 5.78 | 5.53 | none | does not hold |
| `so-spntfe` | random, 1-16 | 74.438 | 7.08 | 5.00 | none | does not hold |
| `so-spntfe` | random, 1-32 | 68.165 | 25.58 | 0.95 | none | holds |
| `sos-futex-spntfe` | fixed, 1 to 16 | 69.557 | 18.18 | 6.25 | 4 | does not hold |
| `sos-futex-spntfe` | random, 1-16 | 76.155 | 11.96 | 2.69 | none | does not hold |
| `sos-futex-spntfe` | random, 1-32 | 69.968 | 28.87 | 1.36 | none | does not hold |

`slp-futex-slptfe` is left out: in all three its runs scatter too much for the halves to say.

- One thread: both ways give `min-now` 18.26 ns a step and about 18.4 ns a sample, the two
  slopes 0.008 ns apart. The regression holds and the two ways agree.
- The period of 4 disappears when the lengths are random. Fixed, `sos-futex-spntfe`'s
  residuals by `inner` modulo 4 are -31.7, +14.9, +0.3 and +16.4 ns a sample, a period that
  accounts for 71% of the other pass's residual scatter. Random over 1-16 they are -0.9, +1.4,
  -0.1 and -0.4 ns, and `linear-regression` finds no period in any random group. Which of the
  three changes above ends it is not known. The fixed period is not noise between processes,
  since thirty processes at one `inner` agree within 0.6 ns.
- What is left is smooth. A sample of one step sits 10 ns over the line for `sos-futex-spntfe`
  and 15 ns for `so-spntfe`, the next few under it, and from about 8 steps the time per step is flat
  to 0.3 ns. So the short lengths bend the line, and over 1-32, where they are a smaller share,
  the halves come within 1.4%.
- The slope depends on the span, and on the run. Over 1-16 every run of ten reads within 0.4 ns
  of 74.4 ns a step for `so-spntfe` and of 76.2 ns for `sos-futex-spntfe`. Over 1-32 eight runs
  of ten read 68.1 to 68.6 and 69.8 to 70.2 ns, and two read 72.3 and 74.1, and 74.6 and 74.8
  ns. So a two-thread bench runs at one of two rates about 6 ns a step apart, a whole run at
  one of them, and the span moves the odds. We think it is a state the two threads settle
  into, and nothing here says what sets it.
- At either rate the ring that can sleep reads slower than the spin-only one by about the same
  amount, 1.7 ns a step over 1-16 and 1.8 ns over 1-32, 2.3% and 2.6%. `linear-regression`
  names no cost for three of those four groups, so this is a pointer and not a measurement.

So for one thread the regression gives the step's cost and the sample's overhead. For two
threads the period disappears with random lengths, and what remains, a bend at short lengths and
two rates a run may land on, still keeps a line from holding in most groups.

## With the overhead taken off

The harness wraps a sample in the same clock reads whatever it times, so its overhead is one
number for a host. `linear-regression` takes it from `min-now`'s intercept, 18.53 ns from the
fixed counts and 18.16 ns from the random lengths, and takes it off each length's sample. What
is left over `inner` is the cost of a step at that length, the `step ns` column of the
command's output. The columns are the bench, its `inner`, and in ns a step: the step at a length
of 1, the lowest step and the length it falls at, and the step at the longest length:

| bench | `inner` | step at 1 | lowest step, at | step at the longest |
|---|---|---|---|---|
| `so-spntfe` | fixed, 1 to 16 | 71.92 | 62.79, 3 | 68.14 |
| `so-spntfe` | random, 1-32 | 78.55 | 68.29, 31 | 68.36 |
| `sos-futex-spntfe` | fixed, 1 to 16 | 74.02 | 64.19, 8 | 66.64 |
| `sos-futex-spntfe` | random, 1-32 | 80.18 | 70.20, 30 | 70.21 |

- Most of what a sample of one step read over a long one was the harness. The times per step
  at `inner` 1 were 90.5 and 92.6 ns, 21 and 25 ns over those at 16, and with the overhead off the
  step at 1 is 3.8 and 7.4 ns over the step at 16.
- With random lengths, a sample of one step costs about 10 ns more than a step of a long burst,
  for both rings. Over 1-32 `so-spntfe` holds a line, and its intercept less the overhead is
  7.42 ns, what the ring pays once a sample. We think it is the ring starting a burst.
- The controls at `--inner 7` and `8` are two lengths, too few to regress, and are given a step
  by the subtraction: `so-spntfe` 68.44 and 68.14 ns, `sos-futex-spntfe` 73.36 and 64.65 ns, the
  period of 4 still in the second.
- Whether a clock read costs the same after a ring step as after another clock read is not
  measured, so the overhead is `min-now`'s and is taken as the ring's.

## Fixed auto against a span around it

Whether auto should choose a span rather than one count, measured at `feat: how inner is chosen
and shown` (2026-10-16) on the 3900X `ccx` pair, CPU-pinned with the clock unpinned. Three
benches, each at fixed auto and at a span around auto's count, about 8 either side and at least
16 counts wide, floored at 1, in three rounds that flip the order, ten runs an invocation. The
records and the session's script are in
[records/inner-auto-3900x-20261016/](../records/inner-auto-3900x-20261016/README.md). The level
is the trimmed mean of the runs, what a run reports, and the slope is `linear-regression`'s over
the span's runs:

| bench | fixed auto | level ns | span | level ns | slope ns/step | line |
|---|---|---|---|---|---|---|
| `min-now` | 23, one run 22 | 24.596 | 15-30 | 24.676 | 23.563 | holds, halves 0.09% apart |
| `zcr-mpsc-v4-2t-single-sos-futex-spntfe` | 6 | 123.044 | 1-16 | 129.526 | 117.976 | holds, 0.09% |
| `zcr-spsc-v4-2t` | 7, one run 6 | 106.327 | 1-16 | 110.229 | 105.735 | does not hold, 2.58% |

`analyze --compare arm=fixed,span` calls all three beyond their claim:

```
bench                                   placement  arm A -> B          A trim     B trim      d%  claim%  d/claim  pairing         verdict
min-now                                 ccx        fixed -> span       24.596     24.676    0.33    0.09      3.7  groups 3/3      detected
zcr-mpsc-v4-2t-single-sos-futex-spntfe  ccx        fixed -> span      123.044    129.526    5.27    0.40     13.2  groups 3/3      detected
zcr-spsc-v4-2t                          ccx        fixed -> span      106.327    110.229    3.67    2.80      1.3  groups 3/3      detected
```

- **A level follows the lengths it averages.** A run's level is the mean over its samples of
  each one's time over its length, so what a sample pays once, the intercept, is paid at
  intercept over length. The level is then the slope plus the intercept times the mean of 1 over
  the length. It checks to within 0.01 ns on `min-now`'s two arms and the mpsc ring's span, the
  ring's fixed arm differing by the extra cost below:
  - `min-now`, intercept 23.79 ns: over 15-30 the mean of 1 over the length is 0.0465, so
    23.563 + 1.105 = 24.668 against 24.676 read, and at 23 it is 23.563 + 1.034 = 24.597
    against 24.596.
  - The mpsc ring, intercept 54.66 ns: over 1-16 the mean is 0.2113, so 117.976 + 11.550 =
    129.526 against 129.526 read.
  So a span's level is set by the span chosen, and 1-16, which weights short samples the most,
  reads every ring higher than fixed auto does.
- **Under a span a ring pays more a sample.** At one length the rings' samples are slower in a
  span than in a fixed run: the mpsc ring at 6 takes 762.85 ns against 738.30, 24.55 ns more,
  and the spsc ring at 7 takes 752.92 against 739.14, 13.78 more, where `min-now` at 23 takes
  565.80 against 565.62. Less 6 steps at the slope, the mpsc ring's fixed sample leaves 30.44 ns
  a sample against the span's intercept of 54.66, so the extra is paid once a sample, in the
  intercept, and the slope does not see it. We think it is a length the ring cannot predict,
  costing it at each sample's start.
- **The spsc ring holds no line over 1-16.** It is high at 1 and 2, 13.39 and 5.26 ns over the
  line, and its runs land on different slopes, 16 of 30 more than 1% from their median, with 21
  ns of scatter between runs at one length. No slope names its cost, and its level in either arm
  is a mix of where its runs landed.
- **A span auto is better only with the slope as the level.** Kept as the mean over samples, a
  span reads higher than fixed auto on all three, by an amount the span sets. The slope does not
  depend on the span where a line holds, two of three here, and does not pay the span's extra
  cost a sample. A bench whose line does not hold, like the spsc ring here, still needs a level
  of some other kind.

## What it does not hold

- `Multi`, the `smt` pair for two threads, and `inner` past 16. A larger sweep was stopped for
  its length, and its one pass, 800 runs, is not committed.
- The 3900X and the Pi 5.
- Why a fixed `inner` gives the ring that can sleep a period of 4, which change random lengths
  bring ends it, and what sets which of the two rates a run lands on.
