# Chores 05

Continuation of [chores-04](chores-04.md). Records landed work;
conventions in [AGENTS.md](../../AGENTS.md#chores-conventions) and
[cycle-protocol.md](../cycle-protocol.md#chores-sections).

## feat: grade the run from raw batches

Commits:

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

- [[N]] 0.23.0-0 `chore: open raw-batch grading cycle`

# References
