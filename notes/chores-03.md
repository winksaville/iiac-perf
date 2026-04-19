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
