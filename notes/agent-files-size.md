# Agent-files size

The line count of the agent-files, one row per landing that changed them, so the set's size is
tracked over time. Smaller is the quasi-goal: a rule stated once is shorter than a rule stated
three times, and a shrinking count is evidence the set is converging, while a growing one is a
prompt to ask what arrived as a paragraph that should have been a line. The count is not a rule,
and a rule is never cut to move it.

The count is `wc -l` over `AGENTS.md`, `custom.md`, and `agent-data/*.md` less `rationale.md`,
since the rationale is the rules' why and grows with every rule that gains one, taken at the
close-out of a cycle that changed an agent-file and recorded here as the closing rung's last
edit, with the cycle title as the row's label. A cycle that touched no agent-file adds no row,
so the table is the history of the count and nothing else. Rows before 2026-09-16 counted
`rationale.md` too.

## Counts

| Landed | Cycle | Files | Lines | Note |
|---|---|---|---|---|
| 2026-08-30 | docs: adopt the family agent-files set | 10 | 2109 | zc-ring-x1's copy at e1bc046c, minus messaging.md, plus the session-rule-identity proposal |
| 2026-09-01 | agent-files(adoption): from vc-x1, 2026-09-01 | 10 | 2158 | vc-x1's copy at 0872ccd8e1ed, the project-declared commit types. Row added 2026-09-02, the count taken from landmark 21ed19e8520c |
| 2026-09-02 | agent-files(adoption): v0.1.0 | 10 | 2230 | vc-x1's copy at 48d678c8efb4, the set versioned, plus the empty agent-files-v0.1.0 file the count does not see |
| 2026-09-04 | agent-files(proposal): v0.2.0 | 10 | 2231 | `## Closed` moved last in the Todo format list, with the `# References` bullet added and the re-pack rule's stale parenthetical dropped beside it |
| 2026-09-05 | agent-files(adoption): v0.2.2 | 10 | 2255 | vc-x1's copy at 59db117ed2f5, the agent-repo located by `.vc-config.md` rather than asserted, `v0.2.1` skipped as superseded |
| 2026-09-07 | agent-files(proposal): v0.2.3 | 10 | 2315 | `## Reference numbering` rewritten naming no file, punctuation conversion paid in a penultimate rung, continuation facts filed before the reset, pushed titles kept through a rename, a waiver's scope recorded, the `#[allow]` obligation tied to the lints, the dual-repo model simplified, the agent-files version tending to the patch, each with its why in `rationale.md`, which carries 53 of the 60 new lines |
| 2026-09-12 | feat: merge batches into blocks | 10 | 2315 | the `agent-files(adoption): v0.2.4` rung, vc-x1's copy: `custom.md`'s messaging clause reworded in place and the version marker, so no count moves. Taken at close-out, before Land |
| 2026-09-16 | agent-files(adoption): v0.2.5 | 9 | 1774 | zc-ring-x1's set at 1bbc72b2eb70, a size row only when an agent-file changed and `rationale.md` out of the count, so the files drop to 9 and the total from 2315 less its 541 |

Per file for the three most recent rows, newest on the left, the window sliding at each close-out
so the earlier history is in the commits. A column is labeled by the agent-files version it carries, the
landings before the set was versioned relative to the first version (`- v0.1.0` one before it,
`-- v0.1.0` two before), and a landed local change carries a `-trailer` version.

| File | v0.2.5 | v0.2.4 | v0.2.3 |
|---|---:|---:|---:|
| AGENTS.md | 384 | 384 | 384 |
| custom.md | 12 | 12 | 12 |
| agent-data/code.md | 94 | 94 | 94 |
| agent-data/commit-model.md | 42 | 42 | 42 |
| agent-data/cycle-model.md | 76 | 76 | 76 |
| agent-data/jj.md | 391 | 391 | 391 |
| agent-data/notes.md | 166 | 166 | 166 |
| agent-data/prose.md | 405 | 405 | 405 |
| agent-data/rationale.md | (543) | 541 | 541 |
| agent-data/versioning.md | 204 | 204 | 204 |
| total | 1774 | 2315 | 2315 |
