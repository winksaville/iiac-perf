# Done

As todo.md `## Done` sections fills move them to here.

See [Todo format](README.md#todo-format)

## Through 0.6.0

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
- Time-based outer loop [12]
- Add `range` column + trimmed mean/stdev to histogram [13]
- Calibration robustness: stable framing, unpin main after cal,
  `-v/--verbose` + log infra, `--no-pin-cal` opt-out [14]

# References

[14]: /notes/chores-02.md#calibration-robustness-060

[13]: /notes/chores-01.md#add-range-column-to-histogram-050

[12]: /notes/chores-01.md#time-based-outer-loop-040
[11]: /notes/chores-01.md#rename-cli-flags--iterations---outer--inner---inner-037
[10]: /notes/chores-01.md#fix-core_affinity-pinning-bug-036
[2]: /notes/chores-01.md#measure-timer-overhead-010
[3]: /notes/chores-01.md#refactor-to-bench-trait--add-channel-bench-020
[4]: /notes/chores-01.md#multi-thread-mpsc--per-bench-files--named-cli-030
[5]: /notes/chores-01.md#tune-duration-default--add-total-duration-flag-031
[6]: /notes/chores-01.md#add-duration-to-bench-header--logfmt-style-metadata-032
[7]: /notes/chores-01.md#auto-size-histogram-columns-033
[8]: /notes/chores-01.md#add-pin-cpu-affinity-flag-034
[9]: /notes/chores-01.md#band-based-histogram-display-035
