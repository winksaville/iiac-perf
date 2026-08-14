# custom-family.md - iiac-perf's layer, as a member of the vc-x1 agent-file family

Read after [custom.md](custom.md), whose single convention entry points here. Present only in a
member repo: a project that is not a member does not carry this file, and nothing in it applies to
one.

**A member's `custom.md` differs from the payload by that one line, and everything of its own lives
here.** That is the family's convention and it buys a property worth having: `diff custom.md
<template>/work/custom.md` is one line for every member, so the file that can never be pinned still
converges, and a member's whole customization surface is this file. The medium below is here for
that reason rather than because it has anything to do with the family.

## Medium and validation

The artifact is the `iiac-perf` CLI, a Rust crate (manifest `Cargo.toml`, package name
`iiac-perf`), with versioning specifics in [versioning.md](agent-data/versioning.md).

**What a version bump promises**: nothing to a dependent, because there are none. `iiac-perf` is
a binary crate with no library target and no external consumers, so `X.Y.Z` is a scope signal to
readers of the history (see
[Advancing X.Y.Z](agent-data/versioning.md#advancing-xyz-scope-decides)) and not a compatibility
contract. Revisit with a compat clause if a library crate ever splits out.

- **Full validation**
  - when: per-commit checklist step 5
  - run as separate invocations, each exit status checked:
    1. `cargo fmt`
    2. `cargo clippy --all-targets -- -D warnings`
    3. `cargo test`
    4. `cargo install --path . --locked`
    5. (re-test if anything substantive changed)
- **Fast validation**
  - when: ladder checklist step 3
  - `cargo test --bins`

The commands are the project's. The rule that each invocation's exit status is checked is
universal and lives in [AGENTS.md](AGENTS.md#working-practices).

## Membership

- **member name**: `iiac-perf`
- **template repository**: `../vc-x1-template`

These two are environment rather than instruction, the same species as `.vc-config.toml`'s
`[repos]` paths, and they belong there instead. They cannot move yet: `vc-x1 config --validate`
rejects keys it does not know (measured on 0.78.4, 2026-08-07), so a config carrying them would
fail its own validator.

## Messaging

Members leave word for each other in per-member mailboxes at the template repository. The protocol
is `../vc-x1-template/MESSAGES.md` and it governs. These are the parts that decide how a session
behaves.

- **At acquaint, check `../vc-x1-template/messages/iiac-perf.md`.** An absent file means no mail.
- **Handle, then delete** the entry, and delete the file once it empties. Mailboxes hold open items
  only.
- **So a message can never be a record.** Anything in one worth keeping is copied into
  `notes/chores/chores-NN.md` *before* the entry is deleted. Learned 2026-08-05, when a
  convergence message carried measurements that would have been lost with it.
- **Messages are thin pointers, not state.** Durable coordination state lives in topical files,
  and a message says "action needed, see <file>" rather than restating the details.
- **Write to a member's mailbox, never into their repo.** A repo with a live session is written
  only by its own agent.

## Dogfood log

Dated entries on what this project is trying and where the pinned instructions chafed, failed, or
got amended: the evidence base for the family's convergence decisions
([Changing the agent-files](AGENTS.md#changing-the-agent-files)).

Each entry carries a **status**: `proposed` (in our agent-files, awaiting the family),
`adopted family-wide`, `rejected`, or `permanently local`. In-flight entries only: on
resolution an entry retires to [done.md](notes/done.md) with its narrative in
`notes/chores/chores-NN.md`, adopted and rejected alike. Entries predating the status
convention keep their form until touched.

- 2026-08-14 (`proposed`): the semicolon allowance goes, and the agent-files sweep to zero
  - the pinned rule blessed the "between equals" join and left everything else a judgment call.
    wink's observation: agents take advantage of the exceptions, so the allowance is claimed
    wherever a semicolon is wanted
  - the rule is now flat: prose carries no semicolons, and a semicolon appears only in code
    (code spans, fenced code, source files), where it is syntax. Each prose site converts to a
    period, a comma with a conjunction, or sub-bullets
  - the agent-files (custom* included) carry no historical exemption and sweep to zero. Any
    other historical file keeps its semicolons only until altered, and altering one means
    asking the user whether they should go (wink, tightened mid-rung from a silent
    convert-when-touched). Excluding src/'s ~125 existing comment-line joins from this cycle's
    sweep is wink's call, keeping it focused on agent-file convergence
  - the rule tightened is one vc-x1 wrote and swept at ~140 sites. Proposed to them by message
    after this cycle lands, with their own instruction back: review the rule, not each instance
  - rationale in
    [chores-07](notes/chores/chores-07.md#docs-semicolons-leave-the-agent-files)

- 2026-08-12 (`proposed`): validation runs at every commit, and the notes-only skip goes
  - the per-commit checklist stamped the version-of-record at step 4 and let step 5 be skipped
    for notes-only commits, so a commit could carry a version that no build ever had. Measured
    the same day: 0.24.5 and 0.24.6 both stamped and neither built, and `-V` answered 0.24.4
    until the next close-out jumped it three
  - wink's rule and his reason: every commit bumps the version precisely so a build exists
    carrying it, which makes an unbuilt bump a version nobody can run and the banner a claim
    nobody checked
  - so the skip goes at all three sites, and the step is conditioned on the medium rather than on
    the commit: run the artifact if the medium has one to run (wink). A first draft made the
    escape "too costly to build", which asks every project to judge its own cost and would be
    claimed by anyone who found validation tedious
  - each of the three sites gets one job so the rule is written once (wink): the checklist
    instructs, the protocol holds the reason and the medium condition, and `custom.md` holds the
    commands. A first pass had the checklist and the protocol carrying the same sentence
  - the same pass fixed a step number this morning's sync left stale, and dropped a
    cycle-at-a-glance clause that named validation as a close-out specialty. The stale number is
    the argument in miniature: it was a restatement, which is what let it drift unnoticed
  - rationale in [chores-06](notes/chores/chores-06.md#docs-validate-every-commit)

- 2026-08-07 (`proposed`): the family layer splits out of `custom.md`
  - `custom.md` had been accumulating things that only make sense because this repo belongs to a
    family: a member name, a template path, and a dogfood log whose status vocabulary reads
    `adopted family-wide`. wink's test is that `custom.md` should be usable as-is by a project that
    has never heard of us, and it failed that test
  - so the family layer moves here, and `custom.md` shrinks to a stub with nothing to substitute:
    a title naming no project, and one section. A member's copy differs from the payload's by the
    single conventions entry that points here, so `diff custom.md <template>/work/custom.md` is one
    line for every member
  - the pointer is that entry rather than an intro sentence (wink). We argued for the intro, on the
    grounds that four members carrying an identical one-line diff looks like a fake divergence.
    The answer is that it is a real project convention, and one line is cheaper than a paragraph
    every member has to keep in sync
  - the chain is `AGENTS.md` -> `custom.md` -> here, all of it prose, which is what lets the pinned
    set stay ignorant of the split. A non-member has no entry and no file
  - `CLAUDE.md` collapses to one line, `@AGENTS.md` (wink). It had grown a second `@` import per
    layer, which made it a second statement of what to read and therefore a second thing to keep
    true. Now `AGENTS.md` is the only source of that truth, and `CLAUDE.md` is payload-identical for
    every project, member or not. The cost is real and worth stating: nothing below `AGENTS.md` is
    auto-loaded any more, so hard rule 0 is load-bearing rather than a formality
  - one consequence needs the family's agreement, not ours: **"Check the mailbox at acquaint" leaves
    the pinned `AGENTS.md`** and lands in `## Messaging` above. vc-x1 pinned it in the 20260802
    snapshot, so this is a reversal. The argument is that a non-member cannot perform it at all,
    since building the mailbox path needs a member name and a template path they do not have, so it
    is dead text naming concepts they lack rather than a harmless no-op
  - deliberately not opened in the same pass: `AGENTS.md` still says "the family" and "member" where
    the generic mechanism is "the template payload and my copy of it". Same inconsistency one level
    up, and folding it in would roughly double the review surface of an already large configuration

