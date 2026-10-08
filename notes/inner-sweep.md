# An inner sweep on the 7600X

A bench's level is a sample's time divided by `inner`, the steps timed back to back in the
sample, and the harness sizes `inner` by itself in every run. This file holds a sweep that fixes
`inner` by hand over a range, to show what the level does as it changes, from `feat: fit, the
cost of a step from samples of many lengths` (2026-10-08). The records are what the `fit`
subcommand is written against. Below the sweep are runs that draw `inner` per sample, taken
once the harness could, and what `fit` says of both.

## Why it was taken

Two overnight sessions on one binary, five invocations of ten runs and one of fifty, disagreed
with each other and with the session in [mpsc-v4-mode-waits.md](mpsc-v4-mode-waits.md) on
two-thread benches at cross-core placements, by more than their claims allowed. Three probes on
the 7600X `ccx` pair, on `zcr-mpsc-v4-2t-single-sos-futex-spntfe`, found what moved the level.
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
  [records/inner-sweep-7600x.jsonl](../records/inner-sweep-7600x.jsonl), 336 runs.

The loop, pass b taking each list reversed:

```
T2="zcr-mpsc-v4-2t-single-so-spntfe zcr-mpsc-v4-2t-single-sos-futex-spntfe \
    zcr-mpsc-v4-2t-single-slp-futex-slptfe"
for i in 13 4 9 1 7 11 2 16 5 14 3 8 12 6 10 15; do
  iiac-perf $T2 --pin-cpus ccx --runs 3 --run-sleep 100-200ms --inner $i \
    --tag pass=a --record-file inner-sweep-7600x.jsonl
done
for i in 60 20 120 40 100 30 80 50; do
  iiac-perf min-now --pin-cpus smt --runs 3 --run-sleep 100-200ms --inner $i \
    --tag pass=a --record-file inner-sweep-7600x.jsonl
done
```

## One thread

`min-now`, the median of six runs. A sample's time is the level times `inner`:

| `inner` | level ns | sample ns |
|---|---|---|
| 20 | 19.190 | 383.8 |
| 30 | 18.885 | 566.6 |
| 40 | 18.729 | 749.2 |
| 50 | 18.635 | 931.7 |
| 60 | 18.572 | 1114.3 |
| 80 | 18.499 | 1479.9 |
| 100 | 18.449 | 1844.9 |
| 120 | 18.418 | 2210.2 |

The level falls as `inner` grows and never settles. A line through sample time against `inner`,
fitted by hand, has a slope of 18.264 ns a step in each pass and an intercept of 18.61 ns in pass
a and 18.57 ns in pass b. So a sample costs about one clock read beyond its steps, and the level
carries that cost divided by `inner`.

## Two threads

`zcr-mpsc-v4-2t-single-*` on `ccx`, the level in ns, the median of six runs:

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

- At `inner` 1 the two spinning benches read 21 and 25 ns above their levels at 16.
- `sos-futex-spntfe` is low at 4, 8, 12 and 16 and high at the odd values between, a period of 4.
  `so-spntfe` has none, only a slow wave, low at 5 and high at 9.
- Which of the two is faster depends on `inner`: at 8 the ring that can sleep reads 3.7 ns under
  the spin-only one, and at 7 it reads 4.7 ns over.
- The two passes agree. The same hand fit, a simple linear regression of sample time on `inner`,
  gives `so-spntfe` 70.48 and 70.46 ns a step and `sos-futex-spntfe` 69.61 and 69.54 ns a step.
- A line does not describe them, though, so those slopes are not the cost of a step. The slope
  moves with the range fitted where `min-now`'s stays at 18.26 ns over any part of its range:

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

## Drawn per sample

A run at a fixed `inner` repeats one burst length, so the sweep cannot say whether a two-thread
bench's cost follows the length or the repetition. `--inner 1-16` draws each sample's length
from the span, and the record holds the samples and their summed time at each length.

- Source: commit `124244db`, jj change `vrxrlwwy`, a clean tree, `feat: inner drawn per sample
  from a range`, `iiac-perf-dev` 0.28.24-2, binary `f53a7f480a66675f`, built on the 3900X.
- The same host, placements, and benches as the sweep. Two passes of five runs, tagged `pass`,
  2026-10-08T17:30Z to 17:34Z, `--run-sleep 100-200ms`:
  - the three two-thread benches on `ccx` at `--inner 1-16` and at `--inner 1-32`
  - `min-now` on `smt` at `--inner 20-120`
  - the two spinning benches at `--inner 7` and at `--inner 8`, fixed, as a control on the
    binary
