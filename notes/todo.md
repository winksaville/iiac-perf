# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

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

See [Foramt details](README.md#todo-format)

-

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

- Add timer overhead measurement comparing minstant vs Instant::now[2]
- Refactor to Bench trait + add std::sync::mpsc channel bench [3]
- Multi-thread mpsc + per-bench files + named CLI + adaptive sizing [4]
- Tune duration default + add `-D/--total-duration` flag [5]
- Add duration to bench header + logfmt-style metadata [6]
- Auto-size histogram columns [7]
- Add `--pin` CPU affinity flag [8]
- Band-based histogram display [9]
- Fix `core_affinity` pinning bug [10]
- Rename CLI flags: `-i` → `-o/--outer`, `-I` → `-i/--inner` [11]

# References

[11]: /notes/chores-01.md#rename-cli-flags--iterations---outer--inner---inner-037
[10]: /notes/chores-01.md#fix-core_affinity-pinning-bug-036
[1]: /README.md#Design-010
[2]: /notes/chores-01.md#measure-timer-overhead-010
[3]: /notes/chores-01.md#refactor-to-bench-trait--add-channel-bench-020
[4]: /notes/chores-01.md#multi-thread-mpsc--per-bench-files--named-cli-030
[5]: /notes/chores-01.md#tune-duration-default--add-total-duration-flag-031
[6]: /notes/chores-01.md#add-duration-to-bench-header--logfmt-style-metadata-032
[7]: /notes/chores-01.md#auto-size-histogram-columns-033
[8]: /notes/chores-01.md#add-pin-cpu-affinity-flag-034
[9]: /notes/chores-01.md#band-based-histogram-display-035
