# The clock experiment

Does an unpinned run's mean follow the clock it ran at, and does the sleep before a run move its
reading? On the 3900X two unpinned invocations of `min-now` a minute apart read 22.8 and 22.5 ns,
and two pinned with `--pin-freq --run-sleep 1s` both read 26.3 ns, but the sleep and the pin changed
together (2026-09-15), so neither can be named the cause.

This file is the one definition both hosts run. It sets only what the experiment fixes: the bench,
ten runs an invocation, and a record of every run. Everything else is the built-in default, and the
host's own files give only its `[freq]`. The four conditions are the same line with a flag or two,
each tagged so the records can be told apart. Run them from the repository's root, in this order,
three times over, so each condition is spread across the session rather than bunched:

```
iiac-perf configs/clock-shift.md --tag condition=unpinned-sleep
iiac-perf configs/clock-shift.md --tag condition=unpinned-nosleep --run-sleep 0
iiac-perf configs/clock-shift.md --tag condition=pinned-sleep --pin-freq
iiac-perf configs/clock-shift.md --tag condition=pinned-nosleep --pin-freq --run-sleep 0
```

Each record carries its run's mean, its `clock_khz` series, its host, and its tags, which is all
the analysis reads: `python3 configs/clock-shift.py records/clock-shift.jsonl`. The records of both
hosts are in that one file, a line a run, and what they showed is in the report guide, under
[A run's mean follows its clock][finding].

[finding]: ../docs/report-guide.md#a-runs-mean-follows-its-clock-and-the-sleep-before-it-does-not-matter

The rest of this file is the starting config as `iiac-perf init-config` wrote it: every key, a
commented-out one with no space after its `#`, and the prose that explains each.

## Run defaults

`benches` is what a run with no bench names on the line runs, a list of names, prefixes, or `"all"`,
or one of them as a string. Names on the line win, then `--benches`, then this key, and with none of
the three the bare command prints the bench list. None by default, so a benchmark directory's
`iiac-perf.md` is its natural home.

```toml
benches = ["min-now"]
```

`duration` is the target wall-clock seconds per bench, the `-d` default. `-d` on the line overrides
it, and so does `-D`, a total budget split across every run of every bench.

`total_duration` is the `-D` default, the budget for the whole invocation. It and `duration` are one
choice, so a file sets one of them, and the nearer file's choice clears the other.

`band_labels` is the histogram's label style: `"zpn"` names nines, zeros, and deciles (`z3`, `p50`,
`n4`), `"frac"` prints the boundary fractions (`0.001`, `0.50`, `0.999_9`), and `"both"` shows them
side by side, which teaches the vocabulary.

`decimals` is the digits on the report's time columns, 0 to 3: 0 is whole nanoseconds, 1 the sub-ns
precision picosecond recording captures, and 3 the recording floor.

```toml
#duration = 5.0
#total_duration = "60s"
#band_labels = "both"
#decimals = 1
```

## Warming

`settle_time` is the seconds the first bench of a process warms the box before it records anything.
Paid once per process, and every bench runs in a process of its own, so every bench pays it.
Without it a bench reports a cold machine's numbers (about 8.6% slow on a 7600x). 0 skips it.

`warm_cap` caps each run's warm-until-stable stretch. A run warms until its trailing probe window
grades A and the delivered clock holds still, or until the cap. A settled box exits in about 50 ms,
so the cap prices only the disturbed case, and hitting it is reported in the grade block. 0 caps
immediately.

```toml
#settle_time = 1.5
#warm_cap = 1.5
```

## Runs

`runs` is the runs of each bench, 1 to 1000, every run a fresh process. A process start re-rolls
where a bench's memory lands, which sets its level, so the runs' means are the replicates behind
`CI95 runs` and `LSC runs`. A bench's runs go back to back, and one run prints its report as a
single process does.

`run_sleep` is the sleep before each run, the first included, a duration or a range with a unit, a
range re-rolled per run, so every run starts alike. `"0"` starts each run as the last ends, which
leaves the first run starting from whatever the host did before and the rest starting hot.

```toml
runs = 10
#run_sleep = "1-2s"
```

## Blocks

