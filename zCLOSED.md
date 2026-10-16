# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record, in `TODO.md > ## Closed` before this file, and the cycles before the
rule in the frozen [notes/chores/](notes/chores) and [notes/done.md](notes/done.md).

## Closed

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
  nine commands rather than fifteen.
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
  a stub holding the binary's absolute path that asks it for every candidate, so it goes stale
  only when the binary moves, and the dev name takes a file of its own.
- The clap types live in `src/cli.rs`, and the commands not yet moved ran through a temporary
  external subcommand that parsed the old flat struct, so every rung left every command working.
  The groups' rung deleted it, and an external subcommand stays as the answer to an old word or a
  bench without `run`.
- `--verbose` is global, so every subcommand's help shows it, though only the run and `qualify`
  log: the bench child takes it before any subcommand.
- The Todo "Help per command word" is done by this cycle and retires with it: each command's `-h`
  is its own and a plain `-h` lists the commands a line each. Its "help all", every flag at once,
  was a "We think" and is not done, and its test that every command word has an entry is
  needless, clap holding each command's flags by construction.

#### Solution

Every command is a clap subcommand of the flags it owns, in `src/cli.rs`: `run`, `qualify`,
`analyze`, `linear-regression` (`lr`), `figures`, `describe-record`, and the groups `freq
read|pin|restore|suggest`, `setup [freq|uninstall]`, and `config init|update`. The run parameters
are one `MeasureArgs`, flattened into `run`, `freq suggest`, and `config init` and `update`,
and `qualify` takes the few it passes to its children. Clap now refuses another command's flag,
so the hand checks, `COMMANDS_HELP`, `COMMAND_WORDS`, and the flat struct are gone, and a bare
`iiac-perf` prints the bench list. An old word or a bench without `run` is answered with the line
to type. The help is clap's, a line per subcommand from its doc comment, and each subcommand's
`-h` shows its flags alone. Each value of a known shape completes, `clap_complete` is pinned
exactly, and the docs install the completion hook once as a file the shell loads on the first
Tab, the docs' command lines rewritten to the subcommand form.

#### Acceptance check

- Every old command word has a subcommand, and a bare `iiac-perf` prints the bench list.
- `iiac-perf figures --help` lists only `figures`' flags, and Tab after `iiac-perf figures --`
  offers only those.
- No "belongs to" check is left in `main.rs`.
- `vc-x1 validate` passes.

Passed at the closing, on the release build.

- Each old word answers: `qualify-environment` runs as the alias, `describe-record`, `analyze`,
  `linear-regression`, `lr`, `figures`, and `setup` run, and the other eight and `all` print
  their new line. A bare `iiac-perf` prints the bench list.
- `figures --help` lists `--trim-runs`, `--bench`, `--series`, `--show`, `--x-axis`, and
  `--out`, with the global `-v` and `-h`, and Tab after `figures --` offers the same.
- The one "belong" left in `main.rs` is a comment on `--compare`'s sides.
- `vc-x1 validate` passed.

#### Ladder

- [feat: the CLI is subcommands opening][1] (done)
- [feat: run is a subcommand][2] (done)
- [feat: the record commands are subcommands][3] (done)
- [feat: qualify is a subcommand][4] (done)
- [feat: freq, setup, and config are subcommands][5] (done)
- [feat: help and completion follow the subcommands][6] (done)
- [docs: command lines in the subcommand form][7] (done)
- [feat: the CLI is subcommands closing][8] (done)

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

Closing out the cycle: the acceptance check run and passed, the version at its bare form, the
record finalized and moved to `zCLOSED.md`, and the Todo "Help per command word" retired.

- Close-out shape: the trapezoid, named in wink's waiver, so Land reshapes the closing into the
  merge of the trunk and this ladder.
- Land waits for wink's go, outside the waiver: the sync, `iiac-perf-dev` renamed back to
  `iiac-perf`, the reshape, the fast-forward of `main`, the install, and the bookmark's deletion.
- What closing taught: a ladder that moves a command line piece by piece needs every rung to keep
  every command working. The temporary external subcommand did that here, at the cost of code
  that lived four rungs, and is the pattern for the next such move.

# References

[1]: #feat-the-cli-is-subcommands-opening
[2]: #feat-run-is-a-subcommand
[3]: #feat-the-record-commands-are-subcommands
[4]: #feat-qualify-is-a-subcommand
[5]: #feat-freq-setup-and-config-are-subcommands
[6]: #feat-help-and-completion-follow-the-subcommands
[7]: #docs-command-lines-in-the-subcommand-form
[8]: #feat-the-cli-is-subcommands-closing
