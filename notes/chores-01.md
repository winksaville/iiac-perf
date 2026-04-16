# Chores-01

Discussions and notes on various chores in github compatible markdown.
There is also a [todo.md](todo.md) file and it tracks tasks and in
general there should be a chore section for each task with the why
and how this task will be completed.

See [Chores format](README.md#chores-format)

## A dummy chore (TBD)

A dummy chore description.

## Measure timer overhead (0.1.0)

Added initial Rust app that measures the overhead of `minstant::Instant::now()`
vs `std::time::Instant::now()`. Uses `minstant` as the reference timer for both
measurements and records 1M iterations into `hdrhistogram` histograms. Reports
min, p50, p90, p99, p99.9, p99.99, max, mean, and stdev in nanoseconds.

This bootstraps the benchmarking project by measuring the measuring stick first,
validating the measurement approach before applying it to actual IIAC techniques.

## Refactor to Bench trait + add channel bench (0.2.0)

Restructure `iiac-perf` so new IIAC benchmarks can be added as self-contained
modules instead of hand-wiring each into `main`. Also add overhead calibration
and the first real IIAC bench: `std::sync::mpsc` round-trip on one thread.

### Why

The README's trajectory (async, serde, io_uring, LAN, WAN, …) implies many
benchmarks. A `match` in `main` would grow unwieldy. Calibrating timer and
loop overhead lets each bench report a per-call cost closer to the technique
itself rather than the measurement apparatus.

### How

Multi-step, target version `0.2.0`. Approach approved by user.

Planned module layout:

```
src/
  main.rs              # CLI: `iiac-perf [timer|channel|all] -i N`
  harness.rs           # Bench trait + runner + histogram printer
  overhead.rs          # timer + loop overhead calibration
  benches/
    mod.rs
    timer.rs           # minstant vs std::time::Instant
    channel.rs         # std::sync::mpsc round trip, single thread
```

Trait sketch:

```rust
pub trait Bench {
    fn name(&self) -> &str;
    fn step(&mut self);  // one unit of work; harness times N per sample
}
```

Overhead subtraction:

- `timer_overhead_ns` — min of back-to-back `Instant::now() / elapsed()` pairs
- `loop_overhead_ns`  — per-iter cost of empty `for _ in 0..N { black_box(()); }`
- `per_call = (elapsed - timer_overhead)/N - loop_overhead`

Print **raw and adjusted side-by-side**. Subtracting a min from a full
distribution is asymmetric — near the floor, adjusted percentiles can go
near zero. Showing both keeps the raw data honest.

### Dev steps

1. `0.2.0-dev1` ✅ chore marker: bump version, write this plan, update todo.
   Also clarified CLAUDE.md: commit-push-finalize runs per step.
2. `0.2.0-dev2` ✅ refactor: `Bench` trait + `harness` module, `benches/timer.rs`.
   `run_bench` is generic (`<B: Bench>`) so monomorphization keeps the hot
   loop free of vtable dispatch — matters for a perf tool. No behavior
   change vs 0.1.0 output.
3. `0.2.0-dev3` ✅ apparatus calibration: run `EmptyBench` (returning `u32`,
   `black_box`-wrapped at the call site) through a dedicated calibration
   loop with `CALIBRATION_INNER = 100` and take per-sample min. Single
   `apparatus/call` number subtracted from each raw value to produce the
   adjusted column. Notes from the experiment:
   - First attempt with INNER=10 + `step()->()` produced 0 ns: TSC reads
     pipeline when there's nothing serial between them.
   - `black_box(())` doesn't introduce a serial dependency (ZST has
     nothing to hold). Required `step()->u32` so `black_box(value)`
     anchors the work.
   - With matching N=10, even with non-trivial step return, total
     framed work (~6 ns) still rounds to 0 under TSC granularity.
   - Decoupling calibration N (=100) from harness N (=10) gives a
     measurable per-call number (~0.6 ns) without changing the bench
     loop. Slight under-subtract of framing-per-call (~1 ns) accepted
     as honest noise floor; revisit if/when a bench needs precision.
