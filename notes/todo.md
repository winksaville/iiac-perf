# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

- Refactor to Bench trait + add channel bench [3]
  - `0.2.0-dev1` ✅ chore marker: version bump + plan (+ CLAUDE.md rule)
  - `0.2.0-dev2` ✅ Bench trait + harness refactor
  - `0.2.0-dev3` ✅ apparatus calibration via EmptyBench + adjusted column
  - `0.2.0-dev4` ✅ `std::sync::mpsc` single-thread bench + CLI dispatch
  - `0.2.0` finalize

## Todo

A markdown list of task to do in the near feature

- Design an app to measure IIAC perforanace written in Rust[1]

See [Foramt details](README.md#todo-format)

-

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

- Add timer overhead measurement comparing minstant vs Instant::now[2]

# References

[1]: /README.md#Design-010
[2]: /notes/chores-01.md#measure-timer-overhead-010
[3]: /notes/chores-01.md#refactor-to-bench-trait--add-channel-bench-020
