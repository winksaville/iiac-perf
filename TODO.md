# Todo and cycle record

This file contains near term tasks with a short description and reference links to more details.
Its shape is [Todo format](agent-data/notes.md#todo-format).

## Continuation notes

Where the agent was, for the agent that comes next: working copy state, the step in flight, an
open question. Ephemeral, never a record. Written before a restart or when a session is about to
lose context, read first at acquaint, acted on, each fact filed into its home or its bullet kept, and
the rest reset to `_None._` by the reader.

_None._

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. The block's
shape is the specimen in [cycle-model.md](agent-data/cycle-model.md), and the rules are in
[The In Progress block](agent-data/notes.md#the-in-progress-block).

### feat: spsc v3 and mpsc v2 benches

#### Problem

zc-ring-x1 has a segmented SPSC, `spsc::v3`, and a segmented MPSC, `mpsc::v2`, each a ring of
segments taken from a pool at init, with a user guide and examples for both (its
`notes/user-guide.md`, 2026-09-16). Nothing here measures either. With a consumer that keeps up a
segmented ring lives in one segment, and the claim is that v3 then costs nothing over v2, and v2
nothing over mpsc v1, so the round-trip pairs are the no-switch baseline the segment switch cost
entry builds on.

#### Solution

Bump the `zc-ring-x1` dependency and add `zcr-spsc-v3-1t`/`-2t` and `zcr-mpsc-v2-1t`/`-2t` beside
the v2 and v1 pairs, the same shapes, the pool sized by each module's `segment_size`, so the `all`
run sets each new pair against its predecessor with CI95 and LSC on both hosts.

#### Acceptance check

On the 3900X, `iiac-perf-dev zcr-spsc-v2-2t zcr-spsc-v3-2t zcr-mpsc-v1-2t zcr-mpsc-v2-2t --runs 5
--pin-cpus 0,1 -d 2` prints each bench's run table, both new benches report zero switches on every
run, and each new pair's mean is within `LSC runs` of its predecessor's or the difference is
recorded in the closing's subsection as a finding. The four 1t benches run under `all` without
error.

#### Ladder

- [feat: spsc v3 and mpsc v2 benches opening][1] (done)
- [feat: the spsc v3 pair over a pool][2] (done)
- [feat: name the pin pool's placement][5] (done)
- [feat: units on every time flag][6] (done)
- [feat: the mpsc v2 pair over a pool][3]
- [feat: spsc v3 and mpsc v2 benches closing][4]

#### Deliberation

- one rung per ring, not one per bench: a pair shares its ring setup in `zcr_common`, so the pool
  sizing and the leak helper are written once per ring and the 1t and 2t benches of a version land
  together
- the switch count is an assertion, not a measurement: the segmented rings expose `switches()` on
  both ends, and a nonzero count in a round trip means the bench is not the no-switch shape it
  claims, so a run reports it beside the row rather than hiding it in a mean
- the segment count is two: one is the baseline, and a second is what makes the ring a segmented
  one at all, so the header lines a switch would touch exist and are cold, as they will be in use
- a placement rung inserted after the v3 pair, wink's pick on 2026-09-16 after reading the pair
  at an SMT pin: a run's pin pool should say what the two CPUs share, as zc-ring-x1's demo does,
  so a reading is placed without a topology map at hand. The small form, sysfs siblings and L3
  sharing, and the `Topology-aware pinning and lCPU terminology` entry keeps the tree
- a units rung inserted after the placement rung, wink's pick on 2026-09-16 while shortening a
  run: `--duration`, `--settle-time`, and `--warm-cap` take bare seconds where the sleeps take a
  span with a unit, so one invocation mixes two spellings of a time

#### Ladder details

##### feat: spsc v3 and mpsc v2 benches opening

The cycle's setup commit: create and publish the bookmark, delete `## Closed`'s contents, move the
Todo entry into this block, rename the package to its dev name, and bump the version-of-record.

##### feat: the spsc v3 pair over a pool

The v2 pair's shapes over `spsc::v3`: a pool of two segments sized by `segment_size`, leaked like
the v2 rings, the 1t loop and the 2t echo, the switch count read at the end and reported.

* The dependency moves from 0.15.8 to 0.17.3, the commit carrying the user guide, and the four
  existing zcr pairs build unchanged against it.
* The pool is one helper, sized by the ring version's own `segment_size`, since the segment header
  differs by version, and the mpsc v2 rung reuses it.
* The 2t bench's worker hands its two ends' switch counts back through its join, so the four counts
  print after the report, and `Drop` runs the same shutdown when the entry point has not.
* The finding, on the 3900X pinned 0,1, three runs of a second: v3 reads 14.9 ns same thread
  against v2's 4.7, three times, and 105.5 across threads against 110.1, both outside `LSC runs`,
  with zero switches in every run. A rerun with one segment read 17.7 same thread, so the cost is
  v3's no-switch path itself and not the second segment. We think the cross-core handoff hides it
  in the 2t shape. Recorded in the report guide, and a lead for zc-ring-x1 at the closing.

##### feat: name the pin pool's placement

The Setup `bench pin` line and the run summary name the CPUs and nothing about them, so a reading
at `11,23` is placed only by whoever knows the host. Label the pool by what its CPUs share, read
from sysfs, `SMT`, `CCX`, or `x-CCX`, in the demo's form, and check the labels against zc-ring-x1's
so the two agree on every pair.

* The demo's words, not new ones: `core N`, `SMT`, `CCX`, `x-CCX`, and `unpinned`, read from the
  same two sysfs files it reads, the first CPU's `thread_siblings_list` and its cache index 3
  `shared_cpu_list`, so the two tools cannot disagree on a pair. Checked on the 3900X at the demo's
  three pairs from CPU 11, and at `0,1`, one CPU, and unpinned.
* The label judges the whole pool from its first CPU, so a pool of three is `CCX` when all share
  the L3 and `x-CCX` when one does not, and an unreadable topology prints no label rather than a
  guess.
* Both lines carry it: the Setup `bench pin` line closes with the label, and the runs header reads
  `at 11,23 SMT`, so a pasted summary places itself without the Setup block.

##### feat: units on every time flag

`--duration`, `--settle-time`, and `--warm-cap` take bare seconds while `--run-sleep`,
`--block-sleep`, and `--block-warmup` take a duration with a unit, so a command line spells a time
two ways. Let the three take a unit too, bare seconds still accepted, and their config keys with
them.

* One parser for the seconds knobs, `-d`, `-D`, `--settle-time`, and `--warm-cap`: a bare number
  is seconds, as those flags always read, a unit is honored, and a range or a negative is refused
  at the flag, so the negative checks after layering are now unreachable from the line and stay
  for the config's number form.
* The three config keys take a number, seconds, or a string with a unit, so a config reads as the
  line does, and the two sleeps' keys keep their string-only form, since a bare number there was
  never seconds.

##### feat: the mpsc v2 pair over a pool

The v1 pair's shapes over `mpsc::v2`, whose send is a closure with no guard and whose producer
handle is Clone, over a pool sized by its own `segment_size`, the three-line segment header
included.

##### feat: spsc v3 and mpsc v2 benches closing

Closing out the cycle.

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

### Segment switch cost

The demo's segment stress prices one switch as a difference of two means at equal capacity, 32x1
against 1x32, with no interval and a consumer whose lag scheduling decides: 16 ns for spsc-v3 and
58 for mpsc-v2 streaming across the 7600X's same-L3 pair, 140 and 270 to 290 cross-CCX on the
3900X, against 3 to 7 and 7 to 13 single-threaded (zc-ring-x1, 2026-09-15). "Expensive" is a
claim about a technique, and this app makes it one with an interval, per host and placement, and
then says which lines a switch moves, so zc-ring-x1's candidates in its `Cheaper segment switches`
entry are chosen and then checked by number.

- the shapes are the one-way entry's burst and producer-only benches over v3 and v2, with
  `--segments` and `--depth` on the line, and the lag made deterministic: at depth 1 a consumer
  holding one slot forces a switch per message, where the demo's lagging rows leave it to the
  scheduler and report no time
- the sweep at equal capacity, 32x1, 16x2, 8x4, down to 1x32, has switches per message of one
  over depth, so per-message cost against that is a line: the slope is the switch cost, the
  intercept the no-switch cost, each with a CI95, and a depth-1 point off the line says the
  first switch has a cost of its own, the cold segment header
- transfers per switch without hardware counters: the slope at cross-CCX less the slope at same-L3,
  over the per-transfer cost the spsc handoff already gives at each placement. The design note's
  informal count is one for spsc-v3 and three to four for mpsc-v2, and this makes it a table by
  placement on both hosts
- that table is the acceptance check for each candidate: a cycle per candidate in zc-ring-x1, each
  closed by rerunning this sweep, the reply to its thread carrying the numbers
- runs pinned with the frequency held, since the cross-CCX slope on the 3900X is inside the
  unpinned shift the clock entry above is chasing

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

_None._

# References

[1]: #feat-spsc-v3-and-mpsc-v2-benches-opening
[2]: #feat-the-spsc-v3-pair-over-a-pool
[3]: #feat-the-mpsc-v2-pair-over-a-pool
[4]: #feat-spsc-v3-and-mpsc-v2-benches-closing
[5]: #feat-name-the-pin-pools-placement
[6]: #feat-units-on-every-time-flag
[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[75]: /notes/chores/chores-05.md#settle-time-is-not-a-grade
[84]: /notes/chores/chores-06.md#docs-experiment-in-the-local-agent-files
