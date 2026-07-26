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

**fix: calibration robust to codegen and noise**

Two calibration constants are derived by differencing or
extrapolating measurements that don't share their assumptions.
`frame_call_ns` subtracts a slope fitted at the dither call site
from a window mean measured at another — the same source loop
compiles ~12% apart at `opt-level=0`, driving `frame/call` to a
clamped `0.000 ns` in debug and ~30% low in release.
`frame_sample_ns` extrapolates a two-point fit back to N=0, and
scheduler interference is one-sided and duration-proportional,
so it inflates the long point ~100x harder than the short one,
tilting the slope up and levering the intercept negative.
Neither averages away with more runs.

Scope grew mid-cycle (2026-07-25, see
[the replanning subsection](notes/chores/chores-04.md#replanning-slope-dither-and-self-checks)):
after -1/-3, `loop_per_iter_ns` is the last constant still
produced by the dithered two-point fit — two points can always
be fitted perfectly, so a violated model is invisible, and with
`N_HIGH/N_LOW = 100` the slope is ~99% determined by the one
point interference hits hardest. -4 retires the fit from
production; -5 makes the self-checks and an environment grade
automatic, since a user can't be assumed to know diagnostics
exist or when to run them.

- [[62]] 0.22.0-1 fix: pair frame/call against a loop-only pass
  (done)
  - shared `#[inline(never)] run_inner` across all three passes
  - `measure_loop_only` beside `measure_window`; `frame_call =
    w_low - l_low`, loop term cancelling exactly
  - warn instead of silently clamping, both constants
- [[63]] 0.22.0-2 fix: fit frame/sample from a low sample quantile
  (done)
  - discard the fastest 1% of samples, take the minimum of the
    remainder, at both fit points: the low tail is the
    uncontaminated part, and discarding its very bottom sheds
    samples that rounded down on the clock lattice
  - window-mean tail selection was tried first and failed —
    under a continuous competitor no `N_HIGH` window escapes
    contamination; kept as the `fast` diagnostic
  - `frame_sample` and `loop_per_iter` both move
- [[64]] 0.22.0-3 fix: derive frame/sample without extrapolating
  (done)
  - `frame_sample = d_low - l_low`, both at `N_LOW` over the
    same `run_inner`: a difference of same-N measurements can't
    be levered negative by a contaminated long point, because
    there is no long point in it
  - assert the physical invariant `frame_sample <= frame_call`
    (a part cannot exceed the whole) and warn when violated —
    seen at 51.323 vs 49.766 ns, an impossibility nothing
    currently checks
  - on a non-physical constant, retry a bounded number of times
    and then report it unavailable; never publish a clamp
- [[65]] 0.22.0-4 fix: slope from multi-N loop-only passes
  (done)
  - `measure_loop_only` over a geometric N ladder
    (100..10,000), samples-per-window scaled ~1/N so window
    *duration* stays roughly constant — the -1/-2 lesson that
    short windows slip between preemptions, applied at every N
  - production `loop_per_iter` = Theil-Sen (median of pairwise
    slopes) over the ladder points — robust to the one-sided
    contamination that condemns a mean-based fit
  - align the dither `N_LOW` pass's window shape with the
    loop-only pass (same windows x samples), so
    `frame_sample = d_low.min_window - l_low` finally
    differences comparable order statistics — chases the ~8%
    `min_window_ns` loose thread below
  - the dithered two-point fit stays computed and logged as a
    diagnostic (its divergence from the loop-only slope is a
    cross-check), no longer a production input
- [[N]] 0.22.0-5 feat: always-on calibration self-checks
  (done)
  - every check runs on every calibration; a passing check is
    silent, a failing one prints a plain-language WARNING —
    the user can't be assumed to know diagnostics exist
  - new checks beside the -3 physical invariants: linearity
    residual over the N ladder, loop-only slope vs dithered-fit
    slope divergence, and an intra-calibration drift check
    (re-measure the `N_LOW` loop-only point last, compare to
    first) — paired differences assume machine state holds
    across the pair, and a -4 debug run showed the failure: a
    regime shift mid-calibration drove frame/sample to 88 ns
    against frame/call 39, caught by the -3 invariant and
    cleared on retry
  - environment grade line after every calibration: letter
    grade + evidence (disturbed-sample fraction, clean-window
    fraction, repeatability of the constants across attempts),
    with the repeatability figure in ns as the headline number
  - statistical thresholds set from quiet-machine spread on
    both boxes; a quiet machine should essentially never warn
  - a calibration with violations is never cached; the cache
    records that its entry passed
- [[N]] 0.22.0 close-out (scope cut 2026-07-26: the planned
  -6 warmup-until-stable and -7 reporting rungs are retired —
  the philosophy they served is dropped, see
  [Replanning II](notes/chores/chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run);
  the redesign is Todo #1)

**Resume here.** -1..-4 are landed and pushed; -5 is committed
(or about to be); only close-out remains, the -6/-7 rungs
having been retired by
[Replanning II](notes/chores/chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run)
(2026-07-26: overhead adjustment is being dropped entirely —
the next cycle is Todo #1).

- Close-out also owes: the `--first-parent` recommendation
  alongside the
  [Merge non-ff recipe](notes/cycle-protocol.md#merge-non-ff-recipe)
  (and eventually upstream to `vc-template-x1`), the `Commits:`
  backfill for -5 (the -1..-4 backfill landed with -5), the
  section's `Commits:` line, and retiring older `## Done`
  entries.
- Publishing shape is **Merge non-ff** (trapezoid), chosen so
  the internal steps stay visible and `--first-parent` skips
  them. The merge must be set up *before* the close-out push,
  and `vc-x1 push` only fully supports keep-separate, so
  expect manual jj steps.
- Then fast-forward `main` to `fix-calibration` and rebase
  `web-claude-tweaks` onto it — that rewrites an already
  published bookmark (needs approval), and its arbitrary
  `0.21.0-b` version needs replacing.
- **r5-7600x is reachable**: plain `ssh r5-7600x` works, but
  `scp` fails with "Network is unreachable" — stream instead,
  `ssh r5-7600x 'cat > /tmp/iiac-rel && chmod +x /tmp/iiac-rel'
  < target/release/iiac-perf`. No `target-cpu=native` anywhere,
  so one release build is valid on both boxes. The copies on
  that machine predate -3.
- **Measurement gotcha**: the bot's sandbox uses `--unshare-pid`,
  so a background spinner started in one shell is invisible to
  every other one — `pgrep`/`pkill` silently find nothing and
  cannot stop it. The only reliable "machine is quiet again"
  signal is the `timeout` expiring. Two rounds of measurements
  were taken under contention before this was understood.
- **Two loose threads**: `d_low.min_window_ns` swung ~8%
  between attempts in a quiet debug run (the invariant catches
  the fallout, but the input is unstable) — we think the
  window-shape mismatch is the cause (d_low was 40 windows x
  2,500 samples against l_low's 1,000 x 1,000, so its min is
  an order statistic over 25x fewer, 2.5x-longer windows), and
  -4's shape alignment is the designed fix; verified gone
  (three release runs repeated `frame/call` to 3 decimals).
  The second — debug `frame_call` on r5-7600x read 11.79 /
  14.72 ns against release's 25.4, backwards, since an
  unoptimized timer pair should cost *more*, not less — was
  never explained; with the constants being dropped
  (Replanning II) it is moot unless the mechanism matters for
  the micro-probe.

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

1. Drop overhead adjustment; grade the run from raw batches —
   the 0.23.0 cycle, decided in
   [Replanning II](notes/chores/chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run)
   - remove startup calibration, the constants block, adjusted
     columns, and the `calibrate` command; raw values only,
     one README sentence on apparatus framing
   - `pick_inner` sizing from a ~1 ms micro-probe (low
     quantile over back-to-back timer pairs), never printed
   - per-run quality gauge computed at the end from the run's
     own data: samples land in raw time-ordered batches,
     per-batch summaries (floor, mean, census counts) feed the
     gauge, then bulk-record into the histogram; relocate the
     -5 grade machinery (signals, thresholds, letter, warnings)
   - absorbs the interference-crossover entry's rate analysis
     (below) — batches give it the time axis
   - the overhead.rs deletion largely replaces the planned
     acquisition/estimation refactor
2. Unit scaling in report columns (`us`/`ms`) — per-row
   auto-scale so columns stay eyeball-comparable (bands are
   monotonic, so a row's first/last/mean share a magnitude),
   or `--units ns|auto` for script-stable output; needs
   `--decimals` landed first (`3.18 ms` vs `3 ms`); candidate
   `-4` for the report-options cycle.
3. Trimmed core stats: `mean/stdev p10-p90` report row,
   additional to (never replacing) `mean` / `mean min-p99`;
   trim bounds possibly configurable (`--trim p10:p90`?) —
   the full mean wobbles ~±1.4% with the run's mode mix while
   the core plateau is ~±0.2% stable, so the trimmed row is
   the run-to-run comparable number. Boundary sensitivity
   (see [[57]]): window edges in the mode-mix smear inherit
   its wobble (p50-p60 ±0.05% vs p40-p50 ~1%), so also
   consider a dominant-*mode* statistic (peak-density region,
   bottom-count-independent) [[57]]
4. Find and label the interference crossover — the band where
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
5. Upstream the ladder commit-ref convention to
   `../vc-template-x1`: In Progress ladder rungs (and the
   chores As-built rungs) carry a prepended `[[N]]`
   commit-ref placeholder, backfilled as each commit
   becomes permanent — template's cycle-protocol.md,
   AGENTS.md, and TODO.md example need the shape; that
   repo has its own approval/push flow
6. Investigate: suspend gap missing from samples. A 0.13.5
   `--no-inhibit` suspend test detected ~1.2 s suspended inside
   the measured window but the max sample was only 4.0 ms,
   while the 0.13.1 test (8.4 s gap) showed the expected 10.4 s
   max sample. We think minstant's TSC may halt across some
   suspends and count through others. Repeat the test comparing
   detected gap vs max sample; if the TSC halts, per-sample
   timing silently loses suspend time — document either way.
7. CLAUDE.md governance model (design cogitation) [20]
8. Revisit probe adjustment under the in-interval vs
   call-to-call split: probes take one call per sample
   (inner=1), so the in-interval timer slice is unamortized
   and unmeasurable — an `adjusted` column can subtract
   nothing defensible; maybe state a bound instead
   [analysis](notes/design.md#timer-overhead-in-interval-vs-call-to-call)
9. Convert `harness` / `Bench` to probe-based measurement. Will
   likely need inner-loop support on `Probe` (batch N calls per
   sample; report divides by N and accounts for per-sample
   framing) so very-small workloads can still amortize timer
   overhead the way `run_adaptive` does today.
10. Rename app
11. Design an app to measure IIAC perforanace written in Rust[1]
12. `ice-ps-2t-wait` — iceoryx2 pub/sub with blocking waits via
    `Listener`/`Notifier` events; completes the {transport} ×
    {wait policy} matrix cell that compares against `mpsc-2t`
13. Switch ice benches to the loan-based zero-copy send path
    (`loan_uninit` + `send`) — the API a perf-sensitive user would
    use, and closer to iceoryx2's own benchmark method
14. Payload-size sweep for the round-trip benches (8 B / 8 KiB /
    1 MiB) — makes iceoryx2's size-independent latency vs channel
    copy cost visible in our own tables
15. `crossbeam-1t` / `crossbeam-2t` — `crossbeam-channel` directly
    (compare to mpsc-1t/2t which use crossbeam under the std API)
16. `tokio-mpsc-1t` / `tokio-mpsc-2t` — `tokio::sync::mpsc` round-trip
    inside a Tokio runtime (async overhead)
17. `flume-1t` / `flume-2t` — `flume` MPMC channel
18. Function-call baselines: direct call vs `Box<dyn Trait>` vs
    `async fn` (poll-once) — anchors the channel/serde numbers
    against the cheapest possible "send a value then receive it" path
19. When the second channel impl lands, extract shared message types
    + round-trip helpers into `src/benches/common.rs` (deferred from 0.2.0)
20. Additional thread control (count, per-thread pin lists, NUMA) —
    shape once a concrete bench needs it
21. Rename crate `iiac-perf` → general-purpose name (breaking; deferred)

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

## Bugs

_See [bugs.md](notes/bugs.md)._

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](notes/done.md) to keep this file small.

- Upstream the AGENTS.md "Plain synopsis after technical
  explanations" section to vc-template-x1 — landed upstream
  (template also gained Speculation marker + Model delegation);
  retired when the converged doc set was copied back here
- docs: converge shared protocol doc set [[58]]
- docs: adopt TODO.md-at-root protocol [[58]]
- feat: amortized + cached calibration [[59]]

# References

[1]: /README.md#Design-010
[20]: /notes/chores/chores-02.md#claudemd-governance-model-071
[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[58]: /notes/chores/chores-04.md#as-built-ladder-1
[59]: /notes/chores/chores-04.md#feat-amortized--cached-calibration
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[62]: https://github.com/winksaville/iiac-perf/commit/6d5784de861b "6d5784de861b872b6012709cf4969be57a383823"
[63]: https://github.com/winksaville/iiac-perf/commit/f9d4770cdf14 "f9d4770cdf1464c856d93ae5d27d2e9468a5ffca"
[64]: https://github.com/winksaville/iiac-perf/commit/50bfadedf33d "50bfadedf33d0b2b39552f810e7631b402de7305"
[65]: https://github.com/winksaville/iiac-perf/commit/275ff298c1dc "275ff298c1dc3108f531c1be05944a79ec3f15ce"
