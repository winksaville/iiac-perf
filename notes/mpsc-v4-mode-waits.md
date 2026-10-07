# What mpsc v4's mode and waits cost

zc-ring-x1's `mpsc::v4` ring takes two type parameters, a segment mode, `Single` or `Multi`, and
how its endpoints wait, `SpinOnly`, `Sleep<Futex>`, or `SpinOrSleep<Futex>`. This file holds what
the fifteen `zcr-mpsc-v4` benches measured of them on the three measuring hosts, beside v3's twelve
from the same binary, from `feat: mpsc v4 benches` (2026-10-06). v3's own tables from the session
are in [mpsc-v3-mode-wake.md](mpsc-v3-mode-wake.md#the-measuring-hosts).

## The session

- Source: commit `df0c1e72`, jj change `qyotptrq`, a clean tree, `feat: the mpsc v4 benches whose
  receivers wait`, against zc-ring-x1 `827e166`, its 0.19.3.
- Binaries: the 3900X and the 7600X ran one, `ea291f9f3f70a91c`, built on the 3900X, and the Pi 5
  its own aarch64 build of the same commit, `0134b16b0b408371`. All three hosts have rustc 1.98.1.
- Build: the release profile at `opt-level` 3 with one codegen unit, no LTO, and functions and
  branch targets aligned to 64 bytes ([build.md](build.md)). zc-ring-x1 read its wake checks at 8%
  at most under one codegen unit with fat LTO and 28% under its default profile (`m-8-7`), so a
  percent here is this build's.
- Hosts and the placements each declares:
  - 3900X, the clock pinned at 3801 MHz: `smt`, CPUs 11 and 23, `ccx`, 11 and 10, `x-ccx`, 11 and
    8, `x-ccd`, 11 and 5, and unpinned.
  - 7600X, at 4701 MHz: `smt`, CPUs 5 and 11, `ccx`, 5 and 4, and unpinned.
  - Pi 5, at 2400 MHz: `ccx`, CPUs 3 and 2, and unpinned. It has no SMT.
  - A one-thread bench runs at a host's first placement and unpinned.
- Benches: `zcr-mpsc-v2-1t` and `-2t`, the twelve `zcr-mpsc-v3`, and the fifteen `zcr-mpsc-v4`, each
  a round trip with one message in flight.
- Run: five invocations of the 29, ten runs each, at every placement, the three hosts starting
  within two minutes of 2026-10-07T01:42Z, the Pi 5 done in 1.6 hours, the 7600X in 2.1, and the
  3900X in 2.9.
- Records, every segment-switch count in them zero:
  - [records/mpsc-v4-3900x.jsonl](../records/mpsc-v4-3900x.jsonl), 5300 runs.
  - [records/mpsc-v4-7600x.jsonl](../records/mpsc-v4-7600x.jsonl), 3700 runs.
  - [records/mpsc-v4-rpi5-20cd.jsonl](../records/mpsc-v4-rpi5-20cd.jsonl), 2900 runs.
- Numbers are `analyze --by placement --compare`'s: each side's trimmed mean in ns, the difference
  as a percent of the first side's, and the claim, the smallest difference the pairing could call
  real. Every pairing is by series, five a side.

## The names

```
zcr-mpsc-v4-<threads>-<mode>-st<spin>-wt<wait>[-<wait choice>]
```

- `st` and `wt` are the receiver's: how long it spins at an empty ring, and how long it then
  sleeps. `stfe` spins forever, and `wtnone` never reaches a sleep.
- The last field is the ring's: nothing for `SpinOnly`, `-sleep-futex` for `Sleep<Futex>`, and
  `-spinorsleep-futex` for `SpinOrSleep<Futex>`. v3's is `-futex`, its ring over a wake.
- The twelve twins are v3's shapes, `1t` and `2t` by `multi-2seg`, `multi-1seg`, and `single`, each
  `-stfe-wtnone` over `SpinOnly` and over `SpinOrSleep<Futex>`. The second is a ring nobody
  sleeps on that pays for being able to.
- A table's row names its two benches as one name, shortened three ways:
  - `*` stands where the two differ, and the two columns of means are headed by what stands
    there in each.
  - `..` stands for what a row leaves off, the names' `zcr-mpsc-v4-`, which every bench here
    begins with.
  - `sos` is `spinorsleep`, so `-sos-futex` is a name's `-spinorsleep-futex`.
  - So the row `..1t-*-stfe-wtnone-sos-futex` under `multi-2seg` and `multi-1seg` is
    `zcr-mpsc-v4-1t-multi-2seg-stfe-wtnone-spinorsleep-futex` against
    `zcr-mpsc-v4-1t-multi-1seg-stfe-wtnone-spinorsleep-futex`.
- The twins wait by `send_spin` and `recv_spin` with `Ticks::FOREVER`, where v3's benches pass
  `policy::spin` to `send` and `reserve_slot_with`.
- The three whose receivers wait are `2t-single`:
  - `st1us-wtnone`: `recv_spin` for 1 us over `SpinOnly`, tried again when it gives up.
  - `st1us-wtfe-sleep-futex`: `recv_spin_sleep`, a 1 us spin and then a sleep until woken.
  - `st0-wtfe-sleep-futex`: `recv_spin_sleep` with no spin, a sleep at every empty look.

## v4 against v3

Each v3 bench against its v4 twin, `NoWake` against `SpinOnly` and `Futex` against
`SpinOrSleep<Futex>`. Here `..` is `zcr-mpsc-` and the first `*` the version, `v3` or `v4`. A
row with a second `*` is the pair whose rings can sleep, where it stands for `futex` in v3's
name and `spinorsleep-futex` in v4's: `zcr-mpsc-v3-1t-single-stfe-wtnone-futex` against
`zcr-mpsc-v4-1t-single-stfe-wtnone-spinorsleep-futex`.

| host | bench | placement | v3 | v4 | d% | claim% | verdict |
|---|---|---|---|---|---|---|---|
| 3900X | `..*-1t-multi-2seg-stfe-wtnone` | smt | 7.774 | 7.753 | -0.28 | 0.18 | detected |
| 3900X | `..*-1t-multi-2seg-stfe-wtnone` | unpinned | 7.860 | 7.823 | -0.47 | 0.33 | detected |
| 3900X | `..*-1t-multi-1seg-stfe-wtnone` | smt | 7.775 | 7.742 | -0.43 | 0.49 | not seen |
| 3900X | `..*-1t-multi-1seg-stfe-wtnone` | unpinned | 7.849 | 7.826 | -0.30 | 0.25 | detected |
| 3900X | `..*-1t-single-stfe-wtnone` | smt | 6.379 | 6.379 | +0.01 | 0.20 | not seen |
| 3900X | `..*-1t-single-stfe-wtnone` | unpinned | 6.444 | 6.439 | -0.08 | 0.40 | not seen |
| 3900X | `..*-2t-multi-2seg-stfe-wtnone` | smt | 66.366 | 66.296 | -0.10 | 0.18 | not seen |
| 3900X | `..*-2t-multi-2seg-stfe-wtnone` | ccx | 123.074 | 123.857 | +0.64 | 0.28 | detected |
| 3900X | `..*-2t-multi-2seg-stfe-wtnone` | x-ccx | 394.369 | 394.337 | -0.01 | 0.52 | not seen |
| 3900X | `..*-2t-multi-2seg-stfe-wtnone` | x-ccd | 389.443 | 388.777 | -0.17 | 0.16 | detected |
| 3900X | `..*-2t-multi-2seg-stfe-wtnone` | unpinned | 128.246 | 125.574 | -2.08 | 5.57 | not seen |
| 3900X | `..*-2t-multi-1seg-stfe-wtnone` | smt | 66.404 | 66.391 | -0.02 | 0.17 | not seen |
| 3900X | `..*-2t-multi-1seg-stfe-wtnone` | ccx | 127.346 | 127.503 | +0.12 | 0.26 | not seen |
| 3900X | `..*-2t-multi-1seg-stfe-wtnone` | x-ccx | 413.931 | 413.700 | -0.06 | 0.26 | not seen |
| 3900X | `..*-2t-multi-1seg-stfe-wtnone` | x-ccd | 404.627 | 405.577 | +0.23 | 0.43 | not seen |
| 3900X | `..*-2t-multi-1seg-stfe-wtnone` | unpinned | 130.041 | 136.255 | +4.78 | 5.21 | not seen |
| 3900X | `..*-2t-single-stfe-wtnone` | smt | 64.100 | 64.019 | -0.13 | 0.16 | not seen |
| 3900X | `..*-2t-single-stfe-wtnone` | ccx | 125.381 | 124.804 | -0.46 | 0.10 | detected |
| 3900X | `..*-2t-single-stfe-wtnone` | x-ccx | 385.963 | 385.756 | -0.05 | 0.21 | not seen |
| 3900X | `..*-2t-single-stfe-wtnone` | x-ccd | 376.603 | 374.468 | -0.57 | 0.35 | detected |
| 3900X | `..*-2t-single-stfe-wtnone` | unpinned | 137.247 | 127.708 | -6.95 | 13.83 | not seen |
| 3900X | `..*-1t-multi-2seg-stfe-wtnone-*` | smt | 8.598 | 8.622 | +0.28 | 0.35 | not seen |
| 3900X | `..*-1t-multi-2seg-stfe-wtnone-*` | unpinned | 8.675 | 8.714 | +0.45 | 0.42 | detected |
| 3900X | `..*-1t-multi-1seg-stfe-wtnone-*` | smt | 8.602 | 8.618 | +0.18 | 0.20 | not seen |
| 3900X | `..*-1t-multi-1seg-stfe-wtnone-*` | unpinned | 8.674 | 8.717 | +0.50 | 0.36 | detected |
| 3900X | `..*-1t-single-stfe-wtnone-*` | smt | 7.516 | 7.497 | -0.24 | 0.11 | detected |
| 3900X | `..*-1t-single-stfe-wtnone-*` | unpinned | 7.579 | 7.579 | 0.00 | 0.41 | not seen |
| 3900X | `..*-2t-multi-2seg-stfe-wtnone-*` | smt | 71.673 | 71.860 | +0.26 | 0.14 | detected |
| 3900X | `..*-2t-multi-2seg-stfe-wtnone-*` | ccx | 124.111 | 127.018 | +2.34 | 0.13 | detected |
| 3900X | `..*-2t-multi-2seg-stfe-wtnone-*` | x-ccx | 397.243 | 397.213 | -0.01 | 0.19 | not seen |
| 3900X | `..*-2t-multi-2seg-stfe-wtnone-*` | x-ccd | 391.054 | 389.698 | -0.35 | 0.17 | detected |
| 3900X | `..*-2t-multi-2seg-stfe-wtnone-*` | unpinned | 130.919 | 129.252 | -1.27 | 2.17 | not seen |
| 3900X | `..*-2t-multi-1seg-stfe-wtnone-*` | smt | 71.662 | 71.860 | +0.28 | 0.07 | detected |
| 3900X | `..*-2t-multi-1seg-stfe-wtnone-*` | ccx | 130.961 | 131.486 | +0.40 | 0.31 | detected |
| 3900X | `..*-2t-multi-1seg-stfe-wtnone-*` | x-ccx | 418.804 | 418.277 | -0.13 | 0.36 | not seen |
| 3900X | `..*-2t-multi-1seg-stfe-wtnone-*` | x-ccd | 411.226 | 409.686 | -0.37 | 0.34 | detected |
| 3900X | `..*-2t-multi-1seg-stfe-wtnone-*` | unpinned | 136.291 | 136.961 | +0.49 | 6.37 | not seen |
| 3900X | `..*-2t-single-stfe-wtnone-*` | smt | 69.650 | 69.218 | -0.62 | 0.08 | detected |
| 3900X | `..*-2t-single-stfe-wtnone-*` | ccx | 126.131 | 124.316 | -1.44 | 0.35 | detected |
| 3900X | `..*-2t-single-stfe-wtnone-*` | x-ccx | 395.841 | 394.813 | -0.26 | 0.06 | detected |
| 3900X | `..*-2t-single-stfe-wtnone-*` | x-ccd | 386.265 | 385.967 | -0.08 | 0.27 | not seen |
| 3900X | `..*-2t-single-stfe-wtnone-*` | unpinned | 130.542 | 128.486 | -1.57 | 0.62 | detected |
| 7600X | `..*-1t-multi-2seg-stfe-wtnone` | smt | 5.035 | 5.080 | +0.90 | 0.17 | detected |
| 7600X | `..*-1t-multi-2seg-stfe-wtnone` | unpinned | 5.040 | 5.086 | +0.91 | 0.04 | detected |
| 7600X | `..*-1t-multi-1seg-stfe-wtnone` | smt | 5.038 | 5.084 | +0.93 | 0.25 | detected |
| 7600X | `..*-1t-multi-1seg-stfe-wtnone` | unpinned | 5.041 | 5.085 | +0.87 | 0.12 | detected |
| 7600X | `..*-1t-single-stfe-wtnone` | smt | 4.015 | 4.002 | -0.32 | 0.22 | detected |
| 7600X | `..*-1t-single-stfe-wtnone` | unpinned | 4.019 | 4.003 | -0.39 | 0.25 | detected |
| 7600X | `..*-2t-multi-2seg-stfe-wtnone` | smt | 48.778 | 50.215 | +2.95 | 0.10 | detected |
| 7600X | `..*-2t-multi-2seg-stfe-wtnone` | ccx | 72.684 | 72.566 | -0.16 | 0.33 | not seen |
| 7600X | `..*-2t-multi-2seg-stfe-wtnone` | unpinned | 73.283 | 72.990 | -0.40 | 0.58 | not seen |
| 7600X | `..*-2t-multi-1seg-stfe-wtnone` | smt | 48.135 | 49.676 | +3.20 | 0.07 | detected |
| 7600X | `..*-2t-multi-1seg-stfe-wtnone` | ccx | 72.635 | 72.390 | -0.34 | 0.48 | not seen |
| 7600X | `..*-2t-multi-1seg-stfe-wtnone` | unpinned | 73.250 | 73.012 | -0.32 | 0.49 | not seen |
| 7600X | `..*-2t-single-stfe-wtnone` | smt | 45.906 | 45.923 | +0.04 | 0.18 | not seen |
| 7600X | `..*-2t-single-stfe-wtnone` | ccx | 70.747 | 70.745 | 0.00 | 0.18 | not seen |
| 7600X | `..*-2t-single-stfe-wtnone` | unpinned | 70.996 | 71.037 | +0.06 | 0.45 | not seen |
| 7600X | `..*-1t-multi-2seg-stfe-wtnone-*` | smt | 5.339 | 5.337 | -0.02 | 0.16 | not seen |
| 7600X | `..*-1t-multi-2seg-stfe-wtnone-*` | unpinned | 5.342 | 5.338 | -0.08 | 0.14 | not seen |
| 7600X | `..*-1t-multi-1seg-stfe-wtnone-*` | smt | 5.342 | 5.337 | -0.09 | 0.19 | not seen |
| 7600X | `..*-1t-multi-1seg-stfe-wtnone-*` | unpinned | 5.340 | 5.340 | -0.01 | 0.08 | not seen |
| 7600X | `..*-1t-single-stfe-wtnone-*` | smt | 4.335 | 4.331 | -0.09 | 0.21 | not seen |
| 7600X | `..*-1t-single-stfe-wtnone-*` | unpinned | 4.337 | 4.329 | -0.17 | 0.06 | detected |
| 7600X | `..*-2t-multi-2seg-stfe-wtnone-*` | smt | 50.219 | 50.936 | +1.43 | 0.04 | detected |
| 7600X | `..*-2t-multi-2seg-stfe-wtnone-*` | ccx | 73.561 | 73.594 | +0.05 | 0.19 | not seen |
| 7600X | `..*-2t-multi-2seg-stfe-wtnone-*` | unpinned | 74.090 | 74.173 | +0.11 | 0.20 | not seen |
| 7600X | `..*-2t-multi-1seg-stfe-wtnone-*` | smt | 50.156 | 50.864 | +1.41 | 0.04 | detected |
| 7600X | `..*-2t-multi-1seg-stfe-wtnone-*` | ccx | 73.344 | 73.438 | +0.13 | 0.07 | detected |
| 7600X | `..*-2t-multi-1seg-stfe-wtnone-*` | unpinned | 74.045 | 74.127 | +0.11 | 0.09 | detected |
| 7600X | `..*-2t-single-stfe-wtnone-*` | smt | 48.984 | 48.820 | -0.33 | 0.40 | not seen |
| 7600X | `..*-2t-single-stfe-wtnone-*` | ccx | 75.643 | 75.463 | -0.24 | 0.10 | detected |
| 7600X | `..*-2t-single-stfe-wtnone-*` | unpinned | 76.083 | 75.858 | -0.30 | 0.08 | detected |
| Pi 5 | `..*-1t-multi-2seg-stfe-wtnone` | ccx | 28.655 | 28.608 | -0.17 | 0.25 | not seen |
| Pi 5 | `..*-1t-multi-2seg-stfe-wtnone` | unpinned | 28.553 | 28.557 | +0.01 | 0.30 | not seen |
| Pi 5 | `..*-1t-multi-1seg-stfe-wtnone` | ccx | 28.608 | 28.641 | +0.12 | 0.21 | not seen |
| Pi 5 | `..*-1t-multi-1seg-stfe-wtnone` | unpinned | 28.551 | 28.613 | +0.22 | 0.58 | not seen |
| Pi 5 | `..*-1t-single-stfe-wtnone` | ccx | 28.508 | 28.464 | -0.15 | 0.13 | detected |
| Pi 5 | `..*-1t-single-stfe-wtnone` | unpinned | 28.492 | 28.491 | 0.00 | 0.21 | not seen |
| Pi 5 | `..*-2t-multi-2seg-stfe-wtnone` | ccx | 246.175 | 246.517 | +0.14 | 0.13 | detected |
| Pi 5 | `..*-2t-multi-2seg-stfe-wtnone` | unpinned | 246.117 | 246.365 | +0.10 | 0.32 | not seen |
| Pi 5 | `..*-2t-multi-1seg-stfe-wtnone` | ccx | 245.938 | 246.007 | +0.03 | 0.14 | not seen |
| Pi 5 | `..*-2t-multi-1seg-stfe-wtnone` | unpinned | 245.594 | 245.924 | +0.13 | 0.34 | not seen |
| Pi 5 | `..*-2t-single-stfe-wtnone` | ccx | 241.878 | 241.867 | 0.00 | 0.08 | not seen |
| Pi 5 | `..*-2t-single-stfe-wtnone` | unpinned | 241.252 | 241.428 | +0.07 | 0.14 | not seen |
| Pi 5 | `..*-1t-multi-2seg-stfe-wtnone-*` | ccx | 30.137 | 30.198 | +0.20 | 0.54 | not seen |
| Pi 5 | `..*-1t-multi-2seg-stfe-wtnone-*` | unpinned | 30.171 | 30.124 | -0.15 | 0.47 | not seen |
| Pi 5 | `..*-1t-multi-1seg-stfe-wtnone-*` | ccx | 30.187 | 30.118 | -0.23 | 0.27 | not seen |
| Pi 5 | `..*-1t-multi-1seg-stfe-wtnone-*` | unpinned | 30.154 | 30.156 | +0.01 | 0.35 | not seen |
| Pi 5 | `..*-1t-single-stfe-wtnone-*` | ccx | 29.696 | 29.783 | +0.29 | 0.74 | not seen |
| Pi 5 | `..*-1t-single-stfe-wtnone-*` | unpinned | 29.656 | 29.671 | +0.05 | 0.69 | not seen |
| Pi 5 | `..*-2t-multi-2seg-stfe-wtnone-*` | ccx | 249.116 | 249.056 | -0.02 | 0.13 | not seen |
| Pi 5 | `..*-2t-multi-2seg-stfe-wtnone-*` | unpinned | 248.509 | 249.162 | +0.26 | 0.25 | detected |
| Pi 5 | `..*-2t-multi-1seg-stfe-wtnone-*` | ccx | 248.772 | 248.650 | -0.05 | 0.29 | not seen |
| Pi 5 | `..*-2t-multi-1seg-stfe-wtnone-*` | unpinned | 248.009 | 248.606 | +0.24 | 0.05 | detected |
| Pi 5 | `..*-2t-single-stfe-wtnone-*` | ccx | 244.704 | 244.851 | +0.06 | 0.43 | not seen |
| Pi 5 | `..*-2t-single-stfe-wtnone-*` | unpinned | 245.177 | 244.678 | -0.20 | 0.15 | detected |

- On the Pi 5 the two read alike: 19 of 24 rows not seen, and the five detected are 0.26% at most
  and of both signs.
- On the 3900X no difference holds: 22 of 42 not seen, 13 with v4 the faster and 7 the slower, and
  all but three of the detected within 0.65%.
- On the 7600X `Single` reads alike, within 0.4%, and `Multi` is slower in v4 at two threads on
  the `smt` pair: 1.4 and 1.5 ns over `SpinOnly`, 3%, and 0.7 ns over `SpinOrSleep<Futex>`, 1.4%.
  Its two-thread `ccx` and unpinned rows are within 0.4%, and at one thread `Multi` over
  `SpinOnly` is 0.05 ns slower, 0.9%.
  - The 3900X ran the same binary and its `smt` rows show none of it. We think it is this build's
    layout meeting that CPU and not v4's code, and one build cannot say.
- zc-ring-x1's one unexplained row, `mpsc-v4-backoff` on the Pi 5 at one producer, 20% behind its
  v3 twin (`m-8-7`), has nothing beside it here: our benches have no backoff flavor, and the Pi
  5's v4 rows are v3's within 0.3%.

## Multi over one segment against two

`multi-2seg` against `multi-1seg`, in v4, over `SpinOnly` and, in the rows whose names end
`-sos-futex`, over `SpinOrSleep<Futex>`. v3's rows read the same
([mpsc-v3-mode-wake.md](mpsc-v3-mode-wake.md#the-measuring-hosts)).

| host | bench | placement | `multi-2seg` | `multi-1seg` | d% | claim% | verdict |
|---|---|---|---|---|---|---|---|
| 3900X | `..1t-*-stfe-wtnone` | smt | 7.753 | 7.742 | -0.14 | 0.37 | not seen |
| 3900X | `..1t-*-stfe-wtnone` | unpinned | 7.823 | 7.826 | +0.04 | 0.34 | not seen |
| 3900X | `..1t-*-stfe-wtnone-sos-futex` | smt | 8.622 | 8.618 | -0.05 | 0.30 | not seen |
| 3900X | `..1t-*-stfe-wtnone-sos-futex` | unpinned | 8.714 | 8.717 | +0.03 | 0.36 | not seen |
| 3900X | `..2t-*-stfe-wtnone` | smt | 66.296 | 66.391 | +0.14 | 0.20 | not seen |
| 3900X | `..2t-*-stfe-wtnone` | ccx | 123.857 | 127.503 | +2.94 | 0.18 | detected |
| 3900X | `..2t-*-stfe-wtnone` | x-ccx | 394.337 | 413.700 | +4.91 | 0.27 | detected |
| 3900X | `..2t-*-stfe-wtnone` | x-ccd | 388.777 | 405.577 | +4.32 | 0.34 | detected |
| 3900X | `..2t-*-stfe-wtnone` | unpinned | 125.574 | 136.255 | +8.51 | 5.85 | detected |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | smt | 71.860 | 71.860 | 0.00 | 0.08 | not seen |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | ccx | 127.018 | 131.486 | +3.52 | 0.22 | detected |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | x-ccx | 397.213 | 418.277 | +5.30 | 0.39 | detected |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | x-ccd | 389.698 | 409.686 | +5.13 | 0.14 | detected |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | unpinned | 129.252 | 136.961 | +5.96 | 3.77 | detected |
| 7600X | `..1t-*-stfe-wtnone` | smt | 5.080 | 5.084 | +0.09 | 0.17 | not seen |
| 7600X | `..1t-*-stfe-wtnone` | unpinned | 5.086 | 5.085 | -0.02 | 0.08 | not seen |
| 7600X | `..1t-*-stfe-wtnone-sos-futex` | smt | 5.337 | 5.337 | 0.00 | 0.11 | not seen |
| 7600X | `..1t-*-stfe-wtnone-sos-futex` | unpinned | 5.338 | 5.340 | +0.03 | 0.07 | not seen |
| 7600X | `..2t-*-stfe-wtnone` | smt | 50.215 | 49.676 | -1.07 | 0.05 | detected |
| 7600X | `..2t-*-stfe-wtnone` | ccx | 72.566 | 72.390 | -0.24 | 0.30 | not seen |
| 7600X | `..2t-*-stfe-wtnone` | unpinned | 72.990 | 73.012 | +0.03 | 0.22 | not seen |
| 7600X | `..2t-*-stfe-wtnone-sos-futex` | smt | 50.936 | 50.864 | -0.14 | 0.05 | detected |
| 7600X | `..2t-*-stfe-wtnone-sos-futex` | ccx | 73.594 | 73.438 | -0.21 | 0.10 | detected |
| 7600X | `..2t-*-stfe-wtnone-sos-futex` | unpinned | 74.173 | 74.127 | -0.06 | 0.20 | not seen |
| Pi 5 | `..1t-*-stfe-wtnone` | ccx | 28.608 | 28.641 | +0.12 | 0.61 | not seen |
| Pi 5 | `..1t-*-stfe-wtnone` | unpinned | 28.557 | 28.613 | +0.19 | 0.42 | not seen |
| Pi 5 | `..1t-*-stfe-wtnone-sos-futex` | ccx | 30.198 | 30.118 | -0.27 | 0.32 | not seen |
| Pi 5 | `..1t-*-stfe-wtnone-sos-futex` | unpinned | 30.124 | 30.156 | +0.11 | 0.54 | not seen |
| Pi 5 | `..2t-*-stfe-wtnone` | ccx | 246.517 | 246.007 | -0.21 | 0.18 | detected |
| Pi 5 | `..2t-*-stfe-wtnone` | unpinned | 246.365 | 245.924 | -0.18 | 0.12 | detected |
| Pi 5 | `..2t-*-stfe-wtnone-sos-futex` | ccx | 249.056 | 248.650 | -0.16 | 0.38 | not seen |
| Pi 5 | `..2t-*-stfe-wtnone-sos-futex` | unpinned | 249.162 | 248.606 | -0.22 | 0.25 | not seen |

- At one thread nothing is seen on any host.
- On the 3900X across cores one segment is the slower, 3 to 5%: 4 ns at `ccx` and 17 to 21 ns at
  `x-ccx` and `x-ccd`, in v3 and in v4, over both wait choices. Its `smt` rows show nothing.
- On the 7600X and the Pi 5 one segment reads level or a little faster, 1.1% at most.
- We do not know why the 3900X differs. No switch happens in either, so we think it is where the
  ring's shared words fall for each geometry and not a segment's cost.
- So `Single` is compared below with `Multi` over one segment, the geometry equal, and over two,
  what a `Multi` ring would be used at.

## Single against Multi

The mode's cost and benefit, which is what the benches are for (wink, 2026-10-06).

Against `Multi` over one segment:

| host | bench | placement | `multi-1seg` | `single` | d% | claim% | verdict |
|---|---|---|---|---|---|---|---|
| 3900X | `..1t-*-stfe-wtnone` | smt | 7.742 | 6.379 | -17.60 | 0.39 | detected |
| 3900X | `..1t-*-stfe-wtnone` | unpinned | 7.826 | 6.439 | -17.72 | 0.24 | detected |
| 3900X | `..1t-*-stfe-wtnone-sos-futex` | smt | 8.618 | 7.497 | -13.00 | 0.17 | detected |
| 3900X | `..1t-*-stfe-wtnone-sos-futex` | unpinned | 8.717 | 7.579 | -13.05 | 0.47 | detected |
| 3900X | `..2t-*-stfe-wtnone` | smt | 66.391 | 64.019 | -3.57 | 0.22 | detected |
| 3900X | `..2t-*-stfe-wtnone` | ccx | 127.503 | 124.804 | -2.12 | 0.19 | detected |
| 3900X | `..2t-*-stfe-wtnone` | x-ccx | 413.700 | 385.756 | -6.75 | 0.29 | detected |
| 3900X | `..2t-*-stfe-wtnone` | x-ccd | 405.577 | 374.468 | -7.67 | 0.41 | detected |
| 3900X | `..2t-*-stfe-wtnone` | unpinned | 136.255 | 127.708 | -6.27 | 8.23 | not seen |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | smt | 71.860 | 69.218 | -3.68 | 0.07 | detected |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | ccx | 131.486 | 124.316 | -5.45 | 0.35 | detected |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | x-ccx | 418.277 | 394.813 | -5.61 | 0.33 | detected |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | x-ccd | 409.686 | 385.967 | -5.79 | 0.17 | detected |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | unpinned | 136.961 | 128.486 | -6.19 | 3.36 | detected |
| 7600X | `..1t-*-stfe-wtnone` | smt | 5.084 | 4.002 | -21.28 | 0.16 | detected |
| 7600X | `..1t-*-stfe-wtnone` | unpinned | 5.085 | 4.003 | -21.27 | 0.22 | detected |
| 7600X | `..1t-*-stfe-wtnone-sos-futex` | smt | 5.337 | 4.331 | -18.85 | 0.14 | detected |
| 7600X | `..1t-*-stfe-wtnone-sos-futex` | unpinned | 5.340 | 4.329 | -18.93 | 0.10 | detected |
| 7600X | `..2t-*-stfe-wtnone` | smt | 49.676 | 45.923 | -7.55 | 0.08 | detected |
| 7600X | `..2t-*-stfe-wtnone` | ccx | 72.390 | 70.745 | -2.27 | 0.55 | detected |
| 7600X | `..2t-*-stfe-wtnone` | unpinned | 73.012 | 71.037 | -2.70 | 0.44 | detected |
| 7600X | `..2t-*-stfe-wtnone-sos-futex` | smt | 50.864 | 48.820 | -4.02 | 0.35 | detected |
| 7600X | `..2t-*-stfe-wtnone-sos-futex` | ccx | 73.438 | 75.463 | +2.76 | 0.10 | detected |
| 7600X | `..2t-*-stfe-wtnone-sos-futex` | unpinned | 74.127 | 75.858 | +2.33 | 0.05 | detected |
| Pi 5 | `..1t-*-stfe-wtnone` | ccx | 28.641 | 28.464 | -0.62 | 0.56 | detected |
| Pi 5 | `..1t-*-stfe-wtnone` | unpinned | 28.613 | 28.491 | -0.43 | 0.43 | not seen |
| Pi 5 | `..1t-*-stfe-wtnone-sos-futex` | ccx | 30.118 | 29.783 | -1.11 | 0.54 | detected |
| Pi 5 | `..1t-*-stfe-wtnone-sos-futex` | unpinned | 30.156 | 29.671 | -1.61 | 0.58 | detected |
| Pi 5 | `..2t-*-stfe-wtnone` | ccx | 246.007 | 241.867 | -1.68 | 0.15 | detected |
| Pi 5 | `..2t-*-stfe-wtnone` | unpinned | 245.924 | 241.428 | -1.83 | 0.13 | detected |
| Pi 5 | `..2t-*-stfe-wtnone-sos-futex` | ccx | 248.650 | 244.851 | -1.53 | 0.52 | detected |
| Pi 5 | `..2t-*-stfe-wtnone-sos-futex` | unpinned | 248.606 | 244.678 | -1.58 | 0.06 | detected |

Against `Multi` over two:

| host | bench | placement | `multi-2seg` | `single` | d% | claim% | verdict |
|---|---|---|---|---|---|---|---|
| 3900X | `..1t-*-stfe-wtnone` | smt | 7.753 | 6.379 | -17.71 | 0.15 | detected |
| 3900X | `..1t-*-stfe-wtnone` | unpinned | 7.823 | 6.439 | -17.69 | 0.56 | detected |
| 3900X | `..1t-*-stfe-wtnone-sos-futex` | smt | 8.622 | 7.497 | -13.04 | 0.20 | detected |
| 3900X | `..1t-*-stfe-wtnone-sos-futex` | unpinned | 8.714 | 7.579 | -13.02 | 0.30 | detected |
| 3900X | `..2t-*-stfe-wtnone` | smt | 66.296 | 64.019 | -3.43 | 0.13 | detected |
| 3900X | `..2t-*-stfe-wtnone` | ccx | 123.857 | 124.804 | +0.76 | 0.14 | detected |
| 3900X | `..2t-*-stfe-wtnone` | x-ccx | 394.337 | 385.756 | -2.18 | 0.35 | detected |
| 3900X | `..2t-*-stfe-wtnone` | x-ccd | 388.777 | 374.468 | -3.68 | 0.26 | detected |
| 3900X | `..2t-*-stfe-wtnone` | unpinned | 125.574 | 127.708 | +1.70 | 4.38 | not seen |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | smt | 71.860 | 69.218 | -3.68 | 0.05 | detected |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | ccx | 127.018 | 124.316 | -2.13 | 0.17 | detected |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | x-ccx | 397.213 | 394.813 | -0.60 | 0.30 | detected |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | x-ccd | 389.698 | 385.967 | -0.96 | 0.10 | detected |
| 3900X | `..2t-*-stfe-wtnone-sos-futex` | unpinned | 129.252 | 128.486 | -0.59 | 0.91 | not seen |
| 7600X | `..1t-*-stfe-wtnone` | smt | 5.080 | 4.002 | -21.21 | 0.15 | detected |
| 7600X | `..1t-*-stfe-wtnone` | unpinned | 5.086 | 4.003 | -21.28 | 0.27 | detected |
| 7600X | `..1t-*-stfe-wtnone-sos-futex` | smt | 5.337 | 4.331 | -18.85 | 0.14 | detected |
| 7600X | `..1t-*-stfe-wtnone-sos-futex` | unpinned | 5.338 | 4.329 | -18.90 | 0.05 | detected |
| 7600X | `..2t-*-stfe-wtnone` | smt | 50.215 | 45.923 | -8.55 | 0.11 | detected |
| 7600X | `..2t-*-stfe-wtnone` | ccx | 72.566 | 70.745 | -2.51 | 0.26 | detected |
| 7600X | `..2t-*-stfe-wtnone` | unpinned | 72.990 | 71.037 | -2.68 | 0.45 | detected |
| 7600X | `..2t-*-stfe-wtnone-sos-futex` | smt | 50.936 | 48.820 | -4.15 | 0.33 | detected |
| 7600X | `..2t-*-stfe-wtnone-sos-futex` | ccx | 73.594 | 75.463 | +2.54 | 0.13 | detected |
| 7600X | `..2t-*-stfe-wtnone-sos-futex` | unpinned | 74.173 | 75.858 | +2.27 | 0.19 | detected |
| Pi 5 | `..1t-*-stfe-wtnone` | ccx | 28.608 | 28.464 | -0.50 | 0.20 | detected |
| Pi 5 | `..1t-*-stfe-wtnone` | unpinned | 28.557 | 28.491 | -0.23 | 0.14 | detected |
| Pi 5 | `..1t-*-stfe-wtnone-sos-futex` | ccx | 30.198 | 29.783 | -1.38 | 0.31 | detected |
| Pi 5 | `..1t-*-stfe-wtnone-sos-futex` | unpinned | 30.124 | 29.671 | -1.50 | 0.51 | detected |
| Pi 5 | `..2t-*-stfe-wtnone` | ccx | 246.517 | 241.867 | -1.89 | 0.15 | detected |
| Pi 5 | `..2t-*-stfe-wtnone` | unpinned | 246.365 | 241.428 | -2.00 | 0.21 | detected |
| Pi 5 | `..2t-*-stfe-wtnone-sos-futex` | ccx | 249.056 | 244.851 | -1.69 | 0.53 | detected |
| Pi 5 | `..2t-*-stfe-wtnone-sos-futex` | unpinned | 249.162 | 244.678 | -1.80 | 0.28 | detected |

- At one thread `Single` is 1.0 to 1.4 ns faster on the x86 hosts, 13 to 21%, and 0.1 to 0.5 ns
  on the Pi 5, 0.2 to 1.6%, one row of its eight not seen.
- At two threads on the `smt` pair `Single` is faster in every row: 2.3 to 2.6 ns on the 3900X,
  3.4 to 3.7%, and 2.0 to 4.3 ns on the 7600X, 4.0 to 8.6%.
- On the Pi 5 at two threads `Single` is 3.8 to 4.9 ns faster in all eight rows, 1.5 to 2.0%.
- Across cores on the x86 hosts it depends on the row:
  - 3900X `x-ccx` and `x-ccd`: `Single` is faster in all eight rows, 5.6 to 7.7% against one
    segment and 0.6 to 3.7% against two.
  - 3900X `ccx`: faster than one segment, 2.1 and 5.5%, and against two it is 0.8% slower over
    `SpinOnly` and 2.1% faster over `SpinOrSleep<Futex>`.
  - 7600X `ccx` and unpinned over `SpinOnly`: `Single` is 1.6 to 2.0 ns faster, 2.3 to 2.7%.
  - 7600X `ccx` and unpinned over `SpinOrSleep<Futex>`: `Single` is 1.7 to 2.0 ns slower, 2.3 to
    2.8%, against either geometry, and v3's rows over `Futex` read the same, +2.7 to +3.1%.
- Of the 64 rows `Single` is detected faster in 55 and slower in 5, with 4 not seen.

## SpinOrSleep<Futex> against SpinOnly

Each twin over `SpinOnly` against the same ring over `SpinOrSleep<Futex>`. Nobody sleeps, so the
difference is the checks left on the message path so that a sleeper could be woken, and not the
cost of a sleep and a wake. The `*` ends the name: nothing there is the ring over `SpinOnly`, the
name as the row gives it without the `*`, and `-sos-futex` the ring that can sleep.

| host | bench | placement | no suffix | `-sos-futex` | d% | claim% | verdict |
|---|---|---|---|---|---|---|---|
| 3900X | `..1t-multi-2seg-stfe-wtnone*` | smt | 7.753 | 8.622 | +11.21 | 0.32 | detected |
| 3900X | `..1t-multi-2seg-stfe-wtnone*` | unpinned | 7.823 | 8.714 | +11.39 | 0.55 | detected |
| 3900X | `..1t-multi-1seg-stfe-wtnone*` | smt | 7.742 | 8.618 | +11.31 | 0.34 | detected |
| 3900X | `..1t-multi-1seg-stfe-wtnone*` | unpinned | 7.826 | 8.717 | +11.39 | 0.48 | detected |
| 3900X | `..1t-single-stfe-wtnone*` | smt | 6.379 | 7.497 | +17.53 | 0.21 | detected |
| 3900X | `..1t-single-stfe-wtnone*` | unpinned | 6.439 | 7.579 | +17.71 | 0.67 | detected |
| 3900X | `..2t-multi-2seg-stfe-wtnone*` | smt | 66.296 | 71.860 | +8.39 | 0.15 | detected |
| 3900X | `..2t-multi-2seg-stfe-wtnone*` | ccx | 123.857 | 127.018 | +2.55 | 0.17 | detected |
| 3900X | `..2t-multi-2seg-stfe-wtnone*` | x-ccx | 394.337 | 397.213 | +0.73 | 0.33 | detected |
| 3900X | `..2t-multi-2seg-stfe-wtnone*` | x-ccd | 388.777 | 389.698 | +0.24 | 0.21 | detected |
| 3900X | `..2t-multi-2seg-stfe-wtnone*` | unpinned | 125.574 | 129.252 | +2.93 | 0.51 | detected |
| 3900X | `..2t-multi-1seg-stfe-wtnone*` | smt | 66.391 | 71.860 | +8.24 | 0.13 | detected |
| 3900X | `..2t-multi-1seg-stfe-wtnone*` | ccx | 127.503 | 131.486 | +3.12 | 0.29 | detected |
| 3900X | `..2t-multi-1seg-stfe-wtnone*` | x-ccx | 413.700 | 418.277 | +1.11 | 0.21 | detected |
| 3900X | `..2t-multi-1seg-stfe-wtnone*` | x-ccd | 405.577 | 409.686 | +1.01 | 0.29 | detected |
| 3900X | `..2t-multi-1seg-stfe-wtnone*` | unpinned | 136.255 | 136.961 | +0.52 | 7.00 | not seen |
| 3900X | `..2t-single-stfe-wtnone*` | smt | 64.019 | 69.218 | +8.12 | 0.11 | detected |
| 3900X | `..2t-single-stfe-wtnone*` | ccx | 124.804 | 124.316 | -0.39 | 0.24 | detected |
| 3900X | `..2t-single-stfe-wtnone*` | x-ccx | 385.756 | 394.813 | +2.35 | 0.18 | detected |
| 3900X | `..2t-single-stfe-wtnone*` | x-ccd | 374.468 | 385.967 | +3.07 | 0.22 | detected |
| 3900X | `..2t-single-stfe-wtnone*` | unpinned | 127.708 | 128.486 | +0.61 | 3.90 | not seen |
| 7600X | `..1t-multi-2seg-stfe-wtnone*` | smt | 5.080 | 5.337 | +5.06 | 0.05 | detected |
| 7600X | `..1t-multi-2seg-stfe-wtnone*` | unpinned | 5.086 | 5.338 | +4.96 | 0.07 | detected |
| 7600X | `..1t-multi-1seg-stfe-wtnone*` | smt | 5.084 | 5.337 | +4.97 | 0.20 | detected |
| 7600X | `..1t-multi-1seg-stfe-wtnone*` | unpinned | 5.085 | 5.340 | +5.01 | 0.08 | detected |
| 7600X | `..1t-single-stfe-wtnone*` | smt | 4.002 | 4.331 | +8.21 | 0.27 | detected |
| 7600X | `..1t-single-stfe-wtnone*` | unpinned | 4.003 | 4.329 | +8.14 | 0.26 | detected |
| 7600X | `..2t-multi-2seg-stfe-wtnone*` | smt | 50.215 | 50.936 | +1.44 | 0.04 | detected |
| 7600X | `..2t-multi-2seg-stfe-wtnone*` | ccx | 72.566 | 73.594 | +1.42 | 0.23 | detected |
| 7600X | `..2t-multi-2seg-stfe-wtnone*` | unpinned | 72.990 | 74.173 | +1.62 | 0.30 | detected |
| 7600X | `..2t-multi-1seg-stfe-wtnone*` | smt | 49.676 | 50.864 | +2.39 | 0.04 | detected |
| 7600X | `..2t-multi-1seg-stfe-wtnone*` | ccx | 72.390 | 73.438 | +1.45 | 0.46 | detected |
| 7600X | `..2t-multi-1seg-stfe-wtnone*` | unpinned | 73.012 | 74.127 | +1.53 | 0.25 | detected |
| 7600X | `..2t-single-stfe-wtnone*` | smt | 45.923 | 48.820 | +6.31 | 0.37 | detected |
| 7600X | `..2t-single-stfe-wtnone*` | ccx | 70.745 | 75.463 | +6.67 | 0.14 | detected |
| 7600X | `..2t-single-stfe-wtnone*` | unpinned | 71.037 | 75.858 | +6.79 | 0.28 | detected |
| Pi 5 | `..1t-multi-2seg-stfe-wtnone*` | ccx | 28.608 | 30.198 | +5.56 | 0.49 | detected |
| Pi 5 | `..1t-multi-2seg-stfe-wtnone*` | unpinned | 28.557 | 30.124 | +5.49 | 0.42 | detected |
| Pi 5 | `..1t-multi-1seg-stfe-wtnone*` | ccx | 28.641 | 30.118 | +5.15 | 0.52 | detected |
| Pi 5 | `..1t-multi-1seg-stfe-wtnone*` | unpinned | 28.613 | 30.156 | +5.39 | 0.51 | detected |
| Pi 5 | `..1t-single-stfe-wtnone*` | ccx | 28.464 | 29.783 | +4.63 | 0.61 | detected |
| Pi 5 | `..1t-single-stfe-wtnone*` | unpinned | 28.491 | 29.671 | +4.14 | 0.43 | detected |
| Pi 5 | `..2t-multi-2seg-stfe-wtnone*` | ccx | 246.517 | 249.056 | +1.03 | 0.27 | detected |
| Pi 5 | `..2t-multi-2seg-stfe-wtnone*` | unpinned | 246.365 | 249.162 | +1.14 | 0.42 | detected |
| Pi 5 | `..2t-multi-1seg-stfe-wtnone*` | ccx | 246.007 | 248.650 | +1.07 | 0.31 | detected |
| Pi 5 | `..2t-multi-1seg-stfe-wtnone*` | unpinned | 245.924 | 248.606 | +1.09 | 0.24 | detected |
| Pi 5 | `..2t-single-stfe-wtnone*` | ccx | 241.867 | 244.851 | +1.23 | 0.40 | detected |
| Pi 5 | `..2t-single-stfe-wtnone*` | unpinned | 241.428 | 244.678 | +1.35 | 0.17 | detected |

- The ring that can sleep is the slower in 45 of 48 rows, with two unpinned rows on the 3900X not
  seen and one row the other way, `2t-single` at the 3900X's `ccx`, 0.4%.
  - That row repeats under another build: one invocation of twenty runs from a later tree, binary
    `4f1a7f7542f94d66`, read 125.54 against 124.05 ns, 1.2% (wink, 2026-10-07). The timed spin is
    2% faster at that placement too, so there more work at an empty look does not cost. We think
    it is when the receiver next touches a line the other core holds, and nothing here shows it.
- At one thread: 0.9 to 1.1 ns on the 3900X, 11 to 18%, 0.25 to 0.33 ns on the 7600X, 5 to 8%,
  and 1.2 to 1.6 ns on the Pi 5, 4 to 6%.
- At two threads on the `smt` pair: 5.2 to 5.6 ns on the 3900X, 8.1 to 8.4%, and 0.7 to 2.9 ns
  on the 7600X, 1.4 to 6.3%.
- Across cores: 0.2 to 3.1% on the 3900X, 1.4 to 1.6% for `Multi` and 6.7 to 6.8% for `Single` on
  the 7600X, 4.7 to 4.8 ns, and 1.0 to 1.4% on the Pi 5, 2.5 to 3.3 ns.
- On the 7600X at two threads `Single` pays the most, 6.3 to 6.8% at every placement against
  `Multi`'s 1.4 to 2.4%, which is why its lead over `Multi` is gone there away from `smt` once
  both rings can sleep.
- v3's `Futex` against `NoWake` reads the same on every host, 45 of 48 slower and three not seen.

## What a wait costs

Each waiting bench against `zcr-mpsc-v4-2t-single-stfe-wtnone`, the same ring spinning without
a limit. A row names its waiting bench whole after the `..`, the `stfe-wtnone` column is the
spin without a limit at that placement, and the next is the bench the row names.

| host | bench | placement | `stfe-wtnone` | the bench | d% | claim% | verdict |
|---|---|---|---|---|---|---|---|
| 3900X | `..2t-single-st1us-wtnone` | smt | 64.019 | 89.038 | +39.08 | 0.11 | detected |
| 3900X | `..2t-single-st1us-wtnone` | ccx | 124.804 | 122.289 | -2.02 | 0.15 | detected |
| 3900X | `..2t-single-st1us-wtnone` | x-ccx | 385.756 | 406.638 | +5.41 | 0.63 | detected |
| 3900X | `..2t-single-st1us-wtnone` | x-ccd | 374.468 | 396.809 | +5.97 | 0.18 | detected |
| 3900X | `..2t-single-st1us-wtnone` | unpinned | 127.708 | 125.483 | -1.74 | 6.91 | not seen |
| 3900X | `..2t-single-st1us-wtfe-sleep-futex` | smt | 64.019 | 2510.190 | +3821.00 | 1904.70 | detected |
| 3900X | `..2t-single-st1us-wtfe-sleep-futex` | ccx | 124.804 | 1183.862 | +848.57 | 308.39 | detected |
| 3900X | `..2t-single-st1us-wtfe-sleep-futex` | x-ccx | 385.756 | 4199.148 | +988.55 | 134.41 | detected |
| 3900X | `..2t-single-st1us-wtfe-sleep-futex` | x-ccd | 374.468 | 3971.185 | +960.49 | 190.13 | detected |
| 3900X | `..2t-single-st1us-wtfe-sleep-futex` | unpinned | 127.708 | 1342.765 | +951.44 | 163.95 | detected |
| 3900X | `..2t-single-st0-wtfe-sleep-futex` | smt | 64.019 | 7895.486 | +12233.01 | 407.73 | detected |
| 3900X | `..2t-single-st0-wtfe-sleep-futex` | ccx | 124.804 | 6638.420 | +5219.07 | 243.28 | detected |
| 3900X | `..2t-single-st0-wtfe-sleep-futex` | x-ccx | 385.756 | 9952.203 | +2479.92 | 18.30 | detected |
| 3900X | `..2t-single-st0-wtfe-sleep-futex` | x-ccd | 374.468 | 9660.384 | +2479.76 | 14.34 | detected |
| 3900X | `..2t-single-st0-wtfe-sleep-futex` | unpinned | 127.708 | 8003.784 | +6167.27 | 295.23 | detected |
| 7600X | `..2t-single-st1us-wtnone` | smt | 45.923 | 58.793 | +28.03 | 0.16 | detected |
| 7600X | `..2t-single-st1us-wtnone` | ccx | 70.745 | 70.051 | -0.98 | 8.42 | not seen |
| 7600X | `..2t-single-st1us-wtnone` | unpinned | 71.037 | 69.515 | -2.14 | 9.04 | not seen |
| 7600X | `..2t-single-st1us-wtfe-sleep-futex` | smt | 45.923 | 193.852 | +322.12 | 35.97 | detected |
| 7600X | `..2t-single-st1us-wtfe-sleep-futex` | ccx | 70.745 | 91.032 | +28.68 | 3.54 | detected |
| 7600X | `..2t-single-st1us-wtfe-sleep-futex` | unpinned | 71.037 | 223.083 | +214.04 | 17.62 | detected |
| 7600X | `..2t-single-st0-wtfe-sleep-futex` | smt | 45.923 | 5903.961 | +12756.17 | 16.06 | detected |
| 7600X | `..2t-single-st0-wtfe-sleep-futex` | ccx | 70.745 | 4343.948 | +6040.27 | 195.81 | detected |
| 7600X | `..2t-single-st0-wtfe-sleep-futex` | unpinned | 71.037 | 4707.771 | +6527.17 | 123.92 | detected |
| Pi 5 | `..2t-single-st1us-wtnone` | ccx | 241.867 | 265.655 | +9.84 | 0.16 | detected |
| Pi 5 | `..2t-single-st1us-wtnone` | unpinned | 241.428 | 265.968 | +10.16 | 0.20 | detected |
| Pi 5 | `..2t-single-st1us-wtfe-sleep-futex` | ccx | 241.867 | 262.772 | +8.64 | 0.26 | detected |
| Pi 5 | `..2t-single-st1us-wtfe-sleep-futex` | unpinned | 241.428 | 263.551 | +9.16 | 0.37 | detected |
| Pi 5 | `..2t-single-st0-wtfe-sleep-futex` | ccx | 241.867 | 4902.470 | +1926.93 | 3.21 | detected |
| Pi 5 | `..2t-single-st0-wtfe-sleep-futex` | unpinned | 241.428 | 4977.942 | +1961.87 | 4.71 | detected |

- A timed spin, `st1us-wtnone`, reads the clock at each empty look:
  - On a shared core it costs most: 25.0 ns on the 3900X's `smt` pair, 39%, and 12.9 ns on the
    7600X's, 28%.
  - On the Pi 5 it costs 23.8 and 24.5 ns, 10%.
  - Across cores on the x86 hosts it is 21 to 22 ns at `x-ccx` and `x-ccd`, 5 to 6%, and at `ccx`
    nothing: the 3900X reads 2% faster, and the 7600X's `ccx` and unpinned rows are not seen
    with claims of 8 and 9%, its invocations there disagreeing.
- A sleep at every empty look, `st0-wtfe-sleep-futex`, puts two sleeps and two wakes on a round
  trip: 6,600 to 10,000 ns on the 3900X, 4,300 to 5,900 ns on the 7600X, and 4,900 to 5,000 ns on
  the Pi 5, 20 to 129 times the spin.
- A 1 us spin, 1,000 ns, and then a sleep, `st1us-wtfe-sleep-futex`, measures the harness as
  much as the ring:
  - A response arrives well inside the spin, so the requester never sleeps. The worker's wait for
    the next request is the harness's time between two steps, and when that passes 1,000 ns
    the worker sleeps and the next round trip pays a wake.
  - On the Pi 5 it reads as the timed spin does, 263 against 266 ns, so we think the worker
    hardly ever sleeps there.
  - On the 7600X it reads 91 to 223 ns with claims of 4 to 36%, and on the 3900X 1,200 to 4,200
    ns with claims above 100%, so on neither is it a level.
  - A spin sized to keep a worker awake has to outlast its caller's longest pause, which this
    bench's 1,000 ns does not on the x86 hosts.

## A button to an LED

What the sleeping bench answers in a system's terms (wink, 2026-10-07): an interrupt handler
takes a button press and sends a message, a thread asleep on the ring wakes and decides, and
the output goes through a second ring to a thread that owns the hardware and lights the LED.
How long is it from the press to the light?

- Two hops, hot: `st0-wtfe-sleep-futex` is that shape, two rings, two sleepers, and two wakes,
  with the sender and the last receiver one thread. It reads 4,300 to 5,900 ns on the 7600X,
  4,900 to 5,000 ns on the Pi 5, and 6,600 to 10,000 ns on the 3900X, against 46 to 386 ns for
  the same round trip spinning. We think nearly all of it is the kernel's, the futex call, the
  wake of another CPU, and the scheduler, and little of it the ring's.
- Hot means the sleeper was asleep for microseconds: the round trips go back to back, so no
  core has idled deeply and no cache has gone cold. A thread asleep for milliseconds wakes
  slower, and how much slower is not measured.
- One hop, a sender to one sleeper that lights the LED itself, is not measured. A bench of it
  needs a pause before each send that is not timed, for the receiver to be asleep, and the
  harness times a step whole. The entry for it, and for the cold wake, is
  [Event benches](../TODO.md#event-benches-a-dithered-gap-before-each-timed-step).
- These hosts run desktop Linux, so the numbers are what the ring and that kernel cost together
  and not what a microcontroller would show on an oscilloscope.

## What the numbers can carry

- One build of each bench on each architecture: a mode or a wait choice is different code at
  different addresses, and layout alone has moved a bench by up to 8% in this repo
  ([build.md](build.md)). A difference in one row is therefore not yet the parameter's.
- What repeats is: the checks' cost has one sign in 45 of 48 rows on three hosts and two
  architectures, in v3 and again in v4, and `Single`'s one-thread gain and its `smt` gain have one
  sign in every row. We think those are the parameters' and not layout's.
- What does not repeat is `Single` across cores on the x86 hosts, where the sign turns on the
  host, the geometry it is compared with, and the wait choice.
- A round trip with one message in flight is not a stream, which zc-ring-x1 measured itself, and
  none of these benches has a second producer, which is what `Multi` is for.
- Two-thread unpinned rows on the 3900X claim up to 14%, each run drawing its own pair of CPUs,
  so they decide little there.

## Whether the mode and the wait choice earn their place

zc-ring-x1's to decide, and these are what the hosts say.

- The mode earns its place. `Single` is faster wherever a round trip is cheapest, 13 to 21% on one
  thread and 3 to 9% on a shared core on the x86 hosts, and 1.5 to 2.0% on the Pi 5, which agrees
  with zc-ring-x1's own finding that dropping the mode costs (`m-8-2`). The laptop's reading, that
  two threads show nothing, does not hold on the measuring hosts.
  - The exception is the 7600X across cores once the ring can sleep: over `SpinOrSleep<Futex>`
    `Single` is 1.7 to 2.0 ns slower than `Multi` at `ccx` and unpinned, 2.3 to 2.8%, as v3 over
    `Futex` is. There the checks cost `Single` 6.7 to 6.8% and `Multi` 1.4 to 1.6%, which takes
    `Single`'s lead and more.
- The wait choice earns its place. A ring that can sleep pays 4 to 18% of a one-thread round trip
  and 0.2 to 8.4% of a two-thread one when nobody sleeps, so `SpinOnly` is worth having for a ring
  that only spins.
- v4 costs nothing against v3 that three hosts agree on, and it adds the waits.
- A timed spin adds 13 to 25 ns to a round trip where it is seen. A sleep at every look adds
  4,000 to 10,000 ns, two sleeps and two wakes, 200 to 450 times what the timed spin adds. So a
  receiver that can afford to spin should, and for how long depends on its caller's pauses.
- A geometry can cost where no switch happens: `Multi` over one segment is 3 to 5% slower than
  over two on the 3900X across cores, 4 ns at `ccx` and 17 to 21 ns at `x-ccx` and `x-ccd`, in v3
  and in v4, and not on the 7600X or the Pi 5. We do not know why, so a comparison of the modes
  names the geometry it was made at.
- Not measured: a ring with one endpoint asleep and another spinning, what `SpinOrSleep<Futex>` is
  for, a second producer, and a stream.
