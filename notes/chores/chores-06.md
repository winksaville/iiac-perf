# Chores 06

Continuation of [chores-05](chores-05.md). Records landed work; conventions in
[agent-data/notes.md](../../agent-data/notes.md#chores-conventions) and
[cycle-protocol.md](../../agent-data/cycle-protocol.md#chores-sections).

## Table of Contents

- [feat: dynamic warmup](#feat-dynamic-warmup)
- [docs: experiment in the local agent-files](#docs-experiment-in-the-local-agent-files)
- [docs: steps are titles, versions are stamps](#docs-steps-are-titles-versions-are-stamps)
- [docs: one owner per rule, one home per record](#docs-one-owner-per-rule-one-home-per-record)
- [docs: the bot pushes again](#docs-the-bot-pushes-again)
- [docs: sync agent-files from vc-x1's draft](#docs-sync-agent-files-from-vc-x1s-draft)
- [chore: sync cycle records and mailbox sweep](#chore-sync-cycle-records-and-mailbox-sweep)
- [docs: adopt the commit-body form](#docs-adopt-the-commit-body-form)
- [docs: validate every commit](#docs-validate-every-commit)
- [chore: complete the landed records](#chore-complete-the-landed-records)

## feat: dynamic warmup

- [[3]] 0.24.0-0 feat: dynamic warmup opening
- [[4]] 0.24.0-1 refactor: one parameterized warm loop
- [[5]] 0.24.0-2 feat: warm until the trailing window grades A
- [[6]] 0.24.0-3 feat: warm where the bench runs
- [[7]] 0.24.0-4 feat: read the clock during warmup
- [[8]] 0.24.0-5 feat: settle follows the warm window grade
- [[9]] 0.24.0-6 feat: configurable warm cap
- [[10]] 0.24.0 feat: dynamic warmup

The 0.24.0 cycle: replace the fixed `WARMUP = 10_000` step count in `harness.rs` with
warm-until-stable. A fixed count's wall-clock scales with step cost, so the fastest benches warm
~10 us against frequency-governor ramps of tens-to-hundreds of ms and `pick_inner` sizes mid-ramp
(the 7600x F diagnosis,
[Replanning II](chores-04.md#replanning-ii-drop-the-adjustment-grade-the-run)), and a timing-only
"did it end settled" test can issue a vacuous A while the box dwells one P-state below the top
(measured 2026-07-29 [[1]]). The design accumulated in the Todo entry over 2026-07-27 to
2026-08-01; the subsections below are its record, plus the decisions made at pickup (2026-08-02).
First cycle run on a topic bookmark (`dynamic-warmup`).

### One warm loop, three policies

The end state, decided 2026-08-01: one parameterized warm loop. Step the bench, probe
periodically, stop when the exit condition holds. The harness's three warms become policies over
that one mechanism:

- the per-run warmup exits when the trailing window grades A, or at the cap
- the process warm (`process_warm`) exits on the `--settle-time` budget
- the block warm (`run_blocked`'s 2 ms spin) is the same loop with a fixed-time exit and probing
  disabled

`process_warm` and `warmup_and_probe` already share one probe series, prober and time origin;
this completes the fusion instead of adding a fourth variant.

Terminology: the warmup unit is a **warmup pass** (the 0.22.0-4 "loop-only passes" sense), a
short, unrecorded, timed burst of bench steps yielding one floor. Not a "probe": `TProbe` is the
measurement instrument, and the "micro-probe" is the 0.23.0 cycle's ~1 ms timer-pair frame
measurement.

### The exit condition: grade the trailing window

Rather than K agreeing floors, grade a *sliding window* over the warmup probe series and warm
until the trailing window reads A, or the cap (design 2026-07-28). The exit condition and the
warmup letter become one computation:

- exit on A means the run started post-ramp by construction
- hitting the cap reports whatever the window actually scored, the "run started unstable" signal,
  not a silent proceed
- window length takes the same "minimum count or minimum wall time, whichever is larger" shape as
  pass length: count because the split detector needs 4 points a side, wall time because the ramp
  is a ms-scale phenomenon
- signals: spread, drift, step; `interference` is the weak one [[2]]
- the minimum wall span is load-bearing: a window can be far shorter than what it certifies
  (`min-now`'s 16 warmup probes span ~17 us against a transition arriving at ~800 ms), so
  agreement alone certifies nothing

Floors, not means, so one preemption doesn't fake (in)stability; a warm box exits
near-immediately.

Convergence is agreement, not direction. The 3900X is bistable (2026-07-27 `calibrate` runs):
sustained rapid repetition climbs it into a fast state (~0.445 ns/iter), low-duty isolated runs
sit at ~0.489 (B), ~9% apart, and transitions straddle windows in *both* directions; a fixed
warmup absorbs only transitions shorter than itself, whatever their sign.

The hard cap sits at governor scale (a few hundred ms; exact constant measured on both boxes
during the rung). The cap doubles as the estimate-phase deadline the `--pin` guard Todo wants, so
slow and non-converging benches share one diagnostic exit.

### Sizing fusion

The warmup pass *is* the step-cost sizing pass: the converged floor is the sizing input, so
sizing is post-ramp by construction and convergence is tested on the number actually consumed.
The `estimate_step_cost` phase folds into the warm loop; the micro-probe supplies the frame
input, run after convergence.

Slow steps (`inner` -> 1 territory): pass length adapts (minimum step count or minimum wall time,
whichever is larger) so a floor is never one sample. The cap exit then distinguishes "floors
disagreed" (unstable, gauge signal) from "too slow to certify" (proceed, label the run
uncertified: at `inner = 1` sizing can't be wrong and framing is negligible, so the stakes are
low there).

### Read the clock, not just the timing

Steadiness cannot tell "settled at the top" from "dwelling at an intermediate P-state", because a
dwell *is* steady. Measured 2026-07-29 on the 7600x [[1]]: the trailing window graded A while the
box held 4841 MHz for ~0.75 s, then stepped +12.4% inside the run. This was the strongest
argument for making the clock reading part of the exit condition rather than an optional extra,
and the rung is in scope (decided 2026-08-02).

- delivered frequency is an unprivileged sysfs read on both AMD boxes
  (`cpufreq/cpuinfo_avg_freq`); the ramp is ~150-200 ms
- gate on **clock stability under load**, never on a fraction of `cpuinfo_max_freq`: a threshold
  would need tuning between 96.1% (3900X) and 99.7% (7600x) sustained, and a thermally-limited
  laptop plateaus lower still while that plateau is its honest clock
- optional by construction: `cpuinfo_avg_freq` is amd-pstate-specific and some drivers'
  `scaling_cur_freq` reports requested rather than delivered, so read where present and fall back
  to timing-only
- report the ratio, do not grade on it

`qualify-environment`'s verdict is not usable as a gate until this cycle lands: it reads NOT
QUALIFIED on any amd-pstate-epp box that dwells then boosts, which is to say on a healthy idle
machine [[1]]. Fixing the exit condition fixes the selftest at the same time, since its
observable is this grade.

As built at -4: `src/freq.rs` reads `cpuinfo_avg_freq` on the calling thread's current CPU
(`sched_getcpu`), one sample per warmup probe, kept parallel to the probe series across the
process-warm handoff.

- the exit gains a second gate: timing-A *and* clock held within 1% (`FREQ_STABLE_TOL`) across
  the exit window; a timing-steady window with a moving clock classifies Unstable (the dwell
  case, unit-tested against the 7600x numbers)
- anything short of clean same-CPU readings falls back to timing-only: file absent, read failure,
  or an unpinned main migrating mid-window (samples carry their CPU id)
- the ratio prints on the `-v` warmup summary line (`clock 4093/4674 MHz (87.6%)` measured on the
  3900X, whose honest sustained clock is ~87% of `cpuinfo_max_freq`: live confirmation that a
  fraction-of-max threshold would misfire and stability-under-load is the right gate)
- review point: the ratio is `-v`-only for now; the design said "report it", and the normal
  grade block's columns are parsed positionally by qualify, so adding it there was deferred to
  review

### Placement: warm where the bench runs

Decided 2026-08-02: the warm follows the bench's pin. Warm on `pin[0]` when `--pin` is set, else
wherever the scheduler has main (a busy thread stays put, and the warm state lands on the core
that measures). The CPU0-default tick-rate warm pin and `--no-pin-cal` are deleted rather than
justified.

- CPU0 is measurably the kernel's busiest core on the 3900X (2026-07-29, cumulative per-CPU
  interrupts: 4.1M `LOC` against ~0.6M on CPU11/CPU23, 6.2M `CAL` against ~0.6M, 4-6x the `RES`,
  3x the `TLB`; no `irqbalance`). CPU0 is the boot CPU and the `nohz_full` housekeeping CPU, so
  this is expected rather than a quirk of this box
- the pin did not matter for the tick-rate read (a ratio of TSC ticks to monotonic ns over
  ~10 ms: interruptions inflate both sides and cancel, ~8e-7 spread across cores), so the current
  default was harmless, not correct; it would have mattered here, where warmup becomes a real
  timing phase converging on per-core frequency state
- the rejected alternative, a topology-aware "not the boot CPU, and a full-frequency core" pin,
  adds machinery this cycle does not need; it stays with the topology Todo. "Use the last core"
  folklore was already rejected there (hybrid parts put E-cores at high indices)
- as built at -3: main pins to `pin[0]` (and stays; it is thread 0 of every bench) only when
  `--pin` is given; `--no-pin-cal` and `pin.rs`'s save/restore pair are deleted; the Setup cell
  is renamed `warm pin` -> `main pin`, naming main's placement for warm and run both

### Report shape

Normal output carries one warmup line: letter plus **settle time**, a real machine
characteristic. `-v` shows the complete warmup picture, the per-probe table with the ramp's
shape. The qualification selftest reading a table of settle times across respawns is a better
observable than a table of blended letters.

### Acceptance test

`tests/qualify_environment.rs` (landed 0.23.0-1, simplified to one loop 2026-07-28):
`#[ignore]`d integration test spawning the real binary per run (`CARGO_BIN_EXE`), one loop of 10
back-to-back runs. The loop's own load provokes the transition; verdict: median >= B and zero
runs with drift/repeat at D/F. Reproduced failing 2026-07-27 on the 3900X (repeat F on the climb
run); `IIAC_PERF_BIN` pins a saved failing build. Part of this cycle's close-out validation.

### As built at -2: one window, and what it showed

The -2 rung's as-built decisions, where they refine the design above:

- **the exit window replaced the fixed 300 ms tail** (`WARMUP_TAIL_SECONDS` deleted): the graded
  warmup tail is now exactly the window the exit condition tested (`RunOutput::warm_tail`), so
  the printed letter is the letter the exit saw. The long tail's job (catch a ramp inside a fixed
  budget) is gone because the exit keeps warming until the window is clean
- provisional constants, sized on the 3900X and flagged for the 7600x pass: pass minimums 8
  steps / 1 ms, window minimums 8 probes / 50 ms, cap 400 ms. 50 ms is governor-transition
  scale, not full-ramp scale; the dwell a timing window cannot see at any length is the clock
  rung's job
- the warm stretch's cost moved from ~4 ms fixed to ~51 ms settled (window span + probe
  overhead); the exit is condition driven, so a disturbed box pays up to the cap instead
- the settle cell now answers by exit verdict: a settled exit reports gauge::settle's time, a cap
  exit prints "not settled" (the exit's own finding), a window that never formed prints
  "uncertified" (parsed as blank by qualify's `parse_settle`)
- sizing reads the exit window's best pass (min per-step cost), and the estimate phase is
  deleted; the cap deadlines every adaptive pass, which also retires the estimate-phase hang
  (bugs.md #1's deadline half; the pool-size guard half remains open)
- observed on the 3900X `all` sweep: two benches printed `A` + `not settled`, a window that
  grades A while an 8-median excursion left the 1% settle band inside it (the bistable flicker at
  grade-invisible scale). Truthful but odd on one line; review whether settle's band should align
  with the window grade's thresholds
- resolved at -5 (wink, 2026-08-02): aligned. Settle became the earliest suffix of the warm
  stretch that grades A, scanned front-to-back and never shorter than the exit window, so the
  letter and the settle cell are one computation and cannot disagree; `SETTLE_TOL`,
  `SETTLE_WINDOW` and the forward-median machinery (`window_floors`) are deleted. A settled
  exit always finds a time ("not settled" reaches the report only on a cap exit), and the
  post-change sweep read A rows with settle 0.01 to 0.44 s and no contradictions

### Acceptance run after -4 (3900X, 2026-08-02)

`qualify_environment` run once with all four rungs in, as review data for close-out (the 7600x,
the box the dwell was measured on, still needs its pass):

- the warmup column is what the cycle promised: 9 A + 1 B, no vacuous letters, settle times
  honest (0.01 s warm-box, 0.74 to 1.41 s when the box had relaxed between runs, two runs "not
  settled" at the cap while the box was still flickering)
- the verdict still reads NOT QUALIFIED, now for a run-side reason: mid-run transitions (env
  bench drift/step D/F on 3 of 10) from the bistable trait, which warmup cannot prevent and the
  report truthfully attributes
- close-out questions this raises: is the verdict's "transition-degraded" rule right for a box
  whose trait this is, and should the 400 ms cap sit above the 3900X's ~1 s relaxation re-ramp
  (runs that settled at 1.2 s did so inside the respawned process warm, not the capped per-run
  stretch)
- resolved at -6 (wink, 2026-08-02): the cap default rises to 1.5 s and becomes `--warm-cap` /
  config `warm_cap` (CLI > config > built-in, the `--settle-time` pattern; zero or more, 0 caps
  immediately). The post-change `all` sweep on the 3900X read zero "not settled" rows where the
  0.4 s cap produced two: the relaxation re-ramp is now absorbable. With `--warm-cap 0
  --settle-time 0` no warmup probes exist at all, so the warmup row is absent (no certificate),
  which is distinct from "uncertified" (probes exist, no valid window). The verdict-rule
  question stays open for the qualification redesign
- also at -6, warm visibility (wink, same day): the Setup banner gains a `warm budget` cell
  (`settle 1.5s once + cap 1.5s per run`, the resolved budgets), and each report's header
  bracket gains `warm=used/budget`, this run's total warm spend over its total allowance
  (first run `warm=1.51/3.0s`, settle + cap; later runs `warm=0.13/1.5s`, cap alone). Setup
  prints before any run, so it carries only the budgets. First cut showed the capped stretch
  alone and read `warm=0.00/1.5s` on every settled-by-process-warm first run (wink caught it):
  truthful about the cap, useless about the cost

The 7600x pass (wink, same day, installed binary):

- `min-now` reads straight A's with settle 0.77 s: the warm loop rode through the 4841 MHz dwell
  and exited after the ~0.8 s boost
- trimmed stdev 0.1 ns; the vacuous-A defect is closed on the box it was measured on
- the cross-respawn `qualify-environment` verdict there is still to be run

Duty cycle selects the bistable state (wink, same day, 3900X unpinned):

- plain `-d 5` and `--blocks 100` climb into the fast state (~21.8 ns) and grade F when the flip
  lands mid-run
- `--blocks 1000` (5 ms bursts between 1-10 ms sleeps) holds the slow state (24.0 ns) for 13 s
  and reads straight A's with CI95 0.0 ns
- grade A certifies internal consistency of the state the run held, not a canonical number; A/B
  wants matched duty cycle
- feeds the "Report interpretation guide" Todo's worked examples
- strengthens the seam-clock idea: sample `cpuinfo_avg_freq` at batch seams so a mid-run step
  gets a "clock moved" attribution

The constant-clock control (wink, same day, 7600x, `--decimals 3`): at a held clock, mode does
not move the number, and block count is a tradeoff, not a dial to max out.

- plain vs `--blocks 2`: trimmed means identical to the third decimal (16.196 ns both), full
  means identical (16.236 ns), band values byte-identical; the 3900X mode divergence was pure
  DVFS state
- `--blocks 2` read CI95 0.003 / LSC 0.001 ns, but from one degree of freedom (t = 12.7, a
  single pair of block means): real agreement, fragile interval; quote LSCs from tens of blocks
- `--blocks 1000` on the same box: mean +1.6% (16.503), trimmed stdev 0.110 -> 0.342 ns, LSC
  8x larger (0.008 ns). We think ~5 ms blocks sit close to every wake, so C-state exit residue
  the 2 ms block warm does not fully re-establish contributes proportionally more and
  between-block variance rises
- the ordering is bench-shape-dependent (wink, same day, 7600x `zcr-with-2t`, a spin-partner
  bench): blocks 2 and 20 agree to 0.002 ns (110.835 / 110.837 trimmed) while 1000 blocks' LSC
  0.022 beats 20's 0.083, the reverse of `min-now`. We think the spinning worker rides through
  main's sleeps, so the box never idles between blocks; wake residue shrinks and replication
  (df 999) wins. The +~1% mean shift at 1000 blocks persists
- the sleep budget selects the state too (wink-requested experiment, same day, 3900X, source
  patched to a fixed 0.5 ms sleep, 1000 blocks): ~67% duty (5 ms measure / ~2.5 ms gap) landed
  in the unstable middle: the box straddled both states (band mass split across 21.8 and
  24.0 ns), a 9.3% step at 5.6 s graded D on both series, and LSC 0.143 ns, 6x worse than the
  1-10 ms sleeps' 0.023. The 1-10 ms budget (~40% duty) holds the slow state; sustained load
  holds the fast one; between them is the flip zone. A `--block-sleep` knob would make this
  explorable without patching source (idea)
  - for change detection, tens of blocks: replication df with per-block cleanliness, the
    tightest defensible LSC; even 1000 blocks' 0.008 ns is 0.05% of the mean
  - for representativeness, high block counts are *more* real, not noisier: real IPC usage is
    bursty (wake, exchange, go quiet), so the +1.6% and the wider tail are the delivered cost
    of a deployment-like duty cycle, which the hot loop's floor number never shows
  - blocks mode still shields the coldest part by design: the 2 ms post-wake warm is
    unrecorded, so true first-call-after-sleep cost never lands in the histogram. A cold-start
    mode that records or separately reports post-wake samples is a natural extension (idea,
    2026-08-02)

### Outcome

What the cycle set out to fix is fixed, measured on the box that motivated it:

- the 7600x reads straight A's with settle 0.77 s: the warm loop rides through the 4841 MHz
  dwell and exits after the ~0.8 s boost the old fixed warmup measured straight through. The
  vacuous-A defect cannot recur by construction: the exit condition, the printed letter, and
  the settle time are one computation over one window
- the warmup certificate is complete on the 3900X too: the close-out acceptance run reads
  10/10 warmup A, settle 0.01 to 1.48 s, zero "not settled" (the 1.5 s cap absorbs every
  relaxation re-ramp the 0.4 s cap truncated). What remains is the mid-run bistable flip (4 of
  10 runs, env-bench D), which no warmup can prevent and the run grade truthfully attributes;
  its consequences moved to Todos: qualification-as-evidence, seam-clock attribution, blocks as
  the first-class mode
- the day's measurement session (duty cycle selects the state; constant-clock control; block
  count picks the question; the 0.5 ms flip zone) is recorded above and feeds the "Report
  interpretation guide" Todo, ranked #1 at close-out
- grew beyond the planned four rungs by two, both wink-driven same-day: settle/grade alignment
  (-5) and the configurable cap with warm visibility (-6)

### Deferred: start-vs-end differential QC

Repeat the warmup pass at run end and compare floors (never absolute, never subtracted) as a "did
the box shift" check. Deferred at pickup (2026-08-02): the run already carries seam probes across
its whole span and the run grade's drift/step signals answer the same question; revisit if batch
data shows frame shifts need separating from per-iteration shifts. The N-sweep slope/intercept
decomposition likewise stays an idea.

## docs: experiment in the local agent-files

- [[11]] 0.24.1 docs: experiment in the local agent-files

A single-commit cycle inverting hard rule 12: a proposed change to the agent-files (`AGENTS.md`,
`custom.md`, `agent-data/*`) is now edited into the member's local copy rather than staged as a
`custom.md` override or written into the template's shared payload. Raised by wink 2026-08-05
during the convergence discussion with vc-x1, and proposed to the family from here rather than as
a template edit, which is the new rule demonstrating itself.

The old rule had two costs, both structural:

- **`custom.md` was the staging area**, so every proposal landed there as an override. That
  guaranteed it accreted and drifted non-generic, against the goal of a `custom.md` that is
  generic at birth. Three of ours (write-to-full-width, cycle bookend titles, scope-based version
  advancement) were proposals wearing override clothing, and the 20260802 snapshot has since
  adopted all three family-wide
- **the shared payload was the only place to propose**, which makes it the one mutable resource
  two members can collide on. Mailboxes are per-member and do not serialize a payload write. This
  was found while drafting a reply to vc-x1: a two-tier "corrections go straight to the payload"
  scheme solved the ceremony and left the race

What replaces them, and why each record exists:

- **the diff** between a member and the payload is the live proposal set. It needs no maintenance
  and cannot go stale, which a hand-kept status list would
- **the commit history** is the durable record, and it matters because the diff is ephemeral by
  construction: at convergence the diff empties and every trace of what was proposed goes with
  it. History keeps the date, the author, the rationale, and via the `ochid:` trailer the
  bot-repo session that reasoned it out. That is why an agent-file change wants its own commit:
  ridden along inside a feature commit, the rule change survives as one file-by-file bullet
- **the dogfood log** carries what neither can derive, the why and the status, and it is now
  in-flight entries only

`custom.md` narrows to what cannot be family-wide, and the two kinds are worth separating because
only one is negotiable:

- **medium-determined** content (our `cargo fmt` / `clippy` / `cargo test` / `cargo doc`
  validation commands) is not a divergence at all. A prose repo could not adopt them if it wanted
  to, so there is nothing to converge on and no pushback is possible
- **elective divergence** is where wink's pushback belongs, and the test is structural rather
  than dependent on anyone noticing: an entry must say why it cannot be family-wide. With no
  answer it is an experiment, and it belongs in the pinned file where the rule lives

The cost, recorded rather than argued away: a local agent-file no longer distinguishes agreed
from proposed by reading it, which the pinned/`custom.md` split used to do structurally. Our own
2026-08-04 entry logged the same failure shape for doc links, that an unchecked surface reads as
authority until someone follows it. We think the fix is an acquaint-time diff against the payload,
one command reporting how far this member has drifted, which pairs with the `vc-x1 mailbox`
command proposed in the `[messages]` thread. Neither is built, so the exposure stands until they
are.

Not in scope here: reclassifying our existing `custom.md` entries against the new contract. Three
of them are already adopted family-wide, so they should be deleted rather than moved, and that
depends on the 20260803 baseline sync landing first. Carried as a `## Todo`.

The rung also formalizes the term the change needed. **Agent-files** (`AGENTS.md`, `custom.md`,
`agent-data/*`) was coined in this session, so it is defined in a promoted `## Terminology`
section rather than left to context. Promoting it flushed out four places running on the older
vocabulary: `AGENTS.md`'s own intro still said every instruction file except `custom.md` must
match the template, which the new model contradicts; `custom.md` called itself the one
agent-editable instruction file; `prose.md`'s speculation-marker scope said "instruction files";
and a `custom.md` link pointed at the terminology block's old home. "Instruction files" is
retired rather than kept as a synonym, since two names for one set is how a rename half-lands.

**Second bite, from writing this commit's own description.** `prose.md` said a commit body is
"file-by-file, one bullet per file changed", with no bound, so the first draft of this
description enumerated eleven files and restated a diff the reader can already see. wink's test
kills the rule as written: a commit importing a thousand files would demand a thousand bullets.
The rule became one bullet per *distinct change* rather than per file, with files sharing a gist
sharing a bullet named by common path or glob, and by count once enumerating stops informing;
the next step retired the list altogether (see
[The body took two passes too](#the-body-took-two-passes-too)). Found by the new model
working as intended: the tension was between a sentence added here (the diff says what differs,
the history says why) and a pinned rule, and it surfaced on first use.

**First bite, inside this commit.** wink corrected a `cmd; echo "exit=$?"` idiom used while
validating: it prints the status while the invocation itself still exits 0, so a failure is
visible only to whoever reads the text. The new rule was about to land beside its siblings in
`custom.md`'s validation section, where the older pipeline rules already sat, and it could not
answer the elective-divergence test: nothing about masking an exit status is medium-specific. So
it graduated on the spot. The whole group (piping into `tail` / `grep`, `&&` after a piped stage,
`${PIPESTATUS[0]}`, the trailing echo, and the report-and-still-fail form) is now a working
practice in `AGENTS.md`, and `custom.md` keeps only the medium's command lists plus a pointer.
That is the contract working on its first day, and a down payment on the reclassification Todo.

The two subsections below are the same session's family-convergence work. They are not about this
commit's change, but they are the durable home for findings that would otherwise live only in a
mailbox message, and the mailbox protocol is handle-then-delete.

### jj revset primer audit (2026-08-05)

The 20260803 baseline carries a Revsets primer in `agent-data/jj.md` that this repo has never
reviewed: 36 lines present in the payload and absent here, since our `jj.md` is the older subset.
A superset adopted sight-unseen is how a bad rule gets pinned family-wide, and one of these is
provably that. Audited against a live repo (132 visible commits, `main` at `pqxtvkmn`, two parked
bookmarks).

Verified and correct:

- revision forms, and ambiguous prefixes rejected rather than guessed: `jj log -r kv` gives
  "Error: Change ID prefix `kv` is ambiguous"
- neighbours `@-` parent, `@--` grandparent, `@+` child
- "a step past the end of the chain is the empty set, not an error": `@+` and `root()-` both
  return empty and exit cleanly
- `all()` is all visible commits, and the arithmetic closes: `all() ~ ::main` is 6, exactly what
  `main..` returns

**The error is the framing bullet, not the gloss beneath it.** The payload's `jj.md:41-42` reads
"Ranges pair a dot form with a direction, and `::` includes the implicit endpoint while `..`
excludes it", and `:43` then reads "`X::` descendants of X including X; `X..` descendants
excluding X". `X..` is not descendants of X. These are two different operators, not one with an
endpoint toggle:

- `A::B` is the DAG range: commits that are both descendants of A and ancestors of B
- `A..B` is the difference `::B ~ ::A`: ancestors of B that are not ancestors of A

The framing accidentally survives the `::X` / `..X` pair, where the only difference really is the
root commit, then fails completely on `X::` / `X..`. So correcting `:43` alone leaves the model
that generated it in place, and the next person to extend the primer reproduces the bug.

Measured here:

```
main..                    6 changes
descendants(main) ~ main  4 changes
~::main                   6 changes   (identical to main..)
::main                  126 changes   (includes main and the root commit)
..main                  125 changes   (same, root removed; main still included)
```

The two extra changes in `main..` are parked bookmarks, one of them `web-claude-tweaks`, which
`TODO.md` carries a standing entry to rebase. An agent following the gloss and running
`jj abandon 'main..'` would destroy work we have written down a plan to keep. Not a near miss: a
different set.

A second, quieter defect in the same block: `::X` and `..X` are both glossed as "ancestors of X"
and both **include X itself**, directly under a bullet that says "including X" explicitly for
`::`. A reader reasonably infers that `::X` excludes X. The "useful sets" bullet inherits it,
since `jj log -r ::@` is glossed "all ancestors of `@`" and is 130 of our 132 commits, `@`
included.

Proposed replacement for the two range bullets:

```
- Two range operators, not one with a toggle:
  - `A::B` is the DAG range: commits that are both descendants of A and ancestors of B,
    both ends included.
  - `A..B` is the difference `::B ~ ::A`: ancestors of B that are not ancestors of A.
- Omitting an operand defaults it, and that is where the two diverge sharply:
  - `X::` is `X::visible_heads()`: descendants of X, X included.
  - `X..` is `X..visible_heads()`, which is `~::X`: everything that is *not* an ancestor of
    X. This is not "descendants of X". On a repo with parked branches it pulls them in too.
    Descendants of X excluding X is `X:: ~ X`.
  - `::X` is `root()::X`: ancestors of X, with both X and the root commit included.
  - `..X` is `root()..X`: the same set with the root commit removed. X is still included.
```

One finding that is forward-looking rather than wrong today: the primer says `jj-tips.md` is
"hosted once in the template repository (custom.md records the template's path)". The file exists
at `vc-x1-template/jj-tips.md`, but our `custom.md` records no such path by name. Its only
`../vc-x1-template/...` strings are the two mailbox lines, and those are exactly what the proposed
`[messages]` move into `.vc-config.toml` deletes. The pointer dangles the moment that lands, and
should name the config key instead.

### Convergence measurements and positions (2026-08-05)

Measured independently rather than taken on report, `iiac-perf` against `vc-x1-template/work/`:

- diff line counts, ours-only / payload-only: `AGENTS.md` 15 / 23, `agent-data/code.md` 0 / 0
  (already byte-identical), `notes.md` 3 / 3, `prose.md` 8 / 19, `jj.md` 1 / 36
- `diff -r` of vc-x1's `AGENTS.md` plus `agent-data/` against the payload exits 0, so the payload
  and vc-x1 are byte-identical and convergence is one-directional: us adopting the baseline, not
  a three-way negotiation
- our `agent-data/` has `cycle.md` where the payload has `cycle-checklists.md`, and lacks
  `cycle-protocol.md` and `versioning.md`, which the payload relocated out of `notes/`
- our `cycle.md` has validation at step 4 and is self-consistent; the payload's own
  `work/custom.md:11` still cites step 4 against a checklist that renumbered validation to 5, so
  the payload is the inconsistent side

Sizing the sync from here: 6 mentions of `agent-data/cycle.md` in 1 file, 18 of
`notes/cycle-protocol.md` across 7, 4 of `notes/versioning.md` across 3. 28 inbound references to
re-point, which is why it is a cycle and not a side edit.

Positions taken on the four questions the family left open, recorded so they survive the mailbox:

- **how a pinned change is proposed**: split by kind. A correction (factual error, typo, stale
  cross-reference) goes straight into the payload, since a wrong gloss has no second opinion to
  gather; a rule change goes through a snapshot directory, reviewed as a set. This commit's own
  change supersedes the question for experiments, which now never touch the payload at all
- **the `jj.md` correction lands before our baseline sync**, not with it, and it must cover the
  framing bullet and not only the gloss, since syncing propagates the model
- **the CLI reads `[messages]`**: if it does not, unknown-key tolerance still has to be decided,
  so reading them answers the question instead of dodging it. One schema note: `other-repo`
  already sits under `[workspace]`, and if `community` is the same species it belongs in the same
  table
- **what a version bump promises**: `versioning.md` defines the scheme, each project's
  `custom.md` states the promise. The promise differs by artifact kind (binary crate, library
  crate, CLI with users, prose repo) and no shared file can assert one for everybody. Related and
  worth a rule of its own: a pinned file names no project, no project's history, and no project's
  versions

## docs: steps are titles, versions are stamps

- [[12]] 0.24.2 docs: steps are titles, versions are stamps

A version written into prose is a second identifier for something that already has one, and it is
the fragile one. This repo proved that on 2026-08-01: renumbering two published versions left
every transcript and pasted report carrying the old banners, and the only thing that can read them
now is a decoder entry in `custom.md`. The titles of those same commits needed nothing. So the
rule inverts. The title is the identifier, a rung's place in the ladder list is its position, and
the version-of-record goes back to being a build stamp that lives in `Cargo.toml` and nowhere
else.

What changed, and why each piece sits where it does:

- **a rung is a bare title.** No version, and no step number either: the rung's place in the
  markdown list already records its place in the ladder, so a number beside it would restate the
  position and then have to be maintained. Inserting or reordering a step edits the list and
  nothing else, which is the whole mechanism
- **the version-of-record still bumps every step**, including a docs-only or agent-file-only one.
  The cadence is unchanged; only its visibility changed. Its suffix is the single number left in
  the system, it names nothing, and nothing dereferences it
- **one prose surface records a version, the chores as-built rung** (wink), and it earns the
  exception by not naming anything: it records what a landed commit carried, beside that commit's
  SHA, so the pair decodes an old `-V` banner or a pasted report. That is the job `custom.md`'s
  renumber entry does by hand today. It obeys the SHA's timing exactly, so an unlanded rung carries
  neither, which is also what makes a rebase free: nothing in the ladder can be falsified by one
  because nothing is written yet
- **titles must be unambiguous, not globally unique**, which is a weaker constraint than the first
  draft of this rule carried. A title is resolved in exactly two places, so those are its scopes:
  within its cycle, so a ladder rung names one step, and within its chores file, since a `##`
  header is also an anchor and GitHub silently resolves a duplicate slug to the first occurrence.
  Across history a title may repeat, and `git log --grep` returning two hits is not a defect
- **a commit body is a problem statement and a solution statement**, both broad, and nothing else.
  The dual-repo model is what makes that safe rather than lossy: the `ochid:` trailer reaches the
  session that reasoned the change out, the as-built rung reaches this file, and the `## Todo`
  entry holds the plan. See [The body took two passes too](#the-body-took-two-passes-too)
- **a topic bookmark is a draft until it lands.** Keeping its series self-consistent may rewrite
  rungs that are already pushed, which is legal because pushing to a bookmark is not publishing.
  The mechanism is content amendment plus a force-push, never a re-describe, so the
  never-re-describe rule and the `ochid:` trailers are both untouched (change ids survive a
  rewrite, as the 2026-07-31 trapezoid experiment measured). Three exceptions are named in the
  protocol so "when practical" is not re-litigated each time

### How the numbering got to zero

The rule arrived in three passes in one session, and the path is worth keeping because each pass
removed something the previous one had assumed was needed.

- **First**, versions came out of prose and ladder rungs took a `step-N` prefix in their place,
  numbered positionally like a `## Todo` rank. That fixed the fragility but kept a number.
- **Second**, the question was whether `step-N` should *be* the manifest's suffix, one numbering
  serving both. Rejected: it rewrites the suffix scheme (the final-`0`-marks-a-Preparation rule,
  the nesting notation, the disambiguation cases) to buy an agreement nothing reads.
- **Third** (wink), the number went away entirely. A rung sits in an ordered markdown list, so
  the number was restating the list position and adding a maintenance obligation, and the
  renumber-on-insert rule the second pass needed was work invented by the first pass. What
  survives is the part that was doing the job: the title.

The cost, accepted rather than argued away: naming a step in conversation is now "the report
renderer step" instead of "step-3", which is longer to say and cannot be ambiguous inside one
cycle. The residue is that `versioning.md`'s suffix is the last number standing, and it is now
explicit there that it names nothing.

### The version was also a landmark, which is how the Done entries got reshaped

Found by wink reading this very step: scanning `TODO.md > ## Done` for the new entry meant looking
for `0.24.2`, not finding it, and concluding the entry was missing. It was there. The version had
been doing a second job nobody had credited it with, a short high-contrast token per entry that
the eye could rest on, and removing it exposed what it had been masking.

The fix is not to restore it, because it was a poor landmark: its absence reported that a predicted
number was missing, not that an entry was. A title answers the question a skim is actually asking.
So the entries changed shape instead.

- **A `## Done` entry is now a bold title line plus sub-bullets** (`agent-data/notes.md`
  [Done entry form](../../agent-data/notes.md#done-entry-form)), matching the `## In Progress`
  block, whose title line was already bold.
- **The old form violated `prose.md` and had done so for a while.** A title with five lines of
  summary trailing off it is the wall-of-prose shape Prose form warns about, and two rules still
  called these entries one-liners (`notes.md`'s migration step and cycle-protocol's Close-out)
  while `prose.md` separately blessed the paragraph. The version was hiding a drift, not
  preventing one; all three now agree.
- **The three live entries were converted**, versions dropped from the two older ones too, at
  wink's call, so the section can be judged in its new form rather than in a mixture. `done.md`'s
  existing entries stay under grandfathering.

Bounded on purpose: Done and `done.md` entries are the only surface where a long body hangs off a
title in a flat list. Chores as-built ladders got *more* skimmable, since a rung is one short line
that lost a token, and `## Todo` keeps its ranks.

### The body took two passes too

The first pass kept the body an edit list. It said "one bullet per *distinct change*, not per
file", called itself "source of truth for the mechanical edit list", and carried sub-rules for
when files share a gist, when to name a glob, and when to switch to a count. That was already an
improvement on one-bullet-per-file, and it survived exactly one use: writing this commit's own
description produced thirteen bullets that restated `git show --stat` in worse English.

The second pass (wink) dropped the list. A body is a **problem statement** then a **solution
statement**, in broad terms, and the diff is left to be the mechanical record it already is.

- **The first pass contradicted itself**, which is visible in the text it replaced: the rule
  warned that "restating what `git show --stat` already prints is a second copy that can drift"
  and then required a per-change list, which is the same duplication at a coarser grain.
- **The glob and count sub-rules existed only to manage the list.** With no list they have no
  job, so the rule got shorter rather than more qualified.
- **The problem statement has to answer for what it takes away.** Writing this one surfaced the
  test now in the rule: the problem said the version supplied the ordering, so the solution owed
  the reader what supplies it now. A body that raises a question and leaves it is not concise, it
  is incomplete.
- **A knock-on correction**, found by the same review: `notes.md` justified chores carrying no
  edit list on the ground that the *commit body* was the mechanical record. That was never quite
  right and is now plainly wrong, and the file already had the better formulation a few lines
  later ("Git owns the mechanical record"). It is now a three-way split: the diff is mechanical,
  the body is problem and solution, chores is the design thinking.
- **Four contradictions were cleared in the same pass**, all of them ours and most of them
  introduced earlier the same day: the body rule stated in full in two files, `prose.md` still
  calling bodies "file-by-file" in one place while forbidding it in another, two examples teaching
  the abandoned shape, and the title length limit reading `<=50` in the authority file against
  `<=72` in three others. The 50 stands, and body wrap stays 72.

Then the pair turned out not to be new (wink). `prose.md` already had a `### Problem + plan shape`
for `## In Progress` blocks, chores intros and `## Todo` entries, glossing its problem statement as
"(the why)", which is what a problem statement is. So the commit body is that same shape for
finished work: **timing picks the second half**, a plan for work ahead, a solution for work done.
One `### Problem-first shape` now covers four surfaces, and the commit-body bullet keeps only
what is commit-specific.

The consolidation immediately caught a term collision we had been carrying. The shape section
glosses the problem statement as "the why", while the body rule said "the *why* and *how* stay
out", so the same word both belonged and did not. What actually stays out is the **deliberation**:
alternatives weighed, evidence, dates, costs accepted. The problem is a why and belongs in the
body. Two files said the wrong thing and now say that one.

### The dynamic-warmup backfill, two cycles late

This step also clears backfill debt, which is in scope because a close-out is where the checklist
puts it and because the debt is what the new rule is about. `## feat: dynamic warmup`'s eight rungs
sat on literal `[[N]]` placeholders even though the trapezoid rewrite had landed all eight on
`main`, where their SHAs and versions were final. They are now filled, refs `[3]` through `[10]`.

Why it was missed is the interesting part, and it is our own reported finding coming back: on
2026-07-31 we told the family that the per-commit checklist has no step for backfilling the
previous push's chores refs, and the 20260802 baseline answered it by adding per-commit step 4,
"Close the records". We have not synced that baseline, so the gap we reported stayed open here and
the next close-out skipped the same step. Backfill lives only in our close-out checklist, at step
6, which is easy to reach the end of a docs cycle without reading.

`## docs: experiment in the local agent-files` is *not* backfilled, correctly: it sits on the
unlanded `agent-files-model`, so its rung keeps the placeholder, and under the timing rule above
its `0.24.1` came out of the rung until the bookmark lands.

Reference slots `[3]`..`[10]` were allocated next-free rather than by re-packing the file into
document order, which would have renumbered the two design citations for cosmetics.

### Grandfathering, and what deliberately keeps its versions

Versioned prose already in the repo is not swept; it converts when its surrounding text is
touched, the same treatment the legacy `Commits:` lines got. Two cases are deliberate rather than
pending:

- **`custom.md`'s 2026-08-01 renumber entry keeps its version mapping.** It *is* the decoder for
  the residue, so stripping the versions out of it would defeat its purpose.
- **the previous commit's body keeps its version-bump bullet.** It is published on this bookmark
  and a body is not editable text; the rule applies from here forward rather than backwards.

### Drafted before the rebase, on purpose

The step lands on `agent-files-model` ahead of rebasing `measure-reproducibility` onto it, so the
next cycle runs its ladder under these rules and the family reply carries evidence instead of
intent. Refinements the dogfood turns up are collected in `TODO.md` and land as a later batch
step, rather than each one triggering another rebase of the in-flight rungs.

## docs: one owner per rule, one home per record

- [[13]] 0.24.3 docs: one owner per rule, one home per record

A single-commit cycle, and the first run under the rules it writes. The six provisional items below
are recorded here rather than in `TODO.md > ## In Progress`, because a single-commit cycle has no
separate opening to write them at; that gap is the cycle's own first finding.

### Problem

Two rules the project had agreed to were written down nowhere, and one it had written down was
wrong. Cycles had run on topic bookmarks since 2026-08-01 on undocumented habit, with neither
creating nor landing one described in any agent-file. `custom.md` had drifted into holding rules the
family should own plus facts that only make sense to a family member, so it could not be handed to a
project that had never heard of us. And `notes.md` mandated building the chores record up per
commit, which meant every ladder rung written twice and every backfill applied twice.

### Solution

The pinned files absorb what belongs to them and the project layer keeps only what they structurally
cannot. Hard rule 13 states that cycles run on a bookmark; `cycle.md` and `jj.md` carry the opening,
the land step, and the commands. `custom.md` shrinks to a stub with nothing to substitute, and
everything of ours moves to `custom-family.md` behind a one-line pointer. A cycle's record gets one
home at a time, `TODO.md` while it runs and chores after a mechanical move. Four of vc-x1's six
2026-08-07 items are adopted along the way.

### Acceptance check

Nothing below `AGENTS.md` is auto-loaded any more: `CLAUDE.md` went from importing each layer to one
line, and the chain from there is prose. So the check is whether an agent starting from what the
harness loads still reaches the project's validation commands, and whether every pointer in the
pinned files resolves to a file that exists.

**Result: the static half passes, the behavioural half is deferred one session and that is
recorded rather than papered over.**

- the chain resolves link by link: `CLAUDE.md` holds `@AGENTS.md` alone; hard rule 0 names
  `custom.md`; `custom.md`'s single conventions entry reads `- Read ./custom-family.md.`;
  `custom-family.md` carries `cargo fmt` / `clippy` / `test` / `install` and the fast
  `cargo test --bins`
- every relative file link across the eleven agent-files resolves to an existing file. Three
  apparent misses are prose examples inside `notes.md` (`[text](url)`, `features.md#feature-x`,
  `../chores-07.md`), all pre-existing and none a real link
- what is *not* tested is an agent actually following the chain, since this session's context was
  built under the old two-import `CLAUDE.md` and the session cannot re-bootstrap itself. The next
  session is the test and it costs nothing: if `custom.md` is absent from its opening context and it
  still arrives at the cargo commands, the check passes in full. If it does not, this cycle broke
  the thing it was most at risk of breaking

### Ladder

- [[13]] docs: one owner per rule, one home per record

### Deliberation

The three changes are separable in principle and were kept together on purpose: rule 12's reframing
is what permits the `custom.md` split, and hard rule 13's land step is what finally gave the chores
backfill an owner. No single honest title covers all of it, which by our own hard rule 9 is evidence
of more than one step; the title above is a compromise and is recorded as such.

**What we got wrong first, twice.** The medium's commands live in `custom-family.md` now, so nine
pinned references naming `custom.md` looked stale and were repointed at a newly defined "project
layer". wink rejected it: an agent reads `custom.md`, meets a one-line directive, and follows it, so
a pinned file asking for something "in custom.md" is already answered. All nine were reverted to
byte-identical with the payload. The same instinct produced a `CLAUDE.md` that `@`-imported every
layer, which wink also rejected, on the ground that it makes `CLAUDE.md` a second statement of what
to read and therefore a second thing to keep true. Both mistakes were the same one: solving in the
pinned set what the project layer had already solved.

**The bookmark categories collide and it is unresolved.** `agent-files-model` has now hosted three
cycles without landing, which by the `jj.md` text written today makes it a long-lived bookmark
(merge-only, never rewritten) at the same time as a topic bookmark (a draft, freely rewritten). The
two are defined as mutually exclusive. Nothing forces a decision this commit, but the
`measure-reproducibility` rebase does, since it relies on the draft reading.

**Three rules this cycle breaks on its way in.** It has no opening commit, so hard rule 13 is
unobserved by the commit that introduces it. The close-out could not "run the acceptance check the
opening stated" because there was no opening. And the six provisional items have no defined home in
a single-commit cycle, since such a cycle skips `## In Progress` by the 2026-07-31 rule; they are in
this section by improvisation. The last one wants fixing in the text.

**vc-x1's contribution, and where we went past it.** Their tier-2 item #1 proposed ladder plus
narrative in one home; we made it six required items with `_None._` for an empty deliberation, and
folded their acceptance-check proposal in as the sixth. Their `chores-15` trial supplied the four
transforms of the move, the finding that two of them fail silently, and the measurement that anchors
survive the depth change because GitHub slugs derive from heading text rather than level. That last
one refuted an objection we had raised against using headings at all.

**Deferred.** `AGENTS.md` still says "the family" and "member" where the generic mechanism is "the
template payload and my copy of it", which is the same test that moved the family layer out of
`custom.md`, applied one level up. Left for its own cycle rather than doubling this one's review
surface.

**`.vc-config.toml` migrated to the `[repos]` schema, because the first push attempt refused to
run.** Both sides were still on the legacy `[workspace] path / other-repo` form. The finding worth
keeping is the asymmetry: read-only commands accept the legacy schema (`vc-x1 chid @` exits 0),
`vc-x1 config --validate` reports it, and `vc-x1 push` **hard-errors before any stage**. So a repo
can run for weeks looking healthy and fail only at the moment it tries to publish. We had tested
the read-only case and concluded "normal commands tolerate it", which was true of what we sampled
and wrong as a generalisation; the claim had already gone to vc-x1 and is corrected in the same
mailbox entry. The migration rides along here rather than as its own cycle because nothing could
be pushed until it landed.

## docs: the bot pushes again

- [[14]] 0.24.4 docs: the bot pushes again

A single-commit cycle retiring a `permanently local` dogfood entry, per
[Retiring Done entries](../../agent-data/notes.md#retiring-done-entries): the narrative lands here
and the entry leaves `custom-family.md`, so that log carries in-flight entries only.

### Problem

Since 2026-08-06 every push had been handed to wink to run from his terminal, because `vc-x1 push`
from the bot's sandboxed shell failed twice on a bot-repo push (`send-pack: unexpected disconnect
while reading sideband packet`). The workaround cost a round trip on every cycle, and the entry
recording it named a wrong cause three separate times.

### Solution

The cause is known and already fixed, so the entry retires and the bot pushes directly again.
**Both repos were cloned over ssh and wink repointed them at https**, which a sandboxed session can
use and ssh it cannot. The account of how we got it wrong four times moves here, where a wrong
cause is a historical note rather than a live instruction.

### Acceptance check

A push from the bot's sandboxed shell completes. **This cycle's own push is the check.**

**Result: passed**, 2026-08-07. It was already passing before this cycle existed, since
`docs: one owner per rule, one home per record` had pushed cleanly an hour earlier, so this
confirms rather than discovers.

### Ladder

- [[14]] docs: the bot pushes again

### Deliberation

**The cause, on vc-x1's evidence and wink's testimony, not on any measurement of ours.** Both
iiac-perf repos were cloned over ssh, as vc-x1's were. A sandbox denies ssh twice over: `~/.ssh`
reads are blocked except the signing key and `known_hosts`, and we think a host allowlist cannot
admit port 22 at all. The network leg is a spawned `git` child
(`git_settings.to_subprocess_options()` handed to `jj_lib::git::push_refs`) which inherits the
sandbox, and that is why the identical command diverged between wink's terminal and ours. wink
repointed both remotes at https, an idea sourced from a conversation with claude-web and settled
with vc-x1. vc-x1's `test: Claude Code can complete a cycle` (0.78.4, 2026-08-07) is the controlled
experiment, and it killed three competing hypotheses by test rather than by argument.

**Four wrong causes, and the same shape every time.** Not one of them was corrected by a
measurement of ours:

- *the sandbox chokes an ssh stream at volume*, from a size correlation, with no ssh reading taken
- *cause unknown, and it is not ssh*, from running `jj git remote list` and writing "we have no
  reason to think they were ever otherwise". That is a claim about the past drawn from a
  measurement of the present, and it was wrong, because the remotes had been switched in between.
  vc-x1 had explicitly left this question open for us and we answered it backwards
- *the in-process jj-lib transport fixed it*, from a log line reading `(in-process)`, which
  describes the squash and not the network leg. jj-lib spawns `git`; there was never an in-process
  transport to credit
- the size framing underneath all three, inherited and unexamined until wink pointed out the file
  is now 3.2 MB, still growing, and pushing fine

**Why size looked causal: it was confounded with the repo.** Every failure arrived at
`squash-push-bot`, and the bot repo is also the only one carrying multi-MB transcripts, so "large
pushes fail" and "bot-repo pushes fail" were indistinguishable in our data. This project has the
identical confound already on record in its measurements, where block count and box history rose
together and history dominated. We know the trap in the harness and walked into it in the process
record.

**One retraction rather than a reconciliation.** The old entry claimed "connect and auth succeed
even on the failing runs", which contradicts vc-x1's finding that no auth key is readable. We are
not going to reconcile them: that observation came from the same session that invented the size
correlation and it has no basis we can verify. Withdrawn.

**The discipline this leaves: record the intervention and the outcome, and let whoever read the
source own the mechanism.** vc-x1 read jj-lib and the sandbox rules; we read a correlation. The
party that looked at the source was right the first time and the party reasoning from symptoms was
wrong four times. The cheapest correct move available throughout was to ask wink, who had made the
change, and that never happened.

**Bookmark note, still unresolved.** This is the fourth cycle on `agent-files-model` without a
landing, which by `jj.md` makes it a long-lived bookmark and a topic bookmark at once. Recorded
again rather than decided again; the `measure-reproducibility` rebase is what forces it.

## docs: sync agent-files from vc-x1's draft

- [[15]] 0.24.5 docs: sync agent-files from vc-x1's draft

A single-commit cycle handling the sync half of vc-x1's 2026-08-08 convergence message; the
mailbox sweep below records what left the mailbox on the same pass.

### Problem

Our pinned set carried two regressions the 2026-08-08 review named (the protocol's worked
example taught a pre-commit with a hand-written `ochid:` trailer; the per-commit version bump
had dropped out of every checklist), lagged the family's merged text, and described conventions
older than the `vc-x1-dev` binary this repo now runs.

### Solution

Byte-copy vc-x1's agent-file set (AGENTS.md, custom.md, agent-data/), taking the
`cycle.md` -> `cycle-checklists.md` rename and the relocation of `cycle-protocol.md` and
`versioning.md` from `notes/` into the pinned `agent-data/`, then re-point every project-local
reference and mirror the new `## In Progress` template into `TODO.md`.

### Acceptance check

`diff -r` between our pinned set and the source revision is empty, and no live project-local
link still targets a moved or renamed file.

**Result: passed**, 2026-08-12. `diff -r` over the set against the source tip is empty, and a
repo-wide grep finds old paths only in records' prose, where they stay.

### Ladder

- [[15]] docs: sync agent-files from vc-x1's draft

### Deliberation

**The draft tip over `main`, wink's call.** The 2026-08-08 message names vc-x1's `main` as the
sync source, and the first copy taken was from `main` (73319b8c). wink redirected the sync to
the tip of vc-x1's open cycle (`docs-freshen-vc-config-and-config-subcmd`, 3ae26bad, 8 commits
ahead of their `main`) because it documents the `vc-x1-dev` binary this repo now runs. Adopted
knowing it is a draft that can be rewritten before it lands; the next sync reconciles.

**What the tip adds over their `main`**: a hard-rules exceptions clause (the rules bind the
bot, none is absolute, wink can bend one explicitly, every exception is recorded); rule 13
tightened (slug-named bookmark carrying every step, deleted locally and remotely once landed;
the bot repo needs no bookmark); a Cycle terminology entry (single-step folds all three stages
into one commit, multi-step is minimum two); convention work runs as its own cycle; the linked
ladder (`- [[N]] [<title>][M]` rungs, `Ladder details` subsections, `[M]: #<slug>` refs).

**Deliberately dropped**: our `notes/cycle-protocol.md` copy's old-binary recovery material
(the 0.22.0 close-out recovery example among it), which vc-x1's 20260803 drop-list had already
named. jj history keeps it.

**A commit-body form was settled during this cycle and dogfooded in this commit's own body**
(wink, 2026-08-12). The full form, written as a proposal for the family, is
[Commit-body form proposal (2026-08-12)](#commit-body-form-proposal-2026-08-12) below.

### Commit-body form proposal (2026-08-12)

Proposed for the family (wink + iiac-perf f5, 2026-08-12). Both repos have been writing
problem/solution bodies with no version and no file list; vc-x1's last commit tried a problem
bullet with a solution sub-bullet, and this cycle generalized that into one recursive shape.

**The form.** A body is an intro paragraph, an optional list of sub-problems, and solutions:

- The **intro paragraph states the general problem** and defines any word the title assumes.
  It is mandatory: prose form wants it, and a body opening with a bullet fails vc-x1's clap
  twice over (the 2026-08-01 leading-hyphen rejection).
- **`*` bullets are the problem's facets**: sub-problems that decompose the intro's general
  problem, not a grab-bag of unrelated fixes. A body needing unrelated problem bullets is
  usually asking to be more than one commit.
- **`-` bullets are solutions**, and a `-` solves the nearest enclosing problem: nested under
  a `*` facet it solves that facet; at top level it solves the intro's general problem, which
  is how one solution says it retires every facet at once (position expresses scope).
- **The trivial commit is the same shape with zero facets**: a prose problem paragraph and one
  or more top-level `-` solutions. Not a second form, the general form with an empty middle.

**The markers are typed on purpose**: `*` always means problem, `-` always means solution.
Indentation alone cannot distinguish them in the trivial case, where a lone top-level `-` is
readable as a solution only because `-` always is one. The typing also makes history greppable
(`^\* ` finds every facet, `^- ` and `^  - ` every solution). Bodies are read as plain text
(`jj log`, terminals), where the markers survive; if a renderer ever flattens them, the
indentation still carries the structure. The mixed markers are deliberate, so a linter's
consistent-marker rule should not "fix" them.

**What stays unchanged**: no version in title or body, no file list (the diff is the
mechanical record), no deliberation (chores, todo, and the session hold that), titles per the
existing rules.

**Evidence**: dogfooded in this cycle's commit (`docs: sync agent-files from vc-x1's draft`,
three facets, three nested solutions, no top-level solution) after vc-x1's single-pair trial
the commit before. One instance each; proposed on shape, not on sample size.

**If adopted, the pinned edits are**: cycle-protocol.md's Commit description `Body` section
(today: "a problem statement and a solution statement, both broad"), cycle-checklists.md
per-commit step 7, and prose.md's Conventional-commit shape. It answers the body-shape half of
the question vc-x1 holds at backlog #50. Our local pinned copies are not yet edited: we synced
byte-identical to vc-x1's draft tip today and would rather not re-diverge the same day, so the
proposal rides here and pins after vc-x1's read.

### Mailbox sweep (2026-08-12)

The 2026-08-08 message listed the entries it supersedes; they are deleted on this pass. The
message itself stays open (our formal review reply and the payload update remain). Copied here
before deletion, per Messaging's handle-then-delete:

- **Residue decoder** (from the 2026-08-06 entry; also vc-x1 bugs.md #8): around the 0.71.0
  stale-push-state incident (2026-08-06), a work commit's `ochid:` can name a bot commit one
  off. When a trailer from that era dereferences to the wrong session, look one bot commit
  earlier.
- **The vc-x1-dev switch** (decided at the 2026-08-06 triage, enacted 2026-08-12): this repo
  runs `vc-x1-dev`, the dogfood binary, well past the 0.77.0 deletion of the push-state
  machinery the incident blamed. The interim remedy (`--restart` or deleting
  `.vc-x1/push-state.toml` before any non-resume push) retires with the switch.
- Everything else in the deleted entries is recorded on vc-x1's side (their chores-16 section
  "docs: adopt the merged agent-file set" and bugs.md #8) and remains reachable through the
  standing 2026-08-08 message.

**Outbound the same day**: our `.vc-config.md` capability feedback, wink's two naming
decisions (the family moves to "agent-repo"; the default agent-repo directory is planned to
become `.agent-session`, hidden), and the
[commit-body form proposal](#commit-body-form-proposal-2026-08-12) went to vc-x1's mailbox,
2026-08-12.

## chore: sync cycle records and mailbox sweep

- [[16]] 0.24.6 chore: sync cycle records and mailbox sweep

A single-commit cycle whose entire product is another cycle's record, which is why it went a day
without one of its own. Written after the fact, on 2026-08-12, from the commit and the session it
was pushed from.

### Problem

The sync cycle landed with its chores section unwritten, and the mailbox sweep it triggered had
no durable record at all, which the messaging rule forbids: a message can never be a record, so
anything in one worth keeping is copied out before the entry is deleted.

### Solution

Write the sync cycle's section, carve the commit-body form proposal out as its own anchored
subsection so a message can point at it, and record the sweep, naming what was deleted and what
was copied out first.

### Acceptance check

Nothing deleted from the mailbox that morning survives only in the mailbox.

**Result: passed**, 2026-08-12. The deleted entries' durable content is the residue decoder and
the `vc-x1-dev` switch, both in the sync section's
[Mailbox sweep](#mailbox-sweep-2026-08-12). Everything else in them was already recorded on
vc-x1's side and remains reachable through the standing 2026-08-08 message, which was not
deleted.

### Ladder

- [[16]] chore: sync cycle records and mailbox sweep

### Deliberation

**Why it had no section of its own until now.** Its work was writing another cycle's record, so
its output lives inside the section above rather than beside it, and the close-out that would have
noticed was the same close-out being repaired. That is the shape of the whole debt this cycle
clears: a records commit is the one kind of commit whose own records are easiest to forget.

## docs: adopt the commit-body form

- [[17]] 0.24.7 docs: adopt the commit-body form

A single-commit cycle taking vc-x1's pin of the commit-body form this repo proposed the same
day. Their reply is the pin itself, which is why adoption is a copy rather than a negotiation.

### Problem

vc-x1 pinned our proposed commit-body form into their `agent-data/prose.md`
(`docs: pin the commit-body form`, 076193f9) and asked us to carry it or name what we differ
on. Our pinned set then differed from theirs in exactly the three files that pin touches, so
the family's newest rule was in force on one side of it only.

### Solution

Copy `prose.md`, `cycle-protocol.md`, and `cycle-checklists.md` from vc-x1, taking the pin
verbatim, including its three deliberate departures from what we proposed.

### Acceptance check

`diff -r agent-data ../vc-x1/agent-data` is empty.

**Result: passed**, 2026-08-12. The diff is silent over the whole pinned set, and it was the
same three files before the copy, which is what made a straight copy safe.

### Ladder

- [[17]] docs: adopt the commit-body form

### Deliberation

**Verbatim, with no differ (wink).** All three of their departures were read before the copy
and all three taken. `prose.md` is the single home, with `cycle-protocol.md` and
`cycle-checklists.md` linking the form rather than restating it, which is one home and two
pointers where our own proposal had implied three restatements. The intro-mandatory rationale
is generalized to "a body a `--body` flag can mistake for an option", dropping our clap history
under their `Pinned files name no project` rule. And whether a rung's `## In Progress` edits are
a facet of the commit's problem is left unpinned, taken as cycle mechanics on one instance.

**The source is their tip, which equals the pin.** vc-x1 offered `076193f9` or their tip; the
two commits after the pin, `f7e90803` and `6ea66d68`, do not touch these three files, measured
before the copy, so the bytes are the same either way.

**Single-step, and the adopted form is what decided the scope.** The record debt left by 0.24.5
and 0.24.6 (two missing `## Done` entries, and no chores section for 0.24.6) was considered for
this commit and left out. The form we are adopting says a body reaching for unrelated problem
bullets is usually asking to be more than one commit, and adopting a family rule and repairing
our own records are unrelated problems. The debt rides the review-and-reply cycle, whose
close-out is in this file anyway.

**What this deliberately does not close**: the formal review of vc-x1's agent-file set, owed
since their 2026-08-08 message, and the two questions their 2026-08-12 message asks, whether a
notes entry stays a numbered list item, becomes a heading with a real anchor, or leaves the repo
for a tracker, and whether we differ on anything in the pin. Both are the next cycle's, and the
mailbox entry stays open until then.

### Gotcha: `-V` lags the version-of-record after notes-only commits

**Problem.** `iiac-perf -V` printed 0.24.4 while the version-of-record was 0.24.6, and it read as
evidence that the ladder's versions were wrong (wink). The cause is a rule rather than a slip: the
per-commit checklist makes validation skip-able for notes-only commits and `cargo install --path .`
sits inside it, so two commits bumped the version without ever building it.

**Solution.** Named here, fixed in the next cycle. wink's position is that every commit bumps the
version-of-record precisely so that a build exists carrying it, which makes an unbuilt bump a
version nobody can run and the banner a claim nobody checked. So the skip-able clause goes, at all
three sites that carry it: `cycle-checklists.md` step 5, `cycle-protocol.md`'s per-commit step 5,
and `custom-family.md`'s `when:` line, which today's sync also left pointing at step 4. Two of the
three are pinned, so it is a family rule change and runs as its own cycle rather than riding this
one, and it goes to vc-x1 with the reply we already owe. This cycle's own close-out install is what
made the size of the gap visible, 0.24.4 to 0.24.7 in one step.

## docs: validate every commit

- [[18]] 0.24.8 docs: validate every commit

A single-commit cycle closing the hole the previous cycle's gotcha named, one beat after naming
it.

### Problem

The per-commit checklist stamps the version-of-record at step 4 and then let step 5 be skipped
for notes-only commits, so a commit could carry a version that no build ever had. It is not
hypothetical: 0.24.5 and 0.24.6 both stamped and neither built, and `iiac-perf -V` answered
0.24.4 until the next close-out jumped it three.

### Solution

Drop the skip at all three sites that carried it, condition the step on whether the medium has a
runnable artifact rather than on what kind of change the commit made, and give each of the three
sites one job so that no part of the rule is written twice.

### Acceptance check

No agent-file says validation is skip-able, and `custom-family.md`'s `when:` line names the step
number the checklist actually uses.

**Result: passed**, 2026-08-12. A repo-wide grep for the clause returns four hits, all in this
file: the previous section's gotcha, which named the hole and pointed here, and this section's
own check. No agent-file is among them.

### Ladder

- [[18]] docs: validate every commit

### Deliberation

**wink's rule, and the reason is his.** Every commit bumps the version-of-record precisely so a
build exists carrying it. An unbuilt bump is therefore a version nobody can run, and `-V` is the
artifact's own report of what it is, so the skip made the banner a claim nobody had checked.

**The condition is the medium, not the commit (wink).** The first draft of this rule kept an
escape for a project whose build is too costly to run every time, which asks every project to
judge its own cost and would have been claimed by anyone who found validation tedious. wink's
condition replaces it: run the artifact if the medium has one to run. That is a fact about the
repo rather than a judgment about the commit, so it cannot be argued into, and it covers the
case the cost clause was really groping at, a prose repo with nothing to execute.

**Nothing gated this edit, and recording that is the point.** Two of the three sites are pinned
files shared with vc-x1. Earlier in the day this session framed the same edit as a proposal
waiting on their reply, which wink corrected: our repo is ours to change, the diff against the
payload is what makes a change a proposal, and what we never do is write in someone else's repo.
The confusion is worth a sentence in the agent-files and gets its own cycle.

**One home, three jobs (wink).** The first pass wrote the same sentence into both the checklist
and the protocol, which is exactly the duplication the commit-body form avoided one commit
earlier by making `prose.md` its single home. The division that holds: the checklist carries the
instruction, because a checklist that only points is not a checklist; the protocol carries the
reason and the medium condition; `custom.md` carries the commands. A medium with nothing runnable
simply has no commands there, which is how the condition reaches a reader without being restated.

**Two things tidied on the way past.** `custom-family.md`'s `when:` line still pointed at step 4
after this morning's sync moved validation to step 5, and now names the step without restating
its rule, restatement being what let the number go stale unnoticed. The cycle-at-a-glance
sentence no longer calls mandatory validation one of the close-out's distinguishing duties, which
distinguishes nothing once every commit validates.

## chore: complete the landed records

- [[19]] 0.24.9 chore: complete the landed records

A single-commit cycle run immediately after `main` moved to the `agent-files-model` tip, because
landing is the beat that makes the records below both possible and due.

### Problem

Eight commits became permanent at once and their records were incomplete in three ways: seven
as-built ladders still held literal `[[N]]` placeholders, waiting on exactly the permanence that
had just arrived; the records cycle at 0.24.6 had no chores section at all; and `## Done` was
missing the entries for 0.24.5 and 0.24.6, so the list of what shipped did not list two things
that had.

### Solution

Backfill the as-built ladders with the SHAs and versions the landing made stable, write the
missing section, and write the two missing `## Done` entries.

### Acceptance check

No `[[N]]` placeholder remains on a landed commit's rung, every landed commit has a chores
section and a `## Done` entry, and every rung citation resolves to a definition in
`# References`.

**Result: passed**, 2026-08-12. Citations `[1]` through `[18]` all resolve, checked by comparing
the cited numbers against the defined ones. The `[[N]]` tokens still in the file are this cycle's
own two rungs, unlanded and correct to leave, and prose quoting the placeholder form.

### Ladder

- [[19]] chore: complete the landed records

### Deliberation

**Why now rather than at leisure.** The backfill was owed the moment `main` moved, and doing it
immediately gave this session's reasoning a home. Landing produces no work-repo commit, so the
conversation that decided to land, and that produced the duplication finding and the
`vc-x1 land` design, had nothing to hang an `ochid:` trailer from and would otherwise have been
swept into the next unrelated commit's session (wink's observation). The backfill is the
work-repo artifact of the landing beat, so filing that session under it is not a convenience, it
is the correct home.

**The version rides with the SHA, and only on the as-built rung.** Hard rule 9 keeps versions out
of ladder prose; the as-built rung is the one named exception, since it records what a landed
commit carried and, beside the SHA, decodes an old `-V` banner. So the six-item `### Ladder`
rungs took their citations and no version.

**A duplication noticed while doing it, not fixed here.** In a single-step cycle the as-built
ladder and the six-item `### Ladder` are the same one rung written twice, which is the same
disease as the checklist and the protocol holding the same nine steps. Recorded for the
duplication cycle rather than solved mid-backfill.

# References

[1]: /notes/chores/chores-05.md#the-7600x-stopped-passing-and-the-grade-is-why
[2]: /notes/chores/chores-05.md#the-clock-behind-the-anomaly
[3]: https://github.com/winksaville/iiac-perf/commit/20b2d66e2318 "20b2d66e231832202d0f95585d1dcf81517a3402"
[4]: https://github.com/winksaville/iiac-perf/commit/f14d4877029d "f14d4877029d67fa116a17852d304134be70f90b"
[5]: https://github.com/winksaville/iiac-perf/commit/425fb524cf78 "425fb524cf78af325bc4fca9e54c2060e109490f"
[6]: https://github.com/winksaville/iiac-perf/commit/7ab2bd76dd6f "7ab2bd76dd6f7b83dcdadc52cf4cfdb587a3eef2"
[7]: https://github.com/winksaville/iiac-perf/commit/9b322aeb8e56 "9b322aeb8e56c63621af9c91ebf8ee49bcd6ea4c"
[8]: https://github.com/winksaville/iiac-perf/commit/221a8cab6367 "221a8cab63672f6bbb6dca4f90fac01053e4ab9a"
[9]: https://github.com/winksaville/iiac-perf/commit/0123c0f6c0ca "0123c0f6c0ca5231ed41bfac555a6aafc99eb0ae"
[10]: https://github.com/winksaville/iiac-perf/commit/3ab165869e9b "3ab165869e9b918e253440ab66cf67e92a38ee25"
[11]: https://github.com/winksaville/iiac-perf/commit/491275c70a21 "491275c70a21255bf6822c372ef3040f034af9f4"
[12]: https://github.com/winksaville/iiac-perf/commit/ae66188c505a "ae66188c505aa3c7599245ad047ed24645418e2d"
[13]: https://github.com/winksaville/iiac-perf/commit/28e385d69688 "28e385d696881963e53c79cd33613eaf3262649b"
[14]: https://github.com/winksaville/iiac-perf/commit/24140b6bc530 "24140b6bc5300de0f1194cc72dae90128f31d5ef"
[15]: https://github.com/winksaville/iiac-perf/commit/e2cf85d454e8 "e2cf85d454e84bfa7a15dcb13cf38176bbaa2536"
[16]: https://github.com/winksaville/iiac-perf/commit/a64c866a7e8b "a64c866a7e8b78d14c2f5576daed392ef4379e15"
[17]: https://github.com/winksaville/iiac-perf/commit/d6ee72dddf42 "d6ee72dddf42f83e5f337f87a9d965c439184ca5"
[18]: https://github.com/winksaville/iiac-perf/commit/3134da16b6f6 "3134da16b6f60b4b38beaa02bc8e2fbfc50c2a5d"
[19]: https://github.com/winksaville/iiac-perf/commit/28bd6daad08d "28bd6daad08dd63f24b2bdb2bad42986abcb7c28"
