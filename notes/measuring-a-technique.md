# Measuring a technique, not a binary

The zcr benches exist to test techniques meant for many applications, compilers, and operating
systems, where every number this tool prints belongs to one host, one build, and one stretch of
time. This file states what a claim about a technique needs on top of a report, what the tool
covers today, and where its portable half ends.

A technique is a way of doing inter- or intra-application communication: a ring layout, a handoff
protocol, a memory ordering choice, a spin against a park. A binary is one implementation of one
technique on one platform, so a claim about the technique has to outlive the binary, which is what
makes it cost more than a number.

Written 2026-09-15 at `feat: CI95 and LSC across processes`, from that cycle's measurements on the
3900X and the 7600x.

## Two claims, two costs

The everyday question and the durable one are not the same claim, and holding both to the stricter
standard prices tuning out of reach.

- **Did this change help?** Two versions of one technique, kept or dropped on the answer. It needs
  the two measured under the same conditions, which means one invocation with the arms alternating,
  and nothing more. The answer is about these two binaries on this host, which is all the decision
  needs.
- **Is this technique faster than that one?** The claim that outlives the binary and reaches another
  application or platform. It needs replicated builds, so a layout difference is not read as the
  technique's, and a table across environments, since no interval computed on one says anything
  about another.
- The cheap claim is the inner two replicates below, the run and the block, plus the pairing. The
  durable claim adds the build and the environment. So a tuning session pays for runs and pairing,
  and only a published claim pays for builds and hosts.

## The claim is a ratio, not a latency

"The v1 ring costs 70.5 ns" is a fact about a 7600x at 4.67 GHz, one build, one kernel, and the
minutes the runs took. Nothing carries it to another machine.

- A ratio between two implementations measured under identical conditions does carry, since the
  conditions cancel in the pair: the clock, the kernel, the microarchitecture, and the day.
- So a technique's claim is a ratio with its interval, and the absolute numbers are context, worth
  publishing as the conditions the ratio was measured under rather than as the result.
- Identical conditions means the same invocation with the arms alternating, not two invocations,
  for the reason the evidence below gives.

## The replicates nest

Four scales, each covering what the one inside it cannot, listed from the outside in.

- **Environment**: host, microarchitecture, OS, kernel, compiler. No interval computed on one
  environment says anything about another, so a technique's results are a table, one row per
  environment with its own ratio and interval, and the range across them stated as a range.
- **Build**: the binary's code layout, fixed for the life of a binary. A rebuild of this repo moved
  `zcr-mpsc-v1-2t` from 67.9 to 60.8 ns with no code change, 11%, which is larger than most
  differences a ring design produces. With one build per arm, layout is confounded with the change
  and no number of runs reveals it. k builds per arm, the same source with a deliberate layout
  perturbation, turn layout into spread that averages.
- **Run**: one process, which re-rolls where the rings and stacks land. Covered by `--runs`, whose
  `CI95 runs` and `LSC runs` are the cycle's subject.
- **Block**: a slice of one run, covered by `--blocks`. It sees the run's own noise and drift and
  nothing outside the process.

The cost of a level goes up as you go out, so the design question is which level to add to, and
the answer is the level whose spread dominates. The tool measures the two inner levels itself, which
with the pairing is what the everyday claim needs, and the outer two are what a durable claim adds.

## What the tool covers today

- Runs and blocks, with their error bars, and the trimmed pair for a host that disturbs some runs.
- The environment, as provenance rather than replication: the record's host block, the clock
  policy, the `Config:` list, and the per-run delivered clock.
- Not builds: a build is one binary, and nothing re-rolls its layout.
- Not the environment as a replicate: a cross-environment table is the "Analyze a directory of
  records" entry's cross-run tier.

## The evidence behind the ratio rule

Same binary, same knobs, an A/A comparison across invocations, clock pinned, both hosts:

