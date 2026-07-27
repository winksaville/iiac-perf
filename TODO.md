# Todo

This file uses [Prose form](AGENTS.md#prose-form). It
contains near term tasks with a short description and
uses links or reference links for more details.

## In Progress

When a `## Todo` item is picked up, its text moves here: the
problem overview and its list of things to do. That is followed
by the "plan" — a bulleted list of the development "ladder":
   - [[N]] 0.xx.y-0 blah (done)
   - [[N]] 0.xx.y-1 blah blah (current)
   - [[N]] 0.xx.y-2 blah blah blah
   - [[N]] 0.xx.y close-out and validation

**feat: grade the run from raw batches**

The 0.23.0 cycle, decided in
[Replanning II](notes/chores/chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run):
the overhead subtraction estimates an ill-defined quantity and
cancels in same-harness A/B anyway, while the calibration-time
grade certified the room, not the exam. Drop the adjustment
machinery; grade the run from its own time-ordered batches.

- [[67]] 0.23.0-0 `chore: open raw-batch grading cycle`
  (done)
- [[N]] 0.23.0-1 `feat: micro-probe inner-loop sizing`
  (done) —
  `pick_inner` inputs from a ~1 ms micro-probe (low quantile
  over back-to-back timer pairs), never printed; unhooks
  sizing from `cfg.overhead`
- [[N]] 0.23.0-2 `refactor: drop overhead calibration` —
  delete overhead.rs, the constants block, adjusted columns,
  the `calibrate` command; raw values only; one README
  sentence on apparatus framing
- [[N]] 0.23.0-3 `feat: time-ordered batch pipeline` —
  samples land in a raw batch buffer; per-batch summaries
  (floor, mean, census counts) taken as batches fill, then
  bulk-record into the histogram; bounded memory
- [[N]] 0.23.0-4 `feat: batch-based run gauge` — relocate
  the -5 grade machinery (signals, thresholds, letter,
  warnings) onto batch summaries: drift from floor movement,
  bursts localized in time, interference rate from census
  counts (absorbs the crossover entry's rate analysis);
  print each signal's own letter beside its value (the
  scores already exist — the composite is their worst, and
  showing them makes every letter self-explaining;
  2026-07-27 settle-test observation: repeat/drift are the
  same transition detector at two timescales)
- [[N]] 0.23.0 `feat: grade the run from raw batches` —
  close-out and validation

## Todo

Entries are in **strict priority rank** — #1 highest,
descending. Reprioritize by moving an entry, then
`vc-x1 fix-todo --no-dry-run TODO.md` to renumber.
The numbers are positional rank, not stable IDs — to refer
to a Todo, name it by its **title** (a greppable mention;
a numbered list item has no anchor to link to), not its
number. Long-tail entries
live in [todo-backlog.md](notes/todo-backlog.md). Use the
[Prose Form in AGENTS.md](AGENTS.md#prose-form); deeper
detail goes in `notes/chores/chores-NN.md` design
subsections (link via `[N]` ref).

1. Dynamic startup warmup — replace the fixed
   `WARMUP = 10_000` step count in `harness.rs` with
   warm-until-stable: a fixed count's wall-clock scales with
   step cost, so the fastest benches warm ~10 us against
   frequency-governor ramps of tens-to-hundreds of ms and
   `pick_inner` sizes mid-ramp (the 7600x F diagnosis,
   [Replanning II](notes/chores/chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run);
   the retired -6 rung's design, revived for the run itself)
   - terminology: the warmup unit is a **warmup pass** (the
     0.22.0-4 "loop-only passes" sense) — a short, unrecorded,
     timed burst of bench steps yielding one floor. Not a
     "probe": `TProbe` is the measurement instrument, and the
     "micro-probe" is the 0.23.0 cycle's ~1 ms timer-pair
     frame measurement
   - fuse with `pick_inner` sizing: the warmup pass *is* the
     step-cost sizing pass, repeated back to back until K
     consecutive floors agree within tolerance; the final
     agreeing floor is the sizing input, so sizing is
     post-ramp by construction and convergence is tested on
     the number actually consumed (the micro-probe supplies
     the frame input, run after convergence)
   - K defaults to 3: two agreeing passes can certify a dwell
     at an intermediate P-state (the staircase ramp), not the
     top
   - convergence is agreement, not direction — the 3900X is
     bistable (2026-07-27 `calibrate` runs): sustained rapid
     repetition climbs it into a fast state (~0.445 ns/iter,
     drift bracket caught 48.6 -> 45.3 mid-window, D/F),
     low-duty isolated runs sit at ~0.489 (B), ~9% apart, and
     transitions straddle windows in *both* directions (the
     climb under quick cadence; the settle-back after load,
     which outlasted an 8 s gap once the box had been hot);
     fixed warmup absorbs only transitions shorter than
     itself, whatever their sign
   - floors, not means, so one preemption doesn't fake
     (in)stability; warm box exits near-immediately
   - slow steps (`inner` -> 1 territory): pass length adapts
     — minimum step count or minimum wall time, whichever is
     larger — so a floor is never one sample; the cap exit
     then distinguishes "floors disagreed" (unstable, gauge
     signal) from "too slow to certify K passes" (proceed,
     label the run uncertified — at `inner = 1` sizing can't
     be wrong and framing is negligible, so the stakes are
     low there)
   - hard cap at governor scale (a few hundred ms); hitting it
     unconverged is a gauge signal ("run started unstable"),
     not a silent proceed — and the cap doubles as the
     estimate-phase deadline the `--pin` guard entry (below)
     wants, so slow and non-converging benches share one
     diagnostic exit
   - differential start-vs-end QC: repeat the same pass at
     run end and compare floors (never absolute, never
     subtracted); the N-sweep slope/intercept decomposition
     stays an idea unless batch data shows frame shifts need
     separating from per-iteration shifts
   - acceptance test — `tests/settle_anomaly.rs` (landed
     0.23.0-1): `#[ignore]`d integration test spawning the
     real binary per run (`CARGO_BIN_EXE`), zero-gap quick
     cadence (triggers the boost climb) vs 8 s waited; both
     cadence medians must reach B *and* no run may grade D/F
     on a transition detector (drift / repeat) — cause-aware
     via the per-signal letters, so ambient contamination
     (disturbed/dirty, e.g. a concurrent build) and the
     resid machine trait can't flake the verdict; grades
     degrade on transition-straddling windows, not in either
     steady state, and cadences may honestly report
     different speeds. Reproduced failing 2026-07-27 on the
     3900X (repeat F on the climb run at quick=0; medians
     3/3 at quick=0/wait=8). `IIAC_PERF_BIN` pins a saved failing build; the
     observable (calibrate letter) migrates to the batch
     gauge when calibrate dies (0.23.0-2/-4). Part of this
     cycle's close-out validation
2. Guard `--pin` pools smaller than the bench's thread
   placements, and deadline the estimate phase — `zcr-mpsc-2t
   --pin 8` put both spinning software threads on one logical
   CPU and appeared hung until ^C (2026-07-26, bug #1 in
   [bugs.md](notes/bugs.md#bugs))
   - track `core_for` requests in `RunCfg` (max `thread_idx`
     asked for); refuse the run when placements exceed unique
     CPUs in the pool — placement only goes through `core_for`
     when pinning is active, so the guard covers every path,
     and no pinning means the scheduler separates the spinners
     itself
   - wall-clock deadline on the open-loop 5x1,000-step
     estimate phase so *any* pathologically slow bench aborts
     with a diagnostic naming per-step cost and pinning,
     instead of hanging
3. Tighten thread/CPU terminology across docs and doc
   comments: "software thread" for what `thread::spawn`
   makes, "logical CPU" (hardware thread) for what `--pin`
   selects and the OS schedules onto, "physical core" for the
   engine SMT siblings share — bare "core"/"CPU"/"thread"
   only where context disambiguates
   - spin-wait bench docs state the precondition: each
     spinning software thread needs its own logical CPU
   - `--pin` help/README say slots are logical CPU ids
4. Rebase `web-claude-tweaks` onto post-0.22.0 `main` —
   rewrites an already-published bookmark (needs approval)
   and its arbitrary `0.21.0-b` version needs replacing;
   owed from the 0.22.0 close-out plan
5. Unit scaling in report columns (`us`/`ms`) — per-row
   auto-scale so columns stay eyeball-comparable (bands are
   monotonic, so a row's first/last/mean share a magnitude),
   or `--units ns|auto` for script-stable output; needs
   `--decimals` landed first (`3.18 ms` vs `3 ms`); candidate
   `-4` for the report-options cycle.
6. Machine-readable report output (`--format json`, or
   key=value lines to stay dependency-light) — design once
   the batch gauge lands (0.23.0-4) so the schema covers the
   surviving surface: report stats, gauge signals, letter.
   Consumers: `tests/settle_anomaly.rs` (drops its
   brittle-but-loud line parsing), placement-map validation
   runs, cross-run comparison scripts. Kin to the
   unit-scaling entry's `--units ns` script-stable concern
   (above) — one flag family.
7. Trimmed core stats: `mean/stdev p10-p90` report row,
   additional to (never replacing) `mean` / `mean min-p99`;
   trim bounds possibly configurable (`--trim p10:p90`?) —
   the full mean wobbles ~±1.4% with the run's mode mix while
   the core plateau is ~±0.2% stable, so the trimmed row is
   the run-to-run comparable number. Boundary sensitivity
   (see [[57]]): window edges in the mode-mix smear inherit
   its wobble (p50-p60 ±0.05% vs p40-p50 ~1%), so also
   consider a dominant-*mode* statistic (peak-density region,
   bottom-count-independent) [[57]]
8. Find and label the interference crossover — the band where
   the tail stops measuring the code and starts measuring the
   machine. Not to hide it: to *name* it, because that is the
   signal TProbe exists to surface (the OS swapping, a drive
   stalling, anything not caused by the code under test).
   - Locate it from the data rather than fixing it at a
     percentile: the giveaway is the band `range` exploding
     (min-now 0.21.0, 3900X: `n3` range 3.0 ns -> `n4` range
     200.4 ns), not a chosen p99.
   - The crossover moves with the bench. A counting argument
     places it: interference arrives at a *rate*, so it can
     only contaminate so many samples. That run's `n2` held
     838,635 of 8,059,469 samples over 5 s = ~168,000/s, and
     nothing in the OS runs at that rate, so `n2` is code. The
     `n4`+ bands total ~1,500/s — timer-interrupt territory.
   - So report the above-crossover count as an **interference
     rate**, and consider surfacing whether the run was quiet
     enough to trust. Calibration wants exactly this signal
     (see [[61]]); a contaminated run is currently only
     detectable by squinting at band ranges.
   - Superseded pointer: the 0.22.0-5 calibration-time grade
     certified the ~1 s window before the run;
     [Replanning II](notes/chores/chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run)
     moves grading onto the run itself. This entry's crossover
     and rate analysis is absorbed by Todo #1's batch design,
     which supplies the time axis the histogram lacks.
   - Pairs with the trimmed-core-stats entry above: that one
     needs a defensible upper bound, and this is how to find
     one per run instead of hardcoding p99.
9. Upstream the ladder commit-ref convention to
   `../vc-template-x1`: In Progress ladder rungs (and the
   chores As-built rungs) carry a prepended `[[N]]`
   commit-ref placeholder, backfilled as each commit
   becomes permanent — template's cycle-protocol.md,
   AGENTS.md, and TODO.md example need the shape; that
   repo has its own approval/push flow
10. Investigate: suspend gap missing from samples. A 0.13.5
    `--no-inhibit` suspend test detected ~1.2 s suspended inside
    the measured window but the max sample was only 4.0 ms,
    while the 0.13.1 test (8.4 s gap) showed the expected 10.4 s
    max sample. We think minstant's TSC may halt across some
    suspends and count through others. Repeat the test comparing
    detected gap vs max sample; if the TSC halts, per-sample
    timing silently loses suspend time — document either way.
11. CLAUDE.md governance model (design cogitation) [20]
12. Revisit probe adjustment under the in-interval vs
    call-to-call split: probes take one call per sample
    (inner=1), so the in-interval timer slice is unamortized
    and unmeasurable — an `adjusted` column can subtract
    nothing defensible; maybe state a bound instead
    [analysis](notes/design.md#timer-overhead-in-interval-vs-call-to-call)
13. Convert `harness` / `Bench` to probe-based measurement. Will
    likely need inner-loop support on `Probe` (batch N calls per
    sample; report divides by N and accounts for per-sample
    framing) so very-small workloads can still amortize timer
    overhead the way `run_adaptive` does today.
14. Rename app
15. Design an app to measure IIAC perforanace written in Rust[1]
16. `ice-ps-2t-wait` — iceoryx2 pub/sub with blocking waits via
    `Listener`/`Notifier` events; completes the {transport} ×
    {wait policy} matrix cell that compares against `mpsc-2t`
17. Switch ice benches to the loan-based zero-copy send path
    (`loan_uninit` + `send`) — the API a perf-sensitive user would
    use, and closer to iceoryx2's own benchmark method
18. Payload-size sweep for the round-trip benches (8 B / 8 KiB /
    1 MiB) — makes iceoryx2's size-independent latency vs channel
    copy cost visible in our own tables
19. `crossbeam-1t` / `crossbeam-2t` — `crossbeam-channel` directly
    (compare to mpsc-1t/2t which use crossbeam under the std API)
20. `tokio-mpsc-1t` / `tokio-mpsc-2t` — `tokio::sync::mpsc` round-trip
    inside a Tokio runtime (async overhead)
21. `flume-1t` / `flume-2t` — `flume` MPMC channel
22. Function-call baselines: direct call vs `Box<dyn Trait>` vs
    `async fn` (poll-once) — anchors the channel/serde numbers
    against the cheapest possible "send a value then receive it" path
23. When the second channel impl lands, extract shared message types
    + round-trip helpers into `src/benches/common.rs` (deferred from 0.2.0)
24. Additional thread control (count, per-thread pin lists, NUMA) —
    shape once a concrete bench needs it
25. Rename crate `iiac-perf` → general-purpose name (breaking; deferred)

## Ideas

Longer-range thoughts, not yet ranked work. `-` bullets, no
numbering; promote into `## Todo` when one becomes actionable.

- Per-bench dependency isolation — motivated by dep provenance:
  the deps are the thing being measured, so a dep bump (e.g.
  iceoryx2 0.9.2 → 0.9.3) legitimately moves that bench's
  numbers and shouldn't ride in silently. Options considered
  (2026-07-08):
  - Caveat first: a Cargo **workspace shares one Cargo.lock**
    across members — it scopes deps per package (ice benches
    alone pay for iceoryx2; faster `-p` builds; harness/probes
    become a library crate) but does *not* give per-bench lock
    isolation, and it splits the single CLI into many binaries.
  - Targeted updates (`cargo update -p <crate>`, never bare
    `cargo update`) — ~90% of the provenance benefit at zero
    structure cost; adoptable immediately as discipline.
  - Feature gates (`--features ice`) — solves build weight in
    the current single package, not lock isolation.
  - Truly standalone crates (own Cargo.lock each) — the only
    real per-bench dep isolation; maximum maintenance, and cuts
    against "same harness, same build" A/B comparability.
  - Current lean: targeted-update discipline now; feature gates
    or workspace only when bench families multiply.
- clap CompleteEnv dynamic completion (the `unstable-dynamic`
  feature): clap's native runtime completer (`COMPLETE=bash
  iiac-perf`) would give bash die-hards a compact column view
  without carapace; revisit if clap stabilizes it.
- Stability selftest mode (2026-07-27) — grade the
  environment more thoroughly than a single run's gauge: a
  product subcommand that respawns its own binary
  (`current_exe()`) N times at configurable cadences and
  reports cross-run gauge agreement ("is this box currently
  trustworthy for A/B?"). Precedent in-product: the
  calibration repeat self-check and `--blocks` both already
  validate by orchestrated repetition; this is the next ring
  out. Subsumes `tests/settle_anomaly.rs`'s orchestration —
  the test reduces to asserting on the verdict, and its
  env-var knobs become clap flags. Build on the batch gauge
  (0.23.0-4), not before it; promote to `## Todo` when the
  gauge lands. Concrete motivation (2026-07-27): the settle
  test can't run on the 7600x, which has only the installed
  binary — environment qualification shouldn't require a
  source tree.
- Tick-phase avoidance (2026-07-27): the scheduler tick is
  periodic per-CPU (~300/s at CONFIG_HZ=300) and a tick hit is
  an unmistakable outlier, so predict the next tick from
  detected hits and pause measuring ~30 us around it — ~1%
  duty cost, no governor exposure with governor+EPP
  `performance`. Doesn't improve the bulk stats (tick hits
  are already detected and trimmed); buys a cleaner
  above-crossover tail on unmodified machines, so rarer
  aperiodic events (device IRQs, SMIs, code slow paths)
  become visible over the periodic contaminant. Check the
  interaction with dither (anti-phase scheduling must not
  introduce a systematic phase bias), and compare against
  `nohz_full`/`isolcpus` isolation — which abolishes the tick
  on a dedicated core and strictly dominates when a reboot is
  allowed — before building.

## Bugs

_See [bugs.md](notes/bugs.md)._

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](notes/done.md) to keep this file small.

- fix: calibration robust to codegen and noise [[66]] —
  the 0.22.0 cycle; validation pass recorded in
  [placement-map.md](notes/placement-map.md)

# References

[1]: /README.md#Design-010
[20]: /notes/chores/chores-02.md#claudemd-governance-model-071
[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[66]: /notes/chores/chores-04.md#fix-calibration-robust-to-codegen-and-noise
[67]: https://github.com/winksaville/iiac-perf/commit/621c5c97dbe1 "621c5c97dbe1418fdcb99db6080eecde40891491"
