# Notes

This directory contains various notes and documentation related to the project.
Each file is organized by topic for easy reference.

The chores-NN.md files in [chores/](chores) and [done.md](done.md) are frozen
history: records of cycles run before the family agent-files set moved the
cycle record into [../TODO.md](../TODO.md) (its `## In Progress` and
`## Closed` sections). They are never appended, still linked. Short term tasks
and their status live at the repo root in [../TODO.md](../TODO.md), with the
long tail in [todo-backlog.md](todo-backlog.md).

User-facing documentation (usage, the report reading guide,
the config file) lives in [../docs/](../docs), with the README
as its front door; this directory holds the records and
rationale behind it.

Durable design analyses (measurement theory, error models,
decisions that outlive a cycle) live in
[design.md](design.md). Measurement results that outlive a
cycle (e.g. the thread-placement map) live in their own topic
files — [placement-map.md](placement-map.md). Known defects
awaiting a fix live in [bugs.md](bugs.md); durable
machine/session ops facts in [ops.md](ops.md); agent-file
findings gathered for family convergence in
[dogfood-log.md](dogfood-log.md). For users new
to jj see [jj-tips.md](jj-tips.md).

## Workflow and conventions

Agent-facing workflow and conventions live in
[`../AGENTS.md`](../AGENTS.md) and its `agent-data/` satellites:

- [Cycle protocol](../AGENTS.md#cycle-protocol): how a cycle runs, its record
  in `TODO.md`, bookmarks, committing and pushing.
- [Todo format](../agent-data/notes.md#todo-format) and
  [The In Progress block](../agent-data/notes.md#the-in-progress-block): the
  shape of `../TODO.md`.
- [Prose form](../agent-data/prose.md#prose-form): how durable text is
  written.
- [code.md](../agent-data/code.md): doc comments and `// OK: ...` on
  `unwrap*` calls.
- [versioning.md](../agent-data/versioning.md): the version scheme, the
  suffix, and the version-of-record.
- [rationale.md](../agent-data/rationale.md): every rule's why, under the
  mirrored heading.
