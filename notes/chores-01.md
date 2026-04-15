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
