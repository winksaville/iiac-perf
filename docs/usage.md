# Usage

The command line: benches, command words, every flag, and shell
completion. Moved verbatim from the README, whose
[Usage section](../README.md#usage) keeps the synopsis. How to
read what a run prints is
[report-guide.md](report-guide.md); the config file is
[config.md](config.md).

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

## Command words

`iiac-perf qualify-environment` (also stand-alone) asks whether
this **machine** is fit to measure on:
it respawns this binary `--runs` times (default 10) at `--gap`
seconds apart, collects each run's environment grade, prints the
table, and gives a verdict, exiting nonzero when the machine
does not qualify. Use it to characterize a box before trusting
numbers from it.

What it runs is, in metrology terms, a repeatability study of the
apparatus plus the machine, which is why the grading module is
called `gauge`.

```
  run   warmup  bench    worst   settle                      mean
  1     A       A        A       4.84->5.24GHz 49% +-0.0%    22.2 ns
  2     B       A        B       4.84->5.24GHz 18% +-0.1%    22.2 ns
  3     F       C        F       4.85->5.24GHz 00%           22.5 ns

  median environment grade: B
  median settled: 18% of warmup (1 of 3 never settled)
  transition-degraded (drift or step at D/F): 1 of 3

  verdict: NOT QUALIFIED
    a state transition landed inside a measurement window
```

It reads the environment grade rather than the run grade,
because the subject is the box: `warmup` is its settling
behaviour across respawns, `bench` whether it then held. The
verdict is grades, not values: median at B or better, and no
run whose `drift` or `step` reached D/F, those two being the
transition detectors. `spread` and `interference` wobble is
ambient contamination and does not fail a run. The `mean` column
is informational, and it is where a two-state machine shows
itself at a glance.

The `settle` column is each child warmup's clock journey and
settled share (`00%` with a warmup F when it never settled; see
[Settle time](report-guide.md#settle-time)). The states
themselves are the fastest read on a two-state box: which clock
each run measured at, and a small share is a box that wants more
`--settle-time` than it got.

Each child runs `min-now` for `-d` seconds (default 1): the box
is the subject, so the leanest bench is the right one. `--pin-cpus`
and `--settle-time` pass through to the children, and
`--print-only` prints the table without deciding a verdict.

`iiac-perf suggest-freq BENCH` (needs root and a declared `[freq]`
steady state in the config) measures the best pin frequency for
that bench: it descends from max-with-boost-off, pins each
candidate (min = max, boost off), drives the real bench with this
command line's `-d` / `--pin-cpus`, and reports the highest
frequency the box *held* (delivered clock stable and on target),
ending with the `pin_mhz = ...` line to paste into the config.
The suggestion is per bench, duration, and pin layout, because
the schedule selects the state the box can hold. The declared
steady state restores on every catchable exit, like `pin-freq`.

Tab completes bench names, command words, and flags once the
shell is hooked to the binary, one line in the shell's rc file.
Without it, `iiac-perf ice<TAB>` has nothing to offer and bench
names must be typed (or copied from the no-args listing) by
hand. See [Shell completion](#shell-completion).

## Flags

Flags (also visible via `-h` / `--help`):
- `-d`, `--duration SECONDS`: target wall-clock seconds per bench
  (default `5.0`); the outer loop runs until this time is reached
  (inner auto-sizes). See chores `0.3.1-dev1` for the empirical
  study behind the default; longer (`-d 30`+) gives
  publication-grade stability. Mutually exclusive with `-D`.
- `-D`, `--total-duration SECONDS`: target total wall-clock seconds
  across all requested benches; budget is split equally per bench
  (e.g. `-D 30` with 6 benches -> 5 s each). Mutually exclusive with
  `-d`.
- `-o`, `--outer N`: override outer loop count (forces count-based
  mode instead of time-based; inner still adapts).
- `-i`, `--inner N`: override inner loop count per histogram sample.
  `inner=1` measures single-call latency (each sample = one step).
  Higher inner measures back-to-back / burst rate (each sample = N
  steps averaged, hides per-call jitter and parking costs).
- `--pin-cpus CPUS`: pin bench threads to CPUs (see
  [Terminology](../README.md#terminology); `--pin` is a hidden
  alias from the flag's old name). `CPUS` is a comma-separated
  list with optional ranges: `0,1`, `0-5`, `0,3-5,7`. Treated as
  a **CPU pool** indexed positionally with wrap-around, so
  thread `i` gets `pool[i % pool.len()]`. Examples:
  - `--pin-cpus 0,1` pins a 2-thread bench to CPUs 0 and 1.
  - `--pin-cpus 0,0` co-locates two threads on the same CPU
    (oversubscription, which measures contention).
  - `--pin-cpus 0-11` defines a 12-CPU pool for larger fanout benches;
    threads wrap over it.

  A `CPUS` value that names a `[profiles]` entry in the
  [config file](config.md) expands to that profile's CPU spec,
  `--pin-cpus smt` with `smt = "0,12"` configured is exactly `--pin-cpus 0,12`.
  A value that isn't a profile name is parsed directly as CPUs, so
  raw specs keep working.

  On AMD Zen 2 (e.g. Ryzen 9 3900X, 12 physical cores × 2 SMT = 24
  CPUs), CPUs `N` and `N+12` are SMT siblings of the same physical
  core. `--pin-cpus 0,12` pairs siblings (max resource contention);
  `--pin-cpus 0,1` uses independent physical cores in the same CCX (best
  latency for channel benches: shared L3, no SMT contention). Check
  your topology with
  `cat /sys/devices/system/cpu/cpu0/topology/thread_siblings_list`.

  Typical measured effect on `mpsc-2t` at `-d 10` (3900X, idle desktop):
  unpinned mean ≈ 7,044 ns / stdev ≈ 6,545 ns / p99.99 ≈ 74 µs;
  `--pin-cpus 0,1` -> mean ≈ 5,636 ns / stdev ≈ 1,321 ns / p99.99 ≈ 17 µs.
  Tail tightens ~4×, stdev ~5×, mean drops ~20 %.
- `-v`, `--verbose`: print internals to stderr: the affinity mask
  at startup, the pin lifecycle, and the TSC tick rate.
  Equivalent to `RUST_LOG=debug`. Default filter is
  `warn`, silent unless something is wrong. `RUST_LOG`, when
  set, wins over `-v` so per-module filtering still works.
- `--band-labels STYLE`: label style for the histogram rows:
  `zpn` (nines/zeros + decile names: `z3`, `p50`, `n4`), `frac`
  (literal boundary fractions with `_` grouping: `0.001`, `0.50`,
  `0.999_9`), or `both` (default), zpn and fraction side by
  side; the juxtaposition teaches the zpn vocabulary, switch to
  `zpn` once fluent. The report header records the active style
  as `labels=<style>` so saved outputs are self-describing.
- `--decimals N`: decimal digits on the report's time columns
  (0-3). Default 1 shows the sub-ns precision that picosecond
  recording captures (values are recorded internally in ps and
  displayed in ns); `0` restores integer ns; `3` is the
  recording floor; more digits would be artifacts. The flag
  covers exactly the band table's time columns and the
  mean/stdev rows. The grade block keeps its own fixed
  precision: its percentages are ratios, not times (at
  `--decimals 0` a `spread 0%` cell would destroy the column's
  signal), and its `step` timestamp prints at two decimals
  because batches flush at ~15-50 ms, so neither series locates
  a step finer than 10 ms. `ticks/ns` in the `Setup:` block is
  likewise a fixed-precision ratio.
- `--blocks N`: N (2-1000) is the **number of measurement
  blocks** the run's budget is divided into: `--blocks 10`
  with `-d 10` measures 10 blocks of ~1 s each (total measured
  time still 10 s; with `-o` the sample count is divided
  instead). Between blocks the harness sleeps and re-warms
  only as `--block-sleep` / `--block-warmup` ask (both default
  0; neither is counted in the budget). The report gains three
  lines (`mean blocks` (mean of the N block means), `CI95`
  (95% **c**onfidence **i**nterval half-width on it), and
  `LSC` (**l**east **s**ignificant **c**hange vs an equal-N
  run), and the header records `blocks=N`. CI95 and LSC print
  `-` when the sleep is 0: sleepless blocks are partitions of
  one continuous run, not independent replicates, and a number
  built on them would be fiction. Blocks nest above batches:
  each block is a contiguous stretch of whole batches (the
  flush aligns batch boundaries to the block gaps), so batches
  stay the grade block's time-series grain while blocks are
  the CI's replication grain. N is also the statistical
  replication count: more blocks -> tighter CI but shorter
  blocks. Interpretation: an honest *within-invocation* error
  bar; treat it as a lower bound on cross-invocation
  confidence and pin the bench (`--pin-cpus`); unpinned,
  per-process thread placement dominates and blocks can't see
  it. Bench-driven benches only; probe benches ignore it. See
  [validation](../notes/design.md#block-validation-results-0210-4-r5-7600x)
  and the
  [design](../notes/design.md#within-invocation-replication-sleep-separated-blocks).
- `--block-sleep SPAN`: sleep between blocks, a duration or
  range with unit (`us`, `ms`, `s`): `--block-sleep 1-10ms`
  re-rolls a random sleep per block (re-rolls
  scheduler/frequency state; a range avoids phase-locking with
  kernel ticks and the flip-zone hazard a fixed value invites),
  `--block-sleep 1s` sleeps exactly 1 s (long sleeps reach deep
  C-states, so wakes start colder). Default 0: never sleep,
  blocks are partitions, replication rows print `-`. Config key
  `block_sleep`. The resolved value prints in `Setup:` whenever
  blocks run and rides the record.
- `--block-warmup DUR`: unrecorded post-wake warmup per block
  (duration with unit). Keeps the frequency ramp and cache
  refill out of the samples after each sleep. Default 0:
  record from the first post-wake call, which is how cold-wake
  behavior is seen. Config key `block_warmup`. Prints in
  `Setup:` and rides the record like the sleep.
- `--no-env-probe`: stop probing the environment at batch
  seams, limiting the `env` grade to the warmup probes (the few
  ms before the bench starts) instead of the whole run. Seam
  probing perturbs a spinning multi-threaded bench by ~0.9%
  (measured on `zcr-spsc-v0-2t`; ~0.5% on a single-threaded one),
  which is common-mode in an A/B between benches but not in an
  absolute number. See
  [The two grades](report-guide.md#the-two-grades).
- `--settle-time SECONDS`: seconds the **first** bench of a
  process spends warming the box before it records anything
  (default `1.5`, or the config `settle_time`). `0` skips the
  warm. Paid once per process, since later benches inherit the
  machine state it wins; the grade block's `settle` cell reports
  the clock's journey and the settled share of the warm. See
  [Settle time](report-guide.md#settle-time).
- `--warm-cap SECONDS`: cap on each run's warm-until-stable
  stretch (default `1.5`, or the config `warm_cap`). Every run
  warms until the trailing probe window grades A (and the
  delivered clock holds still, where readable) or until this cap;
  a settled box exits in ~50 ms, so the cap prices only the
  disturbed case. Hitting it is reported in the grade block
  (a `00%` settle cell with an F, or `uncertified`), never
  silently absorbed. `0`
  caps immediately, which measures what the warm is worth.
- `--no-inhibit`: do not inhibit system sleep for the run. By
  default the process re-execs itself under
  `systemd-inhibit --what=sleep` so an idle-suspend can't poison a
  long measurement (a mid-sample suspend inflates that sample by
  the whole sleep gap; see the `WARNING` lines in
  [report-guide.md](report-guide.md#warnings)). Where
  `systemd-inhibit` is unavailable (absent, or the lock is
  denied (e.g. a headless ssh session with no polkit
  interactive auth), the run continues uninhibited and the
  banner's `sleep inhibit` line says so. Pass this flag to
  keep the process image untouched (strace/gdb/perf wrappers), to
  let the machine sleep on purpose, or to test the
  suspend-detection path, since a sleep inhibitor also blocks manual
  `systemctl suspend`.
- `-t`, `--ticks`: show `TProbe` results in raw hardware tick
  counts (`tk`; x86_64 TSC, aarch64 `CNTVCT_EL0`) instead of
  converting to nanoseconds. Only affects `TProbe`-based benches
  (e.g. `tp-pc`); `Probe`-based output is always in nanoseconds.
  Use this to inspect the underlying tick counts directly, e.g.
  when comparing against the counter frequency.
- `--list-benches`: print the registered bench names, one per
  line, and exit. Machine-readable, for scripts to iterate
  (`for b in $(iiac-perf --list-benches); ...`). The command
  words are not bench names and aren't listed.

## Examples

```
iiac-perf                                # list available benches
iiac-perf all                            # every bench, default ~5s each
iiac-perf min-now -d 30                  # one bench, 30s budget
iiac-perf all -D 30                      # ~30s total split equally
iiac-perf mpsc-2t -i 1                   # explicit single-call latency
iiac-perf mpsc-2t -i 100                 # back-to-back rate
iiac-perf mpsc-2t --pin-cpus 0,1              # pinned, different physical cores
iiac-perf mpsc-2t --pin-cpus 0,12             # pinned, SMT siblings (contention)
iiac-perf mpsc-2t --pin-cpus 0,1 --blocks 10  # pinned + error bar (ci95/lsc lines)
iiac-perf mpsc-2t -v                     # show internals (affinity, warmup table)
RUST_LOG=info iiac-perf mpsc-2t          # info-level only (overrides -v)
```

## Shell completion

The binary completes itself. The shell is hooked once, with one
line in its rc file, and from then on every Tab runs the binary
with `COMPLETE` set, which answers with the candidates and
exits: flags, command words with a one-line description, and
bench names, all from the running build, so nothing about
completion can go stale. This is clap's dynamic completion
(`clap_complete`'s `CompleteEnv`).

```
# bash: ~/.bashrc
source <(COMPLETE=bash iiac-perf)

# zsh: ~/.zshrc
source <(COMPLETE=zsh iiac-perf)

# fish: ~/.config/fish/completions/iiac-perf.fish
COMPLETE=fish iiac-perf | source

# elvish: ~/.elvish/rc.elv
eval (E:COMPLETE=elvish iiac-perf | slurp)

# powershell: $PROFILE
$env:COMPLETE = "powershell"
iiac-perf | Out-String | Invoke-Expression
Remove-Item Env:\COMPLETE
```

The line names the binary, so a build installed under another
name (`iiac-perf-dev`, the dev name a cycle builds under) gets
its own line. The hook is regenerated at every shell start, so an
upgrade needs nothing beyond a new shell, or re-sourcing the line
in the current one.
