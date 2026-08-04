# Todo

This file uses [Prose form](AGENTS.md#prose-form). It contains near term tasks with a short
description and uses links or reference links for more details.

## In Progress

When a `## Todo` item is picked up, its text moves here: the problem overview and its list of
things to do. That is followed by the "plan", a bulleted list of the development "ladder":
   - [[N]] 0.xx.y-0 blah (done)
   - [[N]] 0.xx.y-1 blah blah (current)
   - [[N]] 0.xx.y-2 blah blah blah
   - [[N]] 0.xx.y close-out and validation

**feat: measure reproducibility**

No report this project has produced records the machine's power policy, and no run's numbers
survive the session that produced them. The 2026-08-03 pinning experiment paid both costs at
once: a policy delta that cannot be separated from box history, and a powersave series that
exists only in a transcript because the rerun overwrote its files. This cycle makes a run
self-describing (the policy on the display, one machine-readable record per run on disk), then
reruns the pinning experiment on 3900X, 7600x, and rpi5-20cd.

What the harness is for, stated here because it decides calls like the ones below and is written
down nowhere in the repo (wink, 2026-08-04): **A/B comparison of algorithm changes**, did this
change make it faster or slower. It deserves a permanent home in the README's overview; the cycle
block is where it landed first.

- the far tail is a validity check, not the score: beyond ~n4 on a 20 ns operation it measures OS
  interruptions (context switches, IRQs, faults, migrations), so it says whether to trust a
  comparison, never whether the algorithm improved
- the near tail is not in that bucket: cache miss rates, a ring's wrap boundary and the 2t benches'
  producer/consumer phase relationships live around p99, and a change that improves the median
  while doubling p99 is usually a worse algorithm
- a comparison needs a resolution claim, not just a number, which is what makes rung -4
  load-bearing rather than last: an A/B verdict is only as good as the smallest delta the tool can
  honestly distinguish

Carried from the Todo entry, the measurement that motivates the cycle (measured 2026-08-03; the
session's terminal stamps read 26-08-04 UTC):

- the measurement: `min-now --blocks 200`, 8 pinned + 8 unpinned per policy, alternating so box
  history is shared between conditions, under amd-pstate-epp `powersave` then `performance`
  - powersave unpinned: 23.927 ns, run-to-run stdev 0.785 ns (3.28%), grades 4 A / 2 D / 2 F
  - powersave pinned to one core: 22.207 ns, 0.073 ns (0.33%), all 8 graded D
  - performance unpinned: 21.980 ns, 0.057 ns (0.26%), no D or F in 17 runs
  - performance pinned: 22.359 ns, 0.311 ns (1.39%)
  - what that supports, and how strongly:
    - the policy delta is big and one-directional: 8.9% on the unpinned mean, with unpinned
      run-to-run scatter falling from 0.785 ns to 0.057 ns
    - but policy could not be alternated the way pinning was (setting EPP needs root and is
      global and persistent), so the two policies are separated in time and the delta is
      confounded with box history exactly as the `--blocks` sweep was. We think an effect this
      size survives de-confounding, but this experiment does not show it
    - pinning under powersave is the strongest result here: 7.2% off the mean and 10x off the
      scatter, on alternated runs
    - pinning under performance is the weakest: 1.7% *worse* than unpinned, and the whole gap is
      four consecutive pinned runs (p3-p6) at 22.59-22.70 ns while the pinned runs on either side
      sat at 22.08-22.09 ns and the unpinned runs interleaved with them did not move. So the
      0.311 ns is the width of a state change, not scatter about a mean
    - the pinning answer therefore looks policy-dependent, but "pinning loses under performance"
      rests on that one cluster and should be treated as unreproduced
- the finding that reframes the grade: A was being awarded to the un-boosted floor. A powersave A
  run puts ~73% of its samples on 23.967 ns, the 3.7929 GHz TSC/base rate, while the boosted
  states are 21.79 / 22.22. Flatness is all the grade sees, and base clock is the flattest place
  on the box
- rung -2's spec: move the report renderer out of `harness.rs` into its own module, a pure move
  plus the `_w` -> `_cols` rename that already landed in -1
  - `harness.rs` is ~2,500 lines carrying the `Bench` trait, the warm loop, sizing, the batch
    pipeline, the run loop *and* ~350 lines of renderer, which shares nothing with the measuring
    code except the value it reads
  - the seam already exists: `RunOutput` is what `print_report` takes. What is missing is a
    module boundary, not a design
  - it goes *before* the record and not after, so -3's diff reads as "add the record" rather than
    "move 350 lines and add the record". The record is also what makes the split honest: today
    `RunOutput` has one consumer, so "what belongs in the model" is whatever the printer needs,
    and two consumers is what forces a real answer
  - visible from there, not folded in: `band_table.rs` renders a band table for `tprobe` /
    `tprobe2` with the same shape as `print_report`'s, so the project has two band-table
    renderers. Its own cleanup, once they are in one place
- rung -4's spec: `qualify-environment` reads the policy as a fitness precondition and says so
  before spending minutes on numbers it can predict will scatter
  - a diagnosis, never a mutation: setting the governor needs root and is global and persistent,
    and the 2026-08-03 session's documented revert left EPP at `performance` after the governor
    had already returned to `powersave`
- rung -5's spec, the cycle's most valuable rung given the A/B purpose above: LSC is scoped to
  within-run block agreement but reads as a run-to-run bound
  - the best configuration printed LSC 0.022 ns against a measured run-to-run stdev of 0.057 ns,
    so single-run against single-run resolution is ~0.157 ns (0.71%) where the report prints
    0.10%
  - about 7x optimistic, and it did not improve when the environment did
  - the fix is a **variance-versus-aggregation curve**, not a second measurement: a single run
    runs once, so it cannot measure run-to-run scatter directly. Aggregate its blocks in groups
    of 1, 2, 4, 8, ... and watch whether variance falls as `1/n`. Where it stops falling is the
    drift floor, and that floor is the run's honest resolution
    - this is Allan deviation (IEEE Std 1139), the standard tool in clock metrology for "how
      long should I average"; the harness is already a clock project, so the machinery is
      familiar rather than exotic
    - the alternative, spawning children as `qualify-environment` does, is a much larger change
      and still cannot re-roll thermal or P-state history, so it does not actually reach the
      component we are missing
    - the duration estimate falls out of the same curve for free:
      `t_needed = t_now * (SE_now / SE_target)^2`, valid only in the region where the `1/n`
      scaling still holds, which the curve is what tells us
    - naming it accurately matters as much as computing it: a within-run bound must not print in
      a way that reads as a run-to-run one, which is the whole defect
- evidence is perishable: `tmp/pinexp/` holds only the performance series (the rerun overwrote
  powersave), so the powersave numbers live only in the session transcript until copied into a
  chores design subsection
- cross-cuts the Todo entries below: the interpretation guide's worked trio and the blocks entry's
  duty-cycle evidence were both collected without recording the policy, and a four-run `--blocks`
  sweep proved confounded (block count and box history rose together, history dominating), so
  re-check those examples before they ship as teaching material
- raises "Seam-clock attribution": we think the three states map onto ~3.79 / 4.17 / 4.35 GHz,
  inferred from timing ratios alone, and a seam clock sample would settle it

Design decisions taken at pickup (2026-08-04):

- the record is per bench *result*, not per process: `all` emits one record per bench sharing the
  host / policy / clock stamp, so anything the harness runs is recordable, not just `min-now`
- the display is never traded for the file: the report prints exactly as it does today and
  `--record <path>` appends alongside it, so recording is a side channel, not a mode
- NDJSON, one object per line: `jq -s .` makes an array on demand, an interrupted run leaves a
  file that still parses, and per-run files concatenate with `cat`
- the tool names the file when handed a directory, because a fixed name is exactly what killed
  the powersave series (`run.sh` reused `u1.txt`..`p8.txt` and the rerun clobbered them). Leaving
  naming to the script means relying on the one mechanism with a demonstrated failure
  - `--record <dir>/` writes one file per run, stamped `<ts>-<host>-<bench>.ndjson`;
    `--record <file>` appends to that file. The path's shape picks the mode
  - the open is `O_APPEND | O_CREAT`, never `O_TRUNC`, in both modes. The no-truncate invariant is
    what actually protects the evidence, whoever chose the name
  - basic ISO to the second in the filename (`20260804T093221Z`: no colons, and lexicographic
    order is chronological order), RFC3339 with millis inside the record
    (`2026-08-04T09:32:21.482Z`, which every parser takes for free), plus the local offset as its
    own field. This project has already tripped on that gap once, see the 26-08-04 UTC note above
  - millis are not decoration: with one record per bench, `all` emits several records inside one
    second, so each carries a process id and an index within the process
- `--tag k=v` is recorded verbatim and never interpreted, so a driving script labels condition /
  policy / box without the tool needing a notion of "series"
  - a per-run stamp is not a per-series stamp: only the caller knows which runs form one
    experiment, so `--tag series=<ts>` carries that, in every record of the series
  - no tag key is ever substituted into a path. That is where a template language starts, and the
    next request is `%h` for hostname
  - it lands with the record in -3: a field of the same struct sharing all of its plumbing, so a
    rung of its own would review nothing
- the record carries a **fixed quantile ladder** (0.01 / 0.1 / 1 / 5 / 10 / 25 / 50 / 75 / 90 / 95
  / 99 / 99.9 / 99.99), not the report's populated bands
  - the bands are chosen per run for display and their labels move with the data, so a series of
    records built from them cannot be compared column-to-column
  - it is what makes the A/B estimator question (Todo: pick the A/B estimator by measurement)
    answerable after the fact: candidate estimators get scored on run-to-run scatter over an
    existing series instead of requiring a rerun
  - cheap now, impossible to backfill: a record without it cannot be re-analysed, only replaced
- the record carries the **series of block means**, not just their summary, for the same reason
  - blocks mean / CI95 / LSC are aggregates, and an aggregate cannot be decomposed. The series is
    what lets within-run scatter be compared against across-run scatter, which is the question
    every A/B claim rests on and the one that decides whether a future comparator needs child
    processes at all
  - bounded by a cap: keep every block mean while blocks <= 1000, summarize beyond. 200 floats
    per record is nothing next to being unable to answer the question later
- the record documents its own fields, and a test enforces it
  - `describe-record`, a command word beside `all` / `qualify-environment` /
    `add-completion-yaml`, prints the field dictionary: name, unit, one-line meaning. That is the
    door for whoever opens an archived record in two years with no idea what `frame_ns` was
  - not `--help`, which documents *inputs*. These are outputs, and mixing what you can ask for
    with what you get back makes both harder to scan in an already long help
  - one source of truth is a const descriptor table, kept honest by a test that serializes a
    sample record, walks its keys and fails on any key with no entry. That turns "added a field
    without documenting it" into a build failure rather than a convention to remember
  - every record carries `schema_version`, so a dictionary printed by today's binary can be
    checked against a record written by an older one instead of silently assumed to apply
  - later polish, not -3: per-field lookup (`explain frame_ns`) and generating README text from
    the same table. The report's own rows have the same problem and belong to the interpretation
    guide Todo; the two dictionaries may merge once both exist
- absent is not zero: rpi5-20cd has no EPP and no `cpuinfo_avg_freq`, so every policy field is an
  `Option` recorded as absent
  - three states, not two: absent, present and uniform, present and split across policy groups.
    `freq::PolicyField` carries the token plus a `uniform` flag so the display can say
    `(mixed across CPUs)` rather than let one CPU stand in for the box
- ordering is part of the record: the 2026-08-03 pinned series was bimodal by *position*
  (p3-p6), which is invisible without a per-run wall-clock start
- comparison across the three boxes is within-box only: each box's pinned-vs-unpinned delta and
  its run-to-run scatter, never nanoseconds against nanoseconds
- the clock's quantum is a property of the box, not the bench (measured on rpi5-20cd,
  2026-08-04). `inner` is sized for framing domination,
  `inner = ceil(10 * frame_ns / step_cost_ns)` (`src/harness.rs:1119`), so in
  `q = tick_ns / inner` the step cost cancels and `q / step = tick_ns / (10 * frame_ns)`
  - ~2.2% on the Pi (18.5185 ns tick, ~82 ns frame) against ~0.05% on the 3900X (0.264 ns tick,
    ~50 ns frame): a factor of 40, applying uniformly to every bench either box runs
  - three benches confirm it, predicted `q` against the spacing between value clusters in the
    band table: `min-now` inner 23, q 0.805, seen 0.800 / 0.832 / 0.768; `zcr-with-1t` inner 47,
    q 0.394, seen 0.400 / 0.384 / 0.408; `zcr-with-2t` inner 4, q 4.630, seen 4.352 / 4.608 /
    4.864. The residual wobble is hdrhistogram bucketing (0.016 ns at 17 ns, 0.256 ns at 270 ns)
  - so a printed per-sample spread below ~q describes the clock, not the workload: `min-now`'s
    `stdev p20..p80 0.403` is half a quantum and `zcr-with-1t`'s 0.213 is a two-point split,
    while `zcr-with-2t` spans ~11 quanta and its 4.906 is real to within 4%
    (`q/sqrt(12)` removed in quadrature)
  - the means, the grades and LSC are unaffected: all work from batch or block means over ~1.2M
    samples, where quantization averages away as `1/sqrt(N)`
  - so print and record it, do not size for it: `q` joins the `Setup:` block in rung -1, and
    `ticks_per_ns` and `inner` are already record fields so it stays derivable per result
  - a granularity floor on `inner` is the wrong fix and is its own Todo, not a rung here. The
    dither makes quantization zero-mean, so a 5M-sample mean carries `q/sqrt(12N)` = 0.0001 ns of
    it; raising `inner` would buy nothing there while smearing the tail linearly (each sample is
    an average of `inner` calls, so a spike is divided by `inner` before it is ever seen), and it
    cannot reveal per-call shape below one tick either way
- the dither works on the coarse lattice, twice confirmed on the Pi: `zcr-with-1t`'s mass sits on
  two adjacent lattice points (2.08M at ~16.951, 2.94M at ~17.335) and interpolates to 17.18
  against a printed `mean z4..n2 17.183`, with the two-point stdev prediction 0.19 against a
  printed 0.213
  - so an off-lattice value is recovered rather than snapped, which is what `DITHER_SPAN` exists
    to do; the earlier worry that ~26-32 ns of span against an 18.5 ns quantum would bias it is
    not visible in the data
  - the decisive check remains cheap and unrun: sweep `-i` (23 / 97 / 233) on the Pi and confirm
    the mean does not move as `q` shrinks

The ladder:

- [[N]] 0.25.0-0 feat: measure reproducibility opening (done)
- [[N]] 0.25.0-1 feat: report the power policy and clock quantum (done)
- [[N]] 0.25.0-2 refactor: extract the report renderer (done)
- [[N]] 0.25.0-3 feat: write a per-run JSON record
- [[N]] 0.25.0-4 feat: qualify-environment reads the power policy
- [[N]] 0.25.0-5 fix: LSC gains a run-to-run component
- [[N]] 0.25.0 feat: measure reproducibility

The three-box rerun sits between -3 and -4: it is evidence, recorded in the chores section, not a
rung. Everything the rerun needs has landed by -3. Run it **with `--blocks`**, so every record
carries a block-mean series and the within-run against across-run decomposition comes out of the
same dataset: no code, no extra runs, and without it that question stays a guess.

## Todo

Entries are in **strict priority rank**, #1 highest, descending. Reprioritize by moving an
entry, then `vc-x1 fix-todo --no-dry-run TODO.md` to renumber. The numbers are positional rank,
not stable IDs. To refer to a Todo, name it by its **title** (a greppable mention; a numbered
list item has no anchor to link to), not its number. Long-tail entries live in
[todo-backlog.md](notes/todo-backlog.md). Use the
[Prose Form in AGENTS.md](AGENTS.md#prose-form); deeper detail goes in
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
2. Interleaved multi-arm comparison: take a list of benches as *arms* of one comparison and rank
   them, which is what the harness is actually for (raised 2026-08-04; ranked here on arrival,
   move it up if it should lead)
   - **sequenced after the measure-reproducibility rerun, deliberately.** Every open design
     question below is one that rerun's records answer: which estimator has the smallest
     run-to-run scatter, whether re-exec matters or in-process rounds dominate, and how big each
     box's drift floor is. Building first means guessing all three and rebuilding
   - the vocabulary it needs, added when the code does and not before: an **arm** is one thing
     being compared (a bench, or a bench plus pin plus config), a **round** is one pass over all
     arms. Each level of the existing hierarchy earns its name by what it re-randomizes: a block
     re-rolls short-term scheduling noise, a fresh process re-rolls ASLR / allocator / page
     mapping / thread placement, separation in time re-rolls thermal and P-state history, a
     reboot re-rolls everything
   - interleave, never concatenate: A-then-B confounds the difference with drift, while
     alternating makes every shared factor common-mode so it cancels in the paired difference.
     Counterbalance the order (ABBA, rotating with three or more arms) so linear drift cancels
     rather than aliasing onto the result
   - the cost interleaving buys, and it must be stated in the report: both arms are resident at
     once, so one can evict the other's working set and the delta includes that interaction.
     Separate processes trade it the other way, drift for interference
   - report the **difference and its own uncertainty**, from the per-round paired differences.
     Not each arm's absolute uncertainty: the paired SE shrinks with rounds while the absolutes
     stay noisy, and that is the entire reason to build this rather than eyeball two reports
   - ranking by mean always yields a strict order, including from noise. Report the ordering plus
     which adjacent pairs are **not separated** at the run's resolution; "B and C are
     indistinguishable below 0.4%" is the honest answer
   - deciding the duration needs one number the tool cannot invent, the smallest difference worth
     detecting (default ~1%, `--resolve` to override). Then run until every adjacent pair is
     separated at that target, the budget is spent, or the drift floor says longer will not help,
     and **always report which condition fired**
   - the trap to avoid: stopping the moment a CI first excludes zero inflates false positives.
     Fix the round count from a pilot, or use an always-valid sequential bound
3. Blocks as the first-class mode: knobs, always-on error bars, then a measured default flip
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
4. Always work on a topic bookmark: cycles happen on a bookmark, `main` advances only by landing
   a reviewed bookmark, never by direct push (adopted in principle 2026-08-01; process details
   to settle before first use)
   - buys free pre-landing rewrites: the 2026-08-01 renumber needed a coordinated force-push
     only because cycles push `main` directly
   - cycle-protocol.md already anticipates the shape: topic-branch chores sections defer SHA
     backfill until the branch lands on the permanent branch
   - tooling: `vc-x1 push <bookmark>` already takes any bookmark; missing is a "land" step (ff
     `main` to the bookmark) and the habit; propose to the template after dogfooding here
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
33. Pick the A/B estimator by measurement, not by taste: which location and spread pair best tells
    a real algorithm change from noise (raised 2026-08-04; rank it where it belongs)
    - today's trimmed row is a one-sided 1% upper trim, the mean of everything at or below the n2
      (p99) cut, fixed for every run. Only its *label* moves, since `trim_range_label`
      (`src/harness.rs:1607`) names the populated bands, so the same estimator prints as
      `mean z2..n2` on one box and `mean p20..p80` on another. That alone is worth fixing for A/B,
      where two runs should print comparable row names
    - a 1% cut still admits contamination: the 3900X's n2 band spans 22.2 to 26.6 ns against a
      21.79 ns mode, so OS noise sits inside the estimator. Narrower is probably better, but it
      trades away the near tail, which is algorithmic
    - one-sided is right and should stay: the distribution has a physical floor and a long right
      tail, so the literature's symmetric default (Wilcox's 20% trimmed mean with Yuen's test) is
      shaped for the wrong contamination. Candidates worth scoring: the minimum (Chen and Revels,
      "Robust benchmarking in noisy environments", 2016), a low quantile, the median with the
      Hodges-Lehmann / Mann-Whitney pair, mean <= p99 (today), mean <= p90
    - the decisive move is empirical, and the records make it cheap: compute every candidate over
      one repeat series and keep the one with the smallest *run-to-run* scatter. That is the
      property an A/B verdict actually rests on
    - which needs the record to carry a fixed quantile ladder (0.01 / 0.1 / 1 / 5 / 10 / 25 / 50 /
      75 / 90 / 95 / 99 / 99.9 / 99.99), not just the report's populated bands, or the experiment
      cannot be run after the fact
    - within-run trimming is the smaller half regardless: Kalibera and Jones, "Rigorous
      Benchmarking in Reasonable Time" (ISMM 2013), and Georges et al., "Statistically Rigorous
      Java Performance Evaluation" (OOPSLA 2007), both put the dominant uncertainty at the
      invocation level, which is this project's own finding (LSC 0.022 ns within-run against
      0.057 to 0.785 ns run-to-run) and rung -4's subject
34. Give `inner` a clock-granularity floor, so the per-sample quantum is a choice rather than a
    consequence (ranked last on arrival; move it if it deserves better)
    - today `inner = ceil(10 * frame_ns / step_cost_ns)` sizes for framing domination only, which
      fixes the relative quantum at `tick_ns / (10 * frame_ns)`: ~0.05% on the 3900X but ~2.2% on
      rpi5-20cd, whose Generic Timer ticks at 54 MHz (evidence in the measure-reproducibility
      cycle's chores section)
    - the shape: `inner = max(framing_target, granularity_target)`, the second sized from the tick
      period and a target relative quantum
    - deliberately not done inside the measure-reproducibility cycle: it changes sizing for every
      bench on every box, and that cycle's job is to record the environment, not to change what
      the harness measures
    - the cheap alternative, or the companion: print `q` in the report and flag a per-sample
      spread within ~2q, which is where the number stops describing the workload

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

- feat: dynamic warmup [[83]]: the 0.24.0 cycle, first run on a topic bookmark: one
  parameterized warm loop; warm-until-stable exit (trailing window grades A and the delivered
  clock holds, where readable) fused with sizing; warm follows the bench's pin; settle time is
  the earliest A-grading suffix; configurable 1.5 s cap with warm=used/budget visibility. The
  7600x vacuous-A defect closed (all-A, settle 0.77 s riding through the dwell); older 0.23.x
  entries retired to [done.md](notes/done.md)

# References

[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[75]: /notes/chores/chores-05.md#settle-time-is-not-a-grade
[83]: /notes/chores/chores-06.md#feat-dynamic-warmup
