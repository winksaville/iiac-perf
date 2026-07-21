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

**feat: amortized + cached calibration**

Framing is measured un-amortized, so it inherits the full
~10 ns TSC quantum (run-to-run reports span 0-21 ns on the
3900X; cold runs on r5-7600x clamp to 0.00) and the estimate
sizes `inner`, so a low draw under-sizes the experiment (up to
~9% relative error, worst case ~50% apparatus contamination).
Fix per
[analysis](notes/design.md#calibration-accuracy-framing-quantization),
revised mid-cycle by the in-interval vs call-to-call finding
([analysis](notes/design.md#timer-overhead-in-interval-vs-call-to-call)):
amortized two-point window calibration (error q/M on both
points) yields `frame_call_ns` (sizes `inner`; never
subtracted) and `loop_per_iter_ns` (the only subtraction);
cache the constants in the config file with provenance + a
live validity check, and say cached vs live in the report
header. Calibration banner values print at 3 decimals always
(sub-ns resolution is the point; `--decimals` remains the
report-table knob). The -2 experiment precedes the cache steps
because its outcome decides which constants the cache stores:
if the dithered mean-based fit's intercept is stable
run-to-run
([analysis](notes/design.md#dithering-random-phase-injection)), it
*becomes* the calibration — loop/iter and in-interval framing
with per-block CIs (subtraction returns), plus one window pass
for the call-to-call sizing constant. -3 also dithers the
harness sample seam (Bench-driven benches only: a run's
aggregate means stop carrying the coherent ±q phase bias,
~±2% on fast benches; a seam delay inside a probe bench's
closed loop would perturb what's measured, so that stays a
separate opt-in idea). -4 pulls "Within-invocation
replication: sleep-separated blocks" forward
([analysis](notes/design.md#within-invocation-replication-sleep-separated-blocks)):
`--blocks N` splits a run into N sleep-separated blocks
(random-ms sleep + unrecorded post-wake warm-up per block),
reports block mean ± 95% CI and the LSC, Bench-driven benches
only; ANOVA validation (between-block vs between-invocation)
on r5-7600x decides how honest the single-run CI is.
Revised again at -5: the cache + validity check are dropped —
dithering already made the constants repeatable (the lottery
the cache targeted), and a cached constant goes stale under
hardware / frequency-regime change
([analysis](notes/design.md#design-cached-calibration-in-the-config-file));
-5 instead lands a standalone `calibrate` diagnostic command
(calibration only: constants + raw fit inputs, no bench run).

- [[59]] 0.21.0-0 chore: open cached-calibration cycle (done)
- [[60]] 0.21.0-1 feat: amortized two-point calibration (done)
- [[61]] 0.21.0-2 feat: dithered calibration experiment (done)
- [[62]] 0.21.0-3 feat: dithered calibration + seam dither (done)
- [[63]] 0.21.0-4 feat: sleep-separated block replication (done)
- [[64]] 0.21.0-5 feat: calibrate diagnostic command (done)
- [[65]] 0.21.0-6 feat: shell completion generation (done)
- [[66]] 0.21.0-7 feat: dynamic bench-name completion (done)
  - retitled post-publish from "live bench-name completion"
    (rewrite + force-push, both repos) — before any Commits
    backfill recorded the old SHA, so no reference broke
- [[67]] 0.21.0-8 docs: adopt TODO.md-at-root protocol (done)
  - converge AGENTS.md + notes/cycle-protocol.md with
    vc-template-x1 (TODO.md at repo root, Model delegation,
    per-commit chores build-up); move notes/todo.md →
    /TODO.md; seed notes/bugs.md + notes/todo-backlog.md;
    converge notes/README.md; retire the
    upstream-Plain-synopsis Todo (landed upstream)
- [[N]] 0.21.0-9 feat: completion self-install command (done)
  - add-completion-yaml command: generate the carapace spec
    and write it to --completion-dir (default
    $XDG_CONFIG_HOME/carapace/specs, ~/.config fallback —
    carapace's own lookup), fixed name iiac-perf.yaml;
    create the dir, overwrite on re-run, print the path
  - no-args listing: when the default spec path is absent,
    hint: For command completion execute
    'iiac-perf add-completion-yaml -h'; silent when present
  - rider: add a clap CompleteEnv (unstable-dynamic) idea to
    ## Ideas — compact column view for bash die-hards;
    revisit if clap stabilizes dynamic completion
- [[N]] 0.21.0 close-out
  - usual bookkeeping; backfill remaining As-built refs
    (-8/-9; the close-out's own ref lands one push later);
    Done entry uses the close-out title per convention

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

1. Unit scaling in report columns (`us`/`ms`) — per-row
   auto-scale so columns stay eyeball-comparable (bands are
   monotonic, so a row's first/last/mean share a magnitude),
   or `--units ns|auto` for script-stable output; needs
   `--decimals` landed first (`3.18 ms` vs `3 ms`); candidate
   `-4` for the report-options cycle.
2. Trimmed core stats: `mean/stdev p10-p90` report row,
   additional to (never replacing) `mean` / `mean min-p99`;
   trim bounds possibly configurable (`--trim p10:p90`?) —
   the full mean wobbles ~±1.4% with the run's mode mix while
   the core plateau is ~±0.2% stable, so the trimmed row is
   the run-to-run comparable number. Boundary sensitivity
   (see [[57]]): window edges in the mode-mix smear inherit
   its wobble (p50-p60 ±0.05% vs p40-p50 ~1%), so also
   consider a dominant-*mode* statistic (peak-density region,
   bottom-count-independent) [[57]]
3. Upstream the ladder commit-ref convention to
   `../vc-template-x1`: In Progress ladder rungs (and the
   chores As-built rungs) carry a prepended `[[N]]`
   commit-ref placeholder, backfilled as each commit
   becomes permanent — template's cycle-protocol.md,
   AGENTS.md, and TODO.md example need the shape; that
   repo has its own approval/push flow
4. Investigate: suspend gap missing from samples. A 0.13.5
   `--no-inhibit` suspend test detected ~1.2 s suspended inside
   the measured window but the max sample was only 4.0 ms,
   while the 0.13.1 test (8.4 s gap) showed the expected 10.4 s
   max sample. We think minstant's TSC may halt across some
   suspends and count through others. Repeat the test comparing
   detected gap vs max sample; if the TSC halts, per-sample
   timing silently loses suspend time — document either way.
5. CLAUDE.md governance model (design cogitation) [20]
6. Revisit probe adjustment under the in-interval vs
   call-to-call split: probes take one call per sample
   (inner=1), so the in-interval timer slice is unamortized
   and unmeasurable — an `adjusted` column can subtract
   nothing defensible; maybe state a bound instead
   [analysis](notes/design.md#timer-overhead-in-interval-vs-call-to-call)
7. Convert `harness` / `Bench` to probe-based measurement. Will
   likely need inner-loop support on `Probe` (batch N calls per
   sample; report divides by N and accounts for per-sample
   framing) so very-small workloads can still amortize timer
   overhead the way `run_adaptive` does today.
8. Rename app
9. Design an app to measure IIAC perforanace written in Rust[1]
10. `ice-ps-2t-wait` — iceoryx2 pub/sub with blocking waits via
    `Listener`/`Notifier` events; completes the {transport} ×
    {wait policy} matrix cell that compares against `mpsc-2t`
11. Switch ice benches to the loan-based zero-copy send path
    (`loan_uninit` + `send`) — the API a perf-sensitive user would
    use, and closer to iceoryx2's own benchmark method
12. Payload-size sweep for the round-trip benches (8 B / 8 KiB /
    1 MiB) — makes iceoryx2's size-independent latency vs channel
    copy cost visible in our own tables
13. `crossbeam-1t` / `crossbeam-2t` — `crossbeam-channel` directly
    (compare to mpsc-1t/2t which use crossbeam under the std API)
14. `tokio-mpsc-1t` / `tokio-mpsc-2t` — `tokio::sync::mpsc` round-trip
    inside a Tokio runtime (async overhead)
15. `flume-1t` / `flume-2t` — `flume` MPMC channel
16. Function-call baselines: direct call vs `Box<dyn Trait>` vs
    `async fn` (poll-once) — anchors the channel/serde numbers
    against the cheapest possible "send a value then receive it" path
17. When the second channel impl lands, extract shared message types
    + round-trip helpers into `src/benches/common.rs` (deferred from 0.2.0)
18. Additional thread control (count, per-thread pin lists, NUMA) —
    shape once a concrete bench needs it
19. Rename crate `iiac-perf` → general-purpose name (breaking; deferred)

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

- feat: zcr bench family (raw/with/spin, 1t/2t) [[40]]
- fix: saturate hist records, flag suspended runs [[41]]
- fix: report column alignment [[42]]
- feat: finer report tail bands [[43]]
- feat: inhibit sleep during bench runs [[44]]
- feat: nines/zeros tail bands (z4..n10) [[45]]
- fix: number todo entries per AGENTS todo format [[46]]
- feat: report options + ps recording [[47]]
- feat: config file + pin profiles [[48]]
- refactor: drop zcr raw/spin bench tiers [[49]]
- fix: trim label spans populated bands [[50]]
- fix: upper-closed band intervals [[51]]
- docs: add "Reading a report" to README [[52]]
- feat: zcr-mpsc-1t/2t benches [[53]]
- docs: add notes/design.md (calibration accuracy) [[54]]
- refactor: move chores-01..03 into notes/chores/ [[55]]
- fix: probe decimals + startup robustness [[56]]
- Upstream the AGENTS.md "Plain synopsis after technical
  explanations" section to vc-template-x1 — landed upstream
  (template also gained Speculation marker + Model delegation);
  retired when the converged doc set was copied back here
- docs: converge shared protocol doc set [[58]]
- docs: adopt TODO.md-at-root protocol [[58]]

# References

[1]: /README.md#Design-010
[20]: /notes/chores/chores-02.md#claudemd-governance-model-071
[40]: /notes/chores/chores-04.md#feat-zcr-bench-family-rawwithspin-1t2t
[41]: /notes/chores/chores-04.md#fix-saturate-hist-records-flag-suspended-runs
[42]: /notes/chores/chores-04.md#fix-report-column-alignment
[43]: /notes/chores/chores-04.md#feat-finer-report-tail-bands
[44]: /notes/chores/chores-04.md#feat-inhibit-sleep-during-bench-runs
[45]: /notes/chores/chores-04.md#feat-nineszeros-tail-bands-z4n10
[46]: /notes/chores/chores-04.md#fix-number-todo-entries-per-agents-todo-format
[47]: /notes/chores/chores-04.md#feat-report-options--ps-recording
[48]: /notes/chores/chores-04.md#feat-config-file--pin-profiles
[49]: /notes/chores/chores-04.md#refactor-drop-zcr-rawspin-bench-tiers
[50]: /notes/chores/chores-04.md#fix-trim-label-spans-populated-bands
[51]: /notes/chores/chores-04.md#fix-upper-closed-band-intervals
[52]: /notes/chores/chores-04.md#docs-add-reading-a-report-to-readme
[53]: /notes/chores/chores-04.md#feat-zcr-mpsc-1t2t-benches
[54]: /notes/chores/chores-04.md#docs-add-notesdesignmd-calibration-accuracy
[55]: /notes/chores/chores-04.md#refactor-move-chores-0103-into-noteschores
[56]: /notes/chores/chores-04.md#fix-probe-decimals--startup-robustness
[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[58]: /notes/chores/chores-04.md#as-built-ladder-1
[59]: https://github.com/winksaville/iiac-perf/commit/17ad4f779036 "17ad4f7790366e702eb42b220a8d01a33541bba6"
[60]: https://github.com/winksaville/iiac-perf/commit/c44a49f1dd53 "c44a49f1dd539545e316b132be08cd8be3eccc34"
[61]: https://github.com/winksaville/iiac-perf/commit/7f3d2b923127 "7f3d2b923127f60fac8d436541596d2f35264ed5"
[62]: https://github.com/winksaville/iiac-perf/commit/471422a92dc1 "471422a92dc12b15c68626f5131b55ec4870c15f"
[63]: https://github.com/winksaville/iiac-perf/commit/25ee3f63b053 "25ee3f63b05324dccf5be28a6e4257db7056d1ce"
[64]: https://github.com/winksaville/iiac-perf/commit/d82df5ae17d9 "d82df5ae17d955a0e945b2e72d2a342090316f33"
[65]: https://github.com/winksaville/iiac-perf/commit/19a29ef805af "19a29ef805af5adf46ccd963a8463aee9019ba91"
[66]: https://github.com/winksaville/iiac-perf/commit/f3ee5cc0bb36 "f3ee5cc0bb36702d863ef6c1755a1f649a225496"
[67]: https://github.com/winksaville/iiac-perf/commit/5b5882bc589f "5b5882bc589f2a3f478744898f10318b57d93958"
