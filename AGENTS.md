# AGENTS.md - Bot Instructions

The universal core of this project's bot instructions: the dual-repo model, the hard rules, and
a map of everything else. This file is one of the [agent-files](#terminology), shared across our
dual repos and carried by every family member: a member's diff against the template repository's
payload is what that member has proposed, so drift is a diff, not a mystery.

## Hard rules

The rules whose violation costs the most, numbered so a review can name them. Each links to its
detail; the rule as stated here is binding on its own.

0. **Read [custom.md](custom.md) before acting on anything below**: the project's layer
   (medium, validation commands, overrides), loaded last, wins conflicts with this file and the
   satellites. Already satisfied if your harness auto-loaded it.
1. **A cycle rung is committed by `vc-x1 push`, never pre-committed with `jj commit`.** In an
   instruction, "commit", "push", and "commit + push" all mean `vc-x1 push`; a bare `jj commit`
   is asked for by name and is only for work that never publishes.
   [Committing vs pushing](agent-data/cycle.md#committing-vs-pushing).
2. **Every push needs that push's explicit approval.** Approval of a plan that includes a push
   does not authorize the push; ask again at the moment of pushing. Only an explicit scoped
   delegation waives the stops. [Before any push](agent-data/cycle.md#before-any-push).
3. **Hard stop after the turn's final push or squash-push.** Closing words go before the
   invoke; afterwards, nothing until the user speaks (a bare acknowledgment if the harness
   forces a token). [After the final push](agent-data/cycle.md#after-the-final-push-hard-stop).
4. **Never `jj describe` a published or trailer-carrying commit without coordinating first.**
   When a re-describe is agreed, hand-copy the `ochid:` trailers into the new body.
   [Re-describing](agent-data/jj.md#re-describing-coordinate-first-and-keep-the-trailer).
5. **Never hand-write `ochid:` trailers**; `vc-x1 push` stamps them.
   [ochid trailers](agent-data/jj.md#cross-repo-linking-ochid-trailers).
6. **Use jj, not git**, for version-control operations. [jj basics](agent-data/jj.md#jj-basics).
7. **Read the checklist before the action**: [agent-data/cycle.md](agent-data/cycle.md) before
   commit work and before any push. Validation runs before the push, never after.
8. **Typeable punctuation only** in durable text: no em/en dash, ellipsis, or arrow characters.
   [Typeable punctuation](agent-data/prose.md#typeable-punctuation-only).
9. **One title per step, verbatim in three places**: the ladder rung, the chores `##` header,
   and the commit title line up exactly. See
   [the shape](agent-data/prose.md#conventional-commit-shape-ladder--chores--commit).
10. **Stop and ask** on ambiguous input, on any deviation from the agreed plan, and when 5+
    minutes on a simple task has produced no progress. A clarifying question costs seconds;
    redoing misaligned work costs much more.
11. **Alert the user when introducing an `unwrap` / `expect` / `unwrap_or*` site**, with its
    `// OK: ...` comment. [code.md](agent-data/code.md).
12. **Experiment in your local [agent-files](#terminology); the template's payload is the
    read-only copy.** A proposed rule change is edited into the local copy of the file the rule
    lives in, so the diff against the payload is the proposal set.
    [Changing the agent-files](#changing-the-agent-files).

## Terminology

**Repos.** The two repos of [the dual-repo model](#the-dual-repo-model) below. "Work repo" and
"bot repo" are the standard names; write them as two words, adding a hyphen only when the pair
sits directly in front of another noun ("work-repo commit", "bot-repo side"). Notes:

- `.claude` is the bot repo's *path*, not its name, so commands (`-R .claude`) and ochid paths
  (`/.claude/<chid>`) keep the literal path.
- The vc-x1 CLI's scope name for the work repo is `work` (`--scope=work|bot|work,bot`).
- "Work commit" / "Work-N" (capitalized) is a cycle-stage term, not a repo name; a generic
  commit landing in the work repo is a "work-repo commit", never a bare "work commit".

**Agent-files.** The instruction set an agent reads: `AGENTS.md`, `custom.md`, and
`agent-data/*`. The template repository's payload holds the official copies and every member
repo carries its own; how they change is [Changing the agent-files](#changing-the-agent-files).
Notes:

- Always hyphenated, unlike "work repo" above, because it names one set rather than a two-word
  noun phrase, and it matches its sibling directory `agent-data/`.
- **Pinned** describes an agent-file whose content is meant to match the payload (`AGENTS.md`,
  `agent-data/*`). `custom.md` is an agent-file but is never pinned, since holding what cannot
  be family-wide is its job.
- Retired: "instruction files", which named the same set back when `custom.md` was the only
  editable one. Both terms should not circulate.

## The dual-repo model

This project uses **two separate jj-git repos**:

1. **Work repo** (`.`, the project root): the project's generated artifact, whether code,
   prose, image, song, or whatever it produces.
2. **Bot repo** (`.claude`): Claude Code session data (symlink from
   `~/.claude/projects/<path-to-project-root>/.claude`).

Both are managed with `jj` (Jujutsu), which coexists with git. Every commit in one repo links
to its counterpart in the other via an `ochid:` trailer; see
[agent-data/jj.md](agent-data/jj.md).

## Working practices

- **Stay in the project root**; target other directories with `-R` flags or absolute paths
  rather than `cd` (discuss with the user first if `cd` seems necessary).
- **Shortest unambiguous path** in shell commands (`ls notes/`, not the absolute form).
  Out-of-workspace paths stay absolute, and Read/Edit/Write tool args stay absolute (a
  tool-boundary constraint, not style).
- **One command per shell invocation**; don't bundle steps (`a && b; c`). Bundling hides which
  step produced which output. Exceptions: a genuine pipeline (`grep | sort`) or a tight,
  inseparable pair where the join is the point.
- **Never mask a command's exit status.** What reads the result sees the invocation's status, so
  a command that fails has to make its invocation fail.
  - never pipe a validating command into `tail` / `grep`, and never `&&` after a piped stage: a
    pipeline's status is the last command's. `${PIPESTATUS[0]}` is the escape hatch when a pipe
    is genuinely wanted
  - never trail one with `; echo "exit=$?"`: that prints the status while the invocation itself
    still exits 0, so the failure is visible only to whoever reads the text
  - to report and still fail: `cmd || { rc=$?; echo failed=$rc; exit $rc; }`. Leave `failed=$rc`
    unquoted: it has no spaces to protect, and the quotes can stop a harness permission rule
    from matching a command it would otherwise allow (wink, 2026-08-05)
- **Scratch files go in repo-local `tmp/`** (gitignored, `mkdir -p tmp` on demand, never
  committed). Prefer it over `/tmp` and the harness scratchpad; `/tmp` is for out-of-project
  temporaries.
- **Read the slice you need** from long notes files; the routine acquaint read is `TODO.md`
  `offset=0, limit=60`. [Notes files](agent-data/notes.md).
- **Delegate mechanical subtasks to lesser models** (Haiku / Sonnet); reserve the top model for
  design and tricky work. Top-model tokens are the scarce resource.
- **Don't use the per-project memory directory** (`~/.claude/projects/<path>/memory/`). Durable
  context lives in these committed files: easy for everyone to find beats convenient for the
  bot alone.
- **Mark speculation** in durable text with "We think ..." so a reader can tell the measured
  from the inferred. [Speculation marker](agent-data/prose.md#speculation-marker).
- **End technical explanations in conversation with a plain synopsis**, marked clearly (e.g.
  "The plain version:").
  [Plain synopsis](agent-data/prose.md#plain-synopsis-after-technical-explanations).

## File map

Always loaded:

- `AGENTS.md`: this file.
- [custom.md](custom.md): the project's layer; what cannot be family-wide.

Read at the moment of action, immediately before acting, not from memory (`agent-data/`,
universal, pinned; checklists first, rationale after):

- [cycle.md](agent-data/cycle.md): commit / push / close-out checklists. Read before any commit
  work or push.
- [jj.md](agent-data/jj.md): jj usage, ochid trailers, the re-describe rule, `.vc-config.toml`.
- [prose.md](agent-data/prose.md): prose form, punctuation, commit-title identity. Read before
  writing durable text.
- [notes.md](agent-data/notes.md): TODO / chores / done mechanics, references, anchors. Read
  before editing notes files.
- [code.md](agent-data/code.md): doc comments and unwrap discipline. Read before writing code.

Authoritative protocol and project records (`notes/`):

- [cycle-protocol.md](notes/cycle-protocol.md): the full cycle protocol; it wins over any
  checklist summary of it.
- [versioning.md](notes/versioning.md): the version scheme and version-of-record.
- `TODO.md`, `notes/todo-backlog.md`, `notes/bugs.md`, `notes/chores/`, `notes/done.md`: the
  project's working records; conventions in [agent-data/notes.md](agent-data/notes.md).

## Changing the agent-files

The **agent-files** are `AGENTS.md`, `custom.md`, and `agent-data/*`. The official copies are the
template repository's payload; every member repo carries its own copy of the same set.

- **The payload is the read-only copy.** A member never edits it to experiment.
- **A proposal is edited into the member's local copy of the file the rule lives in**, so the
  diff between a member and the payload *is* that member's open proposal set. It needs no
  maintenance and cannot go stale.
- **An agent-file change is its own commit**, so `git log -- AGENTS.md agent-data/` reads as a
  list of rule changes rather than unrelated feature titles, and the commit's `ochid:` trailer
  links the bot-repo session that reasoned it out. The diff says what differs now; the history
  says when, by whom, and why.
- **A local agent-file may hold an unagreed experiment**, so unlike the payload it does not read
  as family-agreed. Diff against the payload when that distinction matters.
- **At convergence** the family reviews the members' diffs, folds what it accepts into the
  payload, and every member re-syncs. The diff empties; the history keeps the record.
- **A resolved experiment retires** like a finished Todo, at the beat where it resolves: see
  [Retiring Done entries](agent-data/notes.md#retiring-done-entries). Adopted and rejected retire
  the same way.

## custom.md: the project layer

[custom.md](custom.md) is an agent-file like the others, and the aim is that it stays as generic
as they do. Only what cannot be family-wide belongs here:

- **medium-determined**: the medium and its validation commands (what the per-commit checklist's
  "validate the artifact" runs), and versioning specifics beyond
  [versioning.md](notes/versioning.md). Not a divergence and not negotiable, since another
  medium could not adopt them if it wanted to.
- **elective divergence**: somewhere this project deliberately differs. An entry must say **why
  it cannot be family-wide**; with no answer it is an experiment and belongs in the pinned file
  where the rule lives (see [Changing the agent-files](#changing-the-agent-files)).
- **the dogfood log**: dated entries on what this project is trying and where the instructions
  chafed, each carrying a status. In-flight entries only; resolved ones retire.

Precedence: custom.md is loaded last and wins conflicts with this file and the satellites.
