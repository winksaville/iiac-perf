# Todo and cycle record

This file contains near term tasks with a short description and reference links to more details.
Its shape is [Todo format](agent-data/notes.md#todo-format).

## Continuation notes

Where the agent was, for the agent that comes next: working copy state, the step in flight, an
open question. Ephemeral, never a record. Written before a restart or when a session is about to
lose context, read first at acquaint, acted on, each fact filed into its home or its bullet kept, and
the rest reset to `_None._` by the reader.

- Messages: `m-7-1`, `m-7-2`, and `m-7-3` are pending for iiac-perf, and the reply is owed: one
  message, all three accepted, our two claims, and the v3 and v4 numbers from the remeasure, with
  sha-links into `main` (wink, 2026-09-28). `feat: spsc v4 benches` has landed, so nothing blocks
  it. Its draft, `tmp/m-7-4-draft.md`, was not found at the 2026-10-04 acquaint, this checkout
  having no `tmp/`, so the reply is written again unless wink has the draft.

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. The block's
shape is the specimen in [cycle-model.md](agent-data/cycle-model.md), and the rules are in
[The In Progress block](agent-data/notes.md#the-in-progress-block).

### feat: mpsc v3 benches

#### Problem

zc-ring-x1's `mpsc::v3` is v2's segmented ring made attachable: a control block in the region, roles
claimed by count, a segment mode chosen at compile time, `Single` or `Multi`, and a wake type,
`NoWake` or `Futex`, by which a full producer or an empty consumer sleeps. Nothing here measures it,
so what the control block costs against v2 is unknown, and so is what the two type parameters buy:
whether `Single` is faster than `Multi` over one segment, and what a ring that can sleep pays when
nobody does (wink, 2026-10-04).

#### Solution

Twelve benches, the 1t and 2t round trips of `zcr-mpsc-v2` over a v3 ring in each combination of
three segment shapes, `Multi` over two segments, `Multi` over one, and `Single`, and two wakes,
`NoWake` and `Futex`, against zc-ring-x1 at `83ab431`. They are measured in one session per host
and compared by `analyze`: v3 against v2, `Single` against `Multi` over one segment, and `Futex`
against `NoWake`. A note states the differences and what they say about keeping the mode and the
wake as type parameters, and zc-ring-x1 is told.

#### Acceptance check

- All twelve benches run to a report, and every segment-switch count they print reads zero.
- `analyze` compares, over five invocations at each placement the host declares, and each result
  is recorded, whatever it says:
  - `zcr-mpsc-v2-1t` and `-2t` against `zcr-mpsc-v3-1t` and `-2t`
  - the `-1seg` pair against the `-single` pair
  - each `NoWake` bench against its `-futex` twin
- A note in `notes/` holds the three comparisons as tables and answers, with its reasons, whether
  the mode and the wake earn their place, and a message carrying it is sent to zc-ring-x1.

#### Ladder

- [feat: mpsc v3 benches opening][85] (done)
- [feat: the mpsc v3 bench pair][86] (done)
- [feat: the mpsc v3 single and one-segment pairs][87] (done)
- [feat: the mpsc v3 futex pairs][88] (done)
- [docs: what mpsc v3's mode and wake cost][89]
- [feat: mpsc v3 benches closing][90]

#### Deliberation

- Multi-step, and every combination: the pair against v2 first, then `Single` against `Multi` over
  one segment, then the least a futex costs, then whether the mode and the wake are needed at
  all, the cost being so low that the simpler ring is the right one (wink, 2026-10-04).
- Twelve benches, not a subset: three segment shapes by two wakes by 1t and 2t. Two segments under
  `Multi` is the v2 comparison, `SEGMENTS` being 2 in both, and one segment under `Multi` is the
  `Single` comparison, the geometry then equal and only the mode differing.
- The least a futex costs is a ring over `Futex` on which nobody sleeps: the benches wait by
  spinning, `policy::spin` on a send and a spin closure on `reserve_slot_with`, so the difference
  from the `NoWake` twin is the wake checks left on the message path, the producer's look at the
  waiting flag after its commit and the consumer's fence at each half segment of releases. The
  cost of a sleep and a wake is another question and is not measured here.
- Names follow `zcr-mpsc-v2-2t-nop`, the variant after the thread count: `zcr-mpsc-v3-2t`, then
  `-1seg`, `-single`, `-futex`, `-1seg-futex`, and `-single-futex`, and the same six at `1t`.
- One generic bench per thread count, over the mode, the wake, and the segment count, each name an
  instance of it, so the six differ in their type arguments and in nothing a copy could let drift.
- The dependency moves to zc-ring-x1 `83ab431` at the first work rung: v3's closure sends were
  dropped and `send` takes a policy after `8508227`, the locked revision. Every commit between
  the two is titled as mpsc v3 work.
- The report guide is not touched: its tables are one session of one binary per host, and rows
  from another session beside them is what the last cycle removed. The comparisons go to a note,
  and v3 joins the guide at its next remeasure. The agent's recommendation, not yet confirmed.
- Hosts: the 3900X for certain. Whether the 7600X and the Pi 5 run the session too is open, the
  7600X's quieter floor being what a difference near zero wants.

#### Ladder details

##### feat: mpsc v3 benches opening

The cycle's setup commit: create and publish the bookmark, delete `## Closed`'s contents, write
this block, bump the version-of-record, and rename the artifact to its `-dev` name.

##### feat: the mpsc v3 bench pair

Nothing measures mpsc v3. `zcr-mpsc-v3-1t` and `zcr-mpsc-v3-2t` are v2's two benches over a v3
ring, `Multi` over two segments with `NoWake`, its roles claimed by count, with the dependency moved
to `83ab431`, and `analyze` compares them with v2's.

- Each bench is generic over the mode, the wake, and the segment count, and `run_as` is its entry
  under a name and a title, so the next two rungs add registry entries and no bench code.
- The waits are zc-ring-x1's `policy::spin` on both sides, where v2's benches pass a closure of
  the same body, v3's `send` taking a policy and having no `send_with`.
- A v3 ring's roles are claimed by count and never released, the ring handle dropped at
  construction as spsc v4's is, so four `// OK:` sites per bench are v2's and the two claims are
  new.
- The pool constructor takes its segment count, every other ring passing `SEGMENTS` as before.
- The dependency move changes mpsc v3 alone, and takes the crate from 0.18.5 to 0.19.2.
- This checkout's rustc is 1.99.0 where the last cycle's sessions built with 1.98.1, so the
  measuring hosts' compilers are checked before the session.
- On the host this rung was written on, an i5-1135G7 laptop, both benches report with every
  switch count zero at `--pin-freq=no`. The default run stops there, `pin_freq: 0 MHz is outside
  cpu0's range`, since `intel_pstate` gives no nominal frequency and `setup` wrote a `[freq]`
  with no `pin_mhz`. Not this cycle's, and a `## Todo` entry or a bug for wink to place.
- The comparison on that host, wink's run, five invocations of the four benches at
  `--pin-freq=2400 --pin-cpus smt`, CPUs 3 and 7:
  - `zcr-mpsc-v2-1t -> zcr-mpsc-v3-1t`: 22.64 to 22.57 ns, `not seen, below 1.72%`.
  - `zcr-mpsc-v2-2t -> zcr-mpsc-v3-2t`: 120.55 to 112.21 ns, `detected` at -6.92% against a 1.21%
    claim.
  - A laptop's claims are wide, and the measuring hosts' session at the docs rung is the one the
    note reports.

##### feat: the mpsc v3 single and one-segment pairs

Whether `Single` buys anything over `Multi` is unknown, since the pair above has two segments and
`Single` cannot. The `-1seg` and `-single` pairs are the same ring over one segment in each mode,
and `analyze` compares them.

- Four registry entries and no bench code: `-1seg` is `Multi` over one segment and `-single` is
  `Single`, each at `1t` and `2t`, an entry point per name beside the pair's own.
- The registry's thread-count test read a name ending in `-1t` as one thread and every other as
  two, so it now takes `-1t-` inside a name too, the variant staying after the thread count as
  `zcr-mpsc-v2-2t-nop` has it.
- On the laptop, five invocations of the six benches at `--pin-freq=2400 --pin-cpus smt`, every
  switch count in the 300 runs zero:
  - `zcr-mpsc-v3-1t-1seg -> zcr-mpsc-v3-1t-single`: 22.44 to 20.51 ns, `detected` at -8.62%
    against a 0.04% claim.
  - `zcr-mpsc-v3-2t-1seg -> zcr-mpsc-v3-2t-single`: 111.85 to 114.73 ns, `detected` at +2.57%
    against a 0.80% claim, `Single` the slower.
  - `zcr-mpsc-v3-1t -> zcr-mpsc-v3-1t-1seg`: 22.44 to 22.44 ns, `not seen, below 0.06%`, and the
    `2t` pair `not seen, below 0.65%`, so `Multi`'s second segment costs nothing seen when no
    switch happens.
- The two `Single` results point opposite ways, and neither is yet the mode's: `Single` and
  `Multi` are different code at different addresses, and with this build's settings layout alone
  moved the median bench 0.39% and single benches up to 8% ([notes/build.md](notes/build.md)).
  Both differences are inside that, so the measuring hosts' session, and whether the signs
  repeat across hosts and placements, is what the note rests on.

##### feat: the mpsc v3 futex pairs

What a ring that can sleep costs when nobody sleeps is unknown. A `-futex` twin of each of the six
benches is the same ring over `Futex`, still waiting by spinning, and `analyze` compares each with
its twin.

- Six registry entries and no bench code, each `NoWake` bench's ring over `Futex` at its 10 ms
  default timeout, which never runs, nobody sleeping. That makes the twelve.
- `Futex` is Linux's, as this crate's pinning is, so the entries carry no `cfg`.
- On the laptop, five invocations of the twelve at `--pin-freq=2400 --pin-cpus smt`, every
  switch count in the 600 runs zero, each row a bench against its `-futex` twin:

  | bench | `NoWake` ns | `Futex` ns | d% | claim% |
  |---|---|---|---|---|
  | `1t` | 22.46 | 23.31 | +3.82 | 0.07 |
  | `1t-1seg` | 22.45 | 23.32 | +3.86 | 0.05 |
  | `1t-single` | 20.50 | 21.11 | +2.98 | 0.07 |
  | `2t` | 114.33 | 117.98 | +3.19 | 0.50 |
  | `2t-1seg` | 114.21 | 118.22 | +3.50 | 0.21 |
  | `2t-single` | 115.16 | 116.31 | +0.99 | 0.38 |

  - All six are `detected` and all six say `Futex` is the slower, by 0.6 to 0.9 ns at one thread
    and 1.1 to 4.0 ns at two. One sign in six builds of different code is more than layout is
    likely to give, so we think the checks cost about 3 to 4% of a round trip here.
- The modes again, now twice each, `-1seg` against `-single` under each wake:
  - At one thread `Single` is faster under both, -8.67% and -9.45%, and the earlier session's
    -8.62% makes three. We think that one is the mode's.
  - At two threads the sign does not hold, +0.83% under `NoWake` and -1.62% under `Futex`, after
    +2.57% in the earlier session, so no difference between the modes is shown there.
- The `2t` level moved between this session and the last, 111.9 to 114.3 ns for `zcr-mpsc-v3-2t`
  from binaries a rung apart, which is the size of the differences being read at two threads.

##### docs: what mpsc v3's mode and wake cost

The comparisons are in records and nowhere a reader finds them. A note in `notes/` holds the
session's three tables and the answer to whether the mode and the wake earn their place, and a
message tells zc-ring-x1.

##### feat: mpsc v3 benches closing

Closing out the cycle.

## Waiting

Important work that cannot start yet. Each entry names what it waits on and its rank once
unblocked, and every opening checks the conditions.

_None._

## Todo

Entries are in priority order, the first highest, and reprioritizing moves the entry. The
long-tail backlog is in [todo-backlog.md](notes/todo-backlog.md), and deeper detail lives in
the frozen `notes/chores/` design subsections, linked by `[N]` refs.

### Help per command word

Wanted for `analyze --help` to show only `analyze`'s documentation (wink, 2026-09-21), for every
command word, since the mechanism is one table either way. An inserted rung of `feat: analyze
checks a claim across invocations` not reached before its closing.

`iiac-perf-dev analyze --help` prints every flag of every command, since clap sees a command word
as a bench name, and the listing is long enough to bury the few that apply (wink, 2026-09-21, at
`feat: analyze qualifies a group against itself`). With a command word on the line, `-h` and
`--help` print that command's description and only the flags it reads, and the same for every
command word: `analyze`, `init-config`, `update-config`, the freq commands, `describe-record`,
`qualify-environment`.

- One table maps each command word to the flags it reads, and clap renders the help with the
  others hidden, so a flag added to a command is added in one place.
- A plain `-h` lists the commands, a line each, and says `iiac-perf <command> -h` shows one, since
  the full listing is too long to find anything in (wink, 2026-10-01). We think the full listing
  stays reachable, `--help` or a `help all`, for whoever wants every flag at once.
- `iiac-perf <command> -h` prints that command's description and only the flags it reads, the run
  path's own flags included for a bench name (wink, 2026-10-01).
- A test checks every command word has an entry and every flag it names exists.

### The quick config for the 7600x

Not done. No shorter run has been shown trustworthy on any host: the cycle named for it, `feat: a
shorter trustworthy run on the 7600x`, landed its tooling only, and its acceptance check was never
run (wink, 2026-09-21). `iiac-perf.toml` holds a shorter candidate whose values are unjustified,
and the one measurement of it, on the 3900X, claims about 1.1% for `zcr-spsc-v3-2t`, twice the
target.

Unblocked when `feat: analyze checks a claim across invocations` landed, since every step here is
judged by `analyze`'s numbers, and promoted from `## Waiting` at `feat: spsc v4 benches`'s opening.
It needs `configs/knobs.py`'s allocation model, `a`, `o`, `s_p`, and `d*`, which that cycle did
not port and left in the script: its first rung ports the model into `analyze`, or runs the
script. Split
from `feat: a shorter trustworthy run on the 7600x` at its closing (wink, 2026-09-21), whose
problem, acceptance check, measurement deliberation, and three unstarted rungs moved here as they
stood. That cycle's closed record, in the landmark's `## Closed`, holds the tooling it rests on.

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

#### Deliberation

- The 7600X alone, pinned, on its quiet cpus (wink, 2026-09-18). It is the quieter host, so its
  floor is the one that exists; the 3900X's is [A shorter run on the 3900X, and the allocation
  hint](#a-shorter-run-on-the-3900x-and-the-allocation-hint).
  - Pinning is also what makes the blocks independent: lag-1 +0.02 on the 7600X either way, but
    +0.78 unpinned against +0.20 pinned on the 3900X. A floor found on dependent blocks would be
    a floor under a mis-stated error bar.
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
  - Not settled (wink, 2026-09-21): the two polled baselines were never compared with an unpolled
    one on purpose, so the rule rests on a suspicion, and [Measure whether polling disturbs a
    run](#measure-whether-polling-disturbs-a-run) is to test it. Until then it holds as a
    precaution.
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
- No warm-up invocation is discarded (wink, 2026-09-19): the 10-50 trim is what drops a
  disturbed run, and a rule that throws data away before the trim sees it is a second trim.
  - The cost accepted: the trim works on the runs of one invocation, so an invocation disturbed
    from end to end, as the second of the pins session was, keeps its shift.

#### Rungs

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

### Reading analyze's reports

An inserted rung of `feat: analyze checks a claim across invocations` not reached before its
closing (wink, 2026-09-21): each rung documented its flags, and nothing says how to read what
`analyze` prints. A section of `docs/report-guide.md` on the group report and the comparison:
every column, `calib`, `detect%`, and `trend%` with what a good and a bad value look like, the four
pairings and why one each is the weakest, the verdicts and why `not seen` is no finding of no
difference, and the layout caution, with the control runs as the worked example. The README's
Comparing section points at it.

### Port make.py's figures and retire the Python

What `feat: analyze checks a claim across invocations` left undone, its acceptance check's two
failures (2026-09-21). `docs/figures/make.py` still draws the seven figures of
`docs/statistics.md`, on `configs/knobs.py`'s copy of `series.rs`, and the three scripts,
`knobs.py`, `clock-shift.py`, and `make.py`, are still tracked.

- The seven figures go into `iiac-perf figures` beside the block means, drawn on `series.rs`'s
  arithmetic and compared with `make.py`'s, then `make.py` is deleted.
- `knobs.py`'s allocation model, `a`, `o`, `s_p`, and `d*`, is ported into `analyze` or goes with
  the script, whichever [The quick config for the 7600x](#the-quick-config-for-the-7600x) needs
  first, and its calibration is already `analyze`'s `calib`.
- `clock-shift.py`'s sleep question is `analyze --compare condition`, and its clock correlation
  was left to the GHz column (wink), so it goes as it is.
- Then `git ls-files '*.py'` prints nothing.

### Known-cost variants, a positive control for detection

Begun in `feat: analyze compares two sides` (2026-09-21): `zcr-mpsc-v2-2t-nop` and
`zcr-mpsc-v2-2t-store-seqcst` exist, in `src/benches/zcr_mpsc_v2_2t_ops.rs` beside the untouched
base, and their run showed the comparison detects both a real difference and the code's layout,
`-nop` 3.7% from the base, the store 0.45% faster than `-nop`. What is left of the plan below is
the Relaxed store, the `fetch_add` pair, and the `zcr-spsc-v3-2t` variants, and a fence variant,
`fence(SeqCst)`, would tell the barrier from the store.

Whether `analyze` detects a change of a known size, and stays quiet where there is none, is
checked on nothing yet (wink, 2026-09-21). Variants of a quiet two-thread bench that add one
operation of known cost per round trip give it graded, known effects to find: `zcr-mpsc-v2-2t`,
whose `smt` claim on the 7600X is 0.15 to 0.19%, and `zcr-spsc-v3-2t`, whose `smt` cell smears
over sub-levels at 2 to 3% and so tests detection on a noisy bench.

- `-nop`: the hook compiled in and doing nothing. Against the base it measures layout alone, since
  each bench is its own function at its own address, and a `-nop` that differs is itself a finding
  for [Measure whether code layout moves the level](#measure-whether-code-layout-moves-the-level).
- `-store-relaxed` and `-store-seqcst`: an uncontended store on a cache line of its own, on the
  producer's side, per message, kept by `black_box`. On x86 a Relaxed store is a `mov` and a SeqCst
  store an `xchg`, so we think the first is near the floor and the second several percent.
- `-rmw-relaxed` and `-rmw-seqcst`: a `fetch_add` both ways, which on x86 is `lock xadd` either
  way, so the pair must not differ, the negative control.
- The operations must survive the optimizer, or a variant measures nothing and reads as a clean
  negative. We think this takes attributes as well as `black_box`: the hook `#[inline(never)]`,
  so `-nop` is a real call that is not folded away and every variant pays the same call, and the
  atomic reached through `black_box` so its store or `fetch_add` is kept. The disassembly is read
  once per variant, `objdump -d` or `cargo asm`, to see the `mov`, `xchg`, or `lock xadd` is
  there, since an attribute's effect is a claim until the code shows it.
- The expected ordering is written down before the run, so the verdict cannot be read into the
  numbers afterwards, and the comparison is `analyze`'s ladder, bench names in one invocation.
- It is the natural test for `feat: analyze compares two sides`, and whether that rung takes it
  in is decided at that rung.

### Measure whether polling disturbs a run

The rule that a measurement host is not polled while it measures rests on a suspicion: the first
two 7600X baselines were both polled by an ssh watch every 90 s, and a third ran with nothing
watching, so no comparison ever set polled against unpolled on purpose (wink, 2026-09-21, not
convinced polling is bad). An experiment that measures it: one config, sessions alternating
polled and unpolled, the poll at the rates a watcher really uses, 90 s and faster, on both hosts,
judged by `analyze`'s A against B. The answer either retires the rule or gives it a number, and
[The quick config for the 7600x](#the-quick-config-for-the-7600x) runs under whichever holds.

### A record names the bench source's commit

A record says of what was measured only the tool's version, and the benches are code that changes
between commits of one version (wink, 2026-09-21, split from `feat: a record carries its config
and a label`). We think the commit belongs in the record, stamped at build time as `rustc` is, with
a mark when the tree was dirty, so two records that differ can be told apart by their source.

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
- a list the host declares in `[profiles]` does not refuse: its threads past the list run
  unpinned, the banner and the record's `pin_threads` saying which (wink, 2026-09-29, at `feat: a
  named placement pins threads in order`). The refusal below is for the built-in names, where a
  name is an ordering and a spill past its level would be silent
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
- the last rung writes the base configs under `configs/`. The 7600X's re-record across processes
  this entry once carried was done at `docs: the report guide's results from one 7600X session`,
  on the demo's pairs `5,11` and `5,4` through `setup`'s profiles. The acceptance check is one
  config, unedited, running on both hosts and landing on the demo's pairs

### Additional thread control

A bench's thread count is in its name, `mpsc-1t` and `mpsc-2t`, so nothing runs one at three
threads, and the pin pool is a flat list where an mpsc's consumer and its producers are not
interchangeable (wink, 2026-09-20). The entry read "shape once a concrete bench needs it", and
[mpsc at N threads](#mpsc-at-n-threads) is that bench.

- a thread count on the line and in a config, so one bench name spans a sweep and the count
  rides in the record beside the bench. The registry states each bench's threads as a pattern,
  its fixed roles and then a repeated one, `consumer` then `producer` N times, so the count
  expands it (wink, 2026-09-29, built at `feat: a named placement pins threads in order`)
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
  command, what [The quick config for the 7600x](#the-quick-config-for-the-7600x) is to work out
  by hand, and is an entry of its own once that says what to measure first. The cycle first named
  for it closed without doing so.

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

The same dial-in as [The quick config for the 7600x](#the-quick-config-for-the-7600x) on the
noisier host, and the hint the moved entry proposed (wink, 2026-09-18, at the opening of `feat: a
shorter trustworthy run on the 7600x`, which closed without the dial-in). Waits on the 7600X's.

- the 3900X, whose unpinned block means carry a lag-1 autocorrelation of +0.78 against the
  7600X's +0.02, so its blocks are not the independent replicates `CI95 blocks` assumes and its
  floor will sit higher. Pinning takes it to +0.20, which is the first thing to measure
- an allocation hint once the 7600X's dial-in calibrates the model: every invocation already knows
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

- the guide's docs bullet is done at `docs: the report guide's results from one 3900X session`:
  its 3900X table, one binary at one placement each, puts spsc v2 2% over v1 on the SMT pair, 23%
  over within a CCX, and 4% over across CCXs, so the gap is a placement's, not the ring's
  (2026-10-01)
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
- the specifics, from wink's question of 2026-10-01 about what a run on Windows or a Mac would
  take. The layer behind `main` is topology, pin a thread, read, pin, and restore the clock,
  inhibit sleep, and hash the binary, Linux's code its first implementation, and the banner and
  the record say which controls a run had
- Windows: `GetLogicalProcessorInformationEx` gives cores, siblings, and L3 groups, and
  `core_affinity` already pins there. The clock is the weak part: no per-core min = max, only a
  power plan's minimum and maximum processor state and boost mode through `powercfg`, coarse, in
  percent, and an administrator's, with a restore of its own. Reading the delivered clock wants a
  driver for APERF/MPERF, or the coarse `CallNtPowerInformation`. The inhibit is
  `SetThreadExecutionState`, and setup is admin rights in place of a udev rule
- macOS, Apple Silicon above all: no API pins a thread to a core, affinity tags being hints it
  ignores, performance and efficiency cores mix, and the clock can be neither set nor read
  without `powermetrics` as root, so a Mac run is unpinned and unclocked, a sanity check rather
  than a placement table
- we think iceoryx2 runs on all three, and zc-ring-x1's attachable v4 region may lean on Linux's
  shared memory, which wants checking first

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

[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[75]: /notes/chores/chores-05.md#settle-time-is-not-a-grade
[84]: /notes/chores/chores-06.md#docs-experiment-in-the-local-agent-files
[85]: #feat-mpsc-v3-benches-opening
[86]: #feat-the-mpsc-v3-bench-pair
[87]: #feat-the-mpsc-v3-single-and-one-segment-pairs
[88]: #feat-the-mpsc-v3-futex-pairs
[89]: #docs-what-mpsc-v3s-mode-and-wake-cost
[90]: #feat-mpsc-v3-benches-closing
