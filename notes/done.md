# Done

As TODO.md `## Done` sections fills move them to here.

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

## Through 0.12.0

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

# References

[2]: /notes/chores/chores-01.md#measure-timer-overhead-010
[3]: /notes/chores/chores-01.md#refactor-to-bench-trait--add-channel-bench-020
[4]: /notes/chores/chores-01.md#multi-thread-mpsc--per-bench-files--named-cli-030
[5]: /notes/chores/chores-01.md#tune-duration-default--add-total-duration-flag-031
[6]: /notes/chores/chores-01.md#add-duration-to-bench-header--logfmt-style-metadata-032
[7]: /notes/chores/chores-01.md#auto-size-histogram-columns-033
[8]: /notes/chores/chores-01.md#add-pin-cpu-affinity-flag-034
[9]: /notes/chores/chores-01.md#band-based-histogram-display-035
[10]: /notes/chores/chores-01.md#fix-core_affinity-pinning-bug-036
[11]: /notes/chores/chores-01.md#rename-cli-flags--iterations---outer--inner---inner-037
[12]: /notes/chores/chores-01.md#time-based-outer-loop-040
[13]: /notes/chores/chores-01.md#add-range-column-to-histogram-050
[14]: /notes/chores/chores-02.md#calibration-robustness-060
[15]: /notes/chores/chores-02.md#todochores-tidy-070-dev1
[16]: /notes/chores/chores-02.md#reframe-docs-as-general-perf-tool-070-dev2
[17]: /notes/chores/chores-02.md#per-item-doc-comments--print_histogram-rename-070-dev3
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
