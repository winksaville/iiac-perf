# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

_No cycle currently in progress._

## Todo

A markdown list of task to do in the near feature

- CLAUDE.md governance model (design cogitation) [20]
- Add framing adjustment to `Probe::report` (subtract
  `Overhead::framing_per_sample_ns` ≈ 11 ns in an `adjusted`
  column, mirroring `harness::print_report`)
- Convert `harness` / `Bench` to probe-based measurement. Will
  likely need inner-loop support on `Probe` (batch N calls per
  sample; report divides by N and accounts for per-sample
  framing) so very-small workloads can still amortize timer
  overhead the way `run_adaptive` does today.
- Rename app
- Design an app to measure IIAC perforanace written in Rust[1]
- `ice-ps-2t-wait` — iceoryx2 pub/sub with blocking waits via
  `Listener`/`Notifier` events; completes the {transport} ×
  {wait policy} matrix cell that compares against `mpsc-2t`
- Switch ice benches to the loan-based zero-copy send path
  (`loan_uninit` + `send`) — the API a perf-sensitive user would
  use, and closer to iceoryx2's own benchmark method
- Payload-size sweep for the round-trip benches (8 B / 8 KiB /
  1 MiB) — makes iceoryx2's size-independent latency vs channel
  copy cost visible in our own tables
- `crossbeam-1t` / `crossbeam-2t` — `crossbeam-channel` directly
  (compare to mpsc-1t/2t which use crossbeam under the std API)
- `tokio-mpsc-1t` / `tokio-mpsc-2t` — `tokio::sync::mpsc` round-trip
  inside a Tokio runtime (async overhead)
- `flume-1t` / `flume-2t` — `flume` MPMC channel
- Function-call baselines: direct call vs `Box<dyn Trait>` vs
  `async fn` (poll-once) — anchors the channel/serde numbers
  against the cheapest possible "send a value then receive it" path
- When the second channel impl lands, extract shared message types
  + round-trip helpers into `src/benches/common.rs` (deferred from 0.2.0)
- Additional thread control (count, per-thread pin lists, NUMA) —
  shape once a concrete bench needs it
- Rename crate `iiac-perf` → general-purpose name (breaking; deferred)

See [Foramt details](README.md#todo-format)

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
