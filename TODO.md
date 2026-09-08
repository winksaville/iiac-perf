# Todo and cycle record

This file contains near term tasks with a short description and reference links to more details.
Its shape is [Todo format](agent-data/notes.md#todo-format).

## Continuation notes

Where the agent was, for the agent that comes next: working copy state, the step in flight, an
open question. Ephemeral, never a record. Written before a restart or when a session is about to
lose context, read first at acquaint, acted on, each fact filed into its home or its bullet kept, and
the rest reset to `_None._` by the reader.

- The `owner` rename's phase two needs zc-ring-x1 only. vc-x1 has confirmed it reads `owner`, so
  when zc-ring-x1 does, `.owner` goes and the README's transition clause is retired.
- Still to run: the port-and-bug cycle, which creates `notes/perf-findings.md` for the 7600x
  numbers below, appends the `iiac-perf-dev` clause to `notes/ops.md`'s 7600x bullet, writes the
  `restore-freq` entry into `notes/bugs.md`, and adds the "Windows and macOS port considerations"
  Todo entry.
- The 7600x's `[freq]` block omitted `min_mhz` / `max_mhz`, so a `restore-freq` there widens the
  clamp to the hardware floor: on 2026-09-04 it went from 2.99 GHz to 427 MHz, and had returned to
  2.99 by 04:14 through a path nobody identified. Its `~/iiac-perf.md` now declares `min_mhz =
  2991` and `max_mhz = 5457` (seen 2026-09-05), its `~/.config/iiac-perf/config.md` still omits
  them, and a `read-freq` is worth running before trusting that host's numbers. Nothing else
  records the episode until the port-and-bug cycle files it.
- After `feat: host identity in the record` lands: install the plain 0.28.6 on the 7600x, and
  re-record its `all` run with `--record` into a directory that stays, 3-5 runs per cell
  interleaved as the analyze entry asks, since the 2026-09-02 records were deleted rather than
  kept in the old shape. The new records carry the host block, so the cross-host comparison the
  cycle was for can start there.
- The 7600x carries `iiac-perf-dev` 0.28.3 beside the plain 0.28.2, copied by hand 2026-09-05. The
  placement sweep's records are in its `~/iiac-perf-data/placement-20260905` and the 3900X's in
  this repo's ignored `tmp/placement-20260905`, 45 and 27 runs, unfiled. The placement map is their
  home when a docs cycle refreshes it.

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

### Analyze a directory of records

A record exists so a re-analysis can happen without the session that produced it, and nothing reads
one back, so every analysis is a throwaway script whose numbers nobody can check (wink, 2026-09-03,
reading the 7600X duration sweep). An `analyze` subcommand over a directory of records, sharing
`record.rs`'s struct so the schema keeps one owner.

- three tiers, in increasing order of what they are worth:
  - **tabulate**: pivot the records into a table on an axis, duration, host, bench, or run. The
    least interesting tier, and the one the other two are built on
  - **read the series nothing reads**: `batch_mean_ns`, `batch_samples`, `clock_t_ns`, `clock_cpu`,
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
  was the wrong shape. The 7600X re-recording the host identity cycle defers to after its Land
  wants 3-5 runs per cell, interleaved, which is what the guide has said all along
- the output reuses the report's row names, so the guide decodes the new surface for free. Grading
  the set the way a run grades itself is the natural extension: do these runs agree, and is a
  disagreement drift, a step, or one bad run
- `--format csv` / `--format json` for the plotting hand-off, kin to "Machine-readable report
  output" below, one flag family
- ranked first because it reads the record: the host identity cycle changes the shape and the
  file extension under it, and cross-host analysis needs the host block
  that v3 lacks, a hostname alone naming nothing

### Spawn mode, replication across processes

Blocks replicate inside one process and cannot re-roll what a process start re-rolls, so a run's
CI95 is a lower bound on what applications see (wink, 2026-09-05). Measured that day on the 7600x
with `zcr-spsc-v1-2t --inner 100 --blocks 10 --pin-cpus 2,8`: five invocations with 1 s block
sleeps and 100 ms warmups each held their ten blocks within 0.1 ns, and the invocations landed on
two levels 0.15 ns apart, seven times the spread the blocks predicted. The sweep's three
short-sleep invocations at the same cell agreed to 0.1%, which five would not have. A spawn mode
replicates by respawning the binary, `current_exe()` as `qualify-environment` does, one child per
replicate.

- spawns around blocks, never instead of them: each child keeps its blocks, and the report prints
  both spreads, CI95 over blocks and CI95 over spawns, whose ratio says whether per-process state
  dominates
- interleaving is the prize: a parent can alternate A, B, A, B, the guide's standing advice for an
  A/B call done by hand today, and an LSC across processes is the first that is not a lower bound
- the parent is inert, waiting on the child and nothing else, the `suggest-freq` sampler bug in
  [bugs.md](notes/bugs.md) being the warning, and children inherit the config so knobs and pin
  match
- cost is the process warm, not the measuring: within a process blocks agree to 0.1%, so a child
  needs a second or two, and ten spawns is about a minute against fifteen seconds for ten blocks,
  a confirm-step mode beside blocks rather than a new default
- what a process start re-rolls, pinned, is mostly where the rings and stacks land in memory, so
  the level is most likely cache-set aliasing from placement. Spawning shows the level and its
  width, and naming it is a second experiment: children that map the ring regions themselves and
  sweep the second ring's page offset against the first, then huge pages
- names: "block" keeps the within-process replicate, and the between-process one gets its own
  word, so a report row never has to say which it meant
- subsumes the "Stability selftest mode" idea in `## Ideas` and the orchestration in
  `tests/qualify_environment.rs`
- ranked after the analyze entry, whose cross-run tier is the same arithmetic over records

### Define a run in a config file, and a --config flag

A comparison across hosts or days is a bench list and a dozen knobs typed as flags each time, so
two runs meant to be identical differ by whatever a hand forgot (wink, 2026-09-05, after the
placement sweep). A run should be definable as a config file and named on the line.

- `--config PATH` loads that file as the top layer over the XDG and project-local files, the flags
  still winning, and the banner names it with the rest. No such flag exists today, the loader
  knowing only the two fixed locations
- every CLI run parameter gets a config key, the mirror of "Config keys stay CLI-settable" below,
  which pairs each key with a flag. Today `duration`, `band_labels`, `decimals`, `settle_time`,
  `warm_cap`, and the three block keys have keys, and `--total-duration`, `--outer`, `--inner`,
  `--pin-cpus` (profiles name a spec, but nothing selects one by default), `--record`, `--tag`,
  `--no-env-probe`, `--no-inhibit`, `--ticks`, and `--verbose` do not. `--pin-freq` is the
  "Two-regime runs" entry's key
- the bench list is a key too, so a config file is a complete run, `iiac-perf --config
  placement.md` and nothing else on the line
- the `[freq]` exclusion stands: the steady state is the host's declaration, not a run's
- with spawn mode, a config also names the children's knobs, and an A/B is two configs or one
  with two arms, which is the shape a cross-host comparison wants

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

### zcr-spsc-v2-1t/2t benches

zc-ring-x1's `main` landed an SPSC v2 on 2026-09-07, the in-slot seq ring: v1's protocol with the
seq word moved into the slot it publishes, so the commit store and the message travel on one cache
line, and it is now that crate's default `Ring`. Nothing here measures it. A `zcr-spsc-v2-1t/2t`
pair beside the v0 and v1 pairs, built the way `feat: zcr-v1-1t/2t benches` built the v1 pair: a
`leak_v2_ring` in `zcr_common` (v2's region is v0's shape, header then slots, no seq array), two
bench files pinned to `spsc::v2` by explicit path, and the report guide's rows. The slot contract
differs: `T` sits behind a crate-owned slot header, so `Msg` must fit the slot minus those bytes.

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

### Blocks as the first-class mode

Knobs, always-on error bars, then a measured default flip (designed 2026-08-02, the
duty-cycle/LSC session, evidence in chores-06).

- the sleep and warmup knobs land via the measure-reproducibility cycle's "block sleep and
  warmup become knobs" rung (defaults zero, replication rows gated on a nonzero sleep). The
  `--blocks` config key moved out to "A blocks config key, and turn it on for this box"
  above, which also picks this project's operating point. The flip-zone hazard stays this
  entry's, the range-over-fixed argument (fixed 0.5 ms sleeps straddled both 3900X states,
  D grade, LSC 6x worse)
