# Closed: the last cycle's record

The last finished cycle's record, whole. A cycle's closing replaces this file's `## Closed` with
its own block from [zDOING.md](zDOING.md) and this file's references with that block's, so the
last record stays here until the next one is finished. Earlier cycles are in each landmark
commit's copy of the record, in `TODO.md > ## Closed` before this file, and the cycles before the
rule in the frozen [notes/chores/](notes/chores) and [notes/done.md](notes/done.md).

## Closed

### chore: adopt vc-x1's cycle-record files

(wink, 2026-10-16) vc-x1 keeps its cycle record out of `TODO.md`: the running cycle and the
continuation notes in `zDOING.md`, the last finished cycle in `zCLOSED.md`, and `TODO.md` holding
what we might do and nothing else. iiac-perf carries the same agent-files, v0.2.9, `0 of 12
differ`, but kept the record in `TODO.md`, so the opening of the figures cycle was writing it in
the old place. Adopt vc-x1's three files now, before that cycle's first commit.

#### Deliberation

- A layout of project files, not an agent-files adoption: the trial lives in vc-x1's records, its
  agent-files still name `TODO.md`, and ours match them file for file.
  - So `zDOING.md` says it is a trial and names vc-x1's Todo entry, by sha-link, as the cycle
    that makes it the rule.
- vc-x1's files are the model, copied and edited (wink, 2026-10-16): the intros as vc-x1 has
  them, its trial note pointing at vc-x1's entry, and the block's order, the Todo as picked, then
  the Deliberation, the Solution, the Acceptance check, and the Ladder.
- The figures cycle pauses before its first commit, its bookmark published at `0b6b686` and left
  there, and its agreed ladder kept in a continuation note. Its opening moves the bookmark to
  `main`.
- `## Bugs`, a pointer to [bugs.md](notes/bugs.md), goes into `TODO.md`'s intro with the two new
  files, as vc-x1 did. `## Ideas` stays in `TODO.md`.
- The `agent-files(adoption): v0.2.9` record in `## Closed` is deleted, as an opening deletes it,
  and is in its landmark `0b6b686`.
- Single-step, with no dev rename: the change is to the records alone, and a rename made and
  restored in one commit does nothing.
- The title says what is adopted and from whom, so another member finds it by "adopt", and it
  names no agent-files version (wink, 2026-10-16): v0.2.9 was adopted the cycle before, and the
  layout is vc-x1's trial, not part of the set. The recipe under the Solution makes the commit a
  model for the other members.

#### Solution

`zDOING.md` takes `## Continuation notes` and `## In Progress` from `TODO.md`, and `zCLOSED.md`
takes `## Closed`, each with vc-x1's intro. `TODO.md` keeps `## Waiting`, `## Todo`, `## Ideas`,
and its references, and its intro names the two files and `notes/bugs.md`. The notes that said
where the record lives, [notes/README.md](notes/README.md) and [notes/ops.md](notes/ops.md), say
so now.

The recipe, for a member adopting the same files from this commit or vc-x1's:

1. Copy vc-x1's `zDOING.md` and `zCLOSED.md` to the work-repo root, and point the trial note in
   `zDOING.md` at vc-x1's Todo entry by sha-link.
2. Move `TODO.md`'s `## Continuation notes` and `## In Progress` into `zDOING.md`, and its
   `## Closed` into `zCLOSED.md`, retargeting each link into `TODO.md` from them as
   `TODO.md#<slug>`.
3. Take `## Continuation notes`, `## In Progress`, `## Closed`, and a pointer-only `## Bugs` out of
   `TODO.md`, and name the files in its intro, as vc-x1's does.
4. Repoint the notes that say where the record lives.
5. Check: `vc-x1 agent-files diff -c ../vc-x1`, the headings against vc-x1's, and a search for
   links to the moved sections, as the acceptance check below does.

#### Acceptance check

- `vc-x1 agent-files diff -c ../vc-x1` reports `0 of 12 differ`. Passed.
- The headings of `zDOING.md` and `zCLOSED.md` match vc-x1's, and `TODO.md`'s level-two headings
  are `## Waiting`, `## Todo`, and `## Ideas`. Passed, the level-one and level-two headings
  compared by `diff`.
- No file outside the agent-files and the frozen history links to `TODO.md#continuation-notes`,
  `TODO.md#in-progress`, or `TODO.md#closed`. Passed, a search for `](TODO.md#` and
  `](../TODO.md#` with those anchors finding none.

#### Ladder

- chore: adopt vc-x1's cycle-record files (done)

# References