- 3900X, `zcr-mpsc-v1-2t --runs 20 -d 1 --pin-cpus 2,3 --pin-freq`, two invocations about 90
  minutes apart: 383.7 ns with `LSC runs` 3.2 and 387.0 ns with `LSC runs` 2.5. The difference,
  3.3 ns, clears the larger LSC, so the tool called two runs of the same code different.
- 7600x, the same command, three invocations over about two hours: 70.0, 70.4, and 70.6 ns, a
  spread of 0.6 ns against `CI95 runs` of 0.1 to 0.7, every run reading 4.67 GHz.
- So pinning the clock removes the drift an unpinned 3900X showed, 22.8 against 22.5 ns on
  `min-now` where `LSC runs` was 0.1, and something slower survives it. We think the candidates are
  thermal state, the physical pages the kernel hands out as memory fragments over hours, and
  background load, and no run of a single invocation re-rolls any of them.
- A pair measured in one invocation, the arms alternating, cancels whatever that is, which is what
  makes the ratio the portable statistic. Two benches measured back to back in one invocation do
  not alternate, so they carry the drift between their stretches, which is why the guide's
  comparison section asks for the clock pin and a repeat.

## Checking a bar, not reading it

`CI95 runs` and `LSC runs` are read as the precision of the number above them, and the section
before shows a bar can be wrong by more than it claims. Two things break one, and neither shows in
the bar itself.

- **The replicates under it are not independent.** The arithmetic divides a spread by the square
  root of a count, which assumes every replicate is a fresh draw. The 240 records of `feat: a run
  is a config file` put the lag-1 autocorrelation of the block means at +0.78 on the unpinned
  3900X, so a hundred blocks carry about eleven blocks' worth of information and `CI95 blocks` is
  understated about threefold. Pinning the clock takes it to +0.20 there, and the 7600X reads
  +0.02 pinned or not. A wandering clock is what couples one block to the next, so the pin buys
  the independence the arithmetic had already assumed.
- **A level above them moves.** Runs inside one invocation re-roll what a process start re-rolls
  and nothing else, so anything drifting between invocations is invisible to `CI95 runs`. That is
  the A/A above, two invocations of one binary 3.3 ns apart, clearing both their LSCs.

So a bar is checked rather than read. Run the same config several times and compare the spread of
the invocation means against the standard error one invocation claimed, `stdev / sqrt(runs)`. Near
one that ratio says the claim is honest, well above one it is not, and about five invocations are
wanted before the ratio means much, since it carries their count less one degree of freedom.
[`configs/knobs.py`](../configs/knobs.py) prints it beside the claim.

Two consequences. Pin the clock, so the inner replicates are the draws the bar assumes. And where
a claim must be an absolute rather than a ratio, state it with the calibration measured, since the
bar alone does not carry it.

## The portable core and the environment layer

The harness splits, and the seam is worth keeping sharp, since the outer half is Linux-shaped and
the techniques are not.

- **Portable**: the timing loop, the blocks, the histogram, the statistics, the record schema, and
  the report. Arithmetic and formatting, with no host control in them.
- **Linux**: cpufreq pinning and its udev permissions, `systemd-inhibit`, CPU affinity,
  `/dev/cpu_dma_latency`, and the sysfs clock reads.
- **Windows**: thread affinity and priority exist, power plans are settable, and the clock is
  QueryPerformanceCounter. The same interfaces with another implementation.
- **macOS**: no per-core pinning on Apple silicon, affinity hints at best, and no user-level clock
  control. The environment layer mostly becomes reporting what it cannot control, which the grade
  block already does.
- **A bare-metal target**: no processes, so one bench per process has no meaning there. The
  concept is re-rolling placement between replicates, and a child process is this platform's
  implementation of it. Randomizing allocation offsets inside the program, as Stabilizer does
  (Curtsinger and Berger, 2013), is the portable form of the same idea, and it is also what k
  builds approximate for code layout.

The risk this seam guards against is the machine-control half growing faster than the techniques it
exists to test.

# References
