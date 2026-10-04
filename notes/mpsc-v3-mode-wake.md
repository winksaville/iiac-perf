# What mpsc v3's mode and wake cost

zc-ring-x1's `mpsc::v3` ring takes two type parameters, a segment mode, `Single` or `Multi`, and a
wake, `NoWake` or `Futex`. This file holds what the twelve `zcr-mpsc-v3` benches measured of them,
from `feat: mpsc v3 benches` (2026-10-04), and what that says about keeping the two parameters.

It rests on one session on one host, a laptop, and is provisional until the measuring hosts repeat
it ([Measure mpsc v3's mode and wake on the measuring
hosts](../TODO.md#measure-mpsc-v3s-mode-and-wake-on-the-measuring-hosts)).

## The session

- Host: an i5-1135G7 laptop, four cores and eight threads, `intel_pstate`, the clock pinned at
  2400 MHz, its base.
- Benches: `zcr-mpsc-v2-1t` and `-2t`, and the twelve `zcr-mpsc-v3` benches, against zc-ring-x1
  `83ab431`. Each v3 bench is v2's round trip with one message in flight, and every wait spins.
- Run: five invocations of the fourteen, ten runs each, at every placement the host declares:
  `smt`, CPUs 3 and 7, `ccx`, CPUs 3 and 2, two cores on the one L3, and unpinned. A one-thread
  bench has no `ccx` of its own.
- One binary, `e348cbb74d9f7c64`, built with rustc 1.99.0 from the tree one edit past `f27228d`,
  the version-of-record and `TODO.md` alone differing.
- Records: [records/mpsc-v3-fwlaptop.jsonl](../records/mpsc-v3-fwlaptop.jsonl), 1750 runs, every
  segment-switch count in them zero.
- Numbers are `analyze --compare`'s: each side's trimmed mean in ns, the difference as a percent
  of the first side's, and the claim, the smallest difference the pairing could call real.

## v3 against v2

`zcr-mpsc-v2` against `zcr-mpsc-v3`, `Multi` over two segments with `NoWake`, v2's geometry.

| bench | placement | v2 | v3 | d% | claim% | verdict |
|---|---|---|---|---|---|---|
| `1t` | smt | 22.41 | 22.48 | +0.32 | 0.11 | detected |
| `1t` | unpinned | 22.40 | 22.48 | +0.36 | 0.08 | detected |
| `2t` | smt | 117.85 | 112.30 | -4.71 | 0.49 | detected |
| `2t` | ccx | 478.10 | 465.00 | -2.74 | 1.17 | detected |
| `2t` | unpinned | 486.91 | 471.71 | -3.12 | 2.42 | detected |

- At one thread v3 is 0.07 ns slower, a third of a percent.
- At two threads v3 is faster at every placement, by 3 to 5%.

## Multi over one segment against two

`zcr-mpsc-v3` against `zcr-mpsc-v3-*-1seg`, both `Multi` with `NoWake`.

| bench | placement | two | one | d% | claim% | verdict |
|---|---|---|---|---|---|---|
| `1t` | smt | 22.48 | 22.47 | -0.02 | 0.06 | not seen |
| `1t` | unpinned | 22.48 | 22.48 | 0.00 | 0.03 | not seen |
| `2t` | smt | 112.30 | 112.31 | +0.01 | 0.40 | not seen |
| `2t` | ccx | 465.00 | 463.69 | -0.28 | 1.39 | not seen |
| `2t` | unpinned | 471.71 | 467.87 | -0.81 | 1.16 | not seen |

- A second segment costs nothing seen while no switch happens.

## Single against Multi over one segment

`-1seg` against `-single`, the geometry equal and the mode alone differing, under each wake.

| bench | wake | placement | `Multi` | `Single` | d% | claim% | verdict |
|---|---|---|---|---|---|---|---|
| `1t` | `NoWake` | smt | 22.47 | 20.49 | -8.81 | 0.06 | detected |
| `1t` | `NoWake` | unpinned | 22.48 | 20.50 | -8.83 | 0.05 | detected |
| `1t` | `Futex` | smt | 23.31 | 21.11 | -9.45 | 0.06 | detected |
| `1t` | `Futex` | unpinned | 23.31 | 21.11 | -9.45 | 0.05 | detected |
| `2t` | `NoWake` | smt | 112.31 | 114.14 | +1.64 | 0.20 | detected |
| `2t` | `NoWake` | ccx | 463.69 | 459.80 | -0.84 | 1.00 | not seen |
| `2t` | `NoWake` | unpinned | 467.87 | 462.11 | -1.23 | 1.84 | not seen |
| `2t` | `Futex` | smt | 118.30 | 115.66 | -2.23 | 0.94 | detected |
| `2t` | `Futex` | ccx | 474.02 | 472.26 | -0.37 | 0.99 | not seen |
| `2t` | `Futex` | unpinned | 479.69 | 478.66 | -0.22 | 1.11 | not seen |

- At one thread `Single` is 2.0 to 2.2 ns faster, 9%, under both wakes and at both placements,
  and two earlier sessions of earlier binaries read the same, -8.62% and -8.67%.
- At two threads no difference holds: `not seen` four times of six, and the two `detected` at
  `smt` have opposite signs.

## Futex against NoWake

Each `NoWake` bench against its `-futex` twin. Nobody sleeps, so the difference is the wake checks
left on the message path, the producer's look at the waiting flag after its commit and the
consumer's fence at each half segment of releases, and not the cost of a sleep and a wake.

| bench | placement | `NoWake` | `Futex` | d% | claim% | verdict |
|---|---|---|---|---|---|---|
| `1t` | smt | 22.48 | 23.31 | +3.69 | 0.09 | detected |
| `1t` | unpinned | 22.48 | 23.31 | +3.70 | 0.06 | detected |
| `1t-1seg` | smt | 22.47 | 23.31 | +3.72 | 0.08 | detected |
| `1t-1seg` | unpinned | 22.48 | 23.31 | +3.70 | 0.08 | detected |
| `1t-single` | smt | 20.49 | 21.11 | +2.99 | 0.03 | detected |
| `1t-single` | unpinned | 20.50 | 21.11 | +3.00 | 0.03 | detected |
| `2t` | smt | 112.30 | 118.66 | +5.67 | 0.32 | detected |
| `2t` | ccx | 465.00 | 478.97 | +3.00 | 0.96 | detected |
| `2t` | unpinned | 471.71 | 485.27 | +2.87 | 1.43 | detected |
| `2t-1seg` | smt | 112.31 | 118.30 | +5.34 | 0.80 | detected |
| `2t-1seg` | ccx | 463.69 | 474.02 | +2.23 | 1.00 | detected |
| `2t-1seg` | unpinned | 467.87 | 479.69 | +2.53 | 1.51 | detected |
| `2t-single` | smt | 114.14 | 115.66 | +1.33 | 0.84 | detected |
| `2t-single` | ccx | 459.80 | 472.26 | +2.71 | 0.90 | detected |
| `2t-single` | unpinned | 462.11 | 478.66 | +3.58 | 2.23 | detected |

- `Futex` is the slower in all fifteen: 0.6 to 0.8 ns at one thread, 3 to 4%, and 1.5 to 6.4 ns
  at `smt`, 10 to 14 ns at `ccx`, and 12 to 17 ns unpinned at two threads, 1 to 6%.

## What the numbers can carry

- One build of each bench: a mode or a wake is different code at different addresses, and with
  this repo's build settings layout alone moved single benches up to 8% ([build.md](build.md)).
  A difference in one bench at one placement is therefore not yet the parameter's.
- What repeats is: the `Futex` cost has one sign across fifteen rows of six benches, and the
  one-thread `Single` gain has one size across four rows and three sessions' binaries. We think
  both are the parameters' and not layout's.
- One host, and a laptop: its claims at two threads are 0.2 to 2.4%, where a measuring host's
  are tighter, and a ring's ranking has differed between hosts before
  ([Results by placement](../docs/report-guide.md#results-by-placement)).

## Whether the mode and the wake earn their place

Provisional, on the above, and zc-ring-x1's to decide.

- The mode: `Single` buys 2 ns of a 22 ns same-thread round trip and nothing shown where two
  threads hand off, which is what a ring between threads does, and `Multi` over one segment reads
  as `Multi` over two. We think the mode does not earn its place, and a ring that is always
  `Multi` loses only the same-thread case.
- The wake: a ring that can sleep pays 3 to 4% of every round trip when nobody sleeps, at every
  placement. We think that is too much to charge a ring that only spins, so the wake earns its
  place as a parameter unless its checks get cheaper, and what a sleep and a wake cost against a
  spin is not measured here.