- Records, every switch count zero:
  [records/inner-drawn-7600x.jsonl](../records/inner-drawn-7600x.jsonl), 110 runs.

The control reads as the landed binary did: `sos-futex-spntfe` 76.0 ns at 7 and 67.0 ns at 8,
`so-spntfe` 71.1 and 70.6 ns. So the new binary's fixed loop measures what the old one did.

## What fit says

`iiac-perf-dev fit records/inner-sweep-7600x.jsonl` and the same on
`records/inner-drawn-7600x.jsonl`. `fit` is a simple linear regression of sample time on
`inner`, by ordinary least squares. It refits each half of the lengths and names a cost only
where the two slopes agree within 1%, "a line holds". The slope is in ns a step and the
intercept in ns:

| bench | `inner` | slope | intercept | halves apart % | period | a line |
|---|---|---|---|---|---|---|
| `min-now` | fixed, 20 to 120 | 18.264 | 18.53 | 0.01 | | holds |
| `min-now` | drawn, 20-120 | 18.256 | 18.16 | 0.02 | none | holds |
| `so-spntfe` | fixed, 1 to 16 | 70.435 | 5.78 | 5.53 | none | does not hold |
| `so-spntfe` | drawn, 1-16 | 74.438 | 7.08 | 5.00 | none | does not hold |
| `so-spntfe` | drawn, 1-32 | 68.165 | 25.58 | 0.95 | none | holds |
| `sos-futex-spntfe` | fixed, 1 to 16 | 69.557 | 18.18 | 6.25 | 4 | does not hold |
| `sos-futex-spntfe` | drawn, 1-16 | 76.155 | 11.96 | 2.69 | none | does not hold |
| `sos-futex-spntfe` | drawn, 1-32 | 69.968 | 28.87 | 1.36 | none | does not hold |

`slp-futex-slptfe` is left out: in all three its runs scatter too much for the halves to say.

- One thread: both ways give `min-now` 18.26 ns a step and about 18.4 ns a sample, the two
  slopes 0.008 ns apart. The regression holds and the two ways agree.
- The period of 4 followed the repetition. Fixed, `sos-futex-spntfe`'s residuals by `inner`
  modulo 4 are -31.7, +14.9, +0.3 and +16.4 ns a sample, a period that accounts for 71% of the
  other pass's residual scatter. Drawn over 1-16 they are -0.9, +1.4, -0.1 and -0.4 ns, and
  `fit` finds no period in any drawn group.
- What is left is smooth. A sample of one step sits 10 ns over the line for `sos-futex-spntfe`
  and 15 ns for `so-spntfe`, the next few under it, and from about 8 steps the level is flat
  to 0.3 ns. So the short lengths bend the line, and over 1-32, where they are a smaller share,
  the halves come within 1.4%.
- The slope depends on the span, and on the run. Over 1-16 every run of ten reads within 0.4 ns
  of 74.4 ns a step for `so-spntfe` and of 76.2 ns for `sos-futex-spntfe`. Over 1-32 eight runs
  of ten read 68.1 to 68.6 and 69.8 to 70.2 ns, and two read 72.3 and 74.1, and 74.6 and 74.8
  ns. So a two-thread bench runs at one of two rates about 6 ns a step apart, a whole run at
  one of them, and the span moves the odds. We think it is a state the two threads settle
  into, and nothing here says what sets it.
- At either rate the ring that can sleep reads slower than the spin-only one by about the same
  amount, 1.7 ns a step over 1-16 and 1.8 ns over 1-32, 2.3% and 2.6%. `fit` names no cost for
  three of those four groups, so this is a pointer and not a measurement.

So for one thread the regression gives the step's cost and the sample's overhead. For two
threads drawing the lengths removes the period, and what remains, a bend at short lengths and
two rates a run may land on, still keeps a line from holding in most groups.

## What it does not hold

- `Multi`, the `smt` pair for two threads, and `inner` past 16. A larger sweep was stopped for
  its length, and its one pass, 800 runs, is not committed.
- The 3900X and the Pi 5.
- Why repeating one burst length gives the ring that can sleep a period of 4, and what sets
  which of the two rates a run lands on.
