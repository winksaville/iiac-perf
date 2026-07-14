# Todo

This file uses [Prose form](../AGENTS.md#prose-form). It
contains near term tasks with a short description and
uses links or reference links for more details.

## In Progress

When a `## Todo` item is picked up, its text moves here: the
problem overview and its list of things to do. That is followed
by the "plan" — a bulleted list of the development "ladder":
   - 0.xx.y-0 blah (done)
   - 0.xx.y-1 blah blah (current)
   - 0.xx.y-2 blah blah blah
   - 0.xx.y close-out and validation

**feat: amortized + cached calibration**

Framing is measured un-amortized, so it inherits the full
~10 ns TSC quantum (run-to-run reports span 0-21 ns on the
3900X; cold runs on r5-7600x clamp to 0.00) and the estimate
sizes `inner`, so a low draw under-sizes the experiment (up to
~9% relative error, worst case ~50% apparatus contamination).
Fix per
[analysis](design.md#calibration-accuracy-framing-quantization),
revised mid-cycle by the in-interval vs call-to-call finding
([analysis](design.md#timer-overhead-in-interval-vs-call-to-call)):
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
([analysis](design.md#dithering-random-phase-injection)), it
*becomes* the calibration — loop/iter and in-interval framing
with per-block CIs (subtraction returns), plus one window pass
for the call-to-call sizing constant.

- 0.21.0-0 chore: open cached-calibration cycle (done)
- 0.21.0-1 feat: amortized two-point calibration (done)
- 0.21.0-2 feat: dithered calibration experiment (done)
- 0.21.0-3 feat: dithered fit becomes the calibration
- 0.21.0-4 feat: calibrate command + config cache
- 0.21.0-5 feat: cached calibration + validity check
- 0.21.0 close-out

## Todo

Entries are in **strict priority rank** — #1 highest,
descending. Reprioritize by moving an entry, then
`vc-x1 fix-todo --no-dry-run notes/todo.md` to renumber.
The numbers are positional rank, not stable IDs — to refer
to a Todo, name it by its **title** (a greppable mention;
a numbered list item has no anchor to link to), not its
number. Use the
[Prose Form in AGENTS.md](../AGENTS.md#prose-form); deeper
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
   the run-to-run comparable number [[57]]
3. Upstream the AGENTS.md "Plain synopsis after technical
   explanations" section to `../vc-template-x1/AGENTS.md` (the
   template this repo's AGENTS.md derives from) — at or before
   the 0.21.0 close-out; that repo has its own approval/push
   flow
4. docs: README "Comparing implementations" section — surface
   the LSC / 95%-confidence method (interleaved runs, per-run
   statistic, two-sample t) for users; content in
   [design.md](design.md#comparing-implementations-least-significant-change)
5. Investigate: suspend gap missing from samples. A 0.13.5
   `--no-inhibit` suspend test detected ~1.2 s suspended inside
   the measured window but the max sample was only 4.0 ms,
   while the 0.13.1 test (8.4 s gap) showed the expected 10.4 s
   max sample. We think minstant's TSC may halt across some
   suspends and count through others. Repeat the test comparing
   detected gap vs max sample; if the TSC halts, per-sample
   timing silently loses suspend time — document either way.
6. CLAUDE.md governance model (design cogitation) [20]
7. Revisit probe adjustment under the in-interval vs
   call-to-call split: probes take one call per sample
   (inner=1), so the in-interval timer slice is unamortized
   and unmeasurable — an `adjusted` column can subtract
   nothing defensible; maybe state a bound instead
   [analysis](design.md#timer-overhead-in-interval-vs-call-to-call)
8. Convert `harness` / `Bench` to probe-based measurement. Will
   likely need inner-loop support on `Probe` (batch N calls per
   sample; report divides by N and accounts for per-sample
   framing) so very-small workloads can still amortize timer
   overhead the way `run_adaptive` does today.
9. Rename app
10. Design an app to measure IIAC perforanace written in Rust[1]
11. `ice-ps-2t-wait` — iceoryx2 pub/sub with blocking waits via
    `Listener`/`Notifier` events; completes the {transport} ×
    {wait policy} matrix cell that compares against `mpsc-2t`
12. Switch ice benches to the loan-based zero-copy send path
    (`loan_uninit` + `send`) — the API a perf-sensitive user would
    use, and closer to iceoryx2's own benchmark method
13. Payload-size sweep for the round-trip benches (8 B / 8 KiB /
    1 MiB) — makes iceoryx2's size-independent latency vs channel
    copy cost visible in our own tables
14. `crossbeam-1t` / `crossbeam-2t` — `crossbeam-channel` directly
    (compare to mpsc-1t/2t which use crossbeam under the std API)
15. `tokio-mpsc-1t` / `tokio-mpsc-2t` — `tokio::sync::mpsc` round-trip
    inside a Tokio runtime (async overhead)
16. `flume-1t` / `flume-2t` — `flume` MPMC channel
17. Function-call baselines: direct call vs `Box<dyn Trait>` vs
    `async fn` (poll-once) — anchors the channel/serde numbers
    against the cheapest possible "send a value then receive it" path
18. When the second channel impl lands, extract shared message types
    + round-trip helpers into `src/benches/common.rs` (deferred from 0.2.0)
19. Additional thread control (count, per-thread pin lists, NUMA) —
    shape once a concrete bench needs it
20. Rename crate `iiac-perf` → general-purpose name (breaking; deferred)

## Ideas

Longer-range thoughts, not yet ranked work. `-` bullets, no
numbering; promote into `## Todo` when one becomes actionable.

- `--runs n` repeat mode: run a bench n times in one
  invocation, report the per-run statistic, mean ± 95% CI, and
  the implied LSC directly — the honest, user-facing form of
  the
  [LSC method](design.md#comparing-implementations-least-significant-change);
  naive single-run LSC is deliberately not offered (a run
  can't see between-run variance). Possibly superseded by
  sleep-separated blocks + paired A/B interleaving —
  [analysis](design.md#within-invocation-replication-sleep-separated-blocks)
- Dithering: random sub-quantum spin between samples so
  quantization error averages away in means (classic ADC
  dither); possible route to a measurable in-interval framing
  via a dithered mean-based two-point fit —
  [analysis](design.md#dithering-random-phase-injection)
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

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

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
