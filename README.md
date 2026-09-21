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
  with auto-sized sample and inner counts.
- Every run is measured in blocks sized to one sample count, 100
  by default (`--blocks N`), each capped at twice its share of the
  budget. One block is a plain run, and 8 is the
  suggested minimum: below it the grades and stats that need
  more blocks print `-` rather than a number, and the report
  says so.
- Band-based histogram (min->p1, p1->p10, ..., p99->max) with count,
  mean, and range.
- Per-run grades for the workload and for the machine, each
  computed from the run's own data, plus an honest per-run
  resolution claim.
- Per-thread CPU pinning (`--pin-cpus`) and CPU-frequency
  control (`read-freq` / `pin-freq` / `restore-freq` /
  `suggest-freq`), so a comparison can hold the clock still, and
  `setup-freq` to declare the host's clock steady state for them.
- Per-run JSONL records (`--record-dir`, `--record-file`) that outlive the session,
  self-documented by `describe-record`, and `analyze` to read them back and check
  whether a claim held across invocations.
- Plug in new workloads by implementing the `Bench` trait and
  registering in `src/benches/`.

The first benches measure Inter-Intra Application Communication
(function calls, async calls, channels, serde), which is what
seeded the project name. The harness itself is workload-agnostic.
The `ice-*` benches measure iceoryx2 shared-memory IPC inside one
process, in both of its messaging patterns (`ice-ps-*`
publish/subscribe, `ice-rr-*` request/response) at one and two
threads. The `cb-*` benches are crossbeam's unbounded channel and
`SegQueue`, ecosystem baselines for the `zcr-*` benches over the
zc-ring-x1 rings, and the report guide's `all` table says which
queue promises what, since a queue that promises less is expected
to be faster.

## Documentation

The depth lives in `docs/`, one file per question:

- [docs/usage.md](docs/usage.md): the command line: benches,
  command words (`qualify-environment`, `suggest-freq`, the
  freq commands, completions), and every flag.
- [docs/report-guide.md](docs/report-guide.md): how to read a
  report: the Setup banner, the band table, the summary rows,
  the grade block, and what to conclude from each.
- [docs/config.md](docs/config.md): the config file: carriers,
  precedence, keys, pin profiles, replication defaults, and the
  `[freq]` steady state.

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
- **software thread**: what `thread::spawn` makes. The scheduler
  places it on a CPU. Every spinning bench thread needs its own
  CPU.

The words of a report's claim, each explained with its formula, a source, and a figure in
[docs/statistics.md](docs/statistics.md):

- **trimmed mean**: the runs' mean with the lowest 10% and the highest 50% dropped (`--trim-runs`), so
  the runs that landed somewhere slow do not move it.
- **CI95**: how well one invocation knows its own mean, the 95% confidence interval's half-width.
- **LSC**: the least significant change, the smallest difference between two invocations worth
  believing, `sqrt(2)` times CI95's standard error.
- **a**, **s_p**: the two noises, within a run (more measuring shrinks it) and between processes
  (only more runs do).
