# Cycle checklists

Checklists for the moments where slips happen: committing, pushing, closing out. The full
protocol, with rationale and recovery procedures, is
[cycle-protocol.md](../notes/cycle-protocol.md); on any disagreement, that file wins. Re-read
the relevant checklist immediately before the action; don't run it from memory.

Universal file, shared with the template repository; a proposed change is edited here and
converges at the template ([Changing the agent-files](../AGENTS.md#changing-the-agent-files)).
Project-local content goes in [custom.md](../custom.md).

## The cycle at a glance

Every change runs as a **cycle**: Preparation (`X.Y.Z-0`, optional) -> Work commits
(`X.Y.Z-1`, `X.Y.Z-2`, ...) -> Close-out (bare `X.Y.Z`). The version suffix encodes the phase;
the scheme, and where the version-of-record lives, are in
[versioning.md](../notes/versioning.md). Read [cycle-protocol.md](../notes/cycle-protocol.md)
before any commit work, and before any push, cycle or not.

## Committing vs pushing

A cycle rung is committed *by* `vc-x1 push`; never pre-commit it with `jj commit`. Push's
commit stages commit both repos with the approved title/body and stamp each new commit's
`ochid:` trailer (see [ochid trailers](jj.md#cross-repo-linking-ochid-trailers)); a
pre-committed rung leaves `@` empty and push mints a stamped empty duplicate (bugs.md #6). In
an instruction, "commit", "push", and "commit + push" all mean `vc-x1 push`. A bare `jj commit`
is asked for by name ("local commit", "just `jj commit`") and is only for work that never
publishes (local-only saves and loop-and-squash intermediates), with no `ochid:`. The approval
around a push, interactive by default and waived only by an explicit scoped delegation, is the
cycle protocol's [Pushing policy](../notes/cycle-protocol.md#policy).

## Per-commit checklist

Every commit (Preparation, each Work commit, Close-out), per the protocol's
[Per-commit flow](../notes/cycle-protocol.md#per-commit-flow):

1. Mark the rung `(current)` in `TODO.md > ## In Progress`, as the first edit.
2. Do the work. On any deviation from the agreed plan, or any question, stop and surface it.
3. Flip `(current)` -> `(done)`, before validation and the commit.
4. Validate the artifact (the medium-specific commands are in [custom.md](../custom.md)).
   Skip-able for notes-only commits, mandatory at close-out.
5. Say "ready to commit" and stop. The user reviews the working-copy diff; iterate until
   complete.
6. Write the description: <=72-col conventional title, prose body (work repo: file-by-file,
   opening with the version-bump bullet). See
   [Commit description](../notes/cycle-protocol.md#commit-description).
7. Show title + body and stop for review. This review covers the push only when the user's go
   explicitly includes it.
8. On the user's go: `vc-x1 push <bookmark> --title "..." --body "..."`. Never pre-commit with
   `jj commit`; never hand-write `ochid:` trailers.

## Ladder (sub-cycle) checklist

Within a sub-cycle ladder, per the protocol's
[per-Work-commit contract](../notes/cycle-protocol.md#per-work-commit-contract-within-a-ladder):

1. `jj new -R .`: fresh empty `@`.
2. Do the commit's work.
3. Run the fast validation (named in [custom.md](../custom.md)). Non-negotiable.
4. `jj describe -m "..." -m "..." -R .`: scratch working title. This first-time authoring is
   the one permitted describe; the commit is never published and never carries a trailer.

Nothing on a ladder is pushed; the close-out squash
(`jj squash --from "<base>..@-" --into @ -u -R .`) collapses it, and `vc-x1 push` then
publishes the single commit.

## Before any push

- This specific push has the user's explicit approval. Approval of a plan that includes a push
  is not push approval; "commit and push" names the destination, not a waiver of the reviews.
  Only an explicit scoped delegation ("do all of X, don't check in") waives the stops, for that
  bounded task only; destructive ops still pause.
- Validation ran, and passed, after the last edit.
- Closing words are already written; nothing follows the turn's final push (next checklist).

## After the final push: hard stop

Once the turn's final push or bot-repo squash-push is invoked, do no further work: no
verification, no summary, no next-step offers, no edits, until the user speaks. Put all closing
words *before* the invoke. The harness rejects an empty turn, so it may force a visible token
after the tool returns; if so, emit a bare acknowledgment only (e.g. "landed"), never a summary
or more work. Post-push verification happens next turn at the user's direction. A standing
delegation makes intermediate pushes just steps; the hard stop lands on the turn's *final*
push. See
[After push or squash-push](../notes/cycle-protocol.md#after-push-or-squash-push-stop-and-wait).

## Close-out checklist

The cycle's last commit (bare `X.Y.Z`), per the protocol's
[Close-out](../notes/cycle-protocol.md#close-out):

1. Move the picked-up item from `## In Progress` to a one-line `## Done` entry with its chores
   `[N]` ref.
2. Finalize the chores section: sync the header to the final commit title (and every anchor
   back-reference), add design subsections; the intro and As-built rungs are already there.
3. Full validation, mandatory.
4. Update `notes/README.md` if functionality changed.
5. At push time, surface the shape options (squash / trapezoid / keep separate) and wait for
   the user's choice. The trapezoid recipe is
   [in the protocol](../notes/cycle-protocol.md#trapezoid-close-out-recipe); its step 4 is
   `jj git push`, not `vc-x1 push`.
6. Backfill the chores as-built ladder refs (and any remaining legacy `Commits:` lines) for
   the commits the previous push made permanent; never record a SHA that is not on a permanent
   branch.

## vc-x1 push behaviors to keep in mind

Three, independent of project language:

- **No checks of the project's own.** vc-x1 assumes nothing about a repo's contents beyond
  `.jj` and `.vc-config.toml`, and runs no build or tests. The medium's validation is the
  per-commit flow's job, run *before* invoking `vc-x1 push`. The one check that remains is
  `push-work` verifying the bookmark's remote refs are tracked, which is its own precondition.
- **Rerunning is safe.** Push keeps no state and cannot resume. Every stage no-ops when its
  work is already done, so a failed run is re-run, not resumed. If a run fails, push stops and
  reports; getting the repos back to a sensible state is the user's call, not something the
  tool infers.
- **ochid trailers** are injected by `vc-x1 push` itself, so don't hand-write them into the
  commit body or `--title`/`--body`.
