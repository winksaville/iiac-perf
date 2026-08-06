# Cycle protocol

This protocol uses [Prose form](../agent-data/prose.md#prose-form). It
contains instructions on how a commit cycle is accomplished.

The artifact a cycle produces is whatever the bot generates from
the conversation: code, prose, an image, a song, a screenplay.
The steps below use a Rust crate as the running example (the
cargo cycle, `Cargo.toml` versioning); substitute your medium's
equivalents. This project's manifest is recorded in
[versioning.md](versioning.md).

## Cycles

A cycle has three phases:

- **[Preparation](#preparation)**: the cycle's first commit, when it needs setup (a lightweight
  cycle omits it and starts at its first Work step). Sets up the cycle:
  - Bump the version-of-record (where it lives and the suffix scheme are project-specific; see
    [versioning.md](versioning.md)).
  - Pick up a `## Todo` item (typically the top-ranked, #1)
    into `## In Progress` as the cycle's
    [six provisional items](#preparation): title, problem
    statement, solution statement, acceptance check, ladder,
    deliberation.
  - Create the cycle's topic bookmark.
  - Nothing is opened in the chores file; the block is the
    cycle's only home until close-out moves it.
- **[Work-N](#work-n)**: the commits that implement the change. As many as the change needs;
  each runs through the [per-commit flow](#per-commit-flow).
- **[Close-out](#close-out)**: the cycle's last commit. Bookkeeping only:
  - Run the acceptance check and record what it showed.
  - Finalize the six items, then move the block into
    `notes/chores/`, which creates the
    [chores section](#chores-sections).
  - Write the `## Done` entry and clear `## In Progress`.
  - Land the cycle's bookmark.
  - Optionally update `notes/README.md` if functionality
    changed.

A cycle's commits are published to the project remote
either incrementally or as one batch at close-out; the
result must always be published at close-out. See
[Pushing](#pushing).

**Sub-cycles.** When a Work commit's scope grows enough to warrant its own ladder, it subdivides
into its own Preparation / Work / Close-out. The same three-phase shape applies recursively at
every depth, and a sub-ladder's rungs are titles like any other. See
[Step naming](#step-naming) for how a step is identified and
[Sub-cycle ladders](#sub-cycle-ladders) for the local-ladder mechanics.

## Chores sections

A **chores section** is a `##` section in
`notes/chores/chores-NN.md` recording landed work. In
general, every commit that lands on the permanent branch
should have a reference to it on a rung of its section's
as-built ladder in a chores file.

**A cycle's record has one home at a time.** While the cycle
runs it lives entirely in `TODO.md > ## In Progress`, as the
[six provisional items](#preparation) written at Preparation
and revised as steps land. At close-out the whole block
**moves** into the chores file, becoming that cycle's `##`
section. It is never written in two places, so there is
nothing to keep in sync and nothing to write twice.

The move is mechanical, four transforms and no rewriting:

- **Heading levels shift one deeper**: the block's `###`
  title becomes the section's `##`, and its `####` items
  become `###`. Anchors survive untouched, because GitHub
  slugs derive from the heading's *text*, not its level.
- **Rung refs renumber** into the destination file's `[N]`
  namespace (see [Reference numbering](../agent-data/notes.md#reference-numbering)).
- **Repo-root-relative links gain `../`**, since the block
  moves from the root into `notes/chores/`.
- **The block's own forward-looking notes are rewritten**,
  since they described a future that has now happened.

Two of those fail *silently*: a mis-renumbered ref and an
un-rebased link render as plain text or a 404 rather than
erroring. Check both by hand until a `validate-repo` exists.

A single-commit cycle's ladder is one rung, its close-out.
Rung placeholders are backfilled once the commit is
permanent (see [Commits backfill](#commits-backfill) below).

The move also appends the section's title-only
`- [<title>](#<anchor>)` entry to the file's
`## Table of Contents`.

Adopted from vc-x1, which trialled it through a full cycle
and kept it: the dual maintenance disappeared, and the
narrative did not thin out from being written in `TODO.md`,
which was the risk.

Fuller chores conventions (content rules, header sync,
design subsection pattern, rung / reference formatting)
live in
[agent-data/notes.md](../agent-data/notes.md#chores-conventions).

### Commits backfill

An as-built ladder rung cites its commit by SHA and records the version that commit carried, and
neither is stable until the commit lands on a **permanent branch** (`main`, or a long-lived
release/patch branch that won't be rewritten); a rebase or squash rewrites the SHA and may
renumber the version on the way. So:

- A rung is **written with the literal `[[N]]` placeholder and no version**.
- **Backfill once the commit is on a permanent branch**, where both are final. A commit can't
  record its own SHA (that would change the hash), so the fill always lands one push later:
  **each push backfills the rungs of the commits the previous push made permanent.** On a topic
  branch the sections instead wait until the branch lands, so nothing is ever written that a
  later rebase could invalidate.

Backfill replaces the placeholder with a file-local `[[N]]` ref, defined as the commit URL +
40-hex SHA in the file's `# References` (format in agent-data/notes.md
[Chores commit references](../agent-data/notes.md#chores-commit-references)), and writes the
version ahead of the title. Sections predating the ladder form keep their legacy `Commits:`
lines; backfill those where they exist. A section's `##` title matches its commit title, so a
rare deliberate rewrite of a permanent-branch commit re-syncs via `git log --grep "<title>"`.

The per-push cadence is a project choice, not dogma: a **per-close-out** model (recording a
cycle's SHAs at its close-out) is equally valid. The one invariant: what a rung records must be
permanent.

## Preparation

The cycle's first commit, when the cycle needs setup (a lightweight cycle omits it; see
[versioning.md](versioning.md#suffix-scheme)):

- **Bump the version-of-record.** Where it lives, the suffix scheme, and any derived files (a
  lockfile, a sourced manifest version) are project-specific; see
  [versioning.md](versioning.md).
- **Move a `## Todo` item** (if the cycle has one) into
  `## In Progress`, and write the cycle's **six provisional
  items** there. All six are required, all six are revised
  as steps land, and all six move to chores at close-out.
  The first is a `###` heading; the rest are `####` headings
  under it:
  - the **title**, which becomes the chores section header.
  - the **problem statement**: what is wrong, in a sentence
    or two.
  - the **solution statement**: what will be done about it,
    broad. Provisional here, since it is written before the
    work; the close-out's commit body carries the final one.
  - the **acceptance check**: the measure of "are you
    finished?". Not the per-commit validation, which asks
    whether the artifact still works; this asks whether the
    thing the cycle promised actually happened, specifically
    enough that a reader can run it.
  - the **ladder**: one rung per step, a bare title plus a
    `(current)` / `(done)` marker.
  - the **deliberation**: how the five above were decided,
    alternatives weighed, costs accepted. `_None._` when
    there was nothing to deliberate, which is a real answer
    and different from having forgotten to write it.

Nothing is opened in the chores file at Preparation. The
section is created at close-out by moving this block; see
[Chores sections](#chores-sections).

**Why an acceptance check, and why it is provisional.** A
cycle's own per-commit checklists can all pass while its
banner claim is false: vc-x1's seven-cycle program opened
against "end subprocess spawning" and its close-out claimed
the goal met, with about twenty spawn sites surviving, two
inside the facade the program built. Being provisional, the
check can also be revised *toward* what was achieved, which
is the same failure by a slower route. So a changed check is
one of the things the deliberation exists to justify.

## Work-N

The cycle's work commits implement the change. As many as needed:

- Each commit runs through the
  **[per-commit flow](#per-commit-flow)**.
- **Interim pushes** are optional (backup, progress
  visibility).
- Close-out is the only mandatory push (see
  [Pushing](#pushing)).
- **Subdivide into a sub-cycle** if a Work commit's
  scope grows enough (see
  [Sub-cycle ladders](#sub-cycle-ladders)).

## Close-out

The cycle's last commit does bookkeeping only, and the commit body describes that bookkeeping,
not what happens post-squash:

- **Run the acceptance check** the Preparation stated, and
  record what it showed in the block, whether or not it
  passed. A check that was never run is a failed close-out,
  and a check that failed is a finding, not a reason to
  quietly restate the banner.
- **Finalize the six items in place**, before the move: sync
  the title if the cycle's scope shifted, replace the
  provisional solution statement with what was actually
  done, drop the ladder's `(current)` / `(done)` markers
  since as-built implies done, and add any `####` design
  subsections the deliberation grew.
- **Move the block** into `notes/chores/chores-NN.md`,
  applying the four transforms in
  [Chores sections](#chores-sections). This *creates* the
  section; nothing was opened earlier.
- **Write the `## Done` entry**: the version, then a bold
  title line with its chores `[N]` ref and detail as
  sub-bullets (see
  [Done entry form](../agent-data/notes.md#done-entry-form)).
- **Replace the `## In Progress` block** with
  `_No cycle currently in progress._`.
- **Update `notes/README.md`** if functionality changed
  (new flags, new subcommands, changed behavior).

Whether to **squash** the cycle into one commit before the
publishing push, or push as-is, is decided at push time;
see [Pushing](#pushing).

## Step naming

A step has a title and no number. The ladder lists its rungs in order, so position is already
recorded by the list, and a step is referred to by its title, verbatim-identical in the ladder
rung, the chores `##` header and the commit (see
[agent-data/prose.md](../agent-data/prose.md#steps-are-named-not-numbered)). A title has to be
unambiguous within its cycle and within its chores file, where it is also an anchor; it may repeat
across the repo's history.

The version-of-record still bumps for every step, and its suffix still encodes the phase, but that
encoding belongs to the manifest and appears nowhere in prose. It is the one number left in the
system, it names nothing, and nothing dereferences it. The full scheme (disambiguation, nesting,
optional Preparation, the project's version-of-record format, and the per-phase bump) lives in
[versioning.md](versioning.md#suffix-scheme), which is the single source of truth for this repo's
versioning.

## Topic bookmarks are drafts

A topic bookmark is a draft until it lands on a permanent branch. Pushing to the bookmark makes
the work durable and visible; it does not publish it. Landing on the permanent branch is
publication, and that is the line the rules divide at:

- **Before landing**, the series should be self-consistent when practical. Inserting or
  reordering a step changes the ladder, and the rungs that already committed an older version of
  it are brought along, so the branch reads as one coherent ladder rather than a record of how it
  was assembled.
- **After landing**, the commits are history and are not touched. A recorded SHA is only ever
  written for a commit on a permanent branch (see [Commits backfill](#commits-backfill)), which
  is what makes rewriting a draft safe.

Mechanics, and why they cost so little here:

- **Amend content; never re-describe.** Editing `TODO.md` inside a rung and amending it is not a
  `jj describe`, so the never-re-describe rule stays intact. `ochid:` trailers survive, since
  they carry change ids rather than commit ids and a change id is stable across a rewrite.
- **Force-push the bookmark** afterwards, under the same approval any push needs.
- **Exceptions**, since "when practical" is not "always": the bookmark has already landed;
  another branch is stacked on it, so the rewrite becomes someone else's rebase; or the ladder is
  long and only a trailing snapshot disagrees. Name the reason and move on.

A squash-form [sub-cycle ladder](#sub-cycle-ladders) never meets this, because nothing on it is
pushed and the close-out squash collapses it. The rule is for the multi-commit shape, whose rungs
publish one at a time.

## Per-commit flow

Every commit (Preparation, each Work commit, Close-out) goes
through:

1. **Mark this commit `(current)`** as the first edit in
   `TODO.md > ## In Progress` (`TODO.md` is at the repo
   root).
2. **Do the work** (see [Iterative work](#iterative-work)
   for the loop-and-squash technique).
3. **Flip this commit `(current)` -> `(done)`** in `## In
   Progress`, before the cargo cycle and the commit.
4. **Validate the artifact**, a medium-specific step, skip-able
   for notes-only commits, mandatory at close-out. For the Rust
   example the cargo cycle is:
   1. `cargo fmt`
   2. `cargo clippy --all-targets -- -D warnings`
   3. `cargo test`
   4. `cargo install --path . --locked`
   5. (re-test if anything substantive changed)
5. **Work review.** Stop *before* writing any description;
   tell the user "ready to commit." The user reviews the
   changes and we iterate until complete.
6. **Write the commit description**; see
   [Commit description](#commit-description).
7. **Commit Description review.** Show the title + body
   and stop. The user reviews the description. Iterate.
8. **Commit.** `jj commit -m "title" -m "body" -R .` for
   the work repo, `-R .claude` for the bot repo (`-R` last
   keeps the verb visible):

   ```
   jj commit -m \
   "<type>: <short description>" \
   -m "<problem statement>

   <solution statement>

   - <a rule or outcome that followed>
   - <another>

   ochid: /.claude/<chid>" \
   -R .
   ```

**Two overrides apply:**

- **Deviation or question**: any time the work deviates
  from the agreed plan, or a question arises, stop and
  surface it; don't push through.
- **ESC-ESC**: the user can interrupt at any point to pull
  a review or question forward.

## Commit description

[Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <short description>
```

**A commit names no version**, in its title or its body. The version-of-record (where it lives
and its bump cadence, see [versioning.md](versioning.md)) is useful for confirming you are
running the version you are testing; it is not an identifier, and a commit already records it in
the manifest. Writing it into the description copies it into text that cannot be edited: a
version is only stable once it lands on the permanent branch, and even then a history rewrite may
renumber it, at which point every description naming it is wrong forever. See
[Versions live in the version-of-record only](../agent-data/prose.md#versions-live-in-the-version-of-record-only).

### Title

- <=50 chars total.
- Common types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`.
- Favor terse phrasings.
- **Distinct per step**: each of a cycle's commits gets its own descriptive title (no shared
  cycle title with a step marker). Share a greppable stem across the cycle's titles (e.g.
  `ring buffer`) so `git log --grep` collects them; the chores section header matches the
  close-out title.
- **Unambiguous where it is resolved**: the title is the only identifier a record has, so it must
  be distinct within its cycle and within its chores file (a `##` header is also an anchor). It
  may repeat across the repo's history. See
  [Steps are named, not numbered](../agent-data/prose.md#steps-are-named-not-numbered).

### Body

A **problem statement** then a **solution statement**, in
[Prose form](../agent-data/prose.md#prose-form) (intro + bullets), wrapped <=72. The problem
statement says what was wrong and defines any word the title assumes; the solution statement says
what was done about it. Content rules, including why the body carries no file list, are in
[agent-data/prose.md](../agent-data/prose.md#prose-form) and are not repeated here.

Two repo-specific points:

- **Work-repo body**: the problem is the artifact's or the records' problem.
- **Bot-repo (`.claude`) body**: the statements describe in-session activity rather than
  work-repo changes.

### Trailer

`ochid:` as the last line of the body; see
[Cross-repo linking (ochid trailers)](../AGENTS.md#cross-repo-linking-ochid-trailers)
in AGENTS.md for the convention.

For breaking changes, use the hyphenated `BREAKING-CHANGE:`
trailer key. `BREAKING CHANGE:` (with a space) is the only
space-separated key the Conventional Commits spec allows; the
hyphenated form is also valid and avoids the space ambiguity.

## Reviewing changes

Work review looks at the **uncommitted working-copy diff**,
on the way to commit. The user opens diffs in their
editor (Zed, VSCode); jj commands are for terminal:

- `jj diff`: working-copy diff (uncommitted)
- `jj diff -r @-`: diff of the previous commit
- `jj diff --from <X> --to <Y>`: any two revisions
- `jj show -r <X>`: description + diff for one rev

Don't `jj edit -r @-` to view a past commit, because that marks
it mutable and shifts `@`; use `jj diff -r @-` or
`jj show -r @-`.

See [Sub-cycle ladders](#sub-cycle-ladders) for the
close-out squash recipe and recovery; revset primitives
are in [`jj-tips.md`](jj-tips.md#revsets).

## Pushing

### Policy

Push is **discretionary** during the cycle (backup,
progress visibility) and **mandatory at close-out**, since
the cycle's result must be published.

**Approval is per-push.** Every push (any repo, any kind:
cycle push, interim backup, recovery/surgery force-push)
happens only after the user has reviewed the changes to be
published and explicitly approved that specific push.
Approval of a plan that *includes* a push does not authorize
the push itself; stop and ask again at the moment of pushing.

**Default is interactive; an explicit scoped delegation waives
the gates.** The gates above (per-push approval, the
commit-description review (show title+body and stop), and the
hard stop after push/squash-push) are the *interactive
default*.
They yield when the user **explicitly** delegates a complete,
bounded task and authorizes carrying it through ("do all of X
and push each step, don't check in"). The bot then proceeds
through that task's commits and pushes without stopping, and
continues past each push to the next step. Conditions:

- **Explicit grant**: never inferred from a task merely being
  well-scoped; the user's words must authorize unattended
  completion. "Commit and push" (or "then push") names the
  destination, not a waiver: it authorizes the push *after*
  the normal work review and description review, not skipping
  them. Only wording that explicitly waives the stops ("don't
  check in", "no need to review", "carry it through
  unattended") waives them.
- **Bounded goal**: covers the named task only; does not carry
  to the next task or a vaguer follow-on.
- **Destructive ops still pause**: delegation covers the task's
  ordinary commits and pushes; it does *not* pre-authorize a
  genuinely irreversible action (force-push over published
  history, history rewrite, deleting a remote branch). Those can
  permanently destroy work and aren't a normal cycle step, so the
  bot flags one before acting. An ordinary delegated cycle never
  reaches this.
- **Still transparent**: report each commit/push as it lands
  (title + outcome) so the user can catch up.
- **When in doubt, ask**: ambiguous authorization falls back to
  per-push approval.

### Shape at close-out push

At close-out the cycle's *work* is done; its *published
shape* is the remaining choice, made at push time. Surface
the options and get user approval before pushing. Once on
the target, changing shape is a remote rewrite (force-push,
needs approval), so choose deliberately.

- **Squash to one commit**: single entry on the target.
  Right for straightforward changes where the Work-N is
  focused on one or two files.
- **Merge non-ff** *(current default)*: `main` gains the close-out as a merge commit; cycle
  commits stay reachable via two parents. `jj log -r ..@ -n <N>` shows the trapezoidal shape. See
  [Merge non-ff recipe](#merge-non-ff-recipe) for the full setup sequence.
- **Keep separate**: one commit per cycle entry on
  `main`. Use when the decomposition itself is
  informative. Each chores section keeps its own header /
  `Commits:` ref; no consolidation churn.

Set up squash/merge before invoking `vc-x1 push`; use
`jj git push` directly for non-standard shapes.

### Merge non-ff recipe

Setting up a [Merge non-ff](#shape-at-close-out-push)
close-out is a fixed sequence. `<closeout>` is the cycle's
close-out commit, `<prev>` the previous cycle's close-out
(the current `main` tip), `<work-tip>` the cycle's last
Work commit:

1. **Rebase the close-out into a merge**: `jj rebase -r
   <closeout> --onto <prev> --onto <work-tip>`.
   - `-r <closeout>` keeps the `<closeout>` commit in place.
   - `--onto <prev>` becomes its first parent (trunk).
   - `--onto <work-tip>` becomes the second parent.

   Together these make `<closeout>` a merge of
   `<prev>` + `<work-tip>`, forming a trapezoidal commit.
2. **Use `jj new <merge>`** to add an empty `@` above the
   merge. The rebase left `@` *on* the now-content-bearing
   merge, which git/IDE diff views show as uncommitted;
   `jj new` restores the clean empty `@` on top.
3. **Push**: `jj git push --bookmark main -R .`.

**Post-hoc caveat.** If the cycle was already pushed
[Keep separate](#shape-at-close-out-push) its commits are
immutable: the rebase needs `--ignore-immutable` and the
push force-updates `main`.
The standard sequence assumes the merge is set up *before*
the close-out push.

**Viewing the result (not a push step).** After merge
non-ff close-outs land, read the history with
`jj log -r ..@ -n <N>`: the graph renders each cycle as its
trapezoid, with close-out titles down the trunk's left edge and
the cycle's Work rungs indented on the side leg, so trunk and
internals are both visible at once. For git-side tooling
(which lists commits flat instead of drawing the graph),
`git log --first-parent` recovers the trunk view: one
close-out merge per cycle, rungs skipped; plain `git log`
interleaves every cycle's rungs into one long list.

### vc-x1 push wrapper

`vc-x1 push <bookmark>` wraps per-push mechanics. See
`vc-x1 push --help` for current flags. `<bookmark>` names a
work-repo bookmark only; the bot repo is always pinned to
`main` (see [.claude cadence](#claude-cadence)).

**Current limitation**: only fully supports the
[Keep separate](#shape-at-close-out-push) shape; other
shapes need manual jj steps. Planned improvements are
project state, tracked in the project's `TODO.md`;
this protocol describes only the stable mechanism.

### .claude cadence

**Cadence**: one push = one bot-repo commit, paired
with every work-repo commit in that push.

The `.claude` working copy accumulates session data
across the cycle; its change ID stays stable across
snapshots, `jj describe`, and the squash-push fold, so
work-repo `ochid:` trailers resolve.

`.claude` is a linear journal: all session work lives
on `main`, regardless of the work-repo bookmark. **Do
not create or maintain bot-repo bookmarks that mirror
work-repo branches**, which risks the bot steering session
pushes to the wrong remote ref.

Ending a session: if the user runs `/exit` there will be
session information created, which we don't worry about.
The user can close the terminal instead and `@` will
remain empty.

### Bot communication at the reviews

Use plain prose, no insider jargon ("Gate N signal",
"Checkpoint N", etc.):

- **At Work review**: summarize what changed and stop.
  "Work complete. Please review."
- **At Commit Description review**: present `$TITLE`
  and `$BODY` explicitly; ask permission to commit/push.
  Don't spell out the full `vc-x1 push ... --title ...
  --body ...` invocation by default.
- **At Post close-out review**: surface the shape
  options (squash / merge / keep) and the push target;
  wait for the user's choice before any `jj squash` /
  `jj rebase` / `jj git push` invocation.

### After push or squash-push: stop and wait

After a **push** (crossing the remote boundary, by hand or
via the `vc-x1 push` wrapper, whose last stage publishes
the bot repo too) or a manual **squash-push** on the bot
repo, stop for the turn: no next step, edit, tool call, or
text output until the user directs otherwise. **Even when
the next step seems obvious, wait.**

- **Scope**: the stop follows the user's directive, not the
  push. A standing directive covering more work ("finish
  the remaining ladder commits on your own") makes an
  intermediate push just a step; the hard stop lands on the
  turn's *final* push.
- **Why**: the bot repo is a live journal, so everything after
  the invocation (its own record, closing words) lands in
  `@` as a trailing tail. Between delegated pushes the tail
  rides into the next cycle's bot commit; the final push's
  tail has no next commit, and the bot's own squash-push is
  itself session data (`@` refills immediately), so only the
  user, after the turn, can capture it
  (`vc-x1 squash-push -R .claude`).
- **Silence**: put all closing words *before* the final
  push. The harness rejects an empty turn, so it may force a
  visible token after the tool returns; if so, emit a bare
  acknowledgment only (e.g. "landed"), never a summary,
  verification, or next-step offer. There is no "harmless"
  closing line after the push; that is a known slip.
- **Flush**: when the user wants `@` empty (no tail), they
  run `vc-x1 squash-push -R .claude` after the bot goes
  quiet. It flushes all bot session information into the
  published commit. Repeat if new writes land (see
  [Recovery](#recovery)).

### Recovery

- **If push exits before its last stage**, meaning `push-work`
  succeeded but the bot-repo publish didn't run
  (`squash-push-bot` in `vc-x1 push --status` / `--from`
  stage names), run the squash+push by hand:

  ```
  vc-x1 squash-push -R .claude
  ```

  It runs in-process, so a failure is a visible non-zero
  exit, with no log file to chase.
- **Run squash-push again if `@` is non-empty** after a
  pass (also desirable after extra activity by the bot's
  agents).
  - Why: the bot keeps writing session data while the
    command runs: the invocation's own record plus any
    closing response land after the squash.
  - Safe to repeat: bot session data is append-only, so a
    re-run never conflicts or overwrites. (This could
    change; it is not under the user's control.)
  - No guarantees: events outside the bot's control can leave
    `@` non-empty. The bot's back end may decide to
    squash/consolidate session data, which can take minutes
    and land after the pass. The remedy is the same: just
    run squash-push again. This is why a single pass is never
    guaranteed to leave `@` empty.
- **Clear push's saved state** after any out-of-band
  recovery, via `rm .vc-x1/push-state.toml` or `vc-x1 push
  <bookmark> --restart`; otherwise push resumes from a
  stale stage.
- **Late work-repo tweak after the work-repo push succeeded**
  (e.g. updating AGENTS.md or memory) requires `jj
  squash --ignore-immutable` and a re-push; that is a
  remote rewrite and needs explicit approval like any
  push.
- **`vc-x1 push` after a manual merge setup published an
  empty commit** (seen at the 0.22.0 close-out). `vc-x1
  push` *creates* the work commit itself from pending
  changes + `--title`/`--body`; it does not publish an
  already-committed shape. Invoked after the
  [Merge non-ff recipe](#merge-non-ff-recipe) (work repo
  clean, merge in place), it minted an **empty** commit on
  the old `main` tip, stamped the ochid on it, and pushed
  that, leaving the merge unreferenced and `main` without
  the cycle's content. Prevention: after a manual shape
  setup, push with `jj git push --bookmark main -R .`
  directly (the recipe's step 3), never `vc-x1 push`.
  Recovery, in order (nothing is lost, the merge still
  exists):
  1. `jj describe <merge>` to append the
     `ochid: /.claude/<bot-chid>` trailer the stray got
     instead (rewrites the merge's commit id; its chid,
     what the bot side references, is stable).
  2. `jj abandon <stray> --ignore-immutable`.
  3. `jj bookmark set main -r <merge> --allow-backwards`,
     `jj new <merge>`, then `jj git push --bookmark main
     -R .` (remote rewrite, needs approval).
  4. Bot repo: `jj describe <bot-commit> --ignore-immutable
     -R .claude` to point its `ochid:` at the merge's chid,
     then `jj git push --bookmark main -R .claude`
     (restores the `main == main@origin` preflight
     invariant).

  The recovered shape, `jj log -r ..@` (0.22.0, elided):

  ```
  @  smpozlop ... (empty) (no description set)
  ◆    yzvlvtku ... main d9bb5882
  ├─╮  fix: calibration robust to codegen and noise
  │ ◆  ztxvxuru ... fix-calibration f4b155a8
  │ │  feat: always-on calibration self-checks
  │ ~  (three more rungs)
  │ ◆  tskxkxsk ... 6d5784de
  ├─╯  fix: pair frame/call against a loop-only pass
  ◆  sktyvwrq ... f006b09e
  │  feat: amortized + cached calibration
  ```

## Iterative work

When work for a single commit (the **target**) benefits
from incremental review, loop:

1. `jj new -R .`: fresh empty `@` on top of the target.
2. Make the next round of changes.
3. User reviews the round (see
   [Reviewing changes](#reviewing-changes)).
4. `jj squash -R .` folds into the target and creates a
   new empty `@`.
5. If not done, go to step 2.

Same jj mechanics as a
[sub-cycle ladder](#sub-cycle-ladders), but at
single-commit scope, so the version
doesn't change.

## Sub-cycle ladders

When a Work commit subdivides into a sub-cycle (see
[Step naming](#step-naming), and [versioning.md](versioning.md#suffix-scheme) for how the
manifest's suffix nests), its Work
commits typically live as a local jj `@` chain and
**collapse into the sub-cycle's Close-out** before the
parent cycle continues. Ladder commits are scratch,
for review and bisection only.

### Per-Work-commit contract within a ladder

For each Work commit in the ladder:

1. `jj new -R .`: create a fresh empty `@`.
2. Do the commit's work.
3. Run the fast validation (Rust example: `cargo test
   --bins`). **Non-negotiable**: for code, build and clippy
   alone miss regressions until a later commit runs the full
   suite, raising bisection cost.
4. `jj describe -m "..." -m "..." -R .`: working title
   only; the sub-cycle Close-out collects everything
   into one final commit.

### Navigating the ladder

Common moves:

- `jj log -r '<base>::' -R .`: see the whole ladder
  from its base.
- `jj edit -r <prefix> -R .`: jump `@` to any ladder
  commit by chid prefix; useful for bisection.
- `jj edit @-- -R .`: quick-jump back two commits.
- `jj diff -r <chid> -R .`: review one commit in
  isolation.

Modifications to any ladder commit rewrite it in place;
descendants auto-rebase.

### Close-out: squash the ladder

When all ladder Work commits are done and tests pass:

```
jj squash --from "<base>..@-" --into @ -u -R .
```

`<base>` is the parent of the first ladder commit; `-u`
keeps `@`'s description and discards the sources'.
After squash, history is linear: `<base> -> @`;
intermediate commits are auto-abandoned.

Then `vc-x1 push <bookmark>` as for any other commit.

For N = 1 the squash is a no-op (`<base>..@-` is empty
when `@-` is `<base>`); push the single commit directly.

### Recovery

If a ladder commit goes wrong, back out without losing
prior commits:

- **Discard the current commit.** `jj abandon @ -R .`
  drops it; you get a fresh empty `@` on the same
  parent.
- **Edit an earlier commit.** `jj edit -r <chid> -R .`,
  make corrections, then `jj edit -r <last-ladder-chid>`
  to return. Descendants auto-rebase.
- **Discard the entire ladder.** `jj op log -R .` shows
  the op history; `jj op restore <op-id> -R .` reverts
  to that point. Full undo: removes *all* ladder work
  after the chosen op. Use only to start over.

# References

- [`jj-tips.md`](jj-tips.md#revsets): revset primitives
  (chid/cid, `@`/`@-`/`@+`, `..`/`::` ranges, prefix matching).
- The per-commit `cargo test --bins` gate exists because a
  regression introduced in an early ladder commit can go
  uncaught until a later commit runs the full suite, raising
  bisection cost.