4. `0.2.0-dev4` ✅ first IIAC bench: `benches/channel.rs` —
   `StdMpscRoundTrip` keeps both `Sender` and `Receiver` on one thread,
   sends a `u64` then `recv()`s it. CLI gained subcommands: `timer`,
   `channel`, `all` (default when no subcommand). On 3900x: round-trip
   min 20 ns, p50 26 ns, p99 31 ns at 100M calls — close to
   `std::time::Instant::now()` cost since single-thread mpsc has no
   contention/blocking. (Note: `std::sync::mpsc` is implemented on top
   of `crossbeam-channel` since Rust ~1.67, so this is effectively
   measuring crossbeam through the std API.)
   Also widened `Bench::step` return type from `u32` to `u64` — drops
   the `as u32` truncation in channel.rs since the message is `u64`,
   and matches future benches that naturally produce `u64`
   (timestamps, sizes, pointers). Cost on 32-bit CPUs is one extra
   register move per call, negligible.
5. `0.2.0` ✅ finalize: drop `-devN`, move todo entry to Done.

## Multi-thread mpsc + per-bench files + named CLI (0.3.0)

Add the next obvious IIAC bench (cross-thread mpsc round-trip) and
restructure so each bench impl lives in its own file. Replace the
fixed `timer`/`channel`/`all` subcommands with a positional list of
bench names dispatched via a registry — composes naturally as more
benches arrive.

### Why

After 0.2.0, `benches/timer.rs` already held two unrelated `Bench`
impls (minstant vs std::time). Future channel variants (crossbeam,
tokio, flume, …) would compound that. One file per impl scales
without forcing umbrella refactors. The registry-driven CLI
(`iiac-perf min-now mpsc-2t`) lets users pick exactly what they
want without hand-coded subcommands per bench.

### How

File layout:
```
src/benches/
  mod.rs        # REGISTRY: &[(name, RunFn)]
  min_now.rs    # NAME = "min-now"
  std_now.rs    # NAME = "std-now"
  mpsc_1t.rs    # NAME = "mpsc-1t"  (was channel.rs / StdMpscRoundTrip)
  mpsc_2t.rs    # NAME = "mpsc-2t"  (added in dev2)
```

Each bench module exposes `pub const NAME: &str` and
`pub fn run(iterations: u64, overhead: &Overhead)`.

CLI: positional `Vec<String>`. Empty → print one-line help (suggesting
`-h`/`--help`) plus the available bench list, exit 0 — avoids
accidentally launching all benches with a bare `iiac-perf`. `all` →
run every bench in registry order. Unknown name → friendly error
listing valid options.

Breaking CLI change vs 0.2.0 (no more `timer`/`channel` subcommands)
— acceptable on a 0.x release.

### Dev steps

1. `0.3.0-dev1` — split timer into `min_now.rs` + `std_now.rs`,
   rename `channel.rs` → `mpsc_1t.rs`, add registry + named-list CLI.
   No new bench, no behavior change beyond the CLI shape.
2. `0.3.0-dev2` ✅ added `mpsc_2t.rs` — `StdMpsc2Thread` spawns a
   worker thread, uses two `mpsc::channel`s (request + response).
   `step()` sends a `u64` then `recv()`s the echo. Clean shutdown
   in `Drop`: `mem::replace` swaps `req_tx` for a dummy sender so
   the real one drops, the worker's `recv()` returns Err and the
   thread exits and joins. Using replace (not `Option<Sender>`)
   keeps `step()` branch-free.

   Results on 3900x at 10M iterations: **min 270 ns / p50 7,159 ns /
   max 672 µs — ~275× slower than `mpsc-1t` (26 ns p50)**. Bimodal:
   best case ~300 ns when both threads are hot and don't park; typical
   ~7 µs once Linux futex parking/wakeup kicks in. Demonstrates the
   threading cost for tiny messages — every round trip pays two
   wakeups.

   Also appended future bench candidates to todo (crossbeam-channel,
   tokio::mpsc, flume, function-call baselines).
