# Chores-01

Discussions and notes on various chores in github compatible markdown.
There is also a [todo.md](todo.md) file and it tracks tasks and in
general there should be a chore section for each task with the why
and how this task will be completed.

See [Chores format](README.md#chores-format)

## A dummy chore (TBD)

A dummy chore description.

## Measure timer overhead (0.1.0)

Added initial Rust app that measures the overhead of `minstant::Instant::now()`
vs `std::time::Instant::now()`. Uses `minstant` as the reference timer for both
measurements and records 1M iterations into `hdrhistogram` histograms. Reports
min, p50, p90, p99, p99.9, p99.99, max, mean, and stdev in nanoseconds.

This bootstraps the benchmarking project by measuring the measuring stick first,
validating the measurement approach before applying it to actual IIAC techniques.
