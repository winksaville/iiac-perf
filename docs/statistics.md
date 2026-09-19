# The statistics behind a run's claim

iiac-perf exists to answer one question: a change was made, did it help or hurt? This page explains
the numbers the tool prints toward that answer, what each means in plain words, its formula, the
name statisticians give it with a link to a source, and where it misleads. Every figure is drawn
from `records/knobs.jsonl` by [figures/make.py](figures/make.py): 25 invocations of ten 5 s runs
and one of a hundred 1 s runs, all `zcr-spsc-v3-2t` on the 7600X with the clock pinned.

A word first. A *block* is about 55 ms of measuring, a *run* is one process of 100 blocks, and an
*invocation* is one command of several runs. The quantity compared between two versions of the
code is an invocation's mean.

## One number is not enough

Run the same code twice and the two means differ. So a difference between an old version and a
new one says nothing until it is known how far apart two measurements of the *same* code land. All
of what follows is about that distance: measuring it, shrinking it, and spending the least time
doing so.

![Every baseline run, by invocation](figures/runs.svg)

Each gray dot is a run's mean. Most sit in a tight core near 95 ns. Some sit far above it, as high
as 152 ns, and the lower panel is the upper one zoomed on the core. The orange marks are each
invocation's plain mean, which one high run drags up by several percent. The blue marks are its
trimmed mean, which holds still.

| | lowest invocation | highest invocation |
|---|---|---|
| trimmed mean | 94.52 ns | 96.54 ns |
| plain mean | 94.63 ns | 103.27 ns |

## The trimmed mean