3. `0.3.0-dev3` ✅ adaptive INNER + iteration sizing + apparatus split.
   Default 10M iterations was 70 s for `mpsc-2t`; now each bench
   targets ~1 s by default.
   - Warmup estimates per-step cost (median of 5 × 1000 steps).
   - **INNER** = `max(1, ceil(k × framing / step_cost))`, k = 10 —
     bench dominates framing 10×. On 3900x: min-now INNER=13,
     std-now/mpsc-1t INNER=5, mpsc-2t INNER=1.
   - **iterations** = `target_seconds / (INNER × step_cost)`, default
     target 1.0 s. CLI: `-d/--duration SECONDS` (default 1.0),
     `-i/--iterations N` overrides total count (INNER still adapts).
   - **Apparatus split**: `Overhead { framing_per_sample_ns,
     loop_per_iter_ns }`. Two-point fit on EmptyBench at N=100 and
     N=1000 — slope = loop_per_iter, intercept = framing. Cancels
     TSC pipelining of the framing pair (which made single-point
     measurement read 0). Subtracted as
     `framing/INNER + loop_per_iter`, honest at any INNER.
   - 3900x calibration: framing 11.11 ns, loop_iter 0.49 ns.
     mpsc-2t with INNER=1 now shows the bimodal distribution
     directly: min 280 ns (hot), p1 6 µs (typical with futex park),
     max 1.4 ms (jitter spike).

   ### INNER=1 vs INNER=N — what's actually being measured

   Comparing dev2 (`mpsc-2t -i 100000` → INNER=10, 1M round-trips,
   mean 5,600 ns) against dev3 (`mpsc-2t` → INNER=1, ~113k samples,
   mean 8,500 ns) revealed a meaningful semantic shift:

   - **INNER=N** measures **back-to-back rate**. Inside one sample
     the main thread never pauses, so the worker often hasn't parked
     yet from the previous round-trip — subsequent round-trips skip
     the futex wake. The reported mean is amortized over hot bursts.
     `min 274 ns` translates to "10 round-trips averaged 274 ns each"
     — real burst behavior, not single-RT cost.
   - **INNER=1** measures **isolated single-call latency**. Every
     sample has a `now()/record` pause between round-trips, giving
     the worker time to park. Most round-trips pay the wake. The
     bimodal distribution becomes visible: rare hot RTs (~190 ns)
     vs typical parked RTs (~8 µs).

   INNER=1 is the truer per-call latency a real caller pays.
   INNER=N is useful when you specifically want pipelined throughput.
   `-I/--inner N` lets the user pick. Future work could add a
   dedicated `mpsc-2t-burst` bench fixed at high INNER if the burst
   number deserves its own slot.
4. `0.3.0` ✅ finalize: drop `-devN`, move todo entry to Done.

## Tune duration default + add total-duration flag (0.3.1)

Empirical follow-up to 0.3.0. User found `-d 60` gives ~0.3 % run-to-run
mean swing on mpsc-2t vs ~4 % at `-d 1`, but no data points in between.

### Why

A reasonable default `-d` should give "good enough" stability without
forcing the user to wait long. Also, the current `-d` is per-bench;
some workflows want a fixed total wall-clock budget split across the
requested benches.

### Dev steps

1. `0.3.1-dev1` ✅ measured mpsc-2t at `-d` ∈ {1, 3, 5, 10, 30} × 3
   runs on 3900x. Picked **default = 5.0** at the knee.

   | -d | mean range (ns) | mean swing | p50 range | p99 range |
   |----|----------------|-----------|-----------|-----------|
   | 1  | 7,377–8,890    | ~18 %     | 7,075–8,847 | 11,047–12,343 |
   | 3  | 7,399–8,577    | ~15 %     | 7,263–8,847 | 10,775–11,695 |
   | 5  | 6,960–7,611    | **~9 %**  | 6,763–7,363 | 10,319–11,191 |
   | 10 | 7,132–8,072    | ~12 %     | 6,843–8,199 | 10,703–11,583 |
   | 30 | 7,467–7,711    | ~3 %      | 7,295–7,383 | 10,895–11,127 |

   d=5 is the knee: p50 range collapses 1,772→600 ns, mean swing
   nearly halves over d=1. d=10 didn't pay back its 2× cost in
   this sample (3 runs is small for inference; d=10 would likely
   stabilise with more). d=30 is gold-standard but 2 minutes for
   `iiac-perf all` is too long for a default. d=5 → ~25 s for `all`,
   tolerable.

   Default bumped from 1.0 to 5.0 in `main.rs`. Users wanting
   publication-grade stability use `-d 30` explicitly; README notes
   this.
