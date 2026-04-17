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
