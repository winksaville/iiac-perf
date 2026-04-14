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
4. `0.2.0-dev4` — add `benches/channel.rs` (`std::sync::mpsc`, single thread)
   + CLI subcommand dispatch.
5. `0.2.0` — finalize: drop `-devN`, update todo/chores Done.