2. `0.3.1-dev2` ✅ added `-D/--total-duration SECONDS`. Splits the
   budget equally across requested benches; mutually exclusive with
   `-d` via clap `conflicts_with` (clap exits with a friendly error
   if both are passed). `-d` is now `Option<f64>` so `conflicts_with`
   works correctly without faking explicit set; default falls back to
   `DEFAULT_DURATION = 5.0` only when neither flag is given.
   `/README.md` Usage updated with the new flag and an `-D 30`
   example.

   Also fixed `iiac-perf -h` not wrapping long descriptions: clap
   needs the `wrap_help` cargo feature (not in default features) to
   actually wrap, plus `max_term_width = 80` in `#[command(...)]`
   to cap on wide terminals.
3. `0.3.1` ✅ finalize: drop `-devN`, move todo entry to Done.

## Add duration to bench header + logfmt-style metadata (0.3.2)

User asked for the bench header to show the wall-clock duration of
the measurement so it's easy to see how long a run actually took.
While iterating on format we converged on logfmt-style bracketed
`key=value` pairs — self-describing, easy to eyeball, trivial to
parse.

### Change

- `run_bench` now times the whole sampling loop (not warmup/estimate)
  and returns `(Histogram, duration_s)`.
- `run_adaptive` threads `duration_s` through to callers; all four
  bench `run()` functions pass it to `print_histogram`.
- `print_histogram` header switches from
  `{name} ({iters} iters × {inner} inner = {calls} calls; subtract {adj} ns/call)`
  to
  `{name} [duration={:.1}s outer={iters} inner={inner} calls={calls} adj/call={adj}ns]`.

### Why logfmt

- Brackets visually separate metadata from the name.
- Space-separated `key=value` parses with any logfmt lib.
- `outer`/`inner` pair more clearly than the old `iters`/`inner`.
- `adj/call` is more descriptive than `subtract`.

Single-step bump 0.3.1 → 0.3.2 (mechanical change).

## Auto-size histogram columns (0.3.3)

After 0.3.2 the user hand-tuned fixed column widths in `print_histogram`
because values with commas (e.g. `1,701,887 ns`) no longer fit the
previous `{:>10}` fields. Fixed widths are fragile — they overflow on
big `max` values and leave excess whitespace on small runs.

### Change

`print_histogram` now:

1. Builds a unified `Vec<Row>` where `Row { label, raw: f64, adj:
   Option<f64> }` covers every displayed stat. Constructors are
   `Row::with_adj` (percentiles / min / max / mean — with overhead
   subtraction) and `Row::raw_only` (stdev, where overhead
   subtraction on a spread is meaningless).
2. All values render with 0 decimals (whole ns). Rationale: timer
   resolution is ~1 ns and cross-run variance is much larger than
   sub-ns, so extra decimals would overclaim precision. Demonstrated
   at `-d 60`: run-to-run `max` varied 2.8×, `mean` ~5 %, duration
   itself 31 %. 100 ps precision would be fiction.
3. Takes `max(len)` per column from the rendered strings to derive
   `raw_w` / `adj_w`.
4. Emits the header row right-aligned to the numeric column
   right-edges (just before the " ns" suffix).
5. Emits each data row using the computed widths; `raw_only` rows
   fall through a `match` arm that skips the adjusted column.

`print_row` helper removed — rendering is inline with the widths
in scope. Section comments added to each phase so the intent of
header / rows / widths / output is obvious at a glance.

### Why

- No more manual width tuning when values get bigger.
- Each run's table is internally consistent regardless of scale
  (sub-ns to multi-ms).
- Adds ~15 lines of code but deletes the brittle hard-coded widths.

Single-step bump 0.3.2 → 0.3.3 (mechanical refactor, same visible
layout, widths now data-driven).

## Add `--pin` CPU affinity flag (0.3.4)

Observed that unpinned `mpsc-2t` produces an 80 ns min alongside a
3.7 ms max — honest but mixes real measurement with scheduler
noise. Thread migration also invalidates TSC reads across CPUs
(minstant `start`/`elapsed` may span cores if the kernel moves us
mid-measurement), so pinning helps accuracy *and* reproducibility.

### Change

- New dep: `core_affinity = "0.8"`.
- New module: `src/pin.rs` with `parse_cores`, `pin_current`, and
  `plan_summary`. Parser accepts commas + ranges + duplicates (e.g.
  `0,1`, `0-5`, `0,3-5,7`, `0,0`).
