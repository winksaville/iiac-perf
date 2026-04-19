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
