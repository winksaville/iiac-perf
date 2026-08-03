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
  computed from the run's own data.
- Per-thread CPU pinning (`--pin`): thread 0 of a bench measures
  on main, which pins to the pool's first slot; the warm loop
  runs there too, so the frequency state it wins lands on the
  core that measures.
- Plug in new workloads by implementing the `Bench` trait and
  registering in `src/benches/`.

The first benches measure Inter-Intra Application Communication
(function calls, async calls, channels, serde), which is what
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
iiac-perf qualify-environment [--runs N] [--gap SECONDS] [-d SECONDS]
iiac-perf add-completion-yaml
```

`BENCH` is one or more registered bench names, or `all` for every
registered bench. A name that matches no bench exactly runs every
bench it is a prefix of: `ice` runs all iceoryx2 benches, `mpsc`
runs `mpsc-1t` and `mpsc-2t`. **With no arguments, `iiac-perf` prints the
available list and exits, and that's the source of truth for which
benches the current build registers.**

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
  run   warmup  bench    worst   settle   mean
  1     A       A        A       0.86s    22.2 ns
  2     D       A        D       1.31s    22.2 ns
  3     A       C        C       0.61s    22.5 ns

  median environment grade: A
  median settle: 0.86s (0 of 3 never settled)
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

The `settle` column is how long each child's warmup took to
reach the state it measured in (`not` when it never did; see
[Settle time](#settle-time)). It is reported, not judged: a box
still moving when warmup ended already shows up as a `drift` or
`step` D/F on the warmup stretch, so a second criterion would
only restate the first. What it adds is the size of the number:
how much `--settle-time` this box actually wants.

Each child runs `min-now` for `-d` seconds (default 1): the box
is the subject, so the leanest bench is the right one. `--pin`
and `--settle-time` pass through to the children, and
`--print-only` prints the table without deciding a verdict.

`iiac-perf add-completion-yaml` (also stand-alone) installs the
carapace completion spec: Tab then completes bench names, command
words, and flags in any carapace-served shell. Without it,
`iiac-perf ice<TAB>` has nothing to offer and bench names must
be typed (or copied from the no-args listing) by hand. Run it
once after installing iiac-perf, and again after an upgrade
that changes flags or command words; new benches never need a
re-run. See [Shell completion](#shell-completion).

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
- `--pin CORES`: pin bench threads to logical CPUs. `CORES` is a
  comma-separated list with optional ranges: `0,1`, `0-5`, `0,3-5,7`.
  Treated as a **core pool** indexed positionally with wrap-around, so
  thread `i` gets `pool[i % pool.len()]`. Examples:
  - `--pin 0,1` pins a 2-thread bench to logical CPUs 0 and 1.
  - `--pin 0,0` co-locates two threads on the same CPU
    (oversubscription, which measures contention).
  - `--pin 0-11` defines a 12-CPU pool for larger fanout benches;
    threads wrap over it.

  A `CORES` value that names a `[profiles]` entry in the
  [config file](#config-file) expands to that profile's core spec,
  `--pin smt` with `smt = "0,12"` configured is exactly `--pin 0,12`.
  A value that isn't a profile name is parsed directly as cores, so
  raw specs keep working.

  On AMD Zen 2 (e.g. Ryzen 9 3900X, 12 physical × 2 SMT = 24 logical
  CPUs), logical IDs `N` and `N+12` are SMT siblings of the same
  physical core. `--pin 0,12` pairs siblings (max resource contention);
  `--pin 0,1` uses independent physical cores in the same CCX (best
  latency for channel benches: shared L3, no SMT contention). Check
  your topology with
  `cat /sys/devices/system/cpu/cpu0/topology/thread_siblings_list`.

  Typical measured effect on `mpsc-2t` at `-d 10` (3900X, idle desktop):
  unpinned mean ≈ 7,044 ns / stdev ≈ 6,545 ns / p99.99 ≈ 74 µs;
  `--pin 0,1` -> mean ≈ 5,636 ns / stdev ≈ 1,321 ns / p99.99 ≈ 17 µs.
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
  instead). Between blocks the harness sleeps a random 1-10 ms
  (fixed internal range, which re-rolls scheduler/frequency
  state) and warms up unrecorded (~2 ms); neither is counted
  in the budget. The report gains three lines (`mean blocks`
  (mean of the N block means), `CI95` (95% **c**onfidence
  **i**nterval half-width on it), and `LSC` (**l**east
  **s**ignificant **c**hange vs an equal-N run), and the
  header records `blocks=N`. Blocks nest above batches: each
  block is a contiguous stretch of whole batches (the flush
  aligns batch boundaries to the block gaps), so batches stay
  the grade block's time-series grain while blocks are the
  CI's replication grain. N is also the statistical
  replication count: more blocks -> tighter CI but shorter
  blocks. Interpretation: an honest *within-invocation* error
  bar; treat it as a lower bound on cross-invocation
  confidence and pin the bench (`--pin`); unpinned,
  per-process thread placement dominates and blocks can't see
  it. Bench-driven benches only; probe benches ignore it. See
  [validation](notes/design.md#block-validation-results-0210-4-r5-7600x)
  and the
  [design](notes/design.md#within-invocation-replication-sleep-separated-blocks).
- `--no-env-probe`: stop probing the environment at batch
  seams, limiting the `env` grade to the warmup probes (the few
  ms before the bench starts) instead of the whole run. Seam
  probing perturbs a spinning multi-threaded bench by ~0.9%
  (measured on `zcr-with-2t`; ~0.5% on a single-threaded one),
  which is common-mode in an A/B between benches but not in an
  absolute number. See [The two grades](#the-two-grades).
- `--settle-time SECONDS`: seconds the **first** bench of a
  process spends warming the box before it records anything
  (default `1.5`, or the config `settle_time`). `0` skips the
  warm. Paid once per process, since later benches inherit the
  machine state it wins; the grade block's `settle` cell reports
  how long the box actually took. See [Settle time](#settle-time).
- `--no-inhibit`: do not inhibit system sleep for the run. By
  default the process re-execs itself under
  `systemd-inhibit --what=sleep` so an idle-suspend can't poison a
  long measurement (a mid-sample suspend inflates that sample by
  the whole sleep gap; see the `WARNING` lines below). Where
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
- `--completions SHELL`: print a shell-completion artifact to
  stdout and exit; see [Shell completion](#shell-completion).
- `--list-benches`: print the registered bench names, one per
  line, and exit. Machine-readable: the carapace spec's
  exec-macro calls it on every Tab for dynamic bench-name
  candidates, and scripts can iterate it
  (`for b in $(iiac-perf --list-benches); ...`). The `all` /
  `add-completion-yaml` command words are not
  bench names and aren't listed.
- `--completion-dir DIR`: where `add-completion-yaml` writes
  `iiac-perf.yaml`; defaults to `$XDG_CONFIG_HOME/carapace/specs`
  (`~/.config` fallback), carapace's own spec lookup dir.

### Shell completion

`--completions SHELL` generates completion for the flags and
commands above. Two kinds of artifact, one flag:

- **Static scripts** (`bash`, `zsh`, `fish`, `elvish`,
  `powershell`), classic per-shell completion files, no extra
  tooling. Install by writing to your shell's completion dir,
  e.g.:

  ```
  iiac-perf --completions bash \
    > ~/.local/share/bash-completion/completions/iiac-perf
  iiac-perf --completions fish \
    > ~/.config/fish/completions/iiac-perf.fish
  ```

  (zsh: any directory on `$fpath`, named `_iiac-perf`.)
- **carapace spec** (`carapace`): one YAML spec for the
  [carapace-bin](https://github.com/carapace-sh/carapace-bin)
  multi-shell engine, which serves every shell it supports from
  that single file. Self-installs:

  ```
  iiac-perf add-completion-yaml
  ```

  writes the spec to the specs dir (`--completion-dir`, default
  `$XDG_CONFIG_HOME/carapace/specs` with `~/.config` fallback,
  carapace's own lookup), creating the dir and overwriting any
  previous spec; the no-args bench listing nudges toward this
  until the spec exists. `--completions carapace` still prints
  the same spec to stdout for a manual redirect.

  Why a command instead of a redirect: the spec only works if
  it lands in a dir carapace actually reads, under the right
  filename. The command owns that path logic, so setup is one
  word with nothing to copy-paste or get subtly wrong. When to
  run it:

  - once after installing iiac-perf (carapace-bin must already
    be hooked into your shell);
  - again after an upgrade that changes the CLI surface:
    flags and command words are a snapshot in the spec;
  - never for new benches, since names are queried live from the
    installed binary on every Tab.

  Unlike the static scripts, the spec completes **bench names
  dynamically**, queried from the installed binary at
  completion time: its exec-macro runs `iiac-perf
  --list-benches` on every Tab, so `iiac-perf ice<TAB>` offers
  the `ice-*` benches and the list stays current as benches
  are added, with no regeneration needed. The `all` /
  `add-completion-yaml` command words complete alongside, with
  descriptions.

Regenerate the artifact after an upgrade that changes the CLI
surface (flags are a snapshot in both kinds; for carapace just
re-run `iiac-perf add-completion-yaml`); the carapace spec's
bench names are the exception: they come from the installed
binary at completion time.

### Setup banner

Every run opens with a `Setup:` block: the TSC tick rate, the pin
used for the startup tick-rate warm, the bench pinning plan, the
sleep-inhibit state, and which config files were loaded. It is
provenance for the numbers below it, not measurement.

The apparatus cost that used to be measured and subtracted here
is now handled by construction instead. A micro-probe times
back-to-back timer pairs at startup and sizes the inner loop so
framing is a small fraction of the workload's per-call cost; the
cost is never named as a number and never removed from a sample.
See
[in-interval vs call-to-call](notes/design.md#timer-overhead-in-interval-vs-call-to-call)
for why the in-interval slice and the call-to-call cost are
different quantities, and why only the latter is worth measuring
for sizing.

A sub-quantum phase dither still runs between bench samples at
the seam, so a run's aggregate means do not inherit a coherent
bias from where samples happen to land on the clock lattice
([dithering](notes/design.md#dithering-random-phase-injection)).
Per-call costs are machine- and frequency-regime-specific: see
[Frequency dependence](notes/design.md#frequency-dependence-what-is-constant-what-is-not).
To decide whether a difference between two implementations is
real (and how many runs that takes), see
[Comparing implementations: LSC](notes/design.md#comparing-implementations-least-significant-change).

Steadiness is graded per run rather than at startup, from the
run's own data, and prints at the foot of each report. See
[The run grade's signals](#the-run-grades-signals).

### Config file

Defaults and named pin profiles can live in a TOML config file, so
common invocations don't repeat flags. Precedence, lowest to
highest:

- **built-in defaults**: `duration=5.0`, `band_labels=both`,
  `decimals=1`, `settle_time=1.5`;
- **XDG file**: `$XDG_CONFIG_HOME/iiac-perf/config.toml`, or
  `$HOME/.config/iiac-perf/config.toml` when `XDG_CONFIG_HOME` is
  unset; the per-user home for defaults and profiles;
- **project-local file**: `iiac-perf.toml` in the current
  directory (no upward walk); overrides the XDG file field by
  field, profiles merging by key;
- **CLI flags**: always win.

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
settle_time  = 3.0      # default --settle-time seconds; 0 skips the warm

[profiles]              # named --pin core specs
smt = "0,12"           # SMT siblings of one physical core (contention)
ccx = "0,1"            # independent cores, same CCX (best channel latency)
ccd = "0,6"            # cross-CCD
```

