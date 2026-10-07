# What mpsc v3's mode and wake cost

zc-ring-x1's `mpsc::v3` ring takes two type parameters, a segment mode, `Single` or `Multi`, and a
wake, `NoWake` or `Futex`. This file holds what the twelve `zcr-mpsc-v3` benches measured of them,
from `feat: mpsc v3 benches` (2026-10-04), and what that says about keeping the two parameters.

Its first session was one host's, a laptop's, and its answer provisional. The three measuring
hosts ran the benches at `feat: mpsc v4 benches` (2026-10-06), their tables are in [The measuring
hosts](#the-measuring-hosts), and [the answer](#whether-the-mode-and-the-wake-earn-their-place) is
theirs where the two disagree.

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

## The names

The session ran before the benches were renamed, at `feat: mpsc v3 names and waiting benches`
(2026-10-06), so its records hold the earlier names and the tables below keep their suffixes. A
name now states its mode, how long its receivers spin at an empty ring, `st`, how long they then
wait on the ring's waiter, `wt`, and the waiter, where an earlier name said only how it differed
from `Multi` over two segments with `NoWake`.

| earlier, after `zcr-mpsc-v3-<1t or 2t>` | now, after `zcr-mpsc-v3-<1t or 2t>` |
|---|---|
| nothing | `-multi-2seg-stfe-wtnone` |
| `-1seg` | `-multi-1seg-stfe-wtnone` |
| `-single` | `-single-stfe-wtnone` |
| `-futex` | `-multi-2seg-stfe-wtnone-futex` |
| `-1seg-futex` | `-multi-1seg-stfe-wtnone-futex` |
| `-single-futex` | `-single-stfe-wtnone-futex` |

- Every bench is `stfe-wtnone`, its receivers spinning forever and so never reaching a wait.
  `st` and `wt` are the receiver's and the last field is the ring's, so a bench and its twin over
  `Futex` differ in that field alone, as the two rings do.
- `analyze` over the session's records takes the earlier names, and a session run since takes
  the names above.

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

zc-ring-x1's to decide, and on [The measuring hosts](#the-measuring-hosts) both do.

- The mode earns its place. The laptop showed `Single` buying 2 ns of a 22 ns same-thread round
  trip and nothing where two threads hand off, and on that we thought it did not. The measuring
  hosts show it at two threads as well: 2.3 to 4.6%, 1.2 to 2.3 ns, on a shared core on both x86
  hosts, and 1.1 to 1.8% on the Pi 5. So the provisional answer is withdrawn, and zc-ring-x1's
  own finding, that a ring with the mode dropped is slower (`m-8-2`), stands with ours.
- The wake earns its place. A ring that can sleep pays when nobody sleeps on every host, 4 to 18%
  of a one-thread round trip and 0.4 to 8.7% of a two-thread one, more than the laptop's 3 to 4%.
  That is too much to charge a ring that only spins.
- What a sleep and a wake cost against a spin v3 could not show, and v4 does
  ([mpsc-v4-mode-waits.md](mpsc-v4-mode-waits.md#what-a-wait-costs)).

## Since the session

Three things from `feat: mpsc v3 names and waiting benches` (2026-10-06), before the measuring
hosts' session.

- A first reading from a measuring host, the 7600X, one invocation of ten runs each at `smt`,
  CPUs 5 and 11, the clock pinned at 4701 MHz, wink's run:

  | bench | trimmed mean ns |
  |---|---|
  | `zcr-mpsc-v3-2t-single-stfe-wtnone` | 45.87 |
  | `zcr-mpsc-v3-2t-single-stfe-wtnone-futex` | 48.95 |

  - The `Futex` ring is 3.09 ns the slower, 6.7%, the laptop's sign and above its size for this
    pair, +1.33% at `smt`. A run earlier the same day read 2.9 ns, and three runs at one thread
    read 4.05 against 4.34 ns.
  - It is one invocation from an uncommitted tree with no record kept, so it is a reading and
    not a row of the tables above.
- What the cost is of: the checks follow the ring's wake and not the caller's policy, a producer
  looking for a waiting consumer after its commit and a release looking for sleeping producers,
  whoever calls. The endpoint awake pays so the one asleep can be woken, so the cost is the
  ring's, and it is what lets one endpoint spin while another sleeps.
  - zc-ring-x1 was told at `m-8-3`, and its `mpsc::v4` has a ring's type say how its endpoints
    wait, `SpinOnly`, `Sleep<Futex>`, or `SpinOrSleep<Futex>`, the timed calls offered by that
    choice (`m-8-6`). v3 keeps `NoWake` and `Futex`.
- The mode: zc-ring-x1's first measurements of a ring with the mode dropped show a performance
  hit, so v4 keeps `Single` and `Multi` (`m-8-2`, wink, 2026-10-06). That was against this note's
  provisional answer on the mode, which the measuring hosts' session has since withdrawn.

## The measuring hosts

The session of `feat: mpsc v4 benches` (2026-10-06), one binary measuring v2's pair, v3's twelve,
and v4's fifteen on the 3900X, the 7600X, and the Pi 5. Its source, binaries, build, placements,
and records are in [mpsc-v4-mode-waits.md](mpsc-v4-mode-waits.md#the-session), with v4's tables.
The benches ran under the names of [The names](#the-names), and a table's `checks` marks the ring
over `Futex`.

### v3 against v2

| host | bench | placement | v2 | v3 | d% | claim% | verdict |
|---|---|---|---|---|---|---|---|
| 3900X | `1t` | smt | 8.567 | 7.774 | -9.25 | 0.08 | detected |
| 3900X | `1t` | unpinned | 8.618 | 7.860 | -8.80 | 0.12 | detected |
| 3900X | `2t` | smt | 66.680 | 66.366 | -0.47 | 0.06 | detected |
| 3900X | `2t` | ccx | 126.066 | 123.074 | -2.37 | 0.10 | detected |
| 3900X | `2t` | x-ccx | 415.030 | 394.369 | -4.98 | 0.43 | detected |
| 3900X | `2t` | x-ccd | 394.050 | 389.443 | -1.17 | 0.35 | detected |
| 3900X | `2t` | unpinned | 129.955 | 128.246 | -1.32 | 3.51 | not seen |
| 7600X | `1t` | smt | 5.196 | 5.035 | -3.10 | 0.19 | detected |
| 7600X | `1t` | unpinned | 5.197 | 5.040 | -3.03 | 0.11 | detected |
| 7600X | `2t` | smt | 47.885 | 48.778 | +1.87 | 0.18 | detected |
| 7600X | `2t` | ccx | 73.420 | 72.684 | -1.00 | 0.31 | detected |
| 7600X | `2t` | unpinned | 73.884 | 73.283 | -0.81 | 0.43 | detected |
| Pi 5 | `1t` | ccx | 27.946 | 28.655 | +2.54 | 0.37 | detected |
| Pi 5 | `1t` | unpinned | 27.930 | 28.553 | +2.23 | 0.13 | detected |
| Pi 5 | `2t` | ccx | 248.731 | 246.175 | -1.03 | 0.08 | detected |
| Pi 5 | `2t` | unpinned | 248.643 | 246.117 | -1.02 | 0.15 | detected |

- No one answer: v3 is faster in 12 of 16 rows and slower in 3, the Pi 5 at one thread, 2.2 and
  2.5%, and the 7600X's `smt` pair at two, 1.9%.
- On the 3900X v3 is faster at every pinned placement, 9% at one thread and 0.5 to 5% at two.

### Multi over one segment against two

| host | bench | placement | two | one | d% | claim% | verdict |
|---|---|---|---|---|---|---|---|
| 3900X | `1t` | smt | 7.774 | 7.775 | +0.01 | 0.25 | not seen |
| 3900X | `1t` | unpinned | 7.860 | 7.849 | -0.13 | 0.34 | not seen |
| 3900X | `1t, checks` | smt | 8.598 | 8.602 | +0.04 | 0.28 | not seen |
| 3900X | `1t, checks` | unpinned | 8.675 | 8.674 | -0.02 | 0.17 | not seen |
| 3900X | `2t` | smt | 66.366 | 66.404 | +0.06 | 0.09 | not seen |
| 3900X | `2t` | ccx | 123.074 | 127.346 | +3.47 | 0.20 | detected |
| 3900X | `2t` | x-ccx | 394.369 | 413.931 | +4.96 | 0.42 | detected |
| 3900X | `2t` | x-ccd | 389.443 | 404.627 | +3.90 | 0.36 | detected |
| 3900X | `2t` | unpinned | 128.246 | 130.041 | +1.40 | 5.70 | not seen |
| 3900X | `2t, checks` | smt | 71.673 | 71.662 | -0.01 | 0.10 | not seen |
| 3900X | `2t, checks` | ccx | 124.111 | 130.961 | +5.52 | 0.13 | detected |
| 3900X | `2t, checks` | x-ccx | 397.243 | 418.804 | +5.43 | 0.30 | detected |
| 3900X | `2t, checks` | x-ccd | 391.054 | 411.226 | +5.16 | 0.27 | detected |
| 3900X | `2t, checks` | unpinned | 130.919 | 136.291 | +4.10 | 6.37 | not seen |
| 7600X | `1t` | smt | 5.035 | 5.038 | +0.06 | 0.15 | not seen |
| 7600X | `1t` | unpinned | 5.040 | 5.041 | +0.03 | 0.08 | not seen |
| 7600X | `1t, checks` | smt | 5.339 | 5.342 | +0.07 | 0.18 | not seen |
| 7600X | `1t, checks` | unpinned | 5.342 | 5.340 | -0.04 | 0.11 | not seen |
| 7600X | `2t` | smt | 48.778 | 48.135 | -1.32 | 0.12 | detected |
| 7600X | `2t` | ccx | 72.684 | 72.635 | -0.07 | 0.27 | not seen |
| 7600X | `2t` | unpinned | 73.283 | 73.250 | -0.04 | 0.64 | not seen |
| 7600X | `2t, checks` | smt | 50.219 | 50.156 | -0.13 | 0.06 | detected |
| 7600X | `2t, checks` | ccx | 73.561 | 73.344 | -0.30 | 0.12 | detected |
| 7600X | `2t, checks` | unpinned | 74.090 | 74.045 | -0.06 | 0.14 | not seen |
| Pi 5 | `1t` | ccx | 28.655 | 28.608 | -0.16 | 0.68 | not seen |
| Pi 5 | `1t` | unpinned | 28.553 | 28.551 | -0.01 | 0.22 | not seen |
| Pi 5 | `1t, checks` | ccx | 30.137 | 30.187 | +0.16 | 0.37 | not seen |
| Pi 5 | `1t, checks` | unpinned | 30.171 | 30.154 | -0.05 | 0.54 | not seen |
| Pi 5 | `2t` | ccx | 246.175 | 245.938 | -0.10 | 0.13 | not seen |
| Pi 5 | `2t` | unpinned | 246.117 | 245.594 | -0.21 | 0.13 | detected |
| Pi 5 | `2t, checks` | ccx | 249.116 | 248.772 | -0.14 | 0.33 | not seen |
| Pi 5 | `2t, checks` | unpinned | 248.509 | 248.009 | -0.20 | 0.14 | detected |

- At one thread nothing is seen on any host, as on the laptop.
- On the 3900X across cores one segment is the slower in all six pinned rows, 3.5 to 5.5%, 4 to
  22 ns, which the laptop did not show. The 7600X and the Pi 5 read level or one segment a little
  faster, 1.3% at most.
- v4's rows read the same, and what we think of it is in
  [mpsc-v4-mode-waits.md](mpsc-v4-mode-waits.md#multi-over-one-segment-against-two).

### Single against Multi over one segment

| host | bench | placement | `Multi` | `Single` | d% | claim% | verdict |
|---|---|---|---|---|---|---|---|
| 3900X | `1t` | smt | 7.775 | 6.379 | -17.95 | 0.22 | detected |
| 3900X | `1t` | unpinned | 7.849 | 6.444 | -17.90 | 0.20 | detected |
| 3900X | `1t, checks` | smt | 8.602 | 7.516 | -12.62 | 0.13 | detected |
| 3900X | `1t, checks` | unpinned | 8.674 | 7.579 | -12.62 | 0.14 | detected |
| 3900X | `2t` | smt | 66.404 | 64.100 | -3.47 | 0.13 | detected |
| 3900X | `2t` | ccx | 127.346 | 125.381 | -1.54 | 0.11 | detected |
| 3900X | `2t` | x-ccx | 413.931 | 385.963 | -6.76 | 0.33 | detected |
| 3900X | `2t` | x-ccd | 404.627 | 376.603 | -6.93 | 0.34 | detected |
| 3900X | `2t` | unpinned | 130.041 | 137.247 | +5.54 | 11.79 | not seen |
| 3900X | `2t, checks` | smt | 71.662 | 69.650 | -2.81 | 0.05 | detected |
| 3900X | `2t, checks` | ccx | 130.961 | 126.131 | -3.69 | 0.31 | detected |
| 3900X | `2t, checks` | x-ccx | 418.804 | 395.841 | -5.48 | 0.30 | detected |
| 3900X | `2t, checks` | x-ccd | 411.226 | 386.265 | -6.07 | 0.27 | detected |
| 3900X | `2t, checks` | unpinned | 136.291 | 130.542 | -4.22 | 5.50 | not seen |
| 7600X | `1t` | smt | 5.038 | 4.015 | -20.29 | 0.12 | detected |
| 7600X | `1t` | unpinned | 5.041 | 4.019 | -20.28 | 0.11 | detected |
| 7600X | `1t, checks` | smt | 5.342 | 4.335 | -18.85 | 0.19 | detected |
| 7600X | `1t, checks` | unpinned | 5.340 | 4.337 | -18.79 | 0.06 | detected |
| 7600X | `2t` | smt | 48.135 | 45.906 | -4.63 | 0.09 | detected |
| 7600X | `2t` | ccx | 72.635 | 70.747 | -2.60 | 0.54 | detected |
| 7600X | `2t` | unpinned | 73.250 | 70.996 | -3.08 | 0.70 | detected |
| 7600X | `2t, checks` | smt | 50.156 | 48.984 | -2.34 | 0.07 | detected |
| 7600X | `2t, checks` | ccx | 73.344 | 75.643 | +3.13 | 0.08 | detected |
| 7600X | `2t, checks` | unpinned | 74.045 | 76.083 | +2.75 | 0.11 | detected |
| Pi 5 | `1t` | ccx | 28.608 | 28.508 | -0.35 | 0.59 | not seen |
| Pi 5 | `1t` | unpinned | 28.551 | 28.492 | -0.21 | 0.32 | not seen |
| Pi 5 | `1t, checks` | ccx | 30.187 | 29.696 | -1.62 | 0.73 | detected |
| Pi 5 | `1t, checks` | unpinned | 30.154 | 29.656 | -1.65 | 0.63 | detected |
| Pi 5 | `2t` | ccx | 245.938 | 241.878 | -1.65 | 0.08 | detected |
| Pi 5 | `2t` | unpinned | 245.594 | 241.252 | -1.77 | 0.19 | detected |
| Pi 5 | `2t, checks` | ccx | 248.772 | 244.704 | -1.64 | 0.29 | detected |
| Pi 5 | `2t, checks` | unpinned | 248.009 | 245.177 | -1.14 | 0.16 | detected |

- At one thread `Single` is 1.0 to 1.4 ns faster on the x86 hosts, 13 to 20%, and 0.5 ns on the
  Pi 5 over `Futex`, 1.6%, its two `NoWake` rows not seen.
- At two threads `Single` is faster in 16 of 20 rows: every pinned row of the 3900X, 1.5 to 6.9%,
  the 7600X's `smt` pair and its `NoWake` rows, 2.3 to 4.6%, and all four of the Pi 5's, 1.1 to
  1.8%.
- It is slower in two, the 7600X over `Futex` at `ccx` and unpinned, 2.3 and 2.0 ns, 3.1 and
  2.8%, and the 3900X's two unpinned rows are not seen.
- The laptop's two-thread reading, no difference that holds, is not the measuring hosts'.

### Futex against NoWake

| host | bench | placement | `NoWake` | `Futex` | d% | claim% | verdict |
|---|---|---|---|---|---|---|---|
| 3900X | `1t-multi-2seg` | smt | 7.774 | 8.598 | +10.60 | 0.34 | detected |
| 3900X | `1t-multi-2seg` | unpinned | 7.860 | 8.675 | +10.38 | 0.32 | detected |
| 3900X | `1t-multi-1seg` | smt | 7.775 | 8.602 | +10.64 | 0.29 | detected |
| 3900X | `1t-multi-1seg` | unpinned | 7.849 | 8.674 | +10.50 | 0.34 | detected |
| 3900X | `1t-single` | smt | 6.379 | 7.516 | +17.82 | 0.17 | detected |
| 3900X | `1t-single` | unpinned | 6.444 | 7.579 | +17.61 | 0.27 | detected |
| 3900X | `2t-multi-2seg` | smt | 66.366 | 71.673 | +8.00 | 0.11 | detected |
| 3900X | `2t-multi-2seg` | ccx | 123.074 | 124.111 | +0.84 | 0.23 | detected |
| 3900X | `2t-multi-2seg` | x-ccx | 394.369 | 397.243 | +0.73 | 0.35 | detected |
| 3900X | `2t-multi-2seg` | x-ccd | 389.443 | 391.054 | +0.41 | 0.18 | detected |
| 3900X | `2t-multi-2seg` | unpinned | 128.246 | 130.919 | +2.08 | 4.59 | not seen |
| 3900X | `2t-multi-1seg` | smt | 66.404 | 71.662 | +7.92 | 0.13 | detected |
| 3900X | `2t-multi-1seg` | ccx | 127.346 | 130.961 | +2.84 | 0.16 | detected |
| 3900X | `2t-multi-1seg` | x-ccx | 413.931 | 418.804 | +1.18 | 0.23 | detected |
| 3900X | `2t-multi-1seg` | x-ccd | 404.627 | 411.226 | +1.63 | 0.24 | detected |
| 3900X | `2t-multi-1seg` | unpinned | 130.041 | 136.291 | +4.81 | 5.39 | not seen |
| 3900X | `2t-single` | smt | 64.100 | 69.650 | +8.66 | 0.09 | detected |
| 3900X | `2t-single` | ccx | 125.381 | 126.131 | +0.60 | 0.23 | detected |
| 3900X | `2t-single` | x-ccx | 385.963 | 395.841 | +2.56 | 0.26 | detected |
| 3900X | `2t-single` | x-ccd | 376.603 | 386.265 | +2.57 | 0.18 | detected |
| 3900X | `2t-single` | unpinned | 137.247 | 130.542 | -4.89 | 11.51 | not seen |
| 7600X | `1t-multi-2seg` | smt | 5.035 | 5.339 | +6.04 | 0.10 | detected |
| 7600X | `1t-multi-2seg` | unpinned | 5.040 | 5.342 | +6.00 | 0.14 | detected |
| 7600X | `1t-multi-1seg` | smt | 5.038 | 5.342 | +6.05 | 0.26 | detected |
| 7600X | `1t-multi-1seg` | unpinned | 5.041 | 5.340 | +5.93 | 0.08 | detected |
| 7600X | `1t-single` | smt | 4.015 | 4.335 | +7.96 | 0.19 | detected |
| 7600X | `1t-single` | unpinned | 4.019 | 4.337 | +7.90 | 0.07 | detected |
| 7600X | `2t-multi-2seg` | smt | 48.778 | 50.219 | +2.95 | 0.03 | detected |
| 7600X | `2t-multi-2seg` | ccx | 72.684 | 73.561 | +1.21 | 0.27 | detected |
| 7600X | `2t-multi-2seg` | unpinned | 73.283 | 74.090 | +1.10 | 0.32 | detected |
| 7600X | `2t-multi-1seg` | smt | 48.135 | 50.156 | +4.20 | 0.08 | detected |
| 7600X | `2t-multi-1seg` | ccx | 72.635 | 73.344 | +0.98 | 0.38 | detected |
| 7600X | `2t-multi-1seg` | unpinned | 73.250 | 74.045 | +1.09 | 0.49 | detected |
| 7600X | `2t-single` | smt | 45.906 | 48.984 | +6.71 | 0.21 | detected |
| 7600X | `2t-single` | ccx | 70.747 | 75.643 | +6.92 | 0.24 | detected |
| 7600X | `2t-single` | unpinned | 70.996 | 76.083 | +7.16 | 0.24 | detected |
| Pi 5 | `1t-multi-2seg` | ccx | 28.655 | 30.137 | +5.17 | 0.58 | detected |
| Pi 5 | `1t-multi-2seg` | unpinned | 28.553 | 30.171 | +5.66 | 0.60 | detected |
| Pi 5 | `1t-multi-1seg` | ccx | 28.608 | 30.187 | +5.52 | 0.65 | detected |
| Pi 5 | `1t-multi-1seg` | unpinned | 28.551 | 30.154 | +5.61 | 0.28 | detected |
| Pi 5 | `1t-single` | ccx | 28.508 | 29.696 | +4.17 | 0.64 | detected |
| Pi 5 | `1t-single` | unpinned | 28.492 | 29.656 | +4.09 | 0.43 | detected |
| Pi 5 | `2t-multi-2seg` | ccx | 246.175 | 249.116 | +1.19 | 0.28 | detected |
| Pi 5 | `2t-multi-2seg` | unpinned | 246.117 | 248.509 | +0.97 | 0.15 | detected |
| Pi 5 | `2t-multi-1seg` | ccx | 245.938 | 248.772 | +1.15 | 0.25 | detected |
| Pi 5 | `2t-multi-1seg` | unpinned | 245.594 | 248.009 | +0.98 | 0.19 | detected |
| Pi 5 | `2t-single` | ccx | 241.878 | 244.704 | +1.17 | 0.25 | detected |
| Pi 5 | `2t-single` | unpinned | 241.252 | 245.177 | +1.63 | 0.24 | detected |

- `Futex` is the slower in 45 of 48 rows, the 3900X's three unpinned two-thread rows not seen.
- At one thread: 0.8 to 1.1 ns on the 3900X, 10 to 18%, 0.3 ns on the 7600X, 6 to 8%, and 1.2 to
  1.6 ns on the Pi 5, 4 to 6%.
- At two threads on the `smt` pair: 5.3 to 5.6 ns on the 3900X, 7.9 to 8.7%, and 1.4 to 3.1 ns on
  the 7600X, 3.0 to 6.7%.
- Across cores: 0.4 to 2.8% on the 3900X, 1.0 to 1.2% for `Multi` and 6.9 to 7.2% for `Single` on
  the 7600X, and 1.0 to 1.6% on the Pi 5.
- The 7600X's `2t-single` at `smt` reads 45.91 against 48.98 ns, 3.08 ns and 6.7%, which is the
  reading of [Since the session](#since-the-session), 3.09 ns, now a row.

