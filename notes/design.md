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

## Timer overhead: in-interval vs call-to-call

Analysis from a 2026-07-13 session (3900X + r5-7600x),
prompted by implementing the amortized framing fix above: the
amortized measurement came out 5-10x larger than the intercept
fit it replaced (3900X: ~50 ns vs 1-21 ns; r5-7600x: ~23-32 ns
vs 0-4.5 ns). Both are "the cost of the timer pair" — they
measure different slices of it.

A sample is `now()` (clock read A) ... inner loop ...
`elapsed()` (clock read B); the recorded value is B - A. The
two timer calls cost ~23-50 ns in total, but most of that
sits *before read A* and *after read B* — outside the
recorded interval. Only a small sliver lands inside B - A.

- **In-interval overhead** (~1-11 ns) — the sliver: `now()`'s
  tail after read A, loop entry, `elapsed()`'s prologue up to
  read B; out-of-order execution overlaps much of it with the
  workload. This is what subtraction would want to remove
  from a sample.
- **Call-to-call overhead** (~23-50 ns) — the full apparatus
  cost of taking a sample, both clock-read latencies
  included. This is what throughput sees, and what sizing
  (`pick_inner`) and duration budgeting care about.

The measurability is asymmetric, and unfavorably so:

- Call-to-call amortizes: a window around M whole samples
  contains every bit of the pair cost, so q/M quantization →
  measured to ±0.02 ns warm (r5-7600x: 32.355 ns back-to-back
  pairs, identical across 5 runs; 23.07 ns in the two-point
  window shape, ±0.1%).
- In-interval cannot amortize *by construction*: any window
  wraps whole calls and thus measures call-to-call. The only
  estimator is the un-amortized min (the intercept fit), stuck
  at ±one ~10 ns quantum — on a ~1-11 ns quantity.

So the number we can measure precisely is not the number
subtraction wants, and the number subtraction wants cannot be
measured precisely. Consequences (0.21.0 cycle direction):

- Drop the framing term from the `adjusted` column — a
  quantized guess rendered at 3 decimals is false precision.
  Keep the loop/iter term: the amortized slope is measured to
  ±0.001 ns and dominates the per-call apparatus cost anyway.
- Size `inner` from the *call-to-call* constant: stable,
  conservative (over-sizing is harmless), and it removes the
  original harm — a low quantized framing draw under-sizing
  the experiment. The unsubtracted in-interval residue is then
  bounded by ~q/inner, roughly 1-2% of step cost, and can be
  stated as a bound instead of pretended away.
- The two-point window shape (M whole samples per window at
  N_LOW / N_HIGH) is the right call-to-call estimator: it
  keeps the real sample's pipelining context. We think this is
  why it reads lower than bare back-to-back pairs on r5-7600x
  (23.07 vs 32.36 ns — the loop gives the reads work to
  overlap with).

## Closed-loop mean identity

Why producer and consumer full means (and near enough their
min-p99 trimmed means) agree to ~0.1 ns within a run while
moving ~1.4% between runs (observed on r5-7600x tp-pc,
2026-07-13).

