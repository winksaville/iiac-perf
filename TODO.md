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

### feat: a shorter trustworthy run on the 7600x

#### Problem

A bench invocation costs about 72 s: ten runs of 7.2 s, of which 5.1 s measures, 1.5 s warms, and
0.55 s sleeps between blocks. None of those numbers was measured against what a host needs, they
are defaults chosen for safety, and three of them are already known to be unjustified on a quiet
host. The 240 records of `feat: a run is a config file` show the warm is floored at `settle_time`
while the pinned 7600X settles in 10 ms in 30 runs of 30, the block knee sits near 16 of the 100
blocks (0.02% against 0.01%), and `runs` is a plain 1/sqrt(k) dial with no knee at all. So an A/B
costs minutes where it may cost seconds, and nobody knows how far it can be cut before the numbers
stop being worth trusting (wink, 2026-09-18).

"Worth trusting" is itself undefined, and tightness alone will not do: the same records show the
unpinned 3900X's block means carry a lag-1 autocorrelation of +0.78, which leaves about 11
effective blocks of 100 and understates `CI95 blocks` about threefold. A config can print a tight
error bar it cannot back.

#### Solution

Trustworthy is two numbers, precision and calibration: the claimed `LSC runs` is under a target,
and repeated sessions of the same config land inside what it claimed. The per-run overhead `o` is
cut first, since the optimal run length goes as sqrt(o) and searching the length first would find
the wrong one; the run length `d` follows at the reduced overhead; `runs` falls out of the target
rather than being searched. The candidate is then validated by repetition and confirmed on
`min-now` before a config is shipped and the guide says what each knob bought. The 7600X alone,
pinned, on its quiet cpus, the 3900X being the next cycle.

#### Acceptance check

On the 7600X, `iiac-perf configs/quick.toml zcr-spsc-v3-2t` completes in under 15 s, against
today's 91 s, and its `LSC trimmed` is at or under 0.5% of its trimmed mean. Run five times, no
contact with the host while any of them runs, and the five trimmed means agree within 0.5% of each
other and within 1% of a 5 s reference config at the same pin state and cpus. `min-now` under the
same config reports `LSC trimmed` at or under 0.5% too. The report guide says what each knob
bought, why the trimmed pair is the statistic, and what a reader should measure first on a host
that is not this one.

#### Ladder

