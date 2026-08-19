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
- block sleep and warmup become explicit knobs defaulting to zero, so a run never naps or
  discards samples unasked
- an honest run-to-run resolution for LSC
- the pin flag adopts the kernel's CPU vocabulary: `--pin` becomes `--pin-cpus`, and code and
  README sweep to match
- `suggest-freq`: measure the pin frequency the box holds under the intended workload, ending
  in a paste-ready config line

The qualify-environment rung is halted unbuilt (wink, 2026-08-19): the environment rating gets
its redesign first, and the rung's spec waits in `## Todo` under "Rethink environment rating,
then resume the qualify power-policy rung".

The three-box rerun of the pinning experiment (3900X, 7600x, rpi5-20cd) follows the frequency
rung as evidence, and a second evidence stage, the reproducibility of zcr-mpsc-1t and
zcr-mpsc-2t at the suggested frequency across pin-CPU layouts, follows the suggest-freq rung.

#### Acceptance check

The measures:

- A run with `--record` leaves a record that can be re-analysed without its session: it carries
  `schema_version`, the fixed quantile ladder, the block-mean series, and the box's policy
  fields, and `describe-record` documents every field, enforced by a test that fails on any
  undocumented key.
- The settle cell agrees with the clock: a 3900x `powersave` run no longer reads `0.01s` beside
  an F, and the cell names the state it settled into.
- `--block-sleep` and `--block-warmup` default to zero, print in Setup whenever blocks run,
  ride the record, and the block replication rows (CI95 / LSC) print only when the sleep is
  nonzero.
- The report's resolution claim is the variance-curve drift floor, not the within-run LSC that
  read 7x optimistic against measured run-to-run scatter, and it prints on every run,
  `--blocks` or not.
- `pin-freq` then `restore-freq` leaves the box in the declared steady state (governor, EPP,
  min/max, boost), a pinned run restores on normal exit, panic, and SIGINT, and `restore-freq`
  converges from any starting point, an unclean death included.
- `suggest-freq <bench>` reports each candidate frequency, how long it held, under what load
  and schedule, and ends with a paste-ready `pin_mhz = ...` line.
- `--pin-cpus` is the pin flag's name everywhere a reader meets it (help, README, code names),
  the record field `pin_cpus` unchanged, and README states the CPU / core / SMT terminology.
- The port preserved behavior: full validation green on `main`'s line at every rung.

#### Ladder

