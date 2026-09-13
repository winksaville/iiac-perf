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

_No cycle currently in progress._

## Waiting

Important work that cannot start yet. Each entry names what it waits on and its rank once
unblocked, and every opening checks the conditions.

_None._

## Todo

Entries are in priority order, the first highest, and reprioritizing moves the entry. The
long-tail backlog is in [todo-backlog.md](notes/todo-backlog.md), and deeper detail lives in
the frozen `notes/chores/` design subsections, linked by `[N]` refs.

### Config and setup: the Config: list, the config in the record, and a setup subcommand

A run's parameters come from defaults, the XDG file, the project-local file, and flags, and the
report names neither every value nor where each came from, so two records cannot be checked for
matching parameters (wink, 2026-09-12). Beside it, a run should be definable as a config file and
named on the line (wink, 2026-09-05, after the placement sweep).

A host is ready for iiac-perf only after hand work: its `[freq]` declaration written with the
clamp limits, and sudo for every `--pin-freq` and `restore-freq` (wink, 2026-09-14). A missing
limit is a live hazard, [bugs.md](notes/bugs.md)'s `restore-freq` entry. One subcommand should do
both, needing sudo once.

Ranked first, ahead of spawning, which builds on both halves (wink, 2026-09-14): a spawning parent
checks that every child's recorded config matches, the across-process CI95 and LSC refuse runs
whose parameters differ, and pinned children need `--pin-freq` without sudo.

The config half:

- **`Config:` list**: every run parameter with its value and source, `(default)`, a file name, or a
  flag, "same as default" when a source restates it. The block and warm lines leave `Setup:` for it
- **the record's `config` object**: value and source per parameter, plus the loaded files' paths
  and content hashes, schema 6, so an analysis can refuse to compare runs whose parameters differ
- `--config PATH` loads that file as the top layer over the XDG and project-local files, the flags
  still winning, and the banner names it with the rest. No such flag exists today, the loader
  knowing only the two fixed locations
- every CLI run parameter gets a config key, the mirror of "Config keys stay CLI-settable" below,
  which pairs each key with a flag. Today `duration`, `band_labels`, `decimals`, `settle_time`,
  `warm_cap`, and the three block keys have keys, and `--total-duration`, `--samples`, `--inner`,
  `--pin-cpus` (profiles name a spec, but nothing selects one by default), `--record`, `--tag`,
  `--no-env-probe`, `--no-inhibit`, `--ticks`, and `--verbose` do not. `--pin-freq` is the
  "Two-regime runs" entry's key
- the bench list is a key too, so a config file is a complete run, `iiac-perf --config
  placement.md` and nothing else on the line
- the `[freq]` exclusion stands: the steady state is the host's declaration, not a run's, and the
  setup subcommand writes it
- with spawning, a config also names the children's knobs, and an A/B is two configs or one with
  two arms, which is the shape a cross-host comparison wants

The setup half:

- **the host's config**: write `~/.config/iiac-perf/config.md` with a `[freq]` table whose
  `min_mhz` and `max_mhz` are read from the live clamp, refusing to overwrite an existing file
  without a flag. On the 3900X today that is 1745 and 4673 MHz, and no such file exists there
- **the permissions**: a udev rule with a per-user ACL on the cpufreq files and
  `/dev/cpu_dma_latency`, so `--pin-freq`, `restore-freq`, and the "A --pin-idle knob" entry's
  clamp need no sudo. Print-only by default, an apply flag doing the one sudo, and an uninstall
  flag removing the rule
- **the example configs**: convert `iiac-perf.toml.example` to the `.md` carrier as the one
  example, delete the `.toml` one, point `docs/config.md` at it, remove `iiac-perf.md` and ignore
  it in git, and fix the README line crediting it with the hundred blocks. `iiac-perf.md` goes only
  after the 3900X's XDG config exists, since it is that host's only declaration today

### One bench per process, CI95 and LSC across processes

A process start re-rolls where the rings and stacks land in memory, and that placement sets a
bench's level, so a run's CI95 and LSC, computed over blocks inside one process, are lower bounds
that can miss the real spread by a wide margin (wink, 2026-09-05, confirmed 2026-09-12). iiac-perf
should find a bench's CI95 and LSC itself: one bench per process by default, replicated across
fresh processes, interleaved A/B, the error bars computed over process means.

