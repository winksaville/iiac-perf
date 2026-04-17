# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

## Todo

A markdown list of task to do in the near feature

- CLAUDE.md governance model (design cogitation) [20]
- Rename app
- Design an app to measure IIAC perforanace written in Rust[1]
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
