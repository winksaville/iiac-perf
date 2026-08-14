# AGENTS.md - Bot Instructions

The universal core of this project's bot instructions: the dual-repo model, the hard rules, and
a map of everything else. This file is one of the [agent-files](#terminology), shared across our
dual repos and carried by every family member: a member's diff against the template repository's
payload is what that member has proposed, so drift is a diff, not a mystery.

## Hard rules

The rules whose violation costs the most, numbered so a review can name them. Each links to its
detail. The rule as stated here is binding on its own. The rules bind the bot, and none is
absolute: any rule bends when wink says so explicitly at the moment, or in advance as an
explicit scoped delegation (rule 10's stop-and-ask is the path), and a taken exception is
recorded in the cycle's records. No rule bends silently, and no exception is self-granted.

0. **Read [custom.md](custom.md) before acting on anything below**: the project's layer
   (medium, validation commands, conventions), loaded last, wins conflicts with this file and the
   satellites. Read it every session: only `AGENTS.md` is auto-loaded, and what to read past
   `custom.md` is `custom.md`'s to say.
1. **A cycle rung is committed by `vc-x1 push`, never pre-committed with `jj commit`.** In an
   instruction, "commit", "push", and "commit + push" all mean `vc-x1 push`. A bare `jj commit`
   is asked for by name and is only for work that never publishes.
   [Committing vs pushing](agent-data/cycle-checklists.md#committing-vs-pushing).
2. **Every push needs that push's explicit approval.** Approval of a plan that includes a push
   does not authorize the push. Ask again at the moment of pushing. Only an explicit scoped
   delegation waives the stops.
   [Before any push](agent-data/cycle-checklists.md#before-any-push).
3. **Hard stop after the turn's final push or squash-push.** Closing words go before the
   invoke. Afterwards, nothing until the user speaks (a bare acknowledgment if the harness
   forces a token).
   [After the final push](agent-data/cycle-checklists.md#after-the-final-push-hard-stop).
4. **Never `jj describe` a published or trailer-carrying commit without coordinating first.**
   When a re-describe is agreed, hand-copy the `ochid:` trailers into the new body.
   [Re-describing](agent-data/jj.md#re-describing-coordinate-first-and-keep-the-trailer).
5. **Never hand-write `ochid:` trailers.** `vc-x1 push` stamps them.
   [ochid trailers](agent-data/jj.md#cross-repo-linking-ochid-trailers).
6. **Use jj, not git**, for version-control operations. [jj basics](agent-data/jj.md#jj-basics).
7. **Read the checklist before the action**:
   [agent-data/cycle-checklists.md](agent-data/cycle-checklists.md) before commit work and
   before any push. Validation runs before the push, never after.
8. **Typeable punctuation only** in durable text: no em/en dash, ellipsis, or arrow characters.
   [Typeable punctuation](agent-data/prose.md#typeable-punctuation-only).
9. **One title per step, verbatim in three places**: the ladder rung, the chores `##` header,
   and the commit title line up exactly. The title is a step's only identifier, so it carries no
   number and no version, and it must be unambiguous within its cycle and its chores file. See
   [the shape](agent-data/prose.md#conventional-commit-shape-ladder--chores--commit).
10. **Stop and ask** on ambiguous input, on any deviation from the agreed plan, and when 5+
    minutes on a simple task has produced no progress. A clarifying question costs seconds,
    while redoing misaligned work costs much more.
11. **Alert the user when introducing an `unwrap` / `expect` / `unwrap_or*` site**, with its
    `// OK: ...` comment. [code.md](agent-data/code.md).
12. **Intent decides where a rule change is written.** Meant for the family: edit the local copy
    of the pinned file the rule lives in, any time, so the diff against the payload is the
    proposal set. Not meant for the family: it belongs in `custom.md` instead, and has to say why
    it cannot be family-wide. The payload is never edited to experiment.
    [Changing the agent-files](#changing-the-agent-files).
13. **A cycle runs on one topic bookmark in the work repo**, named by the cycle title's slug,
    created at the opening, carrying every step. `main` advances only when the finished cycle
    lands on it, never by pushing commits straight to `main`. Once the bookmark lands on `main`
    the bookmark is deleted, locally and remotely.
    [Cycles run on a bookmark](agent-data/cycle-checklists.md#cycles-run-on-a-bookmark).

## Terminology

**Repos.** The two repos of [the dual-repo model](#the-dual-repo-model) below. "Work repo" and
"bot repo" are the standard names. Write them as two words, adding a hyphen only when the pair
sits directly in front of another noun ("work-repo commit", "bot-repo side"). Notes:

- `.claude` is the bot repo's *path*, not its name, so commands (`-R .claude`) and ochid paths
  (`/.claude/<chid>`) keep the literal path.
- The vc-x1 CLI's scope name for the work repo is `work` (`--scope=work|bot|work,bot`, and the
  same keywords as `vc-x1 config`'s target). `.vc-config.toml` names the same two sides under
  `[repos]` as `work` / `bot`. A config still on the older `[workspace]` schema is what
  `vc-x1 config --validate` reports.
- "Work commit" / "Work-N" (capitalized) is a cycle-stage term, not a repo name. A generic
  commit landing in the work repo is a "work-repo commit", never a bare "work commit".

**Agent-files.** The instruction set an agent reads: `AGENTS.md`, `custom.md`, `agent-data/*`, and
anything `custom.md` points at. The template repository's payload holds the official copies and
every member repo carries its own. How they change is
[Changing the agent-files](#changing-the-agent-files). Notes:

- Always hyphenated, unlike "work repo" above, because it names one set rather than a two-word
  noun phrase, and it matches its sibling directory `agent-data/`.
- **Pinned** describes an agent-file whose content is meant to match the payload (`AGENTS.md`,
  `agent-data/*`). `custom.md` is an agent-file but is never pinned, since holding what the
  pinned files structurally cannot is its job. The same goes for any layer below it.
- Retired: "instruction files", which named the same set back when `custom.md` was the only
  editable one.

**Project layer.** The project's own agent-files, as against the pinned ones: `custom.md` and
anything it points at. Called a *layer* because it loads last and wins conflicts, so it sits over
the pinned set rather than beside it.

**Cycle.** The unit of change: three stages, an opening, one or more work-repo changes, and a
closing (the protocol's Preparation / Work / Close-out, whose bookend commits are the opening
and the close-out). A single-step cycle folds all three stages into one commit. A multi-step
cycle commits them individually, minimum two (a Work commit plus the close-out, the opening
commit being optional), typically three or more. The full protocol is
[cycle-protocol.md](agent-data/cycle-protocol.md).

## The dual-repo model

This project uses **two separate jj-git repos**:

1. **Work repo** (`.`, the project root): the project's generated artifact, whether code,
   prose, image, song, or whatever it produces.
2. **Bot repo** (`.claude`): Claude Code session data. The real directory is `<project>/.claude`.
   Claude Code reaches it through a symlink at `~/.claude/projects/<mangled-project-path>`
   pointing *at* that directory, with no further path component. `vc-x1 symlink` creates it.

Both are managed with `jj` (Jujutsu), which coexists with git. Every commit in one repo links
to its counterpart in the other via an `ochid:` trailer. See
[agent-data/jj.md](agent-data/jj.md).

## Working practices

- **Stay in the project root.** Target other directories with `-R` flags or absolute paths
  rather than `cd` (discuss with the user first if `cd` seems necessary).
- **Shortest unambiguous path** in shell commands (`ls notes/`, not the absolute form).
  Out-of-workspace paths stay absolute, and Read/Edit/Write tool args stay absolute (a
  tool-boundary constraint, not style).
- **One command per shell invocation.** Don't bundle steps (`a && b; c`). Bundling hides which
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
  committed). Prefer it over `/tmp` and the harness scratchpad. `/tmp` is for out-of-project
  temporaries.
- **Read the slice you need** from long notes files. The routine acquaint read is `TODO.md`
  `offset=0, limit=60`. [Notes files](agent-data/notes.md).
- **Use https remotes, not ssh.** Unconditional rather than "when the agent is sandboxed", because
  the remote is chosen at clone time and whether a sandboxed agent will ever touch the repo is not
  knowable then. A sandbox denies ssh twice over: reads of `~/.ssh` are blocked except the signing
  key and `known_hosts`, so no auth key is available, and we think a host allowlist cannot admit
  port 22 at all, since ssh carries no SNI or Host header to match on. The network leg is a
  spawned `git` child that inherits the sandbox, which is why the same config succeeds from a
  human's terminal and fails from a session. So an **ssh remote is the first thing to check when a
  push dies at the network leg**, ahead of any theory about size or timeouts. Both wrong theories
  were held, and eliminated by test, before this rule was written.
  - **Changing a remote's URL needs the user's go**, like any outward-facing change: it moves where
    the repo publishes. Trivially reversible, so this is a confirmation and not a prohibition.
- **Delegate mechanical subtasks to lesser models** (Haiku / Sonnet). Reserve the top model for
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

Read every session (`AGENTS.md` is the one file the harness auto-loads, and hard rule 0 covers
the rest of the chain):

- `AGENTS.md`: this file.
- [custom.md](custom.md): the project's layer, and any further file it points at.

Read at the moment of action, immediately before acting, not from memory. The `agent-data/`
files are universal and pinned, listed checklists first, rationale after:

- [cycle-checklists.md](agent-data/cycle-checklists.md): commit / push / opening / close-out
  checklists. Read before any commit work or push.
- [cycle-protocol.md](agent-data/cycle-protocol.md): the full cycle protocol. It wins over any
  checklist summary of it.
- [jj.md](agent-data/jj.md): jj usage, revsets, ochid trailers, the re-describe rule, cycle and
  long-lived bookmarks.
- [prose.md](agent-data/prose.md): prose form, punctuation, commit-title identity. Read before
  writing durable text.
- [notes.md](agent-data/notes.md): TODO / chores / done mechanics, references, anchors. Read
  before editing notes files.
- [code.md](agent-data/code.md): doc comments and unwrap discipline. Read before writing code.
- [versioning.md](agent-data/versioning.md): the version scheme and version-of-record.

Project records (`notes/` and the repo root): records only, never universal rules. Anything
normative that outgrows the project belongs in `agent-data/` via
[Changing the agent-files](#changing-the-agent-files):

- `TODO.md`, `notes/todo-backlog.md`, `notes/bugs.md`, `notes/chores/`, `notes/done.md`: the
  project's working records. Conventions are in [agent-data/notes.md](agent-data/notes.md).

## Changing the agent-files

The **agent-files** are `AGENTS.md`, `custom.md`, and `agent-data/*`. The official copies are the
template repository's payload, and every member repo carries its own copy of the same set.

- **The payload is the read-only copy.** A member never edits it to experiment. The one thing
  that goes straight in is a *correction*: a factual error, a typo, a stale cross-reference. A
  wrong fact has no second opinion to gather, and leaving it in place misleads every member on
  first read.
- **Intent decides the file, and nothing gates the edit.** A member writes a rule change into its
  local copy of the pinned file whenever it means the family to take it, without asking first.
  The review happens at convergence, on the diff. A change the member does *not* mean the family
  to take goes to `custom.md` and must say why it cannot be family-wide.
- **The diff between a member and the payload *is* that member's open proposal set.** It needs no
  maintenance and cannot go stale.
- **An agent-file change is its own commit**, so `git log -- AGENTS.md agent-data/` reads as a
  list of rule changes rather than unrelated feature titles, and the commit's `ochid:` trailer
  links the bot-repo session that reasoned it out. The diff says what differs now. The history
  says when, by whom, and why.
- **Convention work runs as its own cycle.** A convention itch mid-feature becomes a backlog
  entry or a small dedicated cycle, never an inserted rung in the feature's ladder: rung by
  rung, rule changes bury a feature cycle's records under work its title never promised.
- **A local agent-file may hold an unagreed experiment**, so unlike the payload it does not read
  as family-agreed. Diff against the payload when that distinction matters.
- **At convergence** the family reviews the members' diffs, folds what it accepts into the
  payload, and every member re-syncs. The diff empties, and the history keeps the record.
- **A resolved experiment retires** like a finished Todo, at the beat where it resolves: see
  [Retiring Done entries](agent-data/notes.md#retiring-done-entries). Adopted and rejected retire
  the same way.

## custom.md: the project layer

[custom.md](custom.md) is the project's own layer and, unlike the pinned files, is never pinned:
every project's content differs by construction. It ships from the payload holding nothing but its
own shape, so a project that changes nothing still has a valid one, and a project adds whatever it
needs: the medium and its validation commands, what a version bump promises this artifact's users,
and its conventions.

**`## Project conventions and overrides` is empty at birth and should usually stay that way.** A
rule the project would keep is still a *proposal* until it is rejected, so by default it belongs in
the pinned file where the rule lives (see [Changing the agent-files](#changing-the-agent-files)),
where it shows up as a diff. Writing it here instead hides it from exactly the review that should
decide it. An empty section stays, with `_None._` under it, rather than being deleted.

**An entry that only points at a further file is not an override** and owes no "why not
family-wide" justification, since it supersedes nothing. A project with a wider context to answer
to can hold all of it in that further file and reach it from one line here, which keeps the rest of
this file identical to the payload's. Nothing pinned names the further file or knows what is in it:
a pinned file asking for something "in custom.md" is answered by following the pointer it finds
there.

Precedence: custom.md is loaded last and wins conflicts with this file and the satellites.
