# Ideas

Design sketches that aren't yet scheduled work. Each section captures the state
of a discussion so it can be picked up later without re-deriving the reasoning.

## Tprobe (time probe)

### Motivation

Rename/refactor of the current `Probe` primitive. Two goals:

1. **Production instrumentation, not just benchmarking.** Drop tprobes into
   production code to surface bottlenecks and critical paths that weren't
   flagged as critical up front.
2. **Minimal hot-path overhead.** Record raw TSC, defer math to the report
   phase.

### Record shape

Each tprobe record holds four pieces of information:

- `site_id` — identifies the call site (ideally auto-generated at compile time,
  see below).
- `start_tsc` — TSC reading at scope entry.
- `end_tsc` — TSC reading at scope exit.
- `userinfo` — opaque application-attached payload. For an actor, the runtime
  records one entry per msg-handler invocation and the actor itself fills in
  whatever is useful during the report/save phase.

Raw TSC + deferred math keeps the hot path to a `rdtscp` pair plus an append.
Converting TSC → ns happens once at report time from a calibration reading.

### API shape

Three access patterns, all built on the same primitive:

1. **Auto-scope (preferred where possible).** For constructs with a
   well-defined scope — e.g. an actor msg handler — the runtime opens and
   closes the tprobe automatically. User code does nothing.
2. **Scope guard.** `let _g = tprobe::scope!();` expands to
   `Guard(start!())` with `Drop` calling `end`. Correct by construction for
   straight-line scopes; the common case when auto-scope doesn't fit.
3. **Raw start/end.** `let id: TprobeRecId = tprobe::start!();` … later …
   `tprobe::end(id, userinfo)`. Escape hatch for cases the guard lifetime
   can't cover: cross-thread close, async, conditional close, **arbitrary
   nesting that isn't stack-structured** (e.g. `start1 start2 end1 start3
   end3 end2`).

start/end are the basic elements; guard and auto-scope are sugar on top.

### Id shape — the load-bearing decision

`TprobeRecId` is an opaque handle returned by `start()`. It resolves
"which scope am I closing?" when multiple scopes are open simultaneously
(overlapping, cross-thread, async, non-stack nested). It does **not** by
itself enforce "always closed" — a leaked id leaves an open record — which
is why the guard is the recommended form when the scope is straight-line.

Two candidate implementations:

**Option A — slot reservation.** `start()` allocates a buffer slot, writes
`start_tsc` + sentinel `end_tsc`, returns `(slot, generation)`. `end(id)`
writes `end_tsc` into the slot.

- Drain must skip open slots → head-of-line blocking if one scope is slow.
- Ring-buffer overwrite of an open slot needs the generation to detect.
- Multiple open slots need non-contiguous reservation; without stack
  discipline you can't walk a reservation stack.

**Option B — id carries start.** `TprobeRecId = (site_id, start_tsc)`, ~16
bytes, no slot allocated at start. `end(id, userinfo)` appends one complete
record at close time.

- Buffer only ever holds complete records — drain is trivial, no
  head-of-line blocking, no staleness.
- Arbitrary interleaving (non-symmetric nesting) just works: each end is
  independent.
- Userinfo must be supplied at `end()` time, not incrementally.

**Decision: Option B.** Non-symmetric nesting and drain-to-outside both
push strongly toward B. A background task forwards the stream out of the
process; having it see only complete records makes the forwarding task
trivial.

### Auto-generated site_id (compile-time, zero user effort)

Ideal: the macro captures the call site; the user writes nothing extra.

Recipe — const hash of `file!()`/`line!()`:

```rust
macro_rules! tp_start {
    () => { $crate::start(const { $crate::site_hash(file!(), line!()) }) }
}
```

- `site_id` is a `u64` computed at compile time.
- No runtime registration, no `OnceLock` branch on the hot path.
- For symbolic reports (so output says `foo.rs:123` not `0xabc...`), the
  same macro emits `(hash, file, line)` into an `inventory`/`linkme`
  distributed slice. Runtime uses the hash; report phase walks the slice to
  resolve names. Both are free in the hot path.

**Caveat:** `file!()` is stable within a build but may vary between builds
(CI vs. local checkout paths). Fine in-process. If cross-build id stability
matters (e.g. comparing "site X got slower between v1 and v2"), normalize
paths relative to `CARGO_MANIFEST_DIR` before hashing.

### Stream forwarding

A background task drains the tprobe stream and forwards it outside the
app. Because Option B guarantees only complete records hit the buffer, the
drain is a straight pull-and-forward — no filtering for open slots, no
generation checks, no head-of-line concerns.

### Open questions

- **Buffer policy.** Ring (overwrites oldest — good for "what just
  happened?"), bounded + drain (full traces, needs back-pressure story),
  or pluggable. Shapes the `record()` failure mode: block, overwrite,
  return `Err`?
- **userinfo representation.** Generic `<U>` is fast and type-safe but
  pins one type per probe. Fixed-size POD slot (`[u8; N]`) keeps records
  uniform across actors with different needs, no heap. `Box<dyn Any>` is
  dead in the hot path.
- **TSC portability.** Invariant TSC is safe on modern x86 if the thread
  is pinned (mpsc-2t already does this); unpinned threads see small
  backward jumps across cores. Decide whether the primitive abstracts the
  time source (x86 `rdtscp`, aarch64 `cntvct_el0`, `Instant` fallback) or
  starts x86-only.
- **Double-end detection.** Should `end(id)` detect a double-close? Extra
  branch in the hot path vs. letting a real bug class survive. Decide now,
  not later.
- **Leaked ids.** Option B makes leaks cheap (no reserved slot sitting
  open) but still loses the record. Worth making `TprobeRecId`
  `#[must_use]` and noisily warning in debug builds.