- 2026-08-07 (`proposed`): cycles run on a bookmark, and `custom.md` stops holding rules
  - the bookmark rule had been adopted in principle since 2026-08-01 and was written down
    nowhere: `TODO.md` carried the intention, `cycle.md` said what a topic bookmark *is* once you
    have one, and neither creating nor landing one appeared in any agent-file. Two cycles ran on
    bookmarks anyway, on undocumented habit
  - so hard rule 13 states it, `cycle.md` gains an opening checklist and a land step, and
    `jj.md` gains the commands. Landing turns out to be the interesting half: it is the moment
    the cycle's commits become permanent, which makes it the trigger for the chores backfill
    that waits on permanence. That connection existed and had no owner
  - the create and land *commands* are deliberately separable from the rule, because wink
    expects a `vc-x1 start-change <bookmark>` to own the create half eventually
  - same pass, `custom.md` empties of conventions. Five entries were rules the family already
    has or should have, so they moved into the pinned files (two into `prose.md`, one into
    `versioning.md`, one into `AGENTS.md`) and two retired as answered
  - the sharpened convention behind it (wink): **intent decides the file, and nothing gates the
    edit**. A member writes a rule into its local pinned copy whenever it means the family to
    take it, without asking. `custom.md` is for what the member does *not* offer the family. The
    2026-08-05 entry below framed the same mechanism around experiments, which read as narrower
    than it is
  - cost, and it is the same one the 2026-08-05 entry recorded: our pinned files now hold more
    unreviewed text than before, and nothing at acquaint shows a reader which parts the family
    has agreed to. The diff is the answer and reading it is still manual

