# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

- Multi-thread mpsc + per-bench files + named CLI [4]
  - `0.3.0-dev1` split timer into per-impl files, registry + named-list CLI
  - `0.3.0-dev2` `mpsc-2t` cross-thread round-trip + add future bench todos
  - `0.3.0` finalize

## Todo

A markdown list of task to do in the near feature

- Design an app to measure IIAC perforanace written in Rust[1]

See [Foramt details](README.md#todo-format)

-

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

- Add timer overhead measurement comparing minstant vs Instant::now[2]
- Refactor to Bench trait + add std::sync::mpsc channel bench [3]

# References

[1]: /README.md#Design-010
[2]: /notes/chores-01.md#measure-timer-overhead-010
[3]: /notes/chores-01.md#refactor-to-bench-trait--add-channel-bench-020
[4]: /notes/chores-01.md#multi-thread-mpsc--per-bench-files--named-cli-030
