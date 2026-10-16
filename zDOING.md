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
  the bookmark forward to `main`. It waits again for [feat: the CLI is
  subcommands](#feat-the-cli-is-subcommands) (wink, 2026-10-16), and its `--figure` then lands in
  the `figures` subcommand's own arguments.
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

### feat: the CLI is subcommands

(wink, 2026-10-16) All fifteen commands are words in one positional of one flat `Cli` struct,
`analyze`, `figures`, `pin-freq`, and `init-config` among them, beside the bench names. So
`main.rs` checks by hand which flag belongs to which command ("--by and --compare belong to
'analyze'"), the positional's doc comment is a manual for every command, and the command list is
kept twice, in `COMMANDS_HELP` and `COMMAND_WORDS`. Each command becomes a clap subcommand that
owns its flags, so clap does the checking, the help, and the completion:

- `run [BENCH|FILE.md]...`, `all` a bench name, and a bare `iiac-perf` prints the bench list
- `qualify`, `analyze`, `linear-regression` (`lr`), `figures`, `describe-record`
- `freq read|pin|restore|suggest`, `setup [freq|uninstall]`, `config init|update`
- the measuring flags in one `MeasureArgs`, flattened into `run`, `qualify`, and `freq suggest`,
  `--verbose` global, and the flags of a fixed set of values `ValueEnum`s
- `clap_complete` pinned, since `unstable-dynamic` may break in a minor release, and the README
  showing the one-time file in `~/.local/share/bash-completion/completions/` in place of
  `source <(COMPLETE=bash iiac-perf)`

#### Deliberation

- Waiver (wink, 2026-10-16): "Permission granted to complete this trapezoidal cycle through all
  steps except the landing on main, go".
  - It covers every rung's work review, description review, and push, the bookmark's creation,
    and the hard stop after each push, opening through closing.
  - It does not cover Land: the sync, the rename back, the reshape, the fast-forward of `main`,
    the install, and the bookmark's deletion each wait for wink's go.
  - Stop and ask still holds on ambiguous input and on any deviation from this plan.
- The close-out shape is the trapezoid, named in the waiver.
- No bench names without `run` (wink, 2026-10-16): a bare `iiac-perf` prints the bench list as it
  does today, and `iiac-perf mpsc` is an error that points at `iiac-perf run mpsc`.
  - Clap has no default subcommand. The workaround, a flattened `RunArgs` beside the subcommands,
    puts bench names and subcommand names in one position again, the ambiguity this cycle removes.
- The groups are nested (wink, 2026-10-16): `freq pin`, not `pin-freq`, so the top level lists
  eight commands rather than fifteen.
  - `setup-freq` is `setup freq` and `suggest-freq` is `freq suggest`, the agent's choice in the
    plan wink approved.
  - `qualify-environment` is `qualify`, the old name an alias.
- `qualify` takes its own flags, not `MeasureArgs` as the plan said, the agent's refinement at its
  rung: the run parameters it does not pass to its children would be taken and dropped, the
  silent ignoring this cycle removes.
- Before figures (wink, 2026-10-16): that cycle's `--figure` then lands in the `figures`
  subcommand's own arguments, and its bookmark stays where it is.
- The version's minor advances, the plan's `0.29.0` that wink approved, since every command line
  written so far changes shape.
- Completion by a file bash-completion loads on the first Tab, not a line in `.bashrc`: the file is
  a stub that asks the binary for every candidate, so it goes stale only when the command's name
  changes, and the dev name takes a file of its own.

#### Solution

_Provisional._ A `Command` enum of the subcommands above, each a struct of its own flags, and
`main` dispatching on it, the hand checks of which flag belongs where deleted with the command
words. The help is clap's, a line per subcommand from its doc comment, and each subcommand's
`--help` shows its flags alone.

#### Acceptance check

- Every old command word has a subcommand, and a bare `iiac-perf` prints the bench list.
- `iiac-perf figures --help` lists only `figures`' flags, and Tab after `iiac-perf figures --`
  offers only those.
- No "belongs to" check is left in `main.rs`.
- `vc-x1 validate` passes.

#### Ladder

- [feat: the CLI is subcommands opening][1] (done)
- [feat: run is a subcommand][2] (done)
- [feat: the record commands are subcommands][3] (done)
- [feat: qualify is a subcommand][4] (done)
- [feat: freq, setup, and config are subcommands][5] (done)
- [feat: help and completion follow the subcommands][6] (done)
- [docs: command lines in the subcommand form][7] (done)
- [feat: the CLI is subcommands closing][8]

##### feat: the CLI is subcommands opening

The cycle's setup: the bookmark created and published, this block written, the version bumped and
the package renamed `iiac-perf-dev`.

##### feat: run is a subcommand

The skeleton: a `Command` enum, the measuring flags in `MeasureArgs`, and `run` taking the bench
names and the config file, a bare `iiac-perf` printing the bench list.

- The clap types move to `src/cli.rs`, so `main.rs` holds the dispatch and the run and the
  command line is read in one file.
- `MeasureArgs` is the run parameters, exactly the flags a config file has a key for, so
  `line_values` reads it alone and `--config`, a file to read rather than a key, stays each
  command's own.
- `--verbose` is the top level's, global, since the bench child takes it before any subcommand
  and every command may log.
- The words not yet subcommands reach `Legacy`, the flat struct they were parsed by, through an
  external subcommand, so every rung leaves each command working and the last group's rung
  deletes it. A first word that is no command is a bench, refused with the `run` line to type.
- A bare `iiac-perf` prints the bench list whatever the config says, and `run` with no names
  runs the config's `benches`, else prints the list: the config's benches are a run's, and a
  bare line runs nothing.
- The run itself is one function, `bench_run`, which `run` and the legacy `suggest-freq` both
  call, so the later `freq suggest` changes only its caller.
- qualify's children run `run min-now`, the one place the binary calls itself with a bench.

##### feat: the record commands are subcommands

`analyze`, `linear-regression`, `figures`, and `describe-record` take their own flags, and the
checks of which flag belongs to them go.

- The three that read records share `RecordPaths`, the paths, required, and `--trim-runs`, so
  clap refuses a line naming none and a bad trim the same way in each.
- The flags of a fixed shape parse in clap: `--trim-runs` by `Trim::parse` with its default
  shown, `--show` by `Show::parse`, `--overhead` refusing a negative or non-finite time, and
  `--x-axis` a `ValueEnum`, so its help lists `time` and `block` from their doc comments and
  `XAxis::parse` goes.
- `--compare` stays parsed in `main`, since its hint for records named where the sides belong is
  several lines a clap error would prefix with its own.
- `--series` is now `figures`' own, beside `init-config --from-record`'s, two flags of one name
  rather than one flag two commands read.
- The help's long text for each is its doc comment, the after-help block losing their entries.

##### feat: qualify is a subcommand

`qualify` takes the flags it passes its children and its own, `qualify-environment` an alias.

- `qualify` takes `QualifyArgs`, the flags it spends or passes to its children, `-d`, `--runs`,
  `--run-sleep`, `--pin-cpus`, `--settle-time`, and `--print-only`, not the whole
  `MeasureArgs`, whose other flags it would take and drop.
- Its defaults move into clap, so the help shows the one-second child, the ten runs, and the
  sleep of 0 that its doc comments explain, and `--runs` and `--run-sleep` no longer say what
  they mean for qualify in a run's help.
- It reads the default config for `verbose` alone, as it did, and drops `--config`, which named
  that file only for that key. Its children read their own.
- Its alias is visible, so the old name is in the help and the acceptance test keeps working, and
  it now runs `qualify`.
- In the agent sandbox a child fails on the read-only `cpufreq` files a pin writes, the landed
  binary alike, so the run here stops at its first child. That a qualification cannot pass
  `--pin-freq=no` on to its children is the old behavior, unchanged.

##### feat: freq, setup, and config are subcommands

The clock and config words become the nested groups `freq`, `setup`, and `config`.

- `freq read|pin|restore|suggest`, `setup [freq|uninstall]`, and `config init|update`, each
  variant or struct holding its own flags, `--apply` global under `setup` so it reads on the
  plain command and on each part.
- `config init`'s three starts are one clap group, `--series` requires `--from-record`, and
  `--backup` and `--overwrite` conflict, where `--backup` used to win in silence.
- `config update` takes no start at all, so the refusal of `--from`, `--config`, and
  `--from-record` on its line goes with the flags.
- With the last words moved, `Legacy`, `COMMANDS_HELP`, and `COMMAND_WORDS` go in this rung, not
  the next, since nothing was left in them.
- A first word that is no subcommand is answered with the line to type: an old word with its
  new form, `pin-freq 3800` getting `freq pin 3800`, a bench with `run` before it, anything else
  with `-h`. A bench list naming a subcommand is refused by clap's own list of names.
- Every message, doc comment, and comment naming an old word says the new form, the words being
  the same length so no line re-wraps. The comment of an installed udev rule says the new form
  too, and the one test of an older rule's comment keeps the old one, since that is what a host
  carries.

##### feat: help and completion follow the subcommands

Each argument gets its own completer, `clap_complete` is pinned, and the README says how to
install the completion.

- Each subcommand already offered only its own flags, and paths completed as paths, both from
  clap. What was added is a completer for each value of a known shape: `--pin-cpus` the profiles
  the host's config declares and `nearest`, `all`, and `unpinned`, `--config` the config files,
  `--benches` and `figures --bench` the bench names, `--show`, `--compare`'s keys, and the
  `[freq]` words for `freq pin` and `--pin-freq=`.
- A comma list completes its last element after the ones typed, so `smt,c` offers `smt,ccx`.
- The hook the binary prints holds its absolute path, not its candidates, so installed once as a
  file bash-completion loads on the first Tab it serves every later build, and goes stale only
  when the binary moves. Checked here under a scratch `XDG_DATA_HOME`: no completion at shell
  start, and `figures --s` completing to `--series` and `--show` after the load.
- `clap_complete` is pinned at `=4.6.7`, the version the lock held, since `unstable-dynamic` is
  the protocol between the installed hook and the binary.
- `docs/usage.md`'s Shell completion section installs the file for bash, zsh, and fish, and
  keeps the rc-file line for the shells without a loader, and the README shows the bash file.

##### docs: command lines in the subcommand form

The command lines in the README and `notes/` are rewritten to the subcommand form, the frozen
history left as it is.

- A script did the mechanical part, in code spans and fences only: an old word to its new form,
  and `run` before a bench, a run flag, or a config file after the binary's name, never before a
  subcommand, a version, `-h`, or a redirect. The old words were renamed in prose too.
- By hand: the synopses, which open with `[BENCH...]`, the usage doc's "Command words" heading,
  now "Commands", and its sentence saying every command is a subcommand, `-h` listing them and an
  old word answered with its new form, and the Flags list, now `run`'s and the run parameters'.
- Left as written: the frozen history, `zCLOSED.md`, the messages, a quoted rung title of
  0.23.0-6, `iiac-perf cpus`, a command a Todo proposes, and "pin-freq" where it names the flag.
- The Todo "A command word that decodes a bench name" is retitled "A subcommand ...", no link
  citing it. "Help per command word" is what this cycle does, and the closing retires it.
- The table rows over 100 columns were that long before, the renamed words being the same
  length, and one line of `iiac-perf.example.md` that `run` pushed over is re-wrapped.

##### feat: the CLI is subcommands closing

Closing out the cycle.

# References

[1]: #feat-the-cli-is-subcommands-opening
[2]: #feat-run-is-a-subcommand
[3]: #feat-the-record-commands-are-subcommands
[4]: #feat-qualify-is-a-subcommand
[5]: #feat-freq-setup-and-config-are-subcommands
[6]: #feat-help-and-completion-follow-the-subcommands
[7]: #docs-command-lines-in-the-subcommand-form
[8]: #feat-the-cli-is-subcommands-closing
