# Todo and cycle record

This file contains near term tasks with a short description and reference links to more details.
Its shape is [Todo format](agent-data/notes.md#todo-format).

## Continuation notes

Where the agent was, for the agent that comes next: working copy state, the step in flight, an
open question. Ephemeral, never a record. Written before a restart or when a session is about to
lose context, read first at acquaint, acted on, each fact filed into its home or its bullet kept, and
the rest reset to `_None._` by the reader.

- `feat: CI95 and LSC across processes` is complete on `feat-ci95-and-lsc-across-processes`,
  pushed, not landed (2026-09-15): the opening, fifteen work rungs, and the closing, six of the
  rungs inserted after wink read the pushed work on both hosts. Land is wink's: rename the package
  back to `iiac-perf`, `vc-x1 validate --fast` so the lockfile follows, `jj squash` the edit into
  the closing, reshape to the chosen close-out shape, fast-forward `main`, install the plain
  `iiac-perf`, and delete the bookmark locally and remotely.
- The close-out shape is a trapezoid, recorded in the closing rung's subsection, so Land runs the
  trapezoid recipe in [jj.md](agent-data/jj.md#trapezoid-close-out-recipe) before the fast-forward.
- Owed on the hosts after Land: the plain `iiac-perf` 0.28.14 on both, the 7600x's by copy, and the
  stale `iiac-perf-dev` builds removed.

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. The block's
shape is the specimen in [cycle-model.md](agent-data/cycle-model.md), and the rules are in
[The In Progress block](agent-data/notes.md#the-in-progress-block).

_No cycle currently in progress._

## Waiting

Important work that cannot start yet. Each entry names what it waits on and its rank once
unblocked, and every opening checks the conditions.

_None._

## Todo

Entries are in priority order, the first highest, and reprioritizing moves the entry. The
long-tail backlog is in [todo-backlog.md](notes/todo-backlog.md), and deeper detail lives in
the frozen `notes/chores/` design subsections, linked by `[N]` refs.

### Re-record all on the 7600x across processes

The 7600x's `all` rows were recorded one process for every bench, so each row carries whatever
placement that process drew (split from `feat: CI95 and LSC across processes` at its opening, the
cycle making each bench its own runs). The first use of the cycle's runs.

- `all` with `--record` into a directory that stays, whose records' host block starts the
  cross-host comparison
- a run of the mpsc v1 pair there, whose `all` rows are the renamed v0 rows

### Does the 3900X's unpinned shift follow its clock

Two unpinned 3900X invocations of `min-now` a minute apart read 22.8 and 22.5 ns, each with `LSC
runs` of 0.1 ns, while two pinned with `--pin-freq --run-sleep 1s` both read 26.3 ns (wink,
2026-09-15, in `feat: CI95 and LSC across processes`). We think the shift is the clock's state, but
the pinned pair changed the sleep and the pin together, so neither cause is shown.

- alternating unpinned invocations with `--record`, each run's mean set against its recorded
  delivered clock, `clock_khz`: a shift that follows the clock names the cause
- the same at `--run-sleep 0` against the `1-2s` default, unpinned and pinned, to see whether the
  sleep moves a run's reading at all
- on the 7600x too, whose unpinned `min-now` pair agreed at the display's precision then

### A --config flag and a config key for every run parameter

A comparison across hosts or days is a bench list and a dozen knobs typed as flags each time, so
two runs meant to be identical differ by whatever a hand forgot (wink, 2026-09-05, after the
placement sweep). A run should be definable as a config file and named on the line. Split from
`feat: config and setup` at its opening, which carries the `Config:` list and the record's config,
since spawning can pass flags on the command line and check the children against the record.

- `--config PATH` loads that file as the top layer over the XDG and project-local files, the flags
  still winning, and the banner names it with the rest. No such flag exists today, the loader
  knowing only the two fixed locations
- every CLI run parameter gets a config key, the mirror of "Config keys stay CLI-settable" below,
  which pairs each key with a flag. Today `duration`, `band_labels`, `decimals`, `settle_time`,
  `warm_cap`, and the three block keys have keys, and `--total-duration`, `--samples`, `--inner`,
  `--pin-cpus` (profiles name a spec, but nothing selects one by default), `--record`, `--tag`,
  `--no-env-probe`, `--no-inhibit`, `--ticks`, and `--verbose` do not. `--pin-freq` has one,
  `pin_freq`, from `feat: config and setup`
- the bench list is a key too, so a config file is a complete run, `iiac-perf --config
  placement.md` and nothing else on the line. The `benches` key and `--benches` come from `feat:
  CI95 and LSC across processes`, taken from this entry at its opening, leaving `--config` to name
  the file
- the `[freq]` exclusion stands: the steady state is the host's declaration, not a run's
- with spawning, a config also names the children's knobs, and an A/B is two configs or one with
  two arms, which is the shape a cross-host comparison wants. The cycle runs each bench's runs back
  to back, not interleaved (wink, at its opening)
- a benchmark directory's config pinning the clock (wink, 2026-09-14): a project-local `[freq]`
  with `min_mhz = max_mhz` also moves where a restore returns, so the pin became a run key instead,
  `pin_freq`, in `feat: config and setup`. What remains here is a boost option for a pin that
  should keep boost on, if benchmarking wants one
- the project-local search: today the current directory only. A search up the parents, stopping at
  the nearest file rather than merging every level so a stray `~/iiac-perf.md` does not apply
  everywhere, with the `Config:` list's `files` line naming what loaded

### setup warns when a project-local [freq] shadows the XDG one

A project-local `[freq]` replaces the XDG one whole, so a run in a directory whose `iiac-perf.md`
declares `[freq]` ignores what `setup` wrote to `~/.config`, and a limit-less local table refuses
every pin there (found 2026-09-14, at `docs: one example config in the md carrier`, with the
3900X's untracked `iiac-perf.md` in exactly that shape). `setup` checks only the XDG file.

- `setup` should check the current directory's project-local file too, and say when its `[freq]`
  shadows the XDG declaration, with the check result for the table that actually applies there
- the refusal from a pin or restore could name the file its `[freq]` came from, which the
  `Config:` list's source tracking already knows how to do for scalar keys

### Measure whether code layout moves the level

A rebuild moved `zcr-mpsc-v1-2t` from 67.9 to 60.8 ns while `min-now` held to 0.01 ns (the evidence
in `feat: CI95 and LSC across processes`), so the binary's layout may set a bench's level as a
process's placement does (wink, 2026-09-12). Fat LTO with one codegen unit, and LLVM function and
block alignment, against the default profile, each built twice around a trivial unrelated change,
interleaved, to see whether layout stops moving the level. Wants that cycle's runs first, so the
placement level is measured rather than confounded.

### How often each pinned level comes up on the 7600x

Fresh pinned processes on the 7600x landed `zcr-mpsc-v1-2t` at 60.6 ns three times and 76.0 once, four
runs too few to say how often each level comes up (the evidence in `feat: CI95 and LSC across
processes`). Twenty unpinned runs there read 63.7 to 64.8 ns with one run at 66.5, twice, 64.4 and
64.3 ns agreeing (wink, 2026-09-15).

- the same command pinned, `zcr-mpsc-v1-2t --runs 20 -d 1 --run-sleep 250ms-750ms --pin-cpus 0,1`,
  twice back to back with `--record`, counting the runs at each level
- the counts are the input "Name what sets a process's level" needs: a level that comes up one run
  in four is a different experiment from one that comes up one in twenty
- first pass, wink, 2026-09-15, 0.28.14-10, the command above with `--pin-freq` at 4701 MHz and no
  `--record`, twice: 72.0 and 71.9 ns, `LSC runs` 0.4 ns each, so the two agree. All 40 runs read
  69.8 to 73.1 ns, no run near the 60.6 or 76.0 ns levels the 0.28.11 build showed. The build
  changed as well as the clock pin, so this points at "Measure whether code layout moves the level"
  as much as at placement
- the pinned spread was the wider one: run stdev 0.6-0.7 ns against 0.3-0.5 unpinned, and the
  outlying runs sat low (69.8, 70.2, 71.0) with `CI95 blocks` of 0.3 ns against 0.1-0.2 for the rest,
  where the unpinned outlier sat high with tight blocks. We think CPU 0, which carries the kernel's
  housekeeping, adds that spread: a pass on `--pin-cpus 2,3` would show it
- the pin-cpus and pin-freq effects are not separated: the pinned level, 72 ns at 4.70 GHz with boost
  off, sits 12% above the unpinned 64.3 ns, and boost's 5.46 GHz ceiling is 16% above the pin. We
  think most of the gap is the clock, but the unpinned runs' delivered clock was not shown, and a pass
  with each pin alone would split them
- second pass, wink, 2026-09-15, the same on `--pin-cpus 2,3`, twice: 70.0 ns (`LSC runs` 0.4) and
  70.4 ns (`LSC runs` 1.0), agreeing by the larger LSC. The bulk sat 2 ns faster than on `0,1` and
  tighter, 69.1 to 70.3 ns with `CI95 blocks` of 0.1 throughout, which fits CPU 0 adding the spread.
  Three runs of 40 sat high with tight blocks, 72.2, 72.9, and 76.2 ns, the last on the 0.28.11
  build's 76.0 ns level, so the level survived the rebuild on these CPUs and came up once in 40

### Allocate runs and duration for a fixed wall time

Twenty short runs or five long ones is a guess today (wink, 2026-09-15, in `feat: CI95 and LSC across
processes`). A bench mean's variance is `(s_p^2 + a/d) / R` for between-process spread `s_p`,
within-run noise `a/d` at run duration `d`, and `R` runs, and a wall time `T` buys
`R = T / (o + d)` runs at a fixed per-run overhead `o`, so the variance at a fixed `T` is least at
`d* = sqrt(a * o / s_p^2)`. `CI95 runs`' t multiplier and the stdev's reliability add a further
lean toward more runs.

- the pinned 7600x `zcr-mpsc-v1-2t` numbers, `a` about 0.01 ns^2 s from `CI95 blocks` 0.2 ns at
  1 s, `s_p` about 0.65 ns, and `o` about 4 s (100 s for 20 one-second runs), put `d*` near 0.3 s.
  At 100 s, 5 x 16 s predicts `CI95 runs` 0.80 ns, 20 x 1 s predicts 0.31, which the runs measured,
  and 23 x 0.3 s predicts 0.29
- the overhead bounds the run count more than the duration does: 4 s of every 5 s per run is the
  settle warm, the warm cap, the run sleep, the block sleeps, and the spawn, so whether a shorter
  settle is safe inside runs is a measurement worth making
- a fixed-budget sweep checks the model: one host and bench, about 100 s an invocation at
  5 x 16 s, 10 x 6 s, 20 x 1 s, and 30 x 0.3 s, each 3-4 times with `--record`, alternating
  configurations, comparing each configuration's `CI95 runs` against the actual scatter of its
  invocations' means
- an allocation hint once the sweep calibrates it: every invocation already knows `a` from the
  blocks, `s_p` from the runs, and `o` from wall time minus measured time, so the summary could
  print the run length and count that would minimize `CI95 runs` in the same wall time
- the model's two weak points: the within-run term is white only where the `resolution` row shows
  no drift, and rare levels make the run means a mixture, whose spread a 20-run invocation samples
  unreliably (the "Mark a run that lands on another level" entry), so the count may need to cover
  the rarest level that matters, not only the variance

### Compare two builds in one invocation

An A/B today is two invocations, and the same binary measured twice, clock pinned, differs by more
than its own `LSC runs`: 383.7 against 387.0 ns on the 3900X 90 minutes apart, and 70.0, 70.4, and
70.6 ns on the 7600x (wink, 2026-09-15, in `feat: CI95 and LSC across processes`, the A/A evidence
in [measuring-a-technique.md](notes/measuring-a-technique.md)). A pair measured in one invocation
with the arms alternating cancels whatever drifts between invocations.

- an arm is a binary and its knobs, so `--against PATH` running that binary's children alternately
  with this one's is the small version, and a config with two arms the general one
- the statistic is the paired difference or ratio and its interval, not two independent means: the
  pairing is what removes the invocation's own offset
- the ratio is what a claim about a technique carries between hosts, so this is the surface a
  cross-host table is built from
- it needs the run's arm in the record beside its `series` and `run`

### Replicate builds so layout is not confounded

A rebuild moved `zcr-mpsc-v1-2t` from 67.9 to 60.8 ns with no code change, 11%, so an A/B of one
build per arm mixes the code change with the layout difference between two binaries, and no number
of runs separates them (wink, 2026-09-15, asking why two builds rather than ten). Kin to "Measure
whether code layout moves the level", which asks whether layout moves it at all, where this asks
how to stop it confounding a comparison.

- k builds per arm, the same source rebuilt with a deliberate layout perturbation, turn layout into
  spread that averages instead of a fixed offset
- the perturbation wants one reproducible knob: a build script emitting a padding static sized by
  an environment variable is the cheapest, and function alignment flags or link order are the
  alternatives
- k follows from the between-build spread, which the layout entry's sweep measures first
- Stabilizer (Curtsinger and Berger, 2013) is the runtime form of the same idea, and the argument
  for why an unrandomized layout makes a measured speedup suspect

### Mark a run that lands on another level

One process in twenty unpinned `zcr-mpsc-v1-2t` runs on the 7600x read 66.5 ns, its blocks agreeing
to 0.1 ns, beside nineteen at 63.7 to 64.8 (wink, 2026-09-15, in `feat: CI95 and LSC across
processes`). That run doubled the invocation's stdev and `CI95 runs`, which is honest for a mixture of
levels, but nothing on the output says the bar is wide because of one run.

- the trimmed pair landed in `feat: CI95 and LSC across processes` and its `trimmed` line names
  the runs it dropped, so a disturbed run is called out already. What remains here is a mark on the
  run line itself, and whether a median belongs beside the mean
- mark a run line whose mean sits beyond some multiple of its own `LSC blocks` from the median run
  mean, a level rather than noise
- or print the median run mean beside `mean`, so a mixture shows as the two disagreeing
- either way the error bars stay over every run, since the level really comes up, and the mark only
  says why the bar is wide
- the second 7600x pass on `--pin-cpus 2,3` makes the case: one invocation drew a 76.2 ns run and a
  72.9 ns run among 69.1 to 70.3, and its stdev read 1.5 ns against the other invocation's 0.6,
  while both invocations' median run mean sat near 70.0 ns. A level that comes up once in 40 is
  missed entirely by 60% of 20-run invocations, (39/40)^20, so two invocations' error bars can
  differ by twice through that alone

### Name what sets a process's level

A fresh process lands a zcr bench on one of a few levels, 60.6 against 76.0 ns pinned on the 7600x
(the evidence in `feat: CI95 and LSC across processes`), and nothing says what decides which (split
from that cycle at its opening). We think it is cache-set aliasing from placement.

- children that map the ring regions themselves and sweep the second ring's page offset against
  the first
- then huge pages, which would remove the offset's effect if aliasing is the mechanism

### Measure core isolation on both hosts

Pinning leaves other work free to share the benched cores (wink, 2026-09-12). An interleaved A/B
on the 3900X and the 7600x: a systemd scope restricting other work to the unbenched CPUs first,
no reboot, then `isolcpus` with `nohz_full`. Any scope or boot change on a host is confirmed with
wink first. Kin to the "Tick-phase avoidance" idea, which isolation strictly dominates when a
reboot is allowed.

### Report the v1/v2 replication to the guide and zc-ring-x1

The pinned v2 two-thread handoff read slower than v1's in both of the report guide's record pairs,
once in the ignored `tmp/mpscv1rows/` (2026-09-11) and `tmp/v2rows/`, both lost 2026-09-14, and
zc-ring-x1 has not been told. 2026-09-08 replicated it and found it host-dependent, three
interleaved runs per cell, mean z4..n2: 3900X pinned 0,1 v1 86 ns and v2 105 ns, 7600x pinned 0,1
v1 60 ns and v2 56 ns, 7600x pinned 0,6 v1 32 ns and v2 35 ns, `0,6` being SMT siblings. Records
once in the ignored `tmp/v1v2-20260908/` here, lost 2026-09-14, and in
`~/iiac-perf-data/v1v2-20260908/` on the 7600x, tagged by pin, with both hosts' demo depth sweeps
beside them.

- the guide calls the gap a two-pair lead and does not cite the replication, a docs change
- a message to zc-ring-x1 with these numbers and the placement levels in `feat: CI95 and LSC across
  processes`, which
  make every single-process zcr comparison suspect. Their Todo already carries the demo's pin-pair
  mismatch, cross-L3 on the 3900X and same-L3 on the 7600x, so the message needs only the numbers
- the placement sweep's records (2026-09-05), 45 runs in the 7600x's
  `~/iiac-perf-data/placement-20260905` and 27 once in the ignored `tmp/placement-20260905` here,
  lost 2026-09-14, are
  unfiled, and [placement-map.md](notes/placement-map.md) is their home when it is refreshed

### A clock row in the report's stats

A run reports its latency numbers but not the frequency the measuring core ran at, so a pinned run's
report does not show the pin holding, and an unpinned one does not show where the clock sat (wink,
2026-09-15, after a restore line's live average read an idle 2.99 GHz after a 3300 MHz pin). The
data exists: every block seam samples the measuring core's delivered clock, the record's
`clock_khz`, and the grade block's settle cell already reads the warmup's.

- a `clock` row after `LSC` in the `mean` .. `LSC` list: the median delivered clock over the
  measured blocks, with its spread, `clock  3.29 GHz  (3.28-3.30 across the blocks)`, and `-` when
  the box exposes no readable clock
- a record key beside it, `clock_median_khz` or a name the record's dictionary settles, so an
  analysis need not recompute it from `clock_khz`
- the report guide's stats section explains the row, and says a pinned run's row should sit on the
  pin, a gap meaning the pin did not hold
- the runs tier landed first, in `feat: CI95 and LSC across processes`: a run line's `clock` column
  is the dominant core's seam clock range over the run, one number within the 1% stability
  tolerance, and the summary's `clock` line the range across the runs, both read back from the
  record's `clock_khz` with no new key. What remains here is the row in a run's own report, where a
  median beside the range would suit, and the record key

### One-way zcr benches, producer-only and burst

Every two-thread zcr bench is a round trip with one message in flight, so nothing here measures
what an ISR-to-thread connection costs: a producer that cannot wait, a consumer that trails a
burst, and a boundary crossed inside a burst once the ring is segmented (wink, 2026-09-08, after
the demo's depth sweep showed v2 winning every streaming placement on both hosts while the round
trip, three interleaved runs per cell, shows it 20% slower than v1 on the 3900X's same-L3 pair and
5% faster on the 7600x's). Two benches over each ring version, shaped by what the field settled on,
with a depth knob that doubles as the segment size once zc-ring-x1's segmented queue exists.

- **producer-only**: the step is one non-waiting reserve plus commit, the `|_| false` closure, a
  worker drains on another CPU with a spin, and Full is counted and printed beside the row, never
  waited on. DPDK's enqueue-burst and full-enqueue costs in one bench, `--inner B` making it a
  burst, and the quantile ladder giving the tail an ISR deadline is measured against
- **burst cost**: the step sends B non-waiting messages then waits for the worker's
  acknowledgement of the last, JCTools' QueueBurstCost, timed first send to last receive, B the
  axis at 1, 8, and 64. B above the depth is the overflow edge whose Full count sizes a segment
  pool, and B above the segment size crosses a boundary inside the burst, the consumer's side of
  it, where JCTools' linked queues lost
- **depth as a run knob**: the zcr rings fix `CAPACITY` at 8 and both benches want depth on the
  line, the demo's finding living on that axis. When a segmented v3 lands the same knob is the
  segment size M, M=1 the boundary's worst case, and the sweep over 1, 2, 4, 8, and 64 the
  amortization check: fit cost against base plus boundary over M, and the M=1 point on or off the
  line says whether that path has a cost of its own, the cold four-line header being the suspect
- the histogram is the second view: at M=64 the boundary is 1.6% of messages and lands in the n2
  band, at M=8 it is 12.5% and lands at p90, so one run at a realistic M shows the boundary cost in
  place
- the paced one-way bench, the Disruptor's latency test with a TSC stamp in the message and a
  consumer-side histogram, answers what latency the thread sees at a given interrupt rate. It
  needs pacing and a consumer-side probe, which the probe-style benches have the bones of, and is
  a second entry once these two land
- names follow the pair convention and are decided at the opening, so the prefix runner covers a
  version's four benches with one word
- ranked ahead of the rest: the numbers feed zc-ring-x1's segmented queue, which may start as a
  v3, and no bench today separates the producer's side of the seam from the consumer's, which the
  cross-host reversal needs. Behind the rename and the partition merge since 2026-09-11, so the
  new benches are written in the merged hierarchy's words

### Analyze a directory of records

A record exists so a re-analysis can happen without the session that produced it, and nothing reads
one back, so every analysis is a throwaway script whose numbers nobody can check (wink, 2026-09-03,
reading the 7600X duration sweep). An `analyze` subcommand over a directory of records, sharing
`record.rs`'s struct so the schema keeps one owner.

- three tiers, in increasing order of what they are worth:
  - **tabulate**: pivot the records into a table on an axis, duration, host, bench, or run. The
    least interesting tier, and the one the other two are built on
  - **read the series nothing reads**: `block_mean_ns`, `block_samples`, `clock_t_ns`, `clock_cpu`,
    and `clock_khz` sit in every record and nothing reads them back, roughly a third of the file's
    bytes as dead weight. The drift and step signals are computed at run time and thrown away, and
    the record holds the raw material to recompute them
  - **make a cross-run claim**: the tool cannot make one at all today. Every number it prints is a
    within-invocation claim, and the guide already says so, treating `LSC` as a lower bound and
    telling the reader to run 3-5 interleaved and compare the per-run values by hand ([Comparing
    two implementations](docs/report-guide.md#comparing-two-implementations)). Nothing performs
    that comparison
- the cross-run tier is the entry's point: run-to-run scatter, a confidence interval on the mean of
  run means, and an `LSC` that is not fiction. Single-run resolution understates the real spread
  badly on the contended benches, `cb-chan-2t`'s 5 s and 30 s runs disagreeing by 10.9% while the
  5 s run claims 0.15% resolution, 64x its own claim
- one run per cell cannot say which of the two runs was the off one, so the sweep that found this
  was the wrong shape. 3-5 runs per cell, interleaved, is what the guide has said all along
- the output reuses the report's row names, so the guide decodes the new surface for free. Grading
  the set the way a run grades itself is the natural extension: do these runs agree, and is a
  disagreement drift, a step, or one bad run
- `--format csv` / `--format json` for the plotting hand-off, kin to "Machine-readable report
  output" below, one flag family
- its cross-run arithmetic is `feat: CI95 and LSC across processes`'s, one statistics module
  serving both, and the records carry the host block cross-host analysis needs, a hostname alone
  naming nothing

### A --pin-idle knob, forbidding deep C-states

Pinning cores and pinning frequency both leave the package free to sink into deep idle when only a
couple of cores are busy, and on the 7600x that costs 18% on a cross-core round trip (2026-09-04).
"Cold-wake profile" below already names the lever, the `/dev/cpu_dma_latency` clamp, as a pin-idle
sibling to pin-freq.

- the evidence is the `suggest-freq` entry in [bugs.md](notes/bugs.md): a 20 Hz sysfs sampler
  waking an otherwise idle core recovers the whole 18%, so the droop is real and cheap to defeat
- measured with `zcr-mpsc-2t --pin-cpus 1,2 --pin-freq=4701 -d 60`, three interleaved reps each,
  every run graded A: 74.3 ns without the waker against 62.7 ns with it
- with the clamp held a pinned run should reach 62.7 and need no sampler, which is also how the
  bug's fix gets validated
- it reframes this box's run-to-run effects. The first-bench-of-a-process gap, the cold-against-
  warm-box difference, and the 11% drift over ninety minutes are all idle-state stories, and
  neither `--pin-cpus` nor `--pin-freq` touches any of them
- shape: a guard like `RunPin`, holding an open fd on `/dev/cpu_dma_latency` with a zero written
  to it for the run's life, released on drop, and named in the Setup banner beside the freq pin

### Vyukov's unbounded SPSC

The node-based unbounded SPSC from 1024cores.net, the second implementation the crossbeam
baselines exist to frame (wink, 2026-08-28), zc-ring-x1's SPSC v1 being the first, measured by the
`zcr-spsc-v1` pair.

- producer-side node recycling (`head`, the free-list `first`, the cached `tail_copy`, and the
  shared `tail`) so the steady state never calls the allocator. No crate is that algorithm, so it
  is unsafe code we would own and maintain inside a measurement tool, which is the real cost of the
  entry
- the interesting axis falls out of the designs rather than being invented: Vyukov's avoids the
  allocator by recycling nodes and zc-ring-x1's by drawing segments from a Pool, so each has a
  cold path that allocates and a steady path that does not. The block and warmup knobs already
  separate those, so the honest report is two numbers per queue

### A completion hook that checks itself

The shell hook is one rc-file line per binary name, `source <(COMPLETE=bash iiac-perf)`, and
a second one for the dev name (wink, 2026-09-02). Someday the app does what is necessary on any
run: notice its own hook is missing or stale for the shell it runs in, and say what to run, so
the lines are never typed by hand.

### A cb-chan-2t-spin twin

`cb-chan-2t` parks like `mpsc-2t`, and crossbeam's `recv` spins briefly before parking, so its
band table is bimodal and it grades F on interference in every run (2026-09-02, the report
guide says why). A `try_recv` twin, the peer of `mpsc-2t-spin`, would give the channel one clean
spinning number beside `cb-seg-2t` and the zcr 2t rows.

### Cold-wake profile

Measure the post-wake transient (C-state exit, cache and TLB refill, the clock ramp) instead of
discarding it. A real consumer of an MPSC ring blocks and wakes cold where the spin-wait
benches stay maximally hot, so the cold-wake cost is arguably the number a deployment feels
(wink, 2026-08-19).

- sleep does not flush caches by itself: a long enough idle lets the core enter deep
  C-states, which power-gate the core and lose the private caches, so sleep duration is the
  dial for how cold a wake starts. Unpinned, the wake may also migrate cores, so the cold
  axis wants sweeping with `--pin-cpus`
- the raw instrument lands with the measure-reproducibility cycle's "block sleep and warmup
  become knobs" rung: `--block-sleep` reaches seconds and selects the depth of cold, and
  `--block-warmup 0` records from the first post-wake call, so cold samples already show up
  as a band shoulder
- measured (wink, 2026-08-20, 7600x, `min-now --blocks 100 --block-sleep 1s`): the shoulder
  is real, an n3 band at 24.3-24.8 ns (~1,070 samples per wake) over an 18.1 ns body, and it
  slips under the interference census's 1.5x threshold, so the bands show what the census
  cannot. The same run moved the whole body 16.2 -> 18.1 ns at grade A throughout: the 5%
  duty cycle selects a lower clock state (we think ~4.87 GHz against the sustained 5.44,
  from the ratio), so seconds-scale sleeps are a state-selection probe as much as a cold
  probe, and A/B runs must match their block knobs
- the deliverable is the time-ordered decay after each wake: record the first K per-call
  values post-wake verbatim (the clock-journey move applied to latency), reported as a
  per-block decay profile, never folded into the steady stats, since cold samples would
  contaminate block means, CI95, and the resolution claim
- same family, opposite sign: reproducibility runs may want deep C-states forbidden (the
  `/dev/cpu_dma_latency` clamp, a pin-idle sibling to pin-freq), while this mode wants them
  allowed

### Rethink environment rating

The settle-cell rework improved the grades but wink's verdict is "better but not good enough"
(2026-08-19), so the environment-rating design gets its own discussion and likely redesign, and
the qualify power-policy rung, halted unbuilt the same day, resumes on the result with its spec
preserved here.

- the day's findings to reason from: the settle share is now a graded signal folded into the
  warmup worst, unverifiable claims fail rather than grade well, and the remaining
  questions include whether `not settled` should disqualify outright, whether qualify's
  children should pin, and whether the timing letters and the clock story should compose
  differently
- the halted rung's spec: qualify names the policy before spending minutes on numbers it
  can predict will scatter. A diagnosis only: the mutation lives in the frequency commands,
  and qualify never changes the box
- the caution is earned: the 2026-08-03 session's documented revert left EPP at
  `performance` after the governor had already returned to `powersave`, which is also why
  restore converges to a declared steady state instead of a remembered one
- independent piece, landable any time as a small fix: the qualify-only flags stop being
  silently ignored (wink, 2026-08-18). A bench run given `--runs`, `--gap`, or
  `--print-only` errors naming the qualify word instead of quietly doing something other
  than what the flags asked. Caught live on the 7600x: `min-now --gap 2 --runs 5` ran one
  5 s bench run, not five gapped runs, and nothing said so. clap cannot express "only
  beside this command word" for defaulted flags, so the guard is a main-side check that the
  flag was given at all (clap's `ArgMatches` value source, not a default-value comparison,
  so an explicit `--runs 10` also errors)

### Config keys stay CLI-settable

Adopt the convention that every run-parameter config key has a CLI flag with CLI-wins
precedence, the flag landing in the same commit as the key, so any experiment is runnable
one-shot without editing a file (wink, 2026-08-17).

- already true today: duration, band_labels, decimals, settle_time, warm_cap, and the pin
  target all pair a key with a flag, and a `--pin` profile only names a core spec that also
  passes raw, so the convention mostly writes down existing practice
- the deliberate exclusion: the `[freq]` steady state (governor, epp, boost, min_mhz,
  max_mhz) stays file-only. It is the declared way home, and a per-invocation override is the
  2026-08-03 failure shape, a transient intent outliving its session
- deferred alternative: a generic repeatable `--set key=value` overlaying as a top config
  layer would guarantee the property structurally for future keys, at the cost of clap's
  typed parsing and `--help` discoverability, and it would have to refuse the declaration
  keys. Revisit if a key ever shows up where a dedicated flag feels heavy

### Prepare for expected errors

Before a command whose outcome is not clean (a rebase across diverged lines, a force-push,
dogfooding a dev tool), state the expected output, what unexpected would look like, and the
abort path, then fix stepwise with the user.

- the forward-looking half of stop-and-ask, and family-shaped, so it belongs in the set's
  working practices via its own convention cycle
- born 2026-08-14: a rebase's predicted conflicts arrived unannounced and read as breakage
  (wink stopped the session), the prediction living in a record instead of in the moment

### Always work on a topic bookmark

Cycles happen on a bookmark, `main` advances only by landing a reviewed bookmark, never by
direct push (adopted in principle 2026-08-01, and now the set's own rule).

- buys free pre-landing rewrites: the 2026-08-01 renumber needed a coordinated force-push
  only because cycles push `main` directly
- the retired cycle-protocol.md already anticipated the shape: topic-branch chores sections
  defer SHA backfill until the branch lands on the permanent branch
- the rules are the set's as of the adoption:
  [Cycles run on a bookmark](AGENTS.md#cycles-run-on-a-bookmark), and `jj.md`'s
  [Cycle bookmarks](agent-data/jj.md#cycle-bookmarks-create-and-land). What is left is the
  habit and vc-x1's review
- tooling: `vc-x1 push <bookmark>` already takes any bookmark. Landing is two jj commands and
  wants a `vc-x1 start-change <bookmark>` for the create half eventually (wink)
- one process detail is now settled (2026-08-05): a bookmark is a draft until it lands, so its
  ladder stays self-consistent and may be rewritten and force-pushed while unlanded, per
  [Cycle shape](AGENTS.md#cycle-shape)

### Sync the 20260803 agent-files baseline

Superseded in substance by the `docs: adopt the family agent-files set` cycle, kept until its
close-out confirms nothing below is still owed [[84]].

- blocked on vc-x1 fixing the payload first: its `custom.md` step number is stale against its
  own checklist, and `jj.md`'s range bullets are wrong, so syncing today propagates both
- the sync renames `agent-data/cycle.md` to `cycle-checklists.md` and moves
  `cycle-protocol.md` and `versioning.md` from `notes/` into `agent-data/`: 28 inbound
  references to re-point across 9 files
- the `custom.md` half is done (2026-08-07): the conventions moved into the pinned files
  rather than waiting for the sync, since the pinned copy is where the family reviews them,
  and everything of this project's own moved to `custom-family.md`. `custom.md` is now the
  payload stub plus one pointer line
- remaining risk is textual, not conceptual: our moved rules land in files the sync then
  renames or relocates, so the sync has to merge rather than overwrite

### Qualification reports evidence, not verdicts

Retire the prejudging NOT QUALIFIED stamp (wink, 2026-08-02) in favor of measured statements a
reader judges.

- blocks-based: A/A repeatability (does a same-code delta clear LSC?), CI95/LSC as the
  published sensitivity ("this box resolves X ns on this bench"), stratification by state
  instead of a blended letter
- the 3900X reads NOT QUALIFIED today for mid-run bistable transitions warmup cannot
  prevent: a trait to report, not a disqualification. The 7600x dwell case that motivated
  the gate is fixed by the dynamic-warmup cycle
- entangled with "Qualify the environment without a bench" (below) and machine-readable
  output, and it wants the blocks-knobs entry (above) landed first

### Seam-clock attribution

Sample `cpuinfo_avg_freq` at block seams (the reader exists, `src/freq.rs`) so a mid-run step
gets a "clock moved" label, the way warmup now separates a dwell from the top. Also the natural
home for surfacing the clock ratio in normal output as one coherent story (chores-06: the 3900X
flip at ~2-4 s is almost certainly a visible clock move).

- the sampling half moved into the measure-reproducibility cycle's record rung (2026-08-16):
  seam samples join the record, per-block/per-run frequency stats fall out, and the pin
  verifies itself. What stays here is the report surface, the "clock moved" label
- the ~2-4 s flip is no longer a guess: 0.26.0-1's settle state named the states directly,
  4.09 GHz entry and ~4.53 GHz top on the 3900x under today's policy

### Qualify the environment without a bench

`qualify-environment` respawns children running `min-now`, but every number in its table comes
from the micro-probe series, which never touches the bench. The bench is there only to give the
warm something to do and to produce a report to parse, so the selftest inherits a workload's
character it does not want, and the parent parses prose (see the machine-readable-output entry
below, which this would make moot for the selftest).

- measure the probe series directly: warm and probe with no bench registered, grade the
  stretches, done. The `mean` column becomes the probe's own floor, which is the quantity the
  grade is computed from rather than a second measurement of nearly the same thing
- **the warm's character is the open question.** A probe-driven warm is light. On hardware
  where a heavy workload drives a different clock/power state (AVX offsets), a light warm
  would qualify the box for work it will not do. Moot on the 3900X and 7600x, where `min-now`
  *is* essentially the probe, so decide it with a measurement on a box where it isn't
- **respawn or loop** is a second question, not this one: respawning resets process-local
  state (address space, caches, allocator) and loops do not, but neither resets the machine's
  P-state. What re-rolls that is the gap and the duty cycle. If the answer is loop, the
  results stay structured data and never become text
- coordinate with the "Dynamic warmup" Todo, which owns the convergence rule this would warm
  by, and with the grade-block columns entry, which reformats the table this prints [[75]]

### Move qualify-environment onto the child runner

`qualify-environment` spawns its children with its own loop and parses their report text, while
`feat: CI95 and LSC across processes` gives every bench a child runner that reads each child's
record back (split from that cycle at its opening, whose Todo entry had subsumed this). One runner
serves both.

- the children's results come back as records, so the prose parsing of the `env warmup`, `env
  bench`, and `mean` rows goes, and "Qualify the environment without a bench" above decides what a
  child measures
- the orchestration in `tests/qualify_environment.rs` reduces to asserting on the verdict, the
  rest of the "Stability selftest mode" idea in `## Ideas`

### Guard undersized pin pools and deadline the estimate phase

Guard `--pin` pools smaller than the bench's thread placements: `zcr-mpsc-2t --pin 8` put both
spinning software threads on one logical CPU and appeared hung until ^C (2026-07-26, bug #1 in
[bugs.md](notes/bugs.md#bugs)).

- track `core_for` requests in `RunCfg` (max `thread_idx` asked for), and refuse the run when
  placements exceed unique CPUs in the pool. Placement only goes through `core_for` when
  pinning is active, so the guard covers every path, and no pinning means the scheduler
  separates the spinners itself
- wall-clock deadline on the open-loop 5x1,000-step estimate phase so *any* pathologically
  slow bench aborts with a diagnostic naming per-step cost and pinning, instead of hanging

### Move the block seam's work off the measuring thread

Use the FastForward-style SPSC ring. The block flush stops the bench for ~1-2 ms every block, about
50 ms at the default of 100 blocks over 5 s, so ~2-4% of a run is spent at seams. The 1-2 ms was
measured when the flush sorted a batch, and the pipeline now drains a 65,536-sample stage, so it
wants re-measuring first. Hand the filled buffer to a consumer thread that sorts,
summarizes and records while the producer fills a second one. The seam drops to a pointer swap.

- the payload is one word, a buffer offset, the exact shape `ffq` is built for, and the
  project dogfooding the queue it benchmarks
- double-buffered: at ~1-2 ms of work per 50 ms block the consumer runs ~30x faster than it
  needs to, so two buffers never back up
- honest cost: the consumer's cross-core traffic runs *during* measurement, trading a gap on
  the hot core for background L3 pressure. Measure it the way the -4 seam probe was measured
  (interleaved A/B, pinned, trimmed mean) rather than assuming
- blocked on the ring existing. See the "FastForward-style SPSC ring" entry, currently on the
  `ffq-spsc-notes` bookmark rather than `main`

### Sweep "box" to "host"

The project has two words for one thing. The record field is `host`, the host identity cycle
builds on it, and the prose says "box" about a hundred times, so a reader meeting both is
left wondering whether they name different things (wink, 2026-09-03).

- the count, `\bbox\b`: `docs/report-guide.md` 23, `docs/config.md` 9, `docs/usage.md` 9,
  `src/freqctl.rs` 30, `src/freq.rs` 12, `src/config.rs` 9, `src/qualify.rs` 9, `src/inhibit.rs` 1.
  `README.md` has none, so the front door introduces neither word while the rest of the docs lean
  on one of them constantly
- `host` wins because it is already the schema's word and standard outside this project, while
  "box" is sysadmin colloquial and defined nowhere
- the testing vocabulary does not fit and is worth recording so it is not proposed again: DUT,
  UUT, and SUT all name the thing under test, and here that is the bench, with the machine as the
  environment it runs in
- ranked after the host identity cycle, whose host block is what makes `host` the
  obviously load-bearing word
- scope is prose and doc comments. Published commit bodies keep the wording they shipped with
- the cheap alternative, if the sweep is judged not worth it: one README line defining "box" and
  saying it is the record's `host`

### Tighten thread and CPU terminology

Across docs and doc comments: "software thread" for what `thread::spawn` makes, "logical CPU"
(hardware thread) for what `--pin` selects and the OS schedules onto, "physical core" for the
engine SMT siblings share. Bare "core"/"CPU"/"thread" only where context disambiguates.

- spin-wait bench docs state the precondition: each spinning software thread needs its own
  logical CPU
- `--pin` help/README say slots are logical CPU ids

### Topology-aware pinning and lCPU terminology

Discover the CPU sharing tree at runtime and describe every pin by the nearest shared level,
not "unique CPUs". Evidence: the 2026-08-01 pinning experiment (`zcr-with-2t -d 30 --blocks 5`
on the 3900X, boost on) measured the round trip at ~35 ns on SMT siblings (shared L1/L2),
~133 ns same-CCX (shared L3), ~633 ns cross-CCX (shared fabric only). Cross-CCX vs cross-CCD
differed by 1.6 ns against a ~2 ns LSC, so the L3 boundary is the only fabric tier that matters
on Zen 2, and the unpinned scheduler's ~127-135 ns core mass matches same-CCX placement.

- standardize terms by shared resource, vendor structures as examples only: **lCPU**
  (kernel-schedulable execution context, the `--pin` unit), **core** (lCPUs sharing L1 and
  the execution engine), **cluster** (cores sharing a mid-level cache: Intel E-core module,
  ARM DynamIQ, absent on AMD), **LLC domain** (cores sharing last-level cache: AMD CCX),
  **package** (LLC domains sharing on-package fabric), **NUMA node**. Levels a machine lacks
  collapse out. The tree may be asymmetric (hybrid parts have levels only on some branches), and
  the levels match the kernel sched-domain ladder SMT/CLS/MC/PKG/NUMA
- core *type* (big.LITTLE, P/E cores) is an attribute of a core, not a level: cluster
  identical (part id, capacity, max freq) cores into classes and report the classes. Read
  `cpu_capacity` (ARM/RISC-V arch_topology), `/sys/devices/cpu_core/cpus` +
  `/sys/devices/cpu_atom/cpus` (Intel hybrid), part id + `cpuinfo_max_freq` as fallback. Avoid
  the big/LITTLE branding (DynamIQ ships 3-4 tiers)
- discovery is unprivileged sysfs: partition lCPUs by `cache/index*/shared_cpu_list` per
  cache level, plus `topology/{thread_siblings,cluster_cpus}_list`, `physical_package_id`,
  `/sys/devices/system/node`. Cacheinfo is populated on x86_64 and arm64, patchy on RISC-V,
  so fall back to topology files and mark cache levels unknown
- the Setup `bench pin` line reports the pool's partition and nearest shared level, e.g.
  `[0, 12] (2 slots, 2 lCPUs on 1 core - shared L1/L2)`, and retires bare "CPU" from all output
- auto profiles derived from the discovered tree (`--pin smt`, `--pin llc`, `--pin xllc`) so
  one command line is portable across boxes, and extends the config `[profiles]` mechanism
  `--pin` already resolves
- **placement tracking** (added 2026-08-01): when unpinned, placement is the dominant factor
  (4-18x on the 3900X) but is currently invisible. Observe it instead of only controlling it.
  Two tiers of knowledge, and the report says which one a claim comes from:
  - cooperative (exact): threads placed through the `--pin` pool are known
  - observational (complete but sampled): a bench need not announce threads or
    sub-processes, and the kernel tells us anyway: sweep `/proc/self/task/` at block seams
    (last-ran lCPU is `stat` field 39, children via `/children`, recursively). CPU-time
    deltas between sweeps identify the active threads with no cooperation, and `sched_getcpu`
    (vDSO-cheap) covers the measuring thread exactly. Sampled truth: migrations inside a
    block and threads born and dead between seams are unseen, which matches the step
    detector's block granularity, and cost is ~us per seam against a 1-2 ms seam
  - blocks gain a placement-class label, so a placement migration becomes an *attributed*
    step ("cross-LLC -> same-core"), the way the env grade attributes DVFS
  - unpinned `--blocks` runs stratify block stats by placement class instead of one smeared
    CI: the scheduler's wandering becomes a free stratified experiment (how the SMT fast
    mode was found)
- subsumes the vocabulary half of "Tighten thread and CPU terminology" (above): keep its
  software-thread vs lCPU distinction, adopt lCPU as the standard term

### Windows and macOS port considerations

iiac-perf runs on Linux alone, and a port was named as a Todo on 2026-09-04 without its content
being written down. What is Linux-only today, as a start: CPU pinning through
`sched_setaffinity`, the cpufreq sysfs files behind `--pin-freq`, `read-freq`, and the delivered
clock, `/dev/cpu_dma_latency` for the pin-idle clamp, the host block's `/proc` and `/sys` reads,
the udev rule the setup subcommand would write, and the inhibit guard. Each wants its platform
equivalent or an honest "not measured here" on the report.

- the seam and the per-platform notes are in
  [measuring-a-technique.md](notes/measuring-a-technique.md): a portable core, the timing loop,
  blocks, histogram, statistics, record, and report, under an environment layer that is Linux-shaped
  today. macOS is the awkward one, with affinity hints at best and no user-level clock control, and
  a bare-metal target has no processes at all, so re-rolling placement there means randomizing
  allocation offsets inside the program rather than spawning a child

### Rebase web-claude-tweaks onto post-0.22.0 main

It rewrites an already-published bookmark (needs approval) and its arbitrary `0.21.0-b`
version needs replacing, owed from the 0.22.0 close-out plan.

### Unit scaling in report columns

`us`/`ms`: per-row auto-scale so columns stay eyeball-comparable (bands are monotonic, so a
row's first/last/mean share a magnitude), or `--units ns|auto` for script-stable output. Needs
`--decimals` landed first (`3.18 ms` vs `3 ms`). Candidate `-4` for the report-options cycle.

### Drift and clock plots in the terminal

Every run records a block-mean series and a delivered-clock series and reports them as a grade
letter, so "did this run drift" is answered by a letter with no picture behind it (wink,
2026-09-03). Braille or block characters drawn in the terminal, no plotting dependency.

- two plots sharing one time axis: block means, where the drift and step signals come from, and the
  delivered clock beside it, so a body that moved can be read against a clock that moved
- no plotting crate. The tool's whole value is that its numbers can be trusted, and a plotting
  stack is dependency surface that can move between measurements for reasons having nothing to do
  with measurement. Characters cost nothing and never move
- lands on both surfaces, the live report and the per-record view of `analyze`, so it ranks after
  "Analyze a directory of records" above
- real image output stays out. `analyze --format csv` hands tidy data to gnuplot or matplotlib,
  which also makes the picture reproducible by anyone holding the records, and the drawing is not
  the measurement tool's job
- decide what the picture should show after `analyze` has seen real use. We think the first real
  use names a plot nobody predicted

### Machine-readable report output

`--format json`, or key=value lines to stay dependency-light. Design once the batch gauge lands
(0.23.0-4) so the schema covers the surviving surface: report stats, gauge signals, letter.
Consumers: `tests/qualify_environment.rs` (drops its brittle-but-loud line parsing),
placement-map validation runs, cross-run comparison scripts. Kin to the unit-scaling entry's
`--units ns` script-stable concern (above), one flag family.

### Trimmed core stats

`mean/stdev p10-p90` report row, additional to (never replacing) `mean` / `mean min-p99`. Trim
bounds possibly configurable (`--trim p10:p90`?). Why: the full mean wobbles ~±1.4% with the
run's mode mix while the core plateau is ~±0.2% stable, so the trimmed row is the run-to-run
comparable number. Boundary sensitivity (see [[57]]): window edges in the mode-mix smear
inherit its wobble (p50-p60 ±0.05% vs p40-p50 ~1%), so also consider a dominant-*mode*
statistic (peak-density region, bottom-count-independent) [[57]]

- the runs summary is a natural first user (wink, 2026-09-15, in `feat: CI95 and LSC across
  processes`): it averages each run's full mean, since `mean z4..n2` takes its bounds from the
  bands a run populated, so two runs' trimmed means can cover different spans. A fixed-quantile
  trimmed mean in the record would give `CI95 runs` a statistic that ignores interference spikes

### Find and label the interference crossover

The band where the tail stops measuring the code and starts measuring the machine. Not to hide
it: to *name* it, because that is the signal TProbe exists to surface (the OS swapping, a drive
stalling, anything not caused by the code under test).

- Locate it from the data rather than fixing it at a percentile: the giveaway is the band
  `range` exploding (min-now 0.21.0, 3900X: `n3` range 3.0 ns -> `n4` range 200.4 ns), not
  a chosen p99.
- The crossover moves with the bench. A counting argument places it: interference arrives
  at a *rate*, so it can only contaminate so many samples. That run's `n2` held 838,635 of
  8,059,469 samples over 5 s = ~168,000/s, and nothing in the OS runs at that rate, so `n2`
  is code. The `n4`+ bands total ~1,500/s, timer-interrupt territory.
- So report the above-crossover count as an **interference rate**, and consider surfacing
  whether the run was quiet enough to trust. Calibration wants exactly this signal (see
  [[61]]). A contaminated run is currently only detectable by squinting at band ranges.
- Superseded pointer: the 0.22.0-5 calibration-time grade certified the ~1 s window before
  the run.
  [Replanning II](notes/chores/chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run)
  moves grading onto the run itself. This entry's crossover and rate analysis is absorbed
  by Todo #1's batch design, which supplies the time axis the histogram lacks.
- Pairs with the trimmed-core-stats entry above: that one needs a defensible upper bound,
  and this is how to find one per run instead of hardcoding p99.

### Investigate the suspend gap missing from samples

A 0.13.5 `--no-inhibit` suspend test detected ~1.2 s suspended inside the measured window but
the max sample was only 4.0 ms, while the 0.13.1 test (8.4 s gap) showed the expected 10.4 s
max sample. We think minstant's TSC may halt across some suspends and count through others.
Repeat the test comparing detected gap vs max sample. If the TSC halts, per-sample timing
silently loses suspend time. Document either way.

### CLAUDE.md governance model

Design cogitation.

### Revisit probe adjustment

Under the in-interval vs call-to-call split: probes take one call per sample (inner=1), so the
in-interval timer slice is unamortized and unmeasurable, so an `adjusted` column can subtract
nothing defensible, so maybe state a bound instead
[analysis](notes/design.md#timer-overhead-in-interval-vs-call-to-call).

### Convert harness and Bench to probe-based measurement

Will likely need inner-loop support on `Probe` (N calls per sample, report divides by N
and accounts for per-sample framing) so very-small workloads can still amortize timer overhead
the way `run_adaptive` does today.

### Rename app

### Design an app to measure IIAC performance

Written in Rust.

### ice-ps-2t-wait

iceoryx2 pub/sub with blocking waits via `Listener`/`Notifier` events, completing the
{transport} × {wait policy} matrix cell that compares against `mpsc-2t`.

### Switch ice benches to the loan-based zero-copy send path

`loan_uninit` + `send`, the API a perf-sensitive user would use, and closer to iceoryx2's own
benchmark method.

### Payload-size sweep for the round-trip benches

8 B / 8 KiB / 1 MiB, makes iceoryx2's size-independent latency vs channel copy cost visible in
our own tables.

### tokio-mpsc benches

`tokio-mpsc-1t` / `tokio-mpsc-2t`: `tokio::sync::mpsc` round-trip inside a Tokio runtime
(async overhead).

### flume benches

`flume-1t` / `flume-2t`: `flume` MPMC channel.

### Function-call baselines

Direct call vs `Box<dyn Trait>` vs `async fn` (poll-once): anchors the channel/serde numbers
against the cheapest possible "send a value then receive it" path.

### Extract shared round-trip helpers

When the second channel impl lands, extract shared message types + round-trip helpers into
`src/benches/common.rs` (deferred from 0.2.0).

### Additional thread control

Count, per-thread pin lists, NUMA: shape once a concrete bench needs it.

### Rename crate

`iiac-perf` -> general-purpose name (breaking, deferred).

## Ideas

Longer-range thoughts, not yet ranked work. `-` bullets, no numbering. Promote into `## Todo`
when one becomes actionable.

- Per-bench dependency isolation, motivated by dep provenance: the deps are the thing being
  measured, so a dep bump (e.g. iceoryx2 0.9.2 -> 0.9.3) legitimately moves that bench's
  numbers and shouldn't ride in silently. Options considered (2026-07-08):
  - Caveat first: a Cargo **workspace shares one Cargo.lock** across members. It scopes deps
    per package (ice benches alone pay for iceoryx2, faster `-p` builds, and harness/probes become
    a library crate) but does *not* give per-bench lock isolation, and it splits the single CLI
    into many binaries.
  - Targeted updates (`cargo update -p <crate>`, never bare `cargo update`): ~90% of the
    provenance benefit at zero structure cost, and adoptable immediately as discipline.
  - Feature gates (`--features ice`): solves build weight in the current single package, not
    lock isolation.
  - Truly standalone crates (own Cargo.lock each): the only real per-bench dep isolation, at
    maximum maintenance, and it cuts against "same harness, same build" A/B comparability.
  - Current lean: targeted-update discipline now, with feature gates or workspace only when bench
    families multiply.
- clap CompleteEnv dynamic completion (the `unstable-dynamic` feature): clap's native runtime
  completer (`COMPLETE=bash iiac-perf`) would give bash die-hards a compact column view without
  carapace. Revisit if clap stabilizes it.
- Stability selftest mode (2026-07-27): grade the environment more thoroughly than a single
  run's gauge: a product subcommand that respawns its own binary (`current_exe()`) N times at
  configurable cadences and reports cross-run gauge agreement ("is this box currently
  trustworthy for A/B?"). Precedent in-product: the calibration repeat self-check and
  `--blocks` both already validate by orchestrated repetition. This is the next ring out.
  Subsumes `tests/qualify_environment.rs`'s orchestration: the test reduces to asserting on the
  verdict, and its env-var knobs become clap flags. Concrete motivation (2026-07-27): the
  qualification test can't run on the 7600x, which has only the installed binary, and
  environment qualification shouldn't require a source tree. **Promoted 2026-07-28**: the
  minimal version is the 0.23.0-6 ladder rung (`qualify-environment subcommand`). What remains
  here for later is the fuller mode: cadence sweeps, richer cross-run reporting.
- Cold-start mode (2026-08-02): blocks deliberately shields the coldest samples (the 2 ms
  post-wake warm is unrecorded), so the true first-call-after-sleep cost never lands in the
  histogram. A mode that records or separately reports post-wake samples would measure the wake
  cost applications actually pay ("--blocks 1000 feels more real", taken one step further)
- Tick-phase avoidance (2026-07-27): the scheduler tick is periodic per-CPU (~300/s at
  CONFIG_HZ=300) and a tick hit is an unmistakable outlier, so predict the next tick from
  detected hits and pause measuring ~30 us around it, at ~1% duty cost, no governor exposure
  with governor+EPP `performance`. Doesn't improve the bulk stats (tick hits are already
  detected and trimmed), and buys a cleaner above-crossover tail on unmodified machines, so rarer
  aperiodic events (device IRQs, SMIs, code slow paths) become visible over the periodic
  contaminant. Check the interaction with dither (anti-phase scheduling must not introduce a
  systematic phase bias), and compare against `nohz_full`/`isolcpus` isolation (which abolishes
  the tick on a dedicated core and strictly dominates when a reboot is allowed) before
  building.

## Bugs

_See [bugs.md](notes/bugs.md)._

## Closed

The last cycle's finished record, moved here whole by its closing commit and deleted by the next
opening ([Cycle-record](AGENTS.md#cycle-record)). Earlier cycles are in the landmark commit's
copy of this section, and the cycles before the rule in the frozen [notes/chores/](notes/chores)
and [notes/done.md](notes/done.md).

### feat: CI95 and LSC across processes

#### Problem

A process start re-rolls where the rings and stacks land in memory, and that placement sets a
bench's level, so a run's CI95 and LSC, computed over blocks inside one process, are lower bounds
that can miss the real spread by a wide margin (wink, 2026-09-05, confirmed 2026-09-12).

- **the evidence**, the 7600x on 2026-09-12 (UTC 2026-09-13), `zcr-mpsc-v1-2t -d 5`, 100 blocks,
  1-10 ms sleep, config isolated. Records in the 7600x's `~/iiac-perf-data/warmup-20260913/`, a copy
  and its `analyze.py` once in this repo's ignored `tmp/warmup-7600x-20260913/`, lost
  2026-09-14 (the ops notes' kept-records bullet):
  - one process running the bench four times, three processes: every process read run 1 at 60.4 to
    60.6 ns, run 2 at 62.6 to 63.0, run 3 at 71.7 to 71.9, and run 4 at 63.4 or 71.7, each run
    claiming CI95 under 0.1 ns. The plain 0.28.10 showed its own run-indexed levels, 59.8 to 68.1 ns
  - pinned 0,1, fresh processes read 60.6 to 60.7 ns three times and 76.0 once, and unpinned 63.4
    to 63.6 ns four times
  - the plain 0.28.10 against the dev 0.28.11, pinned: 67.9 against 60.8 ns on the zcr bench and
    16.36 against 16.35 ns on `min-now`, so the harness measures alike and the gap is the binary's
    placement level
  - block warmup 0, 2, and 10 ms, pinned 0,1, three interleaved each: means 60.58, 60.45, and 60.62
    ns, no effect, 10 ms costing 0.9 s a run
- **the earlier evidence**, the 7600x on 2026-09-05, `zcr-spsc-v1-2t --inner 100 --blocks 10
  --pin-cpus 2,8`: five invocations with 1 s block sleeps and 100 ms warmups each held their ten
  blocks within 0.1 ns, and the invocations landed on two levels 0.15 ns apart, seven times the
  spread the blocks predicted. CPUs N and N+6 are SMT siblings on the 7600x, so `2,8` was a
  one-core run

#### Solution

iiac-perf finds a bench's CI95 and LSC itself: every bench of the list runs in its own child
process, `runs` times, back to back, and the error bars are computed over the process means beside
the within-process ones.

- **the bench list as a setting**: the positional `BENCHES`, `--benches`, and a `benches` config
  key, so a config file can name what runs
- **one bench per process**: the parent respawns `current_exe()` once per run, as
  `qualify-environment` does, and starts the sleep inhibit and the clock pin once for all of them.
  It is inert while a child runs, the `suggest-freq` sampler bug in [bugs.md](notes/bugs.md) being
  the warning. A child gets `--pin-cpus` and every run knob, runs one bench, and hands its results
  back as a record
- **replication**: `--runs N` and a `runs` config key, default 5, and `--run-sleep` with a
  `run_sleep` key, a time or a random range between runs, replacing `qualify-environment`'s `--gap`.
  A child needs a second or two, since blocks within a process agree to 0.1%
- **one statistics owner**: CI95 and LSC over a series of means is one module, fed block means
  within a process and process means across them, which the "Analyze a directory of records" entry
  reuses
- **the output**: a line per run as each child finishes, then the bench's summary with both tiers.
  `-v` shows each child's full report
- **the labels**: "block" names the within-process replicate and "run" the between-process one, so
  the rows read `CI95 blocks`, `LSC blocks`, `CI95 runs`, and `LSC runs`, their ratio saying
  whether per-process state dominates, and a record carries a series id grouping one invocation's
  children
- **what the run lines show**, from wink's readings on both hosts: each run's mean, the stdev of
  its block means, its `resolution`, and its delivered clock range, so a run on another level reads
  as an off mean with tight blocks and a run that moved as a high stdev and resolution. The block
  CI95 and LSC left the line, one being 1.41 times the other
- **the trimmed pair**: a trimmed mean, a winsorized stdev, and Yuen's CI95 and LSC print beside the
  plain four from five runs up, with the runs the trim dropped named. The plain pair says what a run
  costs on this host, disturbances included, and the trimmed pair whether a change moved the bench
- **what runs do not cover**: runs back to back share the host's state for their stretch, its clock
  above all, so a comparison across invocations wants `--pin-freq`, and even pinned an invocation
  carries an offset its own runs cannot see ([measuring-a-technique.md](notes/measuring-a-technique.md))
- **precision**: a mean and its stdev print at least as precisely as the claims beside them, so a
  comparison never falls below its own rounding

#### Acceptance check

On the 3900X, `iiac-perf-dev zcr-mpsc-v0-2t zcr-mpsc-v1-2t --runs 3 --pin-cpus 0,1 -d 2 --record
<dir>` writes six records with six distinct pids and one series id, each bench's three runs back to
back, and prints for each bench a line per run with its mean, `stdev blocks`, `resolution`, and
clock, then its mean, `stdev`, `CI95 runs`, `LSC runs`, and clock. The same list given as a
`benches` key in a config file runs the same benches. A unit test reproduces the design notes'
worked LSC, about 131 ns at n=3 from the six-run series. `vc-x1 validate` passes.

Passed, 2026-09-15, on the 3900X:

- the bench list wrote six records under `--record <dir>/`, pids 9 to 19 all distinct, one series
  `20260915T233947Z-8`, schema 7, `zcr-mpsc-v0-2t` runs 1 to 3 then `zcr-mpsc-v1-2t` runs 1 to 3,
  each bench's runs back to back
- each bench printed a line per run with its mean, `stdev blocks`, `resolution`, and clock, then
  `mean`, `stdev`, `CI95 runs`, `LSC runs`, and `clock`. The two benches read 101.7 +- 3.5 and
  99.6 +- 1.0 ns pinned to CPUs 0,1, unpinned in clock
- a scratch directory whose `iiac-perf.md` carried `benches = ["zcr-mpsc-v0-2t", "zcr-mpsc-v1-2t"]`
  ran both from a bare command line, the `Config:` list naming the file as the source
- the statistics tests pass, the design notes' six-run series among them, and `vc-x1 validate`
  passes

#### Ladder

- [feat: CI95 and LSC across processes opening][1] (done)
- [refactor: one owner for the series statistics][2] (done)
- [feat: a benches config key and --benches flag][3] (done)
- [feat: each bench runs in its own child process][4] (done)
- [feat: replicate each bench across processes][5] (done)
- [feat: label block and run error bars][6] (done)
- [docs: runs across processes in guide and usage][7] (done)
- [docs: pay the owed prose punctuation][9] (done)
- [feat: means at the precision of their claims][10] (done)
- [feat: a run sleep before every run by default][11] (done)
- [docs: runs cover placement, not a drifting clock][12] (done)
- [feat: run lines show spread, drift, and clock][13] (done)
- [feat: a trimmed mean and its Yuen interval][14] (done)
- [docs: what a claim about a technique needs][15] (done)
- [docs: define technique and split the two claims][16] (done)
- [fix: line the run table headers up with their cells][17] (done)
- [feat: CI95 and LSC across processes closing][8] (done)

#### Deliberation

- **No interleaving**: wink, at the opening, each bench's runs back to back.
  - tuning one algorithm, iiac-perf's primary purpose, is one bench per invocation, with nothing to
    interleave
  - benches are compared by their own mean, stdev, CI95, and LSC, and it is not a race, the block
    and run sleeps separating the measurements
  - the cost accepted: a slow drift of the host across one invocation lands on the benches measured
    in that stretch as bias, which neither bench's CI95 contains, and a comparison against another
    invocation carries whatever the host did in between. The guide says so
- **`runs` defaults to 5**: wink, at the opening, so the default report's error bars are across
  processes. A plain `all` takes about five times as long as before.
- **The bench list joins this cycle**: wink, at the opening, the positional `BENCHES`, `--benches`,
  and a `benches` key. It was the "A --config flag and a config key for every run parameter" entry's
  bench-list bullet, which now points here.
- **`--run-sleep` replaces `--gap`**: wink, at the opening. `qualify-environment` already respawns
  with a sleep between children, so one knob serves both, and `--runs` is shared, its default 5 for
  benches and 10 for `qualify-environment`.
  - the value takes `block_sleep`'s form, a time or a random range, so run starts do not lock to
    anything periodic on the host
  - default 0: a process start, the tick calibration, and the warmup already stand in front of
    each run, and a cold start is asked for by setting one
- **No in-process mode**: wink, at the opening. Every bench runs in a child.
- **The output**: wink's go at the opening, a line per run, the summary after, and `-v` for the
  children's full reports, so a bench at five runs does not print five band tables.
- **"run" names the between-process replicate**: wink, at the opening. The guide's measurement
  hierarchy already calls a process invocation a run, and "block" keeps the within-process
  replicate.
- **The parent owns the host state**: the sleep inhibit and the clock pin are started once in the
  parent, and a child gets `--no-inhibit` and no pin.
  - the clock pin restores on drop, so a child holding its own would restore the clock the parent
    pinned between two runs
  - CPU affinity does not pass to a child, so `--pin-cpus` goes on each child's line
- **A child's results come back as a record**: the child writes its `Record` to a file the parent
  names, and the parent reads it with the same struct, so `record.rs` keeps the one schema and the
  parent never parses report text as `qualify-environment` does.
- **The rungs under a waiver**: wink, at the opening's review, delegated the cycle through its
  last work rung, stopping before the close-out to review and test it together.
  - covers the work reviews, the description reviews, and every push to
    `feat-ci95-and-lsc-across-processes` from the opening through `docs: pay the owed prose
    punctuation`
  - does not cover the closing rung or its close-out: the acceptance check, the close-out shape,
    and Land are reviewed with wink
- **Three rungs inserted after the punctuation rung**: wink, reviewing the pushed rungs on both hosts
  (2026-09-15), as rungs, with two Todo entries beside them.
  - two unpinned 3900X invocations of `min-now` read 22.8 and 22.5 ns, each with `LSC runs` 0.1 ns,
    while two pinned with `--run-sleep 1s` both read 26.3 ns, so runs back to back share the host's
    clock state and `CI95 runs` is a lower bound wherever the clock drifts
  - the 7600x pair printed `mean 16.4 ns` beside `LSC runs 0.01 ns`, a comparison below its own
    rounding, and at `--decimals 3` read 16.355 ns with `CI95 runs` 0.001 ns, at the display floor
  - the run sleep defaults to `1-2s` before every run, the first included, so no run starts
    differently from the others (wink's proposal of a non-zero default)
  - `--decimals 3` as the default was weighed and not taken: it widens every band table to digits
    past the ps floor, where the comparison needs only the means beside a claim to match the
    claim's precision
  - the rungs follow the punctuation rung, already pushed, and touch only files it paid, rechecked
    by the same count at the last of them
  - the waiver covers them: wink's go to finish the cycle before the close-out, restated as "go
    ahead with the three rungs" (2026-09-15). The closing rung stays outside it
- **A fourth rung inserted, the run line's columns**: wink, reading the 7600x passes (2026-09-15).
  - `LSC blocks` is `CI95 blocks` times 1.41 at 16 blocks or more, and neither says what a run line
    is read for, a run on another level or a run that moved, so the line shows the stdev of the
    run's block means and its `resolution`
  - wink added the measured clock range, the dominant core's delivered clock over the run, one
    number when it holds within the stability tolerance as a pinned run's should, and the range
    across the runs in the summary
  - the acceptance check named the old columns and now names the new ones
  - covered by the same waiver, wink's "do it as a rung" (2026-09-15)
- **A fifth rung inserted, the trimmed pair**: wink, after a 3900X invocation whose last runs the
  host disturbed (2026-09-15), asking for a mean and stdev that ignore outliers so an A/B has an
  answer on a noisy machine.
  - the plain pair read 431.4 ns +- 56.5 where the bulk sat near 385, against a pinned invocation's
    383.7 +- 2.3, so the plain pair could not answer whether a change had moved the bench
  - 20% trimming with Yuen's interval, the standard robust form, rather than a median and MAD,
    which tolerate more but cost efficiency and state an interval awkwardly
  - both pairs print, since they answer different questions, the plain one what a run costs on this
    host and the trimmed one whether the code moved
  - wink asked whether every part should be winsorized: no, the value is trimmed and the spread
    winsorized, which is what makes the interval valid, and the row labels say which is which
  - covered by the same waiver, wink's "do it" (2026-09-15)
- **Split out at the opening**: each its own Todo entry.
  - naming what sets the level, the ring-offset and huge-page experiment
  - the 7600x's `all` re-record, the first use
  - `qualify-environment` moving onto the child runner, which this entry had subsumed

#### Ladder details

##### feat: CI95 and LSC across processes opening

The cycle's setup commit: publish the bookmark, delete `## Closed`'s contents, move the Todo entry
here and split its deferred bullets into their own entries, file the continuation notes, bump to
the opening's version, and rename the package to `iiac-perf-dev`.

##### refactor: one owner for the series statistics

The mean, CI95, and LSC arithmetic lives in `harness.rs` beside the block loop, and
`resolution.rs` applies the same LSC formula on its own, so a series of process means has no home.
One module takes a series of means, weighted or not, and both callers use it.

- `series.rs` owns `t975`, the count-weighted mean, and a `Series` of replicate means with its
  count, plain mean, sample stdev, CI95, and LSC. A replicate is whatever the caller calls one
  draw: a block, a resolution group, and next a process
- the block tier keeps its two means apart: the report's `mean` is the count-weighted one, exact
  over every sample, while CI95 and LSC treat each block mean as an equal replicate, as before
- the resolution curve builds its group means with the weighted mean and takes each level's LSC
  from the series, a level with fewer than two groups yielding no point, a case the loop's guards
  already exclude
- the design notes' six-run tp-pc series is a unit test now: stdev 58 ns and LSC 131, 85, and 55
  ns at n of 3, 5, and 10, matching the worked numbers
- `Series::mean` has no reader outside the tests until the run tier, and carries an allow saying
  so
- no number moved: the refactor keeps every formula, and the whole suite passed unchanged

##### feat: a benches config key and --benches flag

The bench list exists only as positional words, so a config file cannot say what runs. The
positional `BENCHES`, `--benches`, and a `benches` key resolve as one layered run parameter.

- precedence: names on the line, then `--benches`, then the key, and the `Config:` list's first
  line is `benches` with its source, `command line`, `--benches`, or the file. A bare line runs a
  config's benches, and the bench listing prints only when none of the three names any
- `--benches` takes the positional's vocabulary, names, prefixes, and `all`, comma-separated or
  repeated, and conflicts with positional names. The positional's value name became `BENCH`, so
  clap's conflict message tells the two apart
- the key is a list or one string, `benches = "all"`, and an empty list or name is a load error. A
  nearer file's list replaces the lower file's whole, like every scalar
- a command word in `--benches` or the key is refused before anything prints, naming the word and
  the line that runs it, since a command word runs alone and positionally. `suggest-freq BENCH`
  still resolves where it did
- the key sits in `docs/config.md`'s key block and in the example config, commented, and
  `docs/usage.md` gains the synopsis line and the precedence
- tested here from a scratch directory: a file's `benches = "min-now"` ran from a bare line,
  `--benches std-now` overrode it, `--benches setup` exited 2 before the banner, and a positional
  name beside `--benches` was a usage error

##### feat: each bench runs in its own child process

A bench list runs every bench in one process, so each inherits the placement the process drew and
the benches before it left. The parent spawns one child per bench, passing its knobs, and reads the
child's record back.

- the parent writes a spec per child, JSON in a scratch directory under the temp directory named by
  its pid and removed on exit, and runs `current_exe()` with a hidden `--child-spec PATH`. The spec
  holds resolved values, the bench's exact name, every `RunCfg` knob, and the record sink with the
  parent's resolved config, so a child loads no config file and parses no flags but `-v`
- a child skips the inhibit, the config, the banner, and the clock pin, pins main to the pool's
  first CPU, calibrates ticks, runs its bench, and prints only the report, inheriting stdout, so a
  single run per bench reads as it did in one process
- a child records straight to the `--record` target, its `pid` its own and its `config` the
  parent's. Reading the record back moves to `feat: replicate each bench across processes`, the
  first rung with a summary to feed
- the parent waits in `Command::status`, adding no thread, and a child's failure stops the list,
  the scratch directory and the clock pin dropped explicitly before the exit, since `exit` runs no
  destructors
- `suggest-freq` stays in process: it pins each candidate itself and drives the bench between pins
- bench resolution returns names with their run functions, and `find` looks one up exactly
- every bench now pays `settle_time`, 1.5 s by default, since the process warm is per process. The
  flag's help, `docs/usage.md`, the example config, and the guide's settle section said the warm was
  paid once for a whole list, and now say every bench pays it
- tested here: `min-now std-now --record` wrote two records with pids 8 and 9, the sandbox's pid
  namespace, and the scratch directory was gone afterwards. `zcr-mpsc-v1-2t --pin-cpus 0,1 -d 1`
  ran pinned in its child at 103.2 ns. A clock pin with children is untested here, the sandbox's
  sysfs being read-only

##### feat: replicate each bench across processes

One process per bench still gives one draw of its level. `--runs` and `--run-sleep` repeat each
bench in fresh processes, and the summary computes the mean, CI95, and LSC over the process means.

- `runs.rs` owns the loop: a bench's runs back to back, a run sleep before every run after the
  invocation's first, drawn per run from its span, and the spec and result files numbered across
  the whole invocation
- a child writes its record to a result file in the scratch directory as well as to `--record`,
  the recorder now holding several targets, and the parent reads it back through
  `record::read_summaries`, the record's own struct, so no report text is parsed
- output: one run prints the child's report as before. Several discard the children's stdout and
  print a line per run, pid, mean, `CI95 blocks`, and `LSC blocks`, then `mean`, `stdev`, `CI95
  runs`, and `LSC runs` over the run means, through the report's summary-row printer, now shared.
  `-v` keeps each child's report above its line. A probe bench's run prints that it recorded nothing
- the run mean is a plain mean of the run means, each process one draw, where a run's own mean
  stays count-weighted over its blocks
- `--runs` is one flag for both uses, `Option` with 5 for benches and 10 for
  `qualify-environment`, and `--run-sleep` replaced `--gap` there, a span re-rolled per child.
  `qualify-environment`'s children get `--runs 1`, since each is now a parent whose several runs
  would print run lines instead of the report it parses. The config keys `runs` and `run_sleep`
  serve bench runs, `qualify-environment` resolving before the config loads
- `-D` splits its total over benches times runs, where it had split over benches alone and five
  runs would have run five times its budget
- the block sleep and the run sleep draw through one `Dither::span_s`
- tested here: `min-now std-now --runs 3 -d 0.5 --run-sleep 100-300ms --record` took 20 s, wrote
  six records with pids 8 to 13, and printed within-process `CI95 blocks` of 0.2-0.5 ns beside
  `CI95 runs` of 2.6 ns for `min-now` and 1.6 ns for `std-now`, the cycle's problem on this host in
  one run. `-v` showed each report above its line, `-D 1 --runs 2` gave each run 500 ms, and
  `qualify-environment --runs 2 --run-sleep 50ms --print-only` still parsed its children's grades

##### feat: label block and run error bars

A report row cannot say whether its CI95 is within a process or across processes. The rows name
their replicate, block or run, and a record carries the series id that groups its runs.

- a run's report rows are `CI95 blocks` and `LSC blocks`, beside the bench's `CI95 runs` and `LSC
  runs` from the replication rung, so no row says `CI95` alone
- schema 7 adds `series`, the invocation's id, its UTC start to the second and the parent's pid,
  and `run`, the 1-based run among its bench's runs, both null outside a bench child, which leaves
  `suggest-freq`'s in-process records. `run_index` stays, reading 0 in every child's record, and
  the history says so
- the parent makes the id once, the record spec carries it to every child with the run number, and
  the child stamps its recorder before the bench runs
- the report does not print the id: it is a record's grouping key, and the terminal already shows
  which runs belong together
- tested here: two invocations into one file, `--runs 2` and `--runs 1`, wrote schema 7 records
  with series `...-617` runs 1 and 2, and `...-620` run 1, and a single run's report printed
  `CI95 blocks` and `LSC blocks`

##### docs: runs across processes in guide and usage

The guide calls `LSC` a lower bound and tells the reader to run 3-5 times by hand. It documents the
across-process rows, `--runs`, `--run-sleep`, and the drift caveat of comparing benches.

- the measurement hierarchy's run level is a child process, `--runs` of them per bench, and the
  series level is one invocation's runs, the record's `series`, so the tool now computes at the
  run level and the reader compares across invocations
- a new section, `A bench's runs`, decodes the run lines and the summary with the 3900X's `min-now`
  output from the replication rung: three runs whose blocks claimed 0.2-0.5 ns and a `CI95 runs` of
  2.6 ns, the ratio saying per-process state dominates, and the cost, a settle warm per run
- `Comparing two implementations` now runs one bench per invocation with the default five runs and
  compares `mean` against `LSC runs`, the example measured for it on the 3900X, pinned
  `zcr-mpsc-v1-2t -d 2`: run 1 at 111.2 ns with blocks agreeing to 0.5 ns and runs 2-5 near 137 ns,
  so `CI95 runs` 14.4 ns covers both levels where a single process would have claimed either.
  The block rows stay as within-process lower bounds, and the caveat is the deliberation's: a bench's
  bars cover its own stretch, and a comparison across stretches carries the drift between them,
  checked by repeating later or alternating invocations by hand
- the row renames reach `docs/usage.md`'s block entries, the `--blocks` help, and the example
  config, and the header bracket's `warm=` line says every run carries the settle budget. The
  7600x evidence the guide quotes is the one-process levels, 60.4 to 71.9 ns each claiming a CI95
  under 0.1 ns, as the cycle's problem records it

##### docs: pay the owed prose punctuation

A file the cycle edits owes its whole prose's semicolons and untypeable punctuation, the ops notes
from the opening on. This rung converts every touched file's prose, code spans exempt, so each
earlier rung's diff reads as its change alone.

- owed after the docs rung, counted by a script over the cycle's diff with fences and code spans
  blanked: `harness.rs` 71 comment lines, `report.rs` 33, `main.rs` 29, `qualify.rs` 8, the
  qualification test 5, the ops notes 4, `resolution.rs` 3, and one each in the README,
  `config.rs`, and `dither.rs`. The other touched files owed nothing
- the conversion was delegated to three Sonnet agents, one per file group, under the prose rules'
  joins: a period for two claims, a comma with a conjunction for a continuation, a colon for a term
  and its explanation, and `->` and `...` for the arrow and ellipsis. Their diffs were checked to
  change only comment lines, and the recount reads zero
- two user-visible strings carried an em dash and now do not: the banner reads `iiac-perf-dev
  <version> - Rust latency microbenchmark harness`, and the `qualify-environment` help line ends in
  a colon. The guide's quoted old banner is transcribed output and keeps its dash
- left as they were: `≡` and `×` in comments, not among the four banned characters, semicolons
  inside string literals, which the rule covers only in comments, and the lines already past 100
  columns that the conversion did not touch
- one join was redone by hand, the ops notes' sandbox bullet, where the agent's comma-so doubled a
  `so` already in the sentence

##### feat: means at the precision of their claims

A mean prints at `--decimals` while the claims beside it extend until their leading digit shows, so
`mean 16.4 ns` sits beside `LSC runs 0.01 ns` and the comparison falls below the rounding. A mean,
and its stdev, print at least as precisely as the claims in its rows.

- `report::claim_precision` reads the decimals each rendered claim extended to, `<0.001` asking
  for 3 and a withheld `-` for none, and returns the larger of that and `--decimals`, capped at 3
  unless `--decimals` asks for more
- a run's report renders `mean`, `stdev`, and the trimmed pair after its claims, at the precision
  of `resolution`, `CI95 blocks`, and `LSC blocks`. `resolution` starts at 2 decimals, so every
  report's means now print at 2 or more, which is the resolution the run claims
- the runs summary prints `mean` and `stdev` at the precision of `CI95 runs` and `LSC runs`, and
  each run line's mean at the precision of its own block claims. A test holds the 7600x's five
  `min-now` means at `--decimals 1`, which print `16.354` beside an `LSC runs` of `0.002`
- run lines of different precision staggered their decimal points, so each cell pads after its
  unit until the points line up, and a line drops its trailing spaces
- the band table and the quoted outputs in the guide are untouched: `--decimals` still sets the
  band table, and `--decimals 3` stayed off the default (the insertion's deliberation)

##### feat: a run sleep before every run by default

With no run sleep, a bench's first run starts from whatever the host did before the invocation and
the others start hot from the run before, so the first run is structurally different. The default
becomes `1-2s`, drawn before every run, the first included.

- `runs::DEFAULT_RUN_SLEEP_S` is `(1.0, 2.0)`, drawn per run through `Dither::span_s`, and the
  runner sleeps before every run rather than every run after the invocation's first
- `qualify-environment` keeps its default of 0, since its table exists to catch the transitions a
  sustained duty cycle provokes
- the cost is about 1.5 s a run, 7.5 s a bench at five runs: `min-now --runs 2 -d 0.3` took 8 s
  here against 5 s at `--run-sleep 0`
- the flag's help, `docs/usage.md`, `docs/config.md`'s defaults and key block, and the example
  config say the new default, the example's test holding it
- whether a sleep changes what a run reads is not shown by the evidence behind it, wink's 3900X
  pair having changed the sleep and the pin together. The Todo entry the docs rung files measures it

##### docs: runs cover placement, not a drifting clock

The guide, the module docs, and the cycle record call `CI95 runs` the first error bar that is not a
lower bound, while a 3900X pair shows runs back to back sharing a drifting clock state. The docs say
what runs cover, name `--pin-freq` for comparisons across invocations, and file the two Todo
entries.

- the claim now reads that run error bars are the first to cover placement: `runs.rs`'s module
  doc, the guide's `A bench's runs`, and `docs/usage.md`'s `--runs` entry say runs back to back
  share the host's state for their stretch, the clock above all, and quote the 3900X pairs, 22.8
  against 22.5 ns unpinned and 26.3 against 26.3 ns pinned
- the guide's comparison caveat names the clock as the drift measured so far and asks for
  `--pin-freq`, or `pin_freq` in a directory's config, before the repeat-later and alternate-by-hand
  fallbacks
- the ratio bullet gains the 7600x's opposite case, blocks claiming 0.006-0.008 ns under a
  `CI95 runs` at the 0.001 ns floor, and the caution that five runs judge a spread only to a factor
  of 2 or 3. The cost bullet adds the run sleep, and a precision bullet says the quoted output
  predates the claim-precision rung
- Todo entries: `Does the 3900X's unpinned shift follow its clock`, the recorded-clock experiment
  with the sleep's own effect beside it, after the 7600x re-record, and a note on `Trimmed core
  stats` that the runs summary averages full means and would use a fixed-quantile trimmed mean
- the punctuation count over the cycle's files reads zero after the three inserted rungs, so the
  punctuation rung's payment stands

##### feat: run lines show spread, drift, and clock

A run line's `CI95 blocks` and `LSC blocks` repeat each other and show neither a run on another
level nor a run that moved, and nothing shows the clock a run measured at. The line shows the stdev
of the run's block means, its `resolution`, and its delivered clock range, and the summary the clock
range across the runs.

- a run line reads `run  pid  mean  stdev blocks  resolution  clock`. `stdev blocks` is the sample
  stdev of the record's `block_mean_ns`, `resolution` the record's `resolution_ns`, and the mean
  prints at the precision of the two, as it did beside the claims it replaces
- `clock` is the record's seam clock through the gauge's `clock_profile`, which keeps the dominant
  core's samples, so an unpinned run's range is the measuring core's clock rather than a tour of the
  scheduler's placements. It prints one number, the range's midpoint, when the range holds within
  `FREQ_STABLE_TOL`, 1%, and `min-max GHz` otherwise, `-` where no clock is readable
- the summary adds a `clock` line under `LSC runs`, the lowest and highest any run read, padded to
  the rows' labels since the rows' printer carries ns
- `RunSummary` drops the block CI95 and LSC for the stdev, the resolution, and the clock range, all
  derived from fields every record already carries, so the schema did not move
- tested here unpinned on the busy 3900X: `zcr-mpsc-v1-2t --runs 5 -d 1` showed run 1 at 410.4 ns with
  `stdev blocks` 79.7 and `resolution` 42.7 ns beside runs near 110 ns, and clocks from 3.29 to 4.54
  GHz, which the guide now quotes. The one-number clock of a pinned run is tested by unit only, the
  sandbox's sysfs being read-only for a pin
- the guide's `A bench's runs` explains the columns, what a moved run and a level run look like, and
  the fair ratio, the run stdev against `stdev blocks` over the square root of the block count. The
  usage entry names the columns, and the clock-row Todo entry records that the runs tier landed

##### feat: a trimmed mean and its Yuen interval

A host that disturbs a few runs moves the plain mean and widens its bars past use, so a bench list
on a busy desktop cannot answer whether a change helped. A trimmed mean and its Yuen interval print
beside the plain pair, answering the other question.

- `series.rs` gains `Trimmed`: 20% of the runs dropped from each end, the mean of what is kept, the
  winsorized stdev of the whole, and the CI95 and LSC from Yuen's standard error,
  `winsorized stdev / ((1 - 2 * trim) * sqrt(n))`, at `kept - 1` degrees of freedom
- the pairing is the method, not an oversight: the value is trimmed because dropping is what keeps
  a disturbed run out of it, and the spread is winsorized because that run's absence is itself
  uncertainty. The row labels name each, `trimmed mean` over `winsorized stdev`
- the summary prints the four rows from five runs up, below the plain four, and a `trimmed` line
  naming the runs that went, which is the run mark the Todo entry asked for
- tests: the 3900X's twenty run means, where the plain pair reads 431.4 +- 56.5 ns and the trimmed
  pair 385.5 +- 4.0, and a ten-value series checked against hand arithmetic
- the guide's `A bench's runs` says which pair answers which question, quotes both 3900X
  invocations, and prices the trimmed pair on clean data: ten quiet `min-now` runs read `CI95 runs`
  0.2 ns against `CI95 trimmed` 0.4
- the report's `mean z4..n2` row is untouched: it trims a run's sample distribution to describe the
  workload's core, which is not an estimator's robustness against disturbed replicates

##### docs: what a claim about a technique needs

The benches exist to test techniques meant for many applications and platforms, but every number the
tool prints is one host's, and nothing says what a claim about a technique needs beyond them. A notes
file states it: the claim is a ratio, the replicates nest, and the harness has a portable core under
a per-platform environment layer.

- `notes/measuring-a-technique.md` is a new file rather than a section of `notes/design.md`, whose
  132 owed semicolons and dashes would have made this cycle a punctuation sweep
- what it holds: the ratio as the portable claim, since conditions cancel in a pair measured
  together, the four nesting replicates, environment, build, run, and block, with what each covers,
  what the tool covers today, the A/A evidence, and the portable core under the environment layer
  with its per-OS notes, a bare-metal target's lack of processes included
- the A/A evidence is the cycle's own: the same binary, clock pinned, read 383.7 and 387.0 ns on the
  3900X 90 minutes apart, past its own `LSC runs`, and 70.0, 70.4, and 70.6 ns on the 7600x, so an
  invocation carries an offset that its runs cannot see
- two Todo entries follow from it, `Compare two builds in one invocation`, the paired arms that
  cancel that offset, and `Replicate builds so layout is not confounded`, k builds an arm against
  the 11% a rebuild moved. The port entry points at the seam, and `notes/README.md` at the file,
  its one owed semicolon paid

##### docs: define technique and split the two claims

The notes file leaves "technique" undefined and holds every claim to one standard, the durable one,
which overstates what the everyday question costs. It defines the word and splits the two claims,
"did this change help" from "is this technique faster", with the replicates each needs.

- the definition sits under the intro: a technique is a way of doing inter- or intra-application
  communication, a ring layout, a handoff protocol, an ordering choice, a spin against a park, where
  a binary is one implementation of one technique on one platform
- the word stands (wink's question at this rung): `algorithm` is too narrow, since v1 against v2 is
  one algorithm with a different layout and ordering, `tweak` and `refinement` prejudge the size,
  `implementation` names the binary the claim must outlive, and `design` and `mechanism` are no
  clearer. It is also the project's own word, which the one-spelling-per-term rule wants settled
- a `Two claims, two costs` section opens the file: the everyday claim needs the run, the block, and
  the arms paired in one invocation, and the durable claim adds replicated builds and a table across
  environments. The nesting section closes on the same split, so a tuning session is not priced at a
  published claim's cost

##### fix: line the run table headers up with their cells

A run line's ns cells pad after the unit so their decimal points line up, and the headers are
right-aligned to the column's edge, so each header sits two columns right of the values under it.
The ns headers take the same pad.

- each ns header is right-aligned two columns short of its field, `point_cell`'s own pad at the
  usual one or two decimals, and the gap before the clock column absorbs the difference, so all four
  headers end where their cells end
- a test holds it, reading each label's end and its cell's end out of a rendered header and line
- a cell with three decimals still runs two columns past its header, which is the price of lining
  the decimal points up within a column and is the rarer case
- the guide's quoted run table predates this, as its quoted outputs do

##### feat: CI95 and LSC across processes closing

Closing out the cycle.

- the acceptance check passed on the 3900X, its clauses recorded above. The trimmed rows did not
  appear in it, three runs being below the five a trim needs, which is the check reading as it was
  written before that rung
- what outlives the cycle: [measuring-a-technique.md](notes/measuring-a-technique.md), the ops
  notes' clock-pin and pin-pair bullets, and five Todo entries, the two builds compared in one
  invocation, replicated builds, the run allocation model, the pinned level counts, and the level
  mark
- the agent-files were untouched, so `notes/agent-files-size.md` takes no row, its count standing
  at 2315 lines over ten files
- close-out shape: trapezoid, wink's choice at the close-out, since seventeen commits on `main`
  would read as seventeen cycles where several of the rungs correct earlier ones of this cycle.
  Land reshapes it, and every rung stays reachable
- what closing taught: a check written at the opening ages. This one named `CI95 blocks` and `LSC
  blocks` in the run lines, and two rungs later the lines carried `stdev blocks`, `resolution`, and
  a clock, so the check was rewritten at the rung that moved it rather than at the close-out, where
  the rewrite would have looked like fitting the check to the result

# References

[1]: #feat-ci95-and-lsc-across-processes-opening
[2]: #refactor-one-owner-for-the-series-statistics
[3]: #feat-a-benches-config-key-and---benches-flag
[4]: #feat-each-bench-runs-in-its-own-child-process
[5]: #feat-replicate-each-bench-across-processes
[6]: #feat-label-block-and-run-error-bars
[7]: #docs-runs-across-processes-in-guide-and-usage
[8]: #feat-ci95-and-lsc-across-processes-closing
[9]: #docs-pay-the-owed-prose-punctuation
[10]: #feat-means-at-the-precision-of-their-claims
[11]: #feat-a-run-sleep-before-every-run-by-default
[12]: #docs-runs-cover-placement-not-a-drifting-clock
[13]: #feat-run-lines-show-spread-drift-and-clock
[14]: #feat-a-trimmed-mean-and-its-yuen-interval
[15]: #docs-what-a-claim-about-a-technique-needs
[16]: #docs-define-technique-and-split-the-two-claims
[17]: #fix-line-the-run-table-headers-up-with-their-cells
[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[75]: /notes/chores/chores-05.md#settle-time-is-not-a-grade
[84]: /notes/chores/chores-06.md#docs-experiment-in-the-local-agent-files
