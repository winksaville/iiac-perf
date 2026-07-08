# Design

Durable design analyses for iiac-perf. Chores sections record
what landed per commit; this file holds the analysis and
reasoning that should outlive any one cycle — measurement
theory, error models, and the design decisions they drive.
Chores and todo entries link here rather than restating.

## Architecture Overview

iiac-perf is a latency microbenchmark harness: a single binary
that runs named benches (mostly inter-thread communication
round-trips) and renders each as a band-table latency
histogram. The architecture has four layers — startup /
environment control, measurement, benches, and reporting.

### Startup and environment control

`main.rs` runs a fixed pipeline before any bench executes:

- **CLI** (clap) — bench selection by exact name or prefix
  (`zcr` runs every `zcr-*` bench), plus duration, outer/inner
  overrides, pinning, and report options.
- **Sleep inhibition** (`inhibit.rs`) — re-exec the process
  under `systemd-inhibit` so an idle suspend can't poison a
  long run; harness suspend detection remains the backstop.
- **Layered config** (`config.rs`) — built-in < XDG file <
  project-local file < CLI. Holds defaults and named pin
  profiles; the banner reports which files loaded.
- **Pinning** (`pin.rs`) — `--pin` core specs / profiles;
  main is pinned to a stable core for calibration by default
  and the pre-cal affinity restored afterwards, so the cal
  pin doesn't leak into the scheduler's bench placement.
- **Calibration** (`overhead.rs`) — the two-point fit that
  produces the `Overhead` constants (framing/sample,
  loop/iter) used for adjustment; see
  [Calibration accuracy](#calibration-accuracy-framing-quantization).

### Measurement

Two styles coexist, one harness-driven and one self-driven:

- **Harness-driven** (`harness.rs`) — a bench implements the
  `Bench` trait (`step()` = one measured operation). The
  driver estimates step cost, auto-sizes `inner` so framing
  stays ≤~10% of a sample window (`pick_inner`), then times
  `inner` back-to-back calls per sample into an HDR
  histogram, recording in ps so sub-ns per-call values
  survive the divide. Reported values subtract
  `Overhead::per_call_ns(inner)`.
- **Probe-driven** — for benches whose threads drive
  themselves (actor-style), probes are the measurement
  channel and there is no outer `Bench` loop:
  - `Probe` (`probe.rs`) — caller-timed ns deltas into a
    named histogram.
  - `TProbe` (`tprobe.rs`) — same shape but records raw
    hardware tick deltas (no tick→ns mul-shift on the hot
    path); conversion deferred to report time.
  - `TProbe2` (`tprobe2.rs`) — scope API: `start(site)`
    returns an id, `end(id)` appends a record; records are
    drained into the histogram lazily at report time.
  - `ticks.rs` — arch-neutral fixed-rate counter facade
    (`read_ticks` / `ticks_per_ns`) over per-arch impls
    (x86 TSC with CPUID invariant-TSC gate, aarch64 cntvct).

Timing sources: harness and `Probe` use `minstant` (TSC-backed
`Instant`); `TProbe`/`TProbe2` read ticks directly.

### Benches

`benches/mod.rs` is a flat registry: each bench module exposes
`NAME` and `run`, and registration is appending to `REGISTRY`.
Families: timer call-cost baselines (`std-now`, `min-now`),
std mpsc round-trips (1t/2t/spin), zc-ring-x1 SPSC/MPSC
round-trips (`zcr-*`), iceoryx2 pub/sub and request/response
(`ice-*`), and probe-instrumented variants (`probe-mpsc-2t`,
`producer-consumer`, `tp-pc`, `tp2-pc`) used to measure the
probes themselves. The 1t/2t naming is a convention: same transport,
same-thread vs cross-thread, so apparatus and contention
effects separate.

### Reporting

- `bands.rs` — single source of truth for the band ladder
  (deciles in the body, zeros/nines tails z4..n10) and label
  styles; pinned by tests against the README's table.
- `harness::print_report` — the main per-bench table:
  first/last/range/count/mean per band plus adjusted column
  and whole-vs-trimmed mean/stdev summary lines.
- `band_table.rs` — shared renderer for tick-valued probe
  histograms, so `TProbe`/`TProbe2` output matches the main
  table's shape and columns line up under the bench report.

## Calibration accuracy: framing quantization

Analysis from a 2026-07-08 session on the 3900X, prompted by
the framing/sample header value jumping between ~1 ns and
~21 ns across runs while loop/iter held steady at 0.49 ns.

### Observation: framing sits on a ~10 ns lattice

Repeated runs reported framing/sample of 1.02, 11.12, 11.22,
and 21.22 ns — not a continuous drift but discrete steps of
~10.1 ns. Decoding through the two-point fit
(`src/overhead.rs`), the underlying `min_low` measurements
were 50, 60, and 70 ns.

We think the mechanism is TSC granularity: `minstant` reads
the TSC, and on Zen 2 the TSC is derived from the 100 MHz
reference clock, so elapsed-time readings quantize to ~10 ns
steps.

### Why framing wobbles but loop/iter is steady

Both constants come from the same two measurements; they
differ in amortization:

- `framing = min_low - N_LOW * loop_per_iter` — inherits
  `min_low`'s quantization one-for-one (noise amplification
  ~1.01, per the comment in `src/overhead.rs`).