- **the evidence**, the 7600x on 2026-09-12 (UTC 2026-09-13), `zcr-mpsc-v1-2t -d 5`, 100 blocks,
  1-10 ms sleep, config isolated. Records in the 7600x's `~/iiac-perf-data/warmup-20260913/`, a copy
  and its `analyze.py` in the ignored `tmp/warmup-7600x-20260913/`:
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
- **one bench per process**: a bench list, `all` included, runs each bench in its own child, so no
  bench inherits another's placement and the run-indexed levels above cannot appear. The parent
  respawns `current_exe()` as `qualify-environment` does, is inert while a child runs, the
  `suggest-freq` sampler bug in [bugs.md](notes/bugs.md) being the warning, and passes its config
  so knobs and pins match
- **replication across processes**: N children per bench, A and B alternating when comparing, the
  guide's standing advice done by the tool. The CI95 and LSC over process means are the first that
  are not lower bounds, and a child needs a second or two since blocks within a process agree to
  0.1%
- **one statistics owner**: CI95 and LSC over a series of means is one module, fed block means
  within a process and process means across them, and the same arithmetic serves `analyze` over a
  directory of records (the "Analyze a directory of records" entry's cross-run tier)
- **error-bar labels**: a single-process CI95 and LSC print labeled within-process, and an
  across-process row joins them once spawning exists, the ratio of the two saying whether
  per-process state dominates. "Block" keeps the within-process replicate, and the between-process
  one gets its own word, so a row never has to say which it meant
- **naming the level** is a second experiment: children that map the ring regions themselves and
  sweep the second ring's page offset against the first, then huge pages. We think it is cache-set
  aliasing from placement
- **first use**: the 7600x's `all` re-record, `--record` into a directory that stays, which the
  records' host block makes the start of the cross-host comparison, and a run of the mpsc v1 pair
  there, whose `all` rows are the renamed v0 rows
- subsumes the "Stability selftest mode" idea in `## Ideas` and the orchestration in
  `tests/qualify_environment.rs`

### Measure whether code layout moves the level

A rebuild moved `zcr-mpsc-v1-2t` from 67.9 to 60.8 ns while `min-now` held to 0.01 ns (the spawning
entry's evidence), so the binary's layout may set a bench's level as a process's placement does
(wink, 2026-09-12). Fat LTO with one codegen unit, and LLVM function and block alignment, against
the default profile, each built twice around a trivial unrelated change, interleaved, to see
whether layout stops moving the level. Wants spawning first, so the placement level is measured
rather than confounded.

### Measure core isolation on both hosts

Pinning leaves other work free to share the benched cores (wink, 2026-09-12). An interleaved A/B
on the 3900X and the 7600x: a systemd scope restricting other work to the unbenched CPUs first,
no reboot, then `isolcpus` with `nohz_full`. Any scope or boot change on a host is confirmed with
wink first. Kin to the "Tick-phase avoidance" idea, which isolation strictly dominates when a
reboot is allowed.

### Report the v1/v2 replication to the guide and zc-ring-x1

The pinned v2 two-thread handoff read slower than v1's in both of the report guide's record pairs,
in the ignored `tmp/mpscv1rows/` (2026-09-11) and `tmp/v2rows/`, and zc-ring-x1 has not been told.
2026-09-08 replicated it and found it host-dependent, three interleaved runs per cell, mean z4..n2:
3900X pinned 0,1 v1 86 ns and v2 105 ns, 7600x pinned 0,1 v1 60 ns and v2 56 ns, 7600x pinned 0,6
v1 32 ns and v2 35 ns, `0,6` being SMT siblings. Records in the ignored `tmp/v1v2-20260908/` here
and in `~/iiac-perf-data/v1v2-20260908/` on the 7600x, tagged by pin, with both hosts' demo depth
sweeps beside them.

- the guide calls the gap a two-pair lead and does not cite the replication, a docs change
- a message to zc-ring-x1 with these numbers and the spawning entry's placement levels, which
  make every single-process zcr comparison suspect. Their Todo already carries the demo's pin-pair
  mismatch, cross-L3 on the 3900X and same-L3 on the 7600x, so the message needs only the numbers
- the placement sweep's records (2026-09-05), 45 runs in the 7600x's
  `~/iiac-perf-data/placement-20260905` and 27 in the ignored `tmp/placement-20260905` here, are
  unfiled, and [placement-map.md](notes/placement-map.md) is their home when it is refreshed

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
- its cross-run arithmetic is the spawning entry's, one statistics module serving both, and the
  records carry the host block cross-host analysis needs, a hostname alone naming nothing

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

### Two-regime runs

A config key selects the box's default regime, pinned or wandering, and the CLI overrides it
either way, so a tuning campaign pins every run without typing the flag and a quick sanity
check drops back to the real-world clock one-shot (wink, 2026-08-17).

- the workflow it serves (written into the measure-reproducibility cycle's report reading
  guide): tune pinned, where LSC is small enough that "did this tweak clear LSC" resolves in a
  few runs, then confirm the winner unpinned, where the number means what the real world will
  see
- the key is a run parameter, CLI-settable per "Config keys stay CLI-settable" below, not
  part of the `[freq]` declaration: it says which regime runs use, while `[freq]` stays the
  declared way home. We think top-level `pin_freq = true|false` beside `duration`, with
  `--pin-freq` / `--no-pin-freq` as the override pair and `--pin-freq=MHZ` still naming a
  target
- the wandering default stands for an unconfigured box: pinning stays something the user
  asked for, in config or on the line, never a surprise mutation

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

### docs: file the continuation notes into their homes

#### Problem

`## Continuation notes` has grown into a second backlog: the next cycle's agreed Todo entries, an
unrun port-and-bug cycle, a `restore-freq` bug with no entry, the 7600x placement-level series, and
record directories named nowhere else (wink, 2026-09-14, asking for a clean slate).

#### Solution

Every fact was filed into its home and the section reset to `_None._`: a config-and-setup entry
at the top of `## Todo`, the `Config:` list and schema 6 merged with the old `--config` entry and a
setup subcommand for the host's config and permissions, the spawning entry second with the
error-bar labels and the placement-level evidence, new entries for the two measurements, the
zc-ring-x1 report, and the Windows and macOS port, the `restore-freq` bug in `notes/bugs.md`,
host and record facts in `notes/ops.md`, and the Todo entries still saying batch moved to the
block word.

#### Acceptance check

`## Continuation notes` reads `_None._`, and each of its fourteen bullets at `main` has its fact
in a Todo entry, `notes/bugs.md`, or `notes/ops.md`, or is recorded in the landed history. The
first Todo entry is config and setup, the second spawning. `vc-x1 validate` passes.

Passed: the section reads `_None._`, the fourteen bullets are filed as the deliberation and the
solution say, config and setup then spawning head `## Todo`, and `vc-x1 validate` passed.

#### Ladder

- docs: file the continuation notes into their homes (done)

#### Deliberation

- **Bookmark name**: `docs-todo`, not the title's slug, by wink's explicit choice at the opening.
  - covers this cycle's bookmark only, its create and delete pushes included
- **Single-step**: one commit, since every change is a filing of facts already agreed.
- **Config and setup first, spawning second**: one entry, so one ladder, ahead of spawning, by
  wink's choice at the review.
  - spawning builds on both halves: its parent checks the children's recorded config, its
    across-process error bars refuse mismatched parameters, and pinned children need `--pin-freq`
    without sudo
  - the two halves share `config.rs` and `docs/config.md`, and setup writes the file the list names
  - cost accepted: measurements stay single-process lower bounds until spawning lands
- **The port-and-bug cycle dissolves**: its four parts are filed here rather than run as a cycle.
  - `notes/perf-findings.md` was for the 7600x freq numbers, which now live in the bug entry
  - the `iiac-perf-dev` clause for `notes/ops.md` is moot, both hosts carrying the plain 0.28.11
  - the Windows and macOS entry's original content was never written down, so the entry states
    what is Linux-only today rather than recovering it

# References

[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[75]: /notes/chores/chores-05.md#settle-time-is-not-a-grade
[84]: /notes/chores/chores-06.md#docs-experiment-in-the-local-agent-files
