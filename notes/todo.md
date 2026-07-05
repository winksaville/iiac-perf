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

_No cycle currently in progress._

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

1. iiac-perf config file — `~/.config/iiac-perf/config.toml`
   (XDG), optional project-local override, CLI always wins;
   serde+toml deps. Homes for: named pin profiles
   (e.g. smt=0,12 / ccx=0,1 / ccd=0,6), default duration, band
   label style, decimals. Prerequisite for `--band-labels`.
2. `--band-labels zpn|frac|both` — band label style option;
   `frac` = literal fractions with `_` grouping (`0.999_999`);
   default `both` for learnability; header line records the
   style in use so saved outputs are self-describing.
3. Decimals on mean/adjusted columns (`--decimals N` + config) —
   cheap display change, the values are already f64.
4. Record histogram in picoseconds — true sub-ns
   first/last/range resolution (matters for 1t benches where
   inner-division truncates today); touches `round_elapsed`,
   the 60 s clamp constant, and display scaling.
5. Investigate: suspend gap missing from samples. A 0.13.5
   `--no-inhibit` suspend test detected ~1.2 s suspended inside
   the measured window but the max sample was only 4.0 ms,
   while the 0.13.1 test (8.4 s gap) showed the expected 10.4 s
   max sample. We think minstant's TSC may halt across some
   suspends and count through others. Repeat the test comparing
   detected gap vs max sample; if the TSC halts, per-sample
   timing silently loses suspend time — document either way.
6. CLAUDE.md governance model (design cogitation) [20]
7. Add framing adjustment to `Probe::report` (subtract
   `Overhead::framing_per_sample_ns` ≈ 11 ns in an `adjusted`
   column, mirroring `harness::print_report`)
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

# References

[1]: /README.md#Design-010
[15]: /notes/chores-02.md#todochores-tidy-070-dev1
[16]: /notes/chores-02.md#reframe-docs-as-general-perf-tool-070-dev2
[17]: /notes/chores-02.md#per-item-doc-comments--print_histogram-rename-070-dev3
[18]: /notes/chores-02.md#bench-trait--module-split-080-candidate
[19]: /notes/chores-02.md#070-release-070
[20]: /notes/chores-02.md#claudemd-governance-model-071
[21]: /notes/chores-02.md#design-actor-runtime--probe-microbench-system-080-dev0
[22]: /notes/chores-02.md#plan-probe-primitive--probe-mpsc-2t-080-dev1
[23]: /notes/chores-02.md#implement-probe-primitive--probe-mpsc-2t-080-dev2
[24]: /notes/chores-02.md#producer-consumer-bench-probe-only-ux-experiment-080-dev3
[25]: /notes/chores-02.md#tprobe--tp-pc--tsc-gate--ticks-flag-080-dev4
[26]: /notes/chores-02.md#arch-neutral-ticks-module--cpuid-invariant-tsc-080-dev5
[27]: /notes/chores-02.md#080-release--claudemd-memory-policy-080
[28]: /notes/chores-03.md#plan-tprobe-startend-090-dev1
[29]: /notes/chores-03.md#implement-tprobe-startend--buffer-090-dev2
[30]: /notes/chores-03.md#lazy-report-drain-records--histogram-090-dev3
[31]: /notes/chores-03.md#wire-tp-pc-to-tprobe-startend-090-dev4
[32]: /notes/chores-03.md#split-tprobe2--revert-tprobe--tp2-pc-090-dev5
[33]: /notes/chores-03.md#090-release-tprobe2-scope-api--tp2-pc-090
[34]: /notes/chores-03.md#plan-iceoryx2-benches--pubsub--reqres-1t2t-0100-dev1
[35]: /notes/chores-03.md#implement-ice-ps-1t--ice-ps-2t-0100-dev2
[36]: /notes/chores-03.md#implement-ice-rr-1t--ice-rr-2t-0100-dev3
[37]: /notes/chores-03.md#0100-release-iceoryx2-benches-0100
[38]: /notes/chores-03.md#mpsc-2t-spin-bench-0110
[39]: /notes/chores-03.md#aarch64-ticks-impl-0120
[40]: /notes/chores/chores-04.md#feat-zcr-bench-family-rawwithspin-1t2t
[41]: /notes/chores/chores-04.md#fix-saturate-hist-records-flag-suspended-runs
[42]: /notes/chores/chores-04.md#fix-report-column-alignment
[43]: /notes/chores/chores-04.md#feat-finer-report-tail-bands
[44]: /notes/chores/chores-04.md#feat-inhibit-sleep-during-bench-runs
[45]: /notes/chores/chores-04.md#feat-nineszeros-tail-bands-z4n10
[46]: /notes/chores/chores-04.md#fix-number-todo-entries-per-agents-todo-format
