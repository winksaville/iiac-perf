# iiac-perf

A general-purpose latency microbenchmark harness for Rust. Each
bench runs against a wall-clock time budget with auto-sized loop
counts, reports a percentile-band histogram in nanoseconds, and
subtracts calibrated apparatus overhead (timer-pair framing +
loop-per-iter) so the output reflects the workload, not the
measurement loop.

Highlights:

- Time-based runs (`-d SECONDS` per bench, `-D SECONDS` total)
  with auto-sized outer/inner loop counts.
- Band-based histogram (min→p1, p1→p10, …, p99→max) with count,
  mean, range, and an adjusted-mean column.
- Per-thread CPU pinning (`--pin`) with independent calibration
  pinning (`--no-pin-cal`), so the cal environment stays stable
  regardless of where bench threads land.
- Plug in new workloads by implementing the `Bench` trait and
  registering in `src/benches/`.

The first benches measure Inter-Intra Application Communication
— function calls, async calls, channels, serde — which is what
seeded the project name. The harness itself is workload-agnostic.
The `ice-*` benches measure iceoryx2 shared-memory IPC inside one
process, in both of its messaging patterns (`ice-ps-*`
publish/subscribe, `ice-rr-*` request/response) at one and two
threads.

## Design (0.2.0)

Design a Rust app that can measure the cost of various (IIAC) techniques.
By IIAC I mean all communication techniques between and within
applications such as regular function calls vs async function calls vs
channels vs serde/deser of json or zero-copy messaging using io_uring and
potentially any other technique. It would include between threads or
processes or apps on the same computer or between apps on the same LAN or
between apps over a WAN.

Ideally I'd like to be able to see a histogram of the range of times of the
send and receive costs, not just the mean/average of 10^3 or 10^6 invocations.
So the cost of the work should or could be something near-zero such as just
echoing the input, but maybe there is value in doing some constant but
variable amount of work to verify it's in-consequential. One thing I think we
need to be aware of is that if we make the work too simple the compiler could
just keep things in registers which would not be representative of "real" work.

We should start simple, like comparing normal and async function calls and
later expand to other techniques.

## Usage

```
iiac-perf [BENCH...] [-d SECONDS] [-o OUTER] [-i INNER]
```

`BENCH` is one or more registered bench names, or `all` for every
registered bench. A name that matches no bench exactly runs every
bench it is a prefix of — `ice` runs all iceoryx2 benches, `mpsc`
runs `mpsc-1t` and `mpsc-2t`. **With no arguments, `iiac-perf` prints the
available list and exits — that's the source of truth for which
benches the current build registers.**

Flags (also visible via `-h` / `--help`):
- `-d`, `--duration SECONDS` — target wall-clock seconds per bench
  (default `5.0`); the outer loop runs until this time is reached
  (inner auto-sizes). See chores `0.3.1-dev1` for the empirical
  study behind the default — longer (`-d 30`+) gives
  publication-grade stability. Mutually exclusive with `-D`.
- `-D`, `--total-duration SECONDS` — target total wall-clock seconds
  across all requested benches; budget is split equally per bench
  (e.g. `-D 30` with 6 benches → 5 s each). Mutually exclusive with
  `-d`.
- `-o`, `--outer N` — override outer loop count (forces count-based
  mode instead of time-based; inner still adapts).
- `-i`, `--inner N` — override inner loop count per histogram sample.
  `inner=1` measures single-call latency (each sample = one step).
  Higher inner measures back-to-back / burst rate (each sample = N
  steps averaged, hides per-call jitter and parking costs).
