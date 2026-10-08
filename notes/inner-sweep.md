# An inner sweep on the 7600X

A bench's level is a sample's time divided by `inner`, the steps timed back to back in the
sample, and the harness sizes `inner` by itself in every run. This file holds a sweep that fixes
`inner` by hand over a range, to show what the level does as it changes, from `feat: fit, the
cost of a step from samples of many lengths` (2026-10-08). The records are what the `fit`
subcommand is written against.

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

## What it does not hold

- `Multi`, the `smt` pair for two threads, and `inner` past 16. A larger sweep was stopped for
  its length, and its one pass, 800 runs, is not committed.
- The 3900X and the Pi 5.
- Why the ring that can sleep has a period of 4, which `TODO.md`'s entry on it carries.
