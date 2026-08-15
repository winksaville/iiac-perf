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
- [Message to vc-x1: duplicated cycle rules, and landing](#message-to-vc-x1-duplicated-cycle-rules-and-landing)
- [docs: converge the agent-files with vc-x1](#docs-converge-the-agent-files-with-vc-x1)
- [docs: point messaging at the vc-x1-messages repo](#docs-point-messaging-at-the-vc-x1-messages-repo)

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

- [[10]] 0.25.1 docs: always link the closing rung

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

## Message to vc-x1: duplicated cycle rules, and landing

Findings from 2026-08-12 and 2026-08-13, drafted for vc-x1's mailbox and recorded here before
delivery. Two items, sent ahead of the formal reply: the duplication compounds with every commit
either repo writes, and the landing gap is a hole this repo fell into on the 12th.

This section records a **message** rather than a commit, so it carries no as-built ladder. A
message can never be a record ([custom-family.md](../../custom-family.md#messaging)), and these
findings had no home outside a gitignored `tmp/` file. It keeps the decisions and the
measurements, and the route to them is in the session the opening commit's `ochid:` trailer
names.

### Nine steps in two files

wink's line, verbatim:

> Wink thinks the duplication of the 9 rules is guaranteeing drift in the future and we should
> contemplate eliminating the duplication

- **The overlap is nearly total.** `cycle-checklists.md` has 11 sections and almost every one
  restates a `cycle-protocol.md` section. The per-commit lists are the same nine steps in the same
  order, and `Topic bookmarks are drafts` is a section title in both files.

  | cycle-checklists.md | cycle-protocol.md |
  | --- | --- |
  | The cycle at a glance | Cycles |
  | Cycles run on a bookmark | Topic bookmarks are drafts |
  | Opening checklist | Preparation |
  | Per-commit checklist, nine steps | Per-commit flow, the same nine steps |
  | Ladder (sub-cycle) checklist | Per-Work-commit contract within a ladder |
  | Before any push | Pushing / Policy |
  | After the final push | After push or squash-push: stop and wait |
  | Close-out checklist | Close-out |

- **The reader most exposed is the one following the rules.** Hard rule 7 says read the checklist
  before acting; the checklist's own header says the protocol wins on any disagreement. So the file
  an agent is told to read at the moment of action is, by construction, the copy that loses.
- **Three instances the same day**: `custom-family.md` pointing at "step 4" after the morning's
  sync moved validation to step 5; one new rule written into both files, corrected by wink, then
  over-corrected; and a single-step cycle's chores as-built ladder and its six-item `### Ladder`
  being the same rung written twice.
- **Proposed direction**: the checklist keeps the numbered imperatives and owns them; the protocol
  drops the restatement and holds the reasoning, the recovery procedures, the close-out shapes and
  the trapezoid recipe, referring to steps by name so renumbering cannot desynchronize them.
- **The range**: the minimum is that no rule is stated twice, and wink's inclination is one file
  rather than two. We could find no written case for keeping them separate, which is what the
  message asks vc-x1 for.

### Landing, measured

Making one or more already-pushed commits permanent, with `<branch>` the bookmark that advances
and `<cycle>` the topic bookmark it advances to. Measured 2026-08-13 in a throwaway repo pair built
to that premise.

```
jj log -r '<branch>::<cycle>' -R .        # precondition; empty output means <branch> moved, stop
jj bookmark set <branch> -r <cycle> -R .  # local
jj bookmark delete <cycle> -R .           # local
jj git push -b <branch> -b <cycle> -R .   # one push, both halves
```

- **Three commands, not four.** A deletion needs no push of its own: `-b` repeats, and naming a
  locally deleted bookmark publishes its deletion. Bare `jj git push` is not a substitute, since it
  declines to publish deletions without `--deleted` or an explicit `-b`.
- **Both mutating steps are local**, so the sequence has exactly one irreversible moment.
- **Rerunning is safe, for a reason that is a trap.** A second identical run printed
  `Warning: No matching bookmarks for names: mycycle` and `Nothing changed`, and **exited 0**,
  because `-b` matches patterns rather than literal names. So a mistyped bookmark succeeds
  silently, and nothing built on this can use exit status to know the deletion happened.
- **The push's own output is the per-step report** wink wanted a command to produce: one line per
  bookmark, `move forward from ... to ...` and `delete from ...`.
- **Nothing here commits the agent repo**, which is what cost us a record on 2026-08-12. The act
  creates no work-repo commit, so nothing carried an `ochid:` and the session that decided it was
  left dangling.

### The proposal: two flags on `push`

`vc-x1 push --from <cycle> --to <branch> --delete-from` (wink, 2026-08-13), superseding a
`vc-x1 land` subcommand drafted first. The margin is too thin for a new verb: `push` already
commits both repos, advances the destination bookmark and publishes it, measured by
`vc-x1 push main --dry-run` on an empty work `@` running four of its five stages.

Three flags, one of them a rename:

- **`--to <branch>`**, the bookmark that advances, renaming `push`'s existing `[BOOKMARK]`
  positional / `--bookmark` rather than adding an argument. The positional stays as shorthand.
- **`--from <cycle>`**, the bookmark it advances to. Safety and cleanup only, never content
  selection: `bookmark-set` hardcodes `-r @-`, so what publishes is always where you stand.
  Require `<from>` to be at `@-` and refuse otherwise, and a wrong `--from` cannot publish the
  wrong thing.
- **`--delete-from`** (wink), `jj bookmark delete` being the verb a user already has, and the flag
  pairing with `--from` so the bookmark is named once. `--clean` was rejected for naming the
  working copy's state in the same breath, and `--retire` / `--retire-cycle` for inventing a
  synonym and, in the second case, hardcoding a role the other flags keep neutral.
  - Deletion stays opt-in: a default would bake our hard rule 13 into tooling documented as
    assuming nothing about a repo beyond `.jj` and `.vc-config.toml`.

Three behaviors that come with them:

- **Refuse a non-fast-forward outright**, no override flag, which also makes a swapped pair safe.
- **Derive the agent-repo commit's message from `@-`** when work `@` is empty, rather than
  demanding a title and body for a work commit that is never created.
- **Report rather than act**: the as-built backfill is due (permanence is what makes those SHAs
  stable, and the trigger has no enforcement anywhere today), and other bookmark deletions are
  pending. It must not inspect the project's records, `push` assuming nothing about a repo beyond
  `.jj` and `.vc-config.toml`.

**Nothing else changes**: the clean/dirty polymorphism already works, `commit-work` skipping when
`@` is empty and the message stage when neither repo has pending changes, so a dirty `@` needs
`--title` / `--body` and a clean one does not.

### Not building yet: an inferred `--from`

`--from` could default to the current branch, which is what it would be most of the time. It should
not, yet.

- **There is no current bookmark to infer, and it fails in both directions** (wink). At `@-` there
  may be **none**, the normal state mid-ladder where rungs are `jj new` plus `jj describe` and
  nothing is pushed; or **several**, the case below.
- **Several is guaranteed at every cycle opening rather than exotic.** The bookmark is created
  at the opening and nothing has committed on it, so it and `<branch>` sit on the same commit.
  Measured at this cycle's own opening: `docs-converge-the-agent-files-with-vc-x1` and `main` both
  on `28bd6daa`. Paired with `--delete-from`, a wrong guess deletes `main`.
- **jj declines to pick, and so should the command.** wink's shell prompt renders the set,
  `(docs-converge-the-agent-files-with-vc-x1 main+1)`, rather than naming one as current.
- So `--from` stays explicit until a written rule names what inference picks and prints what it
  inferred before acting.

### Naming

- **`land` is retired as the command's name.** It is singular in ordinary use, so a compound
  operation borrowing it promises less than it does (wink). `push-onto` was rejected too, `jj
  rebase` already using `--onto` for its destination.
- **The state sense keeps a word of its own, and the pinned text already has it**: *permanent*.
  `cycle-protocol.md` says "lands on a **permanent branch**", so "backfill once the commit is
  permanent" needs no metaphor and no glossary entry, and command and state stop borrowing from
  each other.
- The agent-files should spell the operation out rather than lean on a term: hard rule 13, the
  close-out step, and `jj.md`'s section title all use "land" as though it were defined.

### Findings carried alongside

- **Beats with no work-repo commit have no `ochid:` anchor** (wink). Landing is one; writing a
  mailbox message is worse, producing no work commit at all, and the template repo has no commits
  to archive it either. This section is that artifact, produced by hand.
- **The chores rollover trigger is undocumented.** `notes.md` gives a new file a `[1]` start and
  says nothing else. Practice across chores-01 through 06 is 552, 1264, 1088, 1342, 1178 and 1255
  lines, so the real trigger is roughly 1100 to 1300, known to us and to nobody else.
- **`prose.md#cycle-bookend-titles` is a dangling anchor in both repos.** `notes.md:190` and
  `cycle-protocol.md:196` link it, while `prose.md:234` carries the text as bold inside a paragraph
  rather than as a heading. Byte-identical on both sides, so it is the family's, and a correction
  rather than a proposal.
- **Nothing says which part of the deliberation goes where.** The pinned files send it to "chores,
  todo, and the session" without dividing them, which is how this section's first draft came out a
  transcript with headings. A line saying chores keeps decisions and measurements while the session
  keeps the route would settle it, and it is the same disease as the nine steps: content with no
  single owner.

## docs: converge the agent-files with vc-x1

- [[11]] 0.25.2 docs: converge the agent-files with vc-x1

The 0.25.2 cycle, run single-step after its three-rung ladder collapsed: the formal review of
vc-x1's set, the answers their messages waited on, and the records that carry the convergence
proposal.

### Problem

Our pinned agent-files and vc-x1's differ, and everything that would reconcile them is owed by
us: the formal review of their set, owed since their 2026-08-08 message; two answers their
2026-08-12 message asks for; and an announcement of the rule 0.24.8 wrote into two pinned files,
which is an open proposal sitting in our diff that they have no way to see. Meanwhile the
2026-08-12 findings, the nine-step duplication and the undefined "land", exist only in a
gitignored `tmp/` file, so the repo's durable record does not hold them at all.

### Solution

The 2026-08-12 findings moved from `tmp/` into this file's message section, and the early entry
was delivered to vc-x1's template mailbox. The formal review walked every hunk of the eight-file
diff and gave each a verdict, all of it our three proposals. Their notes-entry question is
answered, the template mailbox is swept, and the review invitation goes via `../vc-x1-messages`
now that the cycle lands.

### Acceptance check

Three measures, none of which waits on vc-x1's answer:

- `diff -rq agent-data ../vc-x1/agent-data` (plus `AGENTS.md` / `custom.md`) names only files
  whose every difference this cycle's record accounts for with a verdict (ours to keep, theirs
  to take, or open with vc-x1). At the opening it was two files and three hunks, all 0.24.8's
  validation rule. After 0.25.0 and 0.25.1 the diff also carries the flat semicolon rule, its
  sweep, and the always-linked closing rung, all ours to offer.
- Nothing this cycle carries survives only in `tmp/`: every finding is in a committed file.
- The early entry sits in `../vc-x1-template/messages/vc-x1.md` (delivered at the opening), the
  full reply and review invitation are recorded via `../vc-x1-messages` per its protocol, and
  each `Done when` item of their 2026-08-12 message is either answered or named as deliberately
  not answered.

**Result: two measures pass in full, the third half-passes by construction**, 2026-08-14.
Measure 1: all eight differing files walked hunk by hunk, every difference one of the three
proposals, verdicts recorded in the review subsection below. Measure 2: the findings have lived
in this file's message section since the opening commit. Measure 3: the early entry is
delivered, while the reply's own delivery follows landing by construction, the permalink
ordering rule forbidding a pointer to an unlanded target. The abandoned 2026-08-13 close-out
draft failed this same measure outright, and the difference is that delivery is now the next
act rather than unscheduled.

### Ladder

- [[N]] docs: converge the agent-files with vc-x1

### Deliberation

**Why the cycle exists at all, since no `## Todo` entry moved into it at the opening.** The work
is correspondence and convergence, which no ranked entry named. Todo "Sync the 20260803
agent-files baseline" is adjacent and may be wholly overtaken by 0.24.5's sync; deciding that is
the convergence exchange's to do rather than this cycle's to assume. Later, `main` gained the
entry "Converge agent-files with vc-x1" while this cycle was open, and at the rebase it moved in
here.

**Collapsed to single-step at close-out** (wink): the review rung's diff was records only, the
agent-file substance having landed in 0.25.0 and 0.25.1, so the three-rung ladder became one
commit. Squashing a draft on its bookmark is what topic-bookmarks-are-drafts licenses. The
pushed opening's bot twin carried the opening title and is published, so the collapse cost one
coordinated re-describe on the bot side, accepted knowingly.

**The chores file is opened at Preparation, which the protocol forbids** ("Nothing is opened in
the chores file here"). Taken as an explicit exception on wink's instruction, and recorded per
the hard rules' preamble. The justification is that the two rules point in opposite directions
for this one beat: `custom-family.md`'s messaging rule sends a message's durable content to
`notes/chores/chores-NN.md`, while the cycle protocol reserves that file for close-out. The
message section is not this cycle's record; it is the record of a message, whose beat produces
no commit of its own. We think that collision is a real gap in the pinned set rather than a
local awkwardness, and it goes to vc-x1 with the reply.

**Nothing retired from `## Done` at the opening sweep.** Running it returned nothing: all seven
entries were agent-file cycles from 0.24.x, precisely the nearby context a convergence cycle
reads. A sweep that returns nothing is a run sweep, not a skipped one.

**Patch scope, and after two re-stamps the close-out is 0.25.2** (0.24.10-0 at the opening,
0.25.2-0 at the rebase). Records, correspondence, and at most small edits to pinned prose: work
within the existing shape rather than a reshaping of it.

**Rebased onto 0.25.1** (2026-08-14, wink's whichever-lands-second-rebases call): the semicolon
and always-link cycles landed first, and the predicted conflicts (TODO.md, chores-07,
Cargo.toml, Cargo.lock) resolved by keeping main's records and re-applying this cycle's
additions. The block came up to the new rules on the way. The same day's paired-history repairs
(two bot-side re-describes, a landing commit folded into its twin) are recorded in the review
subsection's findings and ride to vc-x1 with the message.

**The bookmark was created unpublished** (`jj bookmark set`, not `jj git push --named`), since
nothing was being pushed yet and a line nobody has approved does not need to be visible.

**Out of scope, deliberately**: folding vc-x1's answers back into our pinned copies. That waits
on their reply, so binding this cycle's close-out to it would make the close-out hostage to
another repo's session.

### The opening's record

Create the bookmark, open the block, and give the 2026-08-12 findings a durable home in the
message section above: the nine-step duplication between `cycle-checklists.md` and
`cycle-protocol.md`, the undefined "land" with the sequence we actually ran, the `vc-x1 land`
design, and the four findings carried alongside. Then deliver the drafted entry to vc-x1's
mailbox.

**The design the intent named no longer exists.** `vc-x1 land` was drafted, then superseded
within the beat by `vc-x1 push --from <cycle> --to <branch> --delete-from` (wink): `push`
already commits both repos, advances the destination and publishes it, so a new verb buys only
a precondition and a cleanup.

**Four measurements replaced four assumptions**, taken in throwaway repo pairs rather than
reasoned from the docs: the sequence is three commands and not four, since `-b` repeats and a
locally deleted bookmark named in a push publishes its deletion; both mutating steps are local,
so there is one irreversible moment; a rerun warns and **exits 0**, because `-b` matches
patterns, so a typo succeeds silently and exit status cannot be trusted; and
`vc-x1 push main --dry-run` on an empty `@` already runs four of its five stages. The first of
those corrected a recipe this beat had already written down.

**The record came out a transcript and was cut by half** (290 lines to 158) after wink called
it, which produced the beat's most general finding: nothing in the agent-files says which part
of the deliberation goes to chores, which to todo, and which to the session. They name all three
and divide none.

**Two stale bookmarks swept on the way past**, `punctuation-sweep` and `fix-calibration`, both
measured fully merged. The three that are not, one being `web-claude-tweaks` with an unlanded
commit, became the backlog's first entry rather than a claim in a record.

### The review of vc-x1's agent-file set

**The review's result: their set is our base, and the whole diff is ours.** Every hunk across
the eight differing files was walked against their working copy (mtimes 2026-08-14, so read
rather than assumed), and each belongs to one of our three proposals: validate every commit
(0.24.8), the flat semicolon rule and its sweep (0.25.0), and the always-linked closing rung
(0.25.1). Nothing of theirs is missing from our set and nothing of theirs arrived that we have
not taken, so the verdict list is: ours to offer, three proposals. Theirs to take, none. Open,
none. `custom.md` is byte-identical, as the stub design promises.

**The notes-entry answer** (their 2026-08-12 question): an entry stays a numbered list item,
cited by its bold title with the number as a hint only, which is their own proposal and we
support it as hard rule 9's extension to the notes files. Headings would buy stable anchors at
the price of hiding the strict rank that is the list's point, and the surfaces needing real
anchors already have them, chores sections and message records both being headings. The tracker
is rejected on their own framing, independently confirmed by our messages-repo design: issues
and databases sit outside the clone, the diff, and jj history, so GitHub issues stay held in
reserve for the one job no file can do, notification.

**The mailbox swept**: their 2026-08-12 and 2026-08-08 entries handled and deleted, the file
with them. The commit-body form was adopted at 0.24.7, their two regression flags were confirmed
fixed by the 0.24.5 sync on their own measurement, and the remaining Done-whens are answered by
this cycle's records and the coming message. One operational warning is carried forward rather
than deleted with the entry: their `repos.agent` hard rename means a future binary will refuse
our `.vc-config.toml` with a fix-it message, a five-second edit at a moment we pick.

**Findings from the day's own mechanics**, for the message: a work-side re-describe desyncs its
bot twin's title unless both are rewritten (measured twice, repaired twice); a `vc-x1 push` with
an empty work `@` mints a bot-only commit whose title matches no work commit, the measured
instance of the derive-from-`@-` proposal; push has no verb that publishes an amended history
without committing something, so post-surgery publishing is bare jj; and a rebase permanently
skews work-list order against the chronological bot journal, so paired-history readers must
match by ochid, never by position.

## docs: point messaging at the vc-x1-messages repo

- [[N]] docs: point messaging at the vc-x1-messages repo

The 0.25.3 cycle, run single-step: `custom-family.md`'s Messaging section catches up with the
`vc-x1-messages` repo.

### Problem

The Messaging section still routed mail through the template repository's mailboxes, which the
converge cycle replaced with the `vc-x1-messages` repo. A reader following it checked
`../vc-x1-template/messages/iiac-perf.md`, a file that no longer exists, and read an absent file
as "no mail" while real records waited in `../vc-x1-messages/iiac-perf.md`. Worse, the section's
handle-then-delete rules would destroy exactly the records the new protocol preserves.

### Solution

The section now names `../vc-x1-messages/iiac-perf.md` as our file and that repo's `README.md`
as the governing protocol, with the behavioral bullets rewritten to the record model:

- open traffic is read off the records themselves: no `read` field means unread, no `outcome-*`
  field means still open
- mark, never delete: a `read:` timestamp on reading, `outcome-local:` / `outcome-remote:` on
  handling, which is what closes a record and tells the sender it arrived
- the copy-into-chores-before-delete step (learned 2026-08-05) retires explicitly, bodies being
  committed files in the sender's repo now
- durable mail is push-then-record, a `remote:` permalink naming a commit that must exist first

### Acceptance check

With the section rewritten, a grep for `mailbox` and `vc-x1-template/messages` over `AGENTS.md`,
`custom.md`, `custom-family.md`, and `agent-data/` hits no live rule: only historical records
(dogfood-log entries, chores narratives) and this cycle's own text.

**Result: passed**, 2026-08-15. The grep returns dogfood-log history in `custom-family.md` (the
2026-07-31, 2026-08-05, and 2026-08-07 entries), chores narratives, and this cycle's own text,
with `AGENTS.md`, `custom.md`, and `agent-data/` clean.

### Ladder

- [[N]] docs: point messaging at the vc-x1-messages repo

### Deliberation

**Run as a single-commit cycle**, no `## Todo` entry moving. The stale pointer surfaced at this
session's acquaint, when the mailbox check went to a repo that no longer receives mail, and wink
confirmed `vc-x1-messages` as the correct place.

**The rewrite stays in `custom-family.md`** rather than a pinned file: the protocol is
family-wide, but performing it needs a member name and a sibling path only the member layer has,
the same reasoning that moved the acquaint check out of `AGENTS.md` on 2026-08-07. The governing
text is deliberately the messages repo's own `README.md`, so this section holds only what
decides a session's behavior.

**Carved out of the in-flight port** (wink): the fix was written while the working copy carried
the uncommitted measure-reproducibility port, split into its own commit off `main` with
`jj split`, and the port commit rebased aside to land after it. wink named 0.25.3 for this cycle
and 0.25.4 for the port.

**Backfills ride along** per the landing's one-push-later timing: the 0.25.1 and 0.25.2 as-built
rungs take their SHAs and versions here.

**No topic bookmark** (wink): the single-step-still-gets-one rule is waived for this cycle, and
`vc-x1 push main` lands the one commit directly, the push being the landing.

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
[10]: https://github.com/winksaville/iiac-perf/commit/c38f8a6087e5 "c38f8a6087e5633e2d83493bf1ceb70ecf77c6b6"
[11]: https://github.com/winksaville/iiac-perf/commit/0520c17ca352 "0520c17ca352da9627ea9c551b79aee8a53de021"
