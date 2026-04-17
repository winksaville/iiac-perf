# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

- `0.7.0` — final release

See [docs/cleanup plan](chores-02.md#todochores-tidy-070-dev1).

## Todo

A markdown list of task to do in the near feature

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

# References

[1]: /README.md#Design-010
[15]: /notes/chores-02.md#todochores-tidy-070-dev1
[16]: /notes/chores-02.md#reframe-docs-as-general-perf-tool-070-dev2
[17]: /notes/chores-02.md#per-item-doc-comments--print_histogram-rename-070-dev3