- 2026-08-05 (`proposed`): a step is a title, and the version and the step number both leave the
  prose record
  - the version was a second identifier for a step, and any prose naming one could be invalidated
    by a history rewrite. This repo's 2026-08-01 renumber is the evidence: it left transcripts and
    pasted reports that only the decoder entry below can read
  - so the title becomes the identifier, a ladder rung carries neither a number nor a version, and
    the version-of-record is a build stamp living only in `Cargo.toml`. Every step still bumps it
  - the step number went too (wink): a rung sits in an ordered list, so a number beside it restates
    the position and then has to be maintained. Nothing renumbers, and a title need only be
    unambiguous within its cycle and within its chores file, where it is also an anchor
  - one exception, the chores as-built rung (wink): it records the version a *landed* commit
    carried, beside that commit's SHA, so the pair decodes an old `-V` banner. It takes the SHA's
    timing exactly, so an unlanded rung carries neither and a rebase cannot falsify a ladder
  - commit bodies tighten with it: a problem statement then a solution statement, both broad, with
    no file list, since the diff is already the mechanical record. The deliberation stays in
    chores, todo, and the session the `ochid:` trailer names, which is what the dual-repo model
    is for
  - that rule also took two passes. The first kept the body an edit list at one bullet per distinct
    change and survived one use, since writing this commit's own description produced thirteen
    bullets restating `git show --stat`. Clearing it exposed four contradictions of our own,
    including a title limit that read 50 in the authority file and 72 in three others
  - a topic bookmark is a draft until it lands, so keeping its ladder self-consistent may rewrite
    unlanded rungs, and the exceptions are named rather than judged case by case
  - first cost measured, on this step itself: the version had been doubling as the eye's landmark
    in `## Done`, so removing it made the section hard to skim. `## Done` entries became a bold
    title plus sub-bullets, which is what `prose.md` had asked for all along and the version had
    been masking. We think other members will hit this wherever a long body hangs off a title in a
    flat list. Superseded in part 2026-08-07: vc-x1 wanted the version greppable, and the form is
    now the version ahead of the bold title, which serves both
  - drafted before the `measure-reproducibility` rebase so the cycle dogfoods it, per
    [Changing the agent-files](AGENTS.md#changing-the-agent-files), with rationale in
    [chores-06](notes/chores/chores-06.md#docs-steps-are-titles-versions-are-stamps)

- 2026-08-05 (`proposed`): experiments move into the local agent-files, and `custom.md` narrows
  to what cannot be family-wide (superseded in scope by the 2026-08-07 entry above, which drops
  "experiment" for "anything the member offers the family")
  - the old rule sent every proposal to `custom.md` as an override, which made `custom.md` the
    staging area and guaranteed it grew non-generic, while the shared payload was the only place
    two members could collide
  - now the diff against the payload is the live proposal set and the commit history is the
    durable one, so neither has to be maintained by hand
  - the cost, recorded because it is real: a local agent-file no longer reads as family-agreed
    the way the payload does. We think an acquaint-time diff is the fix, and it is unbuilt
  - rationale in [chores-06](notes/chores/chores-06.md#docs-experiment-in-the-local-agent-files)
  - same session's family-convergence findings, kept in chores because the mailbox protocol is
    handle-then-delete, so a message can never be a record:
    - [jj revset primer audit](notes/chores/chores-06.md#jj-revset-primer-audit-2026-08-05): the
      payload's range bullets are wrong, and the error is the framing, not the gloss
    - [convergence measurements and positions](notes/chores/chores-06.md#convergence-measurements-and-positions-2026-08-05)

- 2026-08-02 (`proposed`): prose.md's <=100 wrap got misapplied as ~64 to match older files' look
  - the rule needed no change, the application did: surrounding narrow wrap is not a reason to
    wrap narrow, and one fact per sub-bullet beats a paragraph packing several (wink, reviewing
    TODO/chores additions)
  - extended to code comments at wink's direction
  - the application now lives in [prose.md](agent-data/prose.md#prose-form) itself, moved there
    2026-08-07, and this entry stays until the family takes it

- 2026-08-01: published history renumbered under the new scope-based advancement rule
  - mapping: 0.24.0 -> 0.23.2 (grade-block compaction: presentation within the existing
    shape), 0.24.1 -> 0.23.3 (report docs), and the punctuation sweep lands as 0.23.4, while
    0.24.0 stays reserved for the next architectural change
  - executed as a jj history rewrite of the two published commits (version-of-record and
    description, with ochid trailers hand-copied per the re-describe rule) plus a force-push,
    safe because this repo has a single user and no external clones
  - permanent residue: bot-repo session transcripts and reports pasted in conversation carry
    the old `iiac-perf 0.24.0` / `0.24.1` banners, and this mapping is the decoder
  - process finding: the rewrite would have been free had the cycles run on a topic bookmark
    landed onto `main` after review. "always work on a branch" added to TODO.md
- 2026-08-01: `vc-x1 push --body` rejects a body whose first character is `-`
  - a file-by-file body opening with its first bullet failed twice: once at vc-x1's own clap
    (worked around with `--body=`), then again inside push's `jj commit -m <body>` (same clap
    leading-hyphen rejection), which rolled both repos back cleanly
  - workaround that also satisfies prose form: open the body with its intro line, never a
    bare bullet
  - template finding: vc-x1 should pass bodies to jj as `-m=<body>` or via stdin/file

- 2026-07-31: adopted mid-dogfood-window
  - adoption base: the template's `AGENTS-vc-x1-f5-20260730-snapshot/` (AGENTS.md, CLAUDE.md,
    agent-data/), created the same day from vc-x1's live copy, the window's authority
  - local copies verified byte-identical to the snapshot at adoption (`diff -r`)
- 2026-07-31: adoption base amended before first commit
  - the snapshot's AGENTS.md gained rule 0 (read custom.md before acting) and hard-rules-first
    ordering, authored snapshot-side while vc-x1's session was live, with the pending sync to
    vc-x1 tracked in the template's snapshots.md
  - local AGENTS.md re-copied from the snapshot, and the pin set re-verified byte-identical
- 2026-07-31: template restructured into vc-x1-template, and pin lines made generic
  - the template + coordination point is now the vc-x1-template repo: init payload in `work/`
    and `work.claude/`, discussion artifacts in `agents-protocol/`, mailboxes in `messages/`.
    The old vc-x1-work-repo-template and vc-x1-bot-repo-template are untouched pending wink's
    discussion with vc-x1
  - every "pinned to vc-x1-work-repo-template" line in the pin set became "the template
    repository" (snapshot-side amendment, same pending-sync flow as rule 0)
  - local pin set re-copied. Snapshot, `work/`, and this repo verified three-way byte-identical
- 2026-07-31: first push under the new rules, and two checklist gaps found (the 0.23.1 push)
  - given "desc and push", the agent jumped straight to the description: cycle.md's per-commit
    checklist has no step for backfilling the previous push's chores refs, bumping the
    version-of-record, or appending the chores record, so following it verbatim skips all
    three. Proposed template fix: add them as explicit steps before "write the description"
  - rule adopted (wink): **every commit belongs to a cycle, single- or multi-commit, and there
    is no out-of-cycle push.** Mid-ladder the cycle is implied. Otherwise ask "single- or
    multi-step cycle?" before preparing the commit. A single-commit cycle is a bare `X.Y.Z`
    close-out, so its validation is mandatory, and with no planning phase it skips
    `## In Progress` and goes straight to chores + Done
  - convention adopted (wink): chores commit references use the **as-built ladder form** for
    every cycle (rung `- [[N]] X.Y.Z[-n] <title>`), replacing the `Commits:` line. Codified in
    agent-data/notes.md + cycle.md (snapshot-side, pending vc-x1 sync) and both
    cycle-protocol.md copies (this repo's and the template payload's), with pre-existing
    `Commits:` lines grandfathered
  - convention adopted (wink): chores files carry a **title-only `## Table of Contents`**, one
    `- [<title>](#<anchor>)` entry per commit-recording section, with no versions or refs, so
    it never needs backfill (the TOC navigates, the ladder records). Codified beside the ladder
    form, first instance in chores-05.md
- 2026-07-31: Todo #1 run as a single-commit cycle after a four-rung ladder was rejected
  - the proposed rungs split along implementation lines (print+parser / precision / README)
    and failed the test that makes a ladder worth having: no rung was independently valuable
    or revertable, and every intermediate state was a half-reshaped published report
  - refines the single-vs-multi question: multi-commit wants rungs that each stand alone, while
    "one deliverable, several files" is single-commit shaped no matter how many edits it takes
- 2026-07-31: post-facto trapezoid experiment (0.23.0)
  - a published linear cycle was reshaped in place into the merge non-ff form with one
    `jj rebase -s` (close-out becomes the merge, rungs become the side leg, descendants
    follow). Chids and ochid trailers survived, the bot repo needed nothing, and the one
    casualty was the close-out's recorded SHA, re-recorded per the backfill timing rule
  - details in chores-05.md "Post-facto trapezoid rewrite (2026-07-31)", family-relevant if
    other repos want to adopt the trapezoid shape retroactively
  - the pre-restructure AGENTS.md is replaced (preserved in jj history, and verbatim as the
    template's AGENTS-iiac-perf.md)
  - the parked `punctuation-sweep` branch (TODO.md Todo #2) edited the old AGENTS.md. Its
    AGENTS.md hunks are obsolete now that the typeable-punctuation rule ships as hard rule 8
    and prose.md, so the branch needs re-scoping to its README / TODO.md /
    notes/cycle-protocol.md conversions before landing
  - remaining pre-restructure local deltas (e.g. clap `verbatim_doc_comment` guidance) still
    need distilling into this file or proposing into the template, with review findings recorded
    in the template's AGENTS-vc-x1-f5-20260730-review-iiac-perf-f5-20260731.md
