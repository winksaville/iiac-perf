# Notes

This directory contains various notes and documentation related to the project.
Each file is organized by topic for easy reference.

By default there are chores-*.md and todo.md. Chores are general notes
about tasks and todo.md contains short term tasks and their status.

In the future we I expect we may want to create a "notes"
database to better manage the information, TBD.

Examples chore file:
```
# Chores-01.md

General maintenance tasks and considerations for the project see other files for
more specific topics. A chore in a chores file provides quick information on the
how and why of a particular chore. The section header is short and sweet
and the title is appended with the version number of the app when the chore
is completed.

## Create an app that does something interesting (0.1.0)

The app counts from 1 to 100, not to interesting.
```

## Commit Workflow

This project uses a per-step commit / push / finalize flow against
two paired repos (app + bot session). Full rules live in
[CLAUDE.md](../CLAUDE.md#commit-push-finalize-flow) — that file is
the source of truth (the bot loads it automatically; humans can
read it from there too).

## jj tips

For users new to jj see [jj-tips.md](jj-tips.md).

```
## Chores format

Filename: "chores-XX.md"
example: chores-01.md

Format of section labels: "## <short description> (X.Y.Z)"
example: "## Topic format description (0.1.0)"

Example chore file:
```
# Chores-01.md
 
General maintenance tasks and considerations for the project see other files for
more specific topics. A chore in a chores file provides quick information on the
how and why of a particular chore.

## Do something (1.3.1)

Describe something
```

## Versioning during development

This is using jujustiu, jj + git and we'll see how it goes. Below is my
git workflow, jj will be different but we'll have to discover that as
we go.

Every plan must start with a version bump. Choose the approach based on scope:

- **Single-step** (recommended for mechanical/focused changes): bump directly to
  `X.Y.Z`, implement in one commit. Simpler history.
- **Multi-step** (for exploratory/large changes): bump to `X.Y.Z-devN`, implement
  across multiple commits, final commit removes `-devN`.

The plan should recommend one approach and get user approval before starting.

For multi-step:
1. Bump version to `X.Y.Z-devN` with a plan and commit as a chore marker
2. Implement in one or more `-devN` commits (bump N as needed)
3. Final commit removes `-devN`, updates todo/chores — this is the "done" marker

The final release commit (without `-devN`) signals completion rather than amending
prior commits. This keeps the git history readable and makes it easy to see which
commits were exploratory vs final.

**Flesh out chores-*.md incrementally per `-devN`.** When a
multi-step plan starts, write the full chores section only for
the current `-devN`; list the remaining steps as a one-line
preview paragraph at the end. As each subsequent step starts,
fill in its own detailed section (with its edits, findings, and
checkmarks). The plan evolves as work happens, and speculative
detail for later steps usually needs rewriting by the time we
get there. See the `0.6.0` calibration block in
`chores-02.md` — dev1..dev6 were filled in progressively, not
planned in full at dev1.

## Todo format

Todo.md contains two main sections "Todo" and "Done" each item is a
short explanations of a tasks and links to more details using 1 or more
references.

Multiple references must be separated: `[2],[3]` not `[2,3]` or `[2][3]`.
In markdown, `[2,3]` is a single ref key (won't resolve) and `[2][3]`
is parsed as display text `2` with ref key `3` (so `[2]` won't resolve).

**Incremental `-devN` updates.** In a multi-step `-devN` flow,
each devN commit must also update this file: strike the entry
from `## In Progress`, add a bullet under `## Done` with a
`[N]`-style link pointing to the devN section in chores. Don't
batch the moves into the final release commit — the todo file
should reflect what's actually done as each step lands.

Examples:

# Todo

- Add new feature X [details](chores-01.md#feature-x)
- Fix bug Y [1]

# Done

- Fixed issue Z [2],[3]

[1]: chores-01bugs.md#bug-y
