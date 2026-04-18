# Chores-02

Continuation of `chores-01.md`, which crossed 500 lines. Same format;
see [Chores format](README.md#chores-format).

## Calibration robustness (0.6.0)

Calibration in `overhead.rs` occasionally produces a framing value
exactly 2× the typical — e.g. 11.11 ns one run, 22.22 ns the next,
on `iiac-perf mpsc-2t --pin 5,10 -D 1` (3900X). Not a physical 2×
effect — it's the arithmetic of the two-point fit amplifying ~10 ns
of endpoint noise.

### Analysis

Two-point fit:

```
framing  = min_low − N_LOW · (min_high − min_low) / (N_HIGH − N_LOW)
d(framing)/d(min_low)  = N_HIGH / (N_HIGH − N_LOW) = 1000 / 900 ≈ 1.11
d(framing)/d(min_high) = −N_LOW / (N_HIGH − N_LOW) = −100 / 900 ≈ −0.11
```

Reverse-engineering two observed runs (3900X, `--pin 5,10`, `-D 1`):

- Run A: `min_low = 60 ns, min_high = 500 ns` → framing 11.11 ns,
  loop/iter 0.4889 ns.
- Run B: `min_low = 70 ns, min_high = 500 ns` → framing 22.22 ns,
  loop/iter 0.4778 ns.

Only `min_low` moved (+10 ns). The fit amplified that by 10/9 = 1.11
→ +11.11 ns on framing. 11.11 + 11.11 = 22.22. The "exact 2×" is
numerology; the real defect is ~10 ns of slop on `min_low`.

Why `min_low` floats but `min_high` is stable: `measure(N_LOW=100)`
runs first, right after a mere 1,000-iter warmup (~500 µs of work).
That's too short to guarantee the CPU has reached peak boost. By the
time `measure(N_HIGH=1000)` runs, boost is fully ramped, so
`min_high` reliably hits its ~500 ns floor.

### --pin vs unpinned main thread

`main.rs` already pins the main thread to `pin_cores.first()` before
calibration. Measured on 3900X, `-d 3`:

| configuration             | framing    | stdev      | p99.99 tail |
|---------------------------|-----------:|-----------:|------------:|
| `--pin 5,10`              |  11.11 ns  |    787 ns  |    272 µs   |
| no `--pin` (main unpinned)|  27.78 ns  | 12,967 ns  |    5.5 ms   |

So when `--pin` is omitted, calibration itself inherits the unpinned
jitter (migrations, frequency wobble, CCX changes), and the framing
number is clearly elevated beyond the true floor. The adjustment we
subtract is then derived from a different measurement regime than the
one we want.

### Should main be unpinned after calibration?

Functionally irrelevant. When bench threads run, the main thread
blocks in `pthread_join` — it's parked, not runnable, so its affinity
doesn't compete with anything. Keeping or dropping the pin changes
nothing observable.

The cleaner contract is:

- Calibration always runs pinned (stable environment, reliable
  framing/loop numbers).
- Bench runs per `--pin` (user's requested environment).
- Main's pinning during the bench phase is a don't-care.

So: leave main's pin state alone after calibration; don't add a
ceremonial unpin step unless a concrete reason emerges later.

### Recommended changes

1. **Pin main for calibration regardless of `--pin`.** Even when the
   user didn't specify `--pin`, briefly pin main (to `pin_cores[0]`
   if set, otherwise a fixed default such as core 0, or a new
   `--cal-pin` flag) so calibration always runs in a stable
   environment. Biggest coherence win — calibration stops depending
   on how the bench is configured.

   Add a `--no-pin-cal` flag that reverts to the pre-0.6.0 behavior
   (skip the calibration-time pin, so main stays unpinned when
   `--pin` is absent). Makes A/B comparison easy — run the same
   bench with and without `--no-pin-cal` to see the calibration
   difference directly, and gives a quick escape hatch if the new
   default ever misbehaves on someone else's box.

   **Resolved in dev6** — semantics of `--pin` + `--no-pin-cal`:
   `--no-pin-cal` now always wins, regardless of `--pin`. With
   `--pin 5,10 --no-pin-cal`, bench threads still pin to 5,10 but
   calibration runs with main unpinned (affinity = full startup
   mask). Lets the user decouple the calibration environment from
   the bench pinning pool when they want the bench pool fixed but
   the cal free-running.
2. **Longer warmup.** Bump `CAL_WARMUP` from 1,000 to ~100,000
   iterations so boost ramp completes before `measure(N_LOW)` runs.
3. **More samples.** Bump `CAL_SAMPLES` from 10,000 to ~100,000 so
   the min-of-samples estimator reliably hits the theoretical floor.
4. **Wider N spread.** Raise `N_HIGH` to ~10,000 (or drop `N_LOW` to
   ~10). This drops the noise-amplification coefficient on `min_low`
   from 1.11× to ~1.01×, so any remaining endpoint noise barely
   perturbs framing.
5. *(Optional)* **Sanity-check retry.** Run calibration 3–5 times;
   require framing/loop to agree within a tolerance. Retry or warn
   otherwise. Belt-and-suspenders for the rare bad run.

### Interpretation note: the low min-p1 / p1-p10 bands are real

Unpinned `mpsc-2t` runs show a very low `min-p1` band — e.g. `first
= 240 ns, last = 6,003 ns` in the 3s unpinned run above, while the
pinned `--pin 5,10` run's `min-p1` sits at `7,935…8,175 ns`. That
gap isn't calibration noise or a measurement artifact.

When both round-trip endpoints stay hot (scheduler co-locates them
on the same CCX, neither parks in a futex, both spin on the
channel), `std::sync::mpsc` — `crossbeam-channel` under the hood on
modern Rust — can round-trip in well under a microsecond. Pinning
to `5,10` on a 3900X likely forces cross-CCX placement (cores 5 and
10 are on different chiplets), which adds cache-coherence latency
and raises the floor. So the low bands in unpinned runs are a real
operating regime — "both ends hot and spinning" — not a glitch, and
the calibration rework here should not distort them.

Multi-step probably fits better because (1)–(4) are independent and
we'll want a before/after measurement at each step. Rough order:

- `0.6.0-dev1` ✅ chore marker: bump version, write this plan,
  update todo.
- `0.6.0-dev2` ✅ pin main for calibration regardless of `--pin`;
  add `--no-pin-cal` opt-out that restores pre-0.6.0 behavior.
  Relabel startup banner: `pinning` → `bench pin`, and add a new
  `cal pin` line. Dev2 fixes coherence only — framing/loop values
  still show run-to-run variance (expected; dev3/dev4 address
  stability).
- `0.6.0-dev3` ✅ longer warmup (`CAL_WARMUP` 1k → 100k) + more
  samples (`CAL_SAMPLES` 10k → 100k). Empirically dropped the
  framing floor (~11 → ~5 ns baseline) by ensuring boost ramp
  completes and the min-of-samples estimator reliably hits the
  hardware floor. Run-to-run spread is still ~10 ns (4.44 to
  14.44 across 10 runs of `mpsc-2t --pin 5,10 -d 0.5`) — the
  amplification coefficient is unchanged, so dev4 is still needed.
- `0.6.0-dev4` ✅ widened `N_HIGH` 1_000 → 10_000 (`N_LOW` held at
  100), dropping the amplification coefficient from 10/9 ≈ 1.11 to
  10_000/9_900 ≈ 1.01. Empirical: `mpsc-2t --pin 5,10 -d 0.5`
  across 10 consecutive runs shows steady-state framing at 5.47 ns
  with spread ≈ 0.3 ns (was 4.44–14.44 in dev3). First run or two
  after cold start still show the ~6 ns lift (CPU coming out of a
  deep C-state — warmup budget is still short relative to C6 exit).
  Calibration time ~60 ms → ~510 ms (one-time startup cost).
- `0.6.0-dev5` ✅ verbose/log infrastructure: added `-v`/`--verbose`
  flag, `log` + `env_logger` deps, `info!`/`debug!` coverage of
  affinity lifecycle, calibration params, raw min_low/min_high,
  fit values, and cal wall time. No behavior change. `RUST_LOG`
  overrides the flag when set. Added `pin::current_affinity` and
  `pin::affinity_summary` for observability; exposed cal params
  (`CAL_WARMUP`, `CAL_SAMPLES`, `N_LOW`, `N_HIGH`) as pub consts;
  extended `Overhead` with `cal_min_low_ns`, `cal_min_high_ns`,
  `cal_duration` so main can log them. Also fixed `iiac-perf -h`
  to show the version in its first line (via compile-time
  `concat!(env!("CARGO_PKG_VERSION"), …)` in the clap `about`).
- `0.6.0-dev6` ✅ unpin-after-cal + `pin::save_affinity` /
  `restore_affinity` helpers; wire save → pin → cal → restore so
  unpinned benches regain the sub-µs spin-spin fast path.
  Helpers log their action at `info` level — visible under `-v`
  as `save_affinity: mask=0-23 (24 cpus)` before pin and
  `restore_affinity: mask=0-23 (24 cpus)` after cal. Only active
  when `--pin` is absent *and* `--no-pin-cal` is absent; the
  other combinations are untouched. Also rewords the banner:
  `core 0 (default; --no-pin-cal to disable)` →
  `core 0 (unpinned after cal; --no-pin-cal to skip)` so actual
  behavior is explicit.

  Also resolved the dev2 open question: `--no-pin-cal` now
  always wins, even when `--pin` is set. Previously `--pin`
  would overrule `--no-pin-cal` and main was still pinned to
  `pin[0]`; now `--pin 5,10 --no-pin-cal` pins bench threads
  to 5,10 but runs cal with main on the full startup mask.
  CLI help reworded to reflect that `--no-pin-cal` is not a
  no-op when `--pin` is set.

  Empirical (3900X, `mpsc-2t -d 3`, unpinned bench):
  min-p1 first=200 ns (was first=3,507 ns pre-dev6 default);
  stdev ≈ 2,300 ns (tail restored to the pre-fix unpinned regime
  — the scheduler-co-location fast path now dominates).
- `0.6.0-dev7` *(was dev5, optional)* — sanity-check retry loop.
  Skipped: dev4's N_HIGH widening delivered ~0.3 ns steady-state
  spread on framing, which is already tight enough that
  retry-on-outlier logic would be complexity without benefit.
  Revisit only if data from other machines shows instability.
- `0.6.0` final ✅ remove `-devN`; bump Cargo.toml to 0.6.0; move
  the task to `## Done` in todo; add README examples for `-v`,
  `--no-pin-cal`, `--pin + --no-pin-cal`, and `RUST_LOG`.

## Todo/chores tidy (0.7.0-dev1)

First step of the 0.7.0 docs/cleanup pass. Purely housekeeping — no
code changes.

- Move completed todos (items 2–14) from `notes/todo.md` `## Done`
  into `notes/done.md` under a `## Through 0.6.0` section, carrying
  their reference links with them.
- Leave the `## Done` section in `todo.md` as an empty placeholder
  so future completions keep flowing through the same path.
- Replace the old `## In Progress` entry with a flat list of the
  four 0.7.0 steps (dev1/dev2/dev3/final); detail lives here.
- Add two new deferred `## Todo` entries surfaced during planning:
  additional thread control, and the eventual crate rename.
- Bump version to `0.7.0-dev1`. No behavior change.

The rest of the 0.7.0 plan lives as `-dev2` (reframe docs as a
general Rust perf tool), `-dev3` (per-item doc comments on every
pub struct/fn/trait; includes renaming `print_histogram` →
`print_report`), and the `0.7.0` final marker.

## Reframe docs as general perf tool (0.7.0-dev2)

The crate started as a specifically IIAC-measurement app, but the
harness it grew (time budget, adaptive sizing, percentile-band
histogram with overhead subtraction, per-thread pinning, calibration
decoupling) is workload-agnostic — IIAC is just the first category
of bench plugged in. This step updates the user-facing voice to
match, keeping the `iiac-perf` name (rename is deferred) and
preserving historical sections.

Edits:

- `README.md` — replace the two-line IIAC subtitle with a
  general-purpose overview paragraph + a "Highlights" bulleted
  list + a closing paragraph noting IIAC as the original seed
  motivation. `## Design (0.2.0)` is left intact as versioned
  history; the new overview frames it retrospectively. Also
  refreshes the `-v` example's banner line to the new wording.
- `src/main.rs` — clap `about` and the startup banner both
  change `— IIAC performance measurement` →
  `— Rust latency microbenchmark harness`.
- `Cargo.toml` — add a `description` field (general framing,
  one sentence, mentions IIAC as first-bench motivation). Gives
  `cargo metadata`, `cargo doc`, and any future `cargo publish`
  consumers the right pitch without touching the crate name.

Non-goals for dev2: no code changes beyond the two display
strings, no renames (dev3 handles `print_histogram`), no new
docstrings (also dev3). Bench names and CLI flags are unchanged.

Also documents three conventions in the right places (not in
`memory/`, per project preference — CLAUDE.md and notes are
checked in and visible to collaborators):

- CLAUDE.md "Commit Message Style": tightened to require a
  terse, chores-indexed commit body (chores = source of truth).
- notes/README.md "Versioning during development": note that
  chores sections are fleshed out per `-devN` as each step
  starts, not all upfront.
- notes/README.md "Todo format": note that each `-devN` commit
  moves its own entry from In Progress to Done.

Retroactive consequence: `dev1` also moves to Done in this
commit, so the convention applies cleanly going forward.

## Per-item doc comments + print_histogram rename (0.7.0-dev3)

Third step of the 0.7.0 docs/cleanup pass. Adds `///` doc comments
to every user-facing `pub` item, introduces `//!` module-level docs
where purpose isn't self-evident from the module name, and renames
`harness::print_histogram` to `harness::print_report` (the function
emits more than just a histogram — header metadata, per-band rows,
whole-histogram summary, and trimmed mean/stdev).

Edits:

- `src/harness.rs` — `//!` module summary; `Bench` trait + its
  `name`/`step` methods, `RunCfg` + each field, `RunCfg::core_for`,
  `run_adaptive`, `fmt_commas`, `fmt_commas_f64`, and the renamed
  `print_report` all get `///` docs. Internal helpers
  (`estimate_step_cost`, `pick_inner`, `run_counted`, `run_timed`,
  `new_hist`, `record_sample`, `round_elapsed`) stay undocumented
  (non-pub).
- `src/overhead.rs` — `//!` module summary; docs on each of the four
  pub consts (`CAL_WARMUP`, `CAL_SAMPLES`, `N_LOW`, `N_HIGH`),
  `Overhead` + each of its five pub fields, `Overhead::per_call_ns`,
  and `calibrate`.
- `src/pin.rs` — `//!` module summary. Every pub fn already had a
  `///` doc as of dev6 of 0.6.0; left as-is.
- `src/benches/mod.rs` — `//!` module summary; `RunFn`, `REGISTRY`,
  `names`, `resolve`.
- `src/benches/{min_now,std_now,mpsc_1t,mpsc_2t}.rs` — `//!`
  one-liner, then docs on `NAME`, the bench struct, `new` where
  pub, and `run`.
- `src/harness.rs` + four bench files — rename
  `print_histogram` → `print_report` (one definition, four call
  sites, all signatures unchanged).

Scope notes:

- No behavior change. Runtime output, CLI flags, and all numeric
  results are unchanged.
- `src/main.rs` is binary entry — its `Cli` fields already carry
  clap doc comments (which become `-h` output) and its helpers
  are all private, so no new `///` needed there.
- Internal `const`s in `harness.rs` (`WARMUP`, `ESTIMATE_STEPS`,
  etc.) stay undocumented; they're tuning knobs not exposed to
  callers.

Bumps `Cargo.toml` `0.7.0-dev2` → `0.7.0-dev3`.

## Bench trait + module split (0.8.0 candidate)

**Superseded by `0.8.0-dev0`** (design: actor runtime + probe
microbench system). The trait enhancement and harness/bench file
split arise naturally while implementing the BenchX + probe model;
no separate effort. Kept here for historical context. Reasoning
below is still largely valid as input to the follow-on design.

Initial stub; not implemented. Surfaced during 0.7.0 review of
the `Bench` trait — the trait is the most important user-facing
interface, and each bench file currently repeats a near-identical
`pub fn run` wrapper.

- **Add `fn new(cfg: &RunCfg) -> Self` to `Bench`** so the trait
  is a complete contract (construction + execution). Collapses
  each bench's `pub fn run` into one generic driver:
  `fn run<B: Bench>(cfg: &RunCfg)` in the harness. Bench-specific
  setup (e.g. `cfg.core_for(1)` for mpsc-2t's worker pin) moves
  inside that bench's own `new(cfg)`. Trait becomes
  object-unsafe (returns `Self`); no current `dyn Bench` use, so
  no regression.

- **Extract interface vs driver** rather than renaming
  `harness.rs` → `bench.rs` wholesale. Split:
  - `src/bench.rs` — `Bench` trait + `RunCfg` (the user-facing
    contract).
  - `src/harness.rs` — `run_adaptive`, `fmt_commas*`,
    `print_report`, internal helpers (driver).

  Anyone adding a new bench imports `bench`; the driver stays
  implementation detail.

- **Scope.** Breaking on the trait contract; safe because no
  external `Bench` impls exist. Target `0.8.0`. Probably
  single-step.

Open questions to resolve before starting:

- Should `run` be a provided trait method (`B::run(cfg)`) or a
  free generic fn (`run::<B>(cfg)`)?
- Should `name` stay a method or become an associated `const
  NAME: &'static str`? Const is stricter but blocks
  runtime-composed names.
- Order vs the deferred crate-name rename — independent, or pair
  them?

## 0.7.0 release (0.7.0)

Ships the 0.7.0 docs/cleanup pass. See the three `-devN` sections
above for detail.

- `-dev1` ✅ todo/chores tidy — moved items 2–14 to `done.md`,
  added incremental-todo convention.
- `-dev2` ✅ reframe user-facing voice from an IIAC-specific app
  to a general-purpose Rust latency microbenchmark harness
  (README, clap `about`, startup banner, Cargo.toml description).
- `-dev3` ✅ `///` doc comments on every pub struct/fn/trait
  + `//!` module summaries; rename
  `harness::print_histogram` → `print_report`.
- `0.7.0` final — remove `-devN`, bump Cargo.toml to `0.7.0`,
  move the four dev entries and final release to todo's `## Done`,
  and jot the 0.8.0-candidate stub above for the next pass.

No behavior change across the pass; numeric output unchanged.

## CLAUDE.md governance model (0.7.1)

Design discussion captured for later cogitation; not implemented.
Much of this is bot-workflow infrastructure and may live outside
this crate's repo entirely when implemented.

### Origin

During the 0.7.0 wrap-up, noticed that when corrected on commit
style the bot's reflex had been to save a feedback memory rather
than re-consult CLAUDE.md — even though the guidance was already
there. Having text in the system prompt ≠ reliably retrieving
the right section when it applies. The discussion evolved from
"why did the bot ignore CLAUDE.md?" into "how should CLAUDE.md
itself be organized across user/global + project scopes?"

### Observations

- Both `~/.claude/CLAUDE.md` (global) and `<repo>/CLAUDE.md`
  (project) are auto-loaded into the bot's system prompt at
  session start; they arrive together, not sequentially.
- The current project CLAUDE.md is mostly universal conventions
  (dual-repo, ochid trailers, Conventional Commits + version
  suffix, pre-commit checklist, commit-push-finalize flow) and
  very little project-specific content.
- `notes/` references in current CLAUDE.md describe *how to use*
  notes, not pointers at specific project files — so the file as
  written is almost entirely general.
- `~/.claude/projects/<encoded-path>` is a symlink to
  `<repo>/.claude`, so `memory/` is versioned inside the bot
  session repo (jj-managed) — not a truly private agent
  scratchpad. This doesn't change the "prefer CLAUDE.md over
  memory/" guidance, since CLAUDE.md sits in the app repo and
  is visible to all collaborators.

### Proposed design

Three synchronized masters of a general CLAUDE.md, plus one
project-specific extension:

- `~/.claude/CLAUDE.md` — general master on the local machine
- `git@github.com:winksaville/<repo>/CLAUDE.md` — versioned
  master (the source of truth)
- `<repo>/CLAUDE.md` — byte-identical copy of the master,
  dropped in by `vc-x1 init` at project creation and refreshed
  via `vc-x1 sync-claude-md`
- `<repo>/notes/BOT-PROJECT.md` — project-specific bot
  instructions, imported into the bot's context by putting
  `@notes/BOT-PROJECT.md` at the bottom of `<repo>/CLAUDE.md`

Drift policy:

- Warning-only, never auto-update. Auto-update could clobber
  intentional local overrides.
- Byte-for-byte compare of the three CLAUDE.md files (trivially
  simple — no marker parsing, no partial-match logic).
- `BOT-PROJECT.md` is project-local and free to vary.
- Trigger the check from a `SessionStart` hook running a
  `vc-x1 check-claude-md` subcommand — deterministic, doesn't
  rely on bot discipline.

### Why not sync markers in a single file

An earlier iteration considered wrapping the "general" block
with `<!-- sync:begin -->` / `<!-- sync:end -->` markers inside
one CLAUDE.md per repo, letting project-specific content append
below. Rejected because:

- Exact-match on the whole file is simpler to reason about and
  to implement in tooling.
- Easy to accidentally edit inside the marker and create
  spurious drift warnings for legitimate reasons.
- Overloads CLAUDE.md with two concerns (general + project).

The strict-separation model keeps CLAUDE.md meaning exactly
one thing and pushes project variance into a clearly-named
neighbor file.

### Naming

`notes/BOT-PROJECT.md` — reads as "bot-facing, project-specific."
Alternatives considered: `BOT.md` (less explicit about project
scope), `PROJECT.md` (loses the bot-facing signal),
`notes/CLAUDE.md` (overloads the name CLAUDE.md which is doing
specific work in the three-file sync story).

### Open questions

- Is `notes/BOT-PROJECT.md` the right location, or should it
  live at `.claude/BOT-PROJECT.md` (bot repo) instead?
- Does the GitHub master live in a dedicated repo, and does it
  carry other per-user dotfiles too, or only CLAUDE.md?
- Cache strategy for the GitHub master: local clone refreshed
  on demand, or fetched once per session in the hook?
- What happens when the master moves forward but a project
  intentionally pins an older version? A `--pin` flag on
  `vc-x1 sync-claude-md`, or a version header line in the
  project copy?
- Interaction with the existing memory system (MEMORY.md
  index auto-loaded at session start): should any current
  memory content migrate to BOT-PROJECT.md, or is memory/
  strictly user-preference/ephemeral state that coexists?
- Should a pre-commit hook also remind the bot to re-read
  CLAUDE.md's commit sections, as a discipline backstop
  complementary to the drift check?

### Not doing (yet)

- Any implementation (hooks, `vc-x1` subcommands, the master
  GitHub repo, the import footer in CLAUDE.md).
- Any split of current CLAUDE.md content into general vs.
  project-local. That waits until the design is settled.

### Next step

Cogitate. When ready, pick a scope (bot-infra repo vs.
in-project) and open a plan.

## Design: actor runtime + probe microbench system (0.8.0-dev0)

Design discussion captured for later implementation; no code
changes in this step. Splits follow-on work across two repos —
this one (iiac-perf, rename deferred) for the probe / microbench
system, and a new experimental repo (`actor-x1` — "actor
experiment 1", name tentative) for the actor runtime.

### Origin

Session started as "add an `spsc-1t` bench using the existing
`Bench` model" and evolved into a re-examination of the core
measurement abstraction. Three observations drove the pivot:

- `std::sync::mpsc` is MPSC-only. An `spsc-1t` written on it
  would be numerically indistinguishable from `mpsc-1t` — the
  "SPSC" label would describe the *usage pattern*, not the
  *primitive*.
- Each message in an actor-style pipeline has multiple
  independently-measurable costs: get, send, travel, recv,
  drop, process. The existing `Bench::step() -> u64` captures
  one aggregate number per sample — it cannot express this.
- Sketching an actor runtime on top of the old `Bench` model
  surfaced that the *app* naturally wants to be in charge. The
  harness should *observe*, not *drive*.

### Architectural inversion

Old model:

- `Bench` drives the harness. One `step()` returns one number;
  the harness times `inner` calls and records one histogram
  entry per outer iteration.

New model:

- Application drives itself (e.g. under an actor runtime).
- Measurement moves to named observers — **probes** — attached
  to lifecycle points. Each probe owns its own histogram.
- Harness registers probes, runs the app for a time budget,
  then collects and reports all of them.
- The old single-`Bench`-per-run model is the degenerate case
  of a single outermost probe.

### Probe model

Two shapes, one primitive:

- **Free-form probe (primitive)**: `Probe::record(elapsed_ns)`.
  Caller captures endpoints independently. Essential for
  cross-boundary spans like *travel time* (producer stamps
  `sent_at` on the message; consumer computes delta on receipt
  and records it).
- **Scoped probe (sugar)**: closure form
  (`probe.measure(|| ...)`) or RAII
  (`let _g = probe.scope();`). A three-line wrapper around
  `record(now - start)`; built on top of the free-form
  primitive. Pick one of closure / RAII to keep the API small;
  closure is cleanest if early-return isn't a concern.

Implementation notes (for the devN that implements probes):

- **Per-thread histograms, merged at report time.** No hot-path
  locking. Cross-thread probes (travel time) record on whichever
  thread ends the span.
- **Message-carried timestamps** for travel time: `Message`
  gains a `sent_at: Instant` field. Free when the runtime
  already pools messages; avoids a side-channel map.
- **Overhead calibration shifts granularity.** Current harness
  amortizes timer-pair framing per outer sample (across `inner`
  calls). Probes pay one timer pair per `record()` call, so
  calibration becomes per-probe-call instead of per-sample.
  Same idea, different unit.
- **Probe overhead is itself measurable.** With N probes
  instrumenting one message's journey, each call pays
  ~10 ns (minstant/rdtsc) × 2. For sub-µs workloads that's
  non-trivial. Runtime or compile-time toggles would let us
  compare "instrumented" vs "uninstrumented" — which is
  exactly the kind of dimension iiac-perf exists to surface.

### Actor model (sketch; implementation in a separate repo)

The actor runtime is developed in `actor-x1` (or whatever
supersedes that name), bootstrapped via `vc-x1 init`. Captured
here for context because the two projects emerged from the same
discussion.

Shape:

- `Actor::handle(&mut self, rt: &Runtime, msg: &Message)` —
  non-blocking. Mutable self, borrowed runtime, borrowed
  message.
- `rt.get_msg(src, dst, content) -> Message` — runtime hands
  out an owned `Message` with routing + payload baked in.
- `rt.send_msg(msg: Message)` — takes ownership, dispatches.
- `Drop for Message` — returns storage to its pool, or to the
  global allocator if no pool is attached.
- Non-blocking semantics: each actor decides its own work
  quantum (1 message, N messages, X ms) and returns; the
  runtime schedules the next tick.
- `-1t` = cooperative scheduler on one thread; `-2t` =
  thread-per-actor, each running its own tick loop.
- Addressing and scheduler shape deliberately left open — many
  defensible answers; iiac-perf is meant to *measure* those
  differences rather than pre-select one.

### Repo split rationale

- Actor and probe have different audiences; neither is a
  natural dep of the other.
- Dependency DAG is clean: probe has no deps on actor; actor
  *optionally* depends on probe when instrumented.
- Splitting enables parallel work on both.
- Naming deferred: `probe` as a crate name is heavily taken on
  crates.io (9+ pages of hits), so no rush. `iiac-perf` stays
  as-is until a better option surfaces; the deferred
  "Rename app" todo absorbs into that future moment.

### Supersedes

- The earlier `0.8.0 candidate: Bench trait + module split`
  chore. The trait enhancement and harness/bench file split
  arise naturally while implementing the BenchX + probe model;
  no separate step. Old chore is kept for historical context
  with a superseded note at the top of its section.

### Not doing in this step

- Any code. This is a design capture, not an implementation.
- Any actor-runtime code in this repo (lives in the new repo).
- Any rename of `iiac-perf` (deferred until a better name
  surfaces).

### Preview of remaining 0.8.0-devN

Fleshed out incrementally per the chores convention — the
current section is the only detailed one; subsequent sections
will be written as each step starts.

- `dev1` — free-form `Probe` primitive: histogram +
  `record(ns)`, per-thread storage, merge, report integration.
- `dev2` — instrument existing `mpsc-1t` / `mpsc-2t` benches
  with probes as end-to-end validation against known numbers.
- `dev3` — scoped probe sugar (closure form; RAII if useful).
- `dev4+` — TBD as the design matures.
- `0.8.0` — finalize.

## Plan: probe primitive + probe-mpsc-2t (0.8.0-dev1)

Design-only capture of the `dev2` implementation plan for the
free-form `Probe` primitive and its first probed bench. No code
in this step.

The dev0 preview listed a different cadence (dev1 = implement
probe primitive, dev2 = instrument existing benches). Reshuffled
to plan-then-implement to match dev0's pattern and give the
design a separate reviewable checkpoint. The old preview is
left stale; this section is the current truth.

### Scope

Introduce the minimum viable `Probe` primitive and validate it
by instrumenting `mpsc-2t`. Goal is to quantify probe overhead
via side-by-side comparison with the unprobed bench, not to get
a realistic "send cost" number on `std::sync::mpsc` — send
latency on that primitive (~15–30 ns) is comparable to probe
overhead (~20 ns per probe = one minstant `now()` pair plus
histogram record).

That's intentional: dev2 validates the probe primitive under
worst-case signal-to-noise. A probe that produces coherent
numbers here will produce cleaner numbers on any heavier
workload.

### Probe surface (minimal)

    pub struct Probe { name: String, hist: Histogram<u64> }
    impl Probe {
        pub fn new(name: &str) -> Self
        pub fn record(&mut self, ns: u64)
        pub fn report(&self)  // own mini-formatter
    }

- Owned, not shared. `&mut self` for `record`. No internal
  locking.
- `Send`: `Histogram<u64>` is `Send`, so probes can be moved
  across threads (e.g. returned from a `JoinHandle<Probe>`).
  Not `Sync` — cross-thread *sharing* is out of scope.
- Module location: `src/probe.rs`. Registered in `src/main.rs`
  with `mod probe;`.

### Probed bench: `probe-mpsc-2t`

Structural mirror of `mpsc-2t`, diverging only in
instrumentation:

- File: `src/benches/probe_mpsc_2t.rs`
- CLI name: `probe-mpsc-2t` (prefix convention — all probed
  variants will be `probe-*`, keeping them alphabetically
  grouped in the startup listing).
- Struct: `ProbedStdMpsc2Thread`, mirrors `StdMpsc2Thread` plus:
  - `main_probe: Probe` field — records main thread's
    `req_tx.send(c)` duration inside `step()`.
  - Worker thread owns its own `Probe`, records around
    `resp_tx.send(v)`, returns it via `JoinHandle<Probe>` on
    shutdown.
- New method `finish(&mut self) -> (Probe, Probe)` — drops
  `req_tx` (worker `recv` returns `Err`, worker exits), joins
  the worker thread, returns `(main_probe, worker_probe)`.
  `Drop` impl stays as a safety net for panic paths.
- `pub fn run()` flow:
  1. `bench = new(...)`
  2. `(hist, outer, inner, duration_s) = run_adaptive(&mut bench, cfg)`
  3. `(main_probe, worker_probe) = bench.finish()`
  4. `harness::print_report(...)` — existing outer-histogram
     report.
  5. `main_probe.report()`
  6. `worker_probe.report()`

### Probe report format

Own formatter; not a reuse of `harness::print_report`. Reason:
`print_report` assumes outer/inner/adj-per-call semantics that
don't apply to a probe — each `record()` is one sample, period.

MVP shape:

    probe: <name> [count=N]
      <band-table: min-p1, p1-p10, ..., p99-max>
      mean:           ... ns
      stdev:          ... ns
      mean min-p99:   ... ns
      stdev min-p99:  ... ns

Reuse the band-boundary percentages and column layout from
`harness::print_report` so rows line up visually with the outer
report. If the band-table renderer falls out naturally as a
shared helper during implementation, factor it; don't force it
if the shapes diverge.

### Validation experiment

Once dev2 merges, run both variants back-to-back with a large
budget and matched pinning:

    iiac-perf mpsc-2t probe-mpsc-2t -D 60 --pin <core-a>,<core-b>

Extract per-step probe overhead as:

    overhead_per_step = mean(probe-mpsc-2t) - mean(mpsc-2t)

Since `probe-mpsc-2t` adds exactly two probes per step (one on
each thread), divide by 2 for per-probe overhead. Expected
range: ~20–40 ns (2 × minstant `now()` + histogram record).

Both variants run in the same process invocation so CPU state
and framing calibration are shared, minimizing drift between
the two measurements.

### Intentionally out of scope for dev2

- Scoped / RAII probe sugar (slated for a later dev).
- Probing `mpsc-1t` (can follow later if useful).
- Cross-thread *sharing* of a single probe (no use case yet).
- Message-carried `sent_at` timestamps / travel-time probes
  (wait for a bench that motivates them).
- Any modification to `harness` (probes stay bench-local).
- Probe overhead subtraction in the calibration model
  (dev3+ territory).

### Next step

Approval from user, then implement as `0.8.0-dev2`.

## Implement probe primitive + probe-mpsc-2t (0.8.0-dev2)

Implements the 0.8.0-dev1 plan: free-form `Probe` primitive and
the first probed bench.

### Edits

- `src/probe.rs` — new. `Probe { name, hist }` with `new`,
  `record(ns)`, `report()`. Histogram bounds (1 ns — 60 s, 3 sig
  figs) match the harness. Band-table rendering is duplicated
  from `harness::print_report` rather than factored into a
  shared helper: the two outputs diverge enough (no `adjusted`
  column on probes, one deeper indent level, different header)
  that an abstraction would have been premature. Revisit if a
  third consumer shows up.
- `src/main.rs` — `mod probe;` registered alongside the existing
  modules.
- `src/benches/probe_mpsc_2t.rs` — new. Mirrors `mpsc_2t.rs`
  plus:
  - `main_probe: Probe` field, timed around `req_tx.send`.
  - Worker owns its own `Probe`, times around `resp_tx.send`,
    returns it via `JoinHandle<Probe>` on shutdown.
  - `finish(&mut self) -> (Probe, Probe)` — drops `req_tx`,
    joins worker, returns both probes.
  - `Drop` impl kept as a panic-path safety net.
  - `run()` flow: `new → run_adaptive → finish → print_report
    → main_probe.report() → worker_probe.report()`.
- `src/benches/mod.rs` — register `probe_mpsc_2t` at the end of
  `REGISTRY` (alphabetically groups the future `probe-*` family).

### Validation

`iiac-perf mpsc-2t probe-mpsc-2t -D 60 --pin 5,10` on 3900X.
Both benches run back-to-back in the same process, sharing CPU
state and calibration.

|                        |   mean   | mean min-p99 | stdev min-p99 |
| ---                    | -------: | -----------: | ------------: |
| `mpsc-2t` (unprobed)   | 8,351 ns |     8,310 ns |        424 ns |
| `probe-mpsc-2t`        | 8,495 ns |     8,444 ns |        474 ns |
| **Δ (per step)**       |   144 ns |       134 ns |         50 ns |
| **Δ / 2 (per probe)**  |    72 ns |        67 ns |             — |

Per-probe overhead lands at ~67–72 ns, higher than the napkin
~20–40 ns estimate. Plausible breakdown: 2 × minstant rdtsc
(~10 ns) + `Histogram::record` (~20–30 ns in release) + ~20 ns
for struct accesses, counter-increment reordering, and cache
effects of the probe field. Still well under 1% of the ~8,400 ns
round-trip — and, importantly for dev2's goal, the probes
produce coherent numbers under worst-case signal-to-noise.

Probe-side numbers themselves:

- `main send`:   mean 1,278 ns, mean min-p99 1,268 ns
- `worker send`: mean 1,349 ns, mean min-p99 1,341 ns

Send cost here is dominated by the `futex_wake` path when the
peer is parked in `recv`; this is the round-trip-triggering
send, not a raw queue push.

### Notes

- Probe overhead subtraction is not applied to the `adjusted`
  column of the outer histogram — that's the dev3+ territory
  flagged back in dev0.
- The probe `min-p1` band occasionally shows `first=10 ns`,
  plausibly rare spuriously-short elapsed samples caused by
  out-of-order retirement around the paired timer reads.
  Affects <1% of samples and doesn't move the reported means.

## Producer-consumer bench: probe-only UX experiment (0.8.0-dev3)

Single-step: planned and implemented in one pass at user's
request. No separate dev3/dev4 split.

### Motivation

`probe-mpsc-2t` (dev2) fit probes inside the existing `Bench`
trait. That worked but made main do double duty as producer and
harness driver, which we found hard to read. This step is a
UX experiment: what does it feel like to *write and read* a
bench where the application drives itself and probes are the
only measurement channel?

Success criterion: code reads linearly (main orchestrates,
producer produces, consumer consumes), and the output explains
itself cold to a reader. Numbers are a sanity check, not the
point.

### Shape

- **Free-form `run(cfg)`**, no `Bench` trait. The registry
  entry spawns threads, sleeps for `cfg.target_seconds`,
  signals shutdown, joins, prints probe reports. No outer
  histogram.
- **Three threads**:
  - Main: creates channels, spawns Producer + Consumer,
    sleeps, sets the shutdown flag, joins, prints.
  - Producer (on `core_for(0)`):
    `loop { send; recv; probe.record(cycle) }`.
  - Consumer (on `core_for(1)`):
    `loop { recv; send; probe.record(cycle) }`.
- **One probe per actor** — `producer loop` and `consumer
  loop`. In steady state both means should converge (same
  round-trip, opposite viewpoints).

### Shutdown

`Arc<AtomicBool>`. Main sets it after sleeping; producer checks
at the top of its loop and exits. Consumer does not check —
when producer exits its `req_tx` drops, consumer's
`req_rx.recv()` returns `Err`, and consumer exits.

Race-free because both actors' `send`/`recv` pairs always
complete within a few µs:

- Producer's `send → recv` always unblocks (consumer always
  replies to received messages).
- Consumer's `recv → send` exits cleanly once producer is
  gone (either `recv` returns `Err`, or `send` returns `Err`
  because the receiver dropped).

KIS per user directive — a full actor model would send a stop
message; deferred.

### Report format

Two-line bench header (name + mode + duration), then the two
probe sub-reports:

    producer-consumer (2 threads, probe-only) [duration=30.0s]:
      probe: producer loop [count=N] ...
      probe: consumer loop [count=N] ...

The `probe-only` tag is explicit so a cold reader immediately
sees this is a different bench shape from the others.

### Edits

- `Cargo.toml` — version bump to `0.8.0-dev3`.
- `src/benches/producer_consumer.rs` — new, ~80 lines.
- `src/benches/mod.rs` — register `producer_consumer` at the
  end of `REGISTRY`.

### Intentionally out of scope

- Separate send-only probes (already have those from
  `probe-mpsc-2t`).
- Travel-time / message-carried timestamps.
- Actor-message shutdown.
- Cross-bench unified output.
- Validation-focused number comparisons — this bench exists to
  shape the UX, not to measure anything new. Running it
  alongside `mpsc-2t` / `probe-mpsc-2t` is fine as a sanity
  check but isn't the goal.

### Observations during dev3

Surfaced during the back-and-forth after the bench was
implemented and run. Worth capturing alongside the mechanical
edits because these inform the probe crate's next steps and
the actor-x1 repo's design.

- **UX hypothesis validated.** `probe-mpsc-2t` was reported
  as hard to follow — main does double duty as harness
  driver and producer, so the flow of the code fights the
  `Bench` trait's step-driven model. `producer-consumer`
  separates orchestration from workload cleanly: main
  orchestrates, producer produces, consumer consumes. Reads
  linearly, output self-explains, ~⅓ shorter than
  `probe-mpsc-2t`.
- **Producer-consumer runs ~10–30% faster** than `mpsc-2t`
  on the same workload. Factors in suspected order of
  significance:
  1. Tighter hot loop on a dedicated thread — the harness's
     per-sample bookkeeping (histogram record, time-budget
     check) runs on the same thread as the bench step in
     `mpsc-2t`, contaminating cache and branch-predictor
     state between iterations even though it's outside the
     measured interval.
  2. Struct-field indirection (`self.req_tx`, `self.counter`)
     vs local bindings captured in the producer closure
     (registers in release).
  3. Scheduler placement: main + one spawned worker has
     asymmetric start state, where two freshly-spawned
     threads tend to land more symmetrically (often on
     sibling cores in the same CCX) when unpinned.
- **Two-sided probe symmetry.** With both probes active:
  producer loop mean 3,638 ns vs consumer loop mean
  3,645 ns — ~7 ns apart. Same round-trip, opposite
  viewpoints; the symmetry is the thing we hoped to see
  when both probes are active.
- **Band histogram reveals bimodality.** `std::sync::mpsc`
  round-trip under unpinned placement runs in two distinct
  modes — fast-spin (~400–520 ns, both threads hot on
  adjacent cores) and futex-wake (~5,500–8,000 ns, receiver
  parked, send triggers kernel wake + migrate). Transitions
  are rare, so the band straddling the crossover has a
  huge `first → last` range (~5,000 ns). Different
  fast:slow ratios between benches shift where that
  crossover band lands:
  - `mpsc-2t`: crossover is in `p30-p40` (first=530,
    last=5,923).
  - `producer-consumer`: crossover is in `p40-p50`
    (first=510, last=5,503) — producer-consumer spends a
    larger *fraction* of iterations in the fast cluster,
    pushing the crossover percentile higher.
- **Probe overstates by `framing_per_sample` ns.** Each
  probe sample includes the fixed timer-pair cost
  (~11 ns on 3900X). `probe.record` itself is *not* in
  the measured interval — Rust argument-evaluation order
  places the second `now()` call (inside `s.elapsed()`)
  before `record` is invoked. Fix (adjusted column on
  `Probe::report` using `Overhead::framing_per_sample_ns`)
  captured as a follow-on todo; not blocking because every
  probe in this session is overstated by the same amount,
  so cross-probe comparisons stay fair.
- **Probes in the runtime.** Probes fit naturally in
  systems that already have intrinsic loops and lifecycle
  boundaries. In `actor-x1`, probes at `get_msg` /
  `send_msg` / handler entry-exit / message drop could be
  provided *by the runtime* — actor authors get
  instrumentation for free without writing probe code
  themselves. Strengthens the dev0 observation that probes
  align with runtime API boundaries, and suggests the
  probe crate should be a first-class (feature-gated) dep
  of `actor-x1` rather than an optional add-on.
