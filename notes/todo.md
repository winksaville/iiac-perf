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

**fix: probe reports honor --decimals**

`--decimals` only reaches the harness `print_report` path.
`Probe::report` and the shared `band_table` renderer used by
`TProbe`/`TProbe2` hardcode 0 decimals, so `producer-consumer`,
`probe-mpsc-2t`, `tp-pc`, and `tp2-pc` probe tables ignore the
flag.

- 0.20.1-1 fix: thread decimals into probe reports (done)
- 0.20.1 close-out

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

1. Amortized framing calibration + cached constants in config
   — framing is quantized to the ~10 ns TSC lattice and the
   estimate sizes `inner`, so a low draw under-sizes the
   experiment (up to ~9% relative error, worst case ~50%
   apparatus contamination); measure M timer pairs in one
   window (error q/M), cache in config with provenance +
   live validity check, header says cached vs live
   [analysis](design.md#calibration-accuracy-framing-quantization)
2. Unit scaling in report columns (`us`/`ms`) — per-row
   auto-scale so columns stay eyeball-comparable (bands are
   monotonic, so a row's first/last/mean share a magnitude),
   or `--units ns|auto` for script-stable output; needs
   `--decimals` landed first (`3.18 ms` vs `3 ms`); candidate
   `-4` for the report-options cycle.
3. Investigate: suspend gap missing from samples. A 0.13.5
   `--no-inhibit` suspend test detected ~1.2 s suspended inside
   the measured window but the max sample was only 4.0 ms,
   while the 0.13.1 test (8.4 s gap) showed the expected 10.4 s
   max sample. We think minstant's TSC may halt across some
   suspends and count through others. Repeat the test comparing
   detected gap vs max sample; if the TSC halts, per-sample
   timing silently loses suspend time — document either way.
4. CLAUDE.md governance model (design cogitation) [20]
5. Add framing adjustment to `Probe::report` (subtract
   `Overhead::framing_per_sample_ns` ≈ 11 ns in an `adjusted`
   column, mirroring `harness::print_report`)
6. Convert `harness` / `Bench` to probe-based measurement. Will
   likely need inner-loop support on `Probe` (batch N calls per
   sample; report divides by N and accounts for per-sample
   framing) so very-small workloads can still amortize timer
   overhead the way `run_adaptive` does today.
7. Rename app
8. Design an app to measure IIAC perforanace written in Rust[1]
9. `ice-ps-2t-wait` — iceoryx2 pub/sub with blocking waits via
   `Listener`/`Notifier` events; completes the {transport} ×
   {wait policy} matrix cell that compares against `mpsc-2t`
10. Switch ice benches to the loan-based zero-copy send path
    (`loan_uninit` + `send`) — the API a perf-sensitive user would
    use, and closer to iceoryx2's own benchmark method
11. Payload-size sweep for the round-trip benches (8 B / 8 KiB /
    1 MiB) — makes iceoryx2's size-independent latency vs channel
    copy cost visible in our own tables
12. `crossbeam-1t` / `crossbeam-2t` — `crossbeam-channel` directly
    (compare to mpsc-1t/2t which use crossbeam under the std API)
13. `tokio-mpsc-1t` / `tokio-mpsc-2t` — `tokio::sync::mpsc` round-trip
    inside a Tokio runtime (async overhead)
14. `flume-1t` / `flume-2t` — `flume` MPMC channel
15. Function-call baselines: direct call vs `Box<dyn Trait>` vs
    `async fn` (poll-once) — anchors the channel/serde numbers
    against the cheapest possible "send a value then receive it" path
16. When the second channel impl lands, extract shared message types
    + round-trip helpers into `src/benches/common.rs` (deferred from 0.2.0)
17. Additional thread control (count, per-thread pin lists, NUMA) —
    shape once a concrete bench needs it
18. Rename crate `iiac-perf` → general-purpose name (breaking; deferred)

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

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

- `0.7.0-dev1` — todo/chores tidy [15]
- `0.7.0-dev2` — reframe docs as general perf tool [16]
- `0.7.0-dev3` — per-item doc comments + `print_histogram` rename [17]
- `0.7.0` — docs/cleanup release [19]
- `0.7.1` — capture CLAUDE.md governance design note [20]
- `0.8.0-dev0` — design: actor runtime + probe microbench system [21]
- `0.8.0-dev1` — plan: probe primitive + probed mpsc-2t [22]
- `0.8.0-dev2` — implement probe primitive + probed mpsc-2t [23]
- `0.8.0-dev3` — producer-consumer bench (probe-only UX experiment) [24]
- `0.8.0-dev4` — TProbe + tp-pc + TSC gate + `-t/--ticks` [25]
- `0.8.0-dev5` — arch-neutral `ticks` module + CPUID invariant-TSC check [26]
- `0.8.0` — release + CLAUDE.md memory policy [27]
- `0.9.0-dev1` — plan: TProbe start/end [28]
- `0.9.0-dev2` — implement: TProbe start/end + record buffer [29]
- `0.9.0-dev3` — lazy report drain: records → histogram [30]
- `0.9.0-dev4` — wire tp-pc to TProbe start/end [31]
- `0.9.0-dev5` — split TProbe2 + revert TProbe + tp2-pc bench [32]
- `0.9.0` — TProbe2 scope API + tp2-pc release [33]
- `0.10.0-dev1` — plan: iceoryx2 benches ice-ps/ice-rr [34]
- `0.10.0-dev2` — implement ice-ps-1t + ice-ps-2t [35]
- `0.10.0-dev3` — implement ice-rr-1t + ice-rr-2t [36]
- `0.10.0` — iceoryx2 benches release [37]
- `0.11.0` — mpsc-2t-spin bench [38]
- `0.12.0` — aarch64 ticks impl [39]
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

# References

[1]: /README.md#Design-010
[15]: /notes/chores/chores-02.md#todochores-tidy-070-dev1
[16]: /notes/chores/chores-02.md#reframe-docs-as-general-perf-tool-070-dev2
[17]: /notes/chores/chores-02.md#per-item-doc-comments--print_histogram-rename-070-dev3
[18]: /notes/chores/chores-02.md#bench-trait--module-split-080-candidate
[19]: /notes/chores/chores-02.md#070-release-070
[20]: /notes/chores/chores-02.md#claudemd-governance-model-071
[21]: /notes/chores/chores-02.md#design-actor-runtime--probe-microbench-system-080-dev0
[22]: /notes/chores/chores-02.md#plan-probe-primitive--probe-mpsc-2t-080-dev1
[23]: /notes/chores/chores-02.md#implement-probe-primitive--probe-mpsc-2t-080-dev2
[24]: /notes/chores/chores-02.md#producer-consumer-bench-probe-only-ux-experiment-080-dev3
[25]: /notes/chores/chores-02.md#tprobe--tp-pc--tsc-gate--ticks-flag-080-dev4
[26]: /notes/chores/chores-02.md#arch-neutral-ticks-module--cpuid-invariant-tsc-080-dev5
[27]: /notes/chores/chores-02.md#080-release--claudemd-memory-policy-080
[28]: /notes/chores/chores-03.md#plan-tprobe-startend-090-dev1
[29]: /notes/chores/chores-03.md#implement-tprobe-startend--buffer-090-dev2
[30]: /notes/chores/chores-03.md#lazy-report-drain-records--histogram-090-dev3
[31]: /notes/chores/chores-03.md#wire-tp-pc-to-tprobe-startend-090-dev4
[32]: /notes/chores/chores-03.md#split-tprobe2--revert-tprobe--tp2-pc-090-dev5
[33]: /notes/chores/chores-03.md#090-release-tprobe2-scope-api--tp2-pc-090
[34]: /notes/chores/chores-03.md#plan-iceoryx2-benches--pubsub--reqres-1t2t-0100-dev1
[35]: /notes/chores/chores-03.md#implement-ice-ps-1t--ice-ps-2t-0100-dev2
[36]: /notes/chores/chores-03.md#implement-ice-rr-1t--ice-rr-2t-0100-dev3
[37]: /notes/chores/chores-03.md#0100-release-iceoryx2-benches-0100
[38]: /notes/chores/chores-03.md#mpsc-2t-spin-bench-0110
[39]: /notes/chores/chores-03.md#aarch64-ticks-impl-0120
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