- [feat: a shorter trustworthy run on the 7600x opening][1] (done)
- [docs: what a trustworthy run means][2] (done)
- [feat: one record file per invocation, not per run][3] (done)
- [docs: the statistics behind a run's claim][4] (done)
- [feat: a lower-band trimmed mean][5] (done)
- [feat: host facts leave the project configs][6] (done)
- [feat: a record carries its config and a label][7]
- [feat: analyze checks a claim across invocations][8]
- [feat: figures drawn by the tool][9]
- [docs: the overhead floor on the 7600x][10]
- [docs: the run length at the new overhead][11]
- [feat: the quick config for the 7600x][12]
- [feat: a shorter trustworthy run on the 7600x closing][13]

#### Deliberation

- The 7600X alone, pinned, on its quiet cpus (wink, 2026-09-18). It is the quieter host, so its
  floor is the one that exists; the 3900X's is [A shorter run on the 3900X, and the allocation
  hint](#a-shorter-run-on-the-3900x-and-the-allocation-hint).
  - Pinning is also what makes the blocks independent: lag-1 +0.02 on the 7600X either way, but
    +0.78 unpinned against +0.20 pinned on the 3900X. A floor found on dependent blocks would be
    a floor under a mis-stated error bar.
- Trustworthy is precision and calibration, not precision alone (2026-09-18).
  - The autocorrelation finding is why: a config that claims a bar it cannot back is worse than a
    slow one, and only repetition catches it.
- The overhead is cut before the run length, not alongside it.
  - The moved entry's model gives the least-variance run length `d* = sqrt(a * o / s_p^2)` at a
    fixed budget. wink's question is its dual, least time at a fixed precision, and minimising
    `T = (s_p^2 + a/d)(o + d) / Var` gives the same `d*`, so one model serves both.
  - `d*` goes as sqrt(o), so cutting the overhead lowers the optimum for everything after it. At
    the entry's 7600X estimate `d*` is near 0.3 s; an overhead cut of twentyfold would put it
    near 0.07 s. Searching `d` first would find the wrong one and then have to be redone.
- What is a condition and not a knob, so a step cannot be confounded (wink, 2026-09-18):
  `--pin-freq` on, `--pin-cpus 4,5`, one bench, `env_probe` on, `samples` and `inner` auto, one
  build, a quiet host, the sleep inhibit on.
  - One bench for the search, `zcr-spsc-v3-2t`: two threads and a ring are the dynamics most
    likely to break at small settings, where `min-now` is one thread reading a clock. A setting
    safe for the pair is likely safe for `min-now` and not the reverse, so `min-now` is the
    confirmation at the end and not the subject.
  - `env_probe` stays on because the environment grade is part of the criterion, so turning the
    seam probes off would change what is being asked, not only what it costs.
- The trimmed pair is the cycle's statistic, not the plain one (wink, 2026-09-18, at the
  baseline).
  - The baseline reads a tight core and a fat tail: 68 of 80 runs inside 93.7 to 96.1 ns and 12
    from 98.5 to 151.6, at run positions 1 through 9. The plain `LSC runs` is 6.8% at ten runs
    and is entirely the tail; reaching 0.5% with it would take about 1700 runs.
  - Trimmed, the eight invocations agree to 0.45% where plain agree to 1.88%, so the measurement
    underneath is already at the target and the plain statistic is what hides it.
  - The cost accepted: trimming drops four runs of ten and would hide a genuine two-level bench
    as readily as it drops a disturbed run. What the tail is remains open, and
    [Mark a run that lands on another level](#mark-a-run-that-lands-on-another-level) is where
    telling the two apart belongs.
  - `s_p` is contaminated by the same tail, so the model's `d*` is recomputed from the trimmed
    spread before it is used.
- A measurement host is not polled while it measures (wink, 2026-09-18).
  - The first baseline was watched by an ssh poll every 90 s while each invocation took about
    91 s. The repeat meant to clear it was not clean either: the watch from the first was still
    inside its half-hour and polled the repeat at the same rate throughout, which the agent did
    not notice until it expired. So the two baselines are polled against polled and say nothing
    about polling, and a third was run with no watcher of any kind.
  - The rule earns itself twice over: a contaminated measurement cannot be untangled afterwards,
    only repeated, and here the repeat was contaminated the same way by a watch that had outlived
    what it watched. Compute the expected duration, add a tenth, wait that long, look once, and
    arm nothing.
- Records go to one file per experiment, appended, where they went to a file per run (wink,
  2026-09-18).
  - Per experiment rather than per condition, which the agent first proposed: a file's name can
    come to lie about its contents, as the directory `knobs-quiet` did once its runs turned out
    polled, while the `condition` tag is written into each record at run time and cannot.
  - The cost accepted is that a crash mid-append can leave a broken last line where a file per
    run isolated it, so a reader skips a line that will not parse.
  - A directory now means a file per invocation rather than per run (wink, 2026-09-18), so
    neither form can scatter one command's output, and the choice between them is only whether
    commands share a file.
- The analysis moves to Rust, the definitions that are fixed and no others (wink, 2026-09-18).
  - Python stays the tool for exploring, and figures stay outside Rust: the `analyze` entry
    already plans `--format csv` and `json` for the plotting hand-off, so Rust computes and a
    script draws.
- The statistics are written up, with figures, before they are ported (wink, 2026-09-18), since
  a test asserting a verdict turns the agent's threshold into a project fact, and wink asked to
  be taught enough to judge it first.
- `runs` is computed, not searched: at a stated target and a known `o` and `d`, the count follows
  from `R = T / (o + d)`. The records put its knee between 3 and 5 runs and it is 1/sqrt(k)
  thereafter, so there is nothing to discover.
- `--warm-cap` is an output of the cycle, not an input (wink asked why 1.5 s, 2026-09-18).
  - It is a ceiling on the warm-until-stable stretch, paid only when the box does not settle, and
    `warm_exit` reads `settled` in all 240 records, so it never fired. It costs nothing on a quiet
    host and shrinking it buys nothing; the 15 s per bench is `settle_time`, a floor always paid.
  - Its value should come from the observed `settle_s` distribution for the host and pin state,
    about its p99, which is 10 ms on the pinned 7600X. 1.5 s was carried forward by inertia.
- This cycle's configs take the TOML carrier, not the markdown one (wink, 2026-09-18).
  - The search writes and rewrites a config at every step, and the markdown carrier wraps ten keys
    in two hundred lines of prose explaining every other key. The TOML carrier is the section
    headings and the keys alone, which is what `feat: a run is a config file` shaped it for.
  - The shipped `configs/quick.toml` keeps that form, and the prose that explains the numbers goes
    to the report guide, where a reader looks for reasons rather than into a config.
- `blocks` is fixed and block *size* is the variable: they are one quantity, since
  `blocks x block size = d`, and shrinking `-d` with `blocks` held drives each block toward too
  few samples to be a replicate. The records put the knee near 16.
- The free reductions are taken retrospectively where the records allow: `runs` and `blocks` both
  subset out of a generous recording, so only the time-valued knobs need new invocations.
  - The caveat: those records are `min-now`, so the subsetting is re-done on the 2t bench at the
    first rung rather than assumed to transfer.
- The goal is a baseline config, one under which a change between any baseline bench and a
  modification of it is detected reliably, in the least wall time, and not an answer to "how fast
  is bench xyz" (wink, 2026-09-19).
  - So an absolute number owes nothing to the unpinned machine: a pinned clock reads 8 to 17%
    slower than the boosted one, and that costs the goal nothing, since both sides of a
    comparison are read the same way. What the goal needs is the claim, `LSC trimmed`, to be
    small and to be true.
  - Its first use is optimizing `zcr-mpsc-v2` and `zcr-spsc-v3`, a short-term goal and not the
    config's scope, so their two-thread benches are what the pins experiment runs from here.
  - `min-now` is not in the pins experiment (wink, 2026-09-19): one thread reading a clock was
    repeatable to 0.03% in every condition, the unpinned one included, so it cannot tell the
    conditions apart. It stays a bench, and what the acceptance check asks of it is unchanged.
  - The one-thread benches are not in it either (wink, 2026-09-19): they were the simplest
    first bench, one thread has no pair to place, and their levels are a question no pin answers.
- A config is of a kind, and the kind decides what it may hold (wink, 2026-09-20): the host's,
  the project's or a run's, and later one for an analysis.
  - The host's file holds what is true of one machine and wrong on any other, `[freq]` and
    `[profiles]` now and the measured values later. A project's or a run's config names those
    facts, `pin_cpus = "smt"`, `pin_freq = "pin_mhz"`, and never states them, which is why one
    file ran unedited on both hosts.
  - The files already exist by location, the XDG file, the project-local one, a named one, and
    nothing enforces the split: any of them may hold any key, which is how a 3900X clamp reached
    a file named for the 7600X and how `block_warmup` came to depend on the directory a run
    started in.
  - This rung moves the facts and enforces nothing. The loader refusing a host table outside
    the host file, with a documented override, goes with
    [A setup-host command, the pin profiles first](#a-setup-host-command-the-pin-profiles-first).
  - The analysis config waits for `analyze` to exist, the agent's caution: its keys are what
    every experiment ends up passing, not what is guessed now. `trim_runs` is the first
    candidate, a run key today that changes nothing about how a run executes.
- A comparison pins, and where it cannot pin it observes and stratifies (2026-09-20).
  - What pinning buys is not speed, unpinned `ice-rr-2t` being 13% faster than `smt`. It is that
    the placement is held still, and placement is worth 2x here: `ice-rr-2t` reads 1183 ns
    inside one LLC domain and 2382 across two.
  - Unpinned, the floor is between invocations and no number of runs reduces it, each invocation
    drawing a fresh scheduler state: 3.0 to 6.5% on the 3900X against 0.25 to 0.38% pinned, and
    0.30 to 0.67% on the 7600X against 0.13 to 0.15%. So the rule binds hard on a host with four
    LLC domains and lightly on one with a single domain, being a property of the host's placement
    space and not a law.
  - The trim does not rescue an unpinned bench, since it cuts a tail where unpinned is two
    populations. It works when the good placement takes more than half the runs, `ice-rr-2t` at
    twenty runs trimming to 0.69%, and fails when it does not, `zcr-mpsc-v2-2t` at ten keeping
    two good runs and two bad and claiming 36.6%.
  - The warm phase cannot stand in for it: every unpinned run reported `warm_exit: settled` at
    105 ms and then ran up to four blocks at twice the time, the scheduler still migrating the
    pair into one domain. "Settled" is a claim about the clock and the probe grade and says
    nothing about placement, which is this cycle's evidence for the observational half of
    [Topology-aware pinning and lCPU
    terminology](#topology-aware-pinning-and-lcpu-terminology).
  - Where pinning cannot hold placement still, a pool past the lCPU count and an asymmetric
    bench's roles, observing it is what is left.
- Experiments live in one repo of their own, a directory each, and not here (wink, 2026-09-19).
  - A pushed record file is in this repo's history for good, and the pins session alone is
    7 MB beside the 4.7 MB `records/` already holds. An experiment's analysis is also the
    Python wink does not want here, and its cycle is not a rung of a feature's ladder.
  - One repo and not one per experiment, the agent's proposal, since each repo of this family
    is two repos and their agent-files, and experiments are compared with each other.
  - This repo keeps the tool, the configs it ships, and the record slices its tests need.
- The project-local config is tracked, as `iiac-perf.toml`, and is to become the baseline config
  itself, the example of a quick run whose claim holds (wink, 2026-09-19).
  - It reverses the ignore rule's "each checkout's own project-local config": the host's facts,
    `[freq]` and `[profiles]`, belong to the XDG file, so nothing in the project-local file is a
    host's, and a config that pins by a profile name runs unedited on both hosts.
  - It is a new file, not an edit of a tracked one, so it holds this cycle's candidate from the
    start rather than the three block keys it began with (wink, 2026-09-21), with the comments
    saying which values are unjustified. An earlier bullet here had them waiting for a rung's
    evidence, which a file nobody has committed does not need.
  - The candidate is not uniformly short, the agent's "short settings" being a sloppy name wink
    challenged: against the built-in defaults `duration` is 20x shorter, `blocks` 10x fewer, and
    `settle_time` and `warm_cap` 15x shorter, while `runs` is doubled and `block_sleep` is 20 to
    100x *longer*. That last one is now 1.27 s of a 1.66 s run against 0.26 s measured, 77% of
    the cost, where at the old 5 s duration the sleeps were 8%. It buys block independence,
    which serves the block-level claim and not the trimmed pair, and the pins hold still most of
    what it re-rolls, so it is the first knob [docs: the overhead floor on the
    7600x](#docs-the-overhead-floor-on-the-7600x) weighs and every number so far was measured
    at its present value.
  - Pinning by name is what makes the file portable and also what makes an unprepared host fail,
    so this rung owes that failure a message: `pin_cpus: invalid CPU id "smt"` named no fix where
    the `[freq]` refusal beside it names `setup-freq`. A spec that is a bare word no `[profiles]`
    entry has is now refused where it resolves, naming the host's file, an entry's shape, and
    what this host declares, since none and a different set are different mistakes.
  - The cost accepted: a checkout still holding an ignored `iiac-perf.md` fails on the pull, both
    carriers present, until that file is removed.
- No warm-up invocation is discarded (wink, 2026-09-19): the 10-50 trim is what drops a
  disturbed run, and a rule that throws data away before the trim sees it is a second trim.
  - The cost accepted: the trim works on the runs of one invocation, so an invocation disturbed
    from end to end, as the second of the pins session was, keeps its shift.

#### Ladder details

##### feat: a shorter trustworthy run on the 7600x opening

The cycle's setup commit: create and publish the bookmark, empty `## Closed`, move the
`Allocate runs and duration for a fixed wall time` entry into this block and reshape it to the
7600X's dial-in, leave its remainder as a Todo entry, bump the version-of-record, and take the dev
name.

##### docs: what a trustworthy run means

Precision alone would accept a config whose error bar is understated threefold, and the model's
parameters are unmeasured on the 2t bench. The rung writes the criterion, the search config in the
TOML carrier, and the analysis script, then measures `a`, `s_p`, and `o` at today's defaults on
`zcr-spsc-v3-2t`, and re-does the `runs` and `blocks` subsetting there rather than assuming the
`min-now` records transfer.

What was done:

- The criterion went to [Checking a bar, not reading
  it](notes/measuring-a-technique.md#checking-a-bar-not-reading-it), since it outlives the cycle:
  a bar breaks when its replicates are not independent or when a level above them moves, and
  neither shows in the bar, so a bar is checked by repetition rather than read.
- `configs/knobs-7600x.toml` states every knob under study at today's value, so the baseline is
  self-describing, and `configs/knobs.py` reads the model out of the records alone. Its `ci95`,
  `lsc`, and `Trimmed` reproduce `src/series.rs`, checked against that file's own worked example,
  so a number here is the number a report would print.
- The baseline is a tight core and a fat tail: 68 of 80 runs inside 93.7 to 96.1 ns, 12 from 98.5
  to 151.6, at run positions 1 through 9. Trimmed, the eight invocations agree to 0.45% where
  plain agree to 1.88%, and the trimmed calibration is 1.54x, marginally optimistic.
- The within-run term needed correcting before the model could be used. Blocks on this bench
  carry a lag-1 of +0.42 even pinned, where `min-now` reads +0.02, so a hundred blocks are worth
  about forty-one. Over the effective count `a` is 0.054 against 0.017 ns^2 s, and `d*` 0.86 s
  against 0.49, so an uncorrected model would have asked for runs half as long as they should be.
- The allocation is wrong in a nameable way, and not in the direction the cycle assumed. At
  today's overhead `d*` is 0.86 s against the 5 s being spent, and the run count is what the
  target wants more of: a trimmed ten-run series claims 0.93%, twenty 0.61%, thirty 0.48%. So the
  shape is more runs and much shorter ones, where the cycle opened expecting to cut everything.
  Five runs is the floor whatever the arithmetic says, since below it nothing can be trimmed and
  a series of three off this host would claim 11.7%.
- The blocks knee sits near 16 on the 2t bench too, 0.27% against 0.11% at a hundred, so the
  `min-now` subsetting did transfer.
- The tail is the host's, not the watching, on a third baseline run with nothing armed against
  it: 10 tail runs of 80 against 12 and 12, a core of 92.6 to 95.9 ns against 93.6 to 96.1, and
  spreads of 0.61% against 0.58% and 0.53%. Twelve percent against fifteen is well inside what
  80 runs resolves. The first two baselines were both polled every 90 s, the second by a watch
  left armed from the first, so they compared polled against polled and the claim had to be
  retracted and re-earned.
  - The clean arm also calibrates at 1.05x, against 1.54x and 1.16x polled, so the trimmed claim
    covers the spread between invocations almost exactly.
  - One difference outlives the count: the 151 ns level, 1.594x and the most damaging of them,
    appeared three times in the two polled arms and not once untouched. Three of 160 against
    none of 80 claims nothing, and it is the one thing that still looks like it could be the
    poll rather than the host, so it is a test to run rather than a conclusion to draw.
- The tail is discrete levels, not disturbance. The 160 polled runs cluster into steps, 85% in a
  93.6 to 96.1 ns core and the rest at 1.037x, 1.083x, 1.12x, 1.16x, 1.21x and 1.594x, and every
  tail run is flat across all 100 of its blocks: the 151.6 ns run holds 150.3 to 152.8
  throughout. Nothing external holds a process at one level for five seconds and then lets go, so
  this is where the ring landed this process start, the run-level replicate the notes already
  name. `suspended_s` is zero in all 240 runs and `inhibit` is on, so sleep is not in it, and the
  tail runs are indistinguishable from the core before they start, at the same `warm_used_s` of
  1.51 s and `settle_s` of 10 ms.
  - So trimming removes level-landers rather than disturbance, and the run count has to sample
    the level distribution, not merely average noise. Whether the step is memory placement or the
    `4,5` pairing of two independent cores is untested, and an SMT pair would separate them.
- Trimming a fifth is not enough against a tail of 15% that clumps. Twelve tail runs over eight
  invocations average 1.5 each, but they arrive in ones and threes, and an invocation of ten
  trims only two from the top: the two worst quiet invocations kept an outlier and claimed 6.6%
  and 6.8% where their siblings claimed 0.6%. The remedy is the same as the target's, more runs,
  since thirty trims six, so the run count buys robustness as well as precision.

##### feat: one record file per invocation, not per run

An invocation of ten runs writes ten files, so the two experiments so far are 480 files, a push
lists hundreds of lines, and a reader opening one file sees a tenth of an invocation and takes it
for the whole (wink, 2026-09-18). The runs are sequential, so nothing needed the split: it came
from a trailing `/` on the `record` key, carried over from the clock experiment by habit. An
inserted rung. Each experiment's records are concatenated in start-time order into one JSONL file,
both configs' `record` keys name a file, the scripts take a file or a directory, and the places
that cite the directories are corrected.

What was done:

- 480 files became two, `records/clock-shift.jsonl` of 240 lines and `records/knobs.jsonl` of 250.
  Every file held exactly one line, so nothing was re-encoded: the lines were joined in name
  order, which is start-time order, each checked to parse, and both scripts' output was diffed
  before and after and found identical.
- One file per experiment, not per condition as first proposed, for the reason in the
  deliberation: `knobs-quiet` had come to name polled runs, and a tag cannot drift that way.
- `records/knobs.jsonl` gained a fourth baseline, ten runs wink made on the 7600X that morning
  with the same config and no `condition` tag. It went in as it was, since a record is evidence
  and is not edited, so it groups under no condition. It is also the only baseline no agent
  was anywhere near.
- The single-file mode was checked rather than assumed: three runs gave three lines, a second
  invocation six, two series, every line parsing. The append never truncates.
- The template's `record` sample and the config reference both showed the directory form, which
  is what led here, so both now show a file and the prose says which to prefer and why.
- `configs/knobs-7600x.toml` was rewritten by `update-config`, which also wrote out `env_probe`
  and `inhibit`, two of the cycle's fixed conditions that had been left at their defaults unsaid.
- The directory mode itself changed at the review, since a trailing `/` was the trap (wink,
  2026-09-18): it wrote a file per run because each run stamped a name from its own start. A
  directory now gets one file per invocation, `<series>-<host>.jsonl`. The runs are separate
  processes that already share the series id, so each arrives at the same name with no plumbing
  from the parent, and a sink with no series, as `suggest-freq` has, uses an id of its own.
  Checked live: three runs gave one file of three lines, and a second command of two benches by
  two runs gave a second file of four. The rung's title changed with it, being unpushed.
- The search's build changes here, from 0.28.17-1 to this rung's, which wink copied to the 7600X
  (2026-09-18). One build for the whole search is a fixed condition, so what differs is stated:
  the name a directory's record file gets, and the template's prose. Nothing that measures
  changed, and every record carries its `version`, so a reader can tell the two apart.
- `x.json`, a schema-3 record from 2026-09-02 tracked at the repo's root by accident, is removed
  (wink). Nothing referred to it.
- The 7600X ran from a hand-copied config that had not had the change, which is why wink's run
  that morning made ten files after the repo's copy had been fixed. A loose copy drifts, and the
  checkout there is far behind, so this will recur until that host runs from a real checkout.

##### docs: the statistics behind a run's claim

`CI95`, `LSC`, `o`, `a`, `s_p`, and `d*` decide every remaining rung, and the last three are this
project's own symbols for standard ideas, so a search for them finds nothing and wink cannot
check the agent's use of them (wink, 2026-09-18). An inserted rung, placed before the port so the
two choices the port would freeze, the autocorrelation correction to `a` and the calibration
thresholds, are reviewed first. One story told from the 7600X baseline, each term with its plain
meaning, its formula, its standard name, a checked link, a figure drawn from the tracked records,
and where it misleads.

- `docs/statistics.md` is the story, `docs/figures/make.py` draws its seven figures as static SVG
  from `records/knobs.jsonl`, and README's Terminology gains the terms, each a line and a link.
  `make.py` is temporary, replaced by [feat: figures drawn by the tool][9].
- Checking the correction before writing it up found it wrong. It assumed a block's correlation
  dies away geometrically, and the measured one does not: +0.42 at lag 1 and still +0.17 at lag
  5, where a geometric decay gives +0.01. Summing the measured correlations to their first zero
  counts a run's 100 blocks as 23, not 41, so `a` was understated by nearly half.
  `configs/knobs.py` now sums them, and `analyze` is to do the same.
  - The lag-1 of +0.02 the first Deliberation entry gives for the 7600X does not hold for this
    bench: every 5 s condition here reads +0.38 to +0.50. We think that figure came from
    `min-now`. Pinned blocks are not independent, and the count above is what allows for it.
- The model's `a` and `s_p` in the write-up come from the 215 normal-level runs, 0.079 ns² s and
  0.54 ns, since an invocation's own `s_p` swings from 0.5 to 1.0 with how many slow runs its trim
  let through. They put `d*` at 0.99 s for the 3.6 s overhead and 0.28 s for 0.3 s.
- The hundred 1 s runs wink ran are appended to `records/knobs.jsonl` as `condition=r100-d1s`.
  22 of them are slow, past the 20% trim's reach, which the write-up shows as the trim's limit.
- The calibration thresholds are not in the write-up, since nothing yet computes them. They are
  reviewed at the `analyze` rung, which introduces them.

##### feat: a lower-band trimmed mean

An inserted rung (wink, 2026-09-18). The 20% trim fails whenever more than 20% of an invocation's
runs land slow, and on the 7600X `zcr-spsc-v3-2t` does that often: four same-code invocations of
30 one-second runs drew 30%, 13%, 33%, and 17% slow runs, and their trimmed means span 1.68%.
More runs do not help, since the trim's limit is a fraction. The settle/sleep experiment (800
records, `settle_time` 1.5 s against 0.1 s, `run_sleep` 1-2 s against 0.1 s, both benches) found
neither knob moves the median or the slow fraction beyond what two reps of one condition differ
by, so the slow fraction is the obstacle and not the overhead.

- The statistic keeps the runs between the 10th and 50th percentile (wink's proposal, 10..60 the
  other candidate). On the four invocations it spans 0.27%, and on the experiment's cells cut into
  invocations of ten it spans 0.82% where the 20% trim spans 2.49%. It needs no threshold, where
  the agent's alternative, the runs within 2% of the median, invents one.
- The band was chosen on the data it was then judged on, so the rung's check is on data that had
  no part in the choice: the same-code pairs of the 25 baseline invocations in
  `records/knobs.jsonl`, the false-alarm rate of its LSC and its spread against the 20% trim's.
- To verify before coding: that Yuen's standard error carries to an asymmetric trim. Timing noise
  being one-sided is established ground, `timeit`'s advice to take the minimum and Chen and
  Revels 2016, and [one-sided contamination and the two-point fit][61] is this project's own
  earlier meeting with it.
- The band sits on the core's lower part, about 0.2 ns under its centre, so the report names it
  as what it is, keeps the plain mean beside it, and prints how many runs sat above the band's
  reach, since a change to the code could move that fraction and the band hides it.

What the rung did:

- `series.rs` gains `Trim`, the shares cut from each end, `"10-50"` by default, and `Trimmed`
  takes one. The standard error divides by the share the series kept. The report's row names are
  unchanged and its `trimmed` line says what it did, `10-50 keeps 4 of 10 runs, drops low: 4,
  high: 1, 5, 6, 8, 9`, after wink read the first wording, a bare list of run numbers, as the runs
  kept.
- The trim is a knob (wink): the `trim_runs` key and `--trim-runs FROM-TO`, named for what it
  trims since a bare `trim` reads as the blocks, its value the edges of the band kept, as wink
  has spoken of it throughout, `"20-80"` the old trim and `"0-100"` none. The agent first
  coded the two numbers as the shares cut, which reads the same at `10-50` and nowhere else, a
  trap a second Claude wink consulted pointed out. It is in the template, the `Config:` list, and
  `init-config`'s line values. The plain rows print whatever it says. It is not in the record, which holds the raw run means, so an
  analysis applies the trim it wants. The docs say to set it once for a project, since a trim
  picked after the numbers flatters them.
- The check on the 25 baseline invocations passed as an estimate and failed as an error bar.
  Same-code pairs land within 0.69% 95 times in 100 where the old trim gives 1.81%, and 13.7% of
  them exceed their LSC where 5% is honest. Shuffling the runs across invocations gives 7.8%, and
  6.3% at thirty runs, so a small part is the formula on four kept runs, and the larger part is
  that a whole invocation sits about 0.3% high or low, which nothing inside one invocation can
  see. The old trim passed only because slow runs inflated its LSC over that shift. wink's go was
  to ship the estimator with this stated: one invocation against one resolves about 0.7% on the
  7600X, and less needs several invocations a side, which [feat: analyze checks a claim across
  invocations][8] is to provide.
- Trimming blocks instead of runs (wink's question) was tested and does nothing, 7.16% against the
  plain 7.06%, since a slow run is slow in every block. Pooling an invocation's blocks estimates
  as well as the run band and would count ten clumps as a thousand independent values. The run is
  the level the contamination arrives at.
- The count of runs above the band's reach is not printed: any count needs a threshold, the run
  lines show them, and [Mark a run that lands on another
  level](#mark-a-run-that-lands-on-another-level) is where a sound one belongs.
- Stigler 1973 was checked for when a trimmed mean is normal at all, its cut points where the
  data is dense. That the standard error's form holds for unequal cuts is the agent's derivation
  from the winsorized variance, not a quoted result, and the shuffle is its test.
- A lead, untested: wink's `--benches=all` on the 3900X, ten 0.25 s runs pinned to the SMT
  siblings 11,23 with `block_warmup` 2 ms and `block_sleep` 100-200 ms, read far steadier than
  anything on the 7600X, `zcr-spsc-v3-2t` within 0.8% with no large slow level. We think the
  siblings are why, both ends of the ring in one core's cache, and wink has not seen the 3900X
  do it before, so it may be a fluke. A repeat there, and a sibling pair on the 7600X, would say,
  and would bear on [feat: the quick config for the 7600x][12]. It also showed the band's weak
  spot: `ice-ps-1t` split its runs five and five between two levels, where one more high run
  would have put a high run inside the band.
- The push went on wink's "commit, it's late", the description unreviewed, this commit only.
- `configs/knobs.py` and the figures follow the new trim, and `docs/statistics.md` tells the
  trim's history, the check, and the 0.7% bar. The settle/sleep experiment it cites is on the
  7600X at `~/iiac-perf/records/settle-sleep.jsonl` and is not yet tracked: the sandbox cannot
  fetch it, so it comes over at [docs: the overhead floor on the 7600x][10].

##### feat: host facts leave the project configs

An inserted rung (wink, 2026-09-19), from a day of getting ready to measure on both hosts. A
config made with `--from iiac-perf.md` carried the 3900X's `[freq]` clamp into a file named for
the 7600X, where it would have pinned and restored to the wrong clock, and the project-local file
was each checkout's own, ignored, so nothing tracked said what a run in this repo is.

- The project-local config is tracked, as `iiac-perf.toml` in the TOML carrier, holding the three
  block keys the ignored `iiac-perf.md` held and no `[freq]`. The ignore rule for `iiac-perf.md`
  goes, since a second carrier beside the tracked one is a hard error and ignoring it would hide
  the cause.
- Two 7600X configs move into `configs/`, their copied `[freq]` taken out.
- Each host's XDG file is where `[freq]` and `[profiles]` live, `smt`, `ccx`, and `x-ccx` by the
  placements rule, so a config that says `pin_cpus = "smt"` runs unedited on both. Those files
  are outside the repo and the install is wink's.
- A pins experiment ran on the 3900X the same day, the cpu pin (`11,23`, `0,12`, none) crossed
  with the clock pin, 8 invocations of 10 runs a cell at 0.25 s and ten blocks. What bears on
  this cycle:
  - The cpu pin is what makes a two-thread bench repeatable, 0.34% and 0.38% between
    invocations on the two pairs, and which pair mattered little. Unpinned, the same bench reads
    twice the time and wanders 12 to 15%.
  - The clock pin buys calibration, not smoothness: pinned, 0 and 3 of 28 pairs of invocations
    break their `LSC trimmed`, and free, 9 and 6. It also makes two cores agree to 0.01% where
    their boost differs by 4%.
  - The short settings claim about 1.1% for `zcr-spsc-v3-2t` and keep it, twice the 0.5% target.
  - `zcr-spsc-v3-1t` sits on three levels that neither pin moves, so its claim is 6 to 11%.
- The experiment's records, tools, and write-up are not in this commit: they wait on disk for
  the experiments repo, a second 3900X session over every profile and a 7600X one with them.
- Three entries went to `## Todo`: an experiments repo, a `setup-host` command, and `init-config`
  printing TOML when asked.

##### feat: a record carries its config and a label

An inserted rung (wink, 2026-09-20), from recording three experiments by hand. A record is the
only complete account of a run, every layer resolved, where a config file is one layer: a config
made outside the repo lost the `block_warmup` the project-local file had been giving. And a
record target's mode hides in a trailing `/`, a directory's file name cannot say what it holds,
and a file target's directory is not created, which cost the first 7600X pins launch.

- The two modes take spellings of their own, `--record-dir DIR` and `--record-file PATH`, where
  the mode hides today in a trailing `/` and `record = "smooth-records"` quietly appends every
  session to one oddly named file. The banner's `record` line names the mode it took.
- A directory's file is `<label>-<series>-<host>.jsonl`, the label defaulting to the bench
  selector as typed, so `iiac-perf ice-rr-2t --record-dir runs/` writes
  `ice-rr-2t-<series>-<host>.jsonl` with no flag at all (wink, 2026-09-20, wanting the selected
  bench's name to lead). `--record-label NAME` overrides it, the selector is sanitized as the
  host and series already are, and a list past the cap collapses to its count while `all` stays
  `all`.
- The name is the index and `ls` is the query (wink, 2026-09-20), so a tag cannot do the label's
  job, and the agent's suggestion to drop `--record-label` in favour of `--tag experiment=...`
  was wrong: it traded away the one property the request was about.
- Neither label needs a schema bump, which is wink's objection turned into the design: the
  default reads `config.params.benches`, already in every record, and `--record-label NAME` is
  sugar for a `label=NAME` tag that the name is then built from, tags being a free-form map the
  record already carries. So `ls` shows the label, a renamed or copied file still knows it,
  `analyze --by label` groups on it, and the name and the data agree by construction rather
  than by discipline.
- The series stays the tool's own, the start and the parent's pid: it is what lets an
  invocation's children agree on a file without coordinating, and what tells `analyze` which
  runs are one invocation (wink proposed a user-set series, the agent the label beside it).
- No file per bench, the agent pushing back on wink's `--record-individual` (2026-09-20): the
  one-bench case is what the default label answers, an experiment wants fewer files and not
  more, `pins.sh` holding 48 to 96 invocations in one on purpose, `all` would write 28 files an
  invocation, and nothing is lost unsplit, every record naming its bench and `analyze` reading
  a directory. A name template carrying `{bench}` is the general answer if the wish returns
  with a case, one flag in place of a fourth mode.
- A file target's directory is created, as a directory target's is.
- The record carries the run config in the form the loader reads, beside the readable one,
  since `pin_freq` and `freq` print for a reader, and the placement label the banner prints,
  `SMT`, so another host can turn `11,23` into its own pair. We think the bench source's commit
  belongs there too, the tool's version being all a record says of what was measured.
- `init-config --from-record FILE` writes a run config from a record: the run keys as they
  resolved, the host's facts left behind, `[freq]`, a clock in MHz, cpu numbers, paths, each
  named by word or profile where the record allows and left out with a comment where not, and a
  short account of where this host differs from the record's, cpu, kernel, rustc, version. Its
  own rung if this one grows too large.
- One schema bump covers the label, the loadable config, and the placement.

##### feat: analyze checks a claim across invocations

Every analysis so far is a Python script that re-implements `src/series.rs`, is validated by
nothing, and reads records by string key, which is the throwaway script [Analyze a directory of
records](#analyze-a-directory-of-records) was written against, and the agent's two scripts had
two bugs caught only by eye (wink, 2026-09-18). An inserted rung taking that entry's cross-run
tier and no more: an `analyze` command over a record file, on `record.rs`'s struct and
`series.rs`'s arithmetic, printing the statistics whose definitions are fixed, its tests
reproducing the baseline's numbers from the tracked records. `configs/knobs.py` goes. The level
clustering stays out, its 0.8 ns gap being ad hoc, and belongs to [Mark a run that lands on
another level](#mark-a-run-that-lands-on-another-level). `configs/clock-shift.py` goes with it,
its comparison an `analyze` invocation.

Grown on 2026-09-20 from the experiments in `../iiac-perf-expr-1`, whose Python is what it has to
replace, printing the same numbers being the check that it can:

- The units are the run, the invocation (a series, ten runs, a trimmed mean and its claim), and
  the group, the invocations that share a tag's value or a bench name. `--by TAG`, more than one
  for a grid, names the groups.
- A group against itself is the first report, the baseline qualified: the spread of its
  invocations' trimmed means, that spread against the claimed `LSC trimmed`, the pairs beyond
  their claim, about 1 in 20 when it is honest, the claim's size as a percent, since a wide one
  is true and of no use, and the trend over the session's order. From the spread between
  invocations comes the change the group could really detect, which a single invocation's claim
  cannot give, drift being invisible to it.
- A against B pairs neighbors when the invocations alternate, found from `t_start`, and takes
  the differences, so a drift cancels. The verdict is detected, not detected, or could not have
  been seen below some percent.
- A ladder, `zcr-spsc-v{n}-2t`, is bench names in one invocation, paired for free: each against
  the first and each against the one before, never every pair, with each version's detectable
  change beside its step.
- Two sides that ran different configs are said to have, from `config.params`.
- Sessions hours apart differ by 0.1 to 0.9% whatever each claims, so a comparison across
  sessions is reported as that and not as a finding about the bench.

##### feat: figures drawn by the tool

An inserted rung (wink, 2026-09-18): wink does not want Python in the repository, and
`docs/figures/make.py` also leans on `configs/knobs.py`, a second copy of `series.rs` kept in step
by hand. A `figures RECORDS OUT_DIR` subcommand draws the write-up's SVGs on `series.rs`'s
arithmetic, a subcommand because the crate is one binary and an example could not reach its
modules. The figures are regenerated and compared with `make.py`'s, then `make.py` is deleted, so
no tracked Python is left when the cycle closes. From here the agent asks `analyze` its questions
of the data, and where it cannot answer proposes extending it.

##### docs: the overhead floor on the 7600x

About 4 s of every 7.2 s run is overhead, and `d*` goes as its square root, so it is cut first.
`settle_time` and `run_sleep` binary-search down, monotone; `block_sleep` and `block_warmup` get a
three-point comparison instead, since a sleep of zero collapses the blocks into one continuous run
and a sleep provokes the ramp the warmup exists to discard, so neither is monotone in stability.

##### docs: the run length at the new overhead

With the overhead cut, the optimum has moved. The rung measures the run length at the new `o`, and
checks the measured `CI95 runs` against what the model predicts, so the model is calibrated rather
than trusted.

##### feat: the quick config for the 7600x

A candidate is not a finding until it survives repetition. The rung runs the acceptance check's
five sessions and the `min-now` confirmation, ships the config, and writes what each knob bought
into the report guide, including what a reader on another host should measure first.

Open since 2026-09-19, for wink: whether the config shipped is `iiac-perf.toml` itself and the
acceptance check retargets to it from `configs/quick.toml`, and which profile it names. The
pins experiment says the profile is what is measured, `smt` about half of `ccx` on both hosts,
and that on the 7600X `zcr-mpsc-v2-2t` already holds a claim of 0.15 to 0.19% on `smt` while
`zcr-spsc-v3-2t` holds 2 to 3% there and 0.56% on `ccx`.

And a profile is chosen on the claim it makes, never on speed, wink's correction of 2026-09-21
to an agent note that had read faster as better. `ice-rr-2t` on the 3900X, ten runs a profile,
clock pinned: `ccx` trims to 1182.975 ns claiming 0.485% and `smt` to 1368.661 ns claiming
0.115%, so `ccx` is 13.6% faster while `smt` detects a change four times smaller, which is the
whole of what a baseline is for. `smt` is the quieter of the two block to block as well, 1.7 to
4.5 ns against 2.5 to 8.5, and its winsorized stdev is 3.6 times smaller, so the margin is not
the trim's doing.

- The mechanism, we think: two threads on one core share L1 and L2 and never cross the L3, where
  two cores in one LLC domain cross it every round trip and inherit whatever else touches it.
- It is not a rule. `smt` is the quieter profile in three of the five bench-and-host pairs
  measured, level in a fourth, and far worse in the fifth, the 7600X's `zcr-spsc-v3-2t`, whose
  `smt` cell smears over four sub-levels at 2 to 3% where its `ccx` cell holds 0.56%.
- A tight claim is not enough by itself: `zcr-mpsc-v2-2t` on the 3900X's `ccx` claims 0.354%
  against `smt`'s 0.721% and then breaks it 7 times in 28, the drifting cell, where `smt` breaks
  it none. Width and honesty are both required, and only repetition shows the second.
- The two `ice-rr-2t` numbers are one invocation each, so their width is measured and their
  honesty is not. The closing needs repetition before it names a profile for the config.

##### feat: a shorter trustworthy run on the 7600x closing

Closing out the cycle.


## Waiting

Important work that cannot start yet. Each entry names what it waits on and its rank once
unblocked, and every opening checks the conditions.

_None._

## Todo

Entries are in priority order, the first highest, and reprioritizing moves the entry. The
long-tail backlog is in [todo-backlog.md](notes/todo-backlog.md), and deeper detail lives in
the frozen `notes/chores/` design subsections, linked by `[N]` refs.

### Placements by name and a cpus command

`--pin-cpus` takes cpu numbers, which differ by host, so a config that pins is a config per host,
and nearly every pinned record here sits on `0,1`, the busy end of both hosts (wink, 2026-09-17,
at the planning of `feat: a run is a config file`). zc-ring-x1 counted the kernel's per-cpu work
on both and decided where a measurement belongs (2026-09-16, its `notes/ring-buffer-design.md`,
"Measurement placements: the base cpu and its partners"): the idlest-cpu search breaks ties
toward the lowest number and a wakeup scans the L3 ascending, so short work piles onto the low
cpus. The 3900X's first CCX carries ten times the timer ticks of its last, and the 7600X's cpu 0
five times the reschedules of cpu 5. Our own pass agrees: `--pin-cpus 2,3` on the 7600x read 2 ns
faster and tighter than `0,1` (in [How often each pinned level comes up on the
7600x](#how-often-each-pinned-level-comes-up-on-the-7600x)).

- a command that lists the cpus: `iiac-perf cpus` prints a row per cpu, its core, its sibling,
  and its L3 group, the kernel's counts from `/proc/interrupts` and `/proc/softirqs` (timer ticks,
  sched softirq, RCU softirq, function calls), and the labels, which cpu is the base and which
  its `ccx`, `x-ccx`, and `smt` partner. No root is needed
- quietness is a rate: the counts are since boot, so they carry whatever ran in that uptime, which
  we think is the 7600X's busy cpu 7. The default is the count over the uptime, and `--for 10s`
  samples a delta, what the host is doing now
- `--pin-cpus` and `pin_cpus` take `ccx`, `x-ccx`, or `smt` as well as numbers (wink,
  2026-09-17), with `--base-cpu N` as the demo has it. The rule is zc-ring-x1's: the default base
  is the last core's primary cpu, partners prefer a primary cpu and among those the highest
  number, and the SMT partner is the base's sibling. That gives `11,10`, `11,8`, and `11,23` on
  the 3900X and `5,4` and `5,11` on the 7600X, the demo's pairs, so a table from here and one from
  the demo sit on the same cpus
- the names resolve by the rule, never by the measured counts: a name means the same cpus every
  day, and the command is the evidence the rule is checked against. A pin that followed the
  counts could differ between two runs of one config
- a name the host cannot satisfy refuses the run, `x-ccx` on the 7600X's one L3, rather than
  falling back
- a pool of more than two needs a rule too, the mpsc v2 pair running over a pool: we think `ccx`
  at four is the base's L3, primaries first, going down from the base, decided at the opening
- a profile is an ordering of this host's lCPUs from the base and not a pair, so a pool of N
  takes the first N, the `ccx` at four rule above generalized (2026-09-20): `ccx` on the 3900X
  is 11, 10, 9 and then the siblings 23, 22, 21, which fills a three-core LLC domain exactly
  at three threads
- a profile that cannot hold N without spilling to the next level refuses the run, the way
  `x-ccx` refuses on the 7600X's one L3. Otherwise one name is two experiments: a 3900X LLC
  domain holds three cores and a 7600X's holds six, so `pin_cpus = "ccx"` at four threads
  spills onto siblings on one host and not the other, silently, and portability is what the
  names exist for. A deliberate spill takes a name of its own, `ccx+smt`
- an asymmetric bench wants a placement per role rather than one pool, an mpsc's consumer
  being worth more than any one producer: [Additional thread
  control](#additional-thread-control)
- the names here are AMD's where [Topology-aware pinning and lCPU
  terminology](#topology-aware-pinning-and-lcpu-terminology) settles on generic ones, lCPU,
  core, cluster, LLC domain, package, and names the same three `smt`, `llc`, and `xllc`. One
  spelling wins before either is built in
- the record and the banner carry the name and the cpus it resolved to
- this takes the auto-profiles bullet of [Topology-aware pinning and lCPU
  terminology](#topology-aware-pinning-and-lcpu-terminology), whose `--pin smt`, `--pin llc`, and
  `--pin xllc` become these names, built-in beside the `[profiles]` a file declares
- the check before the old pins are dropped: the pinned 20-run `zcr-mpsc-v1-2t` command at `4,5`
  on the 7600x against its `0,1` and `2,3` passes, and the same three on the 3900X
- the last rung writes the base configs under `configs/` and runs [Re-record all on the 7600x
  across processes](#re-record-all-on-the-7600x-across-processes) from one, so that entry closes
  with this cycle. The acceptance check is one config, unedited, running on both hosts and
  landing on the demo's pairs

### Additional thread control

A bench's thread count is in its name, `mpsc-1t` and `mpsc-2t`, so nothing runs one at three
threads, and the pin pool is a flat list where an mpsc's consumer and its producers are not
interchangeable (wink, 2026-09-20). The entry read "shape once a concrete bench needs it", and
[mpsc at N threads](#mpsc-at-n-threads) is that bench.

- a thread count on the line and in a config, so one bench name spans a sweep and the count
  rides in the record beside the bench
- placement by role and not one pool: an mpsc is one consumer and N producers, and the
  consumer's placement is worth more than any producer's, so the placements worth comparing are
  ones a flat list cannot say, the consumer alone on a core with its sibling idle, the
  consumer's sibling holding a producer, the producers spread across LLC domains with the
  consumer fixed
- NUMA, when a host with more than one node turns up

### mpsc at N threads

Every mpsc claim here is at one or two threads and the placement vocabulary is a pair, so
nothing says what happens once the producers outnumber the cores (wink, 2026-09-20). The
question is the baseline config's: does a claim stay small and true as N grows, and does
pinning still pay when every lCPU is loaded.

- every mpsc-capable implementation, so the reading is a comparison between implementations and
  not between one and a memory of another: `mpsc`, `zcr-mpsc-v0`, `v1`, `v2`, `cb-chan`,
  `cb-seg`, and the iceoryx2 pair `ice-ps` and `ice-rr`, whose publisher and client sides are
  the many (wink, 2026-09-20, the agent having left iceoryx2 out)
- N over the interesting ground and past it: 2, 3, 6, 12, 24, then twice, three times, and four
  times the host's lCPUs, 48, 72, and 96 on the 3900X and 24, 36, and 48 on the 7600X. Not
  1000, which measures a scheduler and not a channel (wink, 2026-09-20)
- pinned against unpinned at every N, since "unpinned is poor" was measured at two threads on
  four LLC domains, and we think it stops holding once the placement noise averages over many
  threads
- what to expect, written before the data: a profile name stops meaning anything once every
  lCPU is loaded, the claim widens, and the bench stops measuring a hop and starts measuring
  queueing. The number stays comparable against itself, which is all a baseline asks
- the scale wants designing rather than running: six implementations at eight counts is minutes
  of bench per invocation, so a pinned and unpinned sweep of a few rounds is hours
- it needs [Additional thread control](#additional-thread-control) first, and the observational
  half of [Topology-aware pinning and lCPU
  terminology](#topology-aware-pinning-and-lcpu-terminology) is what reads placement back once
  N is past controlling it
- it runs in the experiments repo as a directory of its own

### An experiments repo

Experiments have been tracked here, config, script, records, and write-up, so the tool's history
carries every record file for good, 4.7 MB so far with a 7 MB session waiting, and the analysis
Python wink does not want in this repo keeps arriving with them (wink, 2026-09-19).

One sibling repo, `../iiac-perf-expr-1` (wink, 2026-09-20), holds a directory per experiment,
each self-contained: its README the question, the method, and a section per host session, its
config and script, its analysis, and its sessions' files.

- Three wait there, not yet under version control: `smooth/`, the all-bench run that raised
  the question, `pins/`, with two 3900X sessions and a 7600X one recorded and written up, and
  `claim/`, a config not yet run. A session is a records file and a screens file named for the
  experiment, the host, and the start.
- Creating the repo is wink's to start, a `vc-x1 init` and its agent-files.
- What stays here: `records/knobs.jsonl` and `records/clock-shift.jsonl`, which the docs cite
  and which are already in history, and whatever slice of an experiment a test reads.
- `analyze` replaces an experiment's Python once it can print the same numbers, which is the
  check that it can.

### A setup-host command, the pin profiles first

The template's `[profiles]` are a 3900X's, `smt = "0,12"`, `ccx = "0,1"`, `ccd = "0,6"`, so a
fresh config on any other host starts from cpu numbers that are wrong there, and on this one from
the busy end (wink, 2026-09-19). `setup-freq` already answers the same problem for `[freq]`, the
live host read and its table written to the XDG file, and nothing does it for the rest.

A `setup-host` command writes every table that is the host's own, `[profiles]` first, with
`setup-freq` becoming its `[freq]` part. It follows [Placements by name and a cpus
command](#placements-by-name-and-a-cpus-command), whose rule picks the cpus.

- The facts are already probed: `host.rs` reads each cache index's `shared_cpus`, L1's naming the
  SMT siblings and L3's the CCX, so the pairs are derived and `lscpu -e` is no longer the advice.
- The cpus come from the placements rule, the base the last core's primary cpu, never from a rule
  of this entry's own, so a written profile and a built-in name land on the same pair: `11,23`,
  `11,10`, and `11,8` on the 3900X.
- The open question is what a written `[profiles]` is for once `smt`, `ccx`, and `x-ccx` resolve
  built-in. We think it is the rule's answer made visible and editable, a pool of four or a
  base other than the default, and that a host whose file has none loses nothing.
- A profile the host cannot form is left out with a comment saying why, `x-ccx` on the 7600X's
  one L3, never written with a wrong pair.
- A `ccd` profile is a fourth placement and not a rename of `x-ccx`, the agent's error of
  2026-09-19 corrected the next day. On the 3900X, Zen 2, a CCD holds two CCXs, so `11,8`
  crosses a CCX inside CCD 1 where the template's `0,6` crosses the dies, and from base core 11
  the cross-die partner is core 5. Sysfs cannot see it here, `die_id` reading 0 on every cpu and
  `cluster_id` unset, so the profile would come from knowing that Zen 2 puts L3 groups 0-1 on
  one die and 2-3 on the other, which is a guess on the next host. What the hop costs was
  measured on 2026-08-01 and equals `x-ccx`, 1.6 ns apart against a 2 ns LSC, so the L3
  boundary is the only fabric tier that matters on Zen 2 ([Topology-aware pinning and lCPU
  terminology](#topology-aware-pinning-and-lcpu-terminology)). It is written by hand because
  the host file is where a fact no probe can find belongs, and a fresh run of it would be a
  confirmation and not a finding.
- The loader enforces the split the files already have by location (wink, 2026-09-20): a
  `[freq]` or `[profiles]` table outside the host's file is refused, with a documented override
  for a benchmark directory that holds its own clock, which `docs/config.md` blesses today as
  "replaces the XDG one whole" and the template with it. A warning first if an error would stop
  what runs today.
- The output follows `init-config`'s rule, TOML without the prose when the XDG file is
  `config.toml` and the markdown form otherwise, creating a missing file, appending a missing
  table, and leaving a declared one alone, as `setup-freq` does.
- The config's values are of three kinds, and only the first is this entry's: a host fact that
  can be read (`[freq]`, `[profiles]`), a host value that must be measured (`pin_mhz`,
  `settle_time`, `warm_cap`, `blocks`, `duration`, `runs`), and a default that is the same
  everywhere (`decimals`, `band_labels`, `trim_runs`). The measured kind is a calibration
  command, what `feat: a shorter trustworthy run on the 7600x` works out by hand, and is an
  entry of its own once that cycle says what to measure first.

### init-config prints TOML when asked

A bare `init-config` prints the markdown form whatever becomes of it, so `init-config > x.toml`
writes 221 lines of prose into a `.toml` name, a file the loader refuses, where
`init-config x.toml` writes the 60-line TOML form (wink, 2026-09-19, at
`config-iiac-perf-7600x-default.toml`). The form follows PATH's extension and the standard
output has none.

- A redirected standard output gets the TOML form (wink, 2026-09-19): the prose is for a reader
  at a terminal, and a redirect is a file being made, whose likeliest name ends in `.toml`. A
  terminal still gets the markdown form.
- A `--format md|toml` flag says it outright (wink, 2026-09-19, "might be a good idea"), so
  `init-config --format md > queue.md` is still possible, and the flag wins over both the
  terminal test and PATH's extension. We think a flag that disagrees with PATH's extension is
  an error rather than a file whose name lies, decided at the opening.
- What `update-config` writes is unchanged, the form of the file it rewrites.

### Shape notes/ops.md by topic

`notes/ops.md` reads as a hodgepodge in no order (wink, 2026-09-18, at the closing of `feat: a run
is a config file`). It is built by accretion: each close-out appends the facts that outlive its
cycle, so the order is the order the cycles ran and nothing regroups them.

- one section per topic, the facts under it kept and dated as they are: the hosts and what is
  installed where, the agent's sandbox and ssh, the clock and its declarations, where run records
  are kept, and measurement gotchas
- a fact that a later one corrected is folded into the correction, the date kept, rather than
  standing beside it
- the close-out step that files a fact says which section it goes under, so the shape holds

### Re-record all on the 7600x across processes

The 7600x's `all` rows were recorded one process for every bench, so each row carries whatever
placement that process drew (split from `feat: CI95 and LSC across processes` at its opening, the
cycle making each bench its own runs). The first use of the cycle's runs.

- `all` with `--record` into a directory that stays, whose records' host block starts the
  cross-host comparison
- a run of the mpsc v1 pair there, whose `all` rows are the renamed v0 rows
- run from a base config as the last rung of [Placements by name and a cpus
  command](#placements-by-name-and-a-cpus-command), on the quiet end rather than `0,1`

### Ring geometry on the line for the zcr benches

The zcr benches fix `CAPACITY` at 8 and `SEGMENTS` at 2 as constants that size `[u8; N]` region
types, so no run can ask for a depth or a segment count, and both the one-way benches and the
segment switch sweep want them on the line (wink, 2026-09-17, at the planning of `feat: a run is
a config file`). Split out of [One-way zcr benches, producer-only and
burst](#one-way-zcr-benches-producer-only-and-burst), whose "depth as a run knob" bullet this is,
because it changes every existing zcr bench and can be checked against numbers we already have.

- `--depth` and `--segments` for the spsc and mpsc benches, each with a config key, since by
  then every flag has one
- the regions become runtime-sized line-aligned allocations, leaked as today
- a ring that is not segmented refuses `--segments` rather than ignoring it, and a depth or a
  count the ring's `init` would reject is an error with the flag's name in it, not an `expect`
- the children get both on their command line like the rest, and the record carries the geometry
- the acceptance check: the round-trip spsc v3 and mpsc v2 at one segment, the no-switch baseline
  the `SEGMENTS` comment promises, against v2 and v1 at the same depth, and every zcr bench at
  the default geometry within `LSC runs` of its reading before the change

### Tags scoped to a bench

`[tags]` and `--tag` mark every record of a run alike, with nothing that says which bench a tag
is about, and the tags we expect first are about the spsc and mpsc benches (wink, 2026-09-17, at
the `tags` decision of `feat: a run is a config file`). Wait to see how the run-level tags get
used before building this.

- a `[[bench_tags]]` list, each entry a `bench` and a nested `tags` table, so no tag name is
  reserved and an unknown key is still an error
- `bench` matches as a bench name on the line does: exact, then prefix, then regular expression.
  A later entry wins on a shared key, and a bench tag wins over a run tag
- a value the bench knows itself, its ring depth or segment count, belongs in the bench's record
  rather than in a tag someone types, which [Ring geometry on the line for the zcr
  benches](#ring-geometry-on-the-line-for-the-zcr-benches) already plans
- open: whether the line gets a form too, since [Config keys stay
  CLI-settable](#config-keys-stay-cli-settable) asks for one

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
- **depth as a run knob**: both benches want depth on the line, the demo's finding living on that
  axis, and the knob is [Ring geometry on the line for the zcr
  benches](#ring-geometry-on-the-line-for-the-zcr-benches), which runs first. Over the segmented
  v3 the same knob is the segment size M, M=1 the boundary's worst case, and the sweep over 1, 2,
  4, 8, and 64 the
  amortization check: fit cost against base plus boundary over M, and the M=1 point on or off the
  line says whether that path has a cost of its own, the cold four-line header being the suspect
- the histogram is the second view: at M=64 the boundary is 1.6% of messages and lands in the n2
  band, at M=8 it is 12.5% and lands at p90, so one run at a realistic M shows the boundary cost in
  place
- the paced one-way bench, the Disruptor's latency test with a TSC stamp in the message and a
  consumer-side histogram, answers what latency the thread sees at a given interrupt rate. It
  needs pacing and a consumer-side probe, which the probe-style benches have the bones of, and is
  a second entry once these two land
- **the producer's side against the consumer's, same thread** (wink, 2026-09-16, after `feat:
  spsc v3 and mpsc v2 benches`): a 1t bench that reserves and commits N times, then reserves and
  releases N times, to see which side pays v3's same-thread gap over v2, three to six times, and
  mpsc v2's over v1, 1.8 times
- names follow the pair convention and are decided at the opening, so the prefix runner covers a
  version's four benches with one word
- ranked ahead of the rest: the numbers feed zc-ring-x1's segmented queue, which may start as a
  v3, and no bench today separates the producer's side of the seam from the consumer's, which the
  cross-host reversal needs. Behind the rename and the partition merge since 2026-09-11, so the
  new benches are written in the merged hierarchy's words

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
  unpinned shift `feat: a run is a config file` is chasing with its clock experiment
- it runs after [Ring geometry on the line for the zcr
  benches](#ring-geometry-on-the-line-for-the-zcr-benches) and [One-way zcr benches, producer-only
  and burst](#one-way-zcr-benches-producer-only-and-burst), which give it `--segments`,
  `--depth`, and its shapes, and from a config on the pairs [Placements by name and a cpus
  command](#placements-by-name-and-a-cpus-command) names
- the two-level runs of `feat: spsc v3 and mpsc v2 benches`, 11.8 or 12.8 ns on the 7600X and
  17.0 or 19.2 on the 3900X, are read here as a placement finding of their own
- the 7600X's SMT pair, where v3 lost across threads with runs from 51 to 67 ns, is rerun at ten
  runs, and at the 3900X's SMT pair, before it goes in a message
- zc-ring-x1 has not been told the findings of `feat: spsc v3 and mpsc v2 benches`, on purpose:
  they are a symptom so far, and the message waits until it carries a diagnosis and the commands
  that reproduce each table on either host. What there is sits in the report guide's spsc v3 and
  mpsc v2 paragraphs

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

### A shorter run on the 3900X, and the allocation hint

What `feat: a shorter trustworthy run on the 7600x` leaves: the same dial-in on the noisier
host, and the hint the moved entry proposed (wink, 2026-09-18, at that cycle's opening).

- the 3900X, whose unpinned block means carry a lag-1 autocorrelation of +0.78 against the
  7600X's +0.02, so its blocks are not the independent replicates `CI95 blocks` assumes and its
  floor will sit higher. Pinning takes it to +0.20, which is the first thing to measure
- an allocation hint once the 7600X cycle calibrates the model: every invocation already knows
  `a` from the blocks, `s_p` from the runs, and `o` from wall time minus measured time, so the
  summary could print the run length and count that would reach a target `LSC runs` soonest
- the model's two weak points, from the moved entry: the within-run term is white only where the
  `resolution` row shows no drift, and rare levels make the run means a mixture, whose spread a
  20-run invocation samples unreliably ([Mark a run that lands on another
  level](#mark-a-run-that-lands-on-another-level)), so the count may need to cover the rarest
  level that matters, not only the variance

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
- until then the arms are two invocations alternated by a script, told apart by a tag, as
  `pins.sh` in `../iiac-perf-expr-1` alternates its conditions, and `analyze --by` pairs the
  neighbors (2026-09-20). The pins and smooth sessions put the drift this cancels at 0.1 to 0.9%
  between sessions and about 0.5% within an hour on the 3900X

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
- an analysis has choices of its own, what to group by, the pairing, the trim band, the target
  claim, and they want a config once `analyze` has shown which ones every experiment passes
  (wink, 2026-09-20). `trim_runs` is the first, a run key that changes nothing about how a run
  executes
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

### Config search up the parents, arms, and a pin's boost

What `feat: a run is a config file` left of the `--config` entry it was opened from (wink,
2026-09-05 and 2026-09-14).

- the project-local search up the parents went into the cycle after all, as its rung `feat:
  iiac-perf.md is found up the parents`
- with spawning, a config also names the children's knobs, and an A/B is two configs or one with
  two arms, which is the shape a cross-host comparison wants. The cycle runs each bench's runs back
  to back, not interleaved (wink, at the opening of `feat: CI95 and LSC across processes`)
- a boost option for a pin that should keep boost on, if benchmarking wants one. A benchmark
  directory's config pinning the clock became the run key `pin_freq` in `feat: config and setup`,
  since a project-local `[freq]` with `min_mhz = max_mhz` also moves where a restore returns

### Config keys stay CLI-settable

Adopt the convention that every run-parameter config key has a CLI flag with CLI-wins
precedence, the flag landing in the same commit as the key, so any experiment is runnable
one-shot without editing a file (wink, 2026-08-17).

- already true today: duration, band_labels, decimals, settle_time, warm_cap, and the pin
  target all pair a key with a flag, and a `--pin` profile only names a core spec that also
  passes raw, so the convention mostly writes down existing practice
- naming the config file belongs to the same convenience (wink, 2026-09-21): a positional
  argument ending in a carrier's extension is already the config, in any position, so
  `iiac-perf min-now quick.toml` works today, and `-c` is unclaimed and becomes the short alias
  for `--config`, which differs from a positional by completing the extension and searching the
  parents and then the XDG directory
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
- auto profiles derived from the discovered tree, so one command line is portable across
  hosts: moved to [Placements by name and a cpus
  command](#placements-by-name-and-a-cpus-command), as `ccx`, `x-ccx`, and `smt`
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

# References

[1]: #feat-a-shorter-trustworthy-run-on-the-7600x-opening
[2]: #docs-what-a-trustworthy-run-means
[3]: #feat-one-record-file-per-invocation-not-per-run
[4]: #docs-the-statistics-behind-a-runs-claim
[5]: #feat-a-lower-band-trimmed-mean
[6]: #feat-host-facts-leave-the-project-configs
[7]: #feat-a-record-carries-its-config-and-a-label
[8]: #feat-analyze-checks-a-claim-across-invocations
[9]: #feat-figures-drawn-by-the-tool
[10]: #docs-the-overhead-floor-on-the-7600x
[11]: #docs-the-run-length-at-the-new-overhead
[12]: #feat-the-quick-config-for-the-7600x
[13]: #feat-a-shorter-trustworthy-run-on-the-7600x-closing
[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[75]: /notes/chores/chores-05.md#settle-time-is-not-a-grade
[84]: /notes/chores/chores-06.md#docs-experiment-in-the-local-agent-files
