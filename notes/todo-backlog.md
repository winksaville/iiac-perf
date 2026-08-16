# Todo Backlog

This file uses [Prose form](../agent-data/prose.md#prose-form). It holds lower-priority
`## Todo` entries, the long tail. When an entry becomes a priority, move it (and any refs it
cites) into `../TODO.md > ## Todo` at its priority rank (the list is strict-ranked, #1 highest),
then `fix-todo` to renumber.

Same formatting rules as `../TODO.md > ## Todo`. See
[Todo format](../agent-data/notes.md#todo-format). Run
`vc-x1 fix-todo --no-dry-run notes/todo-backlog.md` to renumber.

## Todo

1. Decide the three parked bookmarks that still hold unlanded work: `web-claude-tweaks`,
   `measure-reproducibility` and `ffq-spsc-notes`
   - measured 2026-08-13, `main..<bookmark>` per bookmark: `web-claude-tweaks` holds one commit
     (band-table bug fixes and bucket `debug_assert!()`s, 2026-07-24), `measure-reproducibility`
     three (a cycle opened and abandoned mid-ladder, 2026-08-04), `ffq-spsc-notes` one (an SPSC
     Q&A archive, 2026-07-28)
   - so each is land, rework, or discard, and only their owner can say which. The names are all
     that keep those commits reachable
   - `punctuation-sweep` and `fix-calibration` were swept on 2026-08-13, both fully merged into
     `main` with nothing unreachable. `punctuation-sweep` was local-only, never having reached
     `origin`
   - the deletions publish in one push, `jj git push` naming each with a repeated `-b`
     ([chores-07](chores/chores-07.md#landing-measured))
2. Policy pins, parameters stay: move the rules living in `custom*` into the pinned agent-files,
   leaving the layer holding only declared values (raised 2026-08-16, wink review pending, then
   express our opinion to vc-x1)
   - the test for any line in `custom*`: what would a diff between two members mean?
     - a *disagreement* (how to work) is policy, and belongs pinned, where the diff is a
       reviewable proposal
     - an *identity* (a name, a path, a command string, a width) is a value, and only that kind
       of line belongs in the layer
   - wants one new pinned convention, stated once
     - a pinned rule may reference a parameter the project layer declares
     - an undeclared parameter leaves the rule inert, the same shape as versioning.md's medium
       conditionals
   - the migration it licenses
     - the Messaging policy moves pinned, parameterized by the messages repo path and the member
       name, which dissolves the 2026-08-07 dead-text objection that moved it out of `AGENTS.md`
     - membership stays parameters, parked awaiting `vc-x1 config` keys
     - the dogfood log is a per-member record, out of pinned regardless, `notes/` its natural
       long-term home
   - two precedents already model the split
     - validation: the pinned checklists own the slots and the exit-status rule, and
       `custom-family.md` fills the slots with this project's commands
     - vc-x1's `vc-config.md` against `.vc-config.md` is the same schema/instance shape, here
       proposed for the instruction set itself
   - a reversal of a rule vc-x1 took from our 2026-08-07 proposal, so the message to them argues
     the reversal, not just the convention
