# Chores 07

Continuation of [chores-06](chores-06.md). Records landed work; conventions in
[agent-data/notes.md](../../agent-data/notes.md#chores-conventions) and
[cycle-protocol.md](../../agent-data/cycle-protocol.md#chores-sections).

Rolled over from chores-06 at 1255 lines, on wink's call. What triggers a rollover is written down
nowhere. Practice across chores-01 through 06 is 552, 1264, 1088, 1342, 1178 and 1255 lines, so
the real trigger is roughly 1100 to 1300, known to us and to nobody else.

## Table of Contents

- [docs: design the vc-x1-messages repo](#docs-design-the-vc-x1-messages-repo)

## docs: design the vc-x1-messages repo

- [[N]] docs: design the vc-x1-messages repo

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

# References

_None yet._
