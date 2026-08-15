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
- [[N]] [<cycle title> closing][M]

#### Deliberation
<how the five above were decided; `_None._` if there was nothing to deliberate>

#### Ladder details
<one `#####` subsection per rung, closing included, headed by its exact title, opened at
laddering with the rung's intent and completed at landing with the conceptual delta, the
closing rung's at close-out with gotchas in problem/solution form>
```

A multi-cycle program adds one level: the program is the `###`, its current cycle the `####`,
and the six items sit one level below that (headings give the current work durable anchors,
which numbered Todo entries can't). Full rules in
[cycle-protocol.md](agent-data/cycle-protocol.md#preparation); the move's four transforms are
in [Chores sections](agent-data/cycle-protocol.md#chores-sections).

### feat: measure reproducibility

#### Problem

Modern CPUs change their clock frequency continuously to save power. The OS's **power policy**
(the cpufreq governor plus, on AMD, the EPP hint) decides how eagerly the chip boosts under load
and how quickly it retreats, so the clock a run executes at depends on the box's recent history,
not its spec sheet. A latency number is cycles times cycle time, and the 2026-08-03 experiment
showed both failure shapes: under `powersave` the same code differed ~9% run to run as the clock
wandered, and a run parked on the un-boosted base clock graded A, flat being all the grade sees.
For A/B comparison the need is consistency, not speed: a stable clock at any rate beats an
unstable fast one. Today the harness cannot report the policy, cannot hold the clock still, and
no run's numbers survive the session that produced them, so a reader cannot tell a code delta
from a power-management delta. (Continues the cycle stalled since 2026-08-04.)

#### Solution

The stalled branch's finished rungs (the policy and clock-quantum report rows, and the report
renderer extracted into `src/report.rs`) are ported onto `main`'s line as this cycle's opening,
by file copy rather than rebase. The remaining rungs then make a run self-describing, steady,
and durable:

- the settle cell gains the clock gate the warmup exit already has, and names the state it
  settled into
- a per-run NDJSON record
- the config gains vc-x1's markdown carrier, a `.md` whose `toml` fences are the config and
  whose prose documents it, with links, plain `.toml` still accepted (one type per directory)
- frequency control: `read-freq` / `pin-freq` / `restore-freq` command words, a declared steady
  state in that config, and a run that can pin the clock at start and restore it on every exit
  path we can catch
- `qualify-environment` reading the power policy as a fitness precondition
- an honest run-to-run resolution for LSC

The three-box rerun of the pinning experiment (3900X, 7600x, rpi5-20cd) follows the frequency
rung as evidence.

#### Acceptance check

Four measures:

- A run with `--record` leaves a record that can be re-analysed without its session: it carries
  `schema_version`, the fixed quantile ladder, the block-mean series, and the box's policy
  fields, and `describe-record` documents every field, enforced by a test that fails on any
  undocumented key.
- `qualify-environment` names the power policy before spending minutes on numbers it can predict
  will scatter.
- The settle cell agrees with the clock: a 3900x `powersave` run no longer reads `0.01s` beside
  an F, and the cell names the state it settled into.
- The report's resolution claim is the variance-curve drift floor, not the within-run LSC that
  read 7x optimistic against measured run-to-run scatter.
- `pin-freq` then `restore-freq` leaves the box in the declared steady state (governor, EPP,
  min/max, boost), a pinned run restores on normal exit, panic, and SIGINT, and `restore-freq`
  converges from any starting point, an unclean death included.
- The port preserved behavior: full validation green on `main`'s line at every rung.

#### Ladder

- [[N]] [feat: measure reproducibility opening][98] (done)
- [[N]] [fix: the settle cell reads the clock][106]
- [[N]] [feat: write a per-run JSON record][99]
- [[N]] [feat: adopt the markdown config carrier][105]
- [[N]] [feat: read, pin, and restore the CPU frequency][104]
- [[N]] [feat: qualify-environment reads the power policy][100]
- [[N]] [fix: LSC gains a run-to-run component][101]
- [[N]] [feat: measure reproducibility closing][102]

The three-box rerun sits between the frequency rung and the policy rung: it is evidence,
recorded in the chores section, not a rung, and everything it needs has landed by then, records
and pinning both, so it can add a pinned-clock condition to the original's four. Run it
**with `--blocks`**, so every record carries a block-mean series and the within-run against
across-run decomposition comes out of the same dataset.

#### Deliberation

**Ported, not rebased** (wink, 2026-08-15): `jj rebase` of the original `measure-reproducibility`
branch produced four conflicted commits and was op-restored away. What worked was copying the
branch's `*.rs` files onto `main`, which passed `cargo check` on the first try. The full story is
[Port measure reproducibility][103] below.

**The original ladder's first three rungs fold into the opening.** Their code is the port,
their review happened on the original branch, and re-laddering them would re-commit finished
work. The remaining rungs carry over with their specs intact in `Ladder details`.

**The title reuses the stalled cycle's.** That cycle never reached a chores section, so nothing
in chores-07 collides, and the greppable stem stays continuous across both attempts.

**The bookmark keeps wink's name.** `port-measure-reproducibility` was created before this
ladder existed, so it is not the title's slug. A taken exception to the bookmark-naming rule
rather than a rename of wink's bookmark while he is away.

**0.26.0 on wink's call**, minor rather than patch: the record subsystem and the extracted
renderer change the system's shape, and wink stamped `0.26.0-0` for this opening after 0.25.3
and 0.25.4 were spent on the messaging fix and the port's pre-cycle state.

**The settle fix is its own rung, and first** (wink, 2026-08-15). It was born as a bullet on
the qualify rung during a qualify-environment review, and moved out because the diff lands in
`gauge.rs` / `harness.rs` and improves every report, not just qualify's, so burying it would
make that rung's diff disagree with its title. First among the Work rungs because it is ready
today and because order protects the evidence: records and the three-box rerun archive whatever
the settle cell says.

**The config adopts vc-x1's markdown carrier** (wink, 2026-08-15): a `.md` whose `toml` fences
concatenate into the parsed config, chosen for being self-documenting with links. Its own rung
rather than a fold into the frequency rung, so the format change and the frequency feature
review separately. Free of migration, no config file existing anywhere yet. Both types stay
supported with a one-type-per-directory hard error (wink), and `.md` is recommended at the user
level too, diverging from vc-x1, whose user config stays TOML only because their tool
machine-writes it.

**The harness gains the frequency mutation** (wink, 2026-08-15), reversing the original rung
spec's diagnosis-never-mutation rule. The reasons the rule existed (root, global, persistent)
become the design constraints instead: pinning happens only on an explicit command or flag,
every catchable exit path restores, and restore converges to a steady state the user declared
in config rather than to a remembered one. `qualify-environment` stays read-only, the mutation
living in its own commands. A first draft saved the displaced state to a transient file and
restored that. Rejected (wink): transient files are a demonstrated failure mode here, and
displaced-state restore ratchets on back-to-back runs, worst on a battery-powered device.

**The old branch still holds one unlanded records commit** (musl/libc build-matrix design, a
Todo entry plus a chores-06 design subsection). Deliberately not folded into this cycle, being a
different topic. It stays on the old branch until re-homed, and the branch is not deleted.

#### Ladder details

##### feat: measure reproducibility opening

Land the stalled branch's finished work on `main`'s line as one commit: the policy and
clock-quantum report rows (the original's first Work rung), the renderer extraction into
`src/report.rs` (its second), and this cycle's records. The extraction went before the record
rung on purpose, so that rung's diff reads as "add the record" rather than "move 350 lines and
add the record", and the record's second consumer is what makes the model/renderer split honest.

Landed carrying more than the port: the laddering session (2026-08-15) rewrote the problem
statement in plain-programmer terms, grew the ladder by three rungs the original did not have
(the settle clock gate, the markdown config carrier, the frequency commands), moved wink's port
narrative in from chores-07, and swept nine Done entries into done.md at the opening beat.

Gotcha the pre-push validation caught: the `*.rs`-only copy carried the acceptance test's new
doc ("behind the `acceptance` feature") but not the `Cargo.toml` half that enforces it (the
`acceptance` feature and the `[[test]] required-features` gate), so the machine-graded test
silently rejoined plain `cargo test` and failed on a busy box. Both pieces restored from the
branch's `Cargo.toml` by hand, the copy having excluded that file for its version stamp.

##### fix: the settle cell reads the clock

The settle cell lies on exactly the box that needs it (wink, 2026-08-15, rated during a
qualify-environment review). `gauge::settle` reports the earliest warmup suffix that grades A
on timing alone, and a box parked flat on the un-boosted base clock grades A immediately, so
the 3900x under `powersave` reads `0.01s` and then transitions mid-bench to an F. Both cells
are truthful about what they measure, and together they mislead.

- the warmup exit already knows better: `classify_warm` gates a timing-A window behind
  `clock_stable` ("steady is not settled when the box is mid-climb", the measured 7600x dwell).
  `settle` takes the same clock series and the same gate, so a suffix counts as settled only
  when the clock held still across it. The series is already collected, so nothing new is
  measured
- name the state it settled into, not just when: `0.81s @ 4.35 GHz` (or "settled at base
  clock") is diagnostic where a bare `0.01s` misleads by omission
- `qualify-environment`'s `parse_settle` reads the cell text, so the parser updates with the
  format in this same commit
- the cell stays read-but-not-scored (test-enforced), which is what kept the defect from ever
  contaminating a verdict
- first among the Work rungs because it is ready (the clock series exists today, nothing here
  depends on the config, frequency, or policy rungs) and because order protects the evidence:
  landing before the record rung means no archived record ever carries the misleading time, and
  landing before the three-box rerun means the rerun's settle cells are honest

##### feat: write a per-run JSON record

The report prints and is gone. This rung adds `--record <path>`, a side channel that appends one
NDJSON object per bench result alongside the unchanged display. Design decisions taken at the
original pickup (2026-08-04):

- the record is per bench *result*, not per process: `all` emits one record per bench sharing
  the host / policy / clock stamp
- the display is never traded for the file: recording is a side channel, not a mode
- NDJSON, one object per line: `jq -s .` makes an array on demand, an interrupted run still
  parses, and per-run files concatenate with `cat`
- the tool names the file when handed a directory, because a fixed name is exactly what killed
  the powersave series (`run.sh` reused `u1.txt`..`p8.txt` and the rerun clobbered them)
  - `--record <dir>/` writes one file per run, stamped `<ts>-<host>-<bench>.ndjson`, and
    `--record <file>` appends to that file. The path's shape picks the mode
  - the open is `O_APPEND | O_CREAT`, never `O_TRUNC`, in both modes. The no-truncate invariant
    is what actually protects the evidence, whoever chose the name
  - basic ISO to the second in the filename (`20260804T093221Z`: no colons, lexicographic order
    is chronological order), RFC3339 with millis inside the record, plus the local offset as its
    own field. Millis are not decoration: `all` emits several records inside one second, so each
    also carries a process id and an index within the process
- `--tag k=v` is recorded verbatim and never interpreted, so a driving script labels condition /
  policy / box without the tool needing a notion of "series"
  - a per-run stamp is not a per-series stamp: only the caller knows which runs form one
    experiment, so `--tag series=<ts>` carries that, in every record of the series
  - no tag key is ever substituted into a path. That is where a template language starts
  - it lands with the record: a field of the same struct sharing all of its plumbing, so a rung
    of its own would review nothing
- the record carries a **fixed quantile ladder** (0.01 / 0.1 / 1 / 5 / 10 / 25 / 50 / 75 / 90 /
  95 / 99 / 99.9 / 99.99), not the report's populated bands, whose labels move with the data. It
  is what makes the A/B estimator question answerable after the fact, cheap now and impossible
  to backfill
- the record carries the **series of block means**, not just their summary: an aggregate cannot
  be decomposed, and the series is what lets within-run scatter be compared against across-run
  scatter. Bounded by a cap: keep every block mean while blocks <= 1000, summarize beyond
- the record documents its own fields, and a test enforces it
  - `describe-record`, a command word beside `all` / `qualify-environment`, prints the field
    dictionary: name, unit, one-line meaning. Not `--help`, which documents *inputs*
  - one source of truth is a const descriptor table, kept honest by a test that serializes a
    sample record, walks its keys, and fails on any key with no entry
  - every record carries `schema_version`, so a dictionary printed by today's binary can be
    checked against a record written by an older one
  - later polish, not this rung: per-field lookup (`explain frame_ns`) and generating README
    text from the same table
- absent is not zero: rpi5-20cd has no EPP and no `cpuinfo_avg_freq`, so every policy field is
  an `Option` recorded as absent
  - three states, not two: absent, present and uniform, present and split across policy groups.
    `freq::PolicyField` carries the token plus a `uniform` flag so the display can say
    `(mixed across CPUs)` rather than let one CPU stand in for the box
- ordering is part of the record: the 2026-08-03 pinned series was bimodal by *position*
  (p3-p6), which is invisible without a per-run wall-clock start
- comparison across the three boxes is within-box only: each box's pinned-vs-unpinned delta and
  its run-to-run scatter, never nanoseconds against nanoseconds
- the policy fields include the clamp and boost state (governor, EPP, `scaling_min_freq` /
  `scaling_max_freq`, boost), so a pinned run (next rung) is verifiable from its record rather
  than from trust

##### feat: adopt the markdown config carrier

The config becomes a `.md` file read the way vc-x1 reads `vc-config.md` and `.vc-config.md`:
the `toml` fences, concatenated in document order, are the TOML that gets parsed, and the prose
between them never reaches a parser. The prose is the format's point (wink, 2026-08-15): the
config documents itself, with markdown links, one `##` section per key doubling as that key's
anchor.

- the loader keeps its layering (built-in < XDG < project-local) and gains a fence filter ahead
  of the existing TOML parse, the shape of vc-x1's `src/md_fence.rs`
- **both types are read, one type per directory** (wink, 2026-08-15): each layer accepts
  `config.md` or `config.toml` (project-local: `iiac-perf.md` or `iiac-perf.toml`), the fence
  filter running only for `.md`. Finding both in one directory is a hard error naming both
  paths, never a silent precedence: the user editing the ignored file would get no effect and
  no clue. The rule is per directory, so mixing types across layers is fine
- `.md` is the recommended form at both levels, user config included, and nothing forces the
  global to be TOML here: vc-x1 kept their user config TOML because their tool machine-writes
  it, and nothing in iiac-perf writes its own config
- no migration exists to do: `src/config.rs`'s loader exists but no config file does, on this
  box or in the repo
- the freq rung's `[freq]` steady-state section is the first real occupant, and its section's
  prose is where "what is a steady state and why declare one" gets documented for the user

##### feat: read, pin, and restore the CPU frequency

For comparison the need is consistency, not speed (wink, 2026-08-15): a stable clock at any rate
beats an unstable fast one, so the harness gains the ability to hold the clock still and the
user gains commands to check, set, and unset that state at any time.

- three command words: `read-freq` prints the clock state (governor, EPP, min/max, boost,
  current frequency, per policy group), `pin-freq` holds the clock at one frequency (min = max,
  boost off), and `restore-freq` converges the box to the user's declared steady state
- **the default pin target is load-independent** (wink, 2026-08-15): with no frequency
  configured or given, `pin-freq` pins at base clock, the manufacturer's guaranteed frequency
  under sustained all-core load, so the unconfigured pin works for every bench as it stands
  today. Any faster "best" is workload- and schedule-dependent (thermal), and picking one is a
  measurement, not a default
  - discovery order: `acpi_cppc/nominal_freq` (reads 3801 MHz on the 3900x, the 3.8 GHz base
    clock matching the measured 3.7929 GHz TSC rate), then intel_pstate's `base_frequency`,
    else the highest non-boost frequency the driver lists. We think that last fallback covers
    the Pi, to be verified at implementation. `read-freq` prints what it resolved and from
    where
- a declared or given pin value is validated against the box at pin time, never in the
  abstract: the valid range is sysfs's per-policy `cpuinfo_min_freq`..`cpuinfo_max_freq`
  (discrete `scaling_available_frequencies` on drivers that list them), a bad value errors
  naming that range, and `read-freq` prints the range so the user picks informed
- **the restore target is declared, not remembered** (wink, 2026-08-15): the user's preferred
  steady state lives in the layered config's markdown carrier (a `[freq]` section, normally in
  the XDG file `~/.config/iiac-perf/config.md`, since the steady state is the box's, not the
  project's), written once by the user, beside prose saying what it means
  - why not save-and-restore-what-we-found: transient state files are a demonstrated failure
    mode in this repo, and restoring the displaced state ratchets on back-to-back runs. Run 2
    enters while run 1's pin is live, "restores" the pin, and a laptop or a phone is left
    pinned high. Convergence to a declared state is idempotent from any starting point
  - with no declared steady state, `pin-freq` refuses and says what to add, rather than pinning
    with no way home. `read-freq` can print the current state in config form, ready to paste
- `read-freq` needs no root and is shaped for a prompt or a status bar (wink's use case: a
  terminal prompt, a GUI panel beside the time-of-day): one short line, fast, stable format
- pinning writes sysfs and needs root, and the commands say so plainly when they lack it rather
  than half-working
- a run can pin at start and restore at exit, catching every exit path we can: normal return,
  panic, SIGINT/SIGTERM. The uncatchable paths (SIGKILL, power loss) need no saved state to
  recover from: `restore-freq` converges from anywhere, any time
- pin at or below base clock is the stable configuration: boost has nowhere to go and thermal
  throttling is unlikely to reach below base
- the record (previous rung) captures the clamp and boost state either way, so a pinned run is
  verifiable from its record

##### feat: qualify-environment reads the power policy

`qualify-environment` reads the policy as a fitness precondition and says so before spending
minutes on numbers it can predict will scatter. A diagnosis only: the mutation lives in the
frequency rung's explicit commands, and this command never changes the box. The caution is
earned: the 2026-08-03 session's documented revert left EPP at `performance` after the governor
had already returned to `powersave`, which is also why restore converges to a declared steady
state instead of trusting anyone's memory of what was displaced. The settle cell it parses is
already honest by this point, corrected with its parser by the settle rung above.

##### fix: LSC gains a run-to-run component

LSC is scoped to within-run block agreement but reads as a run-to-run bound. The best
configuration printed LSC 0.022 ns against a measured run-to-run stdev of 0.057 ns, so
single-run against single-run resolution is ~0.157 ns (0.71%) where the report prints 0.10%:
about 7x optimistic, and it did not improve when the environment did. Given the A/B purpose,
this is the cycle's most valuable rung.

- the fix is a **variance-versus-aggregation curve**, not a second measurement: a single run
  cannot measure run-to-run scatter directly. Aggregate its blocks in groups of 1, 2, 4, 8, ...
  and watch whether variance falls as `1/n`. Where it stops falling is the drift floor, and that
  floor is the run's honest resolution
  - this is Allan deviation (IEEE Std 1139), the standard tool in clock metrology for "how long
    should I average". The harness is already a clock project, so the machinery is familiar
  - the alternative, spawning children as `qualify-environment` does, is a much larger change
    and still cannot re-roll thermal or P-state history, so it does not reach the missing
    component
  - the duration estimate falls out of the same curve for free:
    `t_needed = t_now * (SE_now / SE_target)^2`, valid only where the `1/n` scaling still
    holds, which the curve is what tells us
- naming it accurately matters as much as computing it: a within-run bound must not print in a
  way that reads as a run-to-run one, which is the whole defect

##### feat: measure reproducibility closing

Closing out the cycle.

#### What the harness is for

**A/B comparison of algorithm changes**: did this change make it faster or slower. Stated here
because it decides calls like the rung specs above and is written down nowhere in the repo
(wink, 2026-08-04). It deserves a permanent home in the README's overview, and the cycle block
is where it landed first.

- the far tail is a validity check, not the score: beyond ~n4 on a 20 ns operation it measures
  OS interruptions (context switches, IRQs, faults, migrations), so it says whether to trust a
  comparison, never whether the algorithm improved
- the near tail is not in that bucket: cache miss rates, a ring's wrap boundary, and the 2t
  benches' producer/consumer phase relationships live around p99, and a change that improves the
  median while doubling p99 is usually a worse algorithm
- a comparison needs a resolution claim, not just a number, which is what makes the LSC rung
  load-bearing rather than last: an A/B verdict is only as good as the smallest delta the tool
  can honestly distinguish

#### The 2026-08-03 pinning experiment

The measurement that motivates the cycle (measured 2026-08-03, the session's terminal stamps
read 26-08-04 UTC):

- the measurement: `min-now --blocks 200`, 8 pinned + 8 unpinned per policy, alternating so box
  history is shared between conditions, under amd-pstate-epp `powersave` then `performance`
  - powersave unpinned: 23.927 ns, run-to-run stdev 0.785 ns (3.28%), grades 4 A / 2 D / 2 F
  - powersave pinned to one core: 22.207 ns, 0.073 ns (0.33%), all 8 graded D
  - performance unpinned: 21.980 ns, 0.057 ns (0.26%), no D or F in 17 runs
  - performance pinned: 22.359 ns, 0.311 ns (1.39%)
- what that supports, and how strongly:
  - the policy delta is big and one-directional: 8.9% on the unpinned mean, with unpinned
    run-to-run scatter falling from 0.785 ns to 0.057 ns
  - but policy could not be alternated the way pinning was (setting EPP needs root and is global
    and persistent), so the two policies are separated in time and the delta is confounded with
    box history. We think an effect this size survives de-confounding, but this experiment does
    not show it
  - pinning under powersave is the strongest result here: 7.2% off the mean and 10x off the
    scatter, on alternated runs
  - pinning under performance is the weakest: 1.7% *worse* than unpinned, and the whole gap is
    four consecutive pinned runs (p3-p6) at 22.59-22.70 ns while the pinned runs on either side
    sat at 22.08-22.09 ns and the interleaved unpinned runs did not move. So the 0.311 ns is the
    width of a state change, not scatter about a mean, and "pinning loses under performance"
    rests on that one cluster and should be treated as unreproduced
- the finding that reframes the grade: A was being awarded to the un-boosted floor. A powersave
  A run puts ~73% of its samples on 23.967 ns, the 3.7929 GHz TSC/base rate, while the boosted
  states are 21.79 / 22.22. Flatness is all the grade sees, and base clock is the flattest place
  on the box
- evidence is perishable: `tmp/pinexp/` held only the performance series (the rerun overwrote
  powersave), so the powersave numbers live only in the session transcript until copied into a
  chores design subsection
- cross-cuts the Todo entries below: the interpretation guide's worked trio and the blocks
  entry's duty-cycle evidence were both collected without recording the policy, and a four-run
  `--blocks` sweep proved confounded (block count and box history rose together, history
  dominating), so re-check those examples before they ship as teaching material
- raises "Seam-clock attribution": we think the three states map onto ~3.79 / 4.17 / 4.35 GHz,
  inferred from timing ratios alone, and a seam clock sample would settle it

#### The clock quantum and the dither

The clock's quantum is a property of the box, not the bench (measured on rpi5-20cd, 2026-08-04).
`inner` is sized for framing domination, `inner = ceil(10 * frame_ns / step_cost_ns)`, so in
`q = tick_ns / inner` the step cost cancels and `q / step = tick_ns / (10 * frame_ns)`.

- ~2.2% on the Pi (18.5185 ns tick, ~82 ns frame) against ~0.05% on the 3900X (0.264 ns tick,
  ~50 ns frame): a factor of 40, applying uniformly to every bench either box runs
- three benches confirm it, predicted `q` against the spacing between value clusters in the band
  table: `min-now` inner 23, q 0.805, seen 0.800 / 0.832 / 0.768. `zcr-with-1t` inner 47,
  q 0.394, seen 0.400 / 0.384 / 0.408. `zcr-with-2t` inner 4, q 4.630, seen 4.352 / 4.608 /
  4.864. The residual wobble is hdrhistogram bucketing (0.016 ns at 17 ns, 0.256 ns at 270 ns)
- so a printed per-sample spread below ~q describes the clock, not the workload: `min-now`'s
  `stdev p20..p80 0.403` is half a quantum and `zcr-with-1t`'s 0.213 is a two-point split, while
  `zcr-with-2t` spans ~11 quanta and its 4.906 is real to within 4% (`q/sqrt(12)` removed in
  quadrature)
- the means, the grades and LSC are unaffected: all work from batch or block means over ~1.2M
  samples, where quantization averages away as `1/sqrt(N)`
- so print and record it, do not size for it: `q` joined the `Setup:` block in the ported work,
  and `ticks_per_ns` and `inner` are record fields so it stays derivable per result
- a granularity floor on `inner` is the wrong fix and is its own Todo entry, not a rung here.
  The dither makes quantization zero-mean, so a 5M-sample mean carries `q/sqrt(12N)` = 0.0001 ns
  of it, while raising `inner` would smear the tail linearly and cannot reveal per-call shape
  below one tick either way
- the dither works on the coarse lattice, twice confirmed on the Pi: `zcr-with-1t`'s mass sits
  on two adjacent lattice points (2.08M at ~16.951, 2.94M at ~17.335) and interpolates to 17.18
  against a printed `mean z4..n2 17.183`, with the two-point stdev prediction 0.19 against a
  printed 0.213
  - so an off-lattice value is recovered rather than snapped, which is what `DITHER_SPAN` exists
    to do. The earlier worry that ~26-32 ns of span against an 18.5 ns quantum would bias it is
    not visible in the data
  - the decisive check remains cheap and unrun: sweep `-i` (23 / 97 / 233) on the Pi and confirm
    the mean does not move as `q` shrinks

#### Port measure reproducibility

I worked with[claude-web](https://claude.ai/share/d8aaa0dd-026a-4973-9906-2b83508ef89b) and
ported the branch measure-reproducibility that claude-code and I worked on 2026-08-04 and
2026-08-03. After that we got side tracked working on agent-files with vc-x1 and it's time to
revive it.

Initially I tried just rebase it:
```
wink@3900x 26-08-15T14:46:49.277Z:~/data/prgs/rust/iiac-perf (main+1)
$ jj rebase -b measure-reproducibility -d @-
Rebased 4 commits to destination
New conflicts appeared in 4 commits:
  sqwrtoyv 2ea22f58 (conflict) (no description set)
  quxttlto 75d0cedf measure-reproducibility* | (conflict) refactor: extract the report renderer
  pvpszqlm e3f3d2d2 (conflict) feat: report the power policy and clock quantum
  zkxrnrrn f282ecea (conflict) feat: measure reproducibility opening
Hint: To resolve the conflicts, start by creating a commit on top of
the first conflicted commit:
  jj new zkxrnrrn
Then use `jj resolve`, or edit the conflict markers in the file directly.
Once the conflicts are resolved, you can inspect the result with `jj diff`.
Then run `jj squash` to move the resolution into the conflicted commit.
wink@3900x 26-08-15T14:51:39.958Z:~/data/prgs/rust/iiac-perf (main+1)
```

But here were too many conflicts so I restored it:
```
wink@3900x 26-08-15T14:51:39.958Z:~/data/prgs/rust/iiac-perf (main+1)
$ jj op restore 75ddc1
Restored to operation: 75ddc14fde07 (2026-08-14 17:30:39) push bookmarks docs-converge-the-agent-files-with-vc-x1, main to git remote origin
Working copy  (@) now at: qpnxsmzl dec5e8bc (empty) (no description set)
Parent commit (@-)      : pnowzzty 0520c17c main | docs: converge the agent-files with vc-x1
Added 0 files, modified 4 files, removed 0 files
Existing conflicts were resolved or abandoned from 5 commits.
wink@3900x 26-08-15T15:09:52.393Z:~/data/prgs/rust/iiac-perf (main+1)
$ jj st
The working copy has no changes.
Working copy  (@) : qpnxsmzl dec5e8bc (empty) (no description set)
Parent commit (@-): pnowzzty 0520c17c main | docs: converge the agent-files with vc-x1
wink@3900x 26-08-15T15:10:37.751Z:~/data/prgs/rust/iiac-perf (main+1)
```

After talking with claude-web the techquite that worked was to just copy over the *.rs files which
we haven't touched so just coping those created something that passed `cargo check`
```
wink@3900x 26-08-15T16:16:56.298Z:~/data/prgs/rust/iiac-perf (main+1)
$ jj bookmark create port-measure-reproducibility -r @
Warning: Target revision is empty.
Created 1 bookmarks pointing to qpnxsmzl dec5e8bc port-measure-reproducibility | (empty) (no description set)
wink@3900x 26-08-15T16:22:21.933Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ jj st
The working copy has no changes.
Working copy  (@) : qpnxsmzl dec5e8bc port-measure-reproducibility | (empty) (no description set)
Parent commit (@-): pnowzzty 0520c17c main | docs: converge the agent-files with vc-x1
wink@3900x 26-08-15T16:22:25.717Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ jj restore --from measure-reproducibility 'glob:**/*rs'
Working copy  (@) now at: qpnxsmzl 870001c9 port-measure-reproducibility | (no description set)
Parent commit (@-)      : pnowzzty 0520c17c main | docs: converge the agent-files with vc-x1
Added 1 files, modified 23 files, removed 0 files
wink@3900x 26-08-15T16:22:52.711Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ jj st
Working copy changes:
M src/band_table.rs
M src/bands.rs
M src/benches/ice_ps_1t.rs
M src/benches/ice_ps_2t.rs
M src/benches/ice_rr_1t.rs
M src/benches/ice_rr_2t.rs
M src/benches/min_now.rs
M src/benches/mpsc_1t.rs
M src/benches/mpsc_2t.rs
M src/benches/mpsc_2t_spin.rs
M src/benches/probe_mpsc_2t.rs
M src/benches/std_now.rs
M src/benches/tp2_pc.rs
M src/benches/zcr_mpsc_1t.rs
M src/benches/zcr_mpsc_2t.rs
M src/benches/zcr_with_1t.rs
M src/benches/zcr_with_2t.rs
M src/freq.rs
M src/gauge.rs
M src/harness.rs
M src/main.rs
M src/probe.rs
A src/report.rs
M tests/qualify_environment.rs
Working copy  (@) : qpnxsmzl 870001c9 port-measure-reproducibility | (no description set)
Parent commit (@-): pnowzzty 0520c17c main | docs: converge the agent-files with vc-x1
wink@3900x 26-08-15T16:23:16.995Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ cargo check
    Checking iiac-perf v0.25.2 (/home/wink/data/prgs/rust/iiac-perf)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.15s
wink@3900x 26-08-15T16:23:34.167Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
```

claude-web recommened running `cargo check --tests` and then `cargo test` and they both worked,
although that `tests/qualify_environment.rs` to 26s and I thought it'd died, but I was patent
and it did complete:
```
wink@3900x 26-08-15T16:23:34.167Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ cargo check --tests
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s
wink@3900x 26-08-15T16:54:07.361Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ cargo test
   Compiling iiac-perf v0.25.2 (/home/wink/data/prgs/rust/iiac-perf)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.20s
     Running unittests src/main.rs (target/debug/deps/iiac_perf-4f8dfa43303d64f4)

running 79 tests
test bands::tests::generated_boundaries_match_documented_ladder ... ok
test config::tests::empty_is_all_none ... ok
test config::tests::bad_band_labels_errs ... ok
..
test harness::tests::batch_pipeline_flushes_full_batches ... ok
test harness::tests::settle_time_finds_the_ramp_end ... ok

test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/qualify_environment.rs (target/debug/deps/qualify_environment-39de42f698c35b50)

running 1 test

test environment_qualifies ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.37s

wink@3900x 26-08-15T16:54:39.693Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
```

I then ran qualify-environment:
```
wink@3900x 26-08-15T17:01:08.097Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ iiac-perf qualify-environment
iiac-perf 0.25.4 — Rust latency microbenchmark harness

qualify-environment: 10 runs of `min-now -d 1`, gap 0s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle   mean
  1     A       F        F       0.01s    22.2 ns
  2     A       F        F       1.38s    22.7 ns
  3     A       B        B       not      21.6 ns
  4     A       A        A       0.95s    21.5 ns
  5     A       A        A       0.01s    24.5 ns
  6     A       A        A       1.26s    21.6 ns
  7     A       C        C       1.97s    22.1 ns
  8     A       C        C       1.62s    24.4 ns
  9     A       A        A       not      21.6 ns
  10    A       F        F       1.37s    24.5 ns

  environment grades: 4A 1B 2C 3F
  median environment grade: B
  median settle: 1.37s (2 of 10 never settled)
  transition-degraded (drift or step at D/F): 3 of 10

  verdict: NOT QUALIFIED
    a state transition landed inside a measurement window
wink@3900x 26-08-15T17:01:49.854Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
```

Look at the original [measure-reproducibility](https://github.com/winksaville/iiac-perf/tree/measure-reproducibility)
branch for more background information, we'll start a new ladder after this commit and I'll have
claude-code create a new ladder based on the original ladder.

Here are some runs of qualify-environment on 7600x a newer Ryzen 5
and 3900x an older Ryzen 9. These are 5 second duration with a 5 seconds
between runs so we measure "startup" ramping each time.

You can see 7600x is has perfect scores each time and a consistent
0.81s settling time:
```
wink@7600x 26-08-15T17:33:08.762Z:~
$ rg -m 1 'model name' /proc/cpuinfo 
5:model name	: AMD Ryzen 5 7600X 6-Core Processor
wink@7600x 26-08-15T17:33:17.592Z:~
$ ./iiac-perf qualify-environment -d 5 --runs 5 --gap 5
iiac-perf 0.25.4 — Rust latency microbenchmark harness

qualify-environment: 5 runs of `min-now -d 5`, gap 5s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle   mean
  1     A       A        A       0.81s    16.2 ns
  2     A       A        A       0.81s    16.2 ns
  3     A       A        A       0.81s    16.2 ns
  4     A       A        A       0.81s    16.2 ns
  5     A       A        A       0.81s    16.2 ns

  environment grades: 5A
  median environment grade: A
  median settle: 0.81s (0 of 5 never settled)
  transition-degraded (drift or step at D/F): 0 of 5

  verdict: QUALIFIED
wink@7600x 26-08-15T17:35:00.407Z:~
```

Where as the code thinks the 3900x settles in 10ms, **which is probably does not**, 
and has terribe numbers:
```
wink@3900x 26-08-15T17:41:28.871Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ rg -m 1 'model name' /proc/cpuinfo
5:model name	: AMD Ryzen 9 3900X 12-Core Processor
wink@3900x 26-08-15T17:41:50.432Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ iiac-perf qualify-environment -d 5 --runs 5 --gap 5
iiac-perf 0.25.4 — Rust latency microbenchmark harness

qualify-environment: 5 runs of `min-now -d 5`, gap 5s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle   mean
  1     A       F        F       0.01s    21.7 ns
  2     A       F        F       0.01s    21.8 ns
  3     A       F        F       0.01s    21.7 ns
  4     A       F        F       0.01s    22.7 ns
  5     A       D        D       0.01s    21.7 ns

  environment grades: 1D 4F
  median environment grade: F
  median settle: 0.01s (0 of 5 never settled)
  transition-degraded (drift or step at D/F): 5 of 5

  verdict: NOT QUALIFIED
    median grade below B — runs are measuring a moving box
    a state transition landed inside a measurement window
wink@3900x 26-08-15T17:43:10.276Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
```

Although both are running the same arch linux version and only the terminal
open, no other apps (no bot either). But the 3900x has a GUI running, thus mouse and a display
running. Where as 7900x is just a server no mouse or display attached and no gui is installed
so just running via an SSH connection over the LAN. So we don't know exactly why but 7600x is a
much better machine for getting consistent numbers, at this point in time.

## Todo

Entries are in **strict priority rank**, #1 highest, descending. Reprioritize by moving an
entry, then `vc-x1 fix-todo --no-dry-run TODO.md` to renumber. The numbers are positional rank,
not stable IDs. To refer to a Todo, name it by its **title** (a greppable mention; a numbered
list item has no anchor to link to), not its number. Long-tail entries live in
[todo-backlog.md](notes/todo-backlog.md). Use the
[Prose form](agent-data/prose.md#prose-form); deeper detail goes in
`notes/chores/chores-NN.md` design subsections (link via `[N]` ref).

1. Prepare for expected errors: before a command whose outcome is not clean (a rebase across
   diverged lines, a force-push, dogfooding a dev tool), state the expected output, what
   unexpected would look like, and the abort path, then fix stepwise with the user
   - the forward-looking half of hard rule 10's stop-and-ask, and family-shaped, so it belongs
     in pinned AGENTS.md working practices via its own convention cycle
   - born 2026-08-14: a rebase's predicted conflicts arrived unannounced and read as breakage
     (wink stopped the session), the prediction living in a record instead of in the moment
2. Report interpretation guide: a reader-oriented "how to read a report" walkthrough in README,
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
5. Sync the 20260803 agent-files baseline [[84]]
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
6. Qualification reports evidence, not verdicts: retire the prejudging NOT QUALIFIED stamp
   (wink, 2026-08-02) in favor of measured statements a reader judges
   - blocks-based: A/A repeatability (does a same-code delta clear LSC?), CI95/LSC as the
     published sensitivity ("this box resolves X ns on this bench"), stratification by state
     instead of a blended letter
   - the 3900X reads NOT QUALIFIED today for mid-run bistable transitions warmup cannot
     prevent: a trait to report, not a disqualification; the 7600x dwell case that motivated
     the gate is fixed by the dynamic-warmup cycle
   - entangled with "Qualify the environment without a bench" (below) and machine-readable
     output; wants the blocks-knobs entry (above) landed first
7. Seam-clock attribution: sample `cpuinfo_avg_freq` at batch seams (the reader exists,
   `src/freq.rs`) so a mid-run step gets a "clock moved" label, the way warmup now separates a
   dwell from the top; also the natural home for surfacing the clock ratio in normal output as
   one coherent story (chores-06: the 3900X flip at ~2-4 s is almost certainly a visible clock
   move)
8. Qualify the environment without a bench. `qualify-environment` respawns children running
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
9. Guard `--pin` pools smaller than the bench's thread placements, and deadline the estimate
   phase: `zcr-mpsc-2t --pin 8` put both spinning software threads on one logical CPU and
   appeared hung until ^C (2026-07-26, bug #1 in [bugs.md](notes/bugs.md#bugs))
   - track `core_for` requests in `RunCfg` (max `thread_idx` asked for); refuse the run when
     placements exceed unique CPUs in the pool. Placement only goes through `core_for` when
     pinning is active, so the guard covers every path, and no pinning means the scheduler
     separates the spinners itself
   - wall-clock deadline on the open-loop 5x1,000-step estimate phase so *any* pathologically
     slow bench aborts with a diagnostic naming per-step cost and pinning, instead of hanging
10. Move the batch seam's work off the measuring thread, using the FastForward-style SPSC ring.
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
11. Tighten thread/CPU terminology across docs and doc comments: "software thread" for what
    `thread::spawn` makes, "logical CPU" (hardware thread) for what `--pin` selects and the OS
    schedules onto, "physical core" for the engine SMT siblings share. Bare "core"/"CPU"/"thread"
    only where context disambiguates
    - spin-wait bench docs state the precondition: each spinning software thread needs its own
      logical CPU
    - `--pin` help/README say slots are logical CPU ids
12. Topology-aware pinning and lCPU terminology: discover the CPU sharing tree at runtime and
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
13. Rebase `web-claude-tweaks` onto post-0.22.0 `main`. It rewrites an already-published
    bookmark (needs approval) and its arbitrary `0.21.0-b` version needs replacing; owed from
    the 0.22.0 close-out plan
14. Unit scaling in report columns (`us`/`ms`): per-row auto-scale so columns stay
    eyeball-comparable (bands are monotonic, so a row's first/last/mean share a magnitude), or
    `--units ns|auto` for script-stable output; needs `--decimals` landed first (`3.18 ms` vs
    `3 ms`); candidate `-4` for the report-options cycle.
15. Machine-readable report output (`--format json`, or key=value lines to stay
    dependency-light). Design once the batch gauge lands (0.23.0-4) so the schema covers the
    surviving surface: report stats, gauge signals, letter. Consumers:
    `tests/qualify_environment.rs` (drops its brittle-but-loud line parsing), placement-map
    validation runs, cross-run comparison scripts. Kin to the unit-scaling entry's `--units ns`
    script-stable concern (above), one flag family.
16. Trimmed core stats: `mean/stdev p10-p90` report row, additional to (never replacing)
    `mean` / `mean min-p99`; trim bounds possibly configurable (`--trim p10:p90`?). Why: the
    full mean wobbles ~±1.4% with the run's mode mix while the core plateau is ~±0.2% stable,
    so the trimmed row is the run-to-run comparable number. Boundary sensitivity (see [[57]]):
    window edges in the mode-mix smear inherit its wobble (p50-p60 ±0.05% vs p40-p50 ~1%), so
    also consider a dominant-*mode* statistic (peak-density region, bottom-count-independent)
    [[57]]
17. Find and label the interference crossover: the band where the tail stops measuring the code
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
18. Investigate: suspend gap missing from samples. A 0.13.5 `--no-inhibit` suspend test
    detected ~1.2 s suspended inside the measured window but the max sample was only 4.0 ms,
    while the 0.13.1 test (8.4 s gap) showed the expected 10.4 s max sample. We think
    minstant's TSC may halt across some suspends and count through others. Repeat the test
    comparing detected gap vs max sample; if the TSC halts, per-sample timing silently loses
    suspend time; document either way.
19. CLAUDE.md governance model (design cogitation) [20]
20. Revisit probe adjustment under the in-interval vs call-to-call split: probes take one call
    per sample (inner=1), so the in-interval timer slice is unamortized and unmeasurable, so an
    `adjusted` column can subtract nothing defensible; maybe state a bound instead
    [analysis](notes/design.md#timer-overhead-in-interval-vs-call-to-call)
21. Convert `harness` / `Bench` to probe-based measurement. Will likely need inner-loop support
    on `Probe` (batch N calls per sample; report divides by N and accounts for per-sample
    framing) so very-small workloads can still amortize timer overhead the way `run_adaptive`
    does today.
22. Rename app
23. Design an app to measure IIAC perforanace written in Rust[1]
24. `ice-ps-2t-wait`: iceoryx2 pub/sub with blocking waits via `Listener`/`Notifier` events;
    completes the {transport} × {wait policy} matrix cell that compares against `mpsc-2t`
25. Switch ice benches to the loan-based zero-copy send path (`loan_uninit` + `send`), the API
    a perf-sensitive user would use, and closer to iceoryx2's own benchmark method
26. Payload-size sweep for the round-trip benches (8 B / 8 KiB / 1 MiB), makes iceoryx2's
    size-independent latency vs channel copy cost visible in our own tables
27. `crossbeam-1t` / `crossbeam-2t`: `crossbeam-channel` directly (compare to mpsc-1t/2t which
    use crossbeam under the std API)
28. `tokio-mpsc-1t` / `tokio-mpsc-2t`: `tokio::sync::mpsc` round-trip inside a Tokio runtime
    (async overhead)
29. `flume-1t` / `flume-2t`: `flume` MPMC channel
30. Function-call baselines: direct call vs `Box<dyn Trait>` vs `async fn` (poll-once): anchors
    the channel/serde numbers against the cheapest possible "send a value then receive it" path
31. When the second channel impl lands, extract shared message types + round-trip helpers into
    `src/benches/common.rs` (deferred from 0.2.0)
32. Additional thread control (count, per-thread pin lists, NUMA): shape once a concrete bench
    needs it
33. Rename crate `iiac-perf` -> general-purpose name (breaking; deferred)
34. `suggest-freq`: measure the best pin frequency instead of defaulting to base clock (raised
    2026-08-15 during the measure-reproducibility laddering, ranked last on arrival)
    - "best" is the highest frequency the box *holds* under the intended workload and schedule,
      so it is thermal and duty-cycle dependent: a short run passes at a frequency a long run
      throttles from, and our 2026-08-02 data showed the schedule selecting the state
    - shape: descend from max-with-boost-off, pin each candidate, drive a bench-like load for at
      least the intended run duration, verify with `clock_stable` plus the grade, report the
      highest candidate that held, named with the schedule it was measured under
    - wants the record and LSC rungs landed first: the block-mean series and the drift floor are
      the evidence a suggestion is judged by
    - until then the load-independent default stands: pin at base clock

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

- 0.24.10 **docs: design the vc-x1-messages repo** [[93]]
- 0.25.0 **docs: semicolons leave the agent-files** [[94]]
  - prose.md's `Semicolons` rule goes flat: prose carries no semicolons, and a semicolon
    appears only in code, where it is syntax
  - the agent-files (custom* included) carry no historical exemption and swept to zero, ninety
    sites across eight files, verified by the blank-code-then-expect-zero grep
  - any other historical file keeps its semicolons until altered, and altering one means asking
    the user whether they should go
  - supersedes the between-equals allowance vc-x1 pinned, offered to them by message now the
    cycle has landed
- 0.25.1 **docs: always link the closing rung** [[95]]
  - a ladder's closing rung is linked like its siblings, and its subsection opens at laddering
    with a one-line stub, completing at close-out with gotchas or `_None._`
  - edits the three pinned statements (checklist opening and close-out, the protocol's
    closing-rung paragraph) plus notes.md's slot note, finishing what wink's template edit
    started
  - the semicolon cycle's as-built rungs backfilled on the landing's one-push-later timing
- 0.25.2 **docs: converge the agent-files with vc-x1** [[96]]
  - the formal review owed since 2026-08-08: every hunk of the eight-file diff verdicted, all
    of it our three proposals (validate every commit, the flat semicolon rule and its sweep,
    the always-linked closing rung), nothing of theirs untaken
  - their notes-entry question answered: entries stay ranked list items cited by bold title,
    and trackers stay reserved for notification
  - the 2026-08-12 findings homed in chores-07, the early entry delivered, the template
    mailbox swept and deleted
  - run single-step after the ladder collapsed, the records being the only remaining diff, and
    the review invitation goes via `vc-x1-messages` now that the cycle lands
  - a shared repo for family correspondence, because the transport was the defect rather than the
    messages riding it: mailboxes live in a repo whose `main` is a single initial commit
  - plain rather than dual, since a managed repo would inherit the rule that a repo with a live
    session is written only by its own agent, making the one repo everyone writes to writable by
    one member
  - bodies stay in the sender's repo and only pointers are shared, which is what lets each file's
    owner choose its persistence without endangering anything
  - `messages/test-msg.md` lands here as the specimen the README's examples point at, and its
    absence from an earlier commit is what taught the ordering rule
- 0.25.3 **docs: point messaging at the vc-x1-messages repo** [[97]]
  - `custom-family.md`'s Messaging section now names `../vc-x1-messages/iiac-perf.md` and that
    repo's README as the governing protocol, replacing the template mailboxes it still pointed at
  - handle-then-delete gives way to mark-never-delete: `read:` on reading, `outcome-*` to close,
    and the copy-into-chores-before-delete step retires, bodies being committed files in the
    sender's repo

# References

[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[75]: /notes/chores/chores-05.md#settle-time-is-not-a-grade
[84]: /notes/chores/chores-06.md#docs-experiment-in-the-local-agent-files
[93]: /notes/chores/chores-07.md#docs-design-the-vc-x1-messages-repo
[94]: /notes/chores/chores-07.md#docs-semicolons-leave-the-agent-files
[95]: /notes/chores/chores-07.md#docs-always-link-the-closing-rung
[96]: /notes/chores/chores-07.md#docs-converge-the-agent-files-with-vc-x1
[97]: /notes/chores/chores-07.md#docs-point-messaging-at-the-vc-x1-messages-repo
[98]: #feat-measure-reproducibility-opening
[99]: #feat-write-a-per-run-json-record
[100]: #feat-qualify-environment-reads-the-power-policy
[101]: #fix-lsc-gains-a-run-to-run-component
[102]: #feat-measure-reproducibility-closing
[103]: #port-measure-reproducibility
[104]: #feat-read-pin-and-restore-the-cpu-frequency
[105]: #feat-adopt-the-markdown-config-carrier
[106]: #fix-the-settle-cell-reads-the-clock
