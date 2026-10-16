# Doing: the cycle record

What is being done: the running cycle's planning and its work's documentation. The last
finished cycle's is in [zCLOSED.md](zCLOSED.md), and what we might do is in [TODO.md](TODO.md). The
block's shape is on trial: it opens with the Todo as it was picked from `TODO.md`, then the
Deliberation, the plan, Solution and Acceptance check, and the Ladder with its details.

This file is a trial, taken from vc-x1's (2026-10-16). The agent-files still name `TODO.md > ## In
Progress` as the record's home, and the cycle that makes this file the rule is vc-x1's [The
cycle-record moves out of TODO.md into
zDOING.md](https://github.com/winksaville/vc-x1/blob/43087e0f81d9/TODO.md#the-cycle-record-moves-out-of-todomd-into-zdoingmd).

## Continuation notes

Where the agent was, for the agent that comes next. Ephemeral, never a record: read first, acted
on, and reset to `_None._` by the reader. Kept here by the trial and no longer in `TODO.md`,
where [Acquaint](AGENTS.md#acquaint) still says to look.

- The bookmark `feat-figures-draws-time-per-step-against-inner` is published at `0b6b686`, the
  opening of [figures draws time per step against
  inner](TODO.md#figures-draws-time-per-step-against-inner) paused before its first commit for the
  cycle that made this file. Its ladder is agreed (wink, 2026-10-16): the opening, `figures`
  learns `--figure` and panels with dots, a categorical colour and a legend, time per step against
  `inner`, the same with the harness overhead off, a random run's slope, the 7600X probe records
  into `records/`, the figures into `notes/random-inner.md`, and the closing. The opening moves
  the bookmark forward to `main`. It waited for `feat: the CLI is subcommands`, landed
  2026-10-16, so its `--figure` lands in the `figures` subcommand's own arguments.
- Messages (2026-10-13): `tmp/m-9-0-draft.md`, the message to zc-ring-x1 carrying the v3 and v4
  notes, a new thread, is held for wink's review and not sent. Its name is stale, `m-9` being
  vc-x1's thread now, so it takes the next id when sent. It still has the old names and a `<sha>`
  placeholder. Hold it until the figures cycle says which of the v4 note's findings stand:
  `Single`'s check cost on the 7600X, which it was to carry, follows `inner` and is withdrawn.
- `tmp/runs-debug.patch` is wink's two debug prints in `src/runs.rs`, set aside at the closing so
  its commit did not carry them.
- The stopped sweep's one pass, 800 runs, five two-thread benches on `smt` and `ccx` at 16 of 24
  `inner` values, is `tmp/layout-7600x/partial-big-sweep-7600x.jsonl` on the 3900X, with the
  probes' records beside it, none committed. They go into `records/probes-7600x-20261008/` in the
  cycle of [figures draws time per step against
  inner](TODO.md#figures-draws-time-per-step-against-inner), with
  `tmp/next-cycle-data/probes-README.md` as its README, which says what each group is and the loop
  that took it, read back from the 2026-10-08 session. `lay-*-r` and `-s` are two invocations with
  address randomization off.
- The 7600X has `iiac-perf-dev` 0.28.24-2 in `~/.cargo/bin`, copied there for the random runs,
  beside the plain `iiac-perf` 0.28.23. The Pi 5 and the 3900X have no dev build of this cycle
  beyond the 3900X's own.
- The overnight sessions of 2026-10-08 on the landed names, five invocations of ten runs and one
  of fifty per host, `--run-sleep 100-200ms`, are in `tmp/session-<host>-20261008/` on the 3900X
  and not committed. `tmp/cmp3.py` compares them with the first session. The v4 note's tables are
  not rebuilt from them, held by [The period of
  4](TODO.md#the-period-of-4-in-a-can-sleep-rings-level).

## In Progress

### feat: how inner is chosen and shown

(wink, 2026-10-16) `--inner` and `--samples` default to auto, each run sizing both for itself, and
a bench of several runs printed neither, though a run's level can follow the loop count it was
measured at, as [The period of 4](TODO.md#the-period-of-4-in-a-can-sleep-rings-level) found. A
span, `--inner 3-18`, gives each sample its own loop count and a run every count in it, which
smooths that dependence, and auto never chooses one. This cycle decides how `inner` is chosen,
how a run uses it, and how the run table shows it:

- the run table shows each run's `inner` and `samples`, and under a span graphs how each loop
  count leaves the run's own line
- the principle that frees the default to change: let history guide us but not limit us
- fixed auto against span auto, measured, the records committed
- auto as a span at least 16 loop counts wide, if the measurement bears it out, with the
  per-step level and the band table's meaning under a span decided with it

#### Deliberation

- A multi-step cycle on its own bookmark, landing as a trapezoid (wink, 2026-10-16): it began as
  the single-step `feat: run tables show inner and samples`, reshaped before its first push once
  its subject was how `inner` works rather than one table. Not the figures cycle, which stays
  paused and can draw from this cycle's records.
- Option 3 of three for the sizing (wink, 2026-10-16): `inner` and `samples` columns in the run
  table, and `inner` and `samples` rows under `clock` in the summary.
  - Above the table, as first asked, cannot be: the header prints before the first run, and each
    run's sizing exists only once that run has finished.
  - Columns alone lose the range at a glance, and a row alone loses which run took which.
  - A range across runs reads `lo to hi`, as `counters` does, since a span already spells `1-16`.
  - The `Config:` list keeps `auto`: the table says what auto chose, run by run.
- Under a span the column graphs each run's residuals from its own least squares line (wink,
  2026-10-16, option 3 of four).
  - A span gives a run no one loop count, each sample picking its own, so the column's title
    carries the span, `inner 3-18`, and each cell is the graph.
  - The run table prints once the last run is done, a progress line, `run 1, 2, 3`, twenty runs a
    line, counting them off as they finish (wink, 2026-10-16). Under `-v` the children's reports
    are the progress. It replaced a graph table of its own after the runs, the run lines staying
    live (option 1 of two, the other the graph off by default), so each run's graph sits beside
    its mean and nothing but a few digits reaches the terminal while a run measures. Drawn in the
    run lines, between runs, the graph disturbed the next run: wink's `--inner 5-12` of `zcr-mpsc-v4-2t-single-so-spnt1us` on the 3900X had seven
    noisy runs of ten, `stdev blocks` up to 25 ns, a residual up to 213 ns, and means of about 91
    to 100 ns. With `--no-graph` straight after, all ten were clean at 88.84 to 89.10 ns, and with
    the graph on but the output sent to a file, all ten were clean too, up to a 3.6 ns residual.
    So the fit is not the cause. We think it is the terminal drawing the block characters,
    unpinned, beside the next run's threads. The other choice was the graph off by default.
  - Samples by loop count, the first graph, were flat on every healthy run, the draw being
    uniform. Mean sample time by count would be a ramp and time a step a falling curve, the same
    on every bench, and dropping the graph for plain `3-18` was the fourth option.
  - The fit is `linear-regression`'s own `ols` and `residuals`, so the column and the command
    cannot disagree.
  - Bars over a braille dot line (wink, 2026-10-16): braille is two dots wide and four tall by
    Unicode, no character holding four across, so a dot line packs two counts a character at four
    heights where a bar is one count at eight.
  - Up to sixteen counts a bar each, not ten (wink, 2026-10-16): `1-16`, the span the regression
    usually draws, then reads count by count. Wider spans take bars of equal counts, each the mean
    of its counts so a short last bar is no dip.
  - The bars span the run's own lowest to highest residual, which magnifies noise, so a `residual`
    column of its own says the size the shape is drawn at (wink, 2026-10-16, each labelled).
  - The residual is signed and at its loop count, `+2.039 ns at 14` (wink, 2026-10-16): a bare `±`
    said neither which bar nor which side of the line. `at 14` over `@14` as the plainer reading,
    the count always two columns so the cells line up on `ns`.
  - The docs say "loop count", wink's word, where the code says length or `inner`, and that `at
    14` is the loop count, not the bar's position, the twelfth bar under `inner 3-18` (wink,
    2026-10-16, after reading it as the bar).
- Let history guide us but not limit us (wink, 2026-10-16): new runs use the best method known,
  and old records stay as the evidence for why it changed, never a baseline the new one must
  match. So the default may move every bench's number.
  - It goes into [measuring-a-technique.md](notes/measuring-a-technique.md), iiac-perf's own
    method, in this cycle (wink, 2026-10-16), not the agent-files: it is how iiac-perf measures,
    not a family rule.
- The measurement takes `min-now`, one `zcr-spsc` two-thread ring, and one can-sleep `zcr-mpsc`
  two-thread ring (wink, 2026-10-16), wink's recent runs being `--inner 3-18
  zcr-mpsc-v4-2t-single-sos-futex-spntfe`. The 3900X runs here, CPU-pinned with the clock
  unpinned, the sandbox's `cpufreq` being read-only, and the 7600X's are wink's.
- The span auto chooses: at least 16 counts wide (wink, 2026-10-16), four whole periods of 4 and
  the graph's limit, anchored on the count auto would have chosen, about eight on either side,
  floored at 1. `--inner N` stays fixed, the tool for one count's latency.
- `linear-regression` stays a reader of records, not a second measuring command (wink,
  2026-10-16, option 1 of three). Wink's runs of `zcr-mpsc-v4-2t-single-so-spnt1us` on the 3900X
  read 89.038 ns at `--inner 5-12` and 88.206 ns at `5-20`. We think that is the per-sample cost
  spread over more loop counts, each sample recording its time over its count, which the slope
  does not carry. So whether a run's level becomes its slope is decided in [feat: auto inner is a
  span at least 16 wide][6], and [feat: fixed and span auto measured][5] compares the mean and the
  slope, those two runs its first evidence.
- Waiver (wink, 2026-10-16, after the review of `feat: a span graphs each run's residuals`):
  "permission to complete the cycle but not land on main". It covers every rung's push and the
  closing's, and not Land.
