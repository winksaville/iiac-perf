# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record, in `TODO.md > ## Closed` before this file, and the cycles before the
rule in the frozen [notes/chores/](notes/chores) and [notes/done.md](notes/done.md).

## Closed

### feat: shorter hints for a bare or unknown line

(wink, 2026-10-16) A bare `iiac-perf` opens on a sentence to read, "no command given: 'iiac-perf
run BENCH...' runs benches, and -h lists the commands.", before the line to type, and a word that
is no command and no bench is told that `-h` lists the commands, where the benches it may have
meant are listed by `iiac-perf run`, in a line too long to read at a glance. Both should say what to type and nothing to read around it,
the banner kept.

#### Deliberation

- Its own cycle, after [feat: the CLI is subcommands](https://github.com/winksaville/iiac-perf/blob/de07985deda7/zCLOSED.md#feat-the-cli-is-subcommands)
  landed (wink, 2026-10-16): editing that cycle's `run` rung in place would have rebased the
  rungs above it, and their agent-repo sessions would describe content changed after them.
- The banner stays (wink, 2026-10-16), the one place a bare line says which build it is.
- The shape is wink's: `Try ...:`, the line indented with no quotes so it copies off the screen,
  then `BENCH...:` and the list. A `run` with no benches says `Try run -h, or:` above the same
  block, one function printing both so they cannot drift.
- The unknown word's line is wink's, "'xyz' is not a command, try '-h' or 'run'": the two words
  to type, with no binary's name and no saying what each lists.
- Single-step with the dev rename: the commit changes code and validation installs it, so the
  rename keeps a mid-cycle install off the plain `iiac-perf`, and Land restores it.

#### Solution

`bench_list` returns the text a bare `iiac-perf` and an empty `run` print: the banner, `Try ...:`,
`  iiac-perf run BENCH...`, and `BENCH...:` over the wrapped bench names. A word that is no
command and no bench is answered "'xyz' is not a command, try '-h' or 'run'".

#### Acceptance check

- A bare `iiac-perf-dev` prints the banner, `Try -h, --help, or run:`, the run line, and
  `BENCH...:` over the list, and `iiac-perf-dev run` the same under `Try run -h, or:`. Passed.
- `iiac-perf-dev xyz` says "'xyz' is not a command, try '-h' or 'run'". Passed.
- `vc-x1 validate` passes.

#### Ladder

- feat: shorter hints for a bare or unknown line (done)

# References