Each bench prints a band-based histogram in nanoseconds. Each row
is one band, labeled by its **upper boundary**, the lower boundary
being the previous printed row. Bands are **right-closed**
`(lower, upper]` (like `pandas.cut`): a sample whose rank lands
exactly on a boundary counts in the band that boundary *caps*, so a
lone median sample reads `p50`, matching the upper-boundary label
and the CDF sense of a percentile. Labels are deciles in the body
(`p10` ... `p90`) and **nines/zeros** notation in both tails, where
`nK`/`zK`
mark the boundary with a fraction 10<sup>-K</sup> of samples above
(`n`) or below (`z`) it, so `n2` ≡ p99, `n3` ≡ p99.9, ... `n10`,
and `z2` ≡ p1, `z3` ≡ p0.1, `z4`. "K nines" is standard
engineering shorthand for proportions near one
([Nines (notation)](https://en.wikipedia.org/wiki/Nines_%28notation%29),
nines = −log₁₀(1−x)); `zK` is this project's mirror of it for the
fast tail (the underlying concept is the
[survival function](https://en.wikipedia.org/wiki/Survival_function)
/ CCDF tail fraction). The slow tail subdivides down to `n10`, the
fast tail only to `z4`, since a latency distribution is floored below
(nothing beats the fast path) and open above. A band only prints
when it has samples, so deep tail rows appear as run length earns
them (populating `n10` takes ~1e10 calls). Each row shows first,
last, range (`last - first + 1`), count, and mean.
The trimmed `mean`/`stdev` rows exclude every band at or above
`n2` (p99); their label names the populated non-tail span (e.g.
`mean z4..n2`, or `p20..n2` when the low tail is empty), so it
tracks the rows that are actually present rather than a fixed
`min..n2`: `min` is never a row (rows are named by their upper
boundary) and the `n2` band can itself be empty.

The full boundary ladder across its range (label styles per
`--band-labels`). The ladder is generated by
[`src/bands.rs`](src/bands.rs), the single source of truth for
boundaries and labels, and this table is pinned by that module's
unit test, so code and docs can't silently drift:

| zpn       | frac              | ≡ percentile    | tail fraction |
|-----------|-------------------|-----------------|---------------|
| `z4`      | `0.000_1`         | p0.01           | 1e-4 below    |
| `z3`      | `0.001`           | p0.1            | 1e-3 below    |
| `z2`      | `0.01`            | p1              | 1e-2 below    |
| `p10`-`p90` | `0.10`-`0.90`   | deciles         | n/a           |
| `n2`      | `0.99`            | p99             | 1e-2 above    |
| `n3`      | `0.999`           | p99.9           | 1e-3 above    |
| `n4`      | `0.999_9`         | p99.99          | 1e-4 above    |
| `n5`      | `0.999_99`        | p99.999         | 1e-5 above    |
| `n6`      | `0.999_999`       | p99.9999        | 1e-6 above    |
| `n7`      | `0.999_999_9`     | p99.99999       | 1e-7 above    |
| `n8`      | `0.999_999_99`    | p99.999999      | 1e-8 above    |
| `n9`      | `0.999_999_999`   | p99.9999999     | 1e-9 above    |
| `n10`     | `0.999_999_999_9` | p99.99999999    | 1e-10 above   |

Every column is raw. The apparatus cost is managed by sizing
rather than by subtraction: a startup micro-probe times
back-to-back timer pairs and `inner` is chosen so framing is a
small fraction of the workload's per-call cost, which leaves a
residue small enough to ignore and, more to the point, common to
both sides of any same-harness comparison. A dither still runs
between bench samples at the seam, so aggregate means carry no
coherent phase bias. See
[design.md](notes/design.md#dithering-random-phase-injection).
The `Setup:` banner reports the `main pin` (main's placement,
covering the warm loop and thread 0 of every bench) and
`bench pin` (per-bench thread pool) separately.

Runs inhibit system sleep by default (see `--no-inhibit`), so the
flags below mainly matter for uninhibited runs. A report may end
with `WARNING` lines (printed last so they can't scroll out of
mind) flagging that `max` and the untrimmed mean/stdev are
poisoned. The few inflated samples land in the
extreme tail band, so percentile boundaries, the bands below the
tail, and the trimmed `mean`/`stdev` rows remain usable:

- **system suspended**: the run spanned a system suspend,
  detected by `CLOCK_BOOTTIME` vs `CLOCK_MONOTONIC` elapsed
  divergence. A mid-sample suspend inflates that one sample by
  the whole sleep gap.
- **sample(s) clamped**: a sample exceeded the histogram's 60 s
  bound and was recorded as 60 s instead of aborting the run
  (visible as a pileup at `max`).

### The two grades

Every report ends with the grade block: one column header over
three rows, each graded A-F from its own data: two `env` rows
for the **box**, one `run` row for *that run*. A row's `worst`
column is its composite, printed beside the signals that earned
it; a blank cell (`-`) means that signal does not apply to that
row, which is the env/run signal mapping made visible:

```
  grade  phase        settle  worst     spread  bursts  interference      drift               step
  env    warmup        0.86s      A    0.47% A       -       0.00% A    0.00% A            0.00% A
  env    bench             -      F    0.48% A       -       0.00% A   11.05% F    11.05% @1.06s F
  run    all               -      F          -   37% B       0.04% A   10.49% F    10.49% @1.04s F
```

Column reference (each signal prints its own letter beside its
value; the sections below carry the depth):

- `grade` / `phase`: row labels. The two `env` rows grade the
  box from micro-probes that never touch the bench (`warmup`:
  did it end settled; `bench`: did it stay settled). The `run`
  row grades the numbers above it, from the run's own batches.
- `settle`: warmup row only. How long the box took to settle,
  or `not settled`; see [Settle time](#settle-time).
- `worst`: the row's composite letter, its worst signal
  outright; always one of the letters printed beside it.
- `spread`: env rows only. How wide a probe's bulk sits above
  its own floor. A timer pair has no workload character, so
  width means the box itself moved.
- `bursts`: run row only. The fraction of batches whose mean
  sits above the run's median batch: whether interference was
  localized in time or spread out.
- `interference`: samples that sat above their batch's floor,
  as a fraction of the run: how much other work leaked in.
- `drift`: floor movement from the run's first quarter to its
  last: did the run finish where it started.
- `step`: the largest floor shift at any split of the run, and
  when (`10.49% @1.04s`): catches a shift-and-return that
  drift's endpoints miss.

The `env` rows are two phases of one probe series, scored
separately: `warmup` is the last 300 ms of the probes taken
before the bench ran ("did the box end settled"), `bench` the
probes taken alongside it ("did it stay settled"). They are
graded apart rather than as one series because absorbing a ramp
is exactly what warmup is *for*: blended, the boundary between a
cold warmup and a hot run reads as a large step that nothing
actually did wrong. The block prints no combined env letter:
each phase's `worst` is visible, and the worse of the two is
what `qualify-environment` computes for its verdict.

The rows answer different questions, and reading them together
says more than either alone. `run` describes the numbers above
it,
and a run's steadiness is largely its workload's character, so a
blocking round-trip reads worse than a spinning one, correctly.
`env` describes the machine: it comes from micro-probes that time
timer pairs and never touch the bench, so no workload character
enters it. An `env` A beside a `run` D means a bursty workload on
a quiet box. The same letter in both, at the same instant, means
the box moved and took the bench with it, as in the example
above: `min-now` on a 7600x, where the environment reports an
11.05% step at 1.06 s and the run reports a 10.49% step at
1.04 s. Same magnitude, same instant, from two series that share
a time axis but not an instrument. Neither grade could make that
call alone.

### Settle time

The warmup row's `settle` cell says how long the box took to
settle: `0.86s`, or `not settled` when it was still moving
when warmup ended. It exists because warmup now *absorbs* the
box coming up to speed rather than being graded on it: the first
run of a process spends `--settle-time` seconds (default 1.5)
stepping the bench before recording anything, so the letter
answers "was it settled when measurement started" and this
answers "how long did that take".

The warm is per **process**, not per bench: the boost it wins is
machine state, so every later bench in the same process inherits
it. Without it the first bench of a process reports a cold
machine's numbers (measured at ~8.6% slow on a 7600x, a wrong
histogram rather than merely a wrong letter) while benches 2..N
read correctly. Cost is ~2% of an `all -d 5` sweep.

Settle time is *when the floor entered, and stayed inside, ±1% of
the level warmup ended at*. Precisely: each probe's floor is its
p10 group cost; the settled level is the median floor over the
300 ms tail window; a running 8-probe median (a quarter of the
series when it is shorter than that, matching what `drift`
compares) has to sit inside the band; the reported time is the
probe after the last one that didn't; and that state has to hold
through the whole tail window, or the reading is `not settled`.

That last requirement is what keeps the number about the box
rather than about the budget. Settle is the last excursion's end,
so on a machine that keeps flickering the last excursion is near
the end of warmup *whenever warmup ends*: on a 3900X,
`--settle-time 1.5` reported a ~1.0 s median settle and
`--settle-time 5` a 4.63 s one, same box, same state. Demanding
that the state hold through the graded window makes "settled at
T" mean the box actually stayed there.

Two things it does not say: it is measured against where *this*
warmup ended, not any absolute best speed, so it never says which
state was the right one; and it is biased early by up to one
window, since the first window that reads settled straddles the
last of the ramp.

`--settle-time 0` skips the warm, which is how you measure what
it is worth on a given box. A box that reads `not settled` at
the default wants more, though that is not always curable: on a
3900X the floor is bistable and moves at arbitrary times, so a
3.5 s warm still left runs moving mid-bench. Replication
(`--blocks`) is the answer there, not a longer warm.

The probes run through warmup and then in the seam at every batch
boundary, so the series covers the whole run on the same time
axis as the batches. `--no-env-probe` limits them to warmup (so
only the warmup row appears),
which costs the grade its span; it exists because seam probing
perturbs a spinning multi-threaded bench by ~0.9% (measured on
`zcr-with-2t`), a bias that is common-mode in an A/B between two
benches but not in an absolute number.

The `env` signals differ slightly from the run's: `spread` (how
wide a probe's bulk sits above its own floor) replaces `bursts`,
because a bench's spread is mostly its workload while a timer
pair has no character of its own. Note that `env interference`
is the weakest of the four: a probe measures the box only while
the measuring thread is running, so preemption is largely
invisible to it and `spread`/`drift`/`step` carry the detection.

### The run grade's signals

- `interference`: samples that sat above their batch's floor, as
  a fraction of the run. How much other work leaked in.
- `bursts`: batches whose mean sits above the run's median batch.
  Whether that interference was localized in time or spread out.
- `drift`: floor movement from the run's first quarter to its
  last. Did the run finish where it started.
- `step`: the largest floor shift any split of the run divides,
  and when. Catches a shift that drift's endpoints miss: a run
  that moves and moves back reads low on `drift` and high on
  `step`.

**The overall letter is the worst signal, outright.** Each signal
scores 0-4 by counting how many of its four ascending cutoffs it
crosses: below all four is 0 = A, above all four is 4 = F (there
is no E). The composite is the maximum of those scores, so one F
anywhere makes the run F and no number of A's pulls it back;
`step` alone earns the F in the example above. That is why every
signal prints its own letter: a row's `worst` is always one of
the letters shown beside it.

The `env` rows work the same way, over their own four signals.
There were once six, when the grade was measured at startup from
a calibration fit: two of them scored how well that *fit* held
(the worst residual of a ladder point against the Theil-Sen line,
and the loop-only slope against a dithered two-point fit). A
bench run fits nothing, so those two have no run-side analog and
none was invented; the reasoning is recorded in
[chores-05.md](notes/chores/chores-05.md#six-calibration-signals-four-run-signals).

Both floor signals compare medians, not extremes, so one hot
batch is a burst rather than a shift.

**It reports; it does not warn.** A low letter is not a fault to
fix. A run's steadiness is largely its workload's character: a
multi-threaded bench carries OS involvement in its own numbers
(scheduling, placement, park/unpark) so on a quiet box `mpsc-2t`
reads `step` F while `mpsc-2t-spin`, the same round-trip spinning
instead of parking, reads A. Both letters are true descriptions.
The report's job is a histogram faithful to what was measured,
and the grade is part of that description rather than a verdict
on it.

Where it earns its keep is the comparison you came for: before
trusting a delta between two runs, check that neither of them
straddled a shift. Comparing the letter between runs of the same
bench is meaningful; comparing it across different benches is
not.

This is a different question from the `env` rows, which grade the
box rather than the run. Judging the box from a bench's own
samples is not possible after the fact, since they mix the two
inseparably; the `env` grade instead comes from micro-probes that
time timer pairs and never touch bench code, so no workload
character enters it.

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
iiac-perf mpsc-2t --pin 0,1 --blocks 10  # pinned + error bar (ci95/lsc lines)
iiac-perf mpsc-2t -v                     # show internals (affinity, warmup table)
RUST_LOG=info iiac-perf mpsc-2t          # info-level only (overrides -v)
```

## Example runs

Measurements below are on a Ryzen 9 3900X, idle desktop. Numbers
vary run-to-run and machine-to-machine; the *shape* of the
differences is the useful signal.

### Reading a report

Each row is one *populated* band (see the boundary ladder above);
empty bands are skipped. Columns:

- **first / last**: the smallest and largest sample *values* in the
  band; `first` of the top row is the fastest call observed.
- **range**: `last − first + 1`, the band's width.
- **count**: samples in the band.
- **mean**: the band's mean, raw. Nothing is subtracted (see
  [Setup banner](#setup-banner)).

Below the bands, `mean` / `stdev` are whole-histogram; the trimmed
`mean X..Y` / `stdev X..Y` drop the `≥ p99` tail so a few ms-scale
outliers don't poison them, and their label names the populated
non-tail span.

**How samples map to bands.** A sample's rank is its
[Hazen plotting position](https://splashback.io/2021/05/hazen-percentile/)
(Allen Hazen, 1914) `mid_rank = (i − 0.5) / n` (`i` = 1-based rank,
`n` = sample count). Bands are **right-closed** `(lower, upper]`, so the
`(` is *open* (excludes the lower boundary), the `]` is *closed*
(includes the upper), so a band holds the ranks
`band_lower < N ≤ band_upper`. A rank landing exactly on a boundary
therefore counts in the band that boundary *caps*. That's the
[`pandas.cut`](https://pandas.pydata.org/docs/reference/api/pandas.cut.html)
convention; computing's other default is left-closed `[lower, upper)`
([`numpy.histogram`](https://numpy.org/doc/stable/reference/generated/numpy.histogram.html),
language ranges,
[Dijkstra EWD831](https://www.cs.utexas.edu/~EWD/transcriptions/EWD08xx/EWD831.html)).
Right-closed matches this report's upper-boundary labels: "the `p50`
row" = samples *up to and including* the 50th percentile.

Ten distinct values (`n = 10`) spread one per band:

| value `i` | `mid_rank = (i−0.5)/10` | band  | interval `(lower, upper]`     |
|----------:|:-----------------------:|:------|:------------------------------|
| 1         | 0.05                    | `p10` | `(0.01, 0.10]` = `(z2, p10]`  |
| 2         | 0.15                    | `p20` | `(0.10, 0.20]`                |
| 3         | 0.25                    | `p30` | `(0.20, 0.30]`                |
| 4         | 0.35                    | `p40` | `(0.30, 0.40]`                |
| 5         | 0.45                    | `p50` | `(0.40, 0.50]`                |
| 6         | 0.55                    | `p60` | `(0.50, 0.60]`                |
| 7         | 0.65                    | `p70` | `(0.60, 0.70]`                |
| 8         | 0.75                    | `p80` | `(0.70, 0.80]`                |
| 9         | 0.85                    | `p90` | `(0.80, 0.90]`                |
| 10        | 0.95                    | `n2`  | `(0.90, 0.99]` = `(p90, n2]`  |

A **single sample** is the degenerate case (every percentile
collapses to that one value) and `mid_rank = (1 − 0.5)/1 = 0.5`
lands it in `p50` (since `0.40 < 0.50 ≤ 0.50`):

| `n` | `mid_rank` | band  |
|----:|:----------:|:------|
| 1   | 0.50       | `p50` |

**Investigating with `-d`.** Because membership is by rank, shrinking
the duration to force a known sample count is a handy way to watch
exactly where values land (the exact `-d` is machine-dependent; tune
it to the count you want; there are no timing guarantees):

```
$ iiac-perf zcr -d 0.000001        # a handful of samples
  p30 0.30       2.8 ns    2.8 ns    0.0 ns    2    2.8 ns      2.0 ns
  p70 0.70       3.0 ns    3.0 ns    0.0 ns    1    3.0 ns      2.3 ns
  p90 0.90       4.2 ns    4.2 ns    0.0 ns    1    4.2 ns      3.4 ns
  mean p30..p90                                     3.2 ns      2.4 ns

$ iiac-perf zcr -d 0.0000001       # one sample -> collapses to p50
  p50 0.50       6.3 ns    6.3 ns    0.0 ns    1    6.3 ns      5.5 ns
  mean p50                                          6.3 ns      5.5 ns
```

### Comparing two implementations (`--blocks`)

"Is B really faster than A, or is it noise?" The workflow:

```
iiac-perf mpsc-2t --pin 0,1 --blocks 10 -d 10
```

`--blocks 10 -d 10` divides the 10-second measuring budget
into **10 blocks of ~1 s each**: same total measurement, now
with an error bar, because each block acts as a mini-run
(random 1-10 ms sleep, unrecorded warm-up, then its share of
the budget). Always pin (`--pin`): unpinned, the OS's thread
placement is re-rolled per *process* and dominates run-to-run
drift, which blocks can't see. The report then ends with:

```
  mean blocks                          4,745.953 ns
  CI95                                    16.115 ns
  LSC                                     21.169 ns
```

- **mean blocks**: the run's headline number: the mean of the
  10 block means.
- **CI95**: 95% confidence interval (half-width) on that
  mean: "the true value is within ±16 ns of 4,746, as far as
  this run can tell."
- **LSC**: least significant change: run the *other*
  implementation the same way (same `-d`, same `--blocks`,
  same pin), and if the two `mean blocks` differ by more than
  roughly the larger of the two `LSC`s, the difference is
  real at 95% confidence.

Caveat: this error bar sees *within-invocation* variation
only. Some per-process state survives the sleeps (measured
~0.6% residual drift even pinned, on an idle Ryzen 5 7600X),
so treat `LSC` as the lower bound; for a decision that
matters, run each implementation 3-5 times interleaved
(A,B,A,B,...) and apply the same comparison to the per-run
`mean blocks` values. Method and worked numbers:
[Comparing implementations](notes/design.md#comparing-implementations-least-significant-change),
[block validation](notes/design.md#block-validation-results-0210-4-r5-7600x).

### Label styles (`--band-labels`)

`--band-labels` selects the row-label vocabulary; the trimmed
`mean`/`stdev` rows and the report header's `labels=` metadata
follow the same style. The trimmed label names the **populated**
non-tail span, and here `min` is never a row (no samples land in the
fast tail), so it reads `p50..n2`, not a fixed `min..n2`. Default
`both` prints the zpn name and its literal fraction side by side
(the juxtaposition teaches the zpn vocabulary):

```
$ iiac-perf min-now -d 1 --band-labels both
minstant::Instant::now() [duration=1.0s outer=1,539,764 inner=23 calls=35,414,572 batches=24 labels=both]:
                       first          last         range        count          mean
  p50 0.50           24.0 ns       24.0 ns        0.0 ns    1,303,881       24.0 ns
  p90 0.90           24.0 ns       24.0 ns        0.0 ns       44,597       24.0 ns
  n2  0.99           24.4 ns       28.8 ns        4.4 ns      175,893       24.6 ns
  ...
  n7  0.999_999_9 2,170.9 ns    2,814.0 ns      643.1 ns            2    2,492.4 ns
  mean                                                                      24.2 ns
  stdev                                                                      7.9 ns
  mean p50..n2                                                              24.0 ns
  stdev p50..n2                                                              0.3 ns
  grade  phase        settle  worst     spread  bursts  interference      drift               step
  env    warmup        0.09s      A    0.30% A       -       0.00% A    0.00% A            0.00% A
  env    bench             -      A    0.30% A       -       0.01% A    0.00% A            0.00% A
  run    all               -      A          -    0% A       0.06% A    0.00% A            0.00% A
```

`zpn` drops the fraction (names only); `frac` drops the name
(fractions only, so the trimmed label reads `0.50..0.99`). Same
bench, separate runs; only the leftmost column and the trim
label change:

```
$ iiac-perf min-now -d 1 --band-labels zpn        $ iiac-perf min-now -d 1 --band-labels frac
  ... labels=zpn]:                                   ... labels=frac]:
  p50    ...                                         0.50      ...
  n2     ...                                         0.99      ...
  ...                                                ...
  mean p50..n2     24.0 ns                           mean 0.50..0.99     24.1 ns
  stdev p50..n2     0.3 ns                           stdev 0.50..0.99     0.3 ns
```

### `all` results (3900X, 0.23.0-7)

One `iiac-perf all -d 2` run (unpinned, idle desktop), whole-run
mean per bench; the probe-only benches have no bench-level mean
and report their producer probe's mean instead. Raw values, so
each includes the apparatus cost described in
[Setup banner](#setup-banner). Same caveat as above: shapes, not
absolutes.

| bench             |      mean | note                          |
|-------------------|----------:|-------------------------------|
| min-now           |   24.1 ns | `minstant::Instant::now`      |
| std-now           |   22.9 ns | `std::time::Instant::now`     |
| mpsc-1t           |   33.3 ns | same-thread channel           |
| mpsc-2t           | 8,058.6 ns | blocking `recv` (park/wake)  |
| mpsc-2t-spin      |  206.0 ns | spin `try_recv`               |
| probe-mpsc-2t     | 7,513.0 ns | same, with probes            |
| producer-consumer | 7,443.2 ns | probe-only                   |
| tp-pc             | 7,663.0 ns | TProbe tick-only             |
| tp2-pc            | 8,060.6 ns | TProbe2 scope API            |
| ice-ps-1t         |  287.0 ns | iceoryx2 pub/sub, 1 thread    |
| ice-ps-2t         |  738.1 ns | iceoryx2 pub/sub, 2 threads   |
| ice-rr-1t         |  744.2 ns | iceoryx2 req/res, 1 thread    |
| ice-rr-2t         | 1,230.1 ns | iceoryx2 req/res, 2 threads  |
| zcr-with-1t       |    5.2 ns | zc-ring-x1 `_with`, 1 thread  |
| zcr-with-2t       |  183.2 ns | zc-ring-x1 `_with`, 2t, spin  |
| zcr-mpsc-1t       |    5.4 ns | zc-ring-x1 mpsc, 1 thread     |
| zcr-mpsc-2t       |  127.7 ns | zc-ring-x1 mpsc, 2t, spin     |

The wait-policy split dominates the 2-thread rows: the parking
benches (`mpsc-2t` and the probe family, all blocking `recv`)
cluster at ~7.4-8.1 µs while the spinning benches sit under
1.3 µs. For context, iceoryx2's own pub/sub benchmark (v0.9.2,
`--bench-all`) on this machine reports 250 ns one-way (~500 ns
round-trip) with pinned realtime threads and untouched payloads,
consistent with `ice-ps-2t`'s 738 ns measured here. The zcr rows
are the in-process zc-ring-x1 SPSC ring: 1t rounds trip in ~5 ns
(two cache-hot atomics) through the `reserve_slot_with` claim;
see notes/chores/chores-04.md for the pinned tier comparison of
the former raw/spin/with API tiers.

### Verbose output (`-v`)

`-v` prints the affinity lifecycle on stderr. Main pins only
when `--pin` is given (to the pool's first slot, where it warms
and measures); otherwise every mask stays as the process
launched.

```
$ iiac-perf mpsc-2t -d 3 -v
iiac-perf 0.23.0-7 — Rust latency microbenchmark harness

[INFO  iiac_perf] startup affinity: 0-23 (24 cpus)
[DEBUG iiac_perf] affinity for warm + run: 0-23 (24 cpus)
[DEBUG iiac_perf] ticks_per_ns: 3.792852
Setup:
  ticks/ns          3.792852
  main pin          none (scheduler placement)
  bench pin         none (unpinned)
  sleep inhibit     active (systemd-inhibit --what=sleep)
  config            none (built-in defaults)

std::sync::mpsc round-trip (2 threads) [duration=3.0s outer=363,598 inner=1 calls=363,598 batches=55 labels=both]:
                         first              last             range     count              mean
  z4  0.000_1         391.2 ns          401.2 ns           10.0 ns        15          400.1 ns
  z3  0.001           410.1 ns          411.1 ns            1.0 ns       409          410.9 ns
  z2  0.01            420.1 ns        6,361.1 ns        5,941.0 ns     3,215        1,133.2 ns
  p10 0.10          6,365.2 ns        6,656.0 ns          290.8 ns    35,233        6,596.7 ns
  ...
  p90 0.90          9,199.6 ns        9,412.6 ns          213.0 ns    35,618        9,298.0 ns
  n2  0.99          9,420.8 ns       11,403.3 ns        1,982.5 ns    32,814        9,793.5 ns
  n3  0.999        11,411.5 ns       16,662.5 ns        5,251.1 ns     3,272       13,153.5 ns
  n4  0.999_9      16,678.9 ns       91,160.6 ns       74,481.7 ns       329       25,497.9 ns
  n5  0.999_99     93,388.8 ns    1,265,631.2 ns    1,172,242.4 ns        32      383,158.3 ns
  n6  0.999_999 1,266,679.8 ns    1,782,579.2 ns      515,899.4 ns         4    1,443,364.9 ns
  mean                                                                              8,089.7 ns
  stdev                                                                             6,804.9 ns
  mean z4..n2                                                                       7,981.4 ns
  stdev z4..n2                                                                      1,296.2 ns
  grade  phase        settle  worst     spread  bursts  interference      drift               step
  env    warmup        0.84s      B    2.10% B       -       0.02% A    0.00% A            0.00% A
  env    bench             -      A    0.33% A       -       0.00% A    0.00% A            0.00% A
  run    all               -      F          -   36% B       5.04% C   10.67% F    25.20% @0.58s F
```

Notice `z4 first = 391 ns`, sub-µs. That's the
"both-ends-hot-and-spinning" fast path, where the scheduler has
co-located bench threads on the same CCX and neither has parked
in a futex. It survives because an unpinned run never pins main,
so the scheduler keeps its placement freedom.

### Default vs `--pin 0,1`

Default (unpinned bench): wide dispersion, but the fast path is
visible.

```
$ iiac-perf mpsc-2t -d 3
Setup:
  ...
  main pin          none (scheduler placement)
  bench pin         none (unpinned)

std::sync::mpsc round-trip (2 threads) [duration=3.0s outer=363,056 inner=1 calls=363,056 batches=55 labels=both]:
  z4  0.000_1         240.1 ns          400.1 ns          160.0 ns        29          374.3 ns
  ...
  n2  0.99          9,363.5 ns       11,255.8 ns        1,892.4 ns    32,539        9,738.6 ns
  n6  0.999_999 1,460,666.4 ns    1,771,044.9 ns      310,378.5 ns         4    1,566,048.3 ns
  mean                                                                              8,104.5 ns
  stdev                                                                             6,693.1 ns
  mean z4..n2                                                                       8,000.6 ns
  stdev z4..n2                                                                      1,312.5 ns
  grade  phase        settle  worst     spread  bursts  interference      drift               step
  env    warmup            -      A    0.33% A       -       0.01% A    0.00% A            0.00% A
  env    bench             -      F    0.33% A       -       0.01% A    0.00% A    12.62% @3.01s F
  run    all               -      F          -   40% B       3.19% B    0.58% A    12.19% @1.16s F
```

Pinned to two physical cores in the same CCX: tighter body, lower
mean.

```
$ iiac-perf mpsc-2t --pin 0,1 -d 3
Setup:
  ...
  main pin          core 0 (pool slot 0; warm + run)
  bench pin         [0, 1] (2 slots, 2 unique CPUs)

std::sync::mpsc round-trip (2 threads) [duration=3.0s outer=417,477 inner=1 calls=417,477 batches=55 labels=both]:
  z4  0.000_1         391.2 ns          470.0 ns           78.8 ns        42          421.2 ns
  ...
  n2  0.99          7,487.5 ns        9,027.6 ns        1,540.1 ns    37,406        7,864.7 ns
  n6  0.999_999 2,929,721.3 ns    3,066,036.2 ns      136,314.9 ns         4    2,988,441.6 ns
  mean                                                                              7,039.8 ns
  stdev                                                                            13,632.2 ns
  mean z4..n2                                                                       6,897.4 ns
  stdev z4..n2                                                                        511.3 ns
  grade  phase        settle  worst     spread  bursts  interference      drift               step
  env    warmup            -      A    0.26% A       -       0.01% A    0.00% A     0.53% @0.07s A
  env    bench             -      F    0.36% A       -       0.01% A   19.12% F    19.12% @1.49s F
  run    all               -      D          -   29% B       0.68% A    9.83% D     9.53% @0.78s D
```

Side-by-side (using the trimmed `z4..n2` rows, which exclude the
ms-scale OS-preemption outliers in the `n3`-`n6` tail bands):

| metric          | default    | `--pin 0,1` | Δ      |
|-----------------|-----------:|------------:|-------:|
| `mean z4..n2`   |   8,001 ns |    6,897 ns | −14 %  |
| `stdev z4..n2`  |   1,313 ns |      511 ns | −61 %  |
| `stdev` untrimmed |  6,693 ns |   13,632 ns | +104 % |

So `--pin 0,1` buys a tighter, lower-mean body at the cost of
being more exposed to a rare preemption: bound to one core, a
single outlier pushes the max to ms-scale, which is why the
untrimmed `stdev` moves the *wrong* way. Use the
`mean/stdev z4..n2` rows for representative central tendency and
spread.

Both runs kept the sub-µs `z4` fast path, where the scheduler has
co-located the threads and neither end has parked. Do not read
the `z4 first` difference between these two runs as an effect of
pinning: that column is the extreme of a sparse tail and moves
run to run by more than the gap between them.


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

[1]: https://github.com/karpathy/autoresearch
