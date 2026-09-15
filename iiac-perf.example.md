# iiac-perf config example

A sample showing every key, at its built-in default where it is set. A markdown config is a document
whose `toml` fences, read in order, are the config, so the prose between them explains the keys to
whoever reads the file. [docs/config.md](docs/config.md) is the full reference.

Copy it to one of these, the nearer file winning field by field:

- `$XDG_CONFIG_HOME/iiac-perf/config.md`, or `~/.config/iiac-perf/config.md` when
  `XDG_CONFIG_HOME` is unset: per-user, the home for the host's `[freq]` steady state
- `./iiac-perf.md`: project-local, the current directory only, no upward walk

Precedence, lowest to highest: built-in defaults, the XDG file, the project-local file, CLI flags.
Every key is optional, and an omitted key keeps its built-in default. A present but malformed file
is a hard error, so a typo surfaces rather than silently reverting to defaults. The report's
`Config:` list names the files loaded and where every value came from.

## Run defaults

`benches` is what a run with no bench names on the line runs, a list of names, prefixes, or `"all"`,
or one of them as a string. Names on the line win, then `--benches`, then this key, and with none of
the three the bare command prints the bench list. None by default, so a benchmark directory's
`iiac-perf.md` is its natural home.

```toml
# benches = ["zcr-mpsc-v0-2t", "zcr-mpsc-v1-2t"]
```

`duration` is the target wall-clock seconds per bench, the `-d` default. `-d` on the line overrides
it, and so does `-D`, a total budget split across every run of every bench.

`band_labels` is the histogram's label style: `"zpn"` names nines, zeros, and deciles (`z3`, `p50`,
`n4`), `"frac"` prints the boundary fractions (`0.001`, `0.50`, `0.999_9`), and `"both"` shows them
side by side, which teaches the vocabulary.

`decimals` is the digits on the report's time columns, 0 to 3: 0 is whole nanoseconds, 1 the sub-ns
precision picosecond recording captures, and 3 the recording floor.

```toml
duration = 5.0
band_labels = "both"
decimals = 1
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
settle_time = 1.5
warm_cap = 1.5
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
runs = 5
run_sleep = "1-2s"
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
blocks = 100
block_sleep = "1-10ms"
block_warmup = "0"
```

## Pin profiles

`[profiles]` maps a name to a `--pin-cpus` spec, so `--pin-cpus <name>` expands to it, and a value
that is not a profile name still parses as a raw spec: `"0,1"`, `"0-5"`, `"0,3-5,7"`. None are
defined by default. These are for a Ryzen 9 3900X, where CPUs N and N+12 are SMT siblings of one
physical core, so adjust them to your topology (`lscpu -e`).

```toml
# [profiles]
# smt = "0,12"   # SMT siblings of one physical core, the most contention
# ccx = "0,1"    # independent cores in one CCX, the best channel latency
# ccd = "0,6"    # across CCDs
```

## The clock steady state

`[freq]` declares the host's steady state, what `restore-freq` converges to and every pin restores
on exit. It belongs in the XDG file, since it describes the host rather than a project, and a
project-local `[freq]` replaces the XDG one whole. Don't copy these values: `iiac-perf setup` prints
the host's own from the live state, and `iiac-perf setup --apply` writes them. `min_mhz` and
`max_mhz` are required by every command that pins or restores.

It goes last, because the fences concatenate in order and a bare key after a table header would land
in that table.

```toml
# [freq]
# governor = "powersave"
# epp = "balance_performance"
# boost = true
# min_mhz = 1745
# max_mhz = 4673
```

## A run's pin

`pin_freq` pins the clock for every run and restores the host's `[freq]` steady state when the run
exits: a frequency in MHz, or a word naming the host's own value, `"pin_mhz"` (else the base clock),
`"min_mhz"`, or `"max_mhz"`, so the same file suits every host. A target must fit under the ceiling
with boost off, which a pin turns off. `"no"`, like leaving the key out, pins nothing,
and `--pin-freq=no` skips a file's pin for one run. A benchmark directory's `iiac-perf.md` is its
natural home. It is a top-level key, so in a real file it goes in a fence above the `[freq]` table.

```toml
# pin_freq = "min_mhz"
```