- `--pin CORES` CLI flag (comma-separated list / ranges). Treated
  as a **core pool**: thread `i` pins to `pool[i % pool.len()]`.
  Wrap-around handles oversubscription naturally.
- `RunCfg::core_for(thread_idx) -> Option<usize>` helper returns
  the pinning target (or `None` when the pool is empty so callers
  treat unpinned and pinned uniformly).
- `main()` pins the measurement thread to `pool[0]` before
  calibration so framing overhead is measured on the same CPU used
  for benches.
- `mpsc_2t::StdMpsc2Thread::new` takes `worker_cpu: Option<usize>`
  and pins the worker on spawn.
- Single-thread benches (`min_now`, `std_now`, `mpsc_1t`) inherit
  pinning automatically since they run on the (already-pinned)
  main thread.
- Banner adds `pinning  [0, 1] (2 slots, 2 unique CPUs)` line.

### Why logical-CPU terminology

`core_affinity`, `taskset`, and the Linux scheduler all operate on
*logical* CPU IDs — i.e. SMT threads. The docs previously conflated
"core" (physical silicon) with "logical CPU" (OS-visible execution
context). On 3900X: 12 physical cores × 2 SMT = 24 logical CPUs.
Logical `N` and `N+12` are SMT siblings of the same physical core
(confirmed via `/sys/devices/system/cpu/cpu0/topology/thread_siblings_list`
→ `0,12`). README explains the distinction with `--pin 0,1`
(independent cores, same CCX) vs `--pin 0,12` (SMT siblings).

### Measured effect (3900X, idle desktop, `-d 10`)

| pinning | mean | stdev | p99.99 | max |
|---------|-----:|------:|-------:|----:|
| none | 7,044 ns | 6,545 ns | 74 µs | 3.7 ms |
| `0,1` (independent) | 5,636 ns | 1,321 ns | 17 µs | 311 µs |
| `0,12` (SMT siblings) | 5,744 ns | 1,476 ns | 16 µs | 66 µs |

Stdev dropped ~5×, p99.99 ~4×, mean ~20 %. Max still has kernel
preemption events (311 µs / 66 µs with pinning vs 3.7 ms without —
even preemption is rarer on pinned cores). Interestingly,
`0,12` (SMT siblings) and `0,1` (independent cores) are nearly
indistinguishable for this latency-bound round-trip, suggesting
SMT helps hide channel-wait latency rather than hurting via
resource contention.

Single-step bump 0.3.3 → 0.3.4.

## Band-based histogram display (0.3.5)

Replaced per-percentile rows with per-band rows. Each band is
defined by two consecutive percentile boundaries (e.g. min→p1,
p1→p10, …, p99→max) and displays first, last, count, mean, and
adjusted mean — computed by iterating the histogram's recorded
buckets and accumulating per-band stats.

### Why

Percentile rows associate a single boundary value with a label
(e.g. "p99 = 10,095 ns"). When we tried to add per-band sample
counts, neither direction of association (prev→current or
current→next) produced symmetric tail counts: p1 and p99 always
ended up with different band widths depending on which direction
was chosen. This caused persistent confusion about what "p99's
count" meant.

Making the *band* the row (not the *point*) resolved the
asymmetry: min-p1 and p99-max are both 1% of total by
construction. The range labels are wider but unambiguous — a
band IS two boundaries and pretending otherwise was the source
of all the confusion.

### Change

- `print_histogram` now iterates `hist.iter_recorded()` to
  accumulate per-band first/last/count/sum. Each histogram
  bucket is assigned to the band containing its midpoint rank.
- Band boundaries: 0%, 1%, 10%, 20%, …, 90%, 99%, 100%.
  Labels: min-p1, p1-p10, p10-p20, …, p90-p99, p99-max.
- Display columns: first, last, count, mean, adjusted.
  Auto-sized column widths (same pattern as 0.3.3).
- Whole-histogram mean and stdev shown below the band table
  without a count column.
- Removed the old `Row` struct and its constructors — rendering
  is now done via a local `BandRow` struct inside
  `print_histogram`.

Single-step bump 0.3.4 → 0.3.5.