- A waiting animation while a run measures is a Todo, not a rung (wink, 2026-10-16, asked for as a
  tweak if easy): it would draw on the terminal through every measurement, which the graph showed
  can disturb a run, so it waits for the test that says it does not.
- The opening carries no dev rename of its own: the package has been `iiac-perf-dev` since this
  cycle began as a single step, and Land restores it.

#### Solution

_Provisional._ The run table carries each run's `inner` and `samples` and the summary their
range, and under a span the `inner LO-HI` graph of each run's residuals with the largest beside it,
the table printed once the last run is done.
`measuring-a-technique.md` states the principle. Fixed and span auto are measured on the three
benches and the records committed, and auto becomes a span at least 16 counts wide where they show
it is the better method, the per-step level and the band table decided with it.

#### Acceptance check

- A bench of several runs prints each run's `inner` and `samples`, and under a span each run's
  graph and `residual`, `run --help`, the README, and the report guide saying how to read them.
- `measuring-a-technique.md` has the principle.
- The comparison's records are in `records/`, and `analyze --compare` on them is quoted with what
  it shows.
- With no `--inner`, a run times a span at least 16 counts wide, or the record says why not.
- `vc-x1 validate` passes.

#### Ladder

- [feat: how inner is chosen and shown opening][1] (done)
- [feat: run tables show inner and samples][2] (done)
- [feat: a span graphs each run's residuals][3] (done)
- [docs: when the method improves][4]
- [feat: fixed and span auto measured][5]
- [feat: auto inner is a span at least 16 wide][6]
- [feat: how inner is chosen and shown closing][7]

##### feat: how inner is chosen and shown opening

The cycle's setup: this block written, the version at the opening's, and the bookmark renamed for
the cycle's title.

##### feat: run tables show inner and samples

Each run line prints the run's `inner` and `samples` after `pid`, a span as written, `3-18`, and
the summary prints their rows under `clock`, one value when every run agrees and `lo to hi` when
they differ. The run summary a parent reads back from each record now carries both, a record from
before spans taking its one `inner` as both ends. `run --help` and the report guide say what the
columns and rows show, the guide's capture kept as taken, with a note that it predates them.

##### feat: a span graphs each run's residuals

Under a span the `inner LO-HI` column graphs each run's residuals from its own line, the largest
in a `residual` column, `run --help`, the README, and the report guide saying how to read it. The
run table waits for the last run, a progress line counting the runs off meanwhile, because drawn
between runs the graph disturbed the next run. `run --no-graph`, wink's flag (2026-10-16) for
testing that, keeps the table to its plain columns.

##### docs: when the method improves

The principle, let history guide us but not limit us, in `measuring-a-technique.md`.

##### feat: fixed and span auto measured

The three benches at fixed auto and span auto, tagged, the records in `records/`, and what
`analyze --compare` shows.

##### feat: auto inner is a span at least 16 wide

Auto chooses a span at least 16 counts wide around its own count, with the per-step level and the
band table's meaning under a span decided, if the measurement bears it out.

##### feat: how inner is chosen and shown closing

Closing out the cycle.

# References

[1]: #feat-how-inner-is-chosen-and-shown-opening
[2]: #feat-run-tables-show-inner-and-samples
[3]: #feat-a-span-graphs-each-runs-residuals
[4]: #docs-when-the-method-improves
[5]: #feat-fixed-and-span-auto-measured
[6]: #feat-auto-inner-is-a-span-at-least-16-wide
[7]: #feat-how-inner-is-chosen-and-shown-closing
