# Agent-files size

The line count of the agent-files, one row per landing that changed them (wink, 2026-09-02: a
cycle that leaves the agent-files untouched adds no row), so the set's size is tracked over time.
Smaller is the quasi-goal: a rule stated once is shorter than a rule stated three times, and a
shrinking count is evidence the set is converging, while a growing one is a prompt to ask what
arrived as a paragraph that should have been a line. The count is not a rule, and a rule is never
cut to move it.

The count is `wc -l AGENTS.md custom.md agent-data/*.md`, taken at close-out and recorded here as
the closing rung's last edit, with the cycle title as the row's label.

## Counts

| Landed | Cycle | Files | Lines | Note |
|---|---|---|---|---|
| 2026-08-30 | docs: adopt the family agent-files set | 10 | 2109 | zc-ring-x1's copy at e1bc046c, minus messaging.md, plus the session-rule-identity proposal |
| 2026-09-01 | agent-files(adoption): from vc-x1, 2026-09-01 | 10 | 2158 | vc-x1's copy at 0872ccd8e1ed, the project-declared commit types. Row added 2026-09-02, the count taken from landmark 21ed19e8520c |
| 2026-09-02 | agent-files(adoption): v0.1.0 | 10 | 2230 | vc-x1's copy at 48d678c8efb4, the set versioned, plus the empty agent-files-v0.1.0 file the count does not see |
| 2026-09-04 | agent-files(proposal): v0.2.0 | 10 | 2231 | `## Closed` moved last in the Todo format list, with the `# References` bullet added and the re-pack rule's stale parenthetical dropped beside it |

Per file for the three most recent rows, newest on the left, the window sliding at each close-out
so the earlier history is in the commits. A column is labeled by the set version it carries, the
landings before the set was versioned relative to the first version (`- v0.1.0` one before it,
`-- v0.1.0` two before), and a landed local change carries a `-trailer` version.

| File | v0.2.0 | v0.1.0 | - v0.1.0 |
|---|---:|---:|---:|
| AGENTS.md | 369 | 369 | 363 |
| custom.md | 12 | 12 | 12 |
| agent-data/code.md | 92 | 92 | 92 |
| agent-data/commit-model.md | 42 | 42 | 42 |
| agent-data/cycle-model.md | 76 | 76 | 76 |
| agent-data/jj.md | 376 | 376 | 376 |
| agent-data/notes.md | 171 | 170 | 170 |
| agent-data/prose.md | 401 | 401 | 393 |
| agent-data/rationale.md | 488 | 488 | 456 |
| agent-data/versioning.md | 204 | 204 | 178 |
| total | 2231 | 2230 | 2158 |