Each thread's probe intervals run back-to-back, so they tile
the run's wall-clock span. Both threads span the same wall
time and record the same count (every handoff appears once in
each thread's series), so:

    mean = sum(intervals) / count ≈ wall_time / count

- Same wall time, same count → same mean, for *any* two
  distributions of the same closed loop. Check: a 5 s tp-pc
  run recorded count = 1,035,886; 5.0 s / 1,035,886 =
  4,827 ns vs reported means 4,814.6 (p) / 4,813.9 (c) — the
  identity minus the small per-iteration gap outside the
  probe.
- `min-p99` trims a same-sized 1% tail from each side's
  series, so it inherits most of the identity (4,778.487 vs
  4,778.378 ns).
- Consequence: on a closed-loop bench the untrimmed mean is a
  *throughput* number, not a latency-shape number. Run-to-run
  mean shifts are real throughput changes (the mode-mix
  wobble), which is the case for the trimmed core stats row
  (see chores-04
  [Trimmed core stats](chores/chores-04.md#trimmed-core-stats-p10-p90)).

## Comparing implementations: least significant change

How to decide whether implementation A is really faster than B
at 95% confidence, worked from a six-run tp-pc series on
r5-7600x (2026-07-13).

The unit of replication is the **run**, not the sample. A run's
millions of samples make its within-run standard error
microscopic; what dominates comparison error is run-to-run
systematic drift (mode mix, frequency regime, calibration).
So compare per-run statistics:

- Do n runs of A and n of B — n is the **replication count per
  implementation** (n=5 → 10 runs total) — **interleaved**
  (A,B,A,B,...) so thermal / frequency drift hits both arms
  alike; same machine, same duration, same pinning; discard an
  initial warm-up run.
- Take one statistic per run (today `mean min-p99`; the
  trimmed p10-p90 row when it lands), giving n values per arm.
- Two-sample t at 95%: LSC = t(0.975, 2n-2) * s * sqrt(2/n),
  with s the pooled run-to-run stdev of the statistic.

From the observed series (`mean min-p99`: 4752.5, 4852.1,
4763.7, 4834.0, 4726.6, 4863.2 ns → s ≈ 58 ns ≈ 1.2%):

- n=3 → LSC ≈ 131 ns (~2.7%)
- n=5 → LSC ≈ 85 ns (~1.8%)
- n=10 → LSC ≈ 55 ns (~1.1%)

Choosing n: invert the formula for the difference Δ you need
to resolve — n ≈ 2 * (t * s / Δ)². At s ≈ 1.2%: Δ=2% → n≈5,
Δ=1% → n≈10-12, Δ=0.5% → n≈40. n=5 is the practical default.

A single run cannot yield an honest LSC: between-run variance
is invisible from inside one run, and a run's internal error
bar (stdev/sqrt(samples) ≈ 0.4 ns here) understates the true
run-to-run spread (~58 ns) by two orders of magnitude. Longer
runs don't substitute for more runs — the -d {5..100} series'
means wobble as much at 100 s as at 5 s, because the dominant
variance is per-invocation systematic state (thread placement,
frequency regime, mode mix): a longer run averages longer over
the *same* draw; only re-running re-rolls it.

### Within-invocation replication: sleep-separated blocks

Idea (2026-07-13 session): chop one invocation into Y blocks
separated by an OS sleep of random ms length, compute the
per-block statistic, and use Y as the replication count in the
LSC formula (df = 2Y-2) — an LSC from a single invocation.
The spin dither handles sub-quantum phase; the sleeps target
the bigger per-invocation state:

- **Mode mix re-rolls** — we think the closed-loop mode mix is
  a semi-stable attractor (why it shows up as *between*-run
  variance); a sleep by one thread breaks the rhythm and
  re-rolls the attractor, converting the dominant hidden
  variance into something blocks can observe.
- **Thread placement re-rolls only weakly** — wake affinity
  returns a thread to its cache-hot core on an idle box; moot
  under pinning.
- **Not re-rolled**: process-start state (memory layout/ASLR,
  the calibration draw — cached constants fix that anyway), so
  block variance may understate invocation variance.
- Mechanics: discard a short post-wake warm-up per block (each
  wake pays a frequency ramp + cache refill — the measured
  cold-calibration effect); randomize sleep length so block
  boundaries don't phase-lock with kernel ticks or workload
  periodicity.
- **Validation required** before trusting the single-run LSC:
  ~10 invocations x Y=10 blocks, compare between-block vs
  between-invocation variance (ANOVA). Close → honest; short →
  report must say so.
- Payoff: block-wise **A/B interleaving** in one invocation
  (A,B,A,B with sleeps between) analyzed as *paired* block
  differences — pairing cancels common-mode drift (thermal,
  ambient load) and tightens the LSC beyond unpaired runs;
  same reason hyperfine alternates commands. Would largely
  subsume a plain `--runs n` mode.

Reducing s beats raising n (LSC shrinks only as 1/sqrt(n)):

- A statistic that ignores the mode-mix wobble — the p10-p90
  trimmed mean's observed run-to-run spread was ~±0.2%, putting
  a 5-run LSC near 0.4%.
- A pinned frequency regime (`performance` governor on the
  bench box) and cached calibration constants (this cycle)
  remove two more run-to-run variables.
- A headless idle machine: r5-7600x repeats warm calibration
  constants to ±0.02%; the desktop 3900X wobbles ~±5%.

## Dithering: random phase injection

Idea from a 2026-07-13 session: insert a random delay between
samples so consecutive measurements stop sharing their phase
relative to the ~10 ns clock lattice ("the whole run commits to
one lattice point"). This is classic **dither** from
signal processing / ADC practice: randomness injected before a
quantizer turns quantization error from a stuck systematic
offset into zero-mean noise that averages away — a mean over N
dithered samples converges through the quantum with error
~q/sqrt(N).

- The dither must be **sub-quantum and fine-grained**: a random
  0..q (~0-10 ns) spin between samples, e.g. spinning on the
  TSC (~0.2-0.3 ns granularity). Not a sleep: `usleep` is
  µs-coarse, costs a syscall, and wakes on kernel timer
  boundaries, which can *re-align* phase rather than randomize
  it. The spin sits outside the timed interval, so it costs
  throughput only, not accuracy.
- Sharpens **means, not mins or percentiles**: a dithered min
  still converges to the lattice floor, and each individual
  sample stays ±q. Band boundaries don't benefit; aggregate
  mean statistics do.
- Sizing the win: the stuck phase shifts a run's aggregate
  means coherently by up to ±q, a run-to-run variance
  component dithering removes. Negligible against ~5 µs
  samples (≤0.2%); decisive for fast benches — `min-now` at
  ~24 ns/call or probes at inner=1, where q is ~40% of the
  value. It does not touch mode-mix or placement variance, so
  it does not rescue a single-run LSC (see
  [Comparing implementations](#comparing-implementations-least-significant-change)).
- Possible payoff: a dithered *mean-based* two-point fit could
  estimate the **in-interval framing** to sub-quantum accuracy
  — the quantity
  [call-to-call windows cannot reach](#timer-overhead-in-interval-vs-call-to-call)
  — potentially reviving a defensible framing subtraction.
  Unvalidated; would need an experiment showing the dithered
  intercept is stable run-to-run.

### Why dither works, and which statistics keep the win

A sample is the difference of two timestamps quantized by the
*same* clock: `elapsed = q*floor((t_B+φ)/q) - q*floor((t_A+φ)/q)`.
With phase φ uniform over the quantum, the expected value of
that difference is exactly `t_B - t_A` — unbiased, no q/2
offset. Mean over N dithered samples → error q/sqrt(12N)
(~0.01 ns at 100k samples, q = 10 ns).

The dithered distribution of a near-constant duration
T = q*(k+f) is **two-point**: k*q with probability 1-f,
(k+1)*q with probability f. All sub-quantum information lives
in the proportions, so only statistics *linear* in the samples
read it:

- Full mean: q*(k+f) = T. ✓
- p40-p60 window mean: the central window sits inside the
  majority pile unless f ∈ (0.4, 0.6) — snaps to a lattice
  point like a median. ✗
- p1 (or min): converges to the floor k*q — the estimator
  dither exists to escape. ✗

Robustness without breaking linearity — interrupt
contamination is **one-sided** (interrupts only add time):

- Mean below p99 (drop the top 1% only; hdrhistogram is the
  natural machinery): sheds spikes at a small *estimable*
  bias, ~ -0.01*q ≈ -0.1 ns when nothing was contaminated.
- Median of window means: mean inside each window (reads the
  proportions, near-Gaussian), median across windows (rejects
  a bad window without snapping — window means are no longer
  lattice-valued). Linear/robust split in the right order.
- Caveat either way: mean-based fits absorb interrupt time in
  proportion to interval length, tilting the two-point slope
  up and the intercept down (~ -0.1 ns on an idle pinned
  core). The validation experiment must bound it.

### Dither validation results (0.21.0-2, r5-7600x)

Ten warm invocations plus two cold (30 s idle) on r5-7600x,
dithered two-point fit logged at debug level alongside the
window calibration:

- **Fast regime** (slope ≈ 0.3677): in-interval framing (p99
  fit) 8.12 / 8.22 / 8.21 / 8.23 / 8.19 / 8.33 — **8.2
  ± 0.06 ns (±0.7%)**, one outlier 9.55. Slope repeats to
  ±0.00002 ns (five significant figures).
- **Slow regime, including both cold starts** (slope ≈
  0.4132-0.4145): framing 8.6-9.4 ns. No 0.00 clamp, no
  lattice jump — the old estimator's cold pathology is gone;
  cold is just the slow clock regime.
- **Regimes scale together**: framing ratio 9.3/8.2 ≈ 1.13
  matches slope ratio 0.414/0.368 ≈ 1.125 — both constants are
  core-clocked, as predicted in
  [Frequency dependence](#frequency-dependence-what-is-constant-what-is-not).
  The slope is a precise **regime fingerprint** for the cache
  validity check.
- Aggregations agree within ~0.05 ns; **mean-below-p99 is the
  tightest**, full mean the loosest, median-of-window-means in
  between — matching the
  [estimator analysis](#why-dither-works-and-which-statistics-keep-the-win).
- The dithered value (~8.2 ns) sits well above the old
  min-based draws (0-4.5 ns): the min sampled the lattice
  floor *and* best-case pipeline overlap, systematically
  under-reading — consistent with the vDSO clock read being
  lfence-serialized (limited overlap with the workload).

Verdict: the dithered fit **is** calibration v3 — in-interval
framing returns as a subtractable constant with ~±0.1 ns
run-to-run repeatability within a regime, loop_per_iter comes
out finer than the window fit, and one window pass still
supplies the call-to-call sizing constant.

### Reproducing the calibration experiments

Since 0.21.0-3 the dithered fit *is* the calibration (banner:
`frame/call`, `frame/sample`, `loop/iter`, every run); `-v`
additionally logs the raw points and alternative fits. Any
bench works as the vehicle; `min-now -d 0.01` keeps the bench
part negligible.

- One run, all calibration lines:

      iiac-perf -v min-now -d 0.01 2>&1 | grep -E 'dither|calibration'

  `calibration fit:` is the production result (frame_call +
  frame_sample + loop). `dither fit(full|p99|medwin):` compares
  the three aggregations — `p99` is the production one.
- Warm repeatability series (the validation shape):

      for i in $(seq 1 10); do
        iiac-perf -v min-now -d 0.01 2>&1 | grep 'dither fit(p99)'
      done

- Cold start (deep-idle regime): `sleep 30` before the run.
  Expect the slow regime — higher loop/iter (the regime
  fingerprint) and proportionally higher framing — not a
  pathological value.
- Raw ingredients per point (`dither d_low:` / `d_high:`
  lines): mean, p99-trimmed mean, median window mean, window
  spread (dispersion / CI signal), and min (lattice floor for
  comparison).

### Block validation results (0.21.0-4, r5-7600x)

`mpsc-2t -d 10 --blocks 10`, checking whether the single-run
block CI predicts between-invocation spread:

- **Unpinned, 8 invocations**: invocation means are bimodal —
  a fast tight state (4,885-4,948 ns, CIs ±19-38) and a slow
  loose one (5,048-5,252 ns, CIs ±97-189). Between-invocation
  s ≈ 124 ns vs a blocks-predicted ≈ 52 ns: blocks captured
  only ~20% of the between-invocation variance. We think the
  missing component is thread placement — it persists for the
  process lifetime, so 1-10 ms sleeps never re-roll it.
- **Pinned (`--pin 0,1`), 6 invocations**: means 4,713-4,795,
  s ≈ 29 ns (0.6%), bimodality gone; single-run CIs 7.5-48 ns
  (LSC 10-63 ns, 0.2-1.3%). Residual between-invocation state
  is still ~4× the block-sampling prediction, but small in
  absolute terms.
- **No blocked-vs-unblocked shift**: three unblocked pinned
  runs' trimmed means (4,709-4,732) sit inside the blocked
  range — the sleeps + post-wake warm-ups don't move the
  measurement.

Interpretation: the report's `CI95` / `LSC` lines are honest
*within-invocation* replication — use them as a lower bound on
cross-invocation confidence, pin to remove the placement state
(unpinned it dominates), and keep interleaved multi-run
comparison for final A/B calls.
