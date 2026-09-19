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
| trimmed mean | 94.26 ns | 95.34 ns |
| plain mean | 94.63 ns | 103.27 ns |

## The trimmed mean

Sort the runs' means, drop the lowest 10% and the highest 50%, and average what is left. Of ten
runs that is the second to the fifth lowest. The runs that landed somewhere slow are dropped rather
than averaged in. This is the [truncated mean](https://en.wikipedia.org/wiki/Truncated_mean), cut
unequally.

The cut leans high because timing noise is one-sided: a run can land slow, and nothing makes it
faster than the code allows. Python's
[timeit](https://docs.python.org/3/library/timeit.html#timeit.Timer.repeat) advises the minimum
over the mean for that reason. The band is a middle road: it leans low as the minimum does, and it
averages several runs rather than trusting one, so it has an error bar. The low 10% goes as a guard
against a stray fast run.

Its uncertainty cannot be the spread of the kept runs, which would be too small because the rest
were thrown away. Yuen's method replaces each dropped run by the nearest kept one
([Winsorizing](https://en.wikipedia.org/wiki/Winsorizing)), takes the standard deviation of that
set, `wsd`, and uses

    se = wsd / (share * sqrt(n))

with `n` the runs and `share` the fraction kept, 0.4 here
([Yuen 1974](https://doi.org/10.1093/biomet/61.1.165) for equal cuts, and
[Stigler 1973](https://doi.org/10.1214/aos/1176342412) for when a trimmed mean behaves normally at
all: the cut points must sit where the runs are dense, which the middle of the core is). `se` is
the *standard error*: how far this trimmed mean is expected to sit from the one an endless number
of runs would give.

The tool first trimmed 20% from each end, and that failed on this data.

![A hundred one-second runs](figures/levels.svg)

Of these hundred 1 s runs 22 sit on a slow level. A 20% trim drops the top 20, so two slow runs
stayed in, its kept band reached 98.7 ns where the core ends near 96, and its mean read 95.82 ns.
More runs do not fix that, because the limit is a fraction of the runs and not a count. Four later
invocations of 30 one-second runs, the same code each time, drew 30%, 13%, 33%, and 17% slow runs,
and their 20% trimmed means spanned 1.7%. The band in the figure is the one the tool keeps now,
94.8 to 95.7 ns, its mean 95.30 ns, and on those four invocations it spans 0.3%.

Where it misleads, two ways. The band is the core's lower part, so the trimmed mean reads about
0.2 ns under the core's centre. Two versions of a bench shift alike, so a comparison is unharmed,
but it is not "the mean", which is why the plain rows stay beside it. And it hides how many runs
landed slow, which a change to the code could move, so look at the run lines too. The band is the
`trim_runs` key, `"10-50"` by default, with `"20-80"` the old one. Set it once for a project: a
trim picked after seeing the numbers flatters them. This one was picked on some of the data here,
so it was then checked on the 25 invocations, which had no part in the choice. That check is
below.

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
100. `t` is the Student-t factor that widens the interval when runs are few, about 3.2 for ten runs
trimmed to four and approaching 1.96 for many. The plain pair is `t(n-1) * sd / sqrt(n)`, the
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
| trimmed means, `10-50` | 0.22% | 0.69% | 41 of 300, 13.7% |
| trimmed means, the old `20-80` | 0.35% | 1.81% | 7 of 300, 2.3% |
| plain means | 1.47% | 7.06% | 6 of 300, 2.0% |

Two answers, and they differ. As an estimate the `10-50` trim is the best of the three by far: two
invocations of the same code land within 0.69% of each other 95 times in 100, where the old trim
gives 1.81% and the plain mean 7%. As an error bar its LSC is too confident: a 95% rule should raise
a false alarm in about 5% of such pairs and it raises 13.7%.

Shuffling all 250 runs into random invocations, which removes anything a whole invocation shares,
brings that to 7.8%, and to 6.3% at thirty runs an invocation. So a small part is the formula
running a little hot on four kept runs. The larger part is real: a whole invocation sits about
0.3% high or low, every run of it together, and nothing computed inside one invocation can see
what all its runs share. The old trim and the plain mean pass only because slow runs inflate their
LSC enough to cover it.

So, on the 7600X today:

> One invocation against one invocation resolves a difference of about 0.7%, whatever LSC prints.
> A smaller claim needs several invocations of each version, alternated, with the spread between
> invocations as the yardstick.

LSC is still what to read within an invocation, it says whether the runs were quiet. The
`analyze` command is to make the several-invocation comparison routine.

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
| 3.6 s (settle 1.5 s, sleep 1-2 s) | 0.99 s | 381 s |
| 0.3 s (settle 0.1 s, sleep 0.1 s) | 0.28 s | 74 s |

Both curves are flat near their lowest point, so a run half or twice `d*` costs little. The lever
is `o`: cutting it moves the whole curve down.

Where it misleads, three ways. The model takes `a`, `s_p`, and the fraction of slow runs to be the
same at every run length and overhead, and the slow fraction alone has read anywhere from 11% to
33% between invocations. The curve is drawn from normal-level runs and knows nothing of the 0.3%
an invocation shifts as a whole, so its seconds are the cost of a tight LSC, not of a true 0.5%.
And it says nothing of bias. An experiment did: settle 1.5 s against 0.1 s and sleep 1-2 s against
0.1 s, a hundred runs a cell on two benches, moved the median by less than two repeats of one
cell differ, so the short overhead is safe here. So the figure says where to look, and an
experiment at that setting decides.

## What is settled and what is not

Settled on the 7600X for this bench: the slow runs are whole-process levels, a trim leaning high
sets them aside, runs vary five times more than they are uncertain, blocks are worth about a
quarter of their count, and a short settle and sleep cost nothing in accuracy.

Not settled: why a whole invocation sits 0.3% high or low, which is what now limits a comparison,
and the several-invocation comparison that would get under it. The formulas are standard. The
judgments around them, the trim's two cuts above all, are this project's own and have had no
outside review.
