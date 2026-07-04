# Chores-03

Continuation of `chores-02.md`, which crossed 1,200 lines. Same
format; see [Chores format](README.md#chores-format).

## Plan: TProbe start/end (0.9.0-dev1)

Planning commit for 0.9.0. No code change in this step. Introduces
a scope-based recording API on `TProbe` (`start` / `end`) that
keeps the hot path to a pair of tick reads plus a record append —
all delta math and histogram ingestion deferred to report time.

Design sketch lives in
[ideas.md — Tprobe](ideas.md#tprobe-time-probe). This plan fixes
the subset that will actually land across 0.9.0.

### Scope

Add `TProbe::start` / `TProbe::end` as a second recording path on
the existing `TProbe`. The existing `record(ticks: u64)` stays
for now; it is not replaced until a follow-on step lands a full
replacement (scope-based recording + new report formatting).

### Surface (to be implemented in dev2)

```rust
#[must_use]
#[derive(Clone, Copy)]
pub struct TProbeRecId {
    site_id: u64,
    start_tsc: u64,
}

impl TProbe {
    pub fn start(&mut self, site_id: u64) -> TProbeRecId;
    pub fn end(&mut self, tpri: TProbeRecId);
}
```

- `start(site_id)` reads `ticks::read_ticks()` and returns a
  `TProbeRecId` carrying `(site_id, start_tsc)`. No buffer
  allocation — the id holds the start tick itself (Option B in
  ideas.md).
- `end(tpri)` reads `ticks::read_ticks()` and pushes a complete
  record `(site_id, start_tsc, end_tsc)` onto the probe's record
  buffer. Buffer only ever holds complete records.

### Record buffer

```rust
struct Record { site_id: u64, start_tsc: u64, end_tsc: u64 }
```

`Vec<Record>` field on `TProbe`, appended to at `end()` time.
Unbounded for the first cut; bounded / ring-buffer policy
deferred.

### Raw ticks, not deltas

Records store raw `start_tsc` + `end_tsc`, not `end − start`.
Two reasons:

1. Skipping the subtraction keeps the hot path to `rdtscp` +
   store, without an arithmetic op between the read and the
   append.
2. Raw TSC values retain record-order information across
   interleaved scopes or sites — useful for correlating a
   stream, which a delta-only layout loses.

Delta = `end_tsc − start_tsc` is computed at report time.

### site_id policy

`site_id: u64` is supplied by the caller. For the first cut, the
simplest generator is a single global `AtomicU64` counter (users
cache the returned id at each call site via `LazyLock` or a
`static`). A per-`TProbe` counter is equivalent for uniqueness
within one probe's scope — pick whichever first caller needs.

Auto-generated site_ids from a compile-time const hash of
`file!()` / `line!()`, macros, and an `inventory`/`linkme` slice
for name resolution stay as [ideas.md open
work](ideas.md#auto-generated-site_id-compile-time-zero-user-effort)
— not in 0.9.0.

### Coexistence with `record(ticks)`

`TProbe::record(ticks)` is untouched and keeps feeding the
existing histogram directly. `start` / `end` populate the new
record buffer. At report time, the buffer is drained into the
same histogram (delta per record), so `report()` renders one
combined band-table regardless of which path produced samples.

Removing `record(ticks)` waits until the scope API has a full
replacement (report reshape + bench migration).

### Report processing (dev3)

First cut: lazy — `report()` drains the record buffer into the
existing histogram on first call (delta = `end_tsc − start_tsc`,
clamped to `1` to respect the histogram lower bound), then prints
the same band-table as today. `site_id` is ignored in this pass.

Per-site sub-tables (one band-table per distinct `site_id`) is a
follow-on once more than one site is in use.

### Hot-path cost

Target: `start` and `end` each compile to roughly one
`read_ticks()` + register moves, plus — for `end` — one `Vec`
push. No branches on the hot path beyond what `Vec::push` itself
needs (capacity check). The deferred-math choice is what buys
this.

### Edits (this dev1 commit)

- `Cargo.toml` — version bump `0.8.0` → `0.9.0-dev1`.
- `notes/todo.md` — In Progress entry + dev1 Done entry +
  reference `[28]`.
- `notes/chores-03.md` — new file; this section.

### Intentionally out of scope

Everything below is deferred:

- Double-end detection (extra branch vs. silent bug class) —
  revisit alongside the dev2 buffer impl.
- Guard / auto-scope sugar on top of `start` / `end`.
- Alternative buffer layouts (double-buffer, ring-buffer, etc.)
  for long-running recording or improved hot-path performance.
- Background drain thread (low priority) that pulls records out
  of the buffer and processes or forwards them.
- Retaining the raw-ticks record stream as long-term trace data
  (beyond what the histogram keeps) — useful when record order
  across sites matters.
- Userinfo payload on records.
- Per-site grouping in the report.
- Auto-generated `site_id` via const hash + macro + inventory
  slice.
- Replacing `record(ticks)`.

### Next step

Approval from user, then implement as `0.9.0-dev2` (types +
`start` / `end` + record buffer).

### Preview of remaining steps

- `dev2` — implement types + `start` / `end` + record buffer
  (no report wiring yet; unit tests for start/end round-trip).
- `dev3` — lazy report ingestion: `report()` drains records into
  the histogram.
- `dev4` — wire `start` / `end` into an existing bench (likely
  `tp-pc`) as the first consumer, validate numbers.
- `0.9.0` final — remove `-devN`, roll the devs up, move entries
  to `## Done`.

## Implement TProbe start/end + buffer (0.9.0-dev2)

Implements the 0.9.0-dev1 plan's surface: `TProbeRecId`,
`TProbe::start` / `end`, and a `records: Vec<Record>` buffer on
the probe. Report wiring is not part of this step; `record(ticks)`
and the histogram path are unchanged. Verified via four new unit
tests and a smoke-test run of `tp-pc` (output unchanged).

### Edits

- `Cargo.toml` — version bump to `0.9.0-dev2`.
- `src/tprobe.rs`:
  - Added `pub struct TProbeRecId { site_id: u64, start_tsc: u64 }`
    with `#[must_use]`, `#[derive(Clone, Copy, Debug)]` and
    private fields.
  - Added internal `struct Record { site_id, start_tsc, end_tsc }`.
  - Added `records: Vec<Record>` field on `TProbe`; initialized
    empty in `TProbe::new`.
  - Added `#[inline] pub fn start(&mut self, site_id: u64)
    -> TProbeRecId`.
  - Added `#[inline] pub fn end(&mut self, tpri: TProbeRecId)`.
  - Added `#[cfg(test)] mod tests` with four tests:
    `start_end_appends_one_record`,
    `start_end_preserves_start_tsc`,
    `start_end_interleaved_non_stack` (non-stack nesting —
    `start(a) start(b) end(a) end(b)` — succeeds per Option B),
    `record_and_start_end_are_independent` (histogram and record
    buffer are unaffected by each other).
- `CLAUDE.md` — pre-commit checklist: added
  `cargo test --release` as step 4 (release inlining / OoO
  coverage for hot-path code).
- `notes/todo.md` — dev2 Done entry + reference `[29]`.
- `notes/chores-03.md` — this section.

### Findings

- **Binary-crate dead-code warnings.** `clippy -D warnings`
  flags the new pub items (`TProbe::start`, `TProbe::end`, the
  `Record` struct, the `records` field) as dead code because
  `iiac-perf` is a binary crate and nothing in `main.rs` reaches
  them yet. Tests under `#[cfg(test)]` don't count for the
  non-test dead-code analysis. Added `#[allow(dead_code)]` to
  each of the four items with a one-line note pointing at the
  dev that will remove the allow (dev3 drains `records` +
  `Record` in `report()`; dev4 wires `start`/`end` into a
  bench). The allows will be stripped in those devs.
- **`&mut self` on `start`.** `start` doesn't strictly need
  `&mut self` — the hot path is `ticks::read_ticks()` +
  moves. Kept `&mut self` to match `record(&mut self, ...)`
  and to leave room for future hot-path state without an API
  break. Since `TProbeRecId` is `Copy`, the exclusive borrow
  is released at the `let id = probe.start(...)` return; the
  later `probe.end(id)` borrow is independent.
- **Monotonic assertion.** The round-trip test asserts
  `end_tsc >= start_tsc`. On invariant TSC with a single-core
  scope this always holds; if a future test migrates scopes
  across cores it may need relaxing.
- **Smoke test.** `iiac-perf tp-pc -d 0.1` (3900X, idle) —
  numeric output matches dev5 within run-to-run variance;
  no regression from adding the buffer field + new methods.
- **`start_tsc` uniqueness is not an invariant.** `TProbeRecId`
  is opaque data the caller hands back at `end()`; there is no
  lookup or "which scope am I closing?" matching — `end()` just
  appends whatever fields the id carries. Two scopes that
  happen to share `(site_id, start_tsc)` produce two
  indistinguishable records, which is a data-fidelity concern,
  not a correctness one. Keeping uniqueness out of the
  invariant set is what lets `start`/`end` stay at "one tick
  read plus record-keeping" — adding dedup would cost more
  than it saves. In practice the stack return trip around
  `start` is comfortably more than one TSC tick, so collisions
  don't occur on current hardware anyway.
- **Release-mode tests run clean.** `cargo test --release`
  passes all 12 (see CLAUDE.md pre-commit checklist step 4 —
  added this pass).

## Lazy report drain: records → histogram (0.9.0-dev3)

Wires the `records` buffer into `TProbe::report`: on each call,
pending `(start_tsc, end_tsc)` pairs are drained and converted to
tick-delta samples in the histogram before the band-table is
rendered. Completes the scope-API path — `start` → `end` →
eventually visible in a `report()` — while keeping the existing
`record(ticks)` path untouched.

### Edits

- `Cargo.toml` — version bump to `0.9.0-dev3`.
- `src/tprobe.rs`:
  - `report(&self, ...)` → `report(&mut self, ...)`.
  - Drain loop at top of `report`: for each `Record`,
    `hist.record((end_tsc.saturating_sub(start_tsc)).max(1))`.
  - Removed `#[allow(dead_code)]` on `Record` and on the
    `records` field (both are live now).
  - Left `#[allow(dead_code)]` on `site_id` (read once per-site
    grouping lands; see out-of-scope list) and on the
    `start` / `end` methods (first non-test caller lands in
    dev4).
  - Added unit test `report_drains_records_into_histogram`:
    start/end twice → hist empty, records = 2 → report →
    records = 0, hist = 2; second report is a no-op.
- `src/benches/tp_pc.rs` — `let producer_probe = …` /
  `let consumer_probe = …` → `let mut …` to satisfy the new
  `&mut self` on `report`. `tp-pc` still populates via
  `record(delta)` manually, so the `records` buffer stays
  empty and the drain is a no-op; output shape unchanged.
- `notes/todo.md` — dev3 Done entry + reference `[30]`.
- `notes/chores-03.md` — this section.

### Findings

- **Drain placement.** Drain runs at the very top of `report`,
  before `sample_count` is computed, so a probe that only used
  `start`/`end` (empty `hist` but non-empty `records`)
  produces the full band-table rather than the
  `sample_count == 0` short-circuit.
- **Delta clamp.** Uses `saturating_sub(..).max(1)` to match the
  existing `record(ticks)` behavior. Back-to-back tick reads on
  a fast core can legitimately produce 0, which the histogram
  rejects (lower bound 1). `saturating_sub` guards against
  any pathological `end < start` case (e.g. a scope that crossed
  cores without invariant-TSC coherence) — would rarely matter,
  but cheaper than a panic path.
- **Idempotent report.** `records.drain(..)` empties the vec,
  so a second `report()` has nothing to drain and just
  re-renders the histogram state. Useful: `tp-pc` calls
  `report` twice (once per probe), each on a distinct probe,
  but a pattern that reports the same probe twice still works.
- **`site_id` dead for now.** The drain path ignores `site_id`
  (all samples lumped into the single histogram). Kept the
  field populated so per-site grouping in a later step is a
  drop-in without changing record shape. `#[allow(dead_code)]`
  scoped to just that field.
- **`&mut self` ripple.** Only `tp-pc` called `TProbe::report`;
  changing the receiver required `let mut …` on two bindings
  there. No other callers.
- **Smoke test.** `iiac-perf tp-pc -d 0.1` — consumer loop
  mean min-p99 ≈ 9,091 ns; producer loop similar. Matches
  dev2 range within normal variance.

## Wire tp-pc to TProbe start/end (0.9.0-dev4)

First non-test consumer of the scope API. Replaces `tp-pc`'s
manual `ticks::read_ticks()` pair + `probe.record(delta)` with
`probe.start(0)` / `probe.end(id)`. With this step, the full
scope-API path — `start` → `end` → records buffer → drain in
`report()` — is exercised end-to-end by a real bench. Removes
the dev2-era `#[allow(dead_code)]` on `TProbe::start` and
`TProbe::end`. `TProbe::record` is no longer called from any
bench and gets its own `#[allow(dead_code)]` pending the
out-of-scope "replace record(ticks)" decision.

### Edits

- `Cargo.toml` — version bump to `0.9.0-dev4`.
- `src/tprobe.rs`:
  - Dropped `#[allow(dead_code)]` on `start` and `end`
    (first non-test caller now exists).
  - Added `#[allow(dead_code)]` on `record(ticks)` with a note
    pointing at the out-of-scope removal decision — no bench
    calls it anymore.
- `src/benches/tp_pc.rs`:
  - Producer loop: `let s = ticks::read_ticks(); … let e =
    ticks::read_ticks(); probe.record(e.wrapping_sub(s));`
    → `let id = probe.start(0); … probe.end(id);`.
  - Consumer loop: same rewrite.
  - Dropped the now-unused `use crate::ticks;`.
  - Updated the `//!` module doc to reference the scope API
    instead of `ticks::read_ticks`.
- `notes/todo.md` — dev4 Done entry + reference `[31]`.
- `notes/chores-03.md` — this section.

### Findings

- **A/B vs. dev3: dev4 is slower per-sample and completes
  fewer iterations.** `iiac-perf tp-pc` (default `-d 5`) on
  the 3900X (idle, `--pin` default):

  | version | producer mean min-p99 | consumer mean min-p99 | loop count (5 s) |
  |--------:|----------------------:|----------------------:|-----------------:|
  | dev3    | ≈ 5,076 ns            | ≈ 5,079 ns            | 965,279          |
  | dev4    | ≈ 6,423 ns            | ≈ 6,426 ns            | 761,259          |

  dev4 adds ~1,350 ns per iteration (mean min-p99) and loses
  ~21 % of throughput. The per-iter overhead and the
  throughput loss are consistent (wall-time per iter:
  ≈ 5,180 ns dev3 vs ≈ 6,570 ns dev4 — matches the min-p99
  delta).

  Likely cause: `record(ticks)` wrote one atomic-increment into
  a bucket that fits in L1; the new path stores 24 bytes per
  sample into a growing `Vec<Record>` that reaches ~17 MB over
  5 s of recording — swamps L2 and triggers doubling
  reallocations. The Vec::push itself is cheap in isolation;
  the cumulative cache footprint + realloc memcpys are what
  bite in a high-rate loop.

  The earlier `-d 0.5` smoke showed dev4 *faster* — that was a
  shorter run where the buffer never grew into its
  cache-pressure regime. The 5 s run is the representative
  measurement.
- **Scope API is currently a flexibility-for-throughput
  trade.** What it buys: record-order info (TSC timeline,
  not just a histogram), optional per-site grouping later,
  optional background drain / ring buffer without API
  change, and a path to long-term trace export. What it
  costs today: high-rate hot-path throughput vs. the direct
  `record(ticks)` path. Mitigations are already listed as
  out-of-scope for 0.9.0 (pre-reserved capacity, ring
  buffer, background drain, per-site grouping); pick them up
  once a use case pulls.
- **Low-tail samples.** Both dev3 and dev4 show a `min-p1`
  band starting in the tens of ns (dev3: 10/50 ns; dev4:
  10/60 ns), consistent with round-trips that complete on
  an already-buffered channel (`recv` short-circuits without
  going to sleep). Not a regression.
- **Leaked ids on shutdown paths.** Both producer and
  consumer loops call `probe.start(0)`, then do channel I/O
  that can break on `Err`, and only call `probe.end(id)` on
  the happy path. On break paths the id is dropped without a
  matching `end`, which per Option-B design just means the
  in-flight sample is lost — no slot reservation to clean
  up, no panic. The buffer invariant (only complete records)
  is preserved.
- **`record(ticks)` retained deliberately.** For high-rate
  probes the direct-histogram path is currently the
  performance-preferred choice; keeping `record(ticks)`
  around gives callers that knob without forcing them onto
  the scope API. The `#[allow(dead_code)]` on it notes this
  pending a later decision.
- **Tests green in both debug and release.** All 13 still
  pass (`cargo test` and `cargo test --release`).

## Split TProbe2 + revert TProbe + tp2-pc (0.9.0-dev5)

Physically splits the two recording primitives that had been
fighting each other on `TProbe`:

- `TProbe` reverts to its 0.8.0 shape — direct-histogram only
  (`record(ticks)` + `report(&self)`). Fast path.
- `TProbe2` is new, in `src/tprobe2.rs`, and owns the scope API
  (`start` / `end` + records buffer + drain-in-report). The
  trade-off surface (cache footprint, future bounded buffer,
  background drain, per-site grouping) lives here without
  dragging the fast path into those decisions.
- `tp_pc` goes back to the fast path (dev3 shape).
- `tp2_pc` is a new bench — same workload as `tp_pc` but on
  `TProbe2`. Having both in the registry means `iiac-perf
  tp-pc tp2-pc -d 5` runs them in one process with shared
  calibration and thermal state, which is the only honest way
  to A/B the hot-path cost.

The dev4 chores section interpreted an across-invocation
comparison (dev3 user run → dev4 user run) as a ~21% scope-API
regression. In-process dev5 measurement (below) shows the real
delta is sub-1 %. The earlier gap was dominated by system-state
drift, not the scope API.

### Edits

- `Cargo.toml` — version bump to `0.9.0-dev5`.
- `src/band_table.rs` — **new**, shared `pub(crate) fn
  render(kind, name, hist, as_ticks)` extracted verbatim from
  the dev3-era `TProbe::report` body. Both primitives delegate
  to it so their table output is visually identical.
- `src/tprobe.rs` — reverted to 0.8.0 shape: `TProbe { name,
  hist }`, `new` / `record` / `report(&self, as_ticks)`. No
  scope API, no records buffer, no dead-code allows. Report
  delegates to `band_table::render`. Module doc points readers
  to `tprobe2` for the scope API.
- `src/tprobe2.rs` — **new**, holds `TProbe2 { name, hist,
  records }`, `TProbe2RecId`, internal `Record`, `new` /
  `start` / `end` / `report(&mut self, as_ticks)`. Report
  drains records before delegating to `band_table::render`.
  Tests moved over (`start_end_appends_one_record`,
  `start_end_preserves_start_tsc`,
  `start_end_interleaved_non_stack`,
  `report_drains_records_into_histogram`); dropped
  `record_and_start_end_are_independent` — moot once
  `record(ticks)` isn't on this type.
- `src/main.rs` — added `mod band_table;` and `mod tprobe2;`.
- `src/benches/tp_pc.rs` — hot loop reverted to the dev3 shape
  (`ticks::read_ticks()` pair + `probe.record(e.wrapping_sub(s))`).
  `let mut` on the two probe bindings dropped (report is
  `&self` again). Module doc restored to the pre-dev4 wording.
- `src/benches/tp2_pc.rs` — **new**, registers as CLI
  `tp2-pc`. Identical structure to `tp_pc` but uses
  `TProbe2::start(0)` / `TProbe2::end(id)`. `TProbe2`'s first
  non-test consumer, so no `#![allow(dead_code)]` needed on
  `tprobe2`.
- `src/benches/mod.rs` — `pub mod tp2_pc;` + `(tp2_pc::NAME,
  tp2_pc::run)` in `REGISTRY`.
- `notes/todo.md` — dev5 Done entry + reference `[32]`.
- `notes/chores-03.md` — this section.

### Findings

- **In-process A/B, short runs (-d 5).** `iiac-perf tp-pc
  tp2-pc -d 5` on the 3900X, `--pin` default:

  | bench   | loop count | producer min-p99 | consumer min-p99 |
  |:--------|-----------:|-----------------:|-----------------:|
  | tp-pc   | 656,306    | 7,468 ns         | 7,476 ns         |
  | tp2-pc  | 653,304    | 7,513 ns         | 7,523 ns         |

  Scope API ≈ 0.6 % slower throughput at 5 s. Small,
  noise-level.

- **In-process A/B, steady-state runs (-d 60).** Same box,
  same invocation style, longer duration:

  | bench   | loop count | producer min-p99 | consumer min-p99 | per-iter |
  |:--------|-----------:|-----------------:|-----------------:|---------:|
  | tp-pc   | 12,918,173 | 4,559 ns         | 4,558 ns         | 4,644 ns |
  | tp2-pc  | 16,575,811 | 3,536 ns         | 3,534 ns         | 3,619 ns |

  Scope API is **~22 % faster** throughput and ~1 µs lower
  per-sample at steady state — the opposite of what dev4
  suggested. Both benches also run faster at 60 s than at 5 s,
  consistent with the boost / thermal ramp effect characterized
  in the 0.6.0 calibration chores (the first few seconds of any
  run don't reach steady-state frequency).

- **The bot thinks** the scope API wins at steady state
  because `hist.record(delta)` does log-scale bucket-index
  compute + an atomic increment, whereas `Vec::push(Record)`
  is a 24-byte store + pointer bump that streams to DRAM at
  bandwidth-unbounded rates for this workload (276 K pushes/s
  is well under memory-subsystem ceilings, and the 400 MB
  buffer at 60 s pays in memory rather than cycles). Not
  measured directly — perf counters would be needed to
  confirm.

- **The dev4 "21 % regression" narrative was noise.** That
  comparison was across two separate invocations run minutes
  apart, with different system states (thermal, scheduler
  load, frequency). Run fairly in one process, the scope API
  is competitive at short durations and wins at long ones.
  dev4's chores should be read as "variance across
  invocations dwarfs the scope-API delta on this workload,"
  not "scope-API is slower."

- **Split stands on API-clarity grounds too.** Even without
  the steady-state perf win, keeping the primitives separate
  keeps the fast path and the scope API from trading off on
  each other's constraints (buffer growth, drain semantics,
  per-site grouping, &mut self requirements on report).
- **band_table extraction is a verbatim move.** The logic in
  `band_table::render` is the dev3 `TProbe::report` body with
  `self.hist` → `hist` and `self.name` → `name`. Output
  shape is unchanged; running tp-pc in dev5 produces the same
  row layout and column widths as in dev3.
- **`tprobe2`'s dead-code allow is gone.** `tp2-pc` consumes
  every pub item on `TProbe2`, so the module-level
  `#![allow(dead_code)]` we needed in a hypothetical
  consumer-less split is not required.
- **Tests green in debug and release.** 12 tests pass (the
  four `tprobe2::tests::*` and the eight `pin::tests::*`).
  `tprobe.rs` has no tests — matches its 0.8.0 shape.

## 0.9.0 release: TProbe2 scope API + tp2-pc (0.9.0)

Ships the 0.9.0 "scope-API probe" pass. See the five `-devN`
sections above for per-step detail. Net delivery:

- `TProbe` (unchanged from 0.8.0): fast-path probe, direct
  histogram; hot path is `record(ticks)` with a manual tick-read
  pair on the caller's side.
- `TProbe2` (new, `src/tprobe2.rs`): scope-API probe;
  `start(site_id) -> TProbe2RecId` / `end(id)` on the hot path
  write `(site_id, start_tsc, end_tsc)` records to an internal
  `Vec<Record>`. Delta math, `max(1)` clamp, and histogram
  ingestion are deferred to `report()`, which drains the buffer
  into the histogram before rendering.
- `band_table::render` (new, `src/band_table.rs`): shared
  tick-valued band-table renderer — both primitives delegate
  to it so output is visually identical.
- `tp-pc` stays on the fast path. `tp2-pc` (new bench) exercises
  `TProbe2` on the same workload; running
  `iiac-perf tp-pc tp2-pc -d N` produces an in-process A/B with
  shared calibration and thermal state.

Per-dev roll-up:

- `-dev1` ✅ plan: TProbe start/end (ideas.md Option B,
  deferred-processing, minimal hot path).
- `-dev2` ✅ implement `TProbe::start` / `end` + record buffer
  + 4 unit tests (on the pre-split TProbe).
- `-dev3` ✅ lazy `report()` drain: records → histogram.
- `-dev4` ✅ wired `tp-pc` to `start` / `end`; cross-invocation
  A/B surfaced what looked like a ~21% regression — later shown
  to be system-state drift, not scope-API cost (see dev5
  findings).
- `-dev5` ✅ physically split: `TProbe` reverted to 0.8.0,
  `TProbe2` in its own module, `tp2-pc` bench added, fast path
  restored on `tp-pc`. In-process A/B measurements corrected the
  dev4 narrative.
- `0.9.0` final — remove `-dev5`, bump Cargo.toml to `0.9.0`,
  move the five dev entries + release to todo's `## Done`.

### Capstone performance characterization

The scope API's hot-path cost vs. the direct-histogram path
varies by pinning regime. Three measurements on the 3900X,
`iiac-perf tp-pc tp2-pc`:

| regime                              | duration | tp-pc loops | tp-pc min-p99 | tp2-pc loops | tp2-pc min-p99 | stdev min-p99 (producer) | winner                |
|:------------------------------------|---------:|------------:|--------------:|-------------:|---------------:|-------------------------:|:----------------------|
| unpinned                            | 60 s     | 12,054,849  | 4,896 ns      | 16,088,230   | 3,649 ns       | ~2,900 ns                | tp2-pc, ~25 % ↑       |
| pinned 0,5,10 (cross-CCX, same-CCD) | 60 s     |  7,316,823  | 8,086 ns      |  7,589,776   | 7,806 ns       | ~300 ns                  | tp2-pc, ~3.5 % ↑      |
| pinned 5,6,7 (cross-CCD)            | 60 s     |  7,267,058  | 8,140 ns      |  7,311,354   | 8,107 ns       | ~330 ns                  | tp2-pc, ~0.5 % ↑ (tie) |

All three rows captured on 0.9.0 at -d 60. A 15-second version
of the `5,6,7` run during dev5 showed tp-pc ahead by ~2.5 %;
the 60-second version flips it to a statistical tie — which the
bot thinks is evidence that the shorter run was
pre-equilibrium noise, but not proof.

Observations:

- **tp2-pc wins or ties all three 60-second regimes.**
  Unpinned: ~25 % margin. Same-CCD pin (`0,5,10`): ~3.5 %
  margin. Cross-CCD pin (`5,6,7`): ~0.5 % margin — within
  one stdev, statistical tie. No regime at 60 s shows tp-pc
  ahead.
- **Pinning absolute min-p99 goes up, spread goes down.**
  Unpinned producer min-p99 ≈ 4,900 ns with stdev ≈
  2,900 ns; pinned (either topology) ≈ 8,100 ns with stdev
  ≈ 320 ns. Absolute latency nearly 2× higher, stdev about
  an order of magnitude tighter. The 0.6.0 calibration
  chores already characterized the tightening for framing;
  probe measurements reconfirm it. Orthogonal to probe
  choice.
- **Cross-CCD vs same-CCD is small at 60 s.** Absolute tp-pc
  min-p99 is 8,086 ns (same-CCD) vs 8,140 ns (cross-CCD)
  — ~50 ns apart, loop counts within ~0.7 %. Most of the
  unpinned→pinned jump happens on any thread separation; the
  additional CCD crossing is a minor adder at this workload.
- **The dev4 "21 % regression" was cross-invocation drift.**
  Two separate runs minutes apart, different thermal /
  frequency / background-load states — not an
  apples-to-apples probe comparison. Running both benches in
  one invocation is the only honest A/B and gives the
  numbers above.

A mechanism-level explanation (why scope API wins at steady
state, why the margin shrinks with pinning, what drives the
~50 ns cross-CCD adder) would need microarch measurements —
L1/L2 miss rates, atomic contention, DRAM bandwidth
saturation on the record buffer, Infinity Fabric latency
counters. The data above shows the *what*; the *why* is
speculative without those.

### Memory characterization

`TProbe2` keeps every sample as a 24-byte `Record` in an
unbounded `Vec` until `report()`. At the unpinned -d 60 rate,
that's ~400 MB for a single probe. Well inside a typical
workstation's RAM but far past L3 — which is fine as long as
the write pattern stays streaming (it does). Two mitigations
are listed as out-of-scope for 0.9.0 and remain so:

- Pre-reserved capacity (`Vec::with_capacity`) to eliminate
  doubling-realloc tail spikes.
- Bounded / ring-buffer policy for long-running or unbounded
  recording.

### When to use which probe

Data-driven guidance:

- `TProbe` — when memory footprint is the binding constraint.
  RAM use is bounded by the fixed HDR histogram size
  regardless of sample count, so very long or very high-rate
  recording doesn't grow.
- `TProbe2` — default choice for new measurement code. Ties
  or wins tp-pc on throughput in every 60-second regime
  measured (unpinned +25 %, same-CCD +3.5 %, cross-CCD tie),
  so no hot-path reason to prefer the direct-histogram path.
  Required when you need record-order information (TSC
  timeline), or expect per-site grouping / background drain /
  trace retention to matter later. Cost is the growing record
  buffer (24 bytes per sample, unbounded until `report()`).

### Edits

- `Cargo.toml` — version bump `0.9.0-dev5` → `0.9.0`.
- `notes/todo.md` — `0.9.0` Done entry + reference `[33]`;
  cleared the In Progress scope-API task.
- `notes/chores-03.md` — this section.

No source changes in this final commit; the release marker
consolidates the five dev commits.

## Plan: zero-copy IPC bench — zc-ring + zerocopy (version TBD)

Sketch of a bench measuring zero-copy IPC with `TProbe2`,
working both intra-process (two threads) and inter-process
(two processes over shared memory), with the transport core
kept `no_std`.

### Stack

Survey conclusion: no existing crate covers inter+intra
process, zero-copy, `no_std` at once. `iceoryx2` is
inter-process but `std`; `bbqueue` is `no_std` zero-copy but
its handles hold pointers into one address space, so
intra-process only. The gap is filled by a small hand-rolled
ring:

- **`zc-ring`** (new crate, `no_std`): SPSC ring operating on
  a caller-provided raw byte region. All state
  offset-addressed (position-independent — the region may map
  at different addresses in each process); coordination via
  atomic read/write indices living in a header inside the
  region itself. Zero-copy via bbqueue-style grant API:
  producer gets a write grant (slice into the region), fills
  in place, commits; consumer gets a read grant, reads in
  place, releases.
- **`zerocopy`** (Google's safe-transmutation crate, `no_std`):
  types the payload bytes. Producer writes via `IntoBytes`,
  consumer reads via `FromBytes::ref_from_bytes` directly on
  the grant slice. `FromBytes` types are valid for any bit
  pattern and pointer-free, which compiler-enforces exactly
  the discipline shared memory needs.
- **`memmap2`** (`std`, bench harness only): the inter-process
  substrate — a `memfd_create` region mapped in parent and
  child.

### Crate layout

The ring lives in its own repo, developed and tested there
first; this repo gains benches only once the ring is
operational. No workspace conversion here.

Ring crate — separate repo at `../zc-ring-x1`:

```
zc-ring-x1/
  Cargo.toml          # no_std, deps: none (core only)
  src/
    lib.rs            # #![no_std]; public Producer/Consumer/split API
    header.rs         # region header layout: atomic indices, capacity,
                      #   magic/version (all offsets, no pointers)
    grant.rs          # WriteGrant / ReadGrant (in-place slices)
  tests/              # std tests: loopback, wraparound, torn-write
```

This repo, later step — bench additions only:

```
Cargo.toml            # + deps: zc-ring-x1 (path = "../zc-ring-x1"),
                      #   zerocopy, memmap2
src/benches/
  zcr_1p.rs           # intra-process: Box<[u8]> region, 2 threads,
                      #   TProbe2 on send/recv scopes
  zcr_2p.rs           # inter-process: memfd + memmap2, re-exec child,
                      #   same ring code, same probes
  zcr_common.rs       # shared payload types (zerocopy derives) +
                      #   round-trip helpers for both benches
```

Notes on the sketch:

- Same ring code in both benches; only the memory substrate
  changes (heap vs shm). That isolates the substrate cost —
  the interesting comparison.
- `zcr-2p` needs a second process: the bench binary re-execs
  itself with a hidden subcommand (child inherits the memfd
  fd), keeping everything in one binary.
- Inter-process probe collection: each process runs its own
  `TProbe2` and reports separately (records are
  process-local); child writes its report to a pipe or the
  parent prints both.

Sequencing: ring development happens in `zc-ring-x1` under
that repo's own versioning. Work in this repo starts when the
ring is operational, as its own plan: add the path dependency
plus `zcr-1p` / `zcr-2p` benches. This section is the
cross-repo design record; the version is assigned when that
plan starts (`0.10.0`, tentatively named here earlier, was
taken by the iceoryx2 benches).

## Plan: iceoryx2 benches — pub/sub + req/res, 1t/2t (0.10.0-dev1)

Multi-step plan adding four benches that measure `iceoryx2`
(shared-memory IPC middleware, v0.9.2) inside one process, in
both of its messaging patterns, at one and two threads. The
interesting comparison is (a) iceoryx2's two patterns against
each other and (b) shm-backed transport against the in-process
channels (`mpsc-1t`/`mpsc-2t`).

### Feasibility (measured, scratchpad prototype)

A release-mode prototype on iceoryx2 0.9.2 / Rust 1.96.1
confirmed single-process operation of pub/sub in both shapes:

- 1t loopback (publish then spin-receive, same thread):
  ~259 ns/iter
- 2t round-trip (main → echo worker → main, two services,
  spin-receive): ~947 ns/iter

The request/response pattern (`request_response::<u64, u64>()`,
`client_builder`/`server_builder`) exists in 0.9.2 and is
API-verified from crate docs, not yet prototyped.

### Benches

Four registry entries, `Bench`-trait style mirroring
`mpsc-1t`/`mpsc-2t` (`step()` = one round-trip, harness
`run_adaptive`):

- `ice-ps-1t` — pub/sub, same thread: `send_copy` then
  spin-`receive` on one service.
- `ice-ps-2t` — pub/sub, echo worker: two services (req/resp
  direction each), worker spin-receives and echoes; main
  measures the round-trip. Worker pinned via `cfg.core_for(1)`
  like `mpsc-2t`.
- `ice-rr-1t` — request/response, same thread: `Client`
  and `Server` on one service; `client.send_copy` →
  `server.receive` → `active_request.send_copy` →
  `pending_response.receive`.
- `ice-rr-2t` — request/response, echo worker: worker holds
  the `Server`, main holds the `Client`; one service carries
  both directions (the pattern's structural advantage over
  pub/sub, which needs two).

### Constraints observed in the prototype

- `Subscriber::receive` is non-blocking only; benches spin.
  The 2t numbers therefore measure the spin-spin fast path,
  comparable to a hot `mpsc-2t`, not park/wake cost.
- iceoryx2 leaves a persistent global-management segment in
  `/dev/shm` and `/tmp/iceoryx2/{services,nodes}` dirs across
  runs; clean exits tear services down, but a killed run can
  leave stale services. Bench doc comments should note this.
- Service names are machine-global; benches use
  `iiac-perf-…`-prefixed names to avoid collisions.
- One cosmetic startup warning ("No config file was loaded")
  prints once per process; accepted.
- Dependency weight: ~79 transitive crates, ~16 s cold
  release build for the dep graph; accepted for a bench crate.

### Steps

- `0.10.0-dev1` — this plan (docs only) + version bump.
- `0.10.0-dev2` — add `iceoryx2` dependency; implement
  `ice-ps-1t` + `ice-ps-2t`; register.
- `0.10.0-dev3` — implement `ice-rr-1t` + `ice-rr-2t`;
  register.
- `0.10.0` — release: README bench list, todo Done entries,
  chores release section; capture measured numbers.

### Edits

- `Cargo.toml` — version bump `0.9.0` → `0.10.0-dev1`.
- `notes/chores-03.md` — this section; retitled the zc-ring
  plan header to "(version TBD)" and noted `0.10.0` was taken
  by this work.
- `notes/todo.md` — In Progress entry + reference `[34]`.

## Implement ice-ps-1t + ice-ps-2t (0.10.0-dev2)

Implements the pub/sub half of the 0.10.0-dev1 plan: `iceoryx2`
dependency plus the `ice-ps-1t` / `ice-ps-2t` benches, registered
after `tp2-pc`. `Bench`-trait style mirroring `mpsc-1t`/`mpsc-2t`.

First 2 s runs (unpinned, adjusted mean): `ice-ps-1t` ~242 ns,
`ice-ps-2t` ~686 ns. For comparison the same-shape prototype gave
259 / 947 ns; the bot thinks the 2t delta is run-to-run thermal /
placement variance, not a code difference worth chasing.

### Findings

- **Lost-first-sample hang.** Pub/sub here has no history:
  a sample published before the peer's subscriber connects is
  silently dropped. `ice-ps-2t`'s first `step()` raced the worker's
  service setup, lost the request, and spun forever on `receive`.
  Fix: constructor handshake — re-ping (1 ms cadence) until an
  echo arrives, then drain the duplicate echoes.
- **Node-before-ports drop order.** First cut declared the
  `Node` field before the port fields; Rust drops fields in
  declaration order, and dropping the node while its ports live
  trips iceoryx2's dead-node detection — the next node creation
  printed a huge `SharedNodeState` `[W]` dump. Fix: declare ports
  first, node last, with a comment pinning the order.
- **Config-file warning suppressed via explicit config.** The
  plan accepted the cosmetic `[W] "No config file was loaded"`
  line, but source reading showed it fires only from
  `Config::global_config()` lazy init, which `NodeBuilder` reaches
  only when built without an explicit config. Passing
  `.config(&Config::default())` at every node build removes the
  warning with identical behavior (the fallback *is*
  `Config::default()`), and keeps the bench hermetic: iceoryx2
  otherwise reads `./config/iceoryx2.toml` (cwd-relative),
  `~/.config/iceoryx2/iceoryx2.toml`, or `/etc/iceoryx2/…`, so a
  machine-local file could silently change what the bench
  measures.

### Edits

- `Cargo.toml` — version bump to `0.10.0-dev2`; add
  `iceoryx2 = "0.9.2"`.
- `src/benches/ice_ps_1t.rs` — new: same-thread
  `send_copy` → spin-`receive` on one pub/sub service; pid in
  the service name to keep concurrent runs from colliding;
  explicit default config per Findings.
- `src/benches/ice_ps_2t.rs` — new: echo worker over two
  services (one per direction); worker opens the services through
  its own node, as a second process would; worker pinned via
  `cfg.core_for(1)`; `AtomicBool` stop flag + join in `Drop`;
  constructor handshake + explicit default config per Findings.
- `src/benches/mod.rs` — register both after `tp2-pc`.
- `CLAUDE.md` — pre-commit checklist: install + retest must run
  manually before `vc-x1 push` (its preflight covers only
  fmt/clippy/test).
- `notes/todo.md` — dev2 Done entry + reference `[35]`.
- `notes/chores-03.md` — this section.

## Implement ice-rr-1t + ice-rr-2t (0.10.0-dev3)

Implements the request/response half of the 0.10.0-dev1 plan:
`ice-rr-1t` / `ice-rr-2t`, registered after `ice-ps-2t`. Same
`Bench`-trait shape as the pub/sub pair; one service carries both
directions (vs. one per direction for pub/sub), with the reply
routed through the `PendingResponse` handle each request returns.

First 1 s runs (unpinned, adjusted mean) alongside dev2's pub/sub
numbers:

| bench     | 1t      | 2t        |
|-----------|---------|-----------|
| ice-ps    | ~250 ns | ~650-690 ns |
| ice-rr    | ~750-850 ns | ~1,100-1,140 ns |

Request/response costs roughly 3× pub/sub at 1t and ~1.7× at 2t.
The bot thinks the per-request `PendingResponse` machinery
(request-id allocation and response routing) accounts for the
gap; pinning down the split is release-step material if it
matters.

### Findings

- Both benches worked on the first build — the dev2 findings
  (handshake against the lost-first-request race, ports-before-
  node field order, explicit default config) were applied from
  the start and no new failure mode appeared. The rr handshake
  differs slightly from ps: each retry's `PendingResponse` is
  simply dropped, which closes that request cleanly, so no
  post-handshake drain is needed.

### Edits

- `Cargo.toml` — version bump to `0.10.0-dev3`.
- `src/benches/ice_rr_1t.rs` — new: same-thread client → server
  → client on one req/res service; pid-suffixed service name;
  explicit default config.
- `src/benches/ice_rr_2t.rs` — new: client on main, echo server
  on the worker (own node, as a second process would); worker
  pinned via `cfg.core_for(1)`; `AtomicBool` stop flag + join in
  `Drop`; constructor handshake.
- `src/benches/mod.rs` — register both after `ice-ps-2t`;
  `resolve` falls back to prefix matching — a requested name with
  no exact match runs every bench it is a prefix of (`ice` → all
  four ice benches), in registry order; error only when a name
  matches nothing.
- `src/main.rs` — CLI doc comment for the prefix-match behavior.
- `README.md` — prefix-match note in the usage section.
- `src/benches/ice_{ps,rr}_{1t,2t}.rs` — report titles now lead
  with the registry name (`ice-ps-1t: iceoryx2 pub/sub …`),
  matching the `tp-pc` convention, so prefix-expanded runs map
  reports back to CLI names. The older benches (mpsc-*, …) don't
  do this yet; candidate release-step or follow-on cleanup.
- `notes/todo.md` — dev3 Done entry + reference `[36]`.
- `notes/chores-03.md` — this section.

## 0.10.0 release: iceoryx2 benches (0.10.0)

Release marker consolidating dev1–dev3: iceoryx2 0.9.2
dependency, four benches (`ice-ps-1t`, `ice-ps-2t`, `ice-rr-1t`,
`ice-rr-2t`), CLI prefix matching, and the report-title
convention (titles lead with the registry name).

### Measured (5 s per bench, unpinned, adjusted mean, 3900X)

| bench     | adjusted mean | note                          |
|-----------|--------------:|-------------------------------|
| mpsc-1t   |         23 ns | anchor: in-process channel    |
| mpsc-2t   |      8,027 ns | anchor: park/wake round-trip  |
| ice-ps-1t |        250 ns | pub/sub, one service          |
| ice-ps-2t |        632 ns | pub/sub, two services, spin   |
| ice-rr-1t |        789 ns | req/res, one service          |
| ice-rr-2t |      1,025 ns | req/res, one service, spin    |

Pattern comparison: request/response costs ~3× pub/sub at 1t and
~1.6× at 2t despite carrying both directions on one service. The
bot thinks the per-request `PendingResponse` machinery
(request-id allocation + response routing) accounts for the gap.
The mpsc-2t anchor is not apples-to-apples with the spin-spin ice
2t benches — `recv()` parks, so it prices wake latency, not
transport.

### Edits

- `Cargo.toml` — version `0.10.0-dev3` → `0.10.0`.
- `README.md` — intro paragraph: `ice-*` bench family mention.
- `CLAUDE.md` — User approval section: approval is of the exact
  commit text; present the full command incl. title/body and
  execute only that verbatim. Added after dev3's push ran with
  inline-composed text the user never saw. Also (from dev2-dev3
  review) checklist step 6: install + retest run manually before
  `vc-x1 push`.
- `notes/todo.md` — In Progress cleared; `0.10.0` Done entry +
  reference `[37]`.
- `notes/chores-03.md` — this section.

## mpsc-2t-spin bench (0.11.0)

Single-step change adding `mpsc-2t-spin`: the `mpsc-2t` round-trip
with both ends spinning on `try_recv` instead of parking in
`recv`. This fills a cell of the {transport} × {wait policy}
comparability matrix — the 0.10.0 ice 2t benches spin, `mpsc-2t`
parks, so the two differed in two dimensions at once and their
numbers weren't attributable to either. With the wait policy held
equal:

| bench        | adjusted mean | isolates                     |
|--------------|--------------:|------------------------------|
| mpsc-2t      |      7,621 ns | + park/wake over spin        |
| mpsc-2t-spin |        121 ns | in-process transport, spin   |
| ice-ps-2t    |        629 ns | shm transport, spin          |

(5 s per bench, unpinned, adjusted mean, 3900X, one run.)

Read: park/wake costs ~7.5 µs per round-trip on this machine —
two orders of magnitude over the transport itself — and iceoryx2's
shm queue costs ~5× the in-process mpsc queue under an identical
spin policy. The remaining matrix cell (iceoryx2 with a blocking
wait via its `Listener`/`Notifier` events) stays in todo.

### Findings: reconciling ice-ps-2t with iceoryx2's claims

The ~5× gap over `mpsc-2t-spin` looked wrong against iceoryx2's
performance reputation, so it was cross-checked instead of taken
on faith:

- Pinning is not the cause: `--pin 0,1` and `--pin 0,12` left
  `ice-ps-2t` at ~630-660 ns while `mpsc-2t-spin` dropped to
  ~68 ns on SMT siblings. The gap is per-operation work.
- iceoryx2's own pub/sub benchmark (repo v0.9.2,
  `benchmark-publish-subscribe --bench-all`), built and run on
  this 3900X, reports ~250 ns latency for both `ipc` and `local`
  service variants (8 KiB samples, 10M iterations).
- Their number is **one-way** (`elapsed / (iterations × 2)`), so
  ~500 ns round-trip — the same ballpark as our 629-699 ns.
  Remaining delta: their harness runs threads pinned at realtime
  priority 255, never writes the payload in default mode
  (`loan_slice_uninit` + `assume_init`, signaling only), and
  reuses the loan-based zero-copy send path; our bench pays
  `send_copy` and writes/reads a real `u64`.
- The bot thinks the right framing is: iceoryx2's performance
  claim is payload-size-independent latency (zero-copy; their
  250 ns holds at 8 KiB where a channel would copy twice) and
  beating classic IPC (unix sockets ~10-20 µs RT) — not beating
  an in-process channel at 8-byte ping-pong, which is a single
  atomic queue op with no cross-process generality to pay for.

Follow-on candidates recorded in todo: loan-based zero-copy send
path in the ice benches; payload-size sweep (8 B / 8 KiB / 1 MiB)
to make the size-independence visible in our own tables.

### Late edits (post-review, same commit)

- `README.md` — new "`all` results (3900X, 0.11.0)" subsection
  under Example runs: one-run adjusted-mean table for all 13
  benches + the wait-policy/iceoryx2-reconciliation context.
- `notes/chores-03.md` — the Findings subsection above.
- `notes/todo.md` — loan-path + payload-size-sweep Todo entries.

### Edits

- `Cargo.toml` — version bump `0.10.0` → `0.11.0`.
- `src/benches/mpsc_2t_spin.rs` — new: `mpsc-2t` structure with
  `try_recv` spin loops on both ends; same dummy-sender Drop
  shutdown; report title leads with the registry name.
- `src/benches/mod.rs` — register after `mpsc-2t`.
- `notes/todo.md` — `0.11.0` Done entry + reference `[38]`;
  Todo entry for the `ice-ps-2t-wait` matrix cell.
- `notes/chores-03.md` — this section.
