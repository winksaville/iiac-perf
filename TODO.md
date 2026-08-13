# Todo

This file uses [Prose form](agent-data/prose.md#prose-form). It contains near term tasks with a
short description and uses links or reference links for more details.

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. At Preparation
the picked-up `## Todo` item **moves** here (never copied, one home per text) and becomes six
provisional items, all required, all revised as steps land. At close-out the whole block moves
into `notes/chores/chores-NN.md` and becomes that cycle's `##` section. It is never written in
two places. Shape:

```
### <type>: <title>

#### Problem
<what is wrong, a sentence or two>

#### Solution
<what will be done about it, broad; provisional until the close-out>

#### Acceptance check
<the measure of "are you finished?">

#### Ladder
- [[N]] [<cycle title> opening][M] (done)
- [[N]] [<title>][M] (current)
- [[N]] [<title>][M]
- [[N]] <cycle title> closing

#### Deliberation
<how the five above were decided; `_None._` if there was nothing to deliberate>

#### Ladder details
<one `#####` subsection per rung, headed by its exact title, opened at laddering with the
rung's intent and completed at landing with the conceptual delta; the closing rung's only at
close-out, gotchas in problem/solution form>
```

A multi-cycle program adds one level: the program is the `###`, its current cycle the `####`,
and the six items sit one level below that (headings give the current work durable anchors,
which numbered Todo entries can't). Full rules in
[cycle-protocol.md](agent-data/cycle-protocol.md#preparation); the move's four transforms are
in [Chores sections](agent-data/cycle-protocol.md#chores-sections).

_No cycle currently in progress._

## Todo

Entries are in **strict priority rank**, #1 highest, descending. Reprioritize by moving an
entry, then `vc-x1 fix-todo --no-dry-run TODO.md` to renumber. The numbers are positional rank,
not stable IDs. To refer to a Todo, name it by its **title** (a greppable mention; a numbered
list item has no anchor to link to), not its number. Long-tail entries live in
[todo-backlog.md](notes/todo-backlog.md). Use the
[Prose form](agent-data/prose.md#prose-form); deeper detail goes in
`notes/chores/chores-NN.md` design subsections (link via `[N]` ref).

1. Report interpretation guide: a reader-oriented "how to read a report" walkthrough in README,
   teaching what each surface means and, above all, what to conclude from it
   - surfaces to cover:
     - the band table: first/last/range/count/mean per quantile band
     - the summary rows: mean/stdev, the trimmed row, blocks mean/CI95/LSC
     - the grade block: env warmup/bench vs run rows, per-signal letters, the settle cell
     - the -v warmup table and its exit/window/clock summary line
   - lead with worked examples, the 2026-08-02 3900X trio (plain -d 5, --blocks 100,
     --blocks 1000):
     - duty cycle selects the bistable state: sustained load climbs into the fast state
       (~21.8 ns), while 5 ms bursts between 1-10 ms sleeps hold the slow state (24.0 ns)
     - grade A certifies internal consistency of the state the run held, not a canonical number
     - so A/B wants matched duty cycle; --blocks 1000 read CI95 0.0 ns unpinned by holding one
       state for 13 s
   - the report is dense by design; the guide is the decoder the grade-block compaction (0.23.2)
     assumed exists
2. Blocks as the first-class mode: knobs, always-on error bars, then a measured default flip
   (designed 2026-08-02, the duty-cycle/LSC session; evidence in chores-06)
   - knobs first, a small cycle: `--blocks` gains a config key, and the hardcoded 1-10 ms sleep
     range becomes `--block-sleep` / config (flip-zone hazard: fixed 0.5 ms sleeps straddled
     both 3900X states, D grade, LSC 6x worse), so a box's config can run blocks = 1000 today
   - CI95 / LSC rows always print, `-` when replication is too thin to quote: display gate ~10
     blocks (the t multiplier is 12.7 at df 1, 2.26 at df 9, flat after); plain runs show `-`
     too, so every report answers "how sure" even when the answer is "can't say"
   - same pass: re-house the summary rows (mean through LSC) in a `Results:` section styled
     like `Setup:`, values next to their labels instead of right-aligned ~80 columns away under
     the band table's mean column (wink, 2026-08-02); grade block stays its own section; the
     qualify mean-row parse and README examples follow
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
3. Always work on a topic bookmark: cycles happen on a bookmark, `main` advances only by landing
   a reviewed bookmark, never by direct push (adopted in principle 2026-08-01; process details
   to settle before first use)
   - buys free pre-landing rewrites: the 2026-08-01 renumber needed a coordinated force-push
     only because cycles push `main` directly
   - cycle-protocol.md already anticipates the shape: topic-branch chores sections defer SHA
     backfill until the branch lands on the permanent branch
   - the rules are written as of 2026-08-07: hard rule 13, `cycle.md`'s
     [Cycles run on a bookmark](agent-data/cycle-checklists.md#cycles-run-on-a-bookmark) plus an opening
     checklist and a land step, and `jj.md`'s
     [Cycle bookmarks](agent-data/jj.md#cycle-bookmarks-create-and-land). What is left is the
     habit and vc-x1's review
   - tooling: `vc-x1 push <bookmark>` already takes any bookmark; landing is two jj commands and
     wants a `vc-x1 start-change <bookmark>` for the create half eventually (wink)
   - one process detail is now settled (2026-08-05): a bookmark is a draft until it lands, so its
     ladder stays self-consistent and may be rewritten and force-pushed while unlanded; see
     [Topic bookmarks are drafts](agent-data/cycle-protocol.md#topic-bookmarks-are-drafts)
4. Sync the 20260803 agent-files baseline [[84]]
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
5. Qualification reports evidence, not verdicts: retire the prejudging NOT QUALIFIED stamp
   (wink, 2026-08-02) in favor of measured statements a reader judges
   - blocks-based: A/A repeatability (does a same-code delta clear LSC?), CI95/LSC as the
     published sensitivity ("this box resolves X ns on this bench"), stratification by state
     instead of a blended letter
   - the 3900X reads NOT QUALIFIED today for mid-run bistable transitions warmup cannot
     prevent: a trait to report, not a disqualification; the 7600x dwell case that motivated
     the gate is fixed by the dynamic-warmup cycle
   - entangled with "Qualify the environment without a bench" (below) and machine-readable
     output; wants the blocks-knobs entry (above) landed first
6. Seam-clock attribution: sample `cpuinfo_avg_freq` at batch seams (the reader exists,
   `src/freq.rs`) so a mid-run step gets a "clock moved" label, the way warmup now separates a
   dwell from the top; also the natural home for surfacing the clock ratio in normal output as
   one coherent story (chores-06: the 3900X flip at ~2-4 s is almost certainly a visible clock
   move)
7. Qualify the environment without a bench. `qualify-environment` respawns children running
   `min-now`, but every number in its table comes from the micro-probe series, which never
   touches the bench. The bench is there only to give the warm something to do and to produce a
   report to parse, so the selftest inherits a workload's character it does not want, and the
   parent parses prose (see the machine-readable-output entry below, which this would make moot
   for the selftest).
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
8. Guard `--pin` pools smaller than the bench's thread placements, and deadline the estimate
   phase: `zcr-mpsc-2t --pin 8` put both spinning software threads on one logical CPU and
   appeared hung until ^C (2026-07-26, bug #1 in [bugs.md](notes/bugs.md#bugs))
   - track `core_for` requests in `RunCfg` (max `thread_idx` asked for); refuse the run when
     placements exceed unique CPUs in the pool. Placement only goes through `core_for` when
     pinning is active, so the guard covers every path, and no pinning means the scheduler
     separates the spinners itself
   - wall-clock deadline on the open-loop 5x1,000-step estimate phase so *any* pathologically
     slow bench aborts with a diagnostic naming per-step cost and pinning, instead of hanging
9. Move the batch seam's work off the measuring thread, using the FastForward-style SPSC ring.
   The batch flush stops the bench for ~1-2 ms (a `select_nth_unstable` over up to 65,536 values
   plus 65,536 histogram records) every 50 ms, so ~2-4% of a run is spent at seams. Hand the
   filled buffer to a consumer thread that sorts, summarizes and records while the producer
   fills a second one; the seam drops to a pointer swap
   - the payload is one word, a buffer offset, the exact shape `ffq` is built for, and the
     project dogfooding the queue it benchmarks
   - double-buffered: at ~1-2 ms of work per 50 ms batch the consumer runs ~30x faster than it
     needs to, so two buffers never back up
   - honest cost: the consumer's cross-core traffic runs *during* measurement, trading a gap on
     the hot core for background L3 pressure. Measure it the way the -4 seam probe was measured
     (interleaved A/B, pinned, trimmed mean) rather than assuming
   - blocked on the ring existing; see the "FastForward-style SPSC ring" entry, currently on the
     `ffq-spsc-notes` bookmark rather than `main`
10. Tighten thread/CPU terminology across docs and doc comments: "software thread" for what
    `thread::spawn` makes, "logical CPU" (hardware thread) for what `--pin` selects and the OS
    schedules onto, "physical core" for the engine SMT siblings share. Bare "core"/"CPU"/"thread"
    only where context disambiguates
    - spin-wait bench docs state the precondition: each spinning software thread needs its own
      logical CPU
    - `--pin` help/README say slots are logical CPU ids
11. Topology-aware pinning and lCPU terminology: discover the CPU sharing tree at runtime and
    describe every pin by the nearest shared level, not "unique CPUs". Evidence: the 2026-08-01
    pinning experiment (`zcr-with-2t -d 30 --blocks 5` on the 3900X, boost on) measured the round
    trip at ~35 ns on SMT siblings (shared L1/L2), ~133 ns same-CCX (shared L3), ~633 ns
    cross-CCX (shared fabric only); cross-CCX vs cross-CCD differed by 1.6 ns against a ~2 ns
    LSC, so the L3 boundary is the only fabric tier that matters on Zen 2, and the unpinned
    scheduler's ~127-135 ns core mass matches same-CCX placement
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
    - subsumes the vocabulary half of "Tighten thread/CPU terminology" (above): keep its
      software-thread vs lCPU distinction, adopt lCPU as the standard term
12. Rebase `web-claude-tweaks` onto post-0.22.0 `main`. It rewrites an already-published
    bookmark (needs approval) and its arbitrary `0.21.0-b` version needs replacing; owed from
    the 0.22.0 close-out plan
13. Unit scaling in report columns (`us`/`ms`): per-row auto-scale so columns stay
    eyeball-comparable (bands are monotonic, so a row's first/last/mean share a magnitude), or
    `--units ns|auto` for script-stable output; needs `--decimals` landed first (`3.18 ms` vs
    `3 ms`); candidate `-4` for the report-options cycle.
14. Machine-readable report output (`--format json`, or key=value lines to stay
    dependency-light). Design once the batch gauge lands (0.23.0-4) so the schema covers the
    surviving surface: report stats, gauge signals, letter. Consumers:
    `tests/qualify_environment.rs` (drops its brittle-but-loud line parsing), placement-map
    validation runs, cross-run comparison scripts. Kin to the unit-scaling entry's `--units ns`
    script-stable concern (above), one flag family.
15. Trimmed core stats: `mean/stdev p10-p90` report row, additional to (never replacing)
    `mean` / `mean min-p99`; trim bounds possibly configurable (`--trim p10:p90`?). Why: the
    full mean wobbles ~±1.4% with the run's mode mix while the core plateau is ~±0.2% stable,
    so the trimmed row is the run-to-run comparable number. Boundary sensitivity (see [[57]]):
    window edges in the mode-mix smear inherit its wobble (p50-p60 ±0.05% vs p40-p50 ~1%), so
    also consider a dominant-*mode* statistic (peak-density region, bottom-count-independent)
    [[57]]
16. Find and label the interference crossover: the band where the tail stops measuring the code
    and starts measuring the machine. Not to hide it: to *name* it, because that is the signal
    TProbe exists to surface (the OS swapping, a drive stalling, anything not caused by the
    code under test).
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
17. Investigate: suspend gap missing from samples. A 0.13.5 `--no-inhibit` suspend test
    detected ~1.2 s suspended inside the measured window but the max sample was only 4.0 ms,
    while the 0.13.1 test (8.4 s gap) showed the expected 10.4 s max sample. We think
    minstant's TSC may halt across some suspends and count through others. Repeat the test
    comparing detected gap vs max sample; if the TSC halts, per-sample timing silently loses
    suspend time; document either way.
18. CLAUDE.md governance model (design cogitation) [20]
19. Revisit probe adjustment under the in-interval vs call-to-call split: probes take one call
    per sample (inner=1), so the in-interval timer slice is unamortized and unmeasurable, so an
    `adjusted` column can subtract nothing defensible; maybe state a bound instead
    [analysis](notes/design.md#timer-overhead-in-interval-vs-call-to-call)
20. Convert `harness` / `Bench` to probe-based measurement. Will likely need inner-loop support
    on `Probe` (batch N calls per sample; report divides by N and accounts for per-sample
    framing) so very-small workloads can still amortize timer overhead the way `run_adaptive`
    does today.
21. Rename app
22. Design an app to measure IIAC perforanace written in Rust[1]
23. `ice-ps-2t-wait`: iceoryx2 pub/sub with blocking waits via `Listener`/`Notifier` events;
    completes the {transport} × {wait policy} matrix cell that compares against `mpsc-2t`
24. Switch ice benches to the loan-based zero-copy send path (`loan_uninit` + `send`), the API
    a perf-sensitive user would use, and closer to iceoryx2's own benchmark method
25. Payload-size sweep for the round-trip benches (8 B / 8 KiB / 1 MiB), makes iceoryx2's
    size-independent latency vs channel copy cost visible in our own tables
26. `crossbeam-1t` / `crossbeam-2t`: `crossbeam-channel` directly (compare to mpsc-1t/2t which
    use crossbeam under the std API)
27. `tokio-mpsc-1t` / `tokio-mpsc-2t`: `tokio::sync::mpsc` round-trip inside a Tokio runtime
    (async overhead)
28. `flume-1t` / `flume-2t`: `flume` MPMC channel
29. Function-call baselines: direct call vs `Box<dyn Trait>` vs `async fn` (poll-once): anchors
    the channel/serde numbers against the cheapest possible "send a value then receive it" path
30. When the second channel impl lands, extract shared message types + round-trip helpers into
    `src/benches/common.rs` (deferred from 0.2.0)
31. Additional thread control (count, per-thread pin lists, NUMA): shape once a concrete bench
    needs it
32. Rename crate `iiac-perf` -> general-purpose name (breaking; deferred)

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

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed and older
`## Done` sections are moved to [done.md](notes/done.md) to keep this file small.

- **feat: dynamic warmup** [[83]]
  - the first cycle run on a topic bookmark
  - one parameterized warm loop, its warm-until-stable exit fused with sizing: the trailing window
    grades A and the delivered clock holds, where readable
  - warm follows the bench's pin
  - settle time is the earliest A-grading suffix
  - configurable 1.5 s cap, with `warm=used/budget` visibility
  - the 7600x vacuous-A defect closed: all-A, settle 0.77 s riding through the dwell
  - older entries retired to [done.md](notes/done.md)
- **docs: experiment in the local agent-files** [[84]]
  - single-commit cycle inverting hard rule 12
  - a proposed agent-file change is edited into the member's local copy, so the diff against the
    template payload is the proposal set and the commit history its durable record
  - `custom.md` narrows to medium-determined content plus elective divergence that must say why it
    cannot be family-wide
  - its dogfood log carries a status, and in-flight entries only
- **docs: steps are titles, versions are stamps** [[85]]
  - single-commit cycle taking both the version and the step number out of durable prose
  - a ladder rung is a bare title, its place in the list being its place in the ladder
  - a title need only be unambiguous within its cycle and within its chores file
  - a commit body is a problem statement plus a solution statement, both broad and with no file
    list; the diff is the mechanical record and the deliberation goes to chores, todo, and the
    session
  - a topic bookmark is a draft whose ladder stays self-consistent until it lands
  - one exception: a chores as-built rung records the version a landed commit carried, beside its
    SHA, and takes the SHA's timing, so an unlanded rung carries neither
  - `## Done` entries become a bold title plus sub-bullets, after the version turned out to have
    been doubling as the eye's landmark in this list
  - clears the `feat: dynamic warmup` backfill debt, eight rungs whose commits landed on `main`
    two cycles ago
- 0.24.3 **docs: one owner per rule, one home per record** [[86]]
  - hard rule 13: cycles run on a topic bookmark, and `main` advances only by landing one;
    `cycle.md` gains an opening checklist and a land step, `jj.md` the commands
  - landing is the beat that makes a cycle's commits permanent, so it now owns the chores backfill
    that had been waiting on permanence with no trigger
  - a cycle's record has one home at a time: `TODO.md > ## In Progress` while it runs, moved into
    chores at close-out, replacing the per-commit build-up that wrote every rung twice
  - the six provisional items a cycle states at Preparation: title, problem statement, solution
    statement, acceptance check, ladder, deliberation
  - `custom.md` shrinks to a payload stub with nothing to substitute; `custom-family.md` holds the
    medium, this project's membership, the messaging rules, and the dogfood log
  - `CLAUDE.md` collapses to `@AGENTS.md`, so nothing below it is auto-loaded and hard rule 0 is
    load-bearing
  - four of vc-x1's six 2026-08-07 items adopted: the symlink correction, the https-remote line,
    the acceptance check, and the version-leading `## Done` form
- 0.24.4 **docs: the bot pushes again** [[87]]
  - retires the 2026-08-06 `permanently local` dogfood entry that routed every push through
    wink's terminal, after a 3.0 MB sandboxed push succeeded where 3.4 MB had failed twice
  - we think vc-x1 0.78.x's in-process jj-lib transport is the fix, inferred rather than
    measured, with the limits of the inference recorded
  - the cycle's own push is its acceptance check, which is why it is a cycle and not an
    amendment: a commit cannot contain evidence produced by pushing it
- 0.24.7 **docs: adopt the commit-body form** [[88]]
  - vc-x1 pinned the commit-body form this repo proposed the same day, so the single-step cycle
    is a straight copy of `prose.md`, `cycle-protocol.md`, and `cycle-checklists.md`
  - their three departures from our proposal all taken: prose.md is the form's single home and
    the other two link it, the intro-mandatory rationale drops our clap history under
    `Pinned files name no project`, and the `## In Progress`-edits question stays unpinned
  - the pinned set is byte-identical to vc-x1's again, which is the acceptance check
  - the formal review owed since 2026-08-08 and the two questions their 2026-08-12 message asks
    are deliberately not closed here

# References

[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[75]: /notes/chores/chores-05.md#settle-time-is-not-a-grade
[83]: /notes/chores/chores-06.md#feat-dynamic-warmup
[84]: /notes/chores/chores-06.md#docs-experiment-in-the-local-agent-files
[85]: /notes/chores/chores-06.md#docs-steps-are-titles-versions-are-stamps
[86]: /notes/chores/chores-06.md#docs-one-owner-per-rule-one-home-per-record
[87]: /notes/chores/chores-06.md#docs-the-bot-pushes-again
[88]: /notes/chores/chores-06.md#docs-adopt-the-commit-body-form
