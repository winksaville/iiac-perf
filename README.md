# iiac-perf

Measure performance of various Inter-Intra Application Communication
(IIAC) techniques.

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
registered bench. **With no arguments, `iiac-perf` prints the
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

Each bench prints a band-based histogram in nanoseconds. Bands are
defined by percentile boundaries (min→p1, p1→p10, …, p99→max) and
show first, last, range (`last - first + 1`), count, mean, and
adjusted mean. The adjusted column subtracts apparatus overhead
(`framing_per_sample / inner + loop_per_iter`), calibrated once at
startup via a two-point fit on an empty bench.

Examples:

```
iiac-perf                        # list available benches
iiac-perf all                    # every bench, default ~5s each
iiac-perf min-now -d 30          # one bench, 30s budget
iiac-perf all -D 30              # ~30s total split equally
iiac-perf mpsc-2t -i 1           # explicit single-call latency
iiac-perf mpsc-2t -i 100         # back-to-back rate
iiac-perf mpsc-2t --pin 0,1      # pinned, different physical cores
iiac-perf mpsc-2t --pin 0,12     # pinned, SMT siblings (contention)
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