Sort the runs' means, drop the highest 20% and the lowest 20%, and average what is left. Of ten
runs that is the middle six. A run that landed somewhere slow is dropped rather than averaged in.
This is the [truncated mean](https://en.wikipedia.org/wiki/Truncated_mean).

Its uncertainty cannot be the spread of the six kept runs, which would be too small because the
extremes were thrown away. Yuen's method replaces each dropped run by the nearest kept one
([Winsorizing](https://en.wikipedia.org/wiki/Winsorizing)), takes the standard deviation of that
set, `wsd`, and uses

    se = wsd / (0.6 * sqrt(n))

with `n` the runs and 0.6 the fraction kept ([Yuen 1974](https://doi.org/10.1093/biomet/61.1.165)).
`se` is the *standard error*: how far this trimmed mean is expected to sit from the one an endless
number of runs would give.

Where it misleads: a 20% trim survives up to 20% bad runs and no more.

![A hundred one-second runs](figures/levels.svg)

Of these hundred 1 s runs 22 sit on a slow level. The trim drops the top 20, so two slow runs stay
in, and the band of kept runs reaches 98.7 ns where the core ends near 96. The trimmed mean, 95.82
ns, is pulled up with it. The 5 s invocations had about 14% slow runs and stayed inside the
trim's reach.

More runs do not fix this, because the limit is a fraction of the runs and not a count. Four later
invocations of 30 one-second runs, the same code each time, drew 30%, 13%, 33%, and 17% slow runs,
and their trimmed means spanned 1.7%. Timing noise is one-sided, a run can land slow and nothing
makes it faster than the code allows, which is why Python's
[timeit](https://docs.python.org/3/library/timeit.html#timeit.Timer.repeat) advises the minimum
over the mean. A trim that cuts more from the top than the bottom follows from that, and keeping
the 10th to the 50th percentile put those four invocations within 0.3% of one another. It is the
next change to the tool, to be checked on data that had no part in choosing it.

## The slow runs are levels, not disturbances

![The blocks of a slow run and of a normal one](figures/blocks.svg)

The slowest run, 151.6 ns, is slow in every one of its hundred blocks, from 150.4 to 152.8 ns. A
disturbance, another program waking up, would show as a spike in a few blocks. A flat line from
start to finish says the process was slow from the moment it started: a property of where it
landed, re-rolled with every process start. We think it is where its memory landed. It is why
runs are separate processes, and why they, not blocks, are what a claim counts.

## CI95: how well one invocation knows its own mean

    CI95 = t * se

The [confidence interval](https://www.itl.nist.gov/div898/handbook/eda/section3/eda352.htm): were
the invocation repeated many times, the interval mean ± CI95 would cover the true mean in 95 of
100. `t` is the Student-t factor that widens the interval when runs are few, about 2.6 for ten runs
trimmed to six and approaching 1.96 for many. The plain pair is `t(n-1) * sd / sqrt(n)`, the
trimmed pair `t(kept-1) * se`. The bars in the first figure are CI95.

Where it misleads: it describes one measurement. Two intervals that overlap a little can still
differ, and CI95 is not the yardstick for comparing two.

## LSC: the smallest difference worth believing

    LSC = t * se * sqrt(2)

The least significant change. A difference is two means subtracted, each with its own error, and
independent errors add as squares, so a difference is `sqrt(2)` times as uncertain as either mean.
This is the two-sample [t-test](https://www.itl.nist.gov/div898/handbook/eda/section3/eda353.htm)
turned around: not "is this difference real" but "how large must one be". The rule it gives:

> Measure the old code and the new under the same config. If their trimmed means differ by more
> than LSC, the change did something. If by less, the measurement cannot tell.

Is the rule honest? That can be checked, because the 25 invocations all ran the same code. Any
two of them are an experiment in which the right answer is "no difference", and there are 300 such
pairs.

![Two invocations of the same code, all 300 pairs](figures/same-code.svg)

| | half the pairs within | 95% within | pairs beyond their LSC |
|---|---|---|---|
| trimmed means | 0.35% | 1.81% | 7 of 300, 2.3% |
| plain means | 1.47% | 7.06% | 6 of 300, 2.0% |

A 95% rule should raise a false alarm in about 5% of such pairs, and both raise fewer, so LSC can
be trusted. The difference between the two is what they can see. The plain pair is honest by
being wide: its LSC is often 5% or more, so it cannot see a 3% improvement. The trimmed pair is
honest and narrow.

Where it misleads: the trimmed histogram has a second hump near 1.5%. Some invocations sit a little
high as a whole, not through any one run. LSC's 2.3% covers them here, but a claim resting on one
pair of invocations is weaker than one that repeats. The `analyze` command is to make that check
routine.

## Two kinds of noise: a and s_p

Why ten runs of 5 s and not one run of 50 s? Because there are two noises and more measuring
shrinks only one of them.

![Noise within a run beside the spread between runs](figures/within-between.svg)

On the left, one run's blocks wander by about ±0.5 ns, but there are a hundred of them, so the run
knows its own mean to ±0.12 ns. On the right are the means of all 215 normal-level runs. They
differ from one another by ±0.55 ns, five times what any of them is unsure of. Each run is
precisely measured and precisely different.

- `a` is the noise within a run, scaled to one second of measuring. A run of `d` seconds has a
  within-run variance of `a / d`, so measuring longer shrinks it. Here `a` is about 0.079 ns² s.
- `s_p` is the spread between processes, what is left of the run-to-run spread once the
  within-run part is taken out. No length of run shrinks it, only more runs. Here about 0.54 ns.

An invocation of `R` runs of `d` seconds has a mean with variance

    (s_p^2 + a/d) / R

This is a [random-effects model](https://en.wikipedia.org/wiki/Random_effects_model), the same
split a [gauge study](https://www.itl.nist.gov/div898/handbook/mpc/section4/mpc4.htm) makes between
repeatability and reproducibility, and the one [Kalibera and Jones](https://kar.kent.ac.uk/33611/)
apply to benchmarking, levels of repetition each with a variance and a cost.

## Blocks are not independent

Computing `a` needs to know how many independent measurements a run's hundred blocks amount to. If
a block tends to resemble its neighbour they are worth fewer than a hundred.

![How a block correlates with later blocks](figures/correlation.svg)

The blue line is the measured [autocorrelation](https://en.wikipedia.org/wiki/Autocorrelation): a
block correlates +0.42 with the next and still +0.17 with the block five later. The common
shortcut assumes the correlation dies away geometrically, the gray line, which by lag five would be
+0.01. It does not, and the shortcut would count the blocks as 41. Summing the measured
correlations until they reach zero, the *effective sample size* as the [Stan reference
manual](https://mc-stan.org/docs/reference-manual/analysis.html) defines it, counts them as 23. An
earlier version of this analysis used the shortcut and understated `a` by nearly half.

Where it misleads: this count is itself an estimate from noisy correlations. Treat it as "about a
quarter", not as 23.

## o and d*: the best run length

Each run also costs time in which nothing is measured: spawning the process, settling, warming,
the sleep before the run. That is `o`, the overhead, in seconds per run. With it an invocation
takes

    T = R * (o + d)

Long runs waste time on `a`, which is already small. Short runs pay `o` over and over. Between
them is a best length,

    d* = sqrt(a * o / s_p^2)

found by minimizing `T` at a fixed variance, and the same `d*` gives the least variance at a fixed
`T`.

![Total time against run length at two overheads](figures/run-length.svg)

| overhead `o` | best run length `d*` | time for a 0.5% LSC |
|---|---|---|
| 3.6 s (settle 1.5 s, sleep 1-2 s) | 0.99 s | 169 s |
| 0.3 s (settle 0.1 s, sleep 0.1 s) | 0.28 s | 33 s |

Both curves are flat near their lowest point, so a run half or twice `d*` costs little. The lever
is `o`: cutting it moves the whole curve down.

Where it misleads, three ways. The model takes `a`, `s_p`, and the fraction of slow runs to be the
same at every run length and overhead, and the hundred 1 s runs already question that: 22% slow
against 14% at 5 s. A short settle leaves a run starting about 0.45% slow in its first 100 ms, a
bias, which no count of runs averages away. And the curve is drawn from normal-level runs, so it
is the cost once the slow runs are dealt with. So the figure says where to look, and an experiment
at that setting decides.

## What is settled and what is not

Settled on the 7600X for this bench: LSC trimmed is an honest yardstick at ten 5 s runs, the slow
runs are whole-process levels, runs vary five times more than they are uncertain, and blocks are
worth about a quarter of their count.

Not settled: a statistic that survives more than 20% slow runs, whether short runs with a short
settle are unbiased, and why some invocations sit high as a whole. The formulas are standard. The
judgments around them, what counts as a slow run, which runs are the core, are this project's own
and have had no outside review.