- **o**, **d\***: a run's overhead in seconds, and the run length that reaches a precision in the
  least time, `sqrt(a * o / s_p^2)`.

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
iiac-perf [BENCH...] [--runs N] [-d SECONDS] [-o OUTER] [-i INNER]
iiac-perf qualify-environment [--runs N] [--run-sleep SPAN] [-d SECONDS]
iiac-perf suggest-freq BENCH [-d SECONDS] [--pin-cpus CPUS]
```

`BENCH` is one or more registered bench names, or `all` for every
registered bench. A name that matches no bench exactly runs every
bench it is a prefix of: `ice` runs all iceoryx2 benches, `mpsc`
runs `mpsc-1t` and `mpsc-2t`. A name that is no prefix either runs
every bench it matches as a regular expression: `zcr-[sm]psc-v[23]`
runs the v2 and v3 pairs of both rings. **With no arguments, `iiac-perf` prints the
available list and exits, and that's the source of truth for which
benches the current build registers.**

The commands, every flag, and shell completion are in
[docs/usage.md](docs/usage.md). A quick taste:

```
iiac-perf all                                 # every bench, default ~5s each
iiac-perf mpsc-2t --pin-cpus 0,1              # pinned to two CPUs, same CCX
iiac-perf min-now --blocks 10 --block-warmup 2ms   # ten replicates, post-wake ramp discarded
sudo iiac-perf suggest-freq zcr-mpsc-v0-2t --pin-cpus 0,12   # find the pin frequency
```

Every run flag has a config key, so a box can set its
replication once and a run needs no flags: `blocks`,
`block_sleep`, `block_warmup`, and the rest in
[docs/config.md](docs/config.md), with
[iiac-perf.example.md](iiac-perf.example.md) as the starting
file, which `iiac-perf init-config PATH` writes. Every
run has a hundred blocks by default, so each carries an error bar
without a config at all.

### Config files

A config names the benches as well as the knobs, so a whole run
is a file: `iiac-perf queue.md`. Every run flag has a key, and the
report's `Config:` list shows every value with where it came from,
a file, a flag, or `(default)`, so what applied is never a guess.
The full reference is [docs/config.md](docs/config.md).

#### Which files are read

A plain run layers up to two files under the flags, and for any
key the nearest file that sets it wins:

| Order | File | What it is for |
|---|---|---|
| 1 | built-in defaults | every key has one, or is off |
| 2 | `~/.config/iiac-perf/config.md` | the host: its `[freq]` clock steady state, its pin `[profiles]` |
| 3 | the nearest `iiac-perf.md`, this directory then each parent | a project's or a tree's defaults |
| 4 | flags on the line | this run |

The search for `iiac-perf.md` stops at the first found, so a file
high in a tree is what the directories below fall back to, never a
layer under a nearer one. `[tags]` and `[profiles]` merge by entry,
and `[freq]` replaces whole.

A named config changes rows 2 and 3. Under `iiac-perf queue.md`, or
`--config queue`, the run keys come from `queue.md` and the defaults
alone, flags still winning, so the same file is the same run on
every host. The host's files then give only `[freq]` and
`[profiles]`, which describe the host rather than the run. The name
is looked for in this directory, each parent, then
`~/.config/iiac-perf/`, so a tree of bench directories shares a
parent's config by name.

`[freq]` is the host's: the governor, EPP, boost, and clamp that
`restore-freq` and every pin's exit return to. No flag sets it and
`init-config` leaves it empty. `iiac-perf setup-freq --apply` writes
it from the live state. A run's pin is the separate key `pin_freq`.

#### The commands

| To | Run |
|---|---|
| see every key, commented out at its default | `iiac-perf init-config \| less` |
| turn a line that worked into a file | `iiac-perf init-config quick.md --benches min-now --blocks 10 -d 0.5s` |
| run a file | `iiac-perf quick.md` |
| run it with one thing changed, once | `iiac-perf quick.md --blocks 20`, or `iiac-perf quick.md std-now` |
| change a key in the file | `iiac-perf update-config quick.md --blocks 20` |
| bring an old file up to date | `iiac-perf update-config old.md --backup` |
| start a new file from another's values | `iiac-perf init-config --from old.md new.md` |
| start from the bare template | `iiac-perf init-config --from /dev/null new.md` |
| rerun what a record measured, on any host | `iiac-perf init-config --from-record file.jsonl new.toml` |

`init-config` writes the run its line would make on this host: the
line's flags, over the run keys the host's files set, and it names
what it took from each file. It never writes over a file unless
`--backup` (keeps `PATH.bak`) or `--overwrite` (keeps nothing) says
so, and the old file's values are then gone. `update-config` keeps
them: it sets the line's flags over the file's own values and
rewrites it in place, checked before it is touched, `--backup`
keeping the old file. Either rewrite loses prose the author added.

#### The two carriers

A `.md` config is a document whose `toml` fences, read in order,
are the config, so the prose between them explains each key. A
`.toml` config is the keys under ruled section headings, with no
prose, since as comments the prose buries the keys. In both, a
commented-out key has no space after its `#` and a comment has one:

```toml
# ---- Blocks ----

blocks = 10
#block_sleep = "1-10ms"
#block_warmup = "0"
```

Top-level keys go before any table, in either carrier: a bare key
after `[tags]` would land inside it.

#### From a line to a file, on two hosts

```
iiac-perf --benches zcr-spsc-v3-2t -d 0.25s --blocks 10 --pin-freq      # a line that works
iiac-perf init-config configs/spsc.md --benches zcr-spsc-v3-2t -d 0.25s --blocks 10 --pin-freq
iiac-perf configs/spsc.md                                                # the same run
```

Commit `configs/spsc.md`, pull it on the other host, and run the
same last line there. The two `Config:` lists then agree key for
key, each naming `configs/spsc.md` as the source, and differ only in
what is the host's: the `freq` row, and the clock the `pin_freq`
word resolves to there. To try a change once, add the flag,
and the list shows that flag as its key's source. To keep it,
`iiac-perf update-config configs/spsc.md --blocks 20`.

#### When it stops

| Message | Cause |
|---|---|
| ``unknown field `bogus`, expected one of ...`` | a key this version does not know: a typo, or a file from another version |
| `duration and total_duration are both set: keep one` | they are one choice, and one file made it twice |
| `a tag needs a record target, from --record-dir, --record-file, ...` | `[tags]` or `--tag` with nowhere to write them |
| `record: a path's shape no longer picks the mode, so the key is gone` | a file from before `record_dir` and `record_file` |
| `the run's config nope.md not found, in:` and the places tried | a named config that is in none of them |
| `queue.md exists, and is left as it is: --backup ... --overwrite ...` | `init-config` over a file, without saying so |
| `The [freq] in use is from iiac-perf.md.` ending a refusal | which file's `[freq]` a pin or restore refused |
| `warning: the [freq] from x.md is not the state this host runs at` | a table from another host: the restore will move this one to it |

A malformed file is always an error, never a silent fallback to the
defaults.

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
