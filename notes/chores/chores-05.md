# Chores 05

Continuation of [chores-04](chores-04.md). Records landed work;
conventions in [AGENTS.md](../../AGENTS.md#chores-conventions) and
[cycle-protocol.md](../cycle-protocol.md#chores-sections).

## feat: grade the run from raw batches

Commits: [[1]]

Decided in
[Replanning II](chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run):
the overhead subtraction estimates an ill-defined quantity —
additivity is an approximation on a superscalar CPU, the
constants moved ~10% with frequency regimes, and the correction
cancels in same-harness A/B anyway — while the calibration-time
grade certified a ~1 s window *before* the run: the room, not
the exam. This cycle removes the adjustment machinery and moves
grading onto the run's own time-ordered batch data.

### As-built ladder

- [[1]] 0.23.0-0 `chore: open raw-batch grading cycle`
- [[N]] 0.23.0-1 `feat: micro-probe inner-loop sizing` —
  `pick_inner`'s frame input now comes from a ~1 ms
  micro-probe (low quantile over back-to-back timer pairs)
  instead of `cfg.overhead.frame_call_ns`; sizing no longer
  depends on startup calibration. Also lands
  `tests/settle_anomaly.rs`, the `#[ignore]`d settle-anomaly
  acceptance test for the dynamic-warmup Todo — captured
  while the failing baseline (calibrate letter) still exists
  — and per-signal letters on the environment line
  (`CalGrade::signal_letters`), a display shakedown for the
  -4 gauge: every composite letter now names its cause

# References

[1]: https://github.com/winksaville/iiac-perf/commit/621c5c97dbe1 "621c5c97dbe1418fdcb99db6080eecde40891491"