- [[N]] [feat: measure reproducibility opening][98] (done)
- [[N]] [fix: the settle cell reads the clock][106] (done)
- [[N]] [feat: write a per-run JSON record][99] (done)
- [[N]] [feat: adopt the markdown config carrier][105] (done)
- [[N]] [feat: read, pin, and restore the CPU frequency][104] (done)
- [[N]] [fix: the settle cell shows the clock's journey][107] (done)
- [[N]] [feat: block sleep and warmup become knobs][110] (done)
- [[N]] [fix: LSC gains a run-to-run component][101]
- [[N]] [fix: the pin flag names CPUs][108]
- [[N]] [feat: suggest-freq measures the pin frequency][109]
- [[N]] [feat: measure reproducibility closing][102]

The three-box rerun sits anywhere after the frequency rung: it is evidence, recorded in the
chores section, not a rung, and everything it needs has landed, records and pinning both, so it
can add a pinned-clock condition to the original's four. Run it **with `--blocks`**, so every
record carries a block-mean series and the within-run against across-run decomposition comes
out of the same dataset. The zcr-mpsc campaign sits between the suggest-freq rung and the
closing the same way: zcr-mpsc-1t and zcr-mpsc-2t at the suggested frequency and across
pin-CPU layouts, every run recorded with `--record` and `--tag` naming series, bench, layout,
and condition.

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

**The settle-journey rung is a mid-cycle insertion** (wink, 2026-08-17). The landed settle cell
prints the measurement floor as a measurement: `0.01s @4.09GHz` on nearly every healthy run,
the time being the first probe's timestamp rather than anything measured. The replacement was
designed in conversation (the start->end journey, the transition time, a settled-state
steadiness rating, and a tick-graph profile line in `-v`), and the rung sits after the
frequency rung so the qualify rung's parser work sees the final format. Taken exception,
recorded here per the draft self-consistency rule: the four pushed rungs carry the
pre-insertion ladder and are not rewritten, a TODO-snapshot rewrite of published commits buying
nothing a reader needs. The two-regime work this conversation also raised (regime config key,
`suggest-freq`) stays out, homed as Todo ranks 2 and 3 for the next cycle.

**The qualify rung halts unbuilt** (wink, 2026-08-19). The settle-cell rework left the
environment rating "better but not good enough", and the rating redesign (Todo's "Rethink
environment rating") has to come before the qualify rung builds more on top of it. Nothing of
the rung is committed, so the halt is a ladder edit on the draft bookmark, the spec moving back
to `## Todo` under "Rethink environment rating, then resume the qualify power-policy rung",
and the pushed rungs keep their pre-edit TODO snapshots, the same taken exception as the
settle-journey insertion above.

**suggest-freq pulls in from Todo** (wink, 2026-08-19), reversing the settle-journey entry's
homing of it to the next cycle: the zcr-mpsc campaign needs the discovered rate, so the
guidance half is part of this cycle's deliverable. Its spec asked for the record and LSC rungs
first, the record has landed, and the LSC rung stays and goes ahead of it, so the dependency
holds. The two-regime config key stays out, still next-cycle.

**The pin flag renames to --pin-cpus** (wink, 2026-08-19), and the project adopts the kernel's
vocabulary: a CPU is the schedulable logical processor sysfs numbers, a core is the physical
core hosting two of them, and SMT siblings are the pair sharing a core. Plural cpus over the
first-draft `--pin-core` because the flag takes a list and the record field is already
`pin_cpus`. Its own rung ahead of suggest-freq, so the format change and the feature review
separately and the campaign's recorded command lines carry the final names.

**Block sleep and warmup become knobs, defaulting to zero** (wink, 2026-08-19). Designed in
conversation after wink's three 7600x runs (all grade A, means 16.2 to 16.5 ns with block
count the only knob turned, LSC printing 0.0 ns throughout) showed the hidden 1-10 ms sleep
and the unrecorded 2 ms warmup shaping results invisibly. Zero is the neutral setting so the
tool never naps or discards samples unasked, the cost taken knowingly: sleepless blocks are
partitions, not replicates, so the CI95 / LSC rows gate on a nonzero sleep. A mid-cycle
insertion like the settle-journey rung, placed ahead of the LSC rung on purpose: gap-separated
blocks are quasi-runs inside one invocation, the validation data the drift floor is checked
against.

**The Work-rung pushes ran on an explicit scoped delegation** (wink, 2026-08-19): rule 2's
per-push stops waived in conversation ("I'm giving you the permission now"), recorded here
as the taken exception the hard rules ask for. Flow was not waived, full validation running
before every push, and the close-out stays outside the scope, waiting on wink's review.

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

As built:

- `clock_stable` moved from the harness into `freq.rs` with its tolerance, one home for the one
  rule both gates now read, and the settle scan gates each candidate suffix on it beside the
  timing grade
- settle is scored in the harness, where the clock series is alive, and rides `RunOutput` as
  `warm_settle`: the report had been recomputing it from probes alone because the series never
  left `warmup_and_probe`, which is *why* the cell was timing-only
- the named state is the settled suffix's **median** GHz, so one odd read cannot name it, and a
  box with no readable driver keeps the bare time, never a guessed state
- `Settle::At` became a struct variant carrying the state, and qualify's parser learned the new
  cell in the same commit, its read-but-not-scored quarantine untouched
- seen live on the 3900x first run: warmup `0.01s @4.09GHz` beside a bench-row step at 1.90s,
  the cell now naming the boosted state warmup certified while the bench watched the box leave
  it
- `qualify-environment`'s own table carries the state too, added when wink's first 0.26.0-1
  runs showed bare times where he was looking: its settle column had been reformatting the
  parsed time alone. First table with it: `0.01s @4.47GHz` beside an F, a boost state the box
  could not hold, against two runs holding ~4.08 GHz

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
- **the clock rides the record too, sampled at batch seams** (wink, 2026-08-16): one delivered
  frequency read per seam (~50 ms apart, one sysfs read, the reader exists in `src/freq.rs`),
  so per-block and per-run min/max/median frequency fall out of the record for free
  - today the clock series stops at warmup's end, which is what hid the 4.09 -> 4.53 GHz
    mid-bench climb from every qualify table: the report named where the box started and that
    it moved, never where it went
  - it is also the pin's verification: under `pin-freq` the per-run stats collapse to the pin
    (min = max), so "the pin held" is a mechanical check on every run's own data, and the
    frequency rung's acceptance leans on it
  - subsumes the sampling half of the "Seam-clock attribution" Todo entry, whose
    report-labeling half ("clock moved" on a mid-run step) stays ranked there
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

As built:

- the shape: `src/record.rs` is `RunOutput`'s second consumer, the one the opening's renderer
  extraction was staged for
  - `build_record` assembles the object, and a `Recorder` resolved at startup writes it
  - a bad `--record` path or `--tag` fails at startup, before any bench spends minutes measuring
  - the harness still never renders, and the display is untouched
- the record's fields
  - absent serializes as `null` with the key kept, never a missing key, so every record carries
    the identical 39-key set
  - the dictionary is enforced both ways: the test fails on an undocumented record key and on a
    dictionary entry naming no key, so `FIELD_DOCS` cannot drift from the data
  - `t_start` is captured, not reconstructed: `RunOutput` gained `wall_start`, taken the moment
    measuring starts, so the stamp is exact and warmup is excluded by construction
  - `BlockStats` now keeps its `means_ns`, so the recorded series is the very values the CI and
    LSC were fit from rather than a recomputation
  - provenance rode along beyond the spec list: binary version, `pin_cpus`, min/mean/stdev/max,
    `suspended_s`, and the warm verdict (`warm_exit`, spend/budget, settle time and clock), each
    a field a re-analysis would otherwise have to ask the dead session for
- the seam clock
  - sampled ungated by `--no-env-probe`: that gate exists for the ~256 us micro-probe, and one
    sysfs read is orders cheaper
  - first smoke run (0.5 s min-now): 12 seam samples, min/max delivered kHz falling out of the
    record with one `jq` line
  - its timestamps are raw integer ns from warmup start (wink, at review): the record stores
    the raw elapsed read and seconds stay derivable, the same rule that keeps policy tokens
    raw. The derived scalar summaries (`duration_s` and kin) stay float seconds
- paths and failure
  - an existing directory without the trailing slash also selects dir mode, since opening it as
    a file could only fail later and less clearly
  - a failed record write exits 1 loudly: the record is the run's evidence, and losing it
    silently is the failure the flag exists against
- the cost: timestamps are hand-rolled UTC (`civil_from_days`) plus `localtime_r` for the
  offset field, so `serde_json` is the rung's one new dependency
- validated live
  - dir and file modes both exercised, and the records parsed with `jq`
  - `describe-record` prints the 39-field dictionary
  - `--tag` requires `--record`, enforced by clap

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

As built:

- `src/md_fence.rs` is vc-x1's filter taken verbatim, its module doc rewritten to carry the
  provenance and the shared-crate candidacy, and its tests living beside it (vc-x1 keeps theirs
  in `config_md.rs`, whose loader is theirs, not ours)
- the sharing question got decided at this rung (wink, 2026-08-17): copy now, propose the crate
  after the cycle lands. A path dependency on vc-x1 was rejected (binary crate, and a
  sibling-checkout dep breaks `cargo install --path . --locked` elsewhere), and making their
  copy a lib module is their agent's work, reached by message. Backlog entry
  "Extract the md -> toml fence filter" holds the plan
- the loader's reshape: `xdg_path` became `xdg_dir`, a `resolve_carrier(md, toml)` decides each
  layer's file (both present is the hard error naming both paths), and `overlay` lost its
  missing-file half (existence is decided at resolve time) while gaining the fence filter keyed
  on the `.md` extension, so a fence diagnostic names the `.md` path and its real line
- left for a later beat, deliberately: `README.md`'s config section and `iiac-perf.toml.example`
  still describe the TOML carrier only. Converting the example to a rendered `.md` document is
  the natural close-out or follow-on edit, once the `[freq]` section exists to document

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

As built:

- the shape: `src/freq.rs` stays read-only and gains base-clock discovery, and the new
  `src/freqctl.rs` is the one module that writes sysfs, carrying the reversal's constraints in
  its module doc
- writes are staged as ordered plans of (path, token) pairs, every clamp move floor-first (min
  to the hardware floor, then max, then min), so each intermediate write satisfies the kernel's
  `min <= max` rule from any starting state, which is what makes restore converge from anywhere
- the declared state is validated against the box at use time, both directions: a knob the box
  exposes must be declared (restoring around it would leave a pin's residue) and a knob it
  lacks must not be, with governor and EPP tokens checked against the sysfs available lists and
  every error naming what the box actually offers
- the `[freq]` table replaces whole across config layers, never field-merges: the steady state
  is one declaration of one box's state, and half of one file's on top of half of another's
  would be a state nobody declared
- MHz at every user surface (config keys, `pin-freq 3800`, `--pin-freq=3800`, error messages),
  kHz only at the sysfs boundary
- the refusal carries its own remedy: `read-freq --as-config` prints the paste-ready `[freq]`
  section the error asks for, and the message warns that sudo may point $HOME at root's config
  while the project-local `iiac-perf.md` still works there
- the run guard engages before the Setup block prints, so the printed policy and the warm loop
  both see the pinned state, and a pin that fails partway restores best-effort before the error
  returns
- the signal path pre-renders the restore plan into C strings at engage time, so the
  SIGINT/SIGTERM handler is raw open/write/close (the async-signal-safe subset) and exits
  128+signal, while normal return and panic restore through the guard's `Drop`
- base-clock discovery per the spec's order, with two hardening details: `nominal_freq` values
  at or above 100000 are taken as kHz (platforms have shipped both units), and the
  `scaling_available_frequencies` fallback skips a top entry sitting exactly 1 MHz above the
  next, the ACPI turbo marker. We think the fallback covers the Pi, unverified until the
  three-box rerun
- the boost knob mirrors the read side: per-CPU `boost` files preferred, the global file the
  fallback, and this box turned out to expose the per-CPU form
- validated live on this box (563-4673 MHz range, base 3801 MHz from `acpi_cppc/nominal_freq`):
  the one-line `read-freq`, `--as-config`, both refusal paths, out-of-range rejection naming
  the range, and the write path failing loudly (the sandbox's read-only /sys standing in for
  the non-root case, whose EACCES adds the sudo hint)

##### fix: the settle cell shows the clock's journey

The settle cell prints the measurement floor as a measurement: on nearly every healthy run the
scan succeeds at probe 0 and the cell reads `0.01s @4.09GHz`, where the time is the first
probe's timestamp, a censored "no ramp seen" dressed as a duration (wink, 2026-08-17: "it
feels like settle = 0.01s @4.09GHz means nothing"). The cell becomes the clock's journey, and
`-v` gains the graph.

- the cell is `start->end time rating`, e.g. `3.60->4.09GHz 0.32s +-0.1%`: the first readable
  clock sample, the settled suffix's median, seconds from warmup start to the suffix, and the
  suffix's relative stdev
  - no observed change (the start sits within the stability band of the settled median):
    `4.09GHz from start +-0.1%`, no fake time and no arrow, the arrow appearing only when a
    real move happened
  - never settled (cap exit): `3.60->4.20GHz not settled`, the journey it was still on when
    warmup gave up. No rating, there being no settled state to rate. Today this case prints a
    clock-free `not settled`
  - clock unreadable (no `cpuinfo_avg_freq`): the timing-only forms stay (`0.32s`,
    `from start`, `not settled`)
- the rating rates the settled state, never the ramp: the arrow owns the ramp, and the rating
  answers "how still is still" inside the 1% gate
  - under a pin it should read `+-0.0%`, the pin certifying itself in every report, and a
    wandering `powersave` box shows a visibly fatter band, the two-regime difference made
    visible
  - on a fast exit the suffix is only the exit window, a handful of samples, so the rating is
    indicative rather than a precision instrument, and the record's seam clock stays the
    evidence
- `-v` gains a clock profile line, e.g.
  `clock: 3.60->4.09GHz ^^^v^----- (min 3.58 max 4.11, settled +-0.1%)`: one tick per
  inter-sample step, `^`/`v` a move the stability band would flag, `-` a hold within it
  - the deadband is `FREQ_STABLE_TOL`, shared with `clock_stable`, so the tick line is the
    stability gate's view of the series: the settled suffix reads all `-` by construction, and
    settle's start is visible as the point the ticks go quiet
  - `^`/`v`/`-` are typeable by design, hard rule 8 barring arrow glyphs from user-visible
    strings. A digit sparkline (level, not direction) was considered and dropped: ticks read
    without mental scaling, and magnitude is bracketed by the line's own numbers
  - direction-change counts were considered and dropped: the profile shows the shape directly,
    and a count without the shape is another number to explain
- `qualify-environment`'s `parse_settle` and its table column update in the same commit, the
  format-and-parser rule the settle rung set

As landed, the conceptual delta:

- `Settle` carries the journey rather than a scan timestamp: `At` gains `held_s`, `start_ghz`,
  and `rating`, `Never` gains its two journey ends. The record schema is untouched: `settle_s`
  keeps the ramp-end `t_s`, which no longer reaches the cell
- the cell's number is the settled share of the warm, a percent (wink, 2026-08-19, the fourth
  form reviewed): the ramp-end time read as a meaningless `0.01s` on a box already at speed,
  an absolute hold read as noise beside the warm budget, and the percent is self-scaling,
  `100%` ready all along down to the exit-window floor (~3% of a default warm), zero-padded
  to two digits so the column's digits align (wink). `00%` is reserved for never-settled, a
  settled share rounding up to `01%`. The spec's no-change and `from start` forms died across
  the same rounds: a journey that went nowhere prints its arrow (`4.09->4.09GHz`)
- the settled share is a graded signal (wink), the one place the clock decides a letter: the
  10-run qualify showed three `not settled` rows graded A, and a fast late ramp can finish
  inside the bench's first batches where no timing detector sees it, so the settle scan is
  the only witness. The share scores against `UNSETTLED` cutoffs (A from a quarter settled, F
  within 2% of never, never itself an F), the letter prints beside the cell like every
  signal's, and it folds into the warmup row's worst. A never-settled-is-F-outright rule came
  first and generalized on wink's "still all A's" review. qualify's median-settled stat
  counts never-settled runs as `00%` rather than skipping them
- every clock statistic reads the dominant CPU only (other CPUs' samples become `None`,
  positions preserved so the gate's probe alignment survives): an unpinned run's sampler rides
  the scheduler across cores, and the mixed series rated placement rather than the clock.
  Measured on the 3900X: +-11.9% mixed against +-0.2% filtered or pinned, found when the new
  rating first exposed what `clock_stable`'s migration fallback waves through
- the scan's clock gate ranges over the filtered series' readable samples
  (`filtered_clock_stable`) instead of reusing `clock_stable`, whose missing-sample fallback
  bailed true at every filtered-out gap and disabled the gate for unpinned runs. Caught by
  wink's grade-A-beside-bare-`not settled` qualify row, and measured as a `+-7.0%` rating
  inside the 1% gate. The `Never` journey's end also falls back to the last readable sample
  when the exit window held none, which is what had emptied that row's journey
- on a clock-readable box, no evidence is no certificate (wink, 2026-08-19): a suffix with
  fewer than two dominant-core readings cannot verify stillness, so it fails the gate and the
  run reads `00% F` rather than a timing-only share. wink's trigger row: an evidence-free
  `18%` graded B beside a verified `04%`'s D, the cell that knew less outgrading the one that
  knew more. A wholly clock-less box keeps its timing-only cells unpenalized, the bare-share
  form now being its exclusive signature
- the report's unstable-exit arm prints the scan's `Never` journey instead of a bare
  `not settled`, with the plain form kept as the defensive fallback
- the `-v` tick line is unabridged, one tick per step: a routine settled run prints ~140
  ticks, long but consistent with the per-probe table's ~141 rows on the same screen
- README's `Settle time` section rewritten around the cell anatomy, having still described the
  pre-clock floor-median algorithm, with the qualify column text, both sample blocks, and the
  `--settle-time` flag doc synced. The explain-it-in-the-README test drove the design: the
  ramp-end time and the mixed-CPU rating both failed it (wink)
- widths widened for the fattest cell (`4.84->5.24GHz 100% +-0.2% A`): the grade block's
  settle column 15 -> 27, qualify's 16 -> 27

##### feat: block sleep and warmup become knobs

Blocks carry two invisible behaviors: a random 1-10 ms sleep between blocks and 2 ms of
unrecorded post-wake warmup, neither shown in any report nor settable. wink's 2026-08-19 7600x
runs made the cost concrete: three A-graded runs whose means span 16.2 to 16.5 ns with block
count the only knob turned, and no printed number honest enough to say whether the movement is
real. The knobs surface, and zero becomes the neutral setting: a run never sleeps or discards
samples unasked (wink, 2026-08-19).

- `--block-sleep <spec>` and `--block-warmup <spec>`, each with a config key landing in the
  same commit per "Config keys stay CLI-settable". Sleep accepts a scalar or a range
  (`1-10ms`), the range re-rolled per block, because fixed sleeps invite the recorded
  flip-zone hazard (fixed 0.5 ms sleeps straddled both 3900X states)
- defaults are zero: sleepless blocks are partitions, not replicates, so the replication rows
  (CI95 / LSC) print `-` unless the sleep is nonzero. The block-mean series still records
- both values print in Setup whenever blocks run, zeros included, and ride the record with
  dictionary entries, so a record is interpretable without its command line
- a long sleep with zero warmup is the cold-wake instrument arriving early: cold samples land
  in the histogram as a visible band shoulder. The separated first-K decay profile stays with
  the Cold-wake Todo entry
- ahead of the LSC rung on purpose: gap-separated blocks are quasi-runs inside one invocation,
  the validation data the drift floor is checked against

As built:

- `src/timespec.rs` is the span grammar's one home: scalar or range, `us`/`ms`/`s`, the unit
  mandatory on any nonzero value (a bare `5` meaning ms to one reader and s to another is the
  ambiguity the knobs exist to remove), a range's trailing unit distributing to its lower end
- `run_blocked` skips the sleep and the warmup at zero and draws the sleep uniformly from the
  span per block. `BlockStats::from_means` gained a `replicated` flag: CI95 / LSC became
  `Option`, `None` for sleepless partitions, so the partitions-cannot-replicate rule lives in
  one place and the report and the record both inherit it
- the report prints a bare `-` (no unit) for a withheld CI95 / LSC, a unit dressing the
  absence up as a number
- the record added `block_sleep_min_s` / `block_sleep_max_s` / `block_warmup_s`, null when
  not blocked, CI95 / LSC now null for partitions too, and `schema_version` bumped to 2
- Setup prints both knobs whenever blocks run, zeros included, each naming its consequence
  ("blocks are partitions", "records from the first post-wake call")
- smoke-verified: sleepless `--blocks 4` shows the partition note and dashes, and
  `--block-sleep 1-10ms --block-warmup 2ms` reproduces the retired hidden behavior explicitly

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
- the curve fits on **batch means**, not block means (wink, 2026-08-19): batches are the
  contiguous ~50 ms slices Allan deviation is defined over, `BatchSummary` already carries
  `mean_ps` and `count` on every run, and a 127-batch run reaches aggregation groups a
  10-point block series cannot
  - the honest resolution therefore prints on every run, `--blocks` or not, which unhooks the
    claim from the blocks-default question
  - the aggregation weights by sample count, batch durations being uneven (fast benches flush
    on the sample buffer at ~15-40 ms, slow ones on the 0.05 s timer)
  - the record gains the batch-mean series beside the block-mean series, or a re-analysis
    cannot reproduce the curve
  - block stats stay as the replication cross-check: blocks re-roll scheduler and frequency
    state across sleeps, a different question from drift within one stretch
- naming it accurately matters as much as computing it: a within-run bound must not print in a
  way that reads as a run-to-run one, which is the whole defect

##### fix: the pin flag names CPUs

`--pin` beside `--pin-freq` no longer says which of the two things it pins, and the code calls
the same list three names: the `CORES` metavar, `pin_cores`, and the record's `pin_cpus`. The
project adopts the kernel's vocabulary (wink, 2026-08-19): a **CPU** is the schedulable logical
processor (sysfs `cpuN`, one affinity-mask bit), a **core** is the physical core hosting two of
them, and **SMT siblings** are the CPUs sharing a core. vCPU stays out, being virtualization
vocabulary with no referent on bare metal.

- the flag becomes `--pin-cpus`, metavar `CPUS`, help text swept. Whether `--pin` survives as a
  hidden clap alias is decided at the rung, nothing external consuming the CLI
- the code sweeps to match: `pin::parse_cores` -> `parse_cpus`, `pin_cores` -> `pin_cpus`, and
  comments saying core where they mean CPU
- the record field `pin_cpus` is already right and is schema, so it does not change
- README renames the flag docs and gains a short Terminology note stating the convention, the
  reader-facing home for the vocabulary placement-map.md already speaks
- the prior terminology discussions stay where they rank: Todo's "Tighten thread/CPU
  terminology" holds the wider docs sweep (software thread against CPU), of which this rung
  delivers only the `--pin` bullet, and the topology-level vocabulary (lCPU, LLC domain) stays
  with "Topology-aware pinning and lCPU terminology"
- ahead of suggest-freq so the campaign's recorded command lines carry the final names

##### feat: suggest-freq measures the pin frequency

Measure the best pin frequency instead of defaulting to base clock (raised 2026-08-15 during
laddering, pulled in from `## Todo` 2026-08-19 for the zcr-mpsc campaign). "Best" is the highest
frequency the box *holds* under the intended workload and schedule, so it is thermal and
duty-cycle dependent: a short run passes at a frequency a long run throttles from, and the
2026-08-02 data showed the schedule selecting the state.

- shape: descend from max-with-boost-off, pin each candidate, drive the load for at least the
  intended run duration, verify with `clock_stable` plus the grade, and report the highest
  candidate that held, named with the schedule it was measured under
- the load is the real bench, not a synthetic stand-in: the command takes the bench word and
  the run flags (`suggest-freq zcr-mpsc-2t --pin-cpus 0,12 -d 5`), because 2t on SMT siblings
  heats differently than 1t on one CPU, so the held frequency is per-bench and per-layout
- it mutates, so it reuses the pin-freq / restore-freq machinery and inherits its exit-path
  restore guarantees (normal exit, panic, SIGINT)
- guidance beyond the number: report what was measured, each candidate, how long it held, and
  under what load and schedule, ending with the config line to paste (`pin_mhz = ...`), the
  same paste-ready shape as `read-freq --as-config`
- its spec wanted the record and LSC rungs landed first, and both are by this point in the
  ladder: the block-mean series and the drift floor are the evidence a suggestion is judged by

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
  - superseded in part 2026-08-16: the settle rung's state column measured the ladder directly,
    and the dwell finding is
    [The 3900x dwells, it does not settle](#the-3900x-dwells-it-does-not-settle-2026-08-16)

#### The 3900x dwells, it does not settle (2026-08-16)

Evidence from the settle rung's own state column on the day it landed, read across wink's runs
on both boxes, transcribed in full below the bullets since evidence in a transcript alone is
perishable. "Settle" assumes convergence to a final state, and under today's policy the 3900x
has none.

- the settle cell on a state-hopping box records when the *last disturbance during warmup*
  happened, not a ramp length: the cell is the earliest suffix flat through warmup's end, so
  one flip at 4.5s of a 6.5s warm reads `4.53s` even when the first 4.5s were glass-flat
- the transition into the ~4.5 GHz top state is stochastic, not a fixed delay: with
  `--settle-time 5` and a 5s gap, one run sat flat at 4.09 GHz through the full 5s of sustained
  warm load and climbed mid-bench anyway, while siblings climbed at 1.9s and 4.8s of theirs. An
  earlier read of "the climb takes ~2s of sustained load" did not survive this data
- state residency runs seconds with heavy tails: one run held ~10s across warm and bench and
  graded A while its neighbors flipped mid-window and took D/F. The box dwells in states, so no
  warmup budget can wait out a coin that keeps flipping, and a longer `--settle-time` only
  moves the odds
- the state ladder is now measured rather than inferred: 4.09 GHz entry and ~4.50-4.54 top on
  the 3900x (`min-now` means ~24.2 ns and ~21.8-21.9 ns respectively), where the seam-clock
  entry had guessed ~3.79 / 4.17 / 4.35 from timing ratios
- the 7600x is the control that makes "dwell" a property of the box rather than the tool: after
  a 5s gap it ramps 0.81-0.82s to 5.44 GHz every run without exception at 16.2 ns, 5A and
  QUALIFIED, and run back-to-back it inherits the warm state and settles at 0.01s at the same
  5.44 GHz. That box genuinely settles: a convergent, deterministic ramp to a state it holds
- the consequences are ones the block already carries: the base-clock pin removes the coin
  entirely, the record rung's seam-sampled clock makes mid-bench flips visible, and the
  qualify verdicts stand: the D/F grades were correct all along, and the settle column now
  explains them instead of contradicting them

The runs the bullets read from, verbatim. The 3900x pair (wink's terminal, GUI desktop, this
bot's session also live on the box):

```
wink@3900x 26-08-16T04:44:39.339Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility+1)
$ iiac-perf qualify-environment --settle-time 5
iiac-perf 0.26.0-1 — Rust latency microbenchmark harness

qualify-environment: 10 runs of `min-now -d 1`, gap 0s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle           mean
  1     A       F        F       1.95s @4.52GHz   22.7 ns
  2     B       A        B       not              21.7 ns
  3     A       D        D       0.01s @4.52GHz   22.1 ns
  4     A       B        B       0.01s @4.52GHz   21.8 ns
  5     A       B        B       0.03s @4.52GHz   21.8 ns
  6     A       B        B       4.53s @4.52GHz   21.7 ns
  7     A       A        A       1.55s @4.09GHz   24.2 ns
  8     B       B        B       not              21.8 ns
  9     A       A        A       0.01s @4.52GHz   21.9 ns
  10    A       B        B       0.09s @4.51GHz   21.7 ns

  environment grades: 2A 6B 1D 1F
  median environment grade: B
  median settle: 0.09s (2 of 10 never settled)
  transition-degraded (drift or step at D/F): 2 of 10

  verdict: NOT QUALIFIED
    a state transition landed inside a measurement window
wink@3900x 26-08-16T04:46:21.929Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility+1)
$ iiac-perf qualify-environment --settle-time 5 -d 5 --runs 5 --gap 5
iiac-perf 0.26.0-1 — Rust latency microbenchmark harness

qualify-environment: 5 runs of `min-now -d 5`, gap 5s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle           mean
  1     A       F        F       0.01s @4.09GHz   23.0 ns
  2     A       F        F       4.79s @4.50GHz   23.7 ns
  3     A       B        B       1.94s @4.51GHz   21.9 ns
  4     A       A        A       4.86s @4.49GHz   21.8 ns
  5     A       B        B       1.88s @4.51GHz   21.8 ns

  environment grades: 1A 2B 2F
  median environment grade: B
  median settle: 1.94s (0 of 5 never settled)
  transition-degraded (drift or step at D/F): 2 of 5

  verdict: NOT QUALIFIED
    a state transition landed inside a measurement window
```

The 7600x pair, minutes later on the headless box (ssh, no GUI, no bot):

```
wink@7600x 26-08-16T04:52:02.804Z:~
$ ./iiac-perf qualify-environment --settle-time 5
iiac-perf 0.26.0-1 — Rust latency microbenchmark harness

qualify-environment: 10 runs of `min-now -d 1`, gap 0s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle           mean
  1     A       A        A       0.80s @5.44GHz   16.2 ns
  2     A       A        A       0.01s @5.44GHz   16.2 ns
  3     A       A        A       0.01s @5.44GHz   16.2 ns
  4     A       A        A       0.01s @5.44GHz   16.2 ns
  5     A       A        A       0.01s @5.44GHz   16.2 ns
  6     A       A        A       0.01s @5.44GHz   16.2 ns
  7     A       A        A       0.01s @5.44GHz   16.2 ns
  8     A       A        A       0.01s @5.44GHz   16.2 ns
  9     A       A        A       0.01s @5.44GHz   16.2 ns
  10    A       A        A       0.01s @5.44GHz   16.2 ns

  environment grades: 10A
  median environment grade: A
  median settle: 0.01s (0 of 10 never settled)
  transition-degraded (drift or step at D/F): 0 of 10

  verdict: QUALIFIED
wink@7600x 26-08-16T04:53:06.854Z:~
$ ./iiac-perf qualify-environment --settle-time 5 -d 5 --runs 5 --gap 5
iiac-perf 0.26.0-1 — Rust latency microbenchmark harness

qualify-environment: 5 runs of `min-now -d 5`, gap 5s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle           mean
  1     A       A        A       0.82s @5.44GHz   16.2 ns
  2     A       A        A       0.81s @5.44GHz   16.2 ns
  3     A       A        A       0.82s @5.44GHz   16.2 ns
  4     A       A        A       0.82s @5.44GHz   16.2 ns
  5     A       A        A       0.82s @5.44GHz   16.2 ns

  environment grades: 5A
  median environment grade: A
  median settle: 0.82s (0 of 5 never settled)
  transition-degraded (drift or step at D/F): 0 of 5

  verdict: QUALIFIED
```

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

1. Two-regime runs: a config key selects the box's default regime, pinned or wandering, and the
   CLI overrides it either way, so a tuning campaign pins every run without typing the flag and
   a quick sanity check drops back to the real-world clock one-shot (wink, 2026-08-17)
   - the workflow it serves (written into the report guide entry below): tune pinned, where LSC
     is small enough that "did this tweak clear LSC" resolves in a few runs, then confirm the
     winner unpinned, where the number means what the real world will see
   - the key is a run parameter, CLI-settable per "Config keys stay CLI-settable" below, not
     part of the `[freq]` declaration: it says which regime runs use, while `[freq]` stays the
     declared way home. We think top-level `pin_freq = true|false` beside `duration`, with
     `--pin-freq` / `--no-pin-freq` as the override pair and `--pin-freq=MHZ` still naming a
     target
   - the wandering default stands for an unconfigured box: pinning stays something the user
     asked for, in config or on the line, never a surprise mutation
2. Cold-wake profile: measure the post-wake transient (C-state exit, cache and TLB refill, the
   clock ramp) instead of discarding it. A real consumer of an MPSC ring blocks and wakes cold
   where the spin-wait benches stay maximally hot, so the cold-wake cost is arguably the number
   a deployment feels (wink, 2026-08-19)
   - sleep does not flush caches by itself: a long enough idle lets the core enter deep
     C-states, which power-gate the core and lose the private caches, so sleep duration is the
     dial for how cold a wake starts. Unpinned, the wake may also migrate cores, so the cold
     axis wants sweeping with `--pin-cpus`
   - the raw instrument lands with the measure-reproducibility cycle's "block sleep and warmup
     become knobs" rung: `--block-sleep` reaches seconds and selects the depth of cold, and
     `--block-warmup 0` records from the first post-wake call, so cold samples already show up
     as a band shoulder
   - the deliverable is the time-ordered decay after each wake: record the first K per-call
     values post-wake verbatim (the clock-journey move applied to latency), reported as a
     per-block decay profile, never folded into the steady stats, since cold samples would
     contaminate block means, CI95, and the resolution claim
   - same family, opposite sign: reproducibility runs may want deep C-states forbidden (the
     `/dev/cpu_dma_latency` clamp, a pin-idle sibling to pin-freq), while this mode wants them
     allowed
3. Rethink environment rating, then resume the qualify power-policy rung: the settle-cell
   rework improved the grades but wink's verdict is "better but not good enough" (2026-08-19),
   so the environment-rating design gets its own discussion and likely redesign, and the
   qualify rung, halted unbuilt the same day, resumes on the result with its spec preserved
   here
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
4. Config keys stay CLI-settable: adopt the convention that every run-parameter config key has a
   CLI flag with CLI-wins precedence, the flag landing in the same commit as the key, so any
   experiment is runnable one-shot without editing a file (wink, 2026-08-17)
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
5. Prepare for expected errors: before a command whose outcome is not clean (a rebase across
   diverged lines, a force-push, dogfooding a dev tool), state the expected output, what
   unexpected would look like, and the abort path, then fix stepwise with the user
   - the forward-looking half of hard rule 10's stop-and-ask, and family-shaped, so it belongs
     in pinned AGENTS.md working practices via its own convention cycle
   - born 2026-08-14: a rebase's predicted conflicts arrived unannounced and read as breakage
     (wink stopped the session), the prediction living in a record instead of in the moment
6. Report interpretation guide: a reader-oriented "how to read a report" walkthrough in README,
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
   - the two-regime workflow (wink, 2026-08-17): tuning runs pin the clock so LSC shrinks to
     where "did this tweak clear LSC" resolves in a few runs, and reporting runs keep the
     wandering clock, whose number is what the real world will see. Tune pinned, then confirm
     the picked winner unpinned. We think a pinned ranking can occasionally flip unpinned
     (boost behavior interacts with how a workload holds cores), which is why the confirm step
     exists. The pyperf tune/reset pair is the same idea, ours being pin-freq / restore-freq
   - the report is dense by design; the guide is the decoder the grade-block compaction (0.23.2)
     assumed exists
7. Blocks as the first-class mode: knobs, always-on error bars, then a measured default flip
   (designed 2026-08-02, the duty-cycle/LSC session; evidence in chores-06)
   - the sleep and warmup knobs land via the measure-reproducibility cycle's "block sleep and
     warmup become knobs" rung (defaults zero, replication rows gated on a nonzero sleep).
     Still this entry's: `--blocks` gains a config key so a box's config can run blocks = 1000,
     and the flip-zone hazard stays the range-over-fixed argument (fixed 0.5 ms sleeps
     straddled both 3900X states, D grade, LSC 6x worse)
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
8. Always work on a topic bookmark: cycles happen on a bookmark, `main` advances only by landing
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
9. Sync the 20260803 agent-files baseline [[84]]
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
10. Qualification reports evidence, not verdicts: retire the prejudging NOT QUALIFIED stamp
    (wink, 2026-08-02) in favor of measured statements a reader judges
    - blocks-based: A/A repeatability (does a same-code delta clear LSC?), CI95/LSC as the
      published sensitivity ("this box resolves X ns on this bench"), stratification by state
      instead of a blended letter
    - the 3900X reads NOT QUALIFIED today for mid-run bistable transitions warmup cannot
      prevent: a trait to report, not a disqualification; the 7600x dwell case that motivated
      the gate is fixed by the dynamic-warmup cycle
    - entangled with "Qualify the environment without a bench" (below) and machine-readable
      output; wants the blocks-knobs entry (above) landed first
11. Seam-clock attribution: sample `cpuinfo_avg_freq` at batch seams (the reader exists,
    `src/freq.rs`) so a mid-run step gets a "clock moved" label, the way warmup now separates a
    dwell from the top; also the natural home for surfacing the clock ratio in normal output as
    one coherent story (chores-06: the 3900X flip at ~2-4 s is almost certainly a visible clock
    move)
    - the sampling half moved into the measure-reproducibility cycle's record rung (2026-08-16):
      seam samples join the record, per-block/per-run frequency stats fall out, and the pin
      verifies itself. What stays here is the report surface, the "clock moved" label
    - the ~2-4 s flip is no longer a guess: 0.26.0-1's settle state named the states directly,
      4.09 GHz entry and ~4.53 GHz top on the 3900x under today's policy
12. Qualify the environment without a bench. `qualify-environment` respawns children running
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
13. Guard `--pin` pools smaller than the bench's thread placements, and deadline the estimate
    phase: `zcr-mpsc-2t --pin 8` put both spinning software threads on one logical CPU and
    appeared hung until ^C (2026-07-26, bug #1 in [bugs.md](notes/bugs.md#bugs))
    - track `core_for` requests in `RunCfg` (max `thread_idx` asked for); refuse the run when
      placements exceed unique CPUs in the pool. Placement only goes through `core_for` when
      pinning is active, so the guard covers every path, and no pinning means the scheduler
      separates the spinners itself
    - wall-clock deadline on the open-loop 5x1,000-step estimate phase so *any* pathologically
      slow bench aborts with a diagnostic naming per-step cost and pinning, instead of hanging
14. Move the batch seam's work off the measuring thread, using the FastForward-style SPSC ring.
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
15. Tighten thread/CPU terminology across docs and doc comments: "software thread" for what
    `thread::spawn` makes, "logical CPU" (hardware thread) for what `--pin` selects and the OS
    schedules onto, "physical core" for the engine SMT siblings share. Bare "core"/"CPU"/"thread"
    only where context disambiguates
    - spin-wait bench docs state the precondition: each spinning software thread needs its own
      logical CPU
    - `--pin` help/README say slots are logical CPU ids
16. Topology-aware pinning and lCPU terminology: discover the CPU sharing tree at runtime and
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
17. Rebase `web-claude-tweaks` onto post-0.22.0 `main`. It rewrites an already-published
    bookmark (needs approval) and its arbitrary `0.21.0-b` version needs replacing; owed from
    the 0.22.0 close-out plan
18. Unit scaling in report columns (`us`/`ms`): per-row auto-scale so columns stay
    eyeball-comparable (bands are monotonic, so a row's first/last/mean share a magnitude), or
    `--units ns|auto` for script-stable output; needs `--decimals` landed first (`3.18 ms` vs
    `3 ms`); candidate `-4` for the report-options cycle.
19. Machine-readable report output (`--format json`, or key=value lines to stay
    dependency-light). Design once the batch gauge lands (0.23.0-4) so the schema covers the
    surviving surface: report stats, gauge signals, letter. Consumers:
    `tests/qualify_environment.rs` (drops its brittle-but-loud line parsing), placement-map
    validation runs, cross-run comparison scripts. Kin to the unit-scaling entry's `--units ns`
    script-stable concern (above), one flag family.
20. Trimmed core stats: `mean/stdev p10-p90` report row, additional to (never replacing)
    `mean` / `mean min-p99`; trim bounds possibly configurable (`--trim p10:p90`?). Why: the
    full mean wobbles ~±1.4% with the run's mode mix while the core plateau is ~±0.2% stable,
    so the trimmed row is the run-to-run comparable number. Boundary sensitivity (see [[57]]):
    window edges in the mode-mix smear inherit its wobble (p50-p60 ±0.05% vs p40-p50 ~1%), so
    also consider a dominant-*mode* statistic (peak-density region, bottom-count-independent)
    [[57]]
21. Find and label the interference crossover: the band where the tail stops measuring the code
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
22. Investigate: suspend gap missing from samples. A 0.13.5 `--no-inhibit` suspend test
    detected ~1.2 s suspended inside the measured window but the max sample was only 4.0 ms,
    while the 0.13.1 test (8.4 s gap) showed the expected 10.4 s max sample. We think
    minstant's TSC may halt across some suspends and count through others. Repeat the test
    comparing detected gap vs max sample; if the TSC halts, per-sample timing silently loses
    suspend time; document either way.
23. CLAUDE.md governance model (design cogitation) [20]
24. Revisit probe adjustment under the in-interval vs call-to-call split: probes take one call
    per sample (inner=1), so the in-interval timer slice is unamortized and unmeasurable, so an
    `adjusted` column can subtract nothing defensible; maybe state a bound instead
    [analysis](notes/design.md#timer-overhead-in-interval-vs-call-to-call)
25. Convert `harness` / `Bench` to probe-based measurement. Will likely need inner-loop support
    on `Probe` (batch N calls per sample; report divides by N and accounts for per-sample
    framing) so very-small workloads can still amortize timer overhead the way `run_adaptive`
    does today.
26. Rename app
27. Design an app to measure IIAC perforanace written in Rust[1]
28. `ice-ps-2t-wait`: iceoryx2 pub/sub with blocking waits via `Listener`/`Notifier` events;
    completes the {transport} × {wait policy} matrix cell that compares against `mpsc-2t`
29. Switch ice benches to the loan-based zero-copy send path (`loan_uninit` + `send`), the API
    a perf-sensitive user would use, and closer to iceoryx2's own benchmark method
30. Payload-size sweep for the round-trip benches (8 B / 8 KiB / 1 MiB), makes iceoryx2's
    size-independent latency vs channel copy cost visible in our own tables
31. `crossbeam-1t` / `crossbeam-2t`: `crossbeam-channel` directly (compare to mpsc-1t/2t which
    use crossbeam under the std API)
32. `tokio-mpsc-1t` / `tokio-mpsc-2t`: `tokio::sync::mpsc` round-trip inside a Tokio runtime
    (async overhead)
33. `flume-1t` / `flume-2t`: `flume` MPMC channel
34. Function-call baselines: direct call vs `Box<dyn Trait>` vs `async fn` (poll-once): anchors
    the channel/serde numbers against the cheapest possible "send a value then receive it" path
35. When the second channel impl lands, extract shared message types + round-trip helpers into
    `src/benches/common.rs` (deferred from 0.2.0)
36. Additional thread control (count, per-thread pin lists, NUMA): shape once a concrete bench
    needs it
37. Rename crate `iiac-perf` -> general-purpose name (breaking; deferred)

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
[101]: #fix-lsc-gains-a-run-to-run-component
[102]: #feat-measure-reproducibility-closing
[103]: #port-measure-reproducibility
[104]: #feat-read-pin-and-restore-the-cpu-frequency
[105]: #feat-adopt-the-markdown-config-carrier
[106]: #fix-the-settle-cell-reads-the-clock
[107]: #fix-the-settle-cell-shows-the-clocks-journey
[108]: #fix-the-pin-flag-names-cpus
[109]: #feat-suggest-freq-measures-the-pin-frequency
[110]: #feat-block-sleep-and-warmup-become-knobs
