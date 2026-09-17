# Todo and cycle record

This file contains near term tasks with a short description and reference links to more details.
Its shape is [Todo format](agent-data/notes.md#todo-format).

## Continuation notes

Where the agent was, for the agent that comes next: working copy state, the step in flight, an
open question. Ephemeral, never a record. Written before a restart or when a session is about to
lose context, read first at acquaint, acted on, each fact filed into its home or its bullet kept, and
the rest reset to `_None._` by the reader.

_None._

## In Progress

A cycle's record has one home at a time, and while the cycle runs this is it. The block's
shape is the specimen in [cycle-model.md](agent-data/cycle-model.md), and the rules are in
[The In Progress block](agent-data/notes.md#the-in-progress-block).

### feat: a run is a config file

#### Problem

A comparison across hosts or days is a bench list and a dozen knobs typed as flags each time, so
two runs meant to be identical differ by whatever a hand forgot (wink, 2026-09-05, after the
placement sweep). The loader knows two fixed locations and no flag names a file. Ten run
parameters have a flag and no key: `--total-duration`, `--samples`, `--inner`, `--pin-cpus`,
`--record`, `--tag`, `--no-env-probe`, `--no-inhibit`, `--ticks`, and `--verbose`. A project-local
`[freq]` replaces the XDG one whole while `setup` checks only the XDG file, so a run in such a
directory ignores what `setup` wrote, and a limit-less local table refuses every pin there (found
2026-09-14, with the 3900X's untracked `iiac-perf.md` in exactly that shape). The first experiment
that wants one definition on two hosts is waiting on all three: whether the 3900X's unpinned shift
follows its clock.

#### Solution

Every run parameter gets a config key, `init-config` writes a starting file, `--config NAME`
makes one file the run's only source of run keys with the flags still winning, and `setup`,
renamed `setup-freq`, and a pin's refusal say which file's `[freq]` applies. The clock experiment
is then written as a tracked config under `configs/` and run from it on both hosts, its finding
going to the report guide.

#### Acceptance check

On each host, `iiac-perf-dev --config configs/clock-shift.md` with nothing else on the line runs
the experiment and records it. The `Config:` list names `configs/clock-shift.md` as the source of
every key the file sets, and the two hosts' lists agree key for key. The same line with
`--run-sleep` or `--pin-freq` added shows that flag as the source of its key and the file for the
rest. The report guide says whether each run's mean follows its record's `clock_khz`, and whether
the sleep moves a run's reading, on the 3900X and the 7600x.

#### Ladder

- [feat: a run is a config file opening][1] (done)
- [feat: a config key for every run parameter][2] (done)
- [feat: init-config writes every key, commented out][3] (done)
- [feat: --config names the run's file][4] (done)
- [feat: init-config takes the line's values][5] (done)
- [feat: update-config rewrites a config in place][6] (done)
- [feat: a config file as a bench argument][7] (done)
- [feat: init-config writes the run this host would make][8] (done)
- [feat: iiac-perf.md is found up the parents][9] (done)
- [refactor: setup is setup-freq][10] (done)
- [feat: setup-freq checks the project-local freq table][11] (done)
- [docs: the README's guide to config files][12] (done)
- [docs: the clock experiment, run from its config][13]
- [feat: a run is a config file closing][14]

#### Deliberation

- Three entries, one cycle: the `--config` entry, the `setup` shadow warning, and the clock
  question run together (wink, 2026-09-17).
  - The shadow warning is in because a third file layer makes "which table applied" harder to
    see, and the refusal's source tracking is the same code the `Config:` list already has.
  - The clock question is the first use rather than a rung of code: it wants alternating
    invocations of one definition on two hosts, which is what the cycle builds.
- A `[freq]` in a `--config` file was to be an error, not ignored (wink, 2026-09-17), and is now
  allowed like any other table (wink, 2026-09-17, at the `init-config` rung's review).
  - The refusal was the one exception to "the nearest file that sets it wins", and the exception
    kept confusing us. `[freq]` already worked that way between the XDG and local files.
  - The table still replaces whole, and `freqctl` still checks it against the hardware.
  - The risk accepted: a shared file carrying one host's clamp passes the range check on another
    host, whose restore then lands on the wrong clamp. The freq-table rung covers it, by naming the
    table's file and by warning at pin time when the declared state is not the live one.
- `--config NAME` is searched for, not only opened (wink, 2026-09-17): a relative NAME is tried in
  the current directory, each parent, then the XDG directory, the first found winning.
  - It serves a tree of bench directories sharing a parent's config by name, a nearer file of
    the same name overriding it.
  - A stray file applying everywhere, the worry in [Config search up the parents, arms, and a
    pin's boost](#config-search-up-the-parents-arms-and-a-pins-boost), is weaker for a file that
    loads only when named. The automatic `iiac-perf.md` search stays there.
  - Sharing is one file found from several directories, not a config that includes another.
- A relative `record` resolves against the current directory, as the flag does (wink, 2026-09-17).
  - A tracked config then carries no host's paths, and the alternative, relative to the config
    file, would write records into the repo's `configs/`.
- Keys before the flag: the keys rung lands first, so the `--config` rung's fixture is a complete
  run and its test is the acceptance check in small.
- Numeric pins only: `pin_cpus` takes what `--pin-cpus` takes today, numbers or a `[profiles]`
  name.
  - The clock experiment is unpinned or pinned by flag, so it needs no portable pin.
  - Names that resolve from the topology are the next cycle, [Placements by name and a cpus
    command](#placements-by-name-and-a-cpus-command), whose last rung writes the base configs.
- Command words get no key: `--print-only`, `--as-config`, `--apply`, `--uninstall`,
  `--list-benches`, and `--child-spec` say what to do, not how a run is shaped, and `--config`
  names the file a key would live in.
- `--config` stands alone for run keys, rather than layering over the XDG and local files (wink,
  2026-09-17, at the keys rung's review).
  - Layered, host A's `blocks = 10` reaches a run that host B's file does not touch, so one
    definition gives two runs and the acceptance check's "agree key for key" fails unless the
    file sets every key.
  - The host's files still give `[freq]` and `[profiles]`, the host's own facts.
- `init-config` updates a file by regenerating it, `--from OLD`, never by editing in place (wink,
  2026-09-17).
  - A commented key is invisible to the parser, a bare key appended after a table header lands
    in that table, and the file holds its author's prose.
  - The values carried are the file's own. Writing a line's values as a config was kept as a
    Todo entry, and became the rung `feat: init-config takes the line's values` once wink typed
    that line on the 7600x and the flags were ignored.
- `init-config` copies the host's run keys into the file it writes, where it was to copy none
  (wink, 2026-09-17, after `q1.toml` ran without the `block_warmup` its line had).
  - The rule against copying kept a file the same on every host, and cost the thing the command
    is for: the file was not the run the line made.
  - wink's terms: it is not the perfect answer everywhere, and it meets the expectation for that
    run on that host. `--from` and `--config` remain the way to start from something else.
- A starting config is an inserted rung (wink, 2026-09-17): a command word and `setup` both
  write it, with the prose. It runs right after the keys rung, while the list of keys is fresh,
  and the `--config` rung can use the generated file as its fixture.
- Left out, and kept as [Config search up the parents, arms, and a pin's
  boost](#config-search-up-the-parents-arms-and-a-pins-boost): the parent-directory search, a
  config with two arms, and a boost option for a pin. An A/B is two configs for now.
- Tags are a `[tags]` table, for the whole run (wink, 2026-09-17, at the keys rung).
  - wink's concern: a run-level tag has no scope, and the tags expected first are about the spsc
    and mpsc benches. Per-bench tags are kept as [Tags scoped to a
    bench](#tags-scoped-to-a-bench), to be shaped by how these get used.
- On/off keys carry positive names, `env_probe`, `inhibit`, `ticks`, `verbose`, the names the
  `Config:` list already printed, and the line undoes a file with `--flag=no` (wink, 2026-09-17).
- `--as-config` is left alone: it is `read-freq`'s, and the first draft's claim that it prints
  the new keys was a slip (wink, 2026-09-17).
- A waiver, from wink (2026-09-17, at the review of `feat: iiac-perf.md is found up the
  parents`): "permission to complete the cycle upto but not including the close-out".
  - It covers the work review, the description review, the per-push approval, and the stop after
    each push, for that rung and every rung after it up to the closing.
  - It does not cover the closing rung, the choice of close-out shape, or Land.
  - Stop and ask still holds: a deviation from a rung's plan, or an ambiguity, stops the work.
- A prose test runs for this cycle (wink, 2026-09-17): the agent thinks as usual, and everything
  it writes is in the plain version, in the conversation, in files, and in commit bodies.
  - The aim is to see whether the agent-repo's session files still hold the detail that the plain
    text leaves out.
  - Text written before the test began, this block's first draft included, is left as it was.

#### Ladder details

##### feat: a run is a config file opening

The cycle's setup commit: create and publish the bookmark, delete `## Closed`'s contents, merge the
three Todo entries into this block, write the entries the planning grew, bump the
version-of-record, and take the dev name.

- The planning turned one request into five cycles. This is the first. The others are Todo
  entries now, in the order they should run.
- One rule was bent, with wink's say-so: the opening was pushed without the description being
  shown first. The go named the bookmark and the opening's push and nothing else. Every later
  rung gets its description review and its own go.

##### feat: a config key for every run parameter

Ten flags had no key, so a file could not say what a command line can. Each now has one, resolved
through the same layering as the rest, so the `Config:` list shows where its value came from.

- The keys: `total_duration`, `samples`, `inner`, `pin_cpus`, `record`, `env_probe`, `inhibit`,
  `ticks`, `verbose`, and the `[tags]` table.
- `duration` and `total_duration` are one choice. A file that sets both is refused, and the
  nearer file's choice clears the other, so a host's `duration` does not fight a run's total.
  The list gains a `total_duration` row, and the `duration` row says when it was split from one.
- The four on/off flags take an optional `=yes` or `=no`, the bare flag meaning yes, so the line
  can undo a file. `pin_cpus`, `record`, `samples`, and `inner` have no undo from the line.
- Tags merge by key across the files, the line's `--tag` adds to them and wins on a shared key.
  `--tag` no longer demands `--record` on the line, since a file may name the record. A tag with
  no record from anywhere is still an error.
- The list's `tag` row is now `tags`, matching the key, which renames that key in a record's
  `config.params`. `verbose` is a new row.
- The config now loads before the logger starts and before the sleep inhibit, since both read a
  key. So `qualify-environment` now stops on a malformed config, where it used to ignore it.

##### feat: init-config writes every key, commented out

Nothing writes a starting config: `iiac-perf.example.md` lives in the repo, sets most of its keys,
and an installed binary cannot produce it, and `setup --apply` writes a `[freq]` table and nothing
else. An inserted rung (wink, 2026-09-17, at the keys rung's review).

- The binary carries one template in the markdown carrier, with the explaining prose, every key
  commented out at its default, and a sample value where a key has no default.
- `init-config [PATH]` prints it, or writes it to PATH and refuses to overwrite a file.
- `init-config --from OLD [PATH]` brings a file up to date by writing a fresh one: the template
  with every key OLD sets uncommented at OLD's value.
  - A missing key arrives with the template, and a stale one fails OLD's parse by name, as a
    load does, so nothing is dropped silently.
  - OLD is never touched, so a diff shows the change. The author's own prose is what it loses.
- `setup` creates a missing XDG file from the same template, the live `[freq]` filled in.
- The example file comes from the template or is tested against it. One test fails when a key is
  missing from the template, and one uncomments every line and parses the result.

What was done:

- The template is `iiac-perf.example.md` itself, compiled into the binary, so there is one file
  and nothing to keep in step. Every key in it is now commented out, and its tables moved after
  every top-level key.
- One rule makes the rest mechanical: inside a `toml` fence every `#` line is a key or a table
  header, and each table has a fence of its own. Explanation stays in the prose.
- `init-config [PATH]` prints or writes it, as TOML when PATH ends in `.toml`, the prose kept as
  comments. It never writes over a file.
- `--from OLD` sets each key OLD sets at OLD's value. A table OLD sets replaces the template's
  sample table whole. OLD goes through the loader's checks first, so a stale key stops it by name.
  A table that will not write back as TOML is an error that writes nothing, never a panic and
  never a file that silently lacks the table (wink, 2026-09-17, at the review).
- `setup` creates a missing XDG file from the template with the live `[freq]` set. A file that
  exists is appended to or left alone, as before.
- The template's `[freq]` is the bare commented header, with no sample values, since one host's
  are wrong on another (wink, 2026-09-17, at the review). `setup` and `--from` fill it.
- The README gains a walkthrough, a run from a config file, and the usage doc the command word
  and the on/off flags' `=no` form (wink, 2026-09-17, at the review).
- The test that holds the template complete names every field of the config, so a new key does
  not compile until the template carries it. It uncomments every key and checks each default
  against the built-in one.

##### feat: --config names the run's file

The loader reads the XDG file and the current directory's and nothing else. `--config NAME` names
the run's file, and the banner and the `Config:` list name the file by the full path found.

- An absolute NAME is taken as given. A relative one is tried in the current directory, each
  parent up to the root, then the XDG directory, and the first found wins. Not found is an
  error listing the places searched.
- At each place a NAME with neither extension is completed with `.md` or `.toml`, both present
  an error, as the other layers resolve their carrier.
- The run keys come from that file and the built-in defaults alone, the flags still winning.
- The `Config:` list's `files` line names the files highest priority first, where it named them
  in load order, the winner last (wink, 2026-09-17, from the first run on the 7600x). The record's
  `config.files` keeps load order, which its field dictionary states and records on disk follow.

What was done:

- `--config NAME` and the search are as planned above. A NAME that is a file as given is taken
  before any extension is tried, so `queue` finds a file named `queue` ahead of `queue.md`.
- The host's files are still read under a named file, then cut down to `[freq]` and `[profiles]`
  with their sources, before the named file is laid over them. So the `files` line lists all
  three, since all three were read.
- `pin-freq` and `restore-freq` take `--config` too, since they read `[freq]` through the same
  loader. `init-config` and `setup` do not read the layers and ignore it.
- The `files` line also shows the home directory as `~`, as the sources beside it already did.
- `[freq]` and `[profiles]` follow the one rule, the nearest file that sets it wins: the named
  file, then the local one, then the XDG one. `[freq]` replaces whole, as it does today.

##### feat: init-config takes the line's values

`init-config quick.toml --config iiac-perf --blocks 10 -d 0.5s --pin-freq` wrote the bare
template: every flag but `--from` was parsed and ignored without a word (wink, 2026-09-17, on the
7600x). An inserted rung, taking in the Todo entry `A run's resolved values as a config`, whose
open question, the spelling, this line answered.

- Every run flag on an `init-config` line sets its key in the new file, as typed. A bare
  `--pin-freq` writes `"pin_mhz"`, and `--benches` sets `benches`, the positional being PATH.
- `--config NAME` on that line starts from that file's values, found by the search, the flags
  winning over it. It is `--from` with a search, so giving both is an error.
- The XDG and local files' values are not copied in: the new file is to stand alone under
  `--config`.
- `-d` clears a `total_duration` the file gave and `-D` a `duration`, and a `--tag` adds to the
  file's `[tags]`.
- A flag that is not a run parameter is refused by name, never ignored.

What was done:

- The flags become a TOML table, keyed as the config keys them, and go over the start file's
  table before the template is filled, so `--from`, `--config`, and the flags share one path.
- Seconds reach the command parsed, so `-d 0.5s` is written `duration = 0.5`, not as typed. The
  span flags are strings and are written as typed.
- The finished text is checked as a load checks it before anything is written or printed, so a
  bad `--run-sleep` stops the command rather than the file's first run.
- Bench names cannot be positional on this line, so `--benches` sets `benches`.

##### feat: update-config rewrites a config in place

Changing a key in an existing config from the line is two steps, `init-config --from` to a new
path and a `mv`, because `init-config` never writes over a file. An inserted rung (wink,
2026-09-17, at the review of `feat: init-config takes the line's values`).

- `update-config FILE [flags]` reads FILE, sets the line's values over its own, fills the
  template, checks the result as a load does, and only then replaces FILE, written beside it
  first and renamed over it.
- `--backup` keeps the old file as `FILE.bak`, overwriting an earlier one. It is optional and
  off by default (wink). Without it the command says so when FILE holds prose or fence comments
  the rewrite loses.
- FILE is a path as given and must exist, never a searched name, and `--from` or `--config` on
  the line is an error, FILE being the start. No flags is the bring-up-to-date case.
- Its own command word, so `init-config` stays "a new file, never over an old one".

What was done:

- As planned. The rewrite shares `init-config`'s fill, so the two cannot differ in what a flag
  or a carried value becomes.
- "Something to lose" is exact, not guessed: the old text is compared with what the file would
  read as holding its values and nothing of its author's. A file that is already that loses
  nothing, and no note prints.
- The TOML carrier drops the prose and keeps the section headings and the keys (wink,
  2026-09-17, at the review, on reading `xyz1.toml`). As comments the prose and the commented
  keys both begin `# `, and a set key was lost among them.
- In the TOML carrier a section's keys run together, the blank lines being the headings' alone
  (wink, 2026-09-17, at the review).
- A commented-out key has no space after its `#`, `#blocks = 100`, in both carriers, and a
  comment has one (wink, 2026-09-17, at the review). It replaces the template's rule that every
  `#` line in a fence is a key, so a fence may hold a comment again.
- `init-config` replaces an existing PATH when asked (wink, 2026-09-17, at the review):
  `--backup` keeps the old file as `PATH.bak` and `--overwrite` keeps nothing. With neither it
  still refuses, its error naming both. It shares `update-config`'s staged write, and unlike it
  keeps none of the old file's values.
- A fault from the rung before, fixed here: clap refused `--benches` beside any positional, so
  `init-config PATH --benches a`, the line the README shows, could not run. `main` now makes
  that check on a bench line alone.
- The new text is staged as `FILE.new` beside the file and renamed over it. `--backup` on any
  other line is an error by name.

##### feat: a config file as a bench argument

Running a config is `--config NAME`, a flag for what wink expects to be the most common line. An
inserted rung (wink, 2026-09-17, at the review of `feat: init-config takes the line's values`).

- A positional ending in `.md` or `.toml` is the run's config, as `--config` with it, found by
  the same search: `iiac-perf queue.md`. No bench name ends that way, so the two never collide.
- Bench names beside it win over the file's `benches`, as names on the line already do.
- A bare name with no extension stays a bench: falling back to a config would turn a mistyped
  bench into a file lookup. `--config queue` is the form that completes the extension.
- Two config files, or one with `--config`, is an error. `init-config` and `update-config` keep
  their own `.md` positional, so the rule holds only when neither leads.
- Tab offers the current directory's `.md` and `.toml` files beside the bench names.

What was done:

- As planned: the file is moved out of the positionals into `--config` right after the line
  parses, so everything after it sees one form.
- Tab offers config files only once something is typed, and in the directory typed so far. With
  nothing typed, every README in the directory would crowd the bench names.
- A pattern with a dot in it, `zcr-.psc`, is still a bench: the rule reads the extension, not
  the dot.

##### feat: init-config writes the run this host would make

A line that worked, then the same line after `init-config q1.toml`, then `iiac-perf q1.toml`, gave
two runs: `block_warmup` was 2 ms from `iiac-perf.md` under the line and the default 0 under the
file, since the host's files were never copied in (wink, 2026-09-17, on the 3900X). An inserted
rung. It reverses that rule: the file is to be the run that line would make on this host.

- With neither `--from` nor `--config`, the start is what the XDG and local files set, layered as
  the loader layers them, the line's flags over it. `--from` or `--config` replaces that start,
  as a run under `--config` leaves the host's run keys out.
- Defaults stay commented out: they are the same on every host, and the file stays readable.
- `[freq]` and `[profiles]` are not copied (wink agreed, 2026-09-17). They are the host's, a run under the new file still
  gets them from the host's files, and a copied clamp is wrong on the next host.
- The command names what it took, a line per file, so nothing is inherited without a word.
- `--from /dev/null` is the bare template.

What was done:

- As planned. The host's files are found by the loader's own lookup, so the start cannot differ
  from what a plain run layers, and the rung that finds `iiac-perf.md` up the parents changes
  both at once.
- The layering follows the loader's rules: the nearer file wins, its choice of `duration` or
  `total_duration` clears the other, tags merge by key, and a file's `pin_freq = "no"` leaves a
  lower file's pin standing.
- The line naming what was taken leaves out a key the line set, since that value is the line's.
  Printing to stdout, the lines go to stderr, the file being the output.

##### feat: iiac-perf.md is found up the parents

The loader reads `iiac-perf.md` in the current directory and no higher, so a file moved to `~/`
was not found from `~/iiac-perf`, while `--config NAME` searches the parents. An inserted rung
(wink, 2026-09-17, from the first run on the 7600x).

- The project-local file is the nearest `iiac-perf.md` or `iiac-perf.toml`, the current
  directory first and then each parent, by the search `--config` uses.
- The search stops at the first found and merges no further level, so a file high in the tree
  is a fallback, never a layer under every directory below it.
- The `files` line names it by its full path, so what applied is never hidden.

What was done:

- As planned, in the loader's one lookup, so a plain run, `init-config`'s start, and the files
  a named config still reads for `[freq]` all find the same file.
- A file in the current directory keeps its bare name, `iiac-perf.md`, in the `files` line and
  the sources, as before. Only one found in a parent shows its full path.
- The search is its own small loop rather than `--config`'s, which also tries the XDG directory
  and completes extensions, neither of which a fixed name wants.

##### refactor: setup is setup-freq

`setup` writes the `[freq]` table and installs the permissions a pin and a restore need, both the
clock's, and with `init-config` making config files its bare name claims more than it does. An
inserted rung (wink, 2026-09-17, at the `init-config` rung's review).

- The word becomes `setup-freq`, beside `read-freq`, `pin-freq`, `restore-freq`, and
  `suggest-freq`, in the help, the hints, the docs, and the template's prose.
- No alias for the old word: we are the only users.
- It still creates a missing XDG file from the template, and its printed plan shows the `[freq]`
  part alone rather than the whole file.

What was done:

- The word is renamed in the help, the completion list, every hint and error, the udev rule's
  comment, the docs, and the template's prose. The source module keeps the name `setup`, which
  nothing outside the code sees.
- The rule file's comment changes, so a host that ran `setup --apply` before carries the old
  wording until `setup-freq --apply` rewrites it. The rule's effect is the same.
- For a missing file the plan prints one line saying the new file is the starting config, then
  the `[freq]` section, where it printed all of the template.

##### feat: setup-freq checks the project-local freq table

A project-local `[freq]` shadows the XDG one without a word. `setup-freq` checks the table that
applies in the current directory and says when it shadows the XDG declaration, and a pin's or a
restore's refusal names the file its `[freq]` came from.

- A pin compares the declared steady state with the live one as it engages, and when they differ
  prints one line: the restore will move the host to the declared state, and the file it came
  from. A warning, not a refusal. It is what catches a shared file carrying another host's clamp.

What was done:

- As planned, all three parts. `setup-freq` runs both checks and reports both, so a failure in
  the XDG file does not hide one in the table that applies here, and it fails when either does.
- The pin's warning is read before the pin changes the live state, and is silent when the live
  clamp is already `min = max`, another pin's state and not the host's.
- Checked against the case the problem statement names: a local table with no clamp limits now
  fails `setup-freq` by file, and `pin-freq`'s refusal ends "The [freq] in use is from
  iiac-perf.toml."
- Left as it is: the refusal's hint still says `setup-freq` writes the limits, which is true of
  the XDG file and not of a local one. The line naming the file is what points at the fix.

##### docs: the README's guide to config files

Each rung added a line or two to the README, which keeps it right and leaves it thin (wink,
2026-09-17, at the `update-config` rung's description review). An inserted rung, placed after
the last rung that changes a command, so the guide is written once.

- The model, once: the files read and their order, the nearest file that sets a key winning,
  what changes under a named config, and why `[freq]` is the host's.
- The three commands side by side, `init-config`, `update-config`, and running a config, with
  `--from`, `--backup`, and `--overwrite`, and which to use when.
- The two carriers, the `#key` and `# comment` rule, and why the TOML form holds no prose.
- A worked example from a command line to a file run on two hosts, one key changed, and the
  `Config:` list read to confirm it.
- The failures a user meets, each with the message they see.

What was done:

- The README's walkthrough becomes a section, `Config files`, of five parts in the planned
  order: which files are read, the commands, the two carriers, a line to a file on two hosts,
  and when it stops.
- The layers and the commands are tables, since each is a lookup, which to use when, rather
  than something read through.
- Every message in the last part was produced by running the case, so the table quotes what a
  user sees. The two from the freq-table rung are in it.

##### docs: the clock experiment, run from its config

Two unpinned 3900X invocations of `min-now` a minute apart read 22.8 and 22.5 ns while two pinned
with `--pin-freq --run-sleep 1s` both read 26.3 ns, the sleep and the pin changed together (wink,
2026-09-15). The experiment becomes `configs/clock-shift.md`, run on both hosts, and the guide
gets what it shows.

- Alternating unpinned invocations with a record, each run's mean against its `clock_khz`: a
  shift that follows the clock names the cause.
- The same at `--run-sleep 0` against the `1-2s` default, unpinned and pinned, to see whether the
  sleep moves a run's reading at all.
- On the 7600x too, whose unpinned `min-now` pair agreed at the display's precision then.

##### feat: a run is a config file closing

Closing out the cycle.

## Waiting

Important work that cannot start yet. Each entry names what it waits on and its rank once
unblocked, and every opening checks the conditions.

_None._

## Todo

Entries are in priority order, the first highest, and reprioritizing moves the entry. The
long-tail backlog is in [todo-backlog.md](notes/todo-backlog.md), and deeper detail lives in
the frozen `notes/chores/` design subsections, linked by `[N]` refs.

### Placements by name and a cpus command

`--pin-cpus` takes cpu numbers, which differ by host, so a config that pins is a config per host,
and nearly every pinned record here sits on `0,1`, the busy end of both hosts (wink, 2026-09-17,
at the planning of `feat: a run is a config file`). zc-ring-x1 counted the kernel's per-cpu work
on both and decided where a measurement belongs (2026-09-16, its `notes/ring-buffer-design.md`,
"Measurement placements: the base cpu and its partners"): the idlest-cpu search breaks ties
toward the lowest number and a wakeup scans the L3 ascending, so short work piles onto the low
cpus. The 3900X's first CCX carries ten times the timer ticks of its last, and the 7600X's cpu 0
five times the reschedules of cpu 5. Our own pass agrees: `--pin-cpus 2,3` on the 7600x read 2 ns
faster and tighter than `0,1` (in [How often each pinned level comes up on the
7600x](#how-often-each-pinned-level-comes-up-on-the-7600x)).

- a command that lists the cpus: `iiac-perf cpus` prints a row per cpu, its core, its sibling,
  and its L3 group, the kernel's counts from `/proc/interrupts` and `/proc/softirqs` (timer ticks,
  sched softirq, RCU softirq, function calls), and the labels, which cpu is the base and which
  its `ccx`, `x-ccx`, and `smt` partner. No root is needed
- quietness is a rate: the counts are since boot, so they carry whatever ran in that uptime, which
  we think is the 7600X's busy cpu 7. The default is the count over the uptime, and `--for 10s`
  samples a delta, what the host is doing now
- `--pin-cpus` and `pin_cpus` take `ccx`, `x-ccx`, or `smt` as well as numbers (wink,
  2026-09-17), with `--base-cpu N` as the demo has it. The rule is zc-ring-x1's: the default base
  is the last core's primary cpu, partners prefer a primary cpu and among those the highest
  number, and the SMT partner is the base's sibling. That gives `11,10`, `11,8`, and `11,23` on
  the 3900X and `5,4` and `5,11` on the 7600X, the demo's pairs, so a table from here and one from
  the demo sit on the same cpus
- the names resolve by the rule, never by the measured counts: a name means the same cpus every
  day, and the command is the evidence the rule is checked against. A pin that followed the
  counts could differ between two runs of one config
- a name the host cannot satisfy refuses the run, `x-ccx` on the 7600X's one L3, rather than
  falling back
- a pool of more than two needs a rule too, the mpsc v2 pair running over a pool: we think `ccx`
  at four is the base's L3, primaries first, going down from the base, decided at the opening
- the record and the banner carry the name and the cpus it resolved to
- this takes the auto-profiles bullet of [Topology-aware pinning and lCPU
  terminology](#topology-aware-pinning-and-lcpu-terminology), whose `--pin smt`, `--pin llc`, and
  `--pin xllc` become these names, built-in beside the `[profiles]` a file declares
- the check before the old pins are dropped: the pinned 20-run `zcr-mpsc-v1-2t` command at `4,5`
  on the 7600x against its `0,1` and `2,3` passes, and the same three on the 3900X
- the last rung writes the base configs under `configs/` and runs [Re-record all on the 7600x
  across processes](#re-record-all-on-the-7600x-across-processes) from one, so that entry closes
  with this cycle. The acceptance check is one config, unedited, running on both hosts and
  landing on the demo's pairs

### Re-record all on the 7600x across processes

The 7600x's `all` rows were recorded one process for every bench, so each row carries whatever
placement that process drew (split from `feat: CI95 and LSC across processes` at its opening, the
cycle making each bench its own runs). The first use of the cycle's runs.

- `all` with `--record` into a directory that stays, whose records' host block starts the
  cross-host comparison
- a run of the mpsc v1 pair there, whose `all` rows are the renamed v0 rows
- run from a base config as the last rung of [Placements by name and a cpus
  command](#placements-by-name-and-a-cpus-command), on the quiet end rather than `0,1`

### Ring geometry on the line for the zcr benches

The zcr benches fix `CAPACITY` at 8 and `SEGMENTS` at 2 as constants that size `[u8; N]` region
types, so no run can ask for a depth or a segment count, and both the one-way benches and the
segment switch sweep want them on the line (wink, 2026-09-17, at the planning of `feat: a run is
a config file`). Split out of [One-way zcr benches, producer-only and
burst](#one-way-zcr-benches-producer-only-and-burst), whose "depth as a run knob" bullet this is,
because it changes every existing zcr bench and can be checked against numbers we already have.

- `--depth` and `--segments` for the spsc and mpsc benches, each with a config key, since by
  then every flag has one
- the regions become runtime-sized line-aligned allocations, leaked as today
- a ring that is not segmented refuses `--segments` rather than ignoring it, and a depth or a
  count the ring's `init` would reject is an error with the flag's name in it, not an `expect`
- the children get both on their command line like the rest, and the record carries the geometry
- the acceptance check: the round-trip spsc v3 and mpsc v2 at one segment, the no-switch baseline
  the `SEGMENTS` comment promises, against v2 and v1 at the same depth, and every zcr bench at
  the default geometry within `LSC runs` of its reading before the change

### Tags scoped to a bench

`[tags]` and `--tag` mark every record of a run alike, with nothing that says which bench a tag
is about, and the tags we expect first are about the spsc and mpsc benches (wink, 2026-09-17, at
the `tags` decision of `feat: a run is a config file`). Wait to see how the run-level tags get
used before building this.

- a `[[bench_tags]]` list, each entry a `bench` and a nested `tags` table, so no tag name is
  reserved and an unknown key is still an error
- `bench` matches as a bench name on the line does: exact, then prefix, then regular expression.
  A later entry wins on a shared key, and a bench tag wins over a run tag
- a value the bench knows itself, its ring depth or segment count, belongs in the bench's record
  rather than in a tag someone types, which [Ring geometry on the line for the zcr
  benches](#ring-geometry-on-the-line-for-the-zcr-benches) already plans
- open: whether the line gets a form too, since [Config keys stay
  CLI-settable](#config-keys-stay-cli-settable) asks for one

### One-way zcr benches, producer-only and burst

Every two-thread zcr bench is a round trip with one message in flight, so nothing here measures
what an ISR-to-thread connection costs: a producer that cannot wait, a consumer that trails a
burst, and a boundary crossed inside a burst once the ring is segmented (wink, 2026-09-08, after
the demo's depth sweep showed v2 winning every streaming placement on both hosts while the round
trip, three interleaved runs per cell, shows it 20% slower than v1 on the 3900X's same-L3 pair and
5% faster on the 7600x's). Two benches over each ring version, shaped by what the field settled on,
with a depth knob that doubles as the segment size once zc-ring-x1's segmented queue exists.

- **producer-only**: the step is one non-waiting reserve plus commit, the `|_| false` closure, a
  worker drains on another CPU with a spin, and Full is counted and printed beside the row, never
  waited on. DPDK's enqueue-burst and full-enqueue costs in one bench, `--inner B` making it a
  burst, and the quantile ladder giving the tail an ISR deadline is measured against
- **burst cost**: the step sends B non-waiting messages then waits for the worker's
  acknowledgement of the last, JCTools' QueueBurstCost, timed first send to last receive, B the
  axis at 1, 8, and 64. B above the depth is the overflow edge whose Full count sizes a segment
  pool, and B above the segment size crosses a boundary inside the burst, the consumer's side of
  it, where JCTools' linked queues lost
- **depth as a run knob**: both benches want depth on the line, the demo's finding living on that
  axis, and the knob is [Ring geometry on the line for the zcr
  benches](#ring-geometry-on-the-line-for-the-zcr-benches), which runs first. Over the segmented
  v3 the same knob is the segment size M, M=1 the boundary's worst case, and the sweep over 1, 2,
  4, 8, and 64 the
  amortization check: fit cost against base plus boundary over M, and the M=1 point on or off the
  line says whether that path has a cost of its own, the cold four-line header being the suspect
- the histogram is the second view: at M=64 the boundary is 1.6% of messages and lands in the n2
  band, at M=8 it is 12.5% and lands at p90, so one run at a realistic M shows the boundary cost in
  place
- the paced one-way bench, the Disruptor's latency test with a TSC stamp in the message and a
  consumer-side histogram, answers what latency the thread sees at a given interrupt rate. It
  needs pacing and a consumer-side probe, which the probe-style benches have the bones of, and is
  a second entry once these two land
- **the producer's side against the consumer's, same thread** (wink, 2026-09-16, after `feat:
  spsc v3 and mpsc v2 benches`): a 1t bench that reserves and commits N times, then reserves and
  releases N times, to see which side pays v3's same-thread gap over v2, three to six times, and
  mpsc v2's over v1, 1.8 times
- names follow the pair convention and are decided at the opening, so the prefix runner covers a
  version's four benches with one word
- ranked ahead of the rest: the numbers feed zc-ring-x1's segmented queue, which may start as a
  v3, and no bench today separates the producer's side of the seam from the consumer's, which the
  cross-host reversal needs. Behind the rename and the partition merge since 2026-09-11, so the
  new benches are written in the merged hierarchy's words

### Segment switch cost

The demo's segment stress prices one switch as a difference of two means at equal capacity, 32x1
against 1x32, with no interval and a consumer whose lag scheduling decides: 16 ns for spsc-v3 and
58 for mpsc-v2 streaming across the 7600X's same-L3 pair, 140 and 270 to 290 cross-CCX on the
3900X, against 3 to 7 and 7 to 13 single-threaded (zc-ring-x1, 2026-09-15). "Expensive" is a
claim about a technique, and this app makes it one with an interval, per host and placement, and
then says which lines a switch moves, so zc-ring-x1's candidates in its `Cheaper segment switches`
entry are chosen and then checked by number.

- the shapes are the one-way entry's burst and producer-only benches over v3 and v2, with
  `--segments` and `--depth` on the line, and the lag made deterministic: at depth 1 a consumer
  holding one slot forces a switch per message, where the demo's lagging rows leave it to the
  scheduler and report no time
- the sweep at equal capacity, 32x1, 16x2, 8x4, down to 1x32, has switches per message of one
  over depth, so per-message cost against that is a line: the slope is the switch cost, the
  intercept the no-switch cost, each with a CI95, and a depth-1 point off the line says the
  first switch has a cost of its own, the cold segment header
- transfers per switch without hardware counters: the slope at cross-CCX less the slope at same-L3,
  over the per-transfer cost the spsc handoff already gives at each placement. The design note's
  informal count is one for spsc-v3 and three to four for mpsc-v2, and this makes it a table by
  placement on both hosts
- that table is the acceptance check for each candidate: a cycle per candidate in zc-ring-x1, each
  closed by rerunning this sweep, the reply to its thread carrying the numbers
- runs pinned with the frequency held, since the cross-CCX slope on the 3900X is inside the
  unpinned shift `feat: a run is a config file` is chasing with its clock experiment
- it runs after [Ring geometry on the line for the zcr
  benches](#ring-geometry-on-the-line-for-the-zcr-benches) and [One-way zcr benches, producer-only
  and burst](#one-way-zcr-benches-producer-only-and-burst), which give it `--segments`,
  `--depth`, and its shapes, and from a config on the pairs [Placements by name and a cpus
  command](#placements-by-name-and-a-cpus-command) names
- the two-level runs of `feat: spsc v3 and mpsc v2 benches`, 11.8 or 12.8 ns on the 7600X and
  17.0 or 19.2 on the 3900X, are read here as a placement finding of their own
- the 7600X's SMT pair, where v3 lost across threads with runs from 51 to 67 ns, is rerun at ten
  runs, and at the 3900X's SMT pair, before it goes in a message
- zc-ring-x1 has not been told the findings of `feat: spsc v3 and mpsc v2 benches`, on purpose:
  they are a symptom so far, and the message waits until it carries a diagnosis and the commands
  that reproduce each table on either host. What there is sits in the report guide's spsc v3 and
  mpsc v2 paragraphs

### Measure whether code layout moves the level

A rebuild moved `zcr-mpsc-v1-2t` from 67.9 to 60.8 ns while `min-now` held to 0.01 ns (the evidence
in `feat: CI95 and LSC across processes`), so the binary's layout may set a bench's level as a
process's placement does (wink, 2026-09-12). Fat LTO with one codegen unit, and LLVM function and
block alignment, against the default profile, each built twice around a trivial unrelated change,
interleaved, to see whether layout stops moving the level. Wants that cycle's runs first, so the
placement level is measured rather than confounded.

### How often each pinned level comes up on the 7600x

Fresh pinned processes on the 7600x landed `zcr-mpsc-v1-2t` at 60.6 ns three times and 76.0 once, four
runs too few to say how often each level comes up (the evidence in `feat: CI95 and LSC across
processes`). Twenty unpinned runs there read 63.7 to 64.8 ns with one run at 66.5, twice, 64.4 and
64.3 ns agreeing (wink, 2026-09-15).

- the same command pinned, `zcr-mpsc-v1-2t --runs 20 -d 1 --run-sleep 250ms-750ms --pin-cpus 0,1`,
  twice back to back with `--record`, counting the runs at each level
- the counts are the input "Name what sets a process's level" needs: a level that comes up one run
  in four is a different experiment from one that comes up one in twenty
- first pass, wink, 2026-09-15, 0.28.14-10, the command above with `--pin-freq` at 4701 MHz and no
  `--record`, twice: 72.0 and 71.9 ns, `LSC runs` 0.4 ns each, so the two agree. All 40 runs read
  69.8 to 73.1 ns, no run near the 60.6 or 76.0 ns levels the 0.28.11 build showed. The build
  changed as well as the clock pin, so this points at "Measure whether code layout moves the level"
  as much as at placement
- the pinned spread was the wider one: run stdev 0.6-0.7 ns against 0.3-0.5 unpinned, and the
  outlying runs sat low (69.8, 70.2, 71.0) with `CI95 blocks` of 0.3 ns against 0.1-0.2 for the rest,
  where the unpinned outlier sat high with tight blocks. We think CPU 0, which carries the kernel's
  housekeeping, adds that spread: a pass on `--pin-cpus 2,3` would show it
- the pin-cpus and pin-freq effects are not separated: the pinned level, 72 ns at 4.70 GHz with boost
  off, sits 12% above the unpinned 64.3 ns, and boost's 5.46 GHz ceiling is 16% above the pin. We
  think most of the gap is the clock, but the unpinned runs' delivered clock was not shown, and a pass
  with each pin alone would split them
- second pass, wink, 2026-09-15, the same on `--pin-cpus 2,3`, twice: 70.0 ns (`LSC runs` 0.4) and
  70.4 ns (`LSC runs` 1.0), agreeing by the larger LSC. The bulk sat 2 ns faster than on `0,1` and
  tighter, 69.1 to 70.3 ns with `CI95 blocks` of 0.1 throughout, which fits CPU 0 adding the spread.
  Three runs of 40 sat high with tight blocks, 72.2, 72.9, and 76.2 ns, the last on the 0.28.11
  build's 76.0 ns level, so the level survived the rebuild on these CPUs and came up once in 40

### Allocate runs and duration for a fixed wall time

Twenty short runs or five long ones is a guess today (wink, 2026-09-15, in `feat: CI95 and LSC across
processes`). A bench mean's variance is `(s_p^2 + a/d) / R` for between-process spread `s_p`,
within-run noise `a/d` at run duration `d`, and `R` runs, and a wall time `T` buys
`R = T / (o + d)` runs at a fixed per-run overhead `o`, so the variance at a fixed `T` is least at
`d* = sqrt(a * o / s_p^2)`. `CI95 runs`' t multiplier and the stdev's reliability add a further
lean toward more runs.

- the pinned 7600x `zcr-mpsc-v1-2t` numbers, `a` about 0.01 ns^2 s from `CI95 blocks` 0.2 ns at
  1 s, `s_p` about 0.65 ns, and `o` about 4 s (100 s for 20 one-second runs), put `d*` near 0.3 s.
  At 100 s, 5 x 16 s predicts `CI95 runs` 0.80 ns, 20 x 1 s predicts 0.31, which the runs measured,
  and 23 x 0.3 s predicts 0.29
- the overhead bounds the run count more than the duration does: 4 s of every 5 s per run is the
  settle warm, the warm cap, the run sleep, the block sleeps, and the spawn, so whether a shorter
  settle is safe inside runs is a measurement worth making
- a fixed-budget sweep checks the model: one host and bench, about 100 s an invocation at
  5 x 16 s, 10 x 6 s, 20 x 1 s, and 30 x 0.3 s, each 3-4 times with `--record`, alternating
  configurations, comparing each configuration's `CI95 runs` against the actual scatter of its
  invocations' means
- an allocation hint once the sweep calibrates it: every invocation already knows `a` from the
  blocks, `s_p` from the runs, and `o` from wall time minus measured time, so the summary could
  print the run length and count that would minimize `CI95 runs` in the same wall time
- the model's two weak points: the within-run term is white only where the `resolution` row shows
  no drift, and rare levels make the run means a mixture, whose spread a 20-run invocation samples
  unreliably (the "Mark a run that lands on another level" entry), so the count may need to cover
  the rarest level that matters, not only the variance

### Compare two builds in one invocation

An A/B today is two invocations, and the same binary measured twice, clock pinned, differs by more
than its own `LSC runs`: 383.7 against 387.0 ns on the 3900X 90 minutes apart, and 70.0, 70.4, and
70.6 ns on the 7600x (wink, 2026-09-15, in `feat: CI95 and LSC across processes`, the A/A evidence
in [measuring-a-technique.md](notes/measuring-a-technique.md)). A pair measured in one invocation
with the arms alternating cancels whatever drifts between invocations.

- an arm is a binary and its knobs, so `--against PATH` running that binary's children alternately
  with this one's is the small version, and a config with two arms the general one
- the statistic is the paired difference or ratio and its interval, not two independent means: the
  pairing is what removes the invocation's own offset
- the ratio is what a claim about a technique carries between hosts, so this is the surface a
  cross-host table is built from
- it needs the run's arm in the record beside its `series` and `run`

### Replicate builds so layout is not confounded

A rebuild moved `zcr-mpsc-v1-2t` from 67.9 to 60.8 ns with no code change, 11%, so an A/B of one
build per arm mixes the code change with the layout difference between two binaries, and no number
of runs separates them (wink, 2026-09-15, asking why two builds rather than ten). Kin to "Measure
whether code layout moves the level", which asks whether layout moves it at all, where this asks
how to stop it confounding a comparison.

- k builds per arm, the same source rebuilt with a deliberate layout perturbation, turn layout into
  spread that averages instead of a fixed offset
- the perturbation wants one reproducible knob: a build script emitting a padding static sized by
  an environment variable is the cheapest, and function alignment flags or link order are the
  alternatives
- k follows from the between-build spread, which the layout entry's sweep measures first
- Stabilizer (Curtsinger and Berger, 2013) is the runtime form of the same idea, and the argument
  for why an unrandomized layout makes a measured speedup suspect

### Mark a run that lands on another level

One process in twenty unpinned `zcr-mpsc-v1-2t` runs on the 7600x read 66.5 ns, its blocks agreeing
to 0.1 ns, beside nineteen at 63.7 to 64.8 (wink, 2026-09-15, in `feat: CI95 and LSC across
processes`). That run doubled the invocation's stdev and `CI95 runs`, which is honest for a mixture of
levels, but nothing on the output says the bar is wide because of one run.

- the trimmed pair landed in `feat: CI95 and LSC across processes` and its `trimmed` line names
  the runs it dropped, so a disturbed run is called out already. What remains here is a mark on the
  run line itself, and whether a median belongs beside the mean
- mark a run line whose mean sits beyond some multiple of its own `LSC blocks` from the median run
  mean, a level rather than noise
- or print the median run mean beside `mean`, so a mixture shows as the two disagreeing
- either way the error bars stay over every run, since the level really comes up, and the mark only
  says why the bar is wide
- the second 7600x pass on `--pin-cpus 2,3` makes the case: one invocation drew a 76.2 ns run and a
  72.9 ns run among 69.1 to 70.3, and its stdev read 1.5 ns against the other invocation's 0.6,
  while both invocations' median run mean sat near 70.0 ns. A level that comes up once in 40 is
  missed entirely by 60% of 20-run invocations, (39/40)^20, so two invocations' error bars can
  differ by twice through that alone

### Name what sets a process's level

A fresh process lands a zcr bench on one of a few levels, 60.6 against 76.0 ns pinned on the 7600x
(the evidence in `feat: CI95 and LSC across processes`), and nothing says what decides which (split
from that cycle at its opening). We think it is cache-set aliasing from placement.

- children that map the ring regions themselves and sweep the second ring's page offset against
  the first
- then huge pages, which would remove the offset's effect if aliasing is the mechanism

### Measure core isolation on both hosts

Pinning leaves other work free to share the benched cores (wink, 2026-09-12). An interleaved A/B
on the 3900X and the 7600x: a systemd scope restricting other work to the unbenched CPUs first,
no reboot, then `isolcpus` with `nohz_full`. Any scope or boot change on a host is confirmed with
wink first. Kin to the "Tick-phase avoidance" idea, which isolation strictly dominates when a
reboot is allowed.

### Report the v1/v2 replication to the guide and zc-ring-x1

The pinned v2 two-thread handoff read slower than v1's in both of the report guide's record pairs,
once in the ignored `tmp/mpscv1rows/` (2026-09-11) and `tmp/v2rows/`, both lost 2026-09-14, and
zc-ring-x1 has not been told. 2026-09-08 replicated it and found it host-dependent, three
interleaved runs per cell, mean z4..n2: 3900X pinned 0,1 v1 86 ns and v2 105 ns, 7600x pinned 0,1
v1 60 ns and v2 56 ns, 7600x pinned 0,6 v1 32 ns and v2 35 ns, `0,6` being SMT siblings. Records
once in the ignored `tmp/v1v2-20260908/` here, lost 2026-09-14, and in
`~/iiac-perf-data/v1v2-20260908/` on the 7600x, tagged by pin, with both hosts' demo depth sweeps
beside them.

- the guide calls the gap a two-pair lead and does not cite the replication, a docs change
- a message to zc-ring-x1 with these numbers and the placement levels in `feat: CI95 and LSC across
  processes`, which
  make every single-process zcr comparison suspect. Their Todo already carries the demo's pin-pair
  mismatch, cross-L3 on the 3900X and same-L3 on the 7600x, so the message needs only the numbers
- the placement sweep's records (2026-09-05), 45 runs in the 7600x's
  `~/iiac-perf-data/placement-20260905` and 27 once in the ignored `tmp/placement-20260905` here,
  lost 2026-09-14, are
  unfiled, and [placement-map.md](notes/placement-map.md) is their home when it is refreshed

### A clock row in the report's stats

A run reports its latency numbers but not the frequency the measuring core ran at, so a pinned run's
report does not show the pin holding, and an unpinned one does not show where the clock sat (wink,
2026-09-15, after a restore line's live average read an idle 2.99 GHz after a 3300 MHz pin). The
data exists: every block seam samples the measuring core's delivered clock, the record's
`clock_khz`, and the grade block's settle cell already reads the warmup's.

- a `clock` row after `LSC` in the `mean` .. `LSC` list: the median delivered clock over the
  measured blocks, with its spread, `clock  3.29 GHz  (3.28-3.30 across the blocks)`, and `-` when
  the box exposes no readable clock
- a record key beside it, `clock_median_khz` or a name the record's dictionary settles, so an
  analysis need not recompute it from `clock_khz`
- the report guide's stats section explains the row, and says a pinned run's row should sit on the
  pin, a gap meaning the pin did not hold
- the runs tier landed first, in `feat: CI95 and LSC across processes`: a run line's `clock` column
  is the dominant core's seam clock range over the run, one number within the 1% stability
  tolerance, and the summary's `clock` line the range across the runs, both read back from the
  record's `clock_khz` with no new key. What remains here is the row in a run's own report, where a
  median beside the range would suit, and the record key

### Analyze a directory of records

A record exists so a re-analysis can happen without the session that produced it, and nothing reads
one back, so every analysis is a throwaway script whose numbers nobody can check (wink, 2026-09-03,
reading the 7600X duration sweep). An `analyze` subcommand over a directory of records, sharing
`record.rs`'s struct so the schema keeps one owner.

- three tiers, in increasing order of what they are worth:
  - **tabulate**: pivot the records into a table on an axis, duration, host, bench, or run. The
    least interesting tier, and the one the other two are built on
  - **read the series nothing reads**: `block_mean_ns`, `block_samples`, `clock_t_ns`, `clock_cpu`,
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
  was the wrong shape. 3-5 runs per cell, interleaved, is what the guide has said all along
- the output reuses the report's row names, so the guide decodes the new surface for free. Grading
  the set the way a run grades itself is the natural extension: do these runs agree, and is a
  disagreement drift, a step, or one bad run
- `--format csv` / `--format json` for the plotting hand-off, kin to "Machine-readable report
  output" below, one flag family
- its cross-run arithmetic is `feat: CI95 and LSC across processes`'s, one statistics module
  serving both, and the records carry the host block cross-host analysis needs, a hostname alone
  naming nothing

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

### Vyukov's unbounded SPSC

The node-based unbounded SPSC from 1024cores.net, the second implementation the crossbeam
baselines exist to frame (wink, 2026-08-28), zc-ring-x1's SPSC v1 being the first, measured by the
`zcr-spsc-v1` pair.

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

### Config search up the parents, arms, and a pin's boost

What `feat: a run is a config file` left of the `--config` entry it was opened from (wink,
2026-09-05 and 2026-09-14).

- the project-local search up the parents went into the cycle after all, as its rung `feat:
  iiac-perf.md is found up the parents`
- with spawning, a config also names the children's knobs, and an A/B is two configs or one with
  two arms, which is the shape a cross-host comparison wants. The cycle runs each bench's runs back
  to back, not interleaved (wink, at the opening of `feat: CI95 and LSC across processes`)
- a boost option for a pin that should keep boost on, if benchmarking wants one. A benchmark
  directory's config pinning the clock became the run key `pin_freq` in `feat: config and setup`,
  since a project-local `[freq]` with `min_mhz = max_mhz` also moves where a restore returns

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

Sample `cpuinfo_avg_freq` at block seams (the reader exists, `src/freq.rs`) so a mid-run step
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

### Move qualify-environment onto the child runner

`qualify-environment` spawns its children with its own loop and parses their report text, while
`feat: CI95 and LSC across processes` gives every bench a child runner that reads each child's
record back (split from that cycle at its opening, whose Todo entry had subsumed this). One runner
serves both.

- the children's results come back as records, so the prose parsing of the `env warmup`, `env
  bench`, and `mean` rows goes, and "Qualify the environment without a bench" above decides what a
  child measures
- the orchestration in `tests/qualify_environment.rs` reduces to asserting on the verdict, the
  rest of the "Stability selftest mode" idea in `## Ideas`

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

### Move the block seam's work off the measuring thread

Use the FastForward-style SPSC ring. The block flush stops the bench for ~1-2 ms every block, about
50 ms at the default of 100 blocks over 5 s, so ~2-4% of a run is spent at seams. The 1-2 ms was
measured when the flush sorted a batch, and the pipeline now drains a 65,536-sample stage, so it
wants re-measuring first. Hand the filled buffer to a consumer thread that sorts,
summarizes and records while the producer fills a second one. The seam drops to a pointer swap.

- the payload is one word, a buffer offset, the exact shape `ffq` is built for, and the
  project dogfooding the queue it benchmarks
- double-buffered: at ~1-2 ms of work per 50 ms block the consumer runs ~30x faster than it
  needs to, so two buffers never back up
- honest cost: the consumer's cross-core traffic runs *during* measurement, trading a gap on
  the hot core for background L3 pressure. Measure it the way the -4 seam probe was measured
  (interleaved A/B, pinned, trimmed mean) rather than assuming
- blocked on the ring existing. See the "FastForward-style SPSC ring" entry, currently on the
  `ffq-spsc-notes` bookmark rather than `main`

### Sweep "box" to "host"

The project has two words for one thing. The record field is `host`, the host identity cycle
builds on it, and the prose says "box" about a hundred times, so a reader meeting both is
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
- ranked after the host identity cycle, whose host block is what makes `host` the
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
- auto profiles derived from the discovered tree, so one command line is portable across
  hosts: moved to [Placements by name and a cpus
  command](#placements-by-name-and-a-cpus-command), as `ccx`, `x-ccx`, and `smt`
- **placement tracking** (added 2026-08-01): when unpinned, placement is the dominant factor
  (4-18x on the 3900X) but is currently invisible. Observe it instead of only controlling it.
  Two tiers of knowledge, and the report says which one a claim comes from:
  - cooperative (exact): threads placed through the `--pin` pool are known
  - observational (complete but sampled): a bench need not announce threads or
    sub-processes, and the kernel tells us anyway: sweep `/proc/self/task/` at block seams
    (last-ran lCPU is `stat` field 39, children via `/children`, recursively). CPU-time
    deltas between sweeps identify the active threads with no cooperation, and `sched_getcpu`
    (vDSO-cheap) covers the measuring thread exactly. Sampled truth: migrations inside a
    block and threads born and dead between seams are unseen, which matches the step
    detector's block granularity, and cost is ~us per seam against a 1-2 ms seam
  - blocks gain a placement-class label, so a placement migration becomes an *attributed*
    step ("cross-LLC -> same-core"), the way the env grade attributes DVFS
  - unpinned `--blocks` runs stratify block stats by placement class instead of one smeared
    CI: the scheduler's wandering becomes a free stratified experiment (how the SMT fast
    mode was found)
- subsumes the vocabulary half of "Tighten thread and CPU terminology" (above): keep its
  software-thread vs lCPU distinction, adopt lCPU as the standard term

### Windows and macOS port considerations

iiac-perf runs on Linux alone, and a port was named as a Todo on 2026-09-04 without its content
being written down. What is Linux-only today, as a start: CPU pinning through
`sched_setaffinity`, the cpufreq sysfs files behind `--pin-freq`, `read-freq`, and the delivered
clock, `/dev/cpu_dma_latency` for the pin-idle clamp, the host block's `/proc` and `/sys` reads,
the udev rule the setup subcommand would write, and the inhibit guard. Each wants its platform
equivalent or an honest "not measured here" on the report.

- the seam and the per-platform notes are in
  [measuring-a-technique.md](notes/measuring-a-technique.md): a portable core, the timing loop,
  blocks, histogram, statistics, record, and report, under an environment layer that is Linux-shaped
  today. macOS is the awkward one, with affinity hints at best and no user-level clock control, and
  a bare-metal target has no processes at all, so re-rolling placement there means randomizing
  allocation offsets inside the program rather than spawning a child

### Rebase web-claude-tweaks onto post-0.22.0 main

It rewrites an already-published bookmark (needs approval) and its arbitrary `0.21.0-b`
version needs replacing, owed from the 0.22.0 close-out plan.

### Unit scaling in report columns

`us`/`ms`: per-row auto-scale so columns stay eyeball-comparable (bands are monotonic, so a
row's first/last/mean share a magnitude), or `--units ns|auto` for script-stable output. Needs
`--decimals` landed first (`3.18 ms` vs `3 ms`). Candidate `-4` for the report-options cycle.

### Drift and clock plots in the terminal

Every run records a block-mean series and a delivered-clock series and reports them as a grade
letter, so "did this run drift" is answered by a letter with no picture behind it (wink,
2026-09-03). Braille or block characters drawn in the terminal, no plotting dependency.

- two plots sharing one time axis: block means, where the drift and step signals come from, and the
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

- the runs summary is a natural first user (wink, 2026-09-15, in `feat: CI95 and LSC across
  processes`): it averages each run's full mean, since `mean z4..n2` takes its bounds from the
  bands a run populated, so two runs' trimmed means can cover different spans. A fixed-quantile
  trimmed mean in the record would give `CI95 runs` a statistic that ignores interference spikes

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

Will likely need inner-loop support on `Probe` (N calls per sample, report divides by N
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

[1]: #feat-a-run-is-a-config-file-opening
[2]: #feat-a-config-key-for-every-run-parameter
[3]: #feat-init-config-writes-every-key-commented-out
[4]: #feat---config-names-the-runs-file
[5]: #feat-init-config-takes-the-lines-values
[6]: #feat-update-config-rewrites-a-config-in-place
[7]: #feat-a-config-file-as-a-bench-argument
[8]: #feat-init-config-writes-the-run-this-host-would-make
[9]: #feat-iiac-perfmd-is-found-up-the-parents
[10]: #refactor-setup-is-setup-freq
[11]: #feat-setup-freq-checks-the-project-local-freq-table
[12]: #docs-the-readmes-guide-to-config-files
[13]: #docs-the-clock-experiment-run-from-its-config
[14]: #feat-a-run-is-a-config-file-closing
[57]: /notes/chores/chores-04.md#trimmed-core-stats-p10-p90
[61]: /notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit
[75]: /notes/chores/chores-05.md#settle-time-is-not-a-grade
[84]: /notes/chores/chores-06.md#docs-experiment-in-the-local-agent-files
