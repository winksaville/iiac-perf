# iiac-perf

A general-purpose latency microbenchmark harness for Rust. Each
bench runs against a wall-clock time budget with auto-sized loop
counts, reports a percentile-band histogram in nanoseconds, and
subtracts calibrated apparatus overhead (the amortized
loop-per-iter cost plus the dither-measured in-interval slice of
the timer pair; the full call-to-call timer cost instead sizes
the inner loop) so the output reflects the workload, not the
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
iiac-perf calibrate
```

`BENCH` is one or more registered bench names, or `all` for every
registered bench. A name that matches no bench exactly runs every
bench it is a prefix of — `ice` runs all iceoryx2 benches, `mpsc`
runs `mpsc-1t` and `mpsc-2t`. **With no arguments, `iiac-perf` prints the
available list and exits — that's the source of truth for which
benches the current build registers.**

`iiac-perf calibrate` runs the startup calibration only — no
bench — and prints the
[Calibration banner](#calibration-banner) plus the raw fit
inputs: the dithered points, the three alternative fits, the
TSC tick rate, and the calibration wall time. Use it to
fingerprint a machine's frequency regime or check constant
drift without spending a bench run. The word must stand alone
(no bench names alongside); `--pin`, `--no-pin-cal`, and `-v`
apply as usual.

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
  startup, calibration parameters, the raw dithered points and
  window value, the three alternative fits, calibration wall
  time). Equivalent to `RUST_LOG=debug`. Default filter is
  `warn` — silent unless something is wrong. `RUST_LOG`, when
  set, wins over `-v` so per-module filtering still works.
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
- `--blocks N` — N (2–1000) is the **number of measurement
  blocks** the run's budget is divided into: `--blocks 10`
  with `-d 10` measures 10 blocks of ~1 s each (total measured
  time still 10 s; with `-o` the sample count is divided
  instead). Between blocks the harness sleeps a random 1–10 ms
  (fixed internal range — it re-rolls scheduler/frequency
  state) and warms up unrecorded (~2 ms); neither is counted
  in the budget. The report gains three lines — `mean blocks`
  (mean of the N block means), `CI95` (95% **c**onfidence
  **i**nterval half-width on it), and `LSC` (**l**east
  **s**ignificant **c**hange vs an equal-N run) — and the
  header records `blocks=N`. N is also the statistical
  replication count: more blocks → tighter CI but shorter
  blocks. Interpretation: an honest *within-invocation* error
  bar; treat it as a lower bound on cross-invocation
  confidence and pin the bench (`--pin`) — unpinned,
  per-process thread placement dominates and blocks can't see
  it. Bench-driven benches only; probe benches ignore it. See
  [validation](notes/design.md#block-validation-results-0210-4-r5-7600x)
  and the
  [design](notes/design.md#within-invocation-replication-sleep-separated-blocks).
- `--no-inhibit` — do not inhibit system sleep for the run. By
  default the process re-execs itself under
  `systemd-inhibit --what=sleep` so an idle-suspend can't poison a
  long measurement (a mid-sample suspend inflates that sample by
  the whole sleep gap; see the `WARNING` lines below). Where
  `systemd-inhibit` is unavailable — absent, or the lock is
  denied (e.g. a headless ssh session with no polkit
  interactive auth) — the run continues uninhibited and the
  banner's `sleep inhibit` line says so. Pass this flag to
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
- `--completions SHELL` — print a shell-completion artifact to
  stdout and exit; see [Shell completion](#shell-completion).

### Shell completion

`--completions SHELL` generates completion for the flags and
commands above. Two kinds of artifact, one flag:

- **Static scripts** (`bash`, `zsh`, `fish`, `elvish`,
  `powershell`) — classic per-shell completion files, no extra
  tooling. Install by writing to your shell's completion dir,
  e.g.:

  ```
  iiac-perf --completions bash \
    > ~/.local/share/bash-completion/completions/iiac-perf
  iiac-perf --completions fish \
    > ~/.config/fish/completions/iiac-perf.fish
  ```

  (zsh: any directory on `$fpath`, named `_iiac-perf`.)
- **carapace spec** (`carapace`) — one YAML spec for the
  [carapace-bin](https://github.com/carapace-sh/carapace-bin)
  multi-shell engine, which serves every shell it supports from
  that single file:

  ```
  iiac-perf --completions carapace \
    > ~/.config/carapace/specs/iiac-perf.yaml
  ```

Regenerate after upgrading iiac-perf — the artifact is a
snapshot of the CLI, not live. Bench *names* aren't completed
yet (the positional accepts free-form prefixes); the carapace
spec format could add that later via an exec-macro.

### Calibration banner

Every run calibrates apparatus overhead at startup and prints
three constants at 3 decimals (the dithered measurement resolves
well below a nanosecond):

- `frame/call` — the full call-to-call cost of taking one
  sample, both clock-read latencies included. It sizes the
  inner loop (`inner ≈ 10 × frame/call ÷ step cost`) and is
  never subtracted — most of it falls outside the recorded
  interval. See
  [in-interval vs call-to-call](notes/design.md#timer-overhead-in-interval-vs-call-to-call).
- `frame/sample` — the in-interval slice: what the timer pair
  actually adds *inside* a recorded sample. Measured by a
  dithered two-point fit — random sub-quantum delays before
  each calibration sample turn the ~10 ns clock quantization
  into zero-mean noise that averages away
  ([dithering](notes/design.md#dithering-random-phase-injection)).
  Subtracted from reported values, amortized by `inner`.
  Repeats to ~±0.1 ns within a CPU frequency regime
  ([validation](notes/design.md#dither-validation-results-0210-2-r5-7600x)).
- `loop/iter` — per-iteration loop overhead (branch +
  `black_box`), the fit's slope; subtracted per call. Repeats
  to 5 significant figures within a regime, so it doubles as
  a frequency-regime fingerprint.

The same dither runs between bench samples (the seam), so a
run's aggregate means don't inherit a coherent phase bias. All
three constants are machine- and frequency-regime-specific —
see
[Frequency dependence](notes/design.md#frequency-dependence-what-is-constant-what-is-not).
To decide whether a difference between two implementations is
real (and how many runs that takes), see
[Comparing implementations: LSC](notes/design.md#comparing-implementations-least-significant-change).
The standalone `iiac-perf calibrate` command prints this banner
plus the raw fit inputs without running a bench.

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
being the previous printed row. Bands are **right-closed**
`(lower, upper]` (like `pandas.cut`): a sample whose rank lands
exactly on a boundary counts in the band that boundary *caps*, so a
lone median sample reads `p50`, matching the upper-boundary label
and the CDF sense of a percentile. Labels are deciles in the body
(`p10` … `p90`) and **nines/zeros** notation in both tails, where
`nK`/`zK`
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
The trimmed `mean`/`stdev` rows exclude every band at or above
`n2` (p99); their label names the populated non-tail span (e.g.
`mean z4..n2`, or `p20..n2` when the low tail is empty), so it
tracks the rows that are actually present rather than a fixed
`min..n2` — `min` is never a row (rows are named by their upper
boundary) and the `n2` band can itself be empty.

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

The adjusted column subtracts apparatus overhead:
`frame_sample/inner + loop_per_iter`, both from a dithered
two-point fit calibrated at startup — a random sub-quantum delay
before each calibration sample turns the ~10 ns clock-quantum
error into zero-mean noise that averages away, making the
in-interval timer slice (`frame/sample` in the banner)
measurable to ~±0.1 ns. The full call-to-call timer cost
(`frame/call`) is never subtracted — most of it falls outside
recorded intervals — but sizes the inner loop. The same dither
runs between bench samples (the seam), so a run's aggregate
means don't carry a coherent phase bias. See
[design.md](notes/design.md#dithering-random-phase-injection).
The startup banner reports `cal pin` (calibration pinning) and
`bench pin` (per-bench thread pool) separately.

Runs inhibit system sleep by default (see `--no-inhibit`), so the
flags below mainly matter for uninhibited runs. A report may end
with `WARNING` lines (printed last so they can't scroll out of
mind) flagging that `max` and the untrimmed mean/stdev are
poisoned. The few inflated samples land in the
extreme tail band, so percentile boundaries, the bands below the
tail, and the trimmed `mean`/`stdev` rows remain usable:

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
iiac-perf mpsc-2t --pin 0,1 --blocks 10  # pinned + error bar (ci95/lsc lines)
iiac-perf mpsc-2t -v                     # show cal internals (affinity, raw fit)
iiac-perf mpsc-2t --no-pin-cal           # skip the cal-time pin (bench unchanged)
iiac-perf mpsc-2t --pin 5,10 --no-pin-cal # bench on 5,10; cal on full mask
RUST_LOG=info iiac-perf mpsc-2t          # info-level only (overrides -v)
```

