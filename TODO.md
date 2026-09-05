# Todo and cycle record

This file contains near term tasks with a short description and reference links to more details.
Its shape is [Todo format](agent-data/notes.md#todo-format).

## Continuation notes

Where the agent was, for the agent that comes next: working copy state, the step in flight, an
open question. Ephemeral, never a record. Written before a restart or when a session is about to
lose context, read first at acquaint, acted on, and reset to `_None._` by the reader.

- `agent-files(proposal): v0.2.3` is **paused**, its work saved as the local commit on the
  local-only bookmark `wip-v023`, never pushed. It was set aside so this adoption could start from
  a clean `main`, since the two cycles want the same bookkeeping files.
  - resuming takes only the `agent-data/notes.md` hunk from `wip-v023` and redoes the bookkeeping
    against the adopted base. `notes.md` is byte-identical between `v0.2.0` and `v0.2.2`, so the
    rewrite still answers the section it was written for
  - patch, not minor: no rule changes, an adopter behaves identically under either text
    ([Advancing X.Y.Z](agent-data/versioning.md#advancing-xyz-patch-by-default)). vc-x1 ran the
    same test against their own `v0.2.2` and reported it does not bend toward them
  - numbered three times, all before a commit: `v0.3.0`, then `v0.2.2` on the patch reading, then
    `v0.2.3` when vc-x1 claimed `v0.2.2` and we conceded it
  - the bookmark `agent-filesproposal-v023` is published at `8d0133a3` and carries no commits
- Validation and install go through `vc-x1-dev validate`.
- Four records from vc-x1 arrived 2026-09-05 and are read but not acknowledged: the section-order
  acceptance, `v0.2.1 and v0.2.2 landed`, the cross-file-links answer, and `vc-x1 reads owner`.
  Acknowledging them is owed.
- The `owner` rename's phase two needs zc-ring-x1 only. vc-x1 has confirmed it reads `owner`, so
  when zc-ring-x1 does, `.owner` goes and the README's transition clause is retired.
- Still owed to us: vc-x1's call on the done-marks branch `readme-carries-done-marks`.
- Still to run: the port-and-bug cycle, which creates `notes/perf-findings.md` for the 7600x
  numbers below, appends the `iiac-perf-dev` clause to `notes/ops.md`'s 7600x bullet, writes the
  `restore-freq` entry into `notes/bugs.md`, and adds the "Windows and macOS port considerations"
  Todo entry.
- The 7600x's `[freq]` block omits `min_mhz` / `max_mhz`, so a `restore-freq` there widens the
  clamp to the hardware floor: on 2026-09-04 it went from 2.99 GHz to 427 MHz, and had returned to
  2.99 by 04:14 through a path nobody identified. The values to declare are `min_mhz = 2991` and
  `max_mhz = 5457`, and a `read-freq` is worth running before trusting that box's numbers. Nothing
  else records these two numbers or that episode, so this bullet is their only copy until the
  port-and-bug cycle files them.
- The agent-files name `TODO.md` 23 times, surveyed 2026-09-04. Fourteen are the cycle-record's
  address (`TODO.md > ## In Progress` and kin) and must stay. Five are illustration, which is
  where the staleness lives, and `notes.md > ## Reference numbering`'s file enumeration is the one
  the paused `v0.2.3` removes. One bare mention remains in `notes.md`'s `## Bugs` pointer line,
  also fixed there. `commit-model.md`'s bare mention is inside a commit-body specimen and is left
  alone. The set owning the file's shape while disclaiming its content is deliberate and stated in
  AGENTS.md's Terminology, so only the illustration should go.

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. The block's
shape is the specimen in [cycle-model.md](agent-data/cycle-model.md), and the rules are in
[The In Progress block](agent-data/notes.md#the-in-progress-block).

### feat: zcr-v1-1t/2t benches

#### Problem

zc-ring-x1's seam-word SPSC v1 exists on its `feat-segmented-seam-word-spsc-v1` bookmark and
nothing here measures it. The `zcr-with` pair reads the v0 ring, whose two bounced index lines v1
was designed to retire, and the crossbeam baselines landed 2026-09-02 to frame exactly this ring, so
until v1 is in the registry the v0, v1, and mpsc comparison exists only in zc-ring-x1's own demo,
under a different harness.

#### Solution

Advance the `zc-ring-x1` dependency to the v1 bookmark and add `zcr-v1-1t` / `zcr-v1-2t`, mirroring
the `zcr-with` pair over `spsc::v1::Ring`, with their rows in the report guide's `all` table.

#### Acceptance check

`iiac-perf-dev zcr -d 2` runs six benches to a report, `iiac-perf-dev zcr-v1-2t --pin-cpus 0,1`
produces a graded report, the `all` table in `docs/report-guide.md` carries the two new rows with a
run note beside them, and `vc-x1-dev validate` passes.

#### Ladder

- [feat: zcr-v1-1t/2t benches opening][1] (done)
- [chore: point zc-ring-x1 at the spsc v1 bookmark][2] (done)
- [feat: add the zcr-v1-1t and zcr-v1-2t benches][3] (done)
- [docs: place the zcr-v1 rows in the report guide][4]
- [feat: zcr-v1-1t/2t benches closing][5]

#### Deliberation

- **Approvals waived for this cycle** (wink, 2026-09-05): "complete it as you see fit, I'll review
  the completed benches when I get home", read as a scoped waiver of the per-push approvals and the
  two per-rung reviews up to the close-out, recorded here as [Rules](AGENTS.md#rules) asks.
  - Land is not covered: the cycle stops at its closing push, bookmark unlanded, for the review.
- **Benches here, not in zc-ring-x1.** The request names spsc v1 "in ../zc-ring-x1 on a branch",
  read as where the ring lives: the registry is this repo, and zc-ring-x1 has its own agent-files
  and a cycle mid-ladder on that bookmark.
- **Split from the Vyukov half.** The Todo entry paired both implementations for one pass against
  the same baselines, and v1 is the half that exists, so Vyukov's stays as its own entry.
- **Dependency on the bookmark, not `main`.** The bookmark carries the dev package name, so the
  dependency is `package = "zc-ring-x1-dev"` with `branch = "feat-segmented-seam-word-spsc-v1"`,
  and the lock pins the commit.
  - a `## Waiting` entry re-points at `main` once zc-ring-x1 lands the cycle, written at the closing
  - the crate root still re-exports every name the v0 and mpsc benches use, checked by diff before
    the opening, so advancing should not touch them
- **Names `zcr-v1-1t` / `zcr-v1-2t`.** `zcr-with` is v0, named for its API tier when that was the
  only ring, so the ring version is the axis that distinguishes the new pair. The 2t waits spin like
  every zcr row.
- **Numbers on the 3900X, own run note.** The `all` table is one 0.27.0-5 run on the 7600X, and
  this cycle runs on the 3900X, so the two rows cannot join that run. They carry their own run note,
  and the same run measures the `zcr-with` and `zcr-mpsc` pairs beside them so the reader gets the
  v0, v1, and mpsc comparison from one run.
- **Version 0.28.3**, patch by default.
- **Continuation notes trimmed, not reset.** The stale bullet, the landed adoption cycle, is
  dropped. The others hold facts with no other home yet, the 7600x frequency numbers among them,
  and their filing is the port-and-bug cycle's work.

#### Ladder details

##### feat: zcr-v1-1t/2t benches opening

The cycle's setup commit: create and publish the bookmark, delete `## Closed`'s contents, move the
Todo entry into this block, rename the package to `iiac-perf-dev`, and bump the version-of-record.

##### chore: point zc-ring-x1 at the spsc v1 bookmark

The dependency was pinned to a `main` commit from before v1 existed, and the bookmark that holds v1
carries the dev package name, so a plain `branch` key would name a package the bookmark lacks.

* The dependency names a package the bookmark does not carry.
  - `package = "zc-ring-x1-dev"` beside the `branch` key, so the crate is still `zc_ring_x1` at
    every `use` and the lock pins the bookmark's commit, `0bb201ee`
  - the dependency moves into its own `[dependencies.zc-ring-x1]` table, since three keys and the
    comment explaining them no longer fit one line
* The four existing zcr benches were written against 0.11.1, and the crate has been reorganized
  into versioned modules since.
  - the crate root still re-exports every name they use, so 0.15.5-3 builds them unchanged and
    `iiac-perf-dev zcr -d 1` runs all four
  - the lock entry changes name, `zc-ring-x1` to `zc-ring-x1-dev`, which is the one visible trace
    of the bookmark's mid-cycle state and reverts when the dependency re-points at `main`

##### feat: add the zcr-v1-1t and zcr-v1-2t benches

Nothing measured v1, and the shared zcr setup had no way to build a v1 region, whose layout adds a
seq array between the header and the slots.

* The seq array's stride is still being probed on the bookmark, packed or one line per seq.
  - `leak_v1_ring` sizes the array at one line per seq, the widest, since `Ring::init` accepts a
    region larger than it needs. The surplus is 448 B, leaked once per ring like the rest
* The pair should differ from `zcr-with` in the ring alone, so a v0 to v1 difference in the table
  is the protocol's.
  - same `Msg`, `CAPACITY`, `STOP` sentinel, wait closures, and echo-worker shape, with only the
    endpoint types changed to `spsc::v1`
  - the `expect` sites carry `// OK:` comments, the first in the zcr benches to do so, and the
    older sites are left as they are
* Touching `zcr_common.rs` and `mod.rs` brings their comments under the prose rules.
  - one comment semicolon, three em dashes, and two arrows convert in this commit, whole file,
    which is the rule's scope

##### docs: place the zcr-v1 rows in the report guide

The `all` table has no v1 rows. Run the six zcr benches together on this box, add the two rows with
a run note, and say what the same-run comparison shows.

##### feat: zcr-v1-1t/2t benches closing

Closing out the cycle.

## Waiting

Important work that cannot start yet. Each entry names what it waits on and its rank once
unblocked, and every opening checks the conditions.

_None._

## Todo

Entries are in priority order, the first highest, and reprioritizing moves the entry. The
long-tail backlog is in [todo-backlog.md](notes/todo-backlog.md), and deeper detail lives in
the frozen `notes/chores/` design subsections, linked by `[N]` refs.

### Host identity in the record

A record names its box by hostname alone, so a file read on another machine cannot say what CPU,
topology, memory, kernel, or toolchain produced it, and cross-host comparison is by memory
(found 2026-09-02 reading the 7600X `all` run's records). Schema version 4 adds a host block.

- readable without root: CPU model and microcode from `/proc/cpuinfo`, sockets, cores, threads,
  SMT, and the L3 domains from sysfs under `/sys/devices/system/cpu/` (the CCX map a 2t
  placement note needs), `MemTotal` from `/proc/meminfo`, board and BIOS from
  `/sys/class/dmi/id/`, the kernel from `uname`
- root only: memory speed and channel count live in the DMI tables (`dmidecode -t memory`), so
  they are a `[host]` config declaration pasted once, the way `read-freq --as-config` fills
  `[freq]`
- build-time: rustc's version and the target-cpu flags are not in cargo's environment, so a
  `build.rs` bakes them in
- same cycle: the record file extension moves from `.ndjson` to `.jsonl`, the name the family
  already uses (the agent-repo's session files, vc-x1's records), the format being identical
- afterwards, re-record the `all` run on the 7600X into a directory that stays, since the
  2026-09-02 records were deleted rather than kept in the old shape

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
  was the wrong shape. The re-recording the host-identity entry above asks for wants 3-5 runs per
  cell, interleaved, which is what the guide has said all along
- the output reuses the report's row names, so the guide decodes the new surface for free. Grading
  the set the way a run grades itself is the natural extension: do these runs agree, and is a
  disagreement drift, a step, or one bad run
- `--format csv` / `--format json` for the plotting hand-off, kin to "Machine-readable report
  output" below, one flag family
- ranked here, immediately after the host-identity entry, because it reads the record: schema v4
  changes the shape and the file extension under it, and cross-host analysis needs the host block
  that v3 lacks, a hostname alone naming nothing

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

### Punctuation conversion lands in a penultimate rung

[Semicolons](agent-data/prose.md#semicolons) says a commit that edits a file "converts that whole
file's prose semicolons in the same commit", and [Typeable punctuation
only](agent-data/prose.md#typeable-punctuation-only) says the same for dashes. So the conversion
rides the rung that touched the file, and every hunk of the resulting diff has to be read to tell
repunctuation from a real change (wink, 2026-09-03, reading this cycle's docs rung).

- set-level rather than project-level, so it is an `agent-files` proposal cycle and the diff
  against the payload is the proposal
- the draft: prose-punctuation conversion owed by a touched file is paid in a penultimate rung,
  not in the rung that touched it. New prose is written correct and is never a sweep item. When a
  file's count is large enough that converting means rewriting rather than repunctuating, it
  becomes its own cycle
- it relocates the obligation without loosening it. What is owed still follows from touching the
  file and only the placement moves, so the wording has to keep "whether" and "when" apart or it
  reads as permission to skip
- dashes as well as semicolons, since one rung here swept seven em dashes into a content change
  beside six semicolons
- it names its exceptions rather than hedging with "generally", which tells a reader that a
  deviation is allowed without telling them when
- the ceiling case is being tested by this cycle's own `TODO.md` conversion at 45 instances, which
  stayed a penultimate rung rather than becoming its own cycle only because `## Closed` was empty
  and no landed record would be reworded

### Vyukov's unbounded SPSC

The node-based unbounded SPSC from 1024cores.net, the second implementation the crossbeam
baselines exist to frame (wink, 2026-08-28), zc-ring-x1's SPSC v1 being the first, measured by the
`zcr-v1` pair.

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

The project has two words for one thing. The record field is `host`, "Host identity in the record"
above builds on it, and the prose says "box" about a hundred times, so a reader meeting both is
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
- ranked after "Host identity in the record" above, whose host block is what makes `host` the
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

# References

[1]: #feat-zcr-v1-1t2t-benches-opening
[2]: #chore-point-zc-ring-x1-at-the-spsc-v1-bookmark
[3]: #feat-add-the-zcr-v1-1t-and-zcr-v1-2t-benches
[4]: #docs-place-the-zcr-v1-rows-in-the-report-guide
[5]: #feat-zcr-v1-1t2t-benches-closing

[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[75]: /notes/chores/chores-05.md#settle-time-is-not-a-grade
[84]: /notes/chores/chores-06.md#docs-experiment-in-the-local-agent-files
