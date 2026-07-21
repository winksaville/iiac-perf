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

_No cycle currently in progress._

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