`blocks` is the measurement blocks per run, 1 to 1000, every block sized to one sample count. Blocks
are the time axis and the replicates at once: the grades and the resolution curve read the block
series, and each block's mean is one point of the spread behind `CI95 blocks` and `LSC blocks`.
Eight is where the stats that need blocks start printing, and 100 makes a five-second run's blocks
about 50 ms.

`block_sleep` is the sleep between blocks, a duration or a range with a unit (`us`, `ms`, `s`). A
range re-rolls per block, which re-rolls scheduler and frequency state and avoids phase-locking with
kernel ticks. `"0"` never sleeps, leaving the blocks partitions of one continuous run, where
`CI95 blocks` and `LSC blocks` print `-`.

`block_warmup` is an unrecorded warmup after each block's sleep, keeping the frequency ramp and cache
refill out of the samples. `"0"` records from the first call after the wake, which is how cold-wake
behavior is seen.

```toml
#blocks = 100
#block_sleep = "1-10ms"
#block_warmup = "0"
```

## Sizing, placement, and output

`samples` and `inner` are the `--samples` and `--inner` defaults, fixed counts in place of the
auto-sizing. `inner = 1` measures single-call latency.

`pin_cpus` is the `--pin-cpus` default, a CPU spec or a `[profiles]` name. CPU numbers differ by
host, so a file that pins this way suits one host.

`record` is the `--record` default, a file to append to or a directory ending in `/`. A relative
path resolves against the current directory, as the flag's does, so a shared file carries no host's
paths.

```toml
#samples = 100000
#inner = 1
#pin_cpus = "0,1"
record = "records/clock-shift.jsonl"
```

`env_probe = false` is `--no-env-probe`, `inhibit = false` is `--no-inhibit`, `ticks = true` is
`--ticks`, and `verbose = true` is `--verbose`. The line undoes a file's choice by giving the flag a
value: `--no-env-probe=no`, `--no-inhibit=no`, `--ticks=no`, `--verbose=no`.

```toml
#env_probe = true
#inhibit = true
#ticks = false
#verbose = false
```

## A run's pin

`pin_freq` pins the clock for every run and restores the host's `[freq]` steady state when the run
exits: a frequency in MHz, or a word naming the host's own value, `"pin_mhz"` (else the base clock),
`"min_mhz"`, or `"max_mhz"`, so the same file suits every host. A target must fit under the ceiling
with boost off, which a pin turns off. `"no"`, like leaving the key out, pins nothing,
and `--pin-freq=no` skips a file's pin for one run. A benchmark directory's `iiac-perf.md` is its
natural home.

```toml
#pin_freq = "min_mhz"
```

## Pin profiles

`[profiles]` maps a name to a `--pin-cpus` spec, so `--pin-cpus <name>` expands to it, and a value
that is not a profile name still parses as a raw spec: `"0,1"`, `"0-5"`, `"0,3-5,7"`. None are
defined by default. These are for a Ryzen 9 3900X, where CPUs N and N+12 are SMT siblings of one
physical core, so adjust them to your topology (`lscpu -e`): `smt` is the two siblings of one core,
the most contention, `ccx` two independent cores in one CCX, the best channel latency, and `ccd` two
cores across CCDs.

The tables come after every top-level key, here and in any config, because the fences concatenate in
order and a bare key after a table header would land in that table.

```toml
#[profiles]
#smt = "0,12"
#ccx = "0,1"
#ccd = "0,6"
```

## The clock steady state

`[freq]` declares the host's steady state, what `restore-freq` converges to and every pin restores
on exit: `governor`, `epp`, `boost`, the clamp `min_mhz` to `max_mhz`, and optionally `pin_mhz`. It
belongs in the XDG file, since it describes the host rather than a project, and a project-local
`[freq]` replaces the XDG one whole. `min_mhz` and `max_mhz` are required by every command that pins
or restores.

No values are shown, because one host's are wrong on another: `iiac-perf setup-freq` prints this
host's table from the live state, and `iiac-perf setup-freq --apply` writes it here.

```toml
#[freq]
```

## Tags

`[tags]` puts a `KEY=VALUE` on every record, each entry a `--tag`. The tool never reads one: the
caller knows which runs form an experiment. The files merge by key, a `--tag` on the line adds to
them and wins on a shared key, and a tag with no record is an error.

```toml
[tags]
experiment = "clock-shift"
```
