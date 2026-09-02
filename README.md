# iiac-perf

A general-purpose latency microbenchmark harness for Rust. Each
bench runs against a wall-clock time budget with auto-sized loop
counts and reports a percentile-band histogram in nanoseconds.

Numbers are raw: nothing is subtracted, so a column is what the
apparatus measured. The apparatus does cost something (a timer
pair plus the loop that drives it), and the inner loop is sized
so that cost is a small fraction of the workload's, which is what
makes a raw number usable rather than merely honest. What the
harness will not do is estimate that cost and subtract it: the
estimate is ill-defined at this scale, and it cancels anyway in
the same-harness A/B comparison the tool exists for.

Highlights:

- Time-based runs (`-d SECONDS` per bench, `-D SECONDS` total)
  with auto-sized outer/inner loop counts.
- Band-based histogram (min->p1, p1->p10, ..., p99->max) with count,
  mean, and range.
- Per-run grades for the workload and for the machine, each
  computed from the run's own data, plus an honest per-run
  resolution claim.
- Per-thread CPU pinning (`--pin-cpus`) and CPU-frequency
  control (`read-freq` / `pin-freq` / `restore-freq` /
  `suggest-freq`), so a comparison can hold the clock still.
- Per-run NDJSON records (`--record`) that outlive the session,
  self-documented by `describe-record`.
- Plug in new workloads by implementing the `Bench` trait and
  registering in `src/benches/`.

The first benches measure Inter-Intra Application Communication
(function calls, async calls, channels, serde), which is what
seeded the project name. The harness itself is workload-agnostic.
The `ice-*` benches measure iceoryx2 shared-memory IPC inside one
process, in both of its messaging patterns (`ice-ps-*`
publish/subscribe, `ice-rr-*` request/response) at one and two
threads.

## Documentation

The depth lives in `docs/`, one file per question:

- [docs/usage.md](docs/usage.md): the command line: benches,
  command words (`qualify-environment`, `suggest-freq`, the
  freq commands, completions), and every flag.
- [docs/report-guide.md](docs/report-guide.md): how to read a
  report: the Setup banner, the band table, the summary rows,
  the grade block, and what to conclude from each.
- [docs/config.md](docs/config.md): the config file: carriers,
  precedence, keys, pin profiles, and the `[freq]` steady
  state.

Design rationale and measurement records live in
[notes/](notes/README.md).

## Terminology

The docs and flags use the Linux kernel's words, because every
number passed in ends up in a kernel interface:

- **CPU**: one schedulable logical processor, the kernel's atom:
  `/sys/devices/system/cpu/cpuN`, one `sched_setaffinity` mask
  bit, one `lscpu -e` row. What `--pin-cpus` names. ("Logical
  CPU" is the same thing spelled defensively.)
- **core**: the physical core (`core_id` in the topology files).
  With SMT on, one core hosts two CPUs.
- **SMT siblings**: the CPUs sharing one core
  (`topology/core_cpus_list`). Intel brands SMT
  "Hyper-Threading".
- **software thread**: what `thread::spawn` makes; the scheduler
  places it on a CPU. Every spinning bench thread needs its own
  CPU.

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
iiac-perf qualify-environment [--runs N] [--gap SECONDS] [-d SECONDS]
iiac-perf suggest-freq BENCH [-d SECONDS] [--pin-cpus CPUS]
```

`BENCH` is one or more registered bench names, or `all` for every
registered bench. A name that matches no bench exactly runs every
bench it is a prefix of: `ice` runs all iceoryx2 benches, `mpsc`
runs `mpsc-1t` and `mpsc-2t`. **With no arguments, `iiac-perf` prints the
available list and exits, and that's the source of truth for which
benches the current build registers.**

The commands, every flag, and shell completion are in
[docs/usage.md](docs/usage.md). A quick taste:

```
iiac-perf all                                 # every bench, default ~5s each
iiac-perf mpsc-2t --pin-cpus 0,1              # pinned to two CPUs, same CCX
iiac-perf min-now --blocks 10 --block-sleep 1-10ms   # replicated, with error bars
sudo iiac-perf suggest-freq zcr-mpsc-2t --pin-cpus 0,12   # find the pin frequency
```

What a run prints, and what to conclude from it, is
[docs/report-guide.md](docs/report-guide.md).

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
spec, a single source of truth so the bot can't drift from the
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
