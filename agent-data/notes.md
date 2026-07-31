# Notes file conventions

Conventions the bot follows when reading and writing notes files (`TODO.md`,
`notes/todo-backlog.md`, `notes/bugs.md`, `notes/chores/chores-NN.md`, `notes/done.md`). One
source of truth lives here; [`notes/README.md`](../notes/README.md) points back. Read this
before editing a notes file.

Universal file, pinned to the template repository; do not edit here. Project overrides go in
[custom.md](../custom.md).

## File reads: read the slice you need

Long notes files are appended to over time. Read only the slice your task needs; grep or read
further on demand.

- **`TODO.md`** (the routine acquaint read): the first ~60 lines covers intro + `## In
  Progress` + the top of the ranked `## Todo` (priorities, #1 highest). `Read` with
  `offset=0, limit=60`. `## Ideas` sits below `## Todo`; read further only when chasing a
  lower-ranked entry, an Idea, a `[N]` ref, or auditing the whole list.
- **`notes/todo-backlog.md`**: the long-tail backlog (lower-priority entries below the ranked
  `## Todo`). Read only when picking up a backlog item; grep to locate it first.
- **`notes/bugs.md`**: the bug list. Small; read whole when triaging a bug or chasing the
  `## Bugs` pointer in TODO.md.
- **`notes/done.md`** + **`notes/chores/chores-NN.md`**: historical / append-mostly. Scan
  headings first (`grep '^## ' notes/chores/chores-NN.md`), then read only the section you
  need.

**Why:** the routine read should stay small. `TODO.md` grows every cycle, so the backlog and
bugs live in files under `notes/` rather than inline; the same "slice you need" rule applies
to historical files.

## Notes references

Reference *citations* are double-bracketed so the brackets render: `[[N]]`, or `[[2]],[[3]]`
for several (comma-separated, not `[2,3]` or `[[2]][[3]]`). The `[N]:` definitions in a file's
`# References` section and inline `[text](url)` / `[text](#anchor)` links stay
single-bracketed.

## Reference numbering

Every note file (`TODO.md`, `todo-backlog.md`, `bugs.md`, `chores-NN.md`, `done.md`) keeps a
file-local `# References` section at the bottom. Reference numbers are scoped to that file:
`[1]` in `chores-07.md` and `[1]` in `chores-01.md` are independent slots that may point at
completely different URLs. New chores files start their numbering at `[1]`.

Treat `[N]` like a **footnote**: the number is a local slot, only meaningful within its file's
`# References`. So a `[N]` *citation* (bare `[N]`, or doubled `[[N]]`) never reuses another
file's number; to cite a target a sibling file references, pick your own next-local slot and
define it (the same target may carry a different number in each file). A `[N]` *inside a code
span* (`` `[72]` ``) is different: that's a quoted identifier, literal text naming a ref-key
(often from another file's namespace), data, not a citation; it needs no definition here. To
point at a section of another file from prose, use an inline link with an anchor,
`[that section](../chores-07.md#...)`, not a bare number.

A `chores-NN.md` `# References` entry is usually a `/notes/<file>.md#anchor` (or
`/ARCHITECTURE.md`) path, but may also be a **commit reference**,
`[N]: <commit-url-with-12-hex-SHA> "<full-40-hex-SHA>"`, cited by a rung of a section's
as-built ladder. See [Chores commit references](#chores-commit-references) for the why and the
exact shape.

A file's `# References` can be **re-packed** to a contiguous `[1]..[N]` in
first-citation-appearance order: walk the file's prose in document order (`TODO.md`: `## Todo`
then `## Done`; `chores-NN.md`: top to bottom) and number refs as their first `[[N]]` citation
appears. This is a file-local rewrite, so only that file's `[[N]]` citations and `[N]:`
definitions move; every target and sibling file is untouched. A `[[N]]` inside a `` ` `` code
span is a literal token, not a citation, and is left alone. Do it opportunistically (when the
namespace has drifted enough to annoy), not on a schedule: `TODO.md` fragments fastest (entries
land and get pruned every cycle) and is the usual candidate; `chores-NN.md` / `done.md` are
append-mostly and only need it after an unusual event (e.g. a bulk retrofit that allocated
slots out of document order).

## Markdown anchor links

GitHub anchor algorithm: lowercase, strip non-alphanumeric characters in place, map remaining
spaces to hyphens 1-for-1. Do **not** collapse adjacent whitespace, so `a + b` -> `a--b`
(spaces on both sides of `+`), but `a: b` -> `a-b` (only trailing space on `:`). General
markdown reference: [markdownguide.org](https://www.markdownguide.org). GitHub publishes no
official spec for auto-generated anchors; the de-facto reference implementation is
[github-slugger](https://github.com/Flet/github-slugger).

## Todo format

`TODO.md` is organized into `## In Progress`, `## Todo` (strict priority rank, #1 highest;
long-tail backlog in [todo-backlog.md](../notes/todo-backlog.md)), `## Ideas`, `## Bugs`
(pointer to [bugs.md](../notes/bugs.md)), and `## Done` sections. Each item is a short
description with reference links to more detail.

`## Todo` and `## Bugs` entries carry explicit `1.` `2.` ... numbers in the source. For
`## Todo` the number is its **priority rank** (#1 highest, descending); for `## Bugs` it's just
an index. They're for grepping and at-a-glance "let's do #1", **not stable IDs**: reorder (to
reprioritize), insert, or delete freely, then `vc-x1 fix-todo --no-dry-run` renumbers and
normalizes continuation-line indent, so any given number is positional. **To refer to a Todo
durably, name it by its title, a plain, greppable text mention.** Not its number (positional,
renumbered) and not a markdown link: a numbered list item has no anchor to link to.

Numbering helps a human orient in a long list but makes links difficult and fragile, especially
an external reference pointing in, which can't be auto-fixed when the list renumbers. A robust
fix (a number-free anchor, or a number-tolerant dereference that matches the title slug and
wildcards the numeric prefix, since a GitHub slug like `5-foo` is encoded, not opaque, so the
title is recoverable) is a `validate-numbering` design question, out of scope here.

`vc-x1 fix-todo` alone only previews; `vc-x1 validate-todo` is the read-only check. The
`## Done` section keeps `-` bullets, since items aren't referenced by number once completed.

Example shape:

```
# Todo
1. Add new feature X [details](features.md#feature-x)
2. Fix bug Y [[1]]

# Done
- Fixed issue Z [[2]],[[3]]

[1]: bugs.md#bug-y
[2]: issues.md#issue-z
[3]: fixes.md#fix-z
```

## Retiring Done entries

`TODO.md`'s `## Done` section is a rolling buffer of recently shipped work, not a permanent
log. Move entries into `done.md` at two natural beats:

- **Closing a ladder**: when the final `X.Y.Z` (no suffix) commit ships, decide which prior
  entries are no longer needed for nearby context and migrate them.
- **Opening a new ladder**: at `X.Y.Z-0`, do the same sweep before bumping the
  version-of-record.

Migration mechanics:

- Move the bullet itself from `TODO.md > ## Done` to `done.md` (preserving the original ref
  number).
- Copy any references the moved entries cite into `done.md`'s `# References` section (those
  refs are file-local, so coexisting with `TODO.md`'s namespace is fine).
- Prune any references in `TODO.md > # References` no longer cited by anything in
  `## In Progress` / `## Todo` / `## Done`. This frees the numbers for future reuse.

## Chores conventions

### Headings and entries that record a commit

A commit's title is reused verbatim across its records; see
[Conventional-commit shape](prose.md#conventional-commit-shape-ladder--chores--commit) for the
rule. Beyond the chores `##` header, that same string is used for the matching
`TODO.md > ## Done` entry and any `[N]` reference to that section. Titles carry **no
`(<version>)` suffix**; see
[Commit description](../notes/cycle-protocol.md#commit-description) for why. E.g. the chores
header `## refactor: extract config loader` and the Done line
`- refactor: extract config loader [[3]]`. The `## Done` entry uses the close-out commit's
title.

This does **not** apply to organizational headings (`## Todo`, `## In Progress`,
`# References`) or to design `###` subsections inside a chores section; those are named for
whatever fits. Among the commit-recording ones, exact match is the strong default (nothing
absolute): a near-miss just makes it harder to line a record up with its commit.

A commit-recording header is provisional while the work is in progress; the *last* edit before
`vc-x1 push` syncs it, and the `## Done` entry / `[N]` anchor for that commit, to the final
commit title. See [Markdown anchor links](#markdown-anchor-links) for the slug algorithm; the
pre-commit checklist catches a dangling `#anchor`, and a future `vc-x1 validate-repo` should
too (and should verify the recorded title matches the commit).

Any pre-existing sections and `## Done` entries that predate this convention keep their
free-form text; the convention applies going forward.

### Chores section content: no edit list; git is the record

A chores section is: the as-built ladder (first content under the header; see below), then
[Prose form](prose.md#prose-form) (intro + bullets) for what landed and why, and any `###`
design subsections. Bullets here are **conceptual** (design points, structural notes), never a
per-file edit list. That lives in the commit message body, which is the source of truth for
"what changed mechanically" (immutable, `git show`-able, naturally scoped to the commit). The
chores section is the source of truth for the design thinking; the two cross-link, neither
restates the other.

The section is **built up per commit**: each work commit appends its own rung to the as-built
ladder + any narrative as it lands, rather than the narrative waiting for close-out (which only
*finalizes*: header title sync, design subsections). Full when-in-the-cycle timing lives in
cycle-protocol.md [Chores sections](../notes/cycle-protocol.md#chores-sections); this note is
the pointer, so the two don't drift.

When the intro starts wanting to explain a mechanism, hypothesis, or wrinkle, don't inflate it;
promote that to its own `###` subsection inside the same `chores-NN.md`. If the wrinkle is a
live design concern (something that *should* change, not just be recorded), also add a
`TODO.md` item with a `[N]` ref pointing at that subsection (todo->chores is the normal ref
direction).

**Why:** a chores edit list and the commit body were specified to be the same content in two
places, and detail written twice drifts. Git owns the mechanical record; chores owns the
narrative; the ladder's commit refs link them.

### Chores commit references

The first content under a chores section header is the **as-built ladder**: one rung per
commit the section records, in landing order, each rung carrying its own `[N]` citation slot.
The same form is used for every cycle, single- or multi-commit; a single-commit cycle is a
one-rung ladder whose rung is the bare `X.Y.Z` close-out:

```
## refactor: extract config loader

- [[2]] 0.42.0-1 refactor: split loader from parser
- [[N]] 0.42.0 refactor: extract config loader

<intro paragraph...>
```

- The rung form is `- [[N]] X.Y.Z[-n] <title>`, with no `(current)` / `(done)` markers:
  as-built implies done (the in-flight markers live in `TODO.md > ## In Progress`).
- A rung is written with the literal `[[N]]` placeholder and backfilled in place with a real
  file-local slot once its commit's SHA is permanent (see Timing below).
- Rung citations use the file-local `[N]` reference machinery (see
  [Reference numbering](#reference-numbering)), **double-bracketed** so the brackets render.
  (`[[3]]` shows as a literal `[`, the `[3]` link, then a literal `]`; the inner `[3]`
  resolves against its `[3]:` definition, and CommonMark / GitHub / VS Code all do this.) The
  `# References` definition puts the **commit URL** as the destination, with the **full
  40-hex SHA** in the title slot:

```
[3]: https://github.com/<owner>/<repo>/commit/<12-hex> "<40-hex>"
```

- The 12-hex short SHA in the URL keeps it short; GitHub / GitLab resolve a unique prefix to
  the canonical commit page (GitLab's path has a `/-/` before `commit/`).
- The full SHA in the title is host-agnostic and unambiguous: it survives a repo host change,
  `git show <40-hex>` works in any clone, and external tooling scraping the notes (a database,
  say) gets the canonical identifier.

**Timing.** A commit's SHA isn't stable until it lands on a **permanent branch** (`main`, or a
long-lived release/patch branch), because a rebase or squash rewrites it on the way. A commit
can't record its own SHA, so the fill lands one push later: every rung opens with the literal
`[[N]]` placeholder, and each push backfills the rungs of the commits the previous push made
permanent. On a topic branch a section waits until the branch lands. The commit itself is the
record, and `git log --grep "<title>"` finds it. See cycle-protocol
[Commits backfill](../notes/cycle-protocol.md#commits-backfill).

Sections that predate this convention keep their `Commits:` lines; the ladder form applies
going forward.

### Chores file Table of Contents

Each `chores-NN.md` carries a `## Table of Contents` section between the file intro and the
first commit-recording section: one `- [<title>](#<anchor>)` entry per commit-recording `##`
section, in file order.

- Entries are title-only, with no version and no `[N]` refs, so the TOC never needs backfill:
  the TOC navigates, the section's as-built ladder records.
- Append the entry when the section opens; the entry text participates in the same last-edit
  title sync as the section header and the `## Done` entry.
- Design `###` subsections stay out of the TOC.
- Files predating the convention gain a TOC opportunistically, not on a schedule.
