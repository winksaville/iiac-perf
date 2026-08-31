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
- [feat: measure reproducibility](#feat-measure-reproducibility)
- [fix: left-align the summary rows](#fix-left-align-the-summary-rows)
- [docs: a session's rules are its own agent-files](#docs-a-sessions-rules-are-its-own-agent-files)

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

## feat: measure reproducibility

- [[26]] 0.26.0-0 [feat: measure reproducibility opening][12]
- [[27]] 0.26.0-1 [fix: the settle cell reads the clock][13]
- [[28]] 0.26.0-2 [feat: write a per-run JSON record][14]
- [[29]] 0.26.0-3 [feat: adopt the markdown config carrier][15]
- [[30]] 0.26.0-4 [feat: read, pin, and restore the CPU frequency][16]
- [[31]] 0.26.0-5 [fix: the settle cell shows the clock's journey][17]
- [[32]] 0.26.0-6 [feat: block sleep and warmup become knobs][18]
- [[33]] 0.26.0-7 [fix: LSC gains a run-to-run component][19]
- [[34]] 0.26.0-8 [fix: the pin flag names CPUs][20]
- [[35]] 0.26.0-9 [feat: suggest-freq measures the pin frequency][21]
- [[36]] 0.26.0-10 [docs: split the README into a docs directory][22]
- [[37]] 0.26.0-11 [docs: the report reading guide][23]
- [[38]] 0.26.0 [feat: measure reproducibility closing][24]

The 0.26.0 cycle: a run becomes self-describing, steady, honest about its resolution,
and documented.

The evidence campaigns did not run inside the cycle: wink re-ordered the tail (docs, then
close-out, then campaign), so the three-box rerun and the zcr-mpsc sweep run as their own
follow-on cycle, with `--blocks` and a real `--block-sleep` so every record carries a
block-mean series, `--record` and `--tag` naming series, bench, layout, and condition, and
the records committed under `data/`. Everything they need landed here, records and pinning
both. A first taste ran live anyway: wink's 2026-08-19 3900X placement sweep, now the report
guide's worked example.

### Problem

Modern CPUs change their clock frequency continuously to save power. The OS's **power policy**
(the cpufreq governor plus, on AMD, the EPP hint) decides how eagerly the chip boosts under load
and how quickly it retreats, so the clock a run executes at depends on the box's recent history,
not its spec sheet. A latency number is cycles times cycle time, and the 2026-08-03 experiment
showed both failure shapes: under `powersave` the same code differed ~9% run to run as the clock
wandered, and a run parked on the un-boosted base clock graded A, flat being all the grade sees.
For A/B comparison the need is consistency, not speed: a stable clock at any rate beats an
unstable fast one. Today the harness cannot report the policy, cannot hold the clock still, and
no run's numbers survive the session that produced them, so a reader cannot tell a code delta
from a power-management delta. (Continues the cycle stalled since 2026-08-04.)

### Solution

The stalled branch's finished rungs (the policy and clock-quantum report rows, and the report
renderer extracted into `src/report.rs`) were ported onto `main`'s line as the opening, by file
copy rather than rebase. Eleven Work rungs then made a run self-describing, steady, durable,
and readable:

- the settle cell gained the clock gate the warmup exit already had, and now narrates the
  clock's whole journey (start, settled state, share, steadiness)
- a per-run NDJSON record (`--record`), self-documented by `describe-record` and enforced by a
  two-way dictionary test
- the config adopted vc-x1's markdown carrier, plain `.toml` still accepted (one type per
  directory)
- frequency control: `read-freq` / `pin-freq` / `restore-freq` command words, a declared
  `[freq]` steady state, `--pin-freq` runs restoring on every catchable exit path, and
  `suggest-freq` measuring the pin the box holds under the real workload, ending in a
  paste-ready config line
- block sleep and warmup became explicit knobs defaulting to zero, the replication rows gated
  on a real sleep
- the resolution claim became the batch-curve drift floor, printed on every run
- the pin flag adopted the kernel's CPU vocabulary: `--pin` became `--pin-cpus`
- the README split into a `docs/` directory and gained the report reading guide, taught from
  this cycle's own live output

The qualify-environment rung halted unbuilt (wink, 2026-08-19): the environment rating gets
its redesign first, and the rung's spec waits in `## Todo` under "Rethink environment rating,
then resume the qualify power-policy rung". The evidence campaigns (the three-box rerun and
the zcr-mpsc reproducibility sweep across pin-CPU layouts) run as their own follow-on cycle,
records committed under `data/`.

### Acceptance check

The measures:

- A run with `--record` leaves a record that can be re-analysed without its session: it carries
  `schema_version`, the fixed quantile ladder, the block-mean series, and the box's policy
  fields, and `describe-record` documents every field, enforced by a test that fails on any
  undocumented key.
- The settle cell agrees with the clock: a 3900x `powersave` run no longer reads `0.01s` beside
  an F, and the cell names the state it settled into.
- `--block-sleep` and `--block-warmup` default to zero, print in Setup whenever blocks run,
  ride the record, and the block replication rows (CI95 / LSC) print only when the sleep is
  nonzero.
- The report's resolution claim is the variance-curve drift floor, not the within-run LSC that
  read 7x optimistic against measured run-to-run scatter, and it prints on every run,
  `--blocks` or not.
- `pin-freq` then `restore-freq` leaves the box in the declared steady state (governor, EPP,
  min/max, boost), a pinned run restores on normal exit, panic, and SIGINT, and `restore-freq`
  converges from any starting point, an unclean death included.
- README is the front door (a few hundred lines) with the depth in `docs/`, and
  `docs/report-guide.md` walks every report surface (Setup, header bracket, band table,
  summary rows, resolution, block rows, grade block, settle cell) against real output.
- `suggest-freq <bench>` reports each candidate frequency, how long it held, under what load
  and schedule, and ends with a paste-ready `pin_mhz = ...` line.
- `--pin-cpus` is the pin flag's name everywhere a reader meets it (help, README, code names),
  the record field `pin_cpus` unchanged, and README states the CPU / core / SMT terminology.
- The port preserved behavior: full validation green on `main`'s line at every rung.

Run at close-out (2026-08-19):

- record: a `--blocks 4 --block-sleep 1-10ms --record` run wrote `schema_version` 3 with the
  13-entry quantile ladder, the block-mean series, the knob fields, and the policy fields,
  and `describe-record`'s two-way dictionary test is green (141 tests)
- settle cell: verified live on the 3900X at 0.26.0-9: pinned runs read
  `3.77->3.77GHz 99% +-0.0% A`, the journey naming start, state, share, and steadiness
- resolution: prints on every run, plain runs included, and the live placement sweep read
  floors of 0.06 / 0.13 / 0.98 ns, scaling with each placement's real drift
- suggest-freq: live on the 3900X it descended from 3801 MHz, verified the hold (3.77 GHz
  delivered, 130 samples), and ended with `pin_mhz = 3801`
- pin and restore: `--pin-freq=3801` runs pinned and restored on normal exit live. The panic
  and SIGINT paths replay the same restore plan through the drop guard and the armed signal
  handler, exercised by review and unit tests rather than live
- `--pin-cpus`, the knobs, and the docs split: verified by smoke runs and inspection
- port preserved behavior: full validation (fmt, clippy -D warnings, tests, install) green at
  all twelve rungs, versions 0.26.0-0 through 0.26.0-11

### Deliberation

**Ported, not rebased** (wink, 2026-08-15): `jj rebase` of the original `measure-reproducibility`
branch produced four conflicted commits and was op-restored away. What worked was copying the
branch's `*.rs` files onto `main`, which passed `cargo check` on the first try. The full story is
[Port measure reproducibility][25] below.

**The original ladder's first three rungs fold into the opening.** Their code is the port,
their review happened on the original branch, and re-laddering them would re-commit finished
work. The remaining rungs carry over with their specs intact in `Ladder details`.

**The title reuses the stalled cycle's.** That cycle never reached a chores section, so nothing
in chores-07 collides, and the greppable stem stays continuous across both attempts.

**The bookmark keeps wink's name.** `port-measure-reproducibility` was created before this
ladder existed, so it is not the title's slug. A taken exception to the bookmark-naming rule
rather than a rename of wink's bookmark while he is away.

**0.26.0 on wink's call**, minor rather than patch: the record subsystem and the extracted
renderer change the system's shape, and wink stamped `0.26.0-0` for this opening after 0.25.3
and 0.25.4 were spent on the messaging fix and the port's pre-cycle state.

**The settle fix is its own rung, and first** (wink, 2026-08-15). It was born as a bullet on
the qualify rung during a qualify-environment review, and moved out because the diff lands in
`gauge.rs` / `harness.rs` and improves every report, not just qualify's, so burying it would
make that rung's diff disagree with its title. First among the Work rungs because it is ready
today and because order protects the evidence: records and the three-box rerun archive whatever
the settle cell says.

**The config adopts vc-x1's markdown carrier** (wink, 2026-08-15): a `.md` whose `toml` fences
concatenate into the parsed config, chosen for being self-documenting with links. Its own rung
rather than a fold into the frequency rung, so the format change and the frequency feature
review separately. Free of migration, no config file existing anywhere yet. Both types stay
supported with a one-type-per-directory hard error (wink), and `.md` is recommended at the user
level too, diverging from vc-x1, whose user config stays TOML only because their tool
machine-writes it.

**The harness gains the frequency mutation** (wink, 2026-08-15), reversing the original rung
spec's diagnosis-never-mutation rule. The reasons the rule existed (root, global, persistent)
become the design constraints instead: pinning happens only on an explicit command or flag,
every catchable exit path restores, and restore converges to a steady state the user declared
in config rather than to a remembered one. `qualify-environment` stays read-only, the mutation
living in its own commands. A first draft saved the displaced state to a transient file and
restored that. Rejected (wink): transient files are a demonstrated failure mode here, and
displaced-state restore ratchets on back-to-back runs, worst on a battery-powered device.

**The old branch still holds one unlanded records commit** (musl/libc build-matrix design, a
Todo entry plus a chores-06 design subsection). Deliberately not folded into this cycle, being a
different topic. It stays on the old branch until re-homed, and the branch is not deleted.

**The settle-journey rung is a mid-cycle insertion** (wink, 2026-08-17). The landed settle cell
prints the measurement floor as a measurement: `0.01s @4.09GHz` on nearly every healthy run,
the time being the first probe's timestamp rather than anything measured. The replacement was
designed in conversation (the start->end journey, the transition time, a settled-state
steadiness rating, and a tick-graph profile line in `-v`), and the rung sits after the
frequency rung so the qualify rung's parser work sees the final format. Taken exception,
recorded here per the draft self-consistency rule: the four pushed rungs carry the
pre-insertion ladder and are not rewritten, a TODO-snapshot rewrite of published commits buying
nothing a reader needs. The two-regime work this conversation also raised (regime config key,
`suggest-freq`) stays out, homed as Todo ranks 2 and 3 for the next cycle.

**The qualify rung halts unbuilt** (wink, 2026-08-19). The settle-cell rework left the
environment rating "better but not good enough", and the rating redesign (Todo's "Rethink
environment rating") has to come before the qualify rung builds more on top of it. Nothing of
the rung is committed, so the halt is a ladder edit on the draft bookmark, the spec moving back
to `## Todo` under "Rethink environment rating, then resume the qualify power-policy rung",
and the pushed rungs keep their pre-edit TODO snapshots, the same taken exception as the
settle-journey insertion above.

**suggest-freq pulls in from Todo** (wink, 2026-08-19), reversing the settle-journey entry's
homing of it to the next cycle: the zcr-mpsc campaign needs the discovered rate, so the
guidance half is part of this cycle's deliverable. Its spec asked for the record and LSC rungs
first, the record has landed, and the LSC rung stays and goes ahead of it, so the dependency
holds. The two-regime config key stays out, still next-cycle.

**The pin flag renames to --pin-cpus** (wink, 2026-08-19), and the project adopts the kernel's
vocabulary: a CPU is the schedulable logical processor sysfs numbers, a core is the physical
core hosting two of them, and SMT siblings are the pair sharing a core. Plural cpus over the
first-draft `--pin-core` because the flag takes a list and the record field is already
`pin_cpus`. Its own rung ahead of suggest-freq, so the format change and the feature review
separately and the campaign's recorded command lines carry the final names.

**Block sleep and warmup become knobs, defaulting to zero** (wink, 2026-08-19). Designed in
conversation after wink's three 7600x runs (all grade A, means 16.2 to 16.5 ns with block
count the only knob turned, LSC printing 0.0 ns throughout) showed the hidden 1-10 ms sleep
and the unrecorded 2 ms warmup shaping results invisibly. Zero is the neutral setting so the
tool never naps or discards samples unasked, the cost taken knowingly: sleepless blocks are
partitions, not replicates, so the CI95 / LSC rows gate on a nonzero sleep. A mid-cycle
insertion like the settle-journey rung, placed ahead of the LSC rung on purpose: gap-separated
blocks are quasi-runs inside one invocation, the validation data the drift floor is checked
against.

**The Work-rung pushes ran on an explicit scoped delegation** (wink, 2026-08-19): rule 2's
per-push stops waived in conversation ("I'm giving you the permission now"), recorded here
as the taken exception the hard rules ask for. Flow was not waived, full validation running
before every push, and the close-out stays outside the scope, waiting on wink's review.

**Documentation joins the cycle as two rungs before the closing** (wink, 2026-08-19): the
interpretation this session's reports needed from an assistant belongs to the tool, "not
everyone is going to have an assistant". The ~1,200-line README splits into a `docs/`
directory with README as the front door, and a report reading guide teaches every surface
from wink's live 3900X placement sweep. Two rungs so the mechanical move and the new prose
review separately. Picks up Todo's "Report interpretation guide". The same conversation set
the close-out shape (a trapezoid back before the opening, wink approving the `main` rewrite
this needs, the repo being single-user) and moved the zcr campaign to its own follow-on
cycle, its records committed under `data/`. The docs pushes ride the standing delegation,
and the close-out stops for approval after its work is prepared.

### Ladder details

#### feat: measure reproducibility opening

Land the stalled branch's finished work on `main`'s line as one commit: the policy and
clock-quantum report rows (the original's first Work rung), the renderer extraction into
`src/report.rs` (its second), and this cycle's records. The extraction went before the record
rung on purpose, so that rung's diff reads as "add the record" rather than "move 350 lines and
add the record", and the record's second consumer is what makes the model/renderer split honest.

Landed carrying more than the port: the laddering session (2026-08-15) rewrote the problem
statement in plain-programmer terms, grew the ladder by three rungs the original did not have
(the settle clock gate, the markdown config carrier, the frequency commands), moved wink's port
narrative in from chores-07, and swept nine Done entries into done.md at the opening beat.

Gotcha the pre-push validation caught: the `*.rs`-only copy carried the acceptance test's new
doc ("behind the `acceptance` feature") but not the `Cargo.toml` half that enforces it (the
`acceptance` feature and the `[[test]] required-features` gate), so the machine-graded test
silently rejoined plain `cargo test` and failed on a busy box. Both pieces restored from the
branch's `Cargo.toml` by hand, the copy having excluded that file for its version stamp.

#### fix: the settle cell reads the clock

The settle cell lies on exactly the box that needs it (wink, 2026-08-15, rated during a
qualify-environment review). `gauge::settle` reports the earliest warmup suffix that grades A
on timing alone, and a box parked flat on the un-boosted base clock grades A immediately, so
the 3900x under `powersave` reads `0.01s` and then transitions mid-bench to an F. Both cells
are truthful about what they measure, and together they mislead.

- the warmup exit already knows better: `classify_warm` gates a timing-A window behind
  `clock_stable` ("steady is not settled when the box is mid-climb", the measured 7600x dwell).
  `settle` takes the same clock series and the same gate, so a suffix counts as settled only
  when the clock held still across it. The series is already collected, so nothing new is
  measured
- name the state it settled into, not just when: `0.81s @ 4.35 GHz` (or "settled at base
  clock") is diagnostic where a bare `0.01s` misleads by omission
- `qualify-environment`'s `parse_settle` reads the cell text, so the parser updates with the
  format in this same commit
- the cell stays read-but-not-scored (test-enforced), which is what kept the defect from ever
  contaminating a verdict
- first among the Work rungs because it is ready (the clock series exists today, nothing here
  depends on the config, frequency, or policy rungs) and because order protects the evidence:
  landing before the record rung means no archived record ever carries the misleading time, and
  landing before the three-box rerun means the rerun's settle cells are honest

As built:

- `clock_stable` moved from the harness into `freq.rs` with its tolerance, one home for the one
  rule both gates now read, and the settle scan gates each candidate suffix on it beside the
  timing grade
- settle is scored in the harness, where the clock series is alive, and rides `RunOutput` as
  `warm_settle`: the report had been recomputing it from probes alone because the series never
  left `warmup_and_probe`, which is *why* the cell was timing-only
- the named state is the settled suffix's **median** GHz, so one odd read cannot name it, and a
  box with no readable driver keeps the bare time, never a guessed state
- `Settle::At` became a struct variant carrying the state, and qualify's parser learned the new
  cell in the same commit, its read-but-not-scored quarantine untouched
- seen live on the 3900x first run: warmup `0.01s @4.09GHz` beside a bench-row step at 1.90s,
  the cell now naming the boosted state warmup certified while the bench watched the box leave
  it
- `qualify-environment`'s own table carries the state too, added when wink's first 0.26.0-1
  runs showed bare times where he was looking: its settle column had been reformatting the
  parsed time alone. First table with it: `0.01s @4.47GHz` beside an F, a boost state the box
  could not hold, against two runs holding ~4.08 GHz

#### feat: write a per-run JSON record

The report prints and is gone. This rung adds `--record <path>`, a side channel that appends one
NDJSON object per bench result alongside the unchanged display. Design decisions taken at the
original pickup (2026-08-04):

- the record is per bench *result*, not per process: `all` emits one record per bench sharing
  the host / policy / clock stamp
- the display is never traded for the file: recording is a side channel, not a mode
- NDJSON, one object per line: `jq -s .` makes an array on demand, an interrupted run still
  parses, and per-run files concatenate with `cat`
- the tool names the file when handed a directory, because a fixed name is exactly what killed
  the powersave series (`run.sh` reused `u1.txt`..`p8.txt` and the rerun clobbered them)
  - `--record <dir>/` writes one file per run, stamped `<ts>-<host>-<bench>.ndjson`, and
    `--record <file>` appends to that file. The path's shape picks the mode
  - the open is `O_APPEND | O_CREAT`, never `O_TRUNC`, in both modes. The no-truncate invariant
    is what actually protects the evidence, whoever chose the name
  - basic ISO to the second in the filename (`20260804T093221Z`: no colons, lexicographic order
    is chronological order), RFC3339 with millis inside the record, plus the local offset as its
    own field. Millis are not decoration: `all` emits several records inside one second, so each
    also carries a process id and an index within the process
- `--tag k=v` is recorded verbatim and never interpreted, so a driving script labels condition /
  policy / box without the tool needing a notion of "series"
  - a per-run stamp is not a per-series stamp: only the caller knows which runs form one
    experiment, so `--tag series=<ts>` carries that, in every record of the series
  - no tag key is ever substituted into a path. That is where a template language starts
  - it lands with the record: a field of the same struct sharing all of its plumbing, so a rung
    of its own would review nothing
- the record carries a **fixed quantile ladder** (0.01 / 0.1 / 1 / 5 / 10 / 25 / 50 / 75 / 90 /
  95 / 99 / 99.9 / 99.99), not the report's populated bands, whose labels move with the data. It
  is what makes the A/B estimator question answerable after the fact, cheap now and impossible
  to backfill
- the record carries the **series of block means**, not just their summary: an aggregate cannot
  be decomposed, and the series is what lets within-run scatter be compared against across-run
  scatter. Bounded by a cap: keep every block mean while blocks <= 1000, summarize beyond
- **the clock rides the record too, sampled at batch seams** (wink, 2026-08-16): one delivered
  frequency read per seam (~50 ms apart, one sysfs read, the reader exists in `src/freq.rs`),
  so per-block and per-run min/max/median frequency fall out of the record for free
  - today the clock series stops at warmup's end, which is what hid the 4.09 -> 4.53 GHz
    mid-bench climb from every qualify table: the report named where the box started and that
    it moved, never where it went
  - it is also the pin's verification: under `pin-freq` the per-run stats collapse to the pin
    (min = max), so "the pin held" is a mechanical check on every run's own data, and the
    frequency rung's acceptance leans on it
  - subsumes the sampling half of the "Seam-clock attribution" Todo entry, whose
    report-labeling half ("clock moved" on a mid-run step) stays ranked there
- the record documents its own fields, and a test enforces it
  - `describe-record`, a command word beside `all` / `qualify-environment`, prints the field
    dictionary: name, unit, one-line meaning. Not `--help`, which documents *inputs*
  - one source of truth is a const descriptor table, kept honest by a test that serializes a
    sample record, walks its keys, and fails on any key with no entry
  - every record carries `schema_version`, so a dictionary printed by today's binary can be
    checked against a record written by an older one
  - later polish, not this rung: per-field lookup (`explain frame_ns`) and generating README
    text from the same table
- absent is not zero: rpi5-20cd has no EPP and no `cpuinfo_avg_freq`, so every policy field is
  an `Option` recorded as absent
  - three states, not two: absent, present and uniform, present and split across policy groups.
    `freq::PolicyField` carries the token plus a `uniform` flag so the display can say
    `(mixed across CPUs)` rather than let one CPU stand in for the box
- ordering is part of the record: the 2026-08-03 pinned series was bimodal by *position*
  (p3-p6), which is invisible without a per-run wall-clock start
- comparison across the three boxes is within-box only: each box's pinned-vs-unpinned delta and
  its run-to-run scatter, never nanoseconds against nanoseconds
- the policy fields include the clamp and boost state (governor, EPP, `scaling_min_freq` /
  `scaling_max_freq`, boost), so a pinned run (next rung) is verifiable from its record rather
  than from trust

As built:

- the shape: `src/record.rs` is `RunOutput`'s second consumer, the one the opening's renderer
  extraction was staged for
  - `build_record` assembles the object, and a `Recorder` resolved at startup writes it
  - a bad `--record` path or `--tag` fails at startup, before any bench spends minutes measuring
  - the harness still never renders, and the display is untouched
- the record's fields
  - absent serializes as `null` with the key kept, never a missing key, so every record carries
    the identical 39-key set
  - the dictionary is enforced both ways: the test fails on an undocumented record key and on a
    dictionary entry naming no key, so `FIELD_DOCS` cannot drift from the data
  - `t_start` is captured, not reconstructed: `RunOutput` gained `wall_start`, taken the moment
    measuring starts, so the stamp is exact and warmup is excluded by construction
  - `BlockStats` now keeps its `means_ns`, so the recorded series is the very values the CI and
    LSC were fit from rather than a recomputation
  - provenance rode along beyond the spec list: binary version, `pin_cpus`, min/mean/stdev/max,
    `suspended_s`, and the warm verdict (`warm_exit`, spend/budget, settle time and clock), each
    a field a re-analysis would otherwise have to ask the dead session for
- the seam clock
  - sampled ungated by `--no-env-probe`: that gate exists for the ~256 us micro-probe, and one
    sysfs read is orders cheaper
  - first smoke run (0.5 s min-now): 12 seam samples, min/max delivered kHz falling out of the
    record with one `jq` line
  - its timestamps are raw integer ns from warmup start (wink, at review): the record stores
    the raw elapsed read and seconds stay derivable, the same rule that keeps policy tokens
    raw. The derived scalar summaries (`duration_s` and kin) stay float seconds
- paths and failure
  - an existing directory without the trailing slash also selects dir mode, since opening it as
    a file could only fail later and less clearly
  - a failed record write exits 1 loudly: the record is the run's evidence, and losing it
    silently is the failure the flag exists against
- the cost: timestamps are hand-rolled UTC (`civil_from_days`) plus `localtime_r` for the
  offset field, so `serde_json` is the rung's one new dependency
- validated live
  - dir and file modes both exercised, and the records parsed with `jq`
  - `describe-record` prints the 39-field dictionary
  - `--tag` requires `--record`, enforced by clap

#### feat: adopt the markdown config carrier

The config becomes a `.md` file read the way vc-x1 reads `vc-config.md` and `.vc-config.md`:
the `toml` fences, concatenated in document order, are the TOML that gets parsed, and the prose
between them never reaches a parser. The prose is the format's point (wink, 2026-08-15): the
config documents itself, with markdown links, one `##` section per key doubling as that key's
anchor.

- the loader keeps its layering (built-in < XDG < project-local) and gains a fence filter ahead
  of the existing TOML parse, the shape of vc-x1's `src/md_fence.rs`
- **both types are read, one type per directory** (wink, 2026-08-15): each layer accepts
  `config.md` or `config.toml` (project-local: `iiac-perf.md` or `iiac-perf.toml`), the fence
  filter running only for `.md`. Finding both in one directory is a hard error naming both
  paths, never a silent precedence: the user editing the ignored file would get no effect and
  no clue. The rule is per directory, so mixing types across layers is fine
- `.md` is the recommended form at both levels, user config included, and nothing forces the
  global to be TOML here: vc-x1 kept their user config TOML because their tool machine-writes
  it, and nothing in iiac-perf writes its own config
- no migration exists to do: `src/config.rs`'s loader exists but no config file does, on this
  box or in the repo
- the freq rung's `[freq]` steady-state section is the first real occupant, and its section's
  prose is where "what is a steady state and why declare one" gets documented for the user

As built:

- `src/md_fence.rs` is vc-x1's filter taken verbatim, its module doc rewritten to carry the
  provenance and the shared-crate candidacy, and its tests living beside it (vc-x1 keeps theirs
  in `config_md.rs`, whose loader is theirs, not ours)
- the sharing question got decided at this rung (wink, 2026-08-17): copy now, propose the crate
  after the cycle lands. A path dependency on vc-x1 was rejected (binary crate, and a
  sibling-checkout dep breaks `cargo install --path . --locked` elsewhere), and making their
  copy a lib module is their agent's work, reached by message. Backlog entry
  "Extract the md -> toml fence filter" holds the plan
- the loader's reshape: `xdg_path` became `xdg_dir`, a `resolve_carrier(md, toml)` decides each
  layer's file (both present is the hard error naming both paths), and `overlay` lost its
  missing-file half (existence is decided at resolve time) while gaining the fence filter keyed
  on the `.md` extension, so a fence diagnostic names the `.md` path and its real line
- left for a later beat, deliberately: `README.md`'s config section and `iiac-perf.toml.example`
  still describe the TOML carrier only. Converting the example to a rendered `.md` document is
  the natural close-out or follow-on edit, once the `[freq]` section exists to document

#### feat: read, pin, and restore the CPU frequency

For comparison the need is consistency, not speed (wink, 2026-08-15): a stable clock at any rate
beats an unstable fast one, so the harness gains the ability to hold the clock still and the
user gains commands to check, set, and unset that state at any time.

- three command words: `read-freq` prints the clock state (governor, EPP, min/max, boost,
  current frequency, per policy group), `pin-freq` holds the clock at one frequency (min = max,
  boost off), and `restore-freq` converges the box to the user's declared steady state
- **the default pin target is load-independent** (wink, 2026-08-15): with no frequency
  configured or given, `pin-freq` pins at base clock, the manufacturer's guaranteed frequency
  under sustained all-core load, so the unconfigured pin works for every bench as it stands
  today. Any faster "best" is workload- and schedule-dependent (thermal), and picking one is a
  measurement, not a default
  - discovery order: `acpi_cppc/nominal_freq` (reads 3801 MHz on the 3900x, the 3.8 GHz base
    clock matching the measured 3.7929 GHz TSC rate), then intel_pstate's `base_frequency`,
    else the highest non-boost frequency the driver lists. We think that last fallback covers
    the Pi, to be verified at implementation. `read-freq` prints what it resolved and from
    where
- a declared or given pin value is validated against the box at pin time, never in the
  abstract: the valid range is sysfs's per-policy `cpuinfo_min_freq`..`cpuinfo_max_freq`
  (discrete `scaling_available_frequencies` on drivers that list them), a bad value errors
  naming that range, and `read-freq` prints the range so the user picks informed
- **the restore target is declared, not remembered** (wink, 2026-08-15): the user's preferred
  steady state lives in the layered config's markdown carrier (a `[freq]` section, normally in
  the XDG file `~/.config/iiac-perf/config.md`, since the steady state is the box's, not the
  project's), written once by the user, beside prose saying what it means
  - why not save-and-restore-what-we-found: transient state files are a demonstrated failure
    mode in this repo, and restoring the displaced state ratchets on back-to-back runs. Run 2
    enters while run 1's pin is live, "restores" the pin, and a laptop or a phone is left
    pinned high. Convergence to a declared state is idempotent from any starting point
  - with no declared steady state, `pin-freq` refuses and says what to add, rather than pinning
    with no way home. `read-freq` can print the current state in config form, ready to paste
- `read-freq` needs no root and is shaped for a prompt or a status bar (wink's use case: a
  terminal prompt, a GUI panel beside the time-of-day): one short line, fast, stable format
- pinning writes sysfs and needs root, and the commands say so plainly when they lack it rather
  than half-working
- a run can pin at start and restore at exit, catching every exit path we can: normal return,
  panic, SIGINT/SIGTERM. The uncatchable paths (SIGKILL, power loss) need no saved state to
  recover from: `restore-freq` converges from anywhere, any time
- pin at or below base clock is the stable configuration: boost has nowhere to go and thermal
  throttling is unlikely to reach below base
- the record (previous rung) captures the clamp and boost state either way, so a pinned run is
  verifiable from its record

As built:

- the shape: `src/freq.rs` stays read-only and gains base-clock discovery, and the new
  `src/freqctl.rs` is the one module that writes sysfs, carrying the reversal's constraints in
  its module doc
- writes are staged as ordered plans of (path, token) pairs, every clamp move floor-first (min
  to the hardware floor, then max, then min), so each intermediate write satisfies the kernel's
  `min <= max` rule from any starting state, which is what makes restore converge from anywhere
- the declared state is validated against the box at use time, both directions: a knob the box
  exposes must be declared (restoring around it would leave a pin's residue) and a knob it
  lacks must not be, with governor and EPP tokens checked against the sysfs available lists and
  every error naming what the box actually offers
- the `[freq]` table replaces whole across config layers, never field-merges: the steady state
  is one declaration of one box's state, and half of one file's on top of half of another's
  would be a state nobody declared
- MHz at every user surface (config keys, `pin-freq 3800`, `--pin-freq=3800`, error messages),
  kHz only at the sysfs boundary
- the refusal carries its own remedy: `read-freq --as-config` prints the paste-ready `[freq]`
  section the error asks for, and the message warns that sudo may point $HOME at root's config
  while the project-local `iiac-perf.md` still works there
- the run guard engages before the Setup block prints, so the printed policy and the warm loop
  both see the pinned state, and a pin that fails partway restores best-effort before the error
  returns
- the signal path pre-renders the restore plan into C strings at engage time, so the
  SIGINT/SIGTERM handler is raw open/write/close (the async-signal-safe subset) and exits
  128+signal, while normal return and panic restore through the guard's `Drop`
- base-clock discovery per the spec's order, with two hardening details: `nominal_freq` values
  at or above 100000 are taken as kHz (platforms have shipped both units), and the
  `scaling_available_frequencies` fallback skips a top entry sitting exactly 1 MHz above the
  next, the ACPI turbo marker. We think the fallback covers the Pi, unverified until the
  three-box rerun
- the boost knob mirrors the read side: per-CPU `boost` files preferred, the global file the
  fallback, and this box turned out to expose the per-CPU form
- validated live on this box (563-4673 MHz range, base 3801 MHz from `acpi_cppc/nominal_freq`):
  the one-line `read-freq`, `--as-config`, both refusal paths, out-of-range rejection naming
  the range, and the write path failing loudly (the sandbox's read-only /sys standing in for
  the non-root case, whose EACCES adds the sudo hint)

#### fix: the settle cell shows the clock's journey

The settle cell prints the measurement floor as a measurement: on nearly every healthy run the
scan succeeds at probe 0 and the cell reads `0.01s @4.09GHz`, where the time is the first
probe's timestamp, a censored "no ramp seen" dressed as a duration (wink, 2026-08-17: "it
feels like settle = 0.01s @4.09GHz means nothing"). The cell becomes the clock's journey, and
`-v` gains the graph.

- the cell is `start->end time rating`, e.g. `3.60->4.09GHz 0.32s +-0.1%`: the first readable
  clock sample, the settled suffix's median, seconds from warmup start to the suffix, and the
  suffix's relative stdev
  - no observed change (the start sits within the stability band of the settled median):
    `4.09GHz from start +-0.1%`, no fake time and no arrow, the arrow appearing only when a
    real move happened
  - never settled (cap exit): `3.60->4.20GHz not settled`, the journey it was still on when
    warmup gave up. No rating, there being no settled state to rate. Today this case prints a
    clock-free `not settled`
  - clock unreadable (no `cpuinfo_avg_freq`): the timing-only forms stay (`0.32s`,
    `from start`, `not settled`)
- the rating rates the settled state, never the ramp: the arrow owns the ramp, and the rating
  answers "how still is still" inside the 1% gate
  - under a pin it should read `+-0.0%`, the pin certifying itself in every report, and a
    wandering `powersave` box shows a visibly fatter band, the two-regime difference made
    visible
  - on a fast exit the suffix is only the exit window, a handful of samples, so the rating is
    indicative rather than a precision instrument, and the record's seam clock stays the
    evidence
- `-v` gains a clock profile line, e.g.
  `clock: 3.60->4.09GHz ^^^v^----- (min 3.58 max 4.11, settled +-0.1%)`: one tick per
  inter-sample step, `^`/`v` a move the stability band would flag, `-` a hold within it
  - the deadband is `FREQ_STABLE_TOL`, shared with `clock_stable`, so the tick line is the
    stability gate's view of the series: the settled suffix reads all `-` by construction, and
    settle's start is visible as the point the ticks go quiet
  - `^`/`v`/`-` are typeable by design, hard rule 8 barring arrow glyphs from user-visible
    strings. A digit sparkline (level, not direction) was considered and dropped: ticks read
    without mental scaling, and magnitude is bracketed by the line's own numbers
  - direction-change counts were considered and dropped: the profile shows the shape directly,
    and a count without the shape is another number to explain
- `qualify-environment`'s `parse_settle` and its table column update in the same commit, the
  format-and-parser rule the settle rung set

As landed, the conceptual delta:

- `Settle` carries the journey rather than a scan timestamp: `At` gains `held_s`, `start_ghz`,
  and `rating`, `Never` gains its two journey ends. The record schema is untouched: `settle_s`
  keeps the ramp-end `t_s`, which no longer reaches the cell
- the cell's number is the settled share of the warm, a percent (wink, 2026-08-19, the fourth
  form reviewed): the ramp-end time read as a meaningless `0.01s` on a box already at speed,
  an absolute hold read as noise beside the warm budget, and the percent is self-scaling,
  `100%` ready all along down to the exit-window floor (~3% of a default warm), zero-padded
  to two digits so the column's digits align (wink). `00%` is reserved for never-settled, a
  settled share rounding up to `01%`. The spec's no-change and `from start` forms died across
  the same rounds: a journey that went nowhere prints its arrow (`4.09->4.09GHz`)
- the settled share is a graded signal (wink), the one place the clock decides a letter: the
  10-run qualify showed three `not settled` rows graded A, and a fast late ramp can finish
  inside the bench's first batches where no timing detector sees it, so the settle scan is
  the only witness. The share scores against `UNSETTLED` cutoffs (A from a quarter settled, F
  within 2% of never, never itself an F), the letter prints beside the cell like every
  signal's, and it folds into the warmup row's worst. A never-settled-is-F-outright rule came
  first and generalized on wink's "still all A's" review. qualify's median-settled stat
  counts never-settled runs as `00%` rather than skipping them
- every clock statistic reads the dominant CPU only (other CPUs' samples become `None`,
  positions preserved so the gate's probe alignment survives): an unpinned run's sampler rides
  the scheduler across cores, and the mixed series rated placement rather than the clock.
  Measured on the 3900X: +-11.9% mixed against +-0.2% filtered or pinned, found when the new
  rating first exposed what `clock_stable`'s migration fallback waves through
- the scan's clock gate ranges over the filtered series' readable samples
  (`filtered_clock_stable`) instead of reusing `clock_stable`, whose missing-sample fallback
  bailed true at every filtered-out gap and disabled the gate for unpinned runs. Caught by
  wink's grade-A-beside-bare-`not settled` qualify row, and measured as a `+-7.0%` rating
  inside the 1% gate. The `Never` journey's end also falls back to the last readable sample
  when the exit window held none, which is what had emptied that row's journey
- on a clock-readable box, no evidence is no certificate (wink, 2026-08-19): a suffix with
  fewer than two dominant-core readings cannot verify stillness, so it fails the gate and the
  run reads `00% F` rather than a timing-only share. wink's trigger row: an evidence-free
  `18%` graded B beside a verified `04%`'s D, the cell that knew less outgrading the one that
  knew more. A wholly clock-less box keeps its timing-only cells unpenalized, the bare-share
  form now being its exclusive signature
- the report's unstable-exit arm prints the scan's `Never` journey instead of a bare
  `not settled`, with the plain form kept as the defensive fallback
- the `-v` tick line is unabridged, one tick per step: a routine settled run prints ~140
  ticks, long but consistent with the per-probe table's ~141 rows on the same screen
- README's `Settle time` section rewritten around the cell anatomy, having still described the
  pre-clock floor-median algorithm, with the qualify column text, both sample blocks, and the
  `--settle-time` flag doc synced. The explain-it-in-the-README test drove the design: the
  ramp-end time and the mixed-CPU rating both failed it (wink)
- widths widened for the fattest cell (`4.84->5.24GHz 100% +-0.2% A`): the grade block's
  settle column 15 -> 27, qualify's 16 -> 27

#### feat: block sleep and warmup become knobs

Blocks carry two invisible behaviors: a random 1-10 ms sleep between blocks and 2 ms of
unrecorded post-wake warmup, neither shown in any report nor settable. wink's 2026-08-19 7600x
runs made the cost concrete: three A-graded runs whose means span 16.2 to 16.5 ns with block
count the only knob turned, and no printed number honest enough to say whether the movement is
real. The knobs surface, and zero becomes the neutral setting: a run never sleeps or discards
samples unasked (wink, 2026-08-19).

- `--block-sleep <spec>` and `--block-warmup <spec>`, each with a config key landing in the
  same commit per "Config keys stay CLI-settable". Sleep accepts a scalar or a range
  (`1-10ms`), the range re-rolled per block, because fixed sleeps invite the recorded
  flip-zone hazard (fixed 0.5 ms sleeps straddled both 3900X states)
- defaults are zero: sleepless blocks are partitions, not replicates, so the replication rows
  (CI95 / LSC) print `-` unless the sleep is nonzero. The block-mean series still records
- both values print in Setup whenever blocks run, zeros included, and ride the record with
  dictionary entries, so a record is interpretable without its command line
- a long sleep with zero warmup is the cold-wake instrument arriving early: cold samples land
  in the histogram as a visible band shoulder. The separated first-K decay profile stays with
  the Cold-wake Todo entry
- ahead of the LSC rung on purpose: gap-separated blocks are quasi-runs inside one invocation,
  the validation data the drift floor is checked against

As built:

- `src/timespec.rs` is the span grammar's one home: scalar or range, `us`/`ms`/`s`, the unit
  mandatory on any nonzero value (a bare `5` meaning ms to one reader and s to another is the
  ambiguity the knobs exist to remove), a range's trailing unit distributing to its lower end
- `run_blocked` skips the sleep and the warmup at zero and draws the sleep uniformly from the
  span per block. `BlockStats::from_means` gained a `replicated` flag: CI95 / LSC became
  `Option`, `None` for sleepless partitions, so the partitions-cannot-replicate rule lives in
  one place and the report and the record both inherit it
- the report prints a bare `-` (no unit) for a withheld CI95 / LSC, a unit dressing the
  absence up as a number
- the record added `block_sleep_min_s` / `block_sleep_max_s` / `block_warmup_s`, null when
  not blocked, CI95 / LSC now null for partitions too, and `schema_version` bumped to 2
- Setup prints both knobs whenever blocks run, zeros included, each naming its consequence
  ("blocks are partitions", "records from the first post-wake call")
- smoke-verified: sleepless `--blocks 4` shows the partition note and dashes, and
  `--block-sleep 1-10ms --block-warmup 2ms` reproduces the retired hidden behavior explicitly

#### fix: LSC gains a run-to-run component

LSC is scoped to within-run block agreement but reads as a run-to-run bound. The best
configuration printed LSC 0.022 ns against a measured run-to-run stdev of 0.057 ns, so
single-run against single-run resolution is ~0.157 ns (0.71%) where the report prints 0.10%:
about 7x optimistic, and it did not improve when the environment did. Given the A/B purpose,
this is the cycle's most valuable rung.

- the fix is a **variance-versus-aggregation curve**, not a second measurement: a single run
  cannot measure run-to-run scatter directly. Aggregate its blocks in groups of 1, 2, 4, 8, ...
  and watch whether variance falls as `1/n`. Where it stops falling is the drift floor, and that
  floor is the run's honest resolution
  - this is Allan deviation (IEEE Std 1139), the standard tool in clock metrology for "how long
    should I average". The harness is already a clock project, so the machinery is familiar
  - the alternative, spawning children as `qualify-environment` does, is a much larger change
    and still cannot re-roll thermal or P-state history, so it does not reach the missing
    component
  - the duration estimate falls out of the same curve for free:
    `t_needed = t_now * (SE_now / SE_target)^2`, valid only where the `1/n` scaling still
    holds, which the curve is what tells us
- the curve fits on **batch means**, not block means (wink, 2026-08-19): batches are the
  contiguous ~50 ms slices Allan deviation is defined over, `BatchSummary` already carries
  `mean_ps` and `count` on every run, and a 127-batch run reaches aggregation groups a
  10-point block series cannot
  - the honest resolution therefore prints on every run, `--blocks` or not, which unhooks the
    claim from the blocks-default question
  - the aggregation weights by sample count, batch durations being uneven (fast benches flush
    on the sample buffer at ~15-40 ms, slow ones on the 0.05 s timer)
  - the record gains the batch-mean series beside the block-mean series, or a re-analysis
    cannot reproduce the curve
  - block stats stay as the replication cross-check: blocks re-roll scheduler and frequency
    state across sleeps, a different question from drift within one stretch
- naming it accurately matters as much as computing it: a within-run bound must not print in a
  way that reads as a run-to-run one, which is the whole defect

As built:

- `src/resolution.rs` fits the curve: count-weighted group means at group sizes 1, 2, 4, ...,
  the LSC formula (`t(0.975, 2J-2) * s * sqrt(2/J)`) applied per level, and the floor is the
  worst level. Under white noise the levels agree, under drift the deep ones rise, so the max
  cannot understate
- levels past the first need J >= 8 groups: at J=2 the t multiplier alone inflates the claim
  2.2x, so a deeper level would report its own estimator noise as drift. The cost is that a 5 s
  run's curve sees drift only up to ~8 batches (~0.4 s), which keeps the floor a lower bound
  and the naming honest
- the report gained a `resolution` row, every run, at least 2 decimals so a real claim never
  rounds to the fictional 0.0 it replaced. The blocks rows stay as the replication cross-check
- the record gained the batch series (`batch_mean_ns` / `batch_samples`, count-weight merged
  past 1,000 points with `batch_agg` saying by how much) and the claim
  (`resolution_ns` / `resolution_batches` / `resolution_groups`), `schema_version` now 3
- one panic site introduced, documented per the unwrap discipline: the floor `max_by` unwraps
  behind the b >= 2 guard that guarantees a nonempty curve
- smoke-verified: a plain 2 s min-now printed `resolution 2.11 ns` beside `LSC`-less rows, and
  a sleepless `--blocks 4 --record` run wrote null block LSC beside a real resolution

#### fix: the pin flag names CPUs

`--pin` beside `--pin-freq` no longer says which of the two things it pins, and the code calls
the same list three names: the `CORES` metavar, `pin_cores`, and the record's `pin_cpus`. The
project adopts the kernel's vocabulary (wink, 2026-08-19): a **CPU** is the schedulable logical
processor (sysfs `cpuN`, one affinity-mask bit), a **core** is the physical core hosting two of
them, and **SMT siblings** are the CPUs sharing a core. vCPU stays out, being virtualization
vocabulary with no referent on bare metal.

- the flag becomes `--pin-cpus`, metavar `CPUS`, help text swept. Whether `--pin` survives as a
  hidden clap alias is decided at the rung, nothing external consuming the CLI
- the code sweeps to match: `pin::parse_cores` -> `parse_cpus`, `pin_cores` -> `pin_cpus`, and
  comments saying core where they mean CPU
- the record field `pin_cpus` is already right and is schema, so it does not change
- README renames the flag docs and gains a short Terminology note stating the convention, the
  reader-facing home for the vocabulary placement-map.md already speaks
- the prior terminology discussions stay where they rank: Todo's "Tighten thread/CPU
  terminology" holds the wider docs sweep (software thread against CPU), of which this rung
  delivers only the `--pin` bullet, and the topology-level vocabulary (lCPU, LLC domain) stays
  with "Topology-aware pinning and lCPU terminology"
- ahead of suggest-freq so the campaign's recorded command lines carry the final names

As built:

- `--pin-cpus` with metavar `CPUS`, and `--pin` kept as a **hidden clap alias** so wink's
  muscle memory and any old script keep working while the help teaches the new name
- the code sweep: `parse_cores` -> `parse_cpus`, `pin_cores` -> `pin_cpus` (RunCfg field, the
  main local, qualify's pass-through), `core_for` -> `cpu_for`, `print_core_id` ->
  `print_cpu_id`, error prefixes, and comments that said core meaning CPU. The record field
  `pin_cpus` was already right and did not change, so no schema bump
- the Setup `main pin` cell now says `CPU 0`, and qualify's banner passes `--pin-cpus`
  to children
- README gained a `## Terminology` section (CPU / core / SMT siblings / software thread, the
  kernel's own words) and the flag docs re-anchored to it, examples renamed throughout
- smoke-verified: `--pin-cpus 0,1` and the `--pin 0` alias both pin, the banner reading
  `CPU 0 (pool slot 0; warm + run)`

#### feat: suggest-freq measures the pin frequency

Measure the best pin frequency instead of defaulting to base clock (raised 2026-08-15 during
laddering, pulled in from `## Todo` 2026-08-19 for the zcr-mpsc campaign). "Best" is the highest
frequency the box *holds* under the intended workload and schedule, so it is thermal and
duty-cycle dependent: a short run passes at a frequency a long run throttles from, and the
2026-08-02 data showed the schedule selecting the state.

- shape: descend from max-with-boost-off, pin each candidate, drive the load for at least the
  intended run duration, verify with `clock_stable` plus the grade, and report the highest
  candidate that held, named with the schedule it was measured under
- the load is the real bench, not a synthetic stand-in: the command takes the bench word and
  the run flags (`suggest-freq zcr-mpsc-2t --pin-cpus 0,12 -d 5`), because 2t on SMT siblings
  heats differently than 1t on one CPU, so the held frequency is per-bench and per-layout
- it mutates, so it reuses the pin-freq / restore-freq machinery and inherits its exit-path
  restore guarantees (normal exit, panic, SIGINT)
- guidance beyond the number: report what was measured, each candidate, how long it held, and
  under what load and schedule, ending with the config line to paste (`pin_mhz = ...`), the
  same paste-ready shape as `read-freq --as-config`
- its spec wanted the record and LSC rungs landed first, and both are by this point in the
  ladder: the block-mean series and the drift floor are the evidence a suggestion is judged by

As built (draft, not yet live-run: pinning needs root and the dev sandbox's sysfs is
read-only, so the descent is exercised to the first pin write and the rest is unit-tested):

- the command lives in `freqctl.rs`, the mutation home, reusing the plan machinery whole:
  `resolve_steady`'s refusal without a declared `[freq]`, `pin_plan` per candidate,
  `restore_plan` on a drop guard plus the armed SIGINT/SIGTERM handler
- `suggest-freq BENCH` rides the normal bench setup path (inhibit, config, Setup block, all
  knobs), then replaces the bench loop with the descent, so the candidate runs use exactly the
  session's run configuration and each candidate prints its ordinary bench report as evidence
- the ladder: a discrete driver descends its `scaling_available_frequencies`, a continuous one
  steps 100 MHz down from the base clock (the boost-off ceiling), 12 candidates at most, and
  no discoverable base clock on a continuous driver is a refusal rather than a guess
- the verdict: a sampler thread reads the delivered clock every 50 ms on the pin pool's first
  CPU while the bench runs. Held = series stable within `FREQ_STABLE_TOL` and median on the
  target. No samples is never a hold, unverifiable claims failing rather than grading well
- what the grade rung would have added is not captured mechanically: the bench's own printed
  grade block is beside each candidate for the human, and parsing it back was left out on
  purpose (qualify's parser is the precedent and its redesign is pending)
- the ending is the paste line, `pin_mhz = NNNN`, named with the bench and duration it was
  measured under
- arg shapes error loudly: zero or two bench words, a multi-match prefix, `--pin-freq`
  alongside, and a box with no readable delivered clock all refuse with named reasons

#### docs: split the README into a docs directory

README grew to ~1,200 lines: a reader hunting one flag scrolls past design history, and the
depth buries the five lines a newcomer needs (wink, 2026-08-19, "let's break up README.md").
README becomes the front door and a `docs/` directory takes the depth.

- README keeps: what the tool is, Terminology, the design brief, a short usage synopsis with
  pointers into `docs/`, and the dev-facing tail (testing, workflow, license)
- `docs/` gains per-question files: `usage.md` (the full flag reference, commands, shell
  completion), `config.md` (the config file), and `report-guide.md` seeded with the existing
  report-describing sections (Setup banner, the two grades, settle time, run-grade signals,
  the example runs)
- the split is verbatim moves plus link fixes, no rewrites, so the diff reviews as structure.
  The next rung rewrites the guide
- in-repo links into README anchors sweep in the same commit

As built:

- README went from ~1,200 lines to ~190: intro and highlights (refreshed to name the record,
  resolution, and frequency features), a new Documentation map, Terminology, the design brief,
  a short Usage with a taste of commands, and the dev tail
- `docs/usage.md` took the command words, the full flag list, the examples, and shell
  completion. `docs/report-guide.md` took every report-describing section verbatim (Setup
  banner, band table and ladder, warnings, reading a report, comparing with blocks, the two
  grades, settle time, run-grade signals, label styles, all results, verbose, default vs
  pinned), ready for the next rung's rewrite
- `docs/config.md` is the one non-verbatim move, its content having gone stale twice over: it
  still said TOML-only (the markdown-carrier rung never updated it) and knew neither the
  block knobs nor `[freq]`. Refreshed while moving, declared here rather than slipped in
- inbound links: chores-04's reading-a-report link retargeted, bands.rs's doc comment points
  at the moved table, and chores-01..03 keep their `#chores-format` links, already dangling
  before this rung, a separate correction if wanted

#### docs: the report reading guide

The report is dense by design, and the decoder lived in sessions rather than in the repo:
this cycle's own output needed an assistant's interpretation twice in one day (wink,
2026-08-19, "not everyone is going to have an assistant"). `docs/report-guide.md` becomes a
reader-oriented walkthrough teaching what each surface means and, above all, what to conclude
from it. Absorbs the "Report interpretation guide" Todo entry, whose spec follows.

- open with the measurement hierarchy, the question wink had to ask in conversation: call ->
  sample (`inner` calls, one timing) -> batch (time axis) -> block (replication axis) -> run,
  and which report number is computed at which level
- surfaces to cover:
  - the Setup block and the header bracket (duration / warm / outer / inner / calls / blocks /
    batches / labels)
  - the band table: first/last/range/count/mean per quantile band
  - the summary rows: mean/stdev, the trimmed row, quantum, resolution, blocks mean/CI95/LSC
    and when the dashes appear
  - the grade block: env warmup/bench vs run rows, per-signal letters, the settle cell's
    journey form
  - the -v warmup table and its exit/window/clock summary line
- lead with worked examples:
  - wink's 2026-08-19 3900X sweep: suggest-freq holding 3801 MHz (delivered 3.77 GHz, the
    ~0.8% nominal gap absorbed by tolerance), then zcr-mpsc-2t at the pinned clock across
    placements reading 61.5 / 107.0 / 395.9 ns trimmed with resolutions 0.06 / 0.13 / 0.98 ns,
    cross-CCX barely moved by the core pin (fabric domain), and its interference C
  - the 2026-08-02 3900X trio (plain -d 5, --blocks 100, --blocks 1000): duty cycle selects
    the bistable state (sustained ~21.8 ns, bursty 24.0 ns), grade A certifies internal
    consistency of the state the run held rather than a canonical number, so A/B wants a
    matched duty cycle
- the two-regime workflow (wink, 2026-08-17): tune pinned, where the resolution shrinks until
  "did this tweak clear it" resolves in a few runs, then confirm the winner unpinned, whose
  number is what the real world sees. We think a pinned ranking can occasionally flip
  unpinned, which is why the confirm step exists. The pyperf tune/reset pair is the same
  idea, ours being pin-freq / restore-freq

As built:

- the guide opens with the two sections a hurried reader needs: the measurement hierarchy
  (call -> sample -> batch -> block -> run, with each report number placed on its level) and
  the header-bracket decoder, both born as questions wink had to ask in conversation
- a new "The summary rows" section reads the whole stack top to bottom (mean/stdev, trimmed,
  quantum, resolution, blocks rows), each row phrased as the question it answers, the
  resolution row's Allan-curve mechanism included
- "What to conclude: a worked example" is the 2026-08-19 3900X session end to end:
  suggest-freq's hold (and the delivered-vs-nominal 0.8% gap), the three-placement sweep
  table with resolutions, the cross-CCX fabric-domain finding marked as We think, the
  two-regime workflow, and the 2026-08-02 duty-cycle lesson ("grade A certifies the state the
  run held, not a canonical number")
- the Setup-banner section caught up with the block-knob and freq-pin lines, and the seeded
  sections from the split rung stand unchanged beneath the new material

#### feat: measure reproducibility closing

Closing out the cycle. Gotchas the close-out surfaced:

- the trapezoid looked like it needed a `main` force-push, and does not: the cycle's first
  five rungs sit on `main` linearly, but the recipe rebases only the close-out commit into
  the merge, so `main` fast-forwards onto it and only the first-parent *view* changes. The
  worry was raised, priced, and dissolved by reading the recipe rather than remembering it
- a cycle whose evidence stage moves out (the campaigns, re-homed to a follow-on cycle) still
  closes cleanly because the acceptance check never named the campaigns: evidence and
  acceptance were separate lists from the start, which is worth keeping deliberate
- the four Work-rung and two docs-rung pushes ran under an explicit scoped delegation, the
  recorded exception in the Deliberation, with the close-out push itself back outside it

### What the harness is for

**A/B comparison of algorithm changes**: did this change make it faster or slower. Stated here
because it decides calls like the rung specs above and is written down nowhere in the repo
(wink, 2026-08-04). It deserves a permanent home in the README's overview, and the cycle block
is where it landed first.

- the far tail is a validity check, not the score: beyond ~n4 on a 20 ns operation it measures
  OS interruptions (context switches, IRQs, faults, migrations), so it says whether to trust a
  comparison, never whether the algorithm improved
- the near tail is not in that bucket: cache miss rates, a ring's wrap boundary, and the 2t
  benches' producer/consumer phase relationships live around p99, and a change that improves the
  median while doubling p99 is usually a worse algorithm
- a comparison needs a resolution claim, not just a number, which is what makes the LSC rung
  load-bearing rather than last: an A/B verdict is only as good as the smallest delta the tool
  can honestly distinguish

### The 2026-08-03 pinning experiment

The measurement that motivates the cycle (measured 2026-08-03, the session's terminal stamps
read 26-08-04 UTC):

- the measurement: `min-now --blocks 200`, 8 pinned + 8 unpinned per policy, alternating so box
  history is shared between conditions, under amd-pstate-epp `powersave` then `performance`
  - powersave unpinned: 23.927 ns, run-to-run stdev 0.785 ns (3.28%), grades 4 A / 2 D / 2 F
  - powersave pinned to one core: 22.207 ns, 0.073 ns (0.33%), all 8 graded D
  - performance unpinned: 21.980 ns, 0.057 ns (0.26%), no D or F in 17 runs
  - performance pinned: 22.359 ns, 0.311 ns (1.39%)
- what that supports, and how strongly:
  - the policy delta is big and one-directional: 8.9% on the unpinned mean, with unpinned
    run-to-run scatter falling from 0.785 ns to 0.057 ns
  - but policy could not be alternated the way pinning was (setting EPP needs root and is global
    and persistent), so the two policies are separated in time and the delta is confounded with
    box history. We think an effect this size survives de-confounding, but this experiment does
    not show it
  - pinning under powersave is the strongest result here: 7.2% off the mean and 10x off the
    scatter, on alternated runs
  - pinning under performance is the weakest: 1.7% *worse* than unpinned, and the whole gap is
    four consecutive pinned runs (p3-p6) at 22.59-22.70 ns while the pinned runs on either side
    sat at 22.08-22.09 ns and the interleaved unpinned runs did not move. So the 0.311 ns is the
    width of a state change, not scatter about a mean, and "pinning loses under performance"
    rests on that one cluster and should be treated as unreproduced
- the finding that reframes the grade: A was being awarded to the un-boosted floor. A powersave
  A run puts ~73% of its samples on 23.967 ns, the 3.7929 GHz TSC/base rate, while the boosted
  states are 21.79 / 22.22. Flatness is all the grade sees, and base clock is the flattest place
  on the box
- evidence is perishable: `tmp/pinexp/` held only the performance series (the rerun overwrote
  powersave), so the powersave numbers live only in the session transcript until copied into a
  chores design subsection
- cross-cuts the Todo entries below: the interpretation guide's worked trio and the blocks
  entry's duty-cycle evidence were both collected without recording the policy, and a four-run
  `--blocks` sweep proved confounded (block count and box history rose together, history
  dominating), so re-check those examples before they ship as teaching material
- raises "Seam-clock attribution": we think the three states map onto ~3.79 / 4.17 / 4.35 GHz,
  inferred from timing ratios alone, and a seam clock sample would settle it
  - superseded in part 2026-08-16: the settle rung's state column measured the ladder directly,
    and the dwell finding is
    [The 3900x dwells, it does not settle](#the-3900x-dwells-it-does-not-settle-2026-08-16)

### The 3900x dwells, it does not settle (2026-08-16)

Evidence from the settle rung's own state column on the day it landed, read across wink's runs
on both boxes, transcribed in full below the bullets since evidence in a transcript alone is
perishable. "Settle" assumes convergence to a final state, and under today's policy the 3900x
has none.

- the settle cell on a state-hopping box records when the *last disturbance during warmup*
  happened, not a ramp length: the cell is the earliest suffix flat through warmup's end, so
  one flip at 4.5s of a 6.5s warm reads `4.53s` even when the first 4.5s were glass-flat
- the transition into the ~4.5 GHz top state is stochastic, not a fixed delay: with
  `--settle-time 5` and a 5s gap, one run sat flat at 4.09 GHz through the full 5s of sustained
  warm load and climbed mid-bench anyway, while siblings climbed at 1.9s and 4.8s of theirs. An
  earlier read of "the climb takes ~2s of sustained load" did not survive this data
- state residency runs seconds with heavy tails: one run held ~10s across warm and bench and
  graded A while its neighbors flipped mid-window and took D/F. The box dwells in states, so no
  warmup budget can wait out a coin that keeps flipping, and a longer `--settle-time` only
  moves the odds
- the state ladder is now measured rather than inferred: 4.09 GHz entry and ~4.50-4.54 top on
  the 3900x (`min-now` means ~24.2 ns and ~21.8-21.9 ns respectively), where the seam-clock
  entry had guessed ~3.79 / 4.17 / 4.35 from timing ratios
- the 7600x is the control that makes "dwell" a property of the box rather than the tool: after
  a 5s gap it ramps 0.81-0.82s to 5.44 GHz every run without exception at 16.2 ns, 5A and
  QUALIFIED, and run back-to-back it inherits the warm state and settles at 0.01s at the same
  5.44 GHz. That box genuinely settles: a convergent, deterministic ramp to a state it holds
- the consequences are ones the block already carries: the base-clock pin removes the coin
  entirely, the record rung's seam-sampled clock makes mid-bench flips visible, and the
  qualify verdicts stand: the D/F grades were correct all along, and the settle column now
  explains them instead of contradicting them

The runs the bullets read from, verbatim. The 3900x pair (wink's terminal, GUI desktop, this
bot's session also live on the box):

```
wink@3900x 26-08-16T04:44:39.339Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility+1)
$ iiac-perf qualify-environment --settle-time 5
iiac-perf 0.26.0-1 — Rust latency microbenchmark harness

qualify-environment: 10 runs of `min-now -d 1`, gap 0s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle           mean
  1     A       F        F       1.95s @4.52GHz   22.7 ns
  2     B       A        B       not              21.7 ns
  3     A       D        D       0.01s @4.52GHz   22.1 ns
  4     A       B        B       0.01s @4.52GHz   21.8 ns
  5     A       B        B       0.03s @4.52GHz   21.8 ns
  6     A       B        B       4.53s @4.52GHz   21.7 ns
  7     A       A        A       1.55s @4.09GHz   24.2 ns
  8     B       B        B       not              21.8 ns
  9     A       A        A       0.01s @4.52GHz   21.9 ns
  10    A       B        B       0.09s @4.51GHz   21.7 ns

  environment grades: 2A 6B 1D 1F
  median environment grade: B
  median settle: 0.09s (2 of 10 never settled)
  transition-degraded (drift or step at D/F): 2 of 10

  verdict: NOT QUALIFIED
    a state transition landed inside a measurement window
wink@3900x 26-08-16T04:46:21.929Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility+1)
$ iiac-perf qualify-environment --settle-time 5 -d 5 --runs 5 --gap 5
iiac-perf 0.26.0-1 — Rust latency microbenchmark harness

qualify-environment: 5 runs of `min-now -d 5`, gap 5s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle           mean
  1     A       F        F       0.01s @4.09GHz   23.0 ns
  2     A       F        F       4.79s @4.50GHz   23.7 ns
  3     A       B        B       1.94s @4.51GHz   21.9 ns
  4     A       A        A       4.86s @4.49GHz   21.8 ns
  5     A       B        B       1.88s @4.51GHz   21.8 ns

  environment grades: 1A 2B 2F
  median environment grade: B
  median settle: 1.94s (0 of 5 never settled)
  transition-degraded (drift or step at D/F): 2 of 5

  verdict: NOT QUALIFIED
    a state transition landed inside a measurement window
```

The 7600x pair, minutes later on the headless box (ssh, no GUI, no bot):

```
wink@7600x 26-08-16T04:52:02.804Z:~
$ ./iiac-perf qualify-environment --settle-time 5
iiac-perf 0.26.0-1 — Rust latency microbenchmark harness

qualify-environment: 10 runs of `min-now -d 1`, gap 0s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle           mean
  1     A       A        A       0.80s @5.44GHz   16.2 ns
  2     A       A        A       0.01s @5.44GHz   16.2 ns
  3     A       A        A       0.01s @5.44GHz   16.2 ns
  4     A       A        A       0.01s @5.44GHz   16.2 ns
  5     A       A        A       0.01s @5.44GHz   16.2 ns
  6     A       A        A       0.01s @5.44GHz   16.2 ns
  7     A       A        A       0.01s @5.44GHz   16.2 ns
  8     A       A        A       0.01s @5.44GHz   16.2 ns
  9     A       A        A       0.01s @5.44GHz   16.2 ns
  10    A       A        A       0.01s @5.44GHz   16.2 ns

  environment grades: 10A
  median environment grade: A
  median settle: 0.01s (0 of 10 never settled)
  transition-degraded (drift or step at D/F): 0 of 10

  verdict: QUALIFIED
wink@7600x 26-08-16T04:53:06.854Z:~
$ ./iiac-perf qualify-environment --settle-time 5 -d 5 --runs 5 --gap 5
iiac-perf 0.26.0-1 — Rust latency microbenchmark harness

qualify-environment: 5 runs of `min-now -d 5`, gap 5s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle           mean
  1     A       A        A       0.82s @5.44GHz   16.2 ns
  2     A       A        A       0.81s @5.44GHz   16.2 ns
  3     A       A        A       0.82s @5.44GHz   16.2 ns
  4     A       A        A       0.82s @5.44GHz   16.2 ns
  5     A       A        A       0.82s @5.44GHz   16.2 ns

  environment grades: 5A
  median environment grade: A
  median settle: 0.82s (0 of 5 never settled)
  transition-degraded (drift or step at D/F): 0 of 5

  verdict: QUALIFIED
```

### The clock quantum and the dither

The clock's quantum is a property of the box, not the bench (measured on rpi5-20cd, 2026-08-04).
`inner` is sized for framing domination, `inner = ceil(10 * frame_ns / step_cost_ns)`, so in
`q = tick_ns / inner` the step cost cancels and `q / step = tick_ns / (10 * frame_ns)`.

- ~2.2% on the Pi (18.5185 ns tick, ~82 ns frame) against ~0.05% on the 3900X (0.264 ns tick,
  ~50 ns frame): a factor of 40, applying uniformly to every bench either box runs
- three benches confirm it, predicted `q` against the spacing between value clusters in the band
  table: `min-now` inner 23, q 0.805, seen 0.800 / 0.832 / 0.768. `zcr-with-1t` inner 47,
  q 0.394, seen 0.400 / 0.384 / 0.408. `zcr-with-2t` inner 4, q 4.630, seen 4.352 / 4.608 /
  4.864. The residual wobble is hdrhistogram bucketing (0.016 ns at 17 ns, 0.256 ns at 270 ns)
- so a printed per-sample spread below ~q describes the clock, not the workload: `min-now`'s
  `stdev p20..p80 0.403` is half a quantum and `zcr-with-1t`'s 0.213 is a two-point split, while
  `zcr-with-2t` spans ~11 quanta and its 4.906 is real to within 4% (`q/sqrt(12)` removed in
  quadrature)
- the means, the grades and LSC are unaffected: all work from batch or block means over ~1.2M
  samples, where quantization averages away as `1/sqrt(N)`
- so print and record it, do not size for it: `q` joined the `Setup:` block in the ported work,
  and `ticks_per_ns` and `inner` are record fields so it stays derivable per result
- a granularity floor on `inner` is the wrong fix and is its own Todo entry, not a rung here.
  The dither makes quantization zero-mean, so a 5M-sample mean carries `q/sqrt(12N)` = 0.0001 ns
  of it, while raising `inner` would smear the tail linearly and cannot reveal per-call shape
  below one tick either way
- the dither works on the coarse lattice, twice confirmed on the Pi: `zcr-with-1t`'s mass sits
  on two adjacent lattice points (2.08M at ~16.951, 2.94M at ~17.335) and interpolates to 17.18
  against a printed `mean z4..n2 17.183`, with the two-point stdev prediction 0.19 against a
  printed 0.213
  - so an off-lattice value is recovered rather than snapped, which is what `DITHER_SPAN` exists
    to do. The earlier worry that ~26-32 ns of span against an 18.5 ns quantum would bias it is
    not visible in the data
  - the decisive check remains cheap and unrun: sweep `-i` (23 / 97 / 233) on the Pi and confirm
    the mean does not move as `q` shrinks

### Port measure reproducibility

I worked with[claude-web](https://claude.ai/share/d8aaa0dd-026a-4973-9906-2b83508ef89b) and
ported the branch measure-reproducibility that claude-code and I worked on 2026-08-04 and
2026-08-03. After that we got side tracked working on agent-files with vc-x1 and it's time to
revive it.

Initially I tried just rebase it:
```
wink@3900x 26-08-15T14:46:49.277Z:~/data/prgs/rust/iiac-perf (main+1)
$ jj rebase -b measure-reproducibility -d @-
Rebased 4 commits to destination
New conflicts appeared in 4 commits:
  sqwrtoyv 2ea22f58 (conflict) (no description set)
  quxttlto 75d0cedf measure-reproducibility* | (conflict) refactor: extract the report renderer
  pvpszqlm e3f3d2d2 (conflict) feat: report the power policy and clock quantum
  zkxrnrrn f282ecea (conflict) feat: measure reproducibility opening
Hint: To resolve the conflicts, start by creating a commit on top of
the first conflicted commit:
  jj new zkxrnrrn
Then use `jj resolve`, or edit the conflict markers in the file directly.
Once the conflicts are resolved, you can inspect the result with `jj diff`.
Then run `jj squash` to move the resolution into the conflicted commit.
wink@3900x 26-08-15T14:51:39.958Z:~/data/prgs/rust/iiac-perf (main+1)
```

But here were too many conflicts so I restored it:
```
wink@3900x 26-08-15T14:51:39.958Z:~/data/prgs/rust/iiac-perf (main+1)
$ jj op restore 75ddc1
Restored to operation: 75ddc14fde07 (2026-08-14 17:30:39) push bookmarks docs-converge-the-agent-files-with-vc-x1, main to git remote origin
Working copy  (@) now at: qpnxsmzl dec5e8bc (empty) (no description set)
Parent commit (@-)      : pnowzzty 0520c17c main | docs: converge the agent-files with vc-x1
Added 0 files, modified 4 files, removed 0 files
Existing conflicts were resolved or abandoned from 5 commits.
wink@3900x 26-08-15T15:09:52.393Z:~/data/prgs/rust/iiac-perf (main+1)
$ jj st
The working copy has no changes.
Working copy  (@) : qpnxsmzl dec5e8bc (empty) (no description set)
Parent commit (@-): pnowzzty 0520c17c main | docs: converge the agent-files with vc-x1
wink@3900x 26-08-15T15:10:37.751Z:~/data/prgs/rust/iiac-perf (main+1)
```

After talking with claude-web the techquite that worked was to just copy over the *.rs files which
we haven't touched so just coping those created something that passed `cargo check`
```
wink@3900x 26-08-15T16:16:56.298Z:~/data/prgs/rust/iiac-perf (main+1)
$ jj bookmark create port-measure-reproducibility -r @
Warning: Target revision is empty.
Created 1 bookmarks pointing to qpnxsmzl dec5e8bc port-measure-reproducibility | (empty) (no description set)
wink@3900x 26-08-15T16:22:21.933Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ jj st
The working copy has no changes.
Working copy  (@) : qpnxsmzl dec5e8bc port-measure-reproducibility | (empty) (no description set)
Parent commit (@-): pnowzzty 0520c17c main | docs: converge the agent-files with vc-x1
wink@3900x 26-08-15T16:22:25.717Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ jj restore --from measure-reproducibility 'glob:**/*rs'
Working copy  (@) now at: qpnxsmzl 870001c9 port-measure-reproducibility | (no description set)
Parent commit (@-)      : pnowzzty 0520c17c main | docs: converge the agent-files with vc-x1
Added 1 files, modified 23 files, removed 0 files
wink@3900x 26-08-15T16:22:52.711Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ jj st
Working copy changes:
M src/band_table.rs
M src/bands.rs
M src/benches/ice_ps_1t.rs
M src/benches/ice_ps_2t.rs
M src/benches/ice_rr_1t.rs
M src/benches/ice_rr_2t.rs
M src/benches/min_now.rs
M src/benches/mpsc_1t.rs
M src/benches/mpsc_2t.rs
M src/benches/mpsc_2t_spin.rs
M src/benches/probe_mpsc_2t.rs
M src/benches/std_now.rs
M src/benches/tp2_pc.rs
M src/benches/zcr_mpsc_1t.rs
M src/benches/zcr_mpsc_2t.rs
M src/benches/zcr_with_1t.rs
M src/benches/zcr_with_2t.rs
M src/freq.rs
M src/gauge.rs
M src/harness.rs
M src/main.rs
M src/probe.rs
A src/report.rs
M tests/qualify_environment.rs
Working copy  (@) : qpnxsmzl 870001c9 port-measure-reproducibility | (no description set)
Parent commit (@-): pnowzzty 0520c17c main | docs: converge the agent-files with vc-x1
wink@3900x 26-08-15T16:23:16.995Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ cargo check
    Checking iiac-perf v0.25.2 (/home/wink/data/prgs/rust/iiac-perf)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.15s
wink@3900x 26-08-15T16:23:34.167Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
```

claude-web recommened running `cargo check --tests` and then `cargo test` and they both worked,
although that `tests/qualify_environment.rs` to 26s and I thought it'd died, but I was patent
and it did complete:
```
wink@3900x 26-08-15T16:23:34.167Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ cargo check --tests
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s
wink@3900x 26-08-15T16:54:07.361Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ cargo test
   Compiling iiac-perf v0.25.2 (/home/wink/data/prgs/rust/iiac-perf)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.20s
     Running unittests src/main.rs (target/debug/deps/iiac_perf-4f8dfa43303d64f4)

running 79 tests
test bands::tests::generated_boundaries_match_documented_ladder ... ok
test config::tests::empty_is_all_none ... ok
test config::tests::bad_band_labels_errs ... ok
..
test harness::tests::batch_pipeline_flushes_full_batches ... ok
test harness::tests::settle_time_finds_the_ramp_end ... ok

test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/qualify_environment.rs (target/debug/deps/qualify_environment-39de42f698c35b50)

running 1 test

test environment_qualifies ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.37s

wink@3900x 26-08-15T16:54:39.693Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
```

I then ran qualify-environment:
```
wink@3900x 26-08-15T17:01:08.097Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ iiac-perf qualify-environment
iiac-perf 0.25.4 — Rust latency microbenchmark harness

qualify-environment: 10 runs of `min-now -d 1`, gap 0s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle   mean
  1     A       F        F       0.01s    22.2 ns
  2     A       F        F       1.38s    22.7 ns
  3     A       B        B       not      21.6 ns
  4     A       A        A       0.95s    21.5 ns
  5     A       A        A       0.01s    24.5 ns
  6     A       A        A       1.26s    21.6 ns
  7     A       C        C       1.97s    22.1 ns
  8     A       C        C       1.62s    24.4 ns
  9     A       A        A       not      21.6 ns
  10    A       F        F       1.37s    24.5 ns

  environment grades: 4A 1B 2C 3F
  median environment grade: B
  median settle: 1.37s (2 of 10 never settled)
  transition-degraded (drift or step at D/F): 3 of 10

  verdict: NOT QUALIFIED
    a state transition landed inside a measurement window
wink@3900x 26-08-15T17:01:49.854Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
```

Look at the original [measure-reproducibility](https://github.com/winksaville/iiac-perf/tree/measure-reproducibility)
branch for more background information, we'll start a new ladder after this commit and I'll have
claude-code create a new ladder based on the original ladder.

Here are some runs of qualify-environment on 7600x a newer Ryzen 5
and 3900x an older Ryzen 9. These are 5 second duration with a 5 seconds
between runs so we measure "startup" ramping each time.

You can see 7600x is has perfect scores each time and a consistent
0.81s settling time:
```
wink@7600x 26-08-15T17:33:08.762Z:~
$ rg -m 1 'model name' /proc/cpuinfo 
5:model name	: AMD Ryzen 5 7600X 6-Core Processor
wink@7600x 26-08-15T17:33:17.592Z:~
$ ./iiac-perf qualify-environment -d 5 --runs 5 --gap 5
iiac-perf 0.25.4 — Rust latency microbenchmark harness

qualify-environment: 5 runs of `min-now -d 5`, gap 5s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle   mean
  1     A       A        A       0.81s    16.2 ns
  2     A       A        A       0.81s    16.2 ns
  3     A       A        A       0.81s    16.2 ns
  4     A       A        A       0.81s    16.2 ns
  5     A       A        A       0.81s    16.2 ns

  environment grades: 5A
  median environment grade: A
  median settle: 0.81s (0 of 5 never settled)
  transition-degraded (drift or step at D/F): 0 of 5

  verdict: QUALIFIED
wink@7600x 26-08-15T17:35:00.407Z:~
```

Where as the code thinks the 3900x settles in 10ms, **which is probably does not**, 
and has terribe numbers:
```
wink@3900x 26-08-15T17:41:28.871Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ rg -m 1 'model name' /proc/cpuinfo
5:model name	: AMD Ryzen 9 3900X 12-Core Processor
wink@3900x 26-08-15T17:41:50.432Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
$ iiac-perf qualify-environment -d 5 --runs 5 --gap 5
iiac-perf 0.25.4 — Rust latency microbenchmark harness

qualify-environment: 5 runs of `min-now -d 5`, gap 5s
  the box is the subject: grades are the environment's, not the run's

  run   warmup  bench    worst   settle   mean
  1     A       F        F       0.01s    21.7 ns
  2     A       F        F       0.01s    21.8 ns
  3     A       F        F       0.01s    21.7 ns
  4     A       F        F       0.01s    22.7 ns
  5     A       D        D       0.01s    21.7 ns

  environment grades: 1D 4F
  median environment grade: F
  median settle: 0.01s (0 of 5 never settled)
  transition-degraded (drift or step at D/F): 5 of 5

  verdict: NOT QUALIFIED
    median grade below B — runs are measuring a moving box
    a state transition landed inside a measurement window
wink@3900x 26-08-15T17:43:10.276Z:~/data/prgs/rust/iiac-perf (port-measure-reproducibility)
```

Although both are running the same arch linux version and only the terminal
open, no other apps (no bot either). But the 3900x has a GUI running, thus mouse and a display
running. Where as 7900x is just a server no mouse or display attached and no gui is installed
so just running via an SSH connection over the LAN. So we don't know exactly why but 7600x is a
much better machine for getting consistent numbers, at this point in time.

## fix: left-align the summary rows

- [[39]] 0.26.1 fix: left-align the summary rows

A single-commit cycle, run between the measure-reproducibility landing and its campaign
follow-on because wink judged the effort minimal and was right ("I say it depends on the
effort").

### Problem

The summary rows (mean through LSC) printed under the band table's mean column, ~80 columns
from their labels: wink's 2026-08-02 complaint, sharpened 2026-08-20 into a sketch during
live 7600x runs. The same runs printed `resolution 0.00` on a very quiet box, the exact
fiction-shaped zero the resolution rung was built against, because "at least 2 decimals" was
not enough for a floor below 5 ps.

### Solution

The summary rows sit in a left-aligned block fenced by blank lines, labels beside a
decimal-aligned value column, per wink's sketch. Claims (resolution, CI95, LSC) extend
precision until the leading digit shows, capped at 3 decimals (the recording floor), then
print `<0.001`, so a claim never prints as a bare zero and the dash stays reserved for "no
claim exists": two different statements, now visibly different.

- `fmt_claim` is the one home for the rule, unit-tested, and the withheld dash prints bare,
  no unit and no trailing padding
- qualify's mean parse tokenizes by whitespace, so it survived unchanged, and the report
  guide's examples and its dash-versus-small explanation follow the new shape
- delivered from the "Blocks as the first-class mode" Todo entry, whose display-gate and
  default-flip halves remain there

## docs: a session's rules are its own agent-files

- [[40]] 0.26.2 docs: a session's rules are its own agent-files

A single-commit cycle, interposed on `main` while `docs: adopt the family agent-files set` was
in flight, so the rule reaches every future session without waiting on that cycle's external
dependencies. The in-flight cycle's bookmark rebases onto this commit, and its copy rung must
carry the rule forward when the family set overwrites `AGENTS.md`.

### Problem

Hard rule 7 said to read the checklists before commit work and before any push, and named no
repos. A session took that opening (2026-08-30, the messages repo): it decided the rule covered
only the cycle's own repos, substituted the sibling repo's write protocol for the checklists,
and committed and pushed there with no checklist read, no work review, and no description
review. The clause that would have caught it, "cycle or not", sat inside the checklist file the
session had already decided not to open.

### Solution

Hard rule 7 now states the session's rule identity outright, where every session auto-loads it:
a session's rules are the agent-files of the project it started in, rules living in any other
repo are ignored unless those files or the user direct otherwise, and the checklist read covers
commit work and pushes in any repo, cycle or not.

- the messages README still governs writes to that repo, because `custom-family.md` delegates
  to it, and the delegation hands over the repo's protocol, never the session's review stops
- a first wording claimed the whole checklists applied in every repo a session writes, which
  contradicted the Messaging section's "the README governs" and would have lost that conflict
  on precedence, `custom.md`'s layer loading last. Stating rule identity plus delegation
  removes the conflict rather than adjudicating it. The dual-repo thought experiment that
  settled it (wink): were the messages repo a full dual-repo project, the answer is to message
  its agent rather than write its repo, so a foreign repo's rules never bind by default
- the wording change is confined to rule 7, and the checklist files are untouched
- the change is this member's open proposal to the family, carried as the diff against the
  payload, per [Changing the agent-files](../../AGENTS.md#changing-the-agent-files)
- found at this cycle's push, worth the family knowing: `vc-x1 push` (0.80.7) refuses a config
  still spelling the agent side `bot`, while `vc-x1 validate` only asks for its missing
  `validate.full` table. So a member adopting the new agent-files cannot push anything until
  its config at least renames `repos.bot` to `repos.agent`, and the full carrier conversion
  wants to ride the adoption's first commit, as this repo's did. This branch, cut from `main`
  before that conversion, carries the minimal rename so the cycle could push at all

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
[12]: #feat-measure-reproducibility-opening
[13]: #fix-the-settle-cell-reads-the-clock
[14]: #feat-write-a-per-run-json-record
[15]: #feat-adopt-the-markdown-config-carrier
[16]: #feat-read-pin-and-restore-the-cpu-frequency
[17]: #fix-the-settle-cell-shows-the-clocks-journey
[18]: #feat-block-sleep-and-warmup-become-knobs
[19]: #fix-lsc-gains-a-run-to-run-component
[20]: #fix-the-pin-flag-names-cpus
[21]: #feat-suggest-freq-measures-the-pin-frequency
[22]: #docs-split-the-readme-into-a-docs-directory
[23]: #docs-the-report-reading-guide
[24]: #feat-measure-reproducibility-closing
[25]: #port-measure-reproducibility
[26]: https://github.com/winksaville/iiac-perf/commit/c1945bec7501 "c1945bec7501c30e4fdf55b63f83a5f661941310"
[27]: https://github.com/winksaville/iiac-perf/commit/8e476bee3538 "8e476bee35389fbd2d7b3cf027fbe36bc973512d"
[28]: https://github.com/winksaville/iiac-perf/commit/610ce16c895a "610ce16c895acf98e2025c366658c0cfad6011a7"
[29]: https://github.com/winksaville/iiac-perf/commit/47eef519165a "47eef519165a3865e020794f36f97ce8fa9e796a"
[30]: https://github.com/winksaville/iiac-perf/commit/194d65122404 "194d65122404923652df65a58acfc928e140ef04"
[31]: https://github.com/winksaville/iiac-perf/commit/7f2c125ab2a4 "7f2c125ab2a49b82909f66623ed3c7d2ad779a39"
[32]: https://github.com/winksaville/iiac-perf/commit/c1aef4707202 "c1aef4707202cb42a466eda411cf7e1109cd2018"
[33]: https://github.com/winksaville/iiac-perf/commit/d0ea16dd5f33 "d0ea16dd5f338310a9eb08eb78f5dd7ad0d2013e"
[34]: https://github.com/winksaville/iiac-perf/commit/3dc6ed653c63 "3dc6ed653c63a3e1148d7200189008633e6d4564"
[35]: https://github.com/winksaville/iiac-perf/commit/22554115f7b7 "22554115f7b76d0f9b10c3f09cfe3ea12c44b08d"
[36]: https://github.com/winksaville/iiac-perf/commit/0b3977889c62 "0b3977889c62cb847db895a8565f37afc1cc4aac"
[37]: https://github.com/winksaville/iiac-perf/commit/73c1063ca1fd "73c1063ca1fdd0d0967c3cedfe41ffa76ae30799"
[38]: https://github.com/winksaville/iiac-perf/commit/7315d94efb2b "7315d94efb2bca011ff2c45affa2b221b15881cd"
[39]: https://github.com/winksaville/iiac-perf/commit/980a6e32fe95 "980a6e32fe9530c76062f8627537493cda50b35b"
[40]: https://github.com/winksaville/iiac-perf/commit/8b702dfe730f "8b702dfe730fade25736ef201e1fd360342791ea"
