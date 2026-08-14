# Chores 07

Continuation of [chores-06](chores-06.md). Records landed work; conventions in
[agent-data/notes.md](../../agent-data/notes.md#chores-conventions) and
[cycle-protocol.md](../../agent-data/cycle-protocol.md#chores-sections).

Rolled over from chores-06 at 1255 lines, on wink's call. What triggers a rollover is written down
nowhere. Practice across chores-01 through 06 is 552, 1264, 1088, 1342, 1178 and 1255 lines, so
the real trigger is roughly 1100 to 1300, known to us and to nobody else.

## Table of Contents

- [docs: design the vc-x1-messages repo](#docs-design-the-vc-x1-messages-repo)
- [docs: semicolons leave the agent-files](#docs-semicolons-leave-the-agent-files)
- [docs: always link the closing rung](#docs-always-link-the-closing-rung)

## docs: design the vc-x1-messages repo

- [[1]] 0.24.10 docs: design the vc-x1-messages repo

A shared repo for family correspondence, built in one sitting and unreviewed by anyone else. The
protocol lives in its own `README.md` at format 0.1.0, and this project's inbox is `iiac-perf.md`
beside it. What is recorded here is what was decided and why, since the README states the rules
without the alternatives they beat.

The repo exists because the transport was the defect rather than the messages riding it. That
diagnosis came out of a day spent finding that everything owed to another member was sitting in a
gitignored scratch file, with no durable place to put it.

- **Why a repo at all.** Mailboxes live in the template repository, whose `main` is a single
  `Initial commit` with everything else uncommitted, so a handled message deleted there is
  unrecoverable. `custom-family.md`'s copy-into-chores-before-deleting rule exists only to
  compensate for that.
- **Plain, with no agent, and the reason is structural rather than taste.** A managed repo
  inherits "a repo with a live session is written only by its own agent", which is the rule that
  created mailboxes in the first place, so a manager would make the one repo everyone must write
  to writable by one member. What replaces the manager is a rule: whoever writes a record commits
  it, in the same act.
- **Bodies stay in the sender's repo and only pointers live in the shared one.** That is what
  makes a notification record losable without loss, which in turn is what lets each file's owner
  choose their own persistence policy without endangering anything.
- **Records, not a positional line.** The line format was revised four times in about an hour,
  which is a format asking to become named fields. A `##` heading gives each record an anchor for
  free, so a reply can cite an exact entry with no invented id.
- **The remote reference is a commit permalink**, since a branch name rots exactly when the
  message becomes worth reading: a topic bookmark is deleted at landing and the permanent branch
  does not carry the file until then. That creates an ordering constraint the README now states,
  because a permalink cannot be written before the commit it names is pushed.
- **Strict for writers, tolerant for readers.** A malformed record is not an error, with one
  exception: the `##` heading must exist, because orphaned field lines join the record above, so
  an interrupted write damages its neighbour rather than itself.
- **Deliberately unsolved: notification.** No file in any repo can reach someone who is not
  looking. GitHub issues would, at the cost of moving correspondence outside the clone, the diff
  and jj history, so they are held in reserve for that one job.
- **Open for vc-x1**: whether a member's file may be created by whoever writes first, which is
  what a first message to a new member requires and which the README carries as a proposal rather
  than a rule.

### The specimen is the point of `messages/test-msg.md`

The one file this cycle adds to this repo is a two-line message. It exists so the README's
examples reference something real: a `local` path that resolves in a sibling clone and a `remote`
permalink that resolves for a reader with none.

**It also proves the ordering rule by needing it.** The README's permalinks pointed at a commit
that did not contain this file, so they answered 404 until it was committed and pushed. The rule
was written from that failure rather than in anticipation of it.

### Where this leaves the messages repo

`vc-x1-messages` is committed separately and is not part of this repo's history. This section is
the durable record of the reasoning, reachable from this commit's `ochid:` trailer, because a
plain repo has no agent repo of its own and its commits carry no trailer.

Still owed and deliberately not in this cycle: telling vc-x1 the repo exists, their review of it,
and whatever the review changes.

## docs: semicolons leave the agent-files

- [[6]] 0.25.0-0 [docs: semicolons leave the agent-files opening][2]
- [[7]] 0.25.0-1 [docs: tighten the semicolon rule][3]
- [[8]] 0.25.0-2 [docs: sweep semicolons from the agent-files][4]
- [[9]] 0.25.0 [docs: semicolons leave the agent-files closing][5]

The 0.25.0 cycle: the pinned semicolon rule goes flat, and the agent-files sweep to zero.

### Problem

The pinned semicolon rule licenses a judgment at every site ("between equals"), and agents take
advantage of judgment exceptions, so the allowance gets claimed wherever a semicolon is wanted.
The agent-files carry roughly 100 semicolon joins under that license, and the rule says nothing
about historical text, so a sweep has no stated boundary.

### Solution

prose.md's `Semicolons` rule now reads flat: prose carries no semicolons, a semicolon appears
only in code, where it is syntax, and every prose site converts to a period, a comma with a
conjunction, or sub-bullets. The agent-files (custom* included) carry no historical exemption
and are swept to zero, ninety sites across eight files. Any other historical file keeps its
semicolons until altered, at which moment the user is asked whether they should go.

### Acceptance check

Three measures:

- With fenced code blocks and backtick spans blanked, `grep ';'` over `AGENTS.md`, `custom.md`,
  `custom-family.md`, and `agent-data/*.md` returns zero hits.
- Full validation passes at every commit.
- `src/` is untouched by this cycle, its comment-line semicolons deliberately excluded.

**Result: passed**, 2026-08-14. The blanked grep returns zero across the set, with seven raw
semicolons surviving inside code spans and fences. Full validation ran and passed at every rung
and at the close-out. The cycle's diff against `main` names no `src/` file.

### Deliberation

**Why the cycle exists with no `## Todo` entry moving.** Convention work runs as its own cycle,
and no ranked entry names it. The itch came out of preparing the formal review owed vc-x1, when
wink asked for the semicolons to go.

**The rule tightened is one vc-x1 wrote.** Their `Semicolons` pin blesses the between-equals
join, and their ~140-join sweep converted everything else. Removing the allowance is offered to
the family the usual way: edited into our local pinned copy, proposed to vc-x1 by message now
that the cycle lands, with their own precedent ("review the rule, not each instance") as the
reading instruction.

**Why absolute rather than judged** (wink): agents take advantage of the exceptions. The
typeable-punctuation section next door already states the general mechanism, that a soft rule
accumulates violations. An absolute rule needs no judgment and is nearly greppable, code spans
being the one exemption a checker must handle.

**`src/` excluded** (wink): the rule covers code comments going forward, but the ~125 existing
comment lines are a sweep twice this one's size, and this cycle stays focused on agent-file
convergence. The code sweep is a candidate follow-up, not scheduled.

**0.25.0 on wink's call.** A family-scoped rule change plus a full-set sweep reads as more than
a patch, and wink named the number.

**Ordering against the open convergence cycle** (wink): whichever lands second rebases. Both
cycles edit `TODO.md`, `notes/chores/chores-07.md`, and `Cargo.toml`, so the conflict is known
and accepted.

### Ladder details

#### docs: semicolons leave the agent-files opening

Create the bookmark, open this block, backfill the as-built rungs the 0.24.9 and 0.24.10
landings made due (chores-06's "chore: complete the landed records", chores-07's "docs: design
the vc-x1-messages repo"), retire the dynamic-warmup Done entry, and bump to 0.25.0-0.

#### docs: tighten the semicolon rule

The between-equals allowance is gone and the rule is flat: prose carries no semicolons, and a
semicolon appears only in code, where it is syntax. Each prose site converts to a period, a
comma with a conjunction, or sub-bullets, and the code allowance is why enforcement blanks code
before expecting zero. The agent-files carry no historical exemption and sweep to zero. Any
other historical file keeps its semicolons only until altered, and altering one means asking
the user whether they should go, a mid-rung tightening by wink from the draft's silent
convert-when-touched. The old rule's three-way structure survived as the conversion list, its
antithesis example now demonstrating the period split. The typeable-punctuation contrast
flipped from "unlike" to "like", stricter only with history. The dogfood entry records the
proposal for vc-x1, whose rule this tightens.

#### docs: sweep semicolons from the agent-files

Ninety prose sites converted across eight files, custom-family.md the heaviest at 35, and the
blanked grep now returns zero across the set. Most sites took the period or the comma with a
conjunction, and the genuine lists-in-prose (the draft-rewrite exceptions in checklist and
protocol, notes.md's record-ownership division) became sub-bullets. Seven raw semicolons
survive, all inside code spans or fences: AGENTS.md's shell examples, code.md's fenced Rust,
and the rule's own specimen. One heading carried a semicolon, notes.md's "no edit list"
section, and the comma replacement leaves its anchor unchanged, so both inbound links hold. A
comma splice introduced mid-sweep was caught and fixed with a conjunction, evidence the
conversions want judgment rather than sed.

#### docs: semicolons leave the agent-files closing

One gotcha, from a rung push rather than the closing itself.

**Problem**: the second rung's `vc-x1-dev push` (0.78.8-8) exited with `error: Concurrent
checkout` at the push-work stage, after both repos had committed and the bookmark was set.
**Solution**: measured rather than assumed, the work-side push had in fact succeeded (the
bookmark's remote ref carried the rung), and only the bot repo's squash-push remained, which
wink completed by hand. We think the bot repo's continuously growing session file raced jj's
working-copy snapshot. wink plans a delay between operations in vc-x1-dev, and the finding
rides to vc-x1 with the convergence message.

## docs: always link the closing rung

- [[N]] docs: always link the closing rung

### Problem

The closing rung was the ladder's one exception: unlinked until close-out gotchas gave it a
subsection, so a link's existence was unpredictable and the rung had no anchor while the cycle
ran. wink's template edit made the semicolon cycle practice the always-linked form, which left
TODO.md's shape template and three pinned statements contradicting each other.

### Solution

Every rung links 1:1 with a `Ladder details` subsection, closing included: the closing rung's
opens at laddering with a one-line stub and completes at close-out with gotchas in
problem/solution form, or `_None._`. Edited: the opening and close-out checklists, the
protocol's closing-rung paragraph, notes.md's slot note, prose.md's ladder-step surface, and
TODO.md's template description.

### Acceptance check

No agent-file states the unlinked-closing exception any more, and TODO.md's shape template
agrees with the checklists and the protocol.

**Result: passed**, 2026-08-14, by grep for the old exception's wording and by reading the
edited statements side by side. The grep earned its keep: it caught a fifth statement, in
prose.md's ladder-step surface, that the edit list had missed.

### Ladder

- [[N]] docs: always link the closing rung

### Deliberation

**Run as a single-commit cycle**, no entry moving to `## In Progress`, per the single-commit
form. The Todo "Change TODO rules so ladders 1:1 with detail" retires with it.

**The rule follows practice.** The semicolon cycle ran always-linked on wink's template edit,
and its closing subsection immediately earned its keep by holding the Concurrent-checkout
gotcha. The cost is a one-line stub per cycle.

**The protocol paragraph's tail was trimmed at review** (wink): the closing-rung paragraph had
accumulated a restatement of the chores move, a program-depth note, and a "still" comparing
against the old rule. The move and the depth shift are `Chores sections`' to state, and the
comparison is history, which this section now holds: the old rule created the closing
subsection only when gotchas occurred and left the rung unlinked until then. One detail
retires here with it, that under a program heading the subsections sit at markdown's heading
floor while the block is in `TODO.md`, which the shallower chores copy relieves.

**Recorded here for adjacency**: before this cycle's backfill, the landed 0.25.0 close-out was
re-described from the bare cycle title to the bookend form with " closing" (wink caught the
dropped suffix). A coordinated rewrite per the re-describe rule: the ochid trailer hand-copied,
both parents preserved, `main` force-pushed, and the backfill then recorded the rewritten SHA.
Residue: the pre-rewrite SHA 9f361686034e appears in this session's transcript, and this
sentence is its decoder.

# References

[1]: https://github.com/winksaville/iiac-perf/commit/55554b452957 "55554b452957ab672bfa3caa84ece5ba778cca64"
[2]: #docs-semicolons-leave-the-agent-files-opening
[3]: #docs-tighten-the-semicolon-rule
[4]: #docs-sweep-semicolons-from-the-agent-files
[5]: #docs-semicolons-leave-the-agent-files-closing
[6]: https://github.com/winksaville/iiac-perf/commit/6e97f2e6103f "6e97f2e6103fa0cc3c9279706a88eaaff3f45042"
[7]: https://github.com/winksaville/iiac-perf/commit/8cab1a0614c3 "8cab1a0614c3efd7a4ddddc05fc402c3b83f13a7"
[8]: https://github.com/winksaville/iiac-perf/commit/800ede0649d2 "800ede0649d244055008fff60c5ade7ce1e1a5c6"
[9]: https://github.com/winksaville/iiac-perf/commit/608ede051940 "608ede051940b4d481f9b3c1e9360e92c5c7ffe9"
