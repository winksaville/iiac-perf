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