## Example runs

Measurements below are on a Ryzen 9 3900X, idle desktop. Numbers
vary run-to-run and machine-to-machine; the *shape* of the
differences is the useful signal.

### Reading a report

Each row is one *populated* band (see the boundary ladder above);
empty bands are skipped. Columns:

- **first / last** — the smallest and largest sample *values* in the
  band; `first` of the top row is the fastest call observed.
- **range** — `last − first + 1`, the band's width.
- **count** — samples in the band.
- **mean / adjusted** — the band's mean, and that mean minus
  calibrated apparatus overhead (`frame_sample/inner +
  loop_per_iter`; see
  [Calibration banner](#calibration-banner)).

Below the bands, `mean` / `stdev` are whole-histogram; the trimmed
`mean X..Y` / `stdev X..Y` drop the `≥ p99` tail so a few ms-scale
outliers don't poison them, and their label names the populated
non-tail span.

**How samples map to bands.** A sample's rank is its
[Hazen plotting position](https://splashback.io/2021/05/hazen-percentile/)
(Allen Hazen, 1914) `mid_rank = (i − 0.5) / n` (`i` = 1-based rank,
`n` = sample count). Bands are **right-closed** `(lower, upper]` — the
`(` is *open* (excludes the lower boundary), the `]` is *closed*
(includes the upper), so a band holds the ranks
`band_lower < N ≤ band_upper`. A rank landing exactly on a boundary
therefore counts in the band that boundary *caps*. That's the
[`pandas.cut`](https://pandas.pydata.org/docs/reference/api/pandas.cut.html)
convention; computing's other default is left-closed `[lower, upper)`
([`numpy.histogram`](https://numpy.org/doc/stable/reference/generated/numpy.histogram.html),
language ranges,
[Dijkstra EWD831](https://www.cs.utexas.edu/~EWD/transcriptions/EWD08xx/EWD831.html)).
Right-closed matches this report's upper-boundary labels — "the `p50`
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

A **single sample** is the degenerate case — every percentile
collapses to that one value — and `mid_rank = (1 − 0.5)/1 = 0.5`
lands it in `p50` (since `0.40 < 0.50 ≤ 0.50`):

| `n` | `mid_rank` | band  |
|----:|:----------:|:------|
| 1   | 0.50       | `p50` |

**Investigating with `-d`.** Because membership is by rank, shrinking
the duration to force a known sample count is a handy way to watch
exactly where values land (the exact `-d` is machine-dependent — tune
it to the count you want; there are no timing guarantees):

```
$ iiac-perf zcr -d 0.000001        # a handful of samples
  p30 0.30       2.8 ns    2.8 ns    0.0 ns    2    2.8 ns      2.0 ns
  p70 0.70       3.0 ns    3.0 ns    0.0 ns    1    3.0 ns      2.3 ns
  p90 0.90       4.2 ns    4.2 ns    0.0 ns    1    4.2 ns      3.4 ns
  mean p30..p90                                     3.2 ns      2.4 ns

$ iiac-perf zcr -d 0.0000001       # one sample → collapses to p50
  p50 0.50       6.3 ns    6.3 ns    0.0 ns    1    6.3 ns      5.5 ns
  mean p50                                          6.3 ns      5.5 ns
```

### Comparing two implementations (`--blocks`)

"Is B really faster than A, or is it noise?" — the workflow:

```
iiac-perf mpsc-2t --pin 0,1 --blocks 10 -d 10
```

`--blocks 10 -d 10` divides the 10-second measuring budget
into **10 blocks of ~1 s each** — same total measurement, now
with an error bar, because each block acts as a mini-run
(random 1–10 ms sleep, unrecorded warm-up, then its share of
the budget). Always pin (`--pin`): unpinned, the OS's thread
placement is re-rolled per *process* and dominates run-to-run
drift — blocks can't see it. The report then ends with:

```
  mean blocks                          4,745.953 ns
  CI95                                    16.115 ns
  LSC                                     21.169 ns
```

- **mean blocks** — the run's headline number: the mean of the
  10 block means.
- **CI95** — 95% confidence interval (half-width) on that
  mean: "the true value is within ±16 ns of 4,746, as far as
  this run can tell."
- **LSC** — least significant change: run the *other*
  implementation the same way (same `-d`, same `--blocks`,
  same pin), and if the two `mean blocks` differ by more than
  roughly the larger of the two `LSC`s, the difference is
  real at 95% confidence.

Caveat: this error bar sees *within-invocation* variation
only. Some per-process state survives the sleeps (measured
~0.6% residual drift even pinned, on an idle Ryzen 5 7600X),
so treat `LSC` as the lower bound — for a decision that
matters, run each implementation 3–5 times interleaved
(A,B,A,B,…) and apply the same comparison to the per-run
`mean blocks` values. Method and worked numbers:
[Comparing implementations](notes/design.md#comparing-implementations-least-significant-change),
[block validation](notes/design.md#block-validation-results-0210-4-r5-7600x).

### Label styles (`--band-labels`)

`--band-labels` selects the row-label vocabulary; the trimmed
`mean`/`stdev` rows and the report header's `labels=` metadata
follow the same style. The trimmed label names the **populated**
non-tail span — here `min` is never a row (no samples land in the
fast tail), so it reads `p20..n2`, not a fixed `min..n2`. Default
`both` prints the zpn name and its literal fraction side by side
(the juxtaposition teaches the zpn vocabulary):

```
$ iiac-perf min-now -d 1 --band-labels both
minstant::Instant::now() [duration=1.0s outer=5,787,017 inner=13 calls=75,231,221 adj/call=1.34ns labels=both]:
                         first           last         range        count           mean       adjusted
  p20 0.20              9.2 ns         9.2 ns        0.0 ns    1,266,420         9.2 ns         7.9 ns
  p30 0.30              9.9 ns         9.9 ns        0.0 ns      435,405         9.9 ns         8.6 ns
  p70 0.70             10.0 ns        10.0 ns        0.0 ns    3,989,304        10.0 ns         8.7 ns
  n2  0.99             10.7 ns        11.5 ns        0.8 ns       43,915        11.0 ns         9.6 ns
  ...
  n6  0.999_999     1,012.7 ns     5,820.4 ns    4,807.7 ns           52     2,513.9 ns     2,512.5 ns
  mean                                                                           9.9 ns         8.6 ns
  stdev                                                                         13.1 ns
  mean p20..n2                                                                   9.8 ns         8.5 ns
  stdev p20..n2                                                                  0.3 ns
```

`zpn` drops the fraction (names only); `frac` drops the name
(fractions only, so the trimmed label reads `0.20..0.99`). Same
bench, separate runs — only the leftmost column and the trim
label change:

```
$ iiac-perf min-now -d 1 --band-labels zpn        $ iiac-perf min-now -d 1 --band-labels frac
  ... labels=zpn]:                                   ... labels=frac]:
  p20    ...                                         0.20      ...
  n2     ...                                         0.99      ...
  ...                                                ...
  mean p20..n2      8.9 ns   7.6 ns                  mean 0.20..0.99      9.9 ns   8.7 ns
  stdev p20..n2     0.6 ns                           stdev 0.20..0.99     1.0 ns
```

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
| zcr-with-1t       |          4 ns | zc-ring-x1 `_with`, 1 thread |
| zcr-with-2t       |        137 ns | zc-ring-x1 `_with`, 2t, spin |

The wait-policy split dominates the 2-thread rows: the parking
benches (`mpsc-2t` and the probe family, all blocking `recv`)
cluster at ~7.4-8.1 µs while the spinning benches sit under
1.2 µs. For context, iceoryx2's own pub/sub benchmark (v0.9.2,
`--bench-all`) on this machine reports 250 ns one-way — ~500 ns
round-trip — with pinned realtime threads and untouched payloads,
consistent with `ice-ps-2t`'s 783 ns measured here. The zcr rows
are the in-process zc-ring-x1 SPSC ring: 1t rounds trip in ~4 ns
(two cache-hot atomics) through the `reserve_slot_with` claim —
see notes/chores/chores-04.md for the pinned tier comparison of
the former raw/spin/with API tiers.

### Verbose output (`-v`)

`-v` prints the affinity/calibration lifecycle on stderr. The
default cal policy is visible: startup mask → `save_affinity` →
pin main to core 0 → calibrate → `restore_affinity` → benches
run on the original (unpinned) mask.

```
$ iiac-perf mpsc-2t -d 3 -v
iiac-perf 0.21.0-3 — Rust latency microbenchmark harness

[INFO  iiac_perf] startup affinity: 0-23 (24 cpus)
[INFO  iiac_perf::pin] save_affinity: mask=0-23 (24 cpus)
[INFO  iiac_perf] pinned main to core 0 for calibration
[DEBUG iiac_perf] affinity during cal: 0 (1 cpu)
[INFO  iiac_perf] calibration params: warmup=100000, dither N_LOW=100 (20x5000), N_HIGH=10000 (20x500), span=64, w_low 100x10000, noise_amp=1.0101
[DEBUG iiac_perf::overhead] dither d_low: mean=81.3993 p99mean=80.3435 medwin=74.6446 spread=29.9762 min=70 ns
[DEBUG iiac_perf::overhead] dither d_high: mean=4973.7741 p99mean=4934.6804 medwin=4966.9860 spread=172.8540 min=4909 ns
[DEBUG iiac_perf::overhead] dither fit(full): in-interval framing=31.9813 ns, loop_per_iter=0.494179 ns
[DEBUG iiac_perf::overhead] dither fit(p99): in-interval framing=31.3098 ns, loop_per_iter=0.490337 ns
[DEBUG iiac_perf::overhead] dither fit(medwin): in-interval framing=25.2270 ns, loop_per_iter=0.494176 ns
[DEBUG iiac_perf] ticks_per_ns: 3.792791
[DEBUG iiac_perf] calibration raw: w_low=101.7762 ns, d_low_p99=80.3435 ns, d_high_p99=4934.6804 ns
[DEBUG iiac_perf] calibration fit: frame_call=52.7425 ns, frame_sample=31.3098 ns, loop_per_iter=0.4903 ns
[INFO  iiac_perf] calibration wall time: 166.61 ms
[INFO  iiac_perf::pin] restore_affinity: mask=0-23 (24 cpus)
Calibration:
  frame/call         52.742 ns  (call-to-call, amortized; sizes inner)
  frame/sample       31.310 ns  (in-interval, dithered; subtracted /inner)
  loop/iter           0.490 ns  (per inner-loop iteration; subtracted)
  cal pin           core 0 (unpinned after cal; --no-pin-cal to skip)
  bench pin         none (unpinned)
  sleep inhibit     active (systemd-inhibit --what=sleep)
  config            none (built-in defaults)

std::sync::mpsc round-trip (2 threads) [duration=3.0s outer=355,664 inner=1 calls=355,664 adj/call=31.80ns labels=both]:
                         first              last           range     count              mean          adjusted
  z4  0.000_1         351.2 ns        4,118.5 ns      3,767.3 ns        36        1,512.6 ns        1,480.8 ns
  z3  0.001         4,329.5 ns        6,365.2 ns      2,035.7 ns       311        5,712.5 ns        5,680.7 ns
  z2  0.01          6,373.4 ns        6,443.0 ns         69.6 ns     3,197        6,416.4 ns        6,384.6 ns
  p10 0.10          6,455.3 ns        6,676.5 ns        221.2 ns    30,421        6,626.7 ns        6,594.9 ns
  ...
  p90 0.90          9,216.0 ns        9,682.9 ns        466.9 ns    35,133        9,423.9 ns        9,392.1 ns
  n2  0.99          9,691.1 ns       12,484.6 ns      2,793.5 ns    32,216       10,302.7 ns       10,270.9 ns
  n3  0.999        12,501.0 ns       29,786.1 ns     17,285.1 ns     3,203       15,571.0 ns       15,539.2 ns
  n4  0.999_9      30,212.1 ns      455,344.1 ns    425,132.0 ns       320       84,150.3 ns       84,118.5 ns
  n5  0.999_99    468,189.2 ns    1,080,033.3 ns    611,844.1 ns        32      745,349.1 ns      745,317.3 ns
  n6  0.999_999 1,286,602.8 ns    1,793,065.0 ns    506,462.2 ns         4    1,455,161.3 ns    1,455,129.5 ns
  mean                                                                            8,315.4 ns        8,283.6 ns
  stdev                                                                           9,285.7 ns
  mean z4..n2                                                                     8,100.0 ns        8,068.2 ns
  stdev z4..n2                                                                    1,144.3 ns
```

Notice `z4 first = 351 ns` — sub-µs. That's the
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

std::sync::mpsc round-trip (2 threads) [duration=3.0s outer=370,718 inner=1 calls=370,718 adj/call=11.61ns labels=both]:
  z4  0.000_1         380.2 ns          410.1 ns           30.0 ns        38          389.1 ns          377.5 ns
  ...
  n2  0.99          9,134.1 ns       11,116.5 ns        1,982.5 ns    33,281        9,603.8 ns        9,592.2 ns
  n6  0.999_999 1,171,259.4 ns    4,276,092.9 ns    3,104,833.5 ns         4    2,238,447.6 ns    2,238,436.0 ns
  mean                                                                              8,012.4 ns        8,000.7 ns
  stdev                                                                             9,735.3 ns
  mean z4..n2                                                                       7,853.4 ns        7,841.8 ns
  stdev z4..n2                                                                        846.1 ns
```

Pinned to two physical cores in the same CCX: tighter body, lower
mean.

```
$ iiac-perf mpsc-2t --pin 0,1 -d 3
Calibration:
  ...
  cal pin           core 0 (from --pin)
  bench pin         [0, 1] (2 slots, 2 unique CPUs)

std::sync::mpsc round-trip (2 threads) [duration=3.0s outer=390,175 inner=1 calls=390,175 adj/call=1.51ns labels=both]:
  z4  0.000_1         370.2 ns          470.0 ns           99.8 ns        38          439.8 ns          438.3 ns
  ...
  n2  0.99          7,827.5 ns        9,625.6 ns        1,798.1 ns    34,816        8,304.1 ns        8,302.6 ns
  n6  0.999_999 4,366,270.5 ns    4,638,900.2 ns      272,629.8 ns         4    4,499,439.6 ns    4,499,438.1 ns
  mean                                                                              7,621.5 ns        7,620.0 ns
  stdev                                                                            37,535.3 ns
  mean z4..n2                                                                       7,070.6 ns        7,069.1 ns
  stdev z4..n2                                                                        632.9 ns
```

Side-by-side (using the trimmed `z4..n2` rows, which exclude the
ms-scale OS-preemption outliers in the `n3`–`n6` tail bands):

| metric          | default    | `--pin 0,1` | Δ      |
|-----------------|-----------:|------------:|-------:|
| `z4` first      |     380 ns |      370 ns |    —   |
| `mean z4..n2`   |   7,853 ns |    7,071 ns | −10 %  |
| `stdev z4..n2`  |     846 ns |      633 ns | −25 %  |

So: default gives you the sub-µs fast path *and* a wider body
(scheduler freedom); `--pin 0,1` gives tighter, lower-mean body
but loses a bit of the best case and is more sensitive to a rare
preemption (the untrimmed `stdev` is actually *wider* pinned here
— 37,535 ns vs 9,735 ns — a single outlier while you're bound to
one core pushes the max to ms-scale). Use the `mean/stdev z4..n2`
rows for representative central tendency and spread:

```
  mean z4..n2                                                                       7,070.6 ns        7,069.1 ns
  stdev z4..n2                                                                        632.9 ns
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