- `loop_per_iter = (min_high - min_low) / 9_900` — the same
  ±1-2 quanta divides by 9,900 → ±0.002 ns, invisible.

Framing is the only quantity in the fit measured
un-amortized; that asymmetry is the whole story.

### The min estimator cannot see inside a quantum

An interval of true length T measures as ⌊T/q⌋ or ⌈T/q⌉
quanta depending on phase alignment; the min over many phase
re-rolls converges to the lower reading. So a min-based
estimate reports the lattice floor and can under-read the
true duration by up to one quantum. The true framing on the
3900X lies somewhere in roughly [1, 11] ns; the 1.02 and
21.22 readings are the floor and ceiling lattice points.
Within one run the phase appears stable (100k samples pick
one quantum); re-entering the measurement re-rolls it.

### Error propagation through `pick_inner`

The framing estimate is load-bearing beyond post-hoc
subtraction — it sizes the experiment
(`src/harness.rs::pick_inner`,
`inner ≈ 10 * framing / step_cost`):

- Auto-sized benches: per-call adjustment error is
  `δ/inner ≈ δ/(10 * framing)` relative to step cost —
  independent of step speed. With framing ≈ 11 ns and δ up
  to a ~10 ns quantum, up to ~9% relative error on every
  auto-sized bench.
- Under-estimated framing under-sizes `inner`: a 1.02 ns
  calibration draw on a ~2.9 ns step picks inner ≈ 4 instead
  of ~43; the true ~11 ns framing is then ~2.8 ns per call
  (~50% apparatus), of which only 0.26 ns is subtracted.
  Nothing in the report reveals it.
- `--inner` override at 1 exposes the full δ (~10-20 ns)
  per call, unamortized.

Consequence: a min-of-mins repeat of the current fit would
be the wrong fix — consistently landing on the *low* quantum
consistently under-sizes `inner`. Over-estimating framing is
harmless (bigger `inner`); under-estimating distorts.

### Fix: amortized framing measurement

Measure the timer-pair cost inside one window: one timer
read at the start, a loop of M back-to-back timer pairs, one
read at the end, divide by M. The ±1-quantum error on the
window amortizes to q/M per pair (~0.001 ns at M = 10,000) —
the same trick that already makes loop/iter steady.

Caveat: this measures the *throughput* cost of paired reads,
which can differ from a timer pair pipelined around real
work. We think it is still a far better subtraction constant
than "somewhere in a 10 ns interval", but it carries a
different (small) bias, not ground truth.

### Frequency dependence: what is constant, what is not

- The TSC rate and its ~10 ns quantum are invariant —
  constant across turbo, throttle, and idle states.
- The framing *cost* is core-clocked work (~tens of cycles
  of `rdtsc` + arithmetic), so its wall-clock value scales
  with core frequency: the same cycles take ~2× longer at a
  2.2 GHz throttle vs 4.6 GHz boost. Same for loop/iter.
- In practice calibration (post-warmup) and benches (hot
  loops) both run boosted, so the constants usually match.
  When frequency does shift, step cost shifts with framing
  cost, so contamination stays roughly proportional rather
  than blowing up; the subtraction just stops being exact.

So a cached calibration is a *machine + frequency-regime*
constant, not a universal one.

### Design: cached calibration in the config file

Store the calibration constants in the existing config file
(`src/config.rs`) rather than measuring fresh each run. The
primary benefit is run-to-run *comparability*, not startup
time: with a pinned adjustment constant, deltas between runs
reflect the bench, not the calibration lottery (~±0.23 ns on
adjusted values at inner = 43 today).

- An explicit calibrate command writes/refreshes the entry;
  runs use it when present, calibrate live otherwise.
- The entry carries provenance: CPU model, iiac-perf
  version, calibration date, raw `min_low`/`min_high`.
- Each run does a quick live amortized framing check
  (milliseconds) against the cached value: within a few
  percent, use the cache silently; off by more (throttle,
  different governor, stale file), warn or fall back to
  live. Cache validity becomes measured, not trusted.
- The report header states which mode adjusted the run —
  cached (with date) vs live — so a report is never
  ambiguous about its constants.

Rejected alternative: scaling calibration length with bench
duration (`-d` percentage). The residual error is
quantization, not sampling depth — more time in one
continuous measurement does not re-roll the phase alignment —
and calibration measures a machine property, not a bench
property.
