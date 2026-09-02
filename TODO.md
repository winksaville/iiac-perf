# Todo and cycle record

This file contains near term tasks with a short description and reference links to more details.
Its shape is [Todo format](agent-data/notes.md#todo-format).

## Continuation notes

Where the agent was, for the agent that comes next: working copy state, the step in flight, an
open question. Ephemeral, never a record. Written before a restart or when a session is about to
lose context, read first at acquaint, acted on, and reset to `_None._` by the reader.

_None._

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. The block's
shape is the specimen in [cycle-model.md](agent-data/cycle-model.md), and the rules are in
[The In Progress block](agent-data/notes.md#the-in-progress-block).

### feat: crossbeam baseline benches

#### Problem

zc-ring-x1's forthcoming segmented SPSC will need ecosystem baselines to land against, and the
registry has none for an unbounded queue: std's `mpsc`, crossbeam underneath since Rust 1.67, is
the only channel measured, and no segmented queue is measured at all.

#### Solution

Four benches mirroring `mpsc-1t` / `mpsc-2t`: `cb-chan-1t` / `cb-chan-2t` over
`crossbeam_channel::unbounded()`, the familiar number, and `cb-seg-1t` / `cb-seg-2t` over
`crossbeam_queue::SegQueue`, the ecosystem's unbounded segmented queue and the closest structural
peer to what zc-ring-x1 is building. The report guide's `all` table gains their rows and the
capability-class sentence, `SegQueue` and the channel MPMC, std's MPSC, zc-ring-x1's SPSC, so
adjacent rows are not misread.

#### Acceptance check

`iiac-perf-dev cb -d 2` runs all four benches to a report, `iiac-perf-dev cb-seg-2t --pin-cpus
0,1` produces a graded report, the `all` table in `docs/report-guide.md` carries four new rows
with the class sentence beside them, and `vc-x1 validate` passes.

#### Ladder

- [feat: crossbeam baseline benches opening][1] (done)
- [feat: add the cb-chan-1t and cb-chan-2t benches][2] (done)
- [fix: name the binary from the package name][6]
- [feat: shorten the no-bench listing to the bench names][7]
- [feat: add the cb-seg-1t and cb-seg-2t benches][3]
- [docs: place the crossbeam rows in the report guide][4]
- [feat: crossbeam baseline benches closing][5]

#### Deliberation

- Multi-step: two dependencies, two queue shapes, and a docs rung that wants measured numbers,
  each reviewable alone.
- The title is shortened from the Todo entry's, which at 60 characters runs past the 50-character
  cap.
- Wait policy: `cb-chan-2t` blocks in `recv` like `mpsc-2t`, so that pair isolates the std
  wrapper's cost over the same crossbeam code. `SegQueue` has no blocking API, so `cb-seg-2t`
  spins on `pop` like `zcr-mpsc-2t`, and its peers are `mpsc-2t-spin` and the zcr 2t rows, not
  `mpsc-2t`. The report guide says so.
- Dependencies: `crossbeam-channel 0.5`, already in the lock at 0.5.15 through `hdrhistogram`,
  and `crossbeam-queue 0.3`, new.
- Dev rename, the rule's first use here: the installed `iiac-perf` is wink's tool and every rung's
  full validation installs, so the opening renames the package to `iiac-perf-dev` and Land
  restores it. The name is only in string literals, so nothing in the build reads it.
- 0.27.0: minor, new benches.
- The open bug "2t benches accept a 1-CPU pin pool and livelock through spin handoffs" applies to
  the new 2t benches and stays open.
- Two rungs inserted after the channel pair (wink, 2026-09-02), unplanned work inside the cycle
  since both surfaced while driving the new benches: the binary hard-codes `iiac-perf` in its
  banner, completion spec, spec file name, and the spec's `--list-benches` call, so the dev build
  has no completion and misnames itself, the dev-name rule's first-use finding here. And the
  no-argument listing repeats `-h`'s Commands block, so it is cut to the banner, the hint line,
  and the bench names.
- Numbers for the docs rung, still open: the `all` table is one 0.23.0-7 run on the 3900X, and
  four rows from a later run on another box would mix runs. Either the rows carry their own run
  note or the whole table is re-run.

#### Ladder details

##### feat: crossbeam baseline benches opening

The cycle's setup commit: the bookmark, `## Closed` emptied, this block, the Todo entry moved in,
and the package bumped to its `-0` under the dev name.

##### feat: add the cb-chan-1t and cb-chan-2t benches

The registry has no crossbeam channel, so `mpsc` against crossbeam is the same code through two
APIs with the std wrapper's cost unmeasured.

* The two benches are `mpsc-1t` and `mpsc-2t` over `crossbeam_channel::unbounded`.
  - Same struct shape, same blocking `recv` on both ends of the 2t bench, same `Drop` that
    disconnects the request channel to stop the worker, so the only difference between each
    pair is the API, and the difference in the numbers is the std wrapper.
  - Registered after the probe family and before the ice benches, so `cb` as a prefix resolves
    to the crossbeam set and the display order groups by transport.
* Each bench's doc comment names its capability class, MPMC, and its spinning peers, so the
  class sentence lives with the code as well as in the report guide.
* `crossbeam-channel 0.5` becomes a direct dependency, resolved to the 0.5.15 already in the
  lock, so the build pulls nothing new.

##### fix: name the binary from the package name

The binary's name is a literal in six places, so a build under the dev name calls itself
`iiac-perf`, writes the stable binary's completion spec, and gets no completion of its own.

##### feat: shorten the no-bench listing to the bench names

Running with no arguments prints the bench names and then the whole Commands block `-h` already
carries, so the list a user came for scrolls off.

##### feat: add the cb-seg-1t and cb-seg-2t benches

No segmented queue is measured, so zc-ring-x1's segmented SPSC will have no structural peer to
land against.

##### docs: place the crossbeam rows in the report guide

The `all` table would show MPMC, MPSC, and SPSC rows side by side with nothing saying a queue that
promises less is allowed to be faster.

##### feat: crossbeam baseline benches closing

Closing out the cycle.

## Closed

The last cycle's finished record, moved here whole by its closing commit and deleted by the next
opening ([Cycle-record](AGENTS.md#cycle-record)). Earlier cycles are in the landmark commit's
copy of this section, and the cycles before the rule in the frozen [notes/chores/](notes/chores)
and [notes/done.md](notes/done.md).

## Waiting

Important work that cannot start yet. Each entry names what it waits on and its rank once
unblocked, and every opening checks the conditions.

_None._

## Todo

Entries are in priority order, the first highest, and reprioritizing moves the entry. The
long-tail backlog is in [todo-backlog.md](notes/todo-backlog.md), and deeper detail lives in
the frozen `notes/chores/` design subsections, linked by `[N]` refs.

### Vyukov's unbounded SPSC and zc-ring-x1's SPSC v1

The two implementations the crossbeam baselines exist to frame (wink, 2026-08-28).

- Vyukov's is the node-based unbounded SPSC from 1024cores.net, with producer-side node
  recycling (`head`, the free-list `first`, the cached `tail_copy`, and the shared `tail`) so
  the steady state never calls the allocator. No crate is that algorithm, so it is unsafe code
  we would own and maintain inside a measurement tool, which is the real cost of the entry
- it waits on the crossbeam baselines landing, and on zc-ring-x1's `feat: segmented seam-word
  SPSC v1` shipping, so both implementations can be measured against the same baselines in one
  pass
- the interesting axis falls out of the designs rather than being invented: Vyukov's avoids the
  allocator by recycling nodes and zc-ring-x1's by drawing segments from a Pool, so each has a
  cold path that allocates and a steady path that does not. The block and warmup knobs already
  separate those, so the honest report is two numbers per queue

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
  warmup become knobs" rung (defaults zero, replication rows gated on a nonzero sleep).
  Still this entry's: `--blocks` gains a config key so a box's config can run blocks = 1000,
  and the flip-zone hazard stays the range-over-fixed argument (fixed 0.5 ms sleeps
  straddled both 3900X states, D grade, LSC 6x worse)
- the flip zone measured on the 7600x (wink, 2026-08-20, `min-now --blocks 100` sleep
  series): 0 and 1 ms sleeps hold the fast state (16.2 ns, A), 1 s holds the bursty state
  (18.3 ns, A), and 100 ms lands the transition inside the run at ~3.3 s, graded F by env
  and run step at the same instant, ~7.7% of samples still in the fast state and
  resolution honestly widening 0.01 -> 0.41 ns. A/B sleeps go on either side of the flip
  zone, never in it, and grade F vetoes the straddlers
- CI95 / LSC rows always print, `-` when replication is too thin to quote: display gate ~10
  blocks (the t multiplier is 12.7 at df 1, 2.26 at df 9, flat after); plain runs show `-`
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
  error bar that low counts can fake by luck; the mean is state-conditional and deliberately
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
- tooling: `vc-x1 push <bookmark>` already takes any bookmark; landing is two jj commands and
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
  prevent: a trait to report, not a disqualification; the 7600x dwell case that motivated
  the gate is fixed by the dynamic-warmup cycle
- entangled with "Qualify the environment without a bench" (below) and machine-readable
  output; wants the blocks-knobs entry (above) landed first

### Seam-clock attribution

Sample `cpuinfo_avg_freq` at batch seams (the reader exists, `src/freq.rs`) so a mid-run step
gets a "clock moved" label, the way warmup now separates a dwell from the top; also the natural
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
- **the warm's character is the open question.** A probe-driven warm is light; on hardware
  where a heavy workload drives a different clock/power state (AVX offsets), a light warm
  would qualify the box for work it will not do. Moot on the 3900X and 7600x, where `min-now`
  *is* essentially the probe, so decide it with a measurement on a box where it isn't
- **respawn or loop** is a second question, not this one: respawning resets process-local
  state (address space, caches, allocator) and loops do not, but neither resets the machine's
  P-state; what re-rolls that is the gap and the duty cycle. If the answer is loop, the
  results stay structured data and never become text
- coordinate with the "Dynamic warmup" Todo, which owns the convergence rule this would warm
  by, and with the grade-block columns entry, which reformats the table this prints [[75]]

### Guard undersized pin pools and deadline the estimate phase

Guard `--pin` pools smaller than the bench's thread placements: `zcr-mpsc-2t --pin 8` put both
spinning software threads on one logical CPU and appeared hung until ^C (2026-07-26, bug #1 in
[bugs.md](notes/bugs.md#bugs)).

- track `core_for` requests in `RunCfg` (max `thread_idx` asked for); refuse the run when
  placements exceed unique CPUs in the pool. Placement only goes through `core_for` when
  pinning is active, so the guard covers every path, and no pinning means the scheduler
  separates the spinners itself
- wall-clock deadline on the open-loop 5x1,000-step estimate phase so *any* pathologically
  slow bench aborts with a diagnostic naming per-step cost and pinning, instead of hanging

### Move the batch seam's work off the measuring thread

Use the FastForward-style SPSC ring. The batch flush stops the bench for ~1-2 ms (a
`select_nth_unstable` over up to 65,536 values plus 65,536 histogram records) every 50 ms, so
~2-4% of a run is spent at seams. Hand the filled buffer to a consumer thread that sorts,
summarizes and records while the producer fills a second one; the seam drops to a pointer swap.

- the payload is one word, a buffer offset, the exact shape `ffq` is built for, and the
  project dogfooding the queue it benchmarks
- double-buffered: at ~1-2 ms of work per 50 ms batch the consumer runs ~30x faster than it
  needs to, so two buffers never back up
- honest cost: the consumer's cross-core traffic runs *during* measurement, trading a gap on
  the hot core for background L3 pressure. Measure it the way the -4 seam probe was measured
  (interleaved A/B, pinned, trimmed mean) rather than assuming
- blocked on the ring existing; see the "FastForward-style SPSC ring" entry, currently on the
  `ffq-spsc-notes` bookmark rather than `main`

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
~133 ns same-CCX (shared L3), ~633 ns cross-CCX (shared fabric only); cross-CCX vs cross-CCD
differed by 1.6 ns against a ~2 ns LSC, so the L3 boundary is the only fabric tier that matters
on Zen 2, and the unpinned scheduler's ~127-135 ns core mass matches same-CCX placement.

- standardize terms by shared resource, vendor structures as examples only: **lCPU**
  (kernel-schedulable execution context, the `--pin` unit), **core** (lCPUs sharing L1 and
  the execution engine), **cluster** (cores sharing a mid-level cache: Intel E-core module,
  ARM DynamIQ; absent on AMD), **LLC domain** (cores sharing last-level cache: AMD CCX),
  **package** (LLC domains sharing on-package fabric), **NUMA node**. Levels a machine lacks
  collapse out; the tree may be asymmetric (hybrid parts have levels only on some branches);
  matches the kernel sched-domain ladder SMT/CLS/MC/PKG/NUMA
- core *type* (big.LITTLE, P/E cores) is an attribute of a core, not a level: cluster
  identical (part id, capacity, max freq) cores into classes and report the classes; read
  `cpu_capacity` (ARM/RISC-V arch_topology), `/sys/devices/cpu_core/cpus` +
  `/sys/devices/cpu_atom/cpus` (Intel hybrid), part id + `cpuinfo_max_freq` as fallback;
  avoid the big/LITTLE branding (DynamIQ ships 3-4 tiers)
- discovery is unprivileged sysfs: partition lCPUs by `cache/index*/shared_cpu_list` per
  cache level, plus `topology/{thread_siblings,cluster_cpus}_list`, `physical_package_id`,
  `/sys/devices/system/node`; cacheinfo is populated on x86_64 and arm64, patchy on RISC-V,
  so fall back to topology files and mark cache levels unknown
- the Setup `bench pin` line reports the pool's partition and nearest shared level, e.g.
  `[0, 12] (2 slots, 2 lCPUs on 1 core - shared L1/L2)`; retires bare "CPU" from all output
- auto profiles derived from the discovered tree (`--pin smt`, `--pin llc`, `--pin xllc`) so
  one command line is portable across boxes; extends the config `[profiles]` mechanism
  `--pin` already resolves
- **placement tracking** (added 2026-08-01): when unpinned, placement is the dominant factor
  (4-18x on the 3900X) but is currently invisible; observe it instead of only controlling it.
  Two tiers of knowledge, and the report says which one a claim comes from:
  - cooperative (exact): threads placed through the `--pin` pool are known
  - observational (complete but sampled): a bench need not announce threads or
    sub-processes, and the kernel tells us anyway: sweep `/proc/self/task/` at batch seams
    (last-ran lCPU is `stat` field 39; children via `/children`, recursively). CPU-time
    deltas between sweeps identify the active threads with no cooperation; `sched_getcpu`
    (vDSO-cheap) covers the measuring thread exactly. Sampled truth: migrations inside a
    batch and threads born and dead between seams are unseen, which matches the step
    detector's batch granularity; cost is ~us per seam against a 1-2 ms seam
  - batches gain a placement-class label, so a placement migration becomes an *attributed*
    step ("cross-LLC -> same-core"), the way the env grade attributes DVFS
  - unpinned `--blocks` runs stratify block stats by placement class instead of one smeared
    CI: the scheduler's wandering becomes a free stratified experiment (how the SMT fast
    mode was found)
- subsumes the vocabulary half of "Tighten thread and CPU terminology" (above): keep its
  software-thread vs lCPU distinction, adopt lCPU as the standard term

### Rebase web-claude-tweaks onto post-0.22.0 main

It rewrites an already-published bookmark (needs approval) and its arbitrary `0.21.0-b`
version needs replacing; owed from the 0.22.0 close-out plan.

### Unit scaling in report columns

`us`/`ms`: per-row auto-scale so columns stay eyeball-comparable (bands are monotonic, so a
row's first/last/mean share a magnitude), or `--units ns|auto` for script-stable output; needs
`--decimals` landed first (`3.18 ms` vs `3 ms`); candidate `-4` for the report-options cycle.

### Machine-readable report output

`--format json`, or key=value lines to stay dependency-light. Design once the batch gauge lands
(0.23.0-4) so the schema covers the surviving surface: report stats, gauge signals, letter.
Consumers: `tests/qualify_environment.rs` (drops its brittle-but-loud line parsing),
placement-map validation runs, cross-run comparison scripts. Kin to the unit-scaling entry's
`--units ns` script-stable concern (above), one flag family.

### Trimmed core stats

`mean/stdev p10-p90` report row, additional to (never replacing) `mean` / `mean min-p99`; trim
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
  [[61]]); a contaminated run is currently only detectable by squinting at band ranges.
- Superseded pointer: the 0.22.0-5 calibration-time grade certified the ~1 s window before
  the run;
  [Replanning II](notes/chores/chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run)
  moves grading onto the run itself. This entry's crossover and rate analysis is absorbed
  by Todo #1's batch design, which supplies the time axis the histogram lacks.
- Pairs with the trimmed-core-stats entry above: that one needs a defensible upper bound,
  and this is how to find one per run instead of hardcoding p99.

### Investigate the suspend gap missing from samples

A 0.13.5 `--no-inhibit` suspend test detected ~1.2 s suspended inside the measured window but
the max sample was only 4.0 ms, while the 0.13.1 test (8.4 s gap) showed the expected 10.4 s
max sample. We think minstant's TSC may halt across some suspends and count through others.
Repeat the test comparing detected gap vs max sample; if the TSC halts, per-sample timing
silently loses suspend time; document either way.

### CLAUDE.md governance model

Design cogitation.

### Revisit probe adjustment

Under the in-interval vs call-to-call split: probes take one call per sample (inner=1), so the
in-interval timer slice is unamortized and unmeasurable, so an `adjusted` column can subtract
nothing defensible; maybe state a bound instead
[analysis](notes/design.md#timer-overhead-in-interval-vs-call-to-call).

### Convert harness and Bench to probe-based measurement

Will likely need inner-loop support on `Probe` (batch N calls per sample; report divides by N
and accounts for per-sample framing) so very-small workloads can still amortize timer overhead
the way `run_adaptive` does today.

### Rename app

### Design an app to measure IIAC performance

Written in Rust.

### ice-ps-2t-wait

iceoryx2 pub/sub with blocking waits via `Listener`/`Notifier` events; completes the
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

`iiac-perf` -> general-purpose name (breaking; deferred).

## Ideas

Longer-range thoughts, not yet ranked work. `-` bullets, no numbering; promote into `## Todo`
when one becomes actionable.

- Per-bench dependency isolation, motivated by dep provenance: the deps are the thing being
  measured, so a dep bump (e.g. iceoryx2 0.9.2 -> 0.9.3) legitimately moves that bench's
  numbers and shouldn't ride in silently. Options considered (2026-07-08):
  - Caveat first: a Cargo **workspace shares one Cargo.lock** across members. It scopes deps
    per package (ice benches alone pay for iceoryx2; faster `-p` builds; harness/probes become
    a library crate) but does *not* give per-bench lock isolation, and it splits the single CLI
    into many binaries.
  - Targeted updates (`cargo update -p <crate>`, never bare `cargo update`): ~90% of the
    provenance benefit at zero structure cost; adoptable immediately as discipline.
  - Feature gates (`--features ice`): solves build weight in the current single package, not
    lock isolation.
  - Truly standalone crates (own Cargo.lock each): the only real per-bench dep isolation;
    maximum maintenance, and cuts against "same harness, same build" A/B comparability.
  - Current lean: targeted-update discipline now; feature gates or workspace only when bench
    families multiply.
- clap CompleteEnv dynamic completion (the `unstable-dynamic` feature): clap's native runtime
  completer (`COMPLETE=bash iiac-perf`) would give bash die-hards a compact column view without
  carapace; revisit if clap stabilizes it.
- Stability selftest mode (2026-07-27): grade the environment more thoroughly than a single
  run's gauge: a product subcommand that respawns its own binary (`current_exe()`) N times at
  configurable cadences and reports cross-run gauge agreement ("is this box currently
  trustworthy for A/B?"). Precedent in-product: the calibration repeat self-check and
  `--blocks` both already validate by orchestrated repetition; this is the next ring out.
  Subsumes `tests/qualify_environment.rs`'s orchestration: the test reduces to asserting on the
  verdict, and its env-var knobs become clap flags. Concrete motivation (2026-07-27): the
  qualification test can't run on the 7600x, which has only the installed binary, and
  environment qualification shouldn't require a source tree. **Promoted 2026-07-28**: the
  minimal version is the 0.23.0-6 ladder rung (`qualify-environment subcommand`); what remains
  here for later is the fuller mode: cadence sweeps, richer cross-run reporting.
- Cold-start mode (2026-08-02): blocks deliberately shields the coldest samples (the 2 ms
  post-wake warm is unrecorded), so the true first-call-after-sleep cost never lands in the
  histogram; a mode that records or separately reports post-wake samples would measure the wake
  cost applications actually pay ("--blocks 1000 feels more real", taken one step further)
- Tick-phase avoidance (2026-07-27): the scheduler tick is periodic per-CPU (~300/s at
  CONFIG_HZ=300) and a tick hit is an unmistakable outlier, so predict the next tick from
  detected hits and pause measuring ~30 us around it, at ~1% duty cost, no governor exposure
  with governor+EPP `performance`. Doesn't improve the bulk stats (tick hits are already
  detected and trimmed); buys a cleaner above-crossover tail on unmodified machines, so rarer
  aperiodic events (device IRQs, SMIs, code slow paths) become visible over the periodic
  contaminant. Check the interaction with dither (anti-phase scheduling must not introduce a
  systematic phase bias), and compare against `nohz_full`/`isolcpus` isolation (which abolishes
  the tick on a dedicated core and strictly dominates when a reboot is allowed) before
  building.

## Bugs

_See [bugs.md](notes/bugs.md)._

# References

[1]: #feat-crossbeam-baseline-benches-opening
[2]: #feat-add-the-cb-chan-1t-and-cb-chan-2t-benches
[3]: #feat-add-the-cb-seg-1t-and-cb-seg-2t-benches
[4]: #docs-place-the-crossbeam-rows-in-the-report-guide
[5]: #feat-crossbeam-baseline-benches-closing
[6]: #fix-name-the-binary-from-the-package-name
[7]: #feat-shorten-the-no-bench-listing-to-the-bench-names
[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[75]: /notes/chores/chores-05.md#settle-time-is-not-a-grade
[84]: /notes/chores/chores-06.md#docs-experiment-in-the-local-agent-files