- the flip zone measured on the 7600x (wink, 2026-08-20, `min-now --blocks 100` sleep
  series): 0 and 1 ms sleeps hold the fast state (16.2 ns, A), 1 s holds the bursty state
  (18.3 ns, A), and 100 ms lands the transition inside the run at ~3.3 s, graded F by env
  and run step at the same instant, ~7.7% of samples still in the fast state and
  resolution honestly widening 0.01 -> 0.41 ns. A/B sleeps go on either side of the flip
  zone, never in it, and grade F vetoes the straddlers
- CI95 / LSC rows always print, `-` when replication is too thin to quote: display gate ~10
  blocks (the t multiplier is 12.7 at df 1, 2.26 at df 9, flat after). Plain runs show `-`
  too, so every report answers "how sure" even when the answer is "can't say"
- the summary-row re-housing (wink's 2026-08-02 ask, sketched 2026-08-20) and the
  never-a-bare-zero claim display landed as the "fix: left-align the summary rows"
  single-commit cycle, leaving this entry the display gate, the `--blocks` config key, and
  the default flip
- the display gate and the default count are different numbers: gate = validity, default =
  operating point. The default flip is its own later cycle (report-contract reshape, 0.25.0
  scale): the default duty cycle re-selects the bistable state (the 3900X headline becomes
  24.0), wall time grows ~2.6x at 1-10 ms sleeps, `duration=` wants a measured-vs-wall
  split, and the qualify parser plus README examples follow
- acceptance for the flip: A/A runs showing LSC bounds same-code deltas (the qualification
  redesign's keystone), and a per-bench overhead survey (spin-partner benches tolerate high
  counts, solo benches pay wake residue: chores-06's 7600x and zcr data)
- philosophy recorded: many blocks are many independent environmental episodes, an honest
  error bar that low counts can fake by luck. The mean is state-conditional and deliberately
  deployment-shaped ("--blocks 1000 feels more real")

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

Sample `cpuinfo_avg_freq` at batch seams (the reader exists, `src/freq.rs`) so a mid-run step
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

### Move the batch seam's work off the measuring thread

Use the FastForward-style SPSC ring. The batch flush stops the bench for ~1-2 ms (a
`select_nth_unstable` over up to 65,536 values plus 65,536 histogram records) every 50 ms, so
~2-4% of a run is spent at seams. Hand the filled buffer to a consumer thread that sorts,
summarizes and records while the producer fills a second one. The seam drops to a pointer swap.

- the payload is one word, a buffer offset, the exact shape `ffq` is built for, and the
  project dogfooding the queue it benchmarks
- double-buffered: at ~1-2 ms of work per 50 ms batch the consumer runs ~30x faster than it
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
    sub-processes, and the kernel tells us anyway: sweep `/proc/self/task/` at batch seams
    (last-ran lCPU is `stat` field 39, children via `/children`, recursively). CPU-time
    deltas between sweeps identify the active threads with no cooperation, and `sched_getcpu`
    (vDSO-cheap) covers the measuring thread exactly. Sampled truth: migrations inside a
    batch and threads born and dead between seams are unseen, which matches the step
    detector's batch granularity, and cost is ~us per seam against a 1-2 ms seam
  - batches gain a placement-class label, so a placement migration becomes an *attributed*
    step ("cross-LLC -> same-core"), the way the env grade attributes DVFS
  - unpinned `--blocks` runs stratify block stats by placement class instead of one smeared
    CI: the scheduler's wandering becomes a free stratified experiment (how the SMT fast
    mode was found)
- subsumes the vocabulary half of "Tighten thread and CPU terminology" (above): keep its
  software-thread vs lCPU distinction, adopt lCPU as the standard term

### Rebase web-claude-tweaks onto post-0.22.0 main

It rewrites an already-published bookmark (needs approval) and its arbitrary `0.21.0-b`
version needs replacing, owed from the 0.22.0 close-out plan.

### Unit scaling in report columns

`us`/`ms`: per-row auto-scale so columns stay eyeball-comparable (bands are monotonic, so a
row's first/last/mean share a magnitude), or `--units ns|auto` for script-stable output. Needs
`--decimals` landed first (`3.18 ms` vs `3 ms`). Candidate `-4` for the report-options cycle.

### Drift and clock plots in the terminal

Every run records a batch-mean series and a delivered-clock series and reports them as a grade
letter, so "did this run drift" is answered by a letter with no picture behind it (wink,
2026-09-03). Braille or block characters drawn in the terminal, no plotting dependency.

- two plots sharing one time axis: batch means, where the drift and step signals come from, and the
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

Will likely need inner-loop support on `Probe` (batch N calls per sample, report divides by N
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

### feat: host identity in the record

#### Problem

A record names its box by hostname alone, so a file read on another machine cannot say what CPU,
topology, memory, kernel, or toolchain produced it, and cross-host comparison is by memory (found
2026-09-02 reading the 7600X `all` run's records). Two smaller things sit beside it: `Record<'a>`
borrows its inputs, so nothing can read a record back through the struct that wrote it, and the
file extension is `.ndjson` where the family writes `.jsonl`.

#### Solution

Schema version 4 turned the `host` string into a host block: a `Host` struct in its own `host`
module, held as `Record`'s `host` field the way the policy fields hold a `PolicyField`, so serde
nests it as a JSON object with no attributes. It rides in every line as the policy fields do, so a
line stands alone (wink, 2026-09-07). Its fields:

- `name`: the hostname, the string the field holds today
- `cpu_model`: the model name from `/proc/cpuinfo`
- `ram_bytes`: `MemTotal` from `/proc/meminfo`, what the kernel has, less than what is installed
- `cache_line_bytes`: L1D's `coherency_line_size`, one scalar, since the levels never differ in
  practice and it is the false-sharing constant the rings are sized by
- `caches`: one entry per sysfs cache index under `/sys/devices/system/cpu/cpu0/cache/`, each with
  `level`, `type`, `size_bytes`, and `shared_cpus` (its `shared_cpu_list`). The array's length is
  the depth, so no separate depth field, which could only agree with it or lie. L1 splits into
  Data and Instruction, so the 3900X has four entries for three levels, recorded as the kernel says
  them. The sharing lists are the topology: L1's names the SMT siblings (`0,12`), L3's the CCX
  (`0-2,12-14`), which is the map a 2t placement note needs and what makes a record's `--pin-cpus`
  readable on another machine
- `kernel`: the release from `uname`
- `rustc`: the compiler version, baked in by a `build.rs` since it is not in cargo's environment

Rules the block follows: units in the names, as `batch_mean_ns` and `clock_khz` do, and everything
in it readable without root, so it fills itself on every run, a read that fails yielding null. The
field-doc table names a nested field by dotted path, `[]` marking an array of objects, and its
test resolves each path into the sample record. Before the block, the record's borrows went: the
struct is owned, derives both `Serialize` and `Deserialize`, and a test round-trips a record through
JSON, so the schema keeps one owner and the analyze entry inherits its reader. After it, the
extension moved from `.ndjson` to `.jsonl`, the name the family already uses, the bytes unchanged.

#### Acceptance check

On this box `iiac-perf-dev min-now -d 1 --record tmp/hid/` writes a `.jsonl` file whose one line
carries `schema_version` 4 and a `host` object with the seven fields, its `caches` holding four
entries each naming its `shared_cpus`. `iiac-perf-dev describe-record` lists every `host.` field.
`vc-x1 validate` passes with a test that round-trips a record through JSON.

#### Ladder

- [feat: host identity in the record opening][1] (done)
- [refactor: own the record's fields][2] (done)
- [feat: probe the host into a Host block][3] (done)
- [feat: write records as .jsonl][4] (done)
- [feat: host identity in the record closing][5] (done)

#### Deliberation

- **The fields are wink's six, reshaped** (wink, 2026-09-07): `host.name`, `cpu_model`,
  `ram_size`, `cache_line_size`, `cache_depth`, `cache_sizes[depth]` were the ask. The depth field
  went, the array's length being the depth. The sizes became entries carrying level, type, and the
  sharing list, since one entry per sysfs index needs no interpretation and the sharing lists are
  the topology the placement notes need. Units went into the names. `kernel` and `rustc` were
  added, one string each, both changing measurements and both what a record exists to avoid looking
  up later.
- **Per line, not per file** (wink, 2026-09-07): the block repeats on every line the way version,
  pid, tags, pin list, and the policy fields already do, so a line explains itself and files
  concatenate with `cat`. A sidecar or a header line would save a few hundred bytes a line and cost
  that property, and in directory mode, where a file is one line, would save nothing. The batch
  and clock series, about 4 KB a line that nothing reads, are where the file shrinks.
- **Owned fields, not offsets** (wink, 2026-09-07): the borrows are a write-side convenience worth
  one clone per record, and they rule out reading a record back through the same struct. Owned
  fields make one struct both writer and reader.
- **Owned fields first, the extension last**: the block is born owned when the struct already is,
  and the extension touches only names, so it rides at the end where its diff stays alone.
- **Deferred**: memory speed and channel count live in the DMI tables (`dmidecode -t memory`),
  root only, so they are a `[host]` config declaration pasted once, the way `read-freq
  --as-config` fills `[freq]`, when wanted. Microcode from `/proc/cpuinfo`, board and BIOS from
  `/sys/class/dmi/id/`, and the target-cpu flags beside `rustc` likewise. The 7600X `all` run is
  re-recorded into a directory that stays after Land, since the 2026-09-02 records were deleted
  rather than kept in the old shape, a continuation note for the next session on that host.
- **Waiver** (wink, 2026-09-07): every push of this cycle, the opening's bookmark push through the
  closing, is approved in advance, the work and description reviews included. Land is outside it
  and waits on the user's review of the finished bookmark.

#### Ladder details

##### feat: host identity in the record opening

The cycle's setup commit: create and publish the bookmark, delete `## Closed`'s contents, move the
Todo entry into this block, bump the version-of-record, and rename the package to `iiac-perf-dev`.

* A late finding about the cycle before: `chore: point zc-ring-x1 at main` landed with its finished
  block under `## In Progress` rather than `## Closed`, where a single-step cycle's one commit
  writes it directly. The landmark is `facad37e`, its block is read there, and this opening
  replaces it as any opening replaces the last block. Not amended, per Cycle-record.

##### refactor: own the record's fields

`Record<'a>` holds `&str`, slices, and a map reference, a write-side convenience that saves one
clone per record and rules out `Deserialize`, since serde cannot borrow a slice or a map from JSON.
The struct becomes owned, derives both directions, and a test round-trips a record through JSON.

* The struct borrowed nine fields from the run, the config, the policy, and two statics.
  - Each is owned now (`String`, `Vec`, `BTreeMap`, `Option<PolicyField>`), cloned once per record
    at assembly, which is nothing against the run that produced it. The lifetime parameter is
    gone from the struct and from `build_record`.
* Nothing could read a record back.
  - `Record` and `PolicyField` derive `Deserialize` beside `Serialize`, and a test writes the
    sample record to a line, reads it back into the struct, and checks the re-serialized value is
    identical, so the schema keeps one owner and the analyze entry has its reader when it runs.

##### feat: probe the host into a Host block

A record names its box by hostname alone. A `Host` struct with the seven fields, probed once when
the `Recorder` is built and carried by every record it writes, the field dictionary naming nested
fields by dotted path so the key test walks into the block, and schema version 4.

* The probes have no home, and the record module is already the longest file's neighbour.
  - A `host` module owns the struct, the cache entry, and the probes: `/proc/cpuinfo`,
    `/proc/meminfo`, CPU 0's sysfs cache directory in index order, `uname`, and `gethostname`,
    which moved there from the record module. Every read that can fail yields `None`, never a
    default, and the cache list is empty rather than absent when sysfs is missing.
* The compiler version is not in cargo's build environment.
  - A `build.rs` runs `$RUSTC --version` and bakes it in as an env var, `unknown` when the call
    fails, so the field is never null.
* The dictionary is flat and its test compared top-level key sets.
  - Entries name nested fields by dotted path, `[]` marking an array of objects
    (`host.caches[].level`). The test now counts a top-level key documented when an entry names
    it or anything under it, and resolves every entry's path into the sample record, so `tags`
    and the policy fields stay documented as wholes and the block is documented member by member.
* The cache `type` word is a Rust keyword.
  - The field is `kind` in the struct and `type` on the wire, one serde rename.

##### feat: write records as .jsonl

The record's extension is `.ndjson` where the family writes `.jsonl`. The extension moves, and the
NDJSON wording in the module doc, the README, and the flag's help follows it.

* Two names for one format, and the family settled on the other one.
  - Directory-mode files are stamped `.jsonl`, and every NDJSON in the module doc, the harness
    doc, the flag's help, the dictionary's header line, and the README says JSONL or spells out
    one JSON object per line. The bytes are unchanged, so a `.ndjson` file from before reads
    with the same tools, and file mode never named an extension.

##### feat: host identity in the record closing

Closing out the cycle.

* Acceptance check, run 2026-09-08 against the installed `iiac-perf-dev` 0.28.6-3: passed. The
  one-second `min-now` run wrote `tmp/hid/20260908T005353Z-3900x-min-now.jsonl`, schema version
  4, a `host` object with the seven keys, four cache entries whose sharing lists read `0,12`
  three times and `0-2,12-14` once, `describe-record` printing ten `host.` lines, and full
  validation green with the round-trip test.
* What must outlive the cycle is in the code: the `host` module doc carries the block's rules and
  the dictionary carries each field's meaning, so no `notes/` file gains a section. The notes
  index describes the notes directory, not features, and stays as it is.
* The 7600X follow-up is a continuation note: install the plain 0.28.6 there after Land and
  re-record the `all` run into a directory that stays.
* Close-out shape: trapezoid, the default, pending wink's review of the finished bookmark before
  Land, per the waiver.

# References

[1]: #feat-host-identity-in-the-record-opening
[2]: #refactor-own-the-records-fields
[3]: #feat-probe-the-host-into-a-host-block
[4]: #feat-write-records-as-jsonl
[5]: #feat-host-identity-in-the-record-closing
[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[75]: /notes/chores/chores-05.md#settle-time-is-not-a-grade
[84]: /notes/chores/chores-06.md#docs-experiment-in-the-local-agent-files
