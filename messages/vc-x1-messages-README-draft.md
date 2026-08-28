# vc-x1-messages v0.1.0

Notifications for the vc-x1 family. A message body lives in the sender's own repo. This repo
holds only pointers to those bodies, one record per message.

This file is the whole protocol and all of the session behavior. A project that takes part needs
one pointer at this file and no local copy of any of it. Instructions come first, worked
examples are at the end in [Examples](#examples).

The version above is the record format's. Changes to it are additive: a field is added, never
renamed or repurposed, so an old reader still reads today's records. The session behavior is not
versioned.

## Your file

Each participant receives on one file here, `<member>.md`. Yours is the one named for your
project.

Projects using the vc-x1 tooling get this repo's path and their own member name from the
`[family]` table of their work-side config, `family.messages` and `family.member`. Other
projects just need those two facts. A project that does not take part needs nothing here.

## At the start of a session

Open your file and read it.

- No `read:` field means nobody has read it yet. Reading it is what adds the field.
- No `outcome-*` field means it is still open, however old it is.

Add `read:` when you read a record. That is how the sender learns it arrived.

Each file's head declares what happens to a handled record. That policy is the file owner's and
it governs.

## Sending a message

1. Write the message in a file in your own repo, under a markdown section heading.
2. Add a record to the destination's file here, directly below its header, so the newest record
   is first.

Whether you push between those two steps is [the choice of mode](#fast-or-durable).

## The record format

A record is a `##` heading followed by a list of fields.

The heading is `<utc-timestamp> <sender>`. It is also the record's anchor, so anyone can link
straight at this record without inventing an id.

The fields are `- name: value` lines. All are optional, and most records carry two or three.
Write at least one of `local` or `remote`, or the record points at nothing.

- `local:` where the message is in the sender's working tree.
- `remote:` a permalink to the message. See [Remote references](#remote-references).
- `read:` a UTC timestamp, added by the recipient.
- `outcome-local:` and `outcome-remote:` added by the recipient, pointing at what came of the
  message. They are what close a record.

A reference can be an inline link or a markdown reference link.

Unknown fields are ignored, so new ones can be added later without breaking old records. Prose
can sit beside the fields. Only the field lines are read.

A malformed record is not an error, just less useful. Take what is there. The `##` heading is
the only part that must be present, because it is what separates one record from the next.

## Handling a request

Turn the request into an entry in your own records before you act on it, and have your reply
cite that entry. The entry outlives the exchange, so what became of the request stays readable
from your repo after this repo has moved on.

Reply with a record in the sender's file. If the reply cites work you have committed, use the
durable mode, because a permalink needs the commit to exist first.

An outcome cites the record of what came of the request. If that record sits in a section a
later commit deletes, a relative link cannot reach it, so use `outcome-remote:`.

## Fast or durable

The message file lives in your repo either way. The difference is whether the record carries a
permalink.

- **Fast.** Write the message file, add the record with `local` alone. No push, no waiting. The
  reader resolves it from their sibling clone.
- **Durable.** Write the message file, commit and push it, then add the record with the `remote`
  that push made resolvable.

**In durable mode the order matters, and this is the step people miss.** A permalink names a
commit, so the commit has to exist and be pushed first. Write the record first and the URL 404s
until the file catches up.

**Write a record once**, when you have everything that goes in it. An early record with an empty
`remote` costs two writes for one message, and a reader may follow it, find nothing, and mark it
read.

Same-day traffic between siblings is fine on `local` alone. Anything worth citing later wants
the push.

## Committing

Whoever writes a record commits it. Committing in the same act is ideal. A short delay, or
batching a few writes into one commit, is fine among friendly participants.

What does not relax: nobody commits someone else's writing. That is what this repo has instead
of a manager.

New records all go directly below the file header, so two writers at once collide on the same
lines. That is normal. Keep both records, in either order. No record depends on its neighbours.

## Remote references

The form is `https://github.com/<owner>/<repo>/blob/<commit-sha>/<path>#<slug>`. The commit SHA
is not optional.

A branch name rots. A topic bookmark is deleted once its work lands, and the permanent branch
does not carry the file until then, so a branch URL breaks exactly when the message becomes
worth reading. A SHA survives all of that and resolves for a reader with no clone. GitHub's `y`
key converts a branch URL into this form.

A local reference names a path, so it resolves to whatever that file says now, and it assumes
the member repos are siblings. A remote reference names a version, so it resolves to what was
actually sent.

## Persistence

A member's file belongs to whoever receives on it. They create it, they curate it, and they
declare at its head what happens to a handled record. A sender needs to know none of that, only
where to add a record.

Marking beats deleting. Whether a message was read is recorded nowhere else, so marking a
handled record tells the sender it arrived and deleting it tells them nothing. This is a
recommendation, not a rule.

Nothing worth keeping is ever only here. The body is a committed file in the sender's repo. A
lost record loses nothing, which is what makes owner-chosen policies safe.

A file that does not exist yet is created by whoever writes first, and it carries no policy
until its owner declares one. Otherwise the first message to a new member would be impossible,
since only the owner may create and only the sender wants to.

## No protection

Any member can modify or delete any file here. This is a cooperative store with no access
control. It works among friendly participants, and history is the only recourse.

## Examples

Staged records, not live ones, fenced so an example is never mistaken for a real record. The
references in them do resolve.

### A new record, durable mode

As it would sit in `vc-x1.md`, sent by iiac-perf.

```
## 2026-08-13T19:31:21.123Z iiac-perf

- local: [../iiac-perf/messages/test-msg.md#message1](../iiac-perf/messages/test-msg.md#message1)
- remote: https://github.com/winksaville/iiac-perf/blob/55554b452957/messages/test-msg.md#message1
```

### A handled record

The same exchange the other way, as it would sit in `iiac-perf.md`, after the recipient read it
and recorded what came of it.

```
## 2026-08-13T20:41:33.512Z vc-x1

- local: [../vc-x1/notes/messages/test-msg.md#message1](../vc-x1/notes/messages/test-msg.md#message1)
- remote: https://github.com/winksaville/vc-x1/blob/437a1e6b93d2/notes/messages/test-msg.md#message1
- read: 2026-08-13T22:04:07.000Z
- outcome-local: [../iiac-perf/notes/chores/chores-07.md#docs-design-the-vc-x1-messages-repo](../iiac-perf/notes/chores/chores-07.md#docs-design-the-vc-x1-messages-repo)
- outcome-remote: https://github.com/winksaville/iiac-perf/blob/55554b452957/notes/chores/chores-07.md#docs-design-the-vc-x1-messages-repo
```

Two things to notice. The body is vc-x1's, under `notes/messages/`, where iiac-perf's sits under
`messages/`, because a body lives wherever its sender chooses. And the `remote:` was written only
after the body's commit pushed, which is the ordering rule above, obeyed in the example itself.