- `--pin CORES` — pin bench threads to logical CPUs. `CORES` is a
  comma-separated list with optional ranges: `0,1`, `0-5`, `0,3-5,7`.
  Treated as a **core pool** indexed positionally with wrap-around, so
  thread `i` gets `pool[i % pool.len()]`. Examples:
  - `--pin 0,1` pins a 2-thread bench to logical CPUs 0 and 1.
  - `--pin 0,0` co-locates two threads on the same CPU
    (oversubscription — measures contention).
  - `--pin 0-11` defines a 12-CPU pool for larger fanout benches;
    threads wrap over it.

  A `CORES` value that names a `[profiles]` entry in the
  [config file](#config-file) expands to that profile's core spec —
  `--pin smt` with `smt = "0,12"` configured is exactly `--pin 0,12`.
  A value that isn't a profile name is parsed directly as cores, so
  raw specs keep working.

  On AMD Zen 2 (e.g. Ryzen 9 3900X, 12 physical × 2 SMT = 24 logical
  CPUs), logical IDs `N` and `N+12` are SMT siblings of the same
  physical core. `--pin 0,12` pairs siblings (max resource contention);
  `--pin 0,1` uses independent physical cores in the same CCX (best
  latency for channel benches — shared L3, no SMT contention). Check
  your topology with
  `cat /sys/devices/system/cpu/cpu0/topology/thread_siblings_list`.

  Typical measured effect on `mpsc-2t` at `-d 10` (3900X, idle desktop):
  unpinned mean ≈ 7,044 ns / stdev ≈ 6,545 ns / p99.99 ≈ 74 µs;
  `--pin 0,1` → mean ≈ 5,636 ns / stdev ≈ 1,321 ns / p99.99 ≈ 17 µs.
  Tail tightens ~4×, stdev ~5×, mean drops ~20 %.
- `--no-pin-cal` — skip pinning the main thread for calibration.
  By default, calibration runs with main pinned to `pin[0]` (if
  `--pin` is set) or core 0, so framing/loop numbers come from a
  stable environment regardless of `--pin`. Pass this flag to
  reproduce pre-0.6.0 behavior (main pinned iff `--pin` is given)
  for A/B comparison. No effect when `--pin` is set.
- `-v`, `--verbose` — print internals to stderr (affinity mask at
  startup, calibration parameters, raw `min_low`/`min_high`,
  precise fit values, calibration wall time). Equivalent to
  `RUST_LOG=debug`. Default filter is `warn` — silent unless
  something is wrong. `RUST_LOG`, when set, wins over `-v` so
  per-module filtering still works.
- `--band-labels STYLE` — label style for the histogram rows:
  `zpn` (nines/zeros + decile names: `z3`, `p50`, `n4`), `frac`
  (literal boundary fractions with `_` grouping: `0.001`, `0.50`,
  `0.999_9`), or `both` (default) — zpn and fraction side by
  side; the juxtaposition teaches the zpn vocabulary, switch to
  `zpn` once fluent. The report header records the active style
  as `labels=<style>` so saved outputs are self-describing.
- `--decimals N` — decimal digits on the report's time columns
  (0–3). Default 1 shows the sub-ns precision that picosecond
  recording captures (values are recorded internally in ps and
  displayed in ns); `0` restores integer ns; `3` is the
  recording floor — more digits would be artifacts.
- `--no-inhibit` — do not inhibit system sleep for the run. By
  default the process re-execs itself under
  `systemd-inhibit --what=sleep` so an idle-suspend can't poison a
  long measurement (a mid-sample suspend inflates that sample by
  the whole sleep gap; see the `WARNING` lines below). Where
  `systemd-inhibit` is unavailable the run continues uninhibited
  and the banner's `sleep inhibit` line says so. Pass this flag to
  keep the process image untouched (strace/gdb/perf wrappers), to
  let the machine sleep on purpose, or to test the
  suspend-detection path — a sleep inhibitor also blocks manual
  `systemctl suspend`.
- `-t`, `--ticks` — show `TProbe` results in raw hardware tick
  counts (`tk`; x86_64 TSC, aarch64 `CNTVCT_EL0`) instead of
  converting to nanoseconds. Only affects `TProbe`-based benches
  (e.g. `tp-pc`); `Probe`-based output is always in nanoseconds.
  Use this to inspect the underlying tick counts directly, e.g.
  when comparing against the counter frequency.

### Config file

Defaults and named pin profiles can live in a TOML config file, so
common invocations don't repeat flags. Precedence, lowest to
highest:

- **built-in defaults** — `duration=5.0`, `band_labels=both`,
  `decimals=1`;
- **XDG file** — `$XDG_CONFIG_HOME/iiac-perf/config.toml`, or
  `$HOME/.config/iiac-perf/config.toml` when `XDG_CONFIG_HOME` is
  unset; the per-user home for defaults and profiles;
- **project-local file** — `iiac-perf.toml` in the current
  directory (no upward walk); overrides the XDG file field by
  field, profiles merging by key;
- **CLI flags** — always win.

The startup banner's `config` line names the files that were
loaded (or `none (built-in defaults)`). A present-but-malformed
file is a hard error rather than a silent fallback, so a typo
surfaces. Every key is optional;
[`iiac-perf.toml.example`](iiac-perf.toml.example) is a ready-to-copy
sample documenting each key and its possible values:

```toml
duration     = 10.0     # default -d seconds
band_labels  = "zpn"    # zpn | frac | both
decimals     = 2        # 0-3

[profiles]              # named --pin core specs
smt = "0,12"           # SMT siblings of one physical core (contention)
ccx = "0,1"            # independent cores, same CCX (best channel latency)
ccd = "0,6"            # cross-CCD
```

Each bench prints a band-based histogram in nanoseconds. Each row
is one band, labeled by its **upper boundary**, the lower boundary
being the previous printed row: deciles in the body (`p10` …
`p90`) and **nines/zeros** notation in both tails, where `nK`/`zK`
mark the boundary with a fraction 10<sup>-K</sup> of samples above
(`n`) or below (`z`) it — so `n2` ≡ p99, `n3` ≡ p99.9, … `n10`,
and `z2` ≡ p1, `z3` ≡ p0.1, `z4`. "K nines" is standard
engineering shorthand for proportions near one
([Nines (notation)](https://en.wikipedia.org/wiki/Nines_%28notation%29),
nines = −log₁₀(1−x)); `zK` is this project's mirror of it for the
fast tail (the underlying concept is the
[survival function](https://en.wikipedia.org/wiki/Survival_function)
/ CCDF tail fraction). The slow tail subdivides down to `n10`, the
fast tail only to `z4` — a latency distribution is floored below
(nothing beats the fast path) and open above. A band only prints
when it has samples, so deep tail rows appear as run length earns
them (populating `n10` takes ~1e10 calls). Each row shows first,
last, range (`last - first + 1`), count, mean, and adjusted mean.
The trimmed `min..n2` rows exclude every band at or above
`n2` (p99).

The full boundary ladder across its range (label styles per
`--band-labels`). The ladder is generated by
[`src/bands.rs`](src/bands.rs) — the single source of truth for
boundaries and labels — and this table is pinned by that module's
unit test, so code and docs can't silently drift:

| zpn       | frac              | ≡ percentile    | tail fraction |
|-----------|-------------------|-----------------|---------------|
| `z4`      | `0.000_1`         | p0.01           | 1e-4 below    |
| `z3`      | `0.001`           | p0.1            | 1e-3 below    |
| `z2`      | `0.01`            | p1              | 1e-2 below    |
| `p10`–`p90` | `0.10`–`0.90`   | deciles         | —             |
| `n2`      | `0.99`            | p99             | 1e-2 above    |
| `n3`      | `0.999`           | p99.9           | 1e-3 above    |
| `n4`      | `0.999_9`         | p99.99          | 1e-4 above    |
| `n5`      | `0.999_99`        | p99.999         | 1e-5 above    |
| `n6`      | `0.999_999`       | p99.9999        | 1e-6 above    |
| `n7`      | `0.999_999_9`     | p99.99999       | 1e-7 above    |
| `n8`      | `0.999_999_99`    | p99.999999      | 1e-8 above    |
| `n9`      | `0.999_999_999`   | p99.9999999     | 1e-9 above    |
| `n10`     | `0.999_999_999_9` | p99.99999999    | 1e-10 above   |

The adjusted column subtracts apparatus overhead
(`framing_per_sample / inner + loop_per_iter`), calibrated once at
startup via a two-point fit on an empty bench. The startup banner
reports `cal pin` (calibration pinning) and `bench pin` (per-bench
thread pool) separately.

Runs inhibit system sleep by default (see `--no-inhibit`), so the
flags below mainly matter for uninhibited runs. A report may end
with `WARNING` lines (printed last so they can't scroll out of
mind) flagging that `max` and the untrimmed mean/stdev are
poisoned. The few inflated samples land in the
extreme tail band, so percentile boundaries, the bands below the
tail, and the trimmed `min..n2` rows remain usable:

- **system suspended** — the run spanned a system suspend,
  detected by `CLOCK_BOOTTIME` vs `CLOCK_MONOTONIC` elapsed
  divergence. A mid-sample suspend inflates that one sample by
  the whole sleep gap.
- **sample(s) clamped** — a sample exceeded the histogram's 60 s
  bound and was recorded as 60 s instead of aborting the run
  (visible as a pileup at `max`).

Examples:

```
iiac-perf                                # list available benches
iiac-perf all                            # every bench, default ~5s each
iiac-perf min-now -d 30                  # one bench, 30s budget
iiac-perf all -D 30                      # ~30s total split equally
iiac-perf mpsc-2t -i 1                   # explicit single-call latency
iiac-perf mpsc-2t -i 100                 # back-to-back rate
iiac-perf mpsc-2t --pin 0,1              # pinned, different physical cores
iiac-perf mpsc-2t --pin 0,12             # pinned, SMT siblings (contention)
iiac-perf mpsc-2t -v                     # show cal internals (affinity, raw fit)
iiac-perf mpsc-2t --no-pin-cal           # skip the cal-time pin (bench unchanged)
iiac-perf mpsc-2t --pin 5,10 --no-pin-cal # bench on 5,10; cal on full mask
RUST_LOG=info iiac-perf mpsc-2t          # info-level only (overrides -v)
```

## Example runs

All measurements below are on a Ryzen 9 3900X, idle desktop,
`mpsc-2t -d 3`. Numbers vary run-to-run and machine-to-machine;
the *shape* of the differences is the useful signal.

### `all` results (3900X, 0.13.0)

One `iiac-perf all` run (default 5 s per bench, unpinned, idle
desktop), adjusted mean per bench; probe-based benches report
their probes' unadjusted means. Same caveat as above: shapes,
not absolutes.

| bench             | adjusted mean | note                         |
|-------------------|--------------:|------------------------------|
| min-now           |          8 ns | `minstant::Instant::now`     |
| std-now           |         21 ns | `std::time::Instant::now`    |
| mpsc-1t           |         28 ns | same-thread channel          |
| mpsc-2t           |      7,658 ns | blocking `recv` (park/wake)  |
| mpsc-2t-spin      |        148 ns | spin `try_recv`              |
| probe-mpsc-2t     |      8,064 ns | probes: send 847 / 879 ns    |
| producer-consumer |             — | probes: 7,802 / 7,820 ns     |
| tp-pc             |             — | probes: 7,627 / 7,629 ns     |
| tp2-pc            |             — | probes: 7,360 / 7,360 ns     |
| ice-ps-1t         |        243 ns | iceoryx2 pub/sub, 1 thread   |
| ice-ps-2t         |        783 ns | iceoryx2 pub/sub, 2 threads  |
| ice-rr-1t         |        789 ns | iceoryx2 req/res, 1 thread   |
| ice-rr-2t         |      1,111 ns | iceoryx2 req/res, 2 threads  |
| zcr-raw-1t        |          4 ns | zc-ring-x1 raw, 1 thread     |
| zcr-raw-2t        |        132 ns | zc-ring-x1 raw, 2t, spin     |
| zcr-with-1t       |          4 ns | zc-ring-x1 `_with`, 1 thread |
| zcr-with-2t       |        137 ns | zc-ring-x1 `_with`, 2t, spin |
| zcr-spin-1t       |          4 ns | zc-ring-x1 `_spin`, 1 thread |
| zcr-spin-2t       |        122 ns | zc-ring-x1 `_spin`, 2t, spin |

The wait-policy split dominates the 2-thread rows: the parking
benches (`mpsc-2t` and the probe family, all blocking `recv`)
cluster at ~7.4-8.1 µs while the spinning benches sit under
1.2 µs. For context, iceoryx2's own pub/sub benchmark (v0.9.2,
`--bench-all`) on this machine reports 250 ns one-way — ~500 ns
round-trip — with pinned realtime threads and untouched payloads,
consistent with `ice-ps-2t`'s 783 ns measured here. The zcr rows
are the in-process zc-ring-x1 SPSC ring: 1t rounds trip in ~4 ns
(two cache-hot atomics), and the three API tiers match within
run-to-run noise — see notes/chores/chores-04.md for the pinned
tier comparison.

### Verbose output (`-v`)

`-v` prints the affinity/calibration lifecycle on stderr. The
default cal policy is visible: startup mask → `save_affinity` →
pin main to core 0 → calibrate → `restore_affinity` → benches
run on the original (unpinned) mask.

```
$ iiac-perf mpsc-2t -d 3 -v
iiac-perf 0.7.0 — Rust latency microbenchmark harness

[INFO  iiac_perf] startup affinity: 0-23 (24 cpus)
[INFO  iiac_perf::pin] save_affinity: mask=0-23 (24 cpus)
[INFO  iiac_perf] pinned main to core 0 for calibration
[DEBUG iiac_perf] affinity during cal: 0 (1 cpu)
[INFO  iiac_perf] calibration params: warmup=100000, samples=100000, N_LOW=100, N_HIGH=10000, noise_amp=1.0101
[DEBUG iiac_perf] calibration raw: min_low=60 ns, min_high=4899 ns
[DEBUG iiac_perf] calibration fit: framing=11.1212 ns, loop_per_iter=0.4888 ns
[INFO  iiac_perf] calibration wall time: 522.92 ms
[INFO  iiac_perf::pin] restore_affinity: mask=0-23 (24 cpus)
Calibration:
  framing/sample      11.12 ns  (timer pair, two-point fit)
  loop/iter            0.49 ns  (per inner-loop iteration)
  cal pin           core 0 (unpinned after cal; --no-pin-cal to skip)
  bench pin         none (unpinned)

std::sync::mpsc round-trip (2 threads) [duration=3.0s outer=402,849 inner=1 calls=402,849 adj/call=11.61ns]:
                 first            last           range        count      mean     adjusted
  min-p1           190 ns        5,923 ns        5,734 ns     3,985     1,622 ns     1,610 ns
  p1-p10         5,931 ns        6,331 ns          401 ns    36,002     6,205 ns     6,193 ns
  p10-p20        6,343 ns        6,575 ns          233 ns    42,113     6,506 ns     6,495 ns
  p20-p30        6,583 ns        6,675 ns           93 ns    39,867     6,627 ns     6,616 ns
  ...
  p90-p99        8,415 ns       10,375 ns        1,961 ns    36,070     8,954 ns     8,942 ns
  p99-max       10,383 ns    3,321,855 ns    3,311,473 ns     4,028    20,705 ns    20,693 ns
  mean                                                                  7,376 ns     7,365 ns
  stdev                                                                11,451 ns
  mean min-p99                                                          7,243 ns     7,232 ns
  stdev min-p99                                                           993 ns
```

Notice `min-p1 first = 190 ns` — sub-µs. That's the
"both-ends-hot-and-spinning" fast path, where the scheduler has
co-located bench threads on the same CCX and neither has parked
in a futex. It survives because `restore_affinity` releases main's
cal pin before benches spawn.

### Default vs `--pin 0,1`

Default (unpinned bench): wide dispersion, but the fast path is
visible.

```
$ iiac-perf mpsc-2t -d 3
Calibration:
  ...
  cal pin           core 0 (unpinned after cal; --no-pin-cal to skip)
  bench pin         none (unpinned)

  min-p1           140 ns        6,003 ns        5,864 ns     4,022     3,763 ns     3,757 ns
  ...
  p99-max       11,311 ns    4,849,663 ns    4,838,353 ns     3,871    21,443 ns    21,437 ns
  mean                                                                  7,669 ns     7,663 ns
  stdev                                                                13,120 ns
  mean min-p99                                                          7,531 ns     7,525 ns
  stdev min-p99                                                         1,148 ns
```

Pinned to two physical cores in the same CCX: tighter body, lower
mean.

```
$ iiac-perf mpsc-2t --pin 0,1 -d 3
Calibration:
  ...
  cal pin           core 0 (from --pin)
  bench pin         [0, 1] (2 slots, 2 unique CPUs)

  min-p1           210 ns        5,319 ns        5,110 ns     4,039     3,184 ns     3,173 ns
  ...
  p99-max        9,479 ns    3,602,431 ns    3,592,953 ns     4,310    44,940 ns    44,929 ns
  mean                                                                  6,886 ns     6,874 ns
  stdev                                                                26,123 ns
  mean min-p99                                                          6,503 ns     6,492 ns
  stdev min-p99                                                           776 ns
```

Side-by-side (using the trimmed `min-p99` rows, which exclude the
ms-scale OS-preemption outliers in the `p99-max` band):

| metric          | default    | `--pin 0,1` | Δ      |
|-----------------|-----------:|------------:|-------:|
| `min-p1` first  |     140 ns |      210 ns |    —   |
| `mean min-p99`  |   7,531 ns |    6,503 ns | −14 %  |
| `stdev min-p99` |   1,148 ns |      776 ns | −32 %  |

So: default gives you the sub-µs fast path *and* a wider body
(scheduler freedom); `--pin 0,1` gives tighter, lower-mean body
but loses a bit of the best case and is more sensitive to a rare
preemption (the untrimmed `stdev` can actually be *wider* pinned
— a single outlier while you're bound to one core pushes the max
to ms-scale). Use the `mean/stdev min-p99` rows for representative
central tendency and spread:

```
  mean min-p99                                                          6,503 ns     6,492 ns
  stdev min-p99                                                           776 ns
```

## Testing

```
cargo test                                    # normal run
cargo test -- --nocapture                     # show eprintln diagnostics
taskset -c 0 cargo test -- --nocapture        # restrict to 1 CPU
```

The `pin_current_can_switch_cores` test verifies that CPU pinning
works after a prior pin (the bug fixed in 0.3.6). It uses
`sched_getaffinity` to detect available CPUs, so under `taskset -c 0`
it skips gracefully rather than failing. Use `--nocapture` to see
which path was taken.

## Workflow

Commits, pushes, and finalizes follow a per-step checkpoint flow
designed for this dual-repo (app + `.claude` bot session) setup.
See [CLAUDE.md](CLAUDE.md#commit-push-finalize-flow) for the full
spec — single source of truth so the bot can't drift from the
human docs.

## Convention

This is the main repo of a dual-repo convention for using
a bot to help in the development of a coding project. The goal
is that this main repo contains the "what", while the partner
bot repo contains "why" and "how". The key to the convention
is each change is cross-referenced to the other. Thus there
is a coherent story of the development of the project across time.

The beginnings of that tool is [vc-x1](https://github.com/winksaville/vc-x1)
which currently does achieve this goal, but is being used as a
first test bed.

## Cloning

Use [vc-x1](https://github.com/winksaville/vc-x1) to clone
the dual-repo project. It handles `git clone --recursive`,
`jj` init for both repos, and the Claude Code symlink:

```
vc-x1 clone winksaville/iiac-perf
```

## jj Tips for Git Users

See [notes/jj-tips](notes/jj-tips.md)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[1]: https://github.com/karpathy/autoresearch
