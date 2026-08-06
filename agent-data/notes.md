# Notes file conventions

Conventions the bot follows when reading and writing notes files (`TODO.md`,
`notes/todo-backlog.md`, `notes/bugs.md`, `notes/chores/chores-NN.md`, `notes/done.md`). One
source of truth lives here; [`notes/README.md`](../notes/README.md) points back. Read this
before editing a notes file.

Universal file, shared with the template repository; a proposed change is edited here and
converges at the template ([Changing the agent-files](../AGENTS.md#changing-the-agent-files)).
Project-local content goes in [custom.md](../custom.md).

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
- **Fixed issue Z** [[2]],[[3]]

[1]: bugs.md#bug-y
[2]: issues.md#issue-z
[3]: fixes.md#fix-z
```

## Done entry form

A `## Done` entry (in `TODO.md` and in `done.md`) is the close-out's **version**, then a **bold
title line** carrying its chores `[[N]]` ref, with any detail as sub-bullets:

```
- 0.42.0 **feat: config loader** [[7]]
  - `--config` resolves per-profile pin pools
  - the parse rejects unknown keys rather than ignoring them
```

- **Bold, and detail below rather than beside.** These entries are read by skimming a list for
  one of them, so the title has to be findable without reading the line it sits on. A title
  followed by a paragraph of summary is the wall-of-prose shape
  [Prose form](prose.md#prose-form) warns about, and it hides the one thing a skim is looking
  for. Bold matches the `## In Progress` block, whose title line is already bold.
- **The version leads and the title stays bold**, so the entry answers both questions a reader
  brings: `grep 0.42.0` finds what shipped in a version, and the eye still lands on titles when
  skimming. Putting the version *inside* the bold, or dropping the bold for a plain rung-shaped
  line, trades one of those for the other. See
  [Versions live in the version-of-record only](prose.md#versions-live-in-the-version-of-record-only)
  for why a Done entry may carry a version at all when a ladder rung may not.
- **A one-liner is still fine** when the title says it all; the sub-bullets are for when it does
  not. What is not fine is the middle case, a title with sentences trailing off it.
- **Sub-bullets are conceptual, like chores bullets**: what shipped, not a file list, and never
  a copy of the chores intro. The `[[N]]` ref is what carries a reader to the full narrative.
- **Entries written before this convention keep their form** and gain a version when touched; a
  sweep would rewrite history's presentation for no reader's benefit.

## Retiring Done entries

`TODO.md`'s `## Done` section is a rolling buffer of recently shipped work, not a permanent
log. Move entries into `done.md` at two natural beats:

- **Closing a ladder**: when the close-out commit ships, decide which prior entries are no
  longer needed for nearby context and migrate them. The entry form is the same in both files
  (see [Done entry form](#done-entry-form)), so a migration moves the bullet as it stands.
- **Opening a new ladder**: at the opening step, do the same sweep before bumping the
  version-of-record.
- **Resolving an agent-file experiment**: at the beat where it resolves, not at a ladder
  boundary. Adopted and rejected retire identically, since "we tried this and dropped it" is what
  history serves worst. The narrative goes in a `chores-NN.md` section and the dogfood entry is
  removed from wherever the project keeps its log, so that log carries in-flight entries only. See
  [Changing the agent-files](../AGENTS.md#changing-the-agent-files).

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
`TODO.md > ## Done` entry and any `[N]` reference to that section. **No title carries a version**;
the two records that carry one carry it beside the title, never inside it, and both are records
of a commit (the backfilled as-built rung and the `## Done` entry); see
[Versions live in the version-of-record only](prose.md#versions-live-in-the-version-of-record-only).
E.g. the chores header `## refactor: extract config loader` and the Done line
`- 0.42.0 **refactor: extract config loader** [[3]]`. The `## Done` entry uses the close-out
commit's title; its shape is in [Done entry form](#done-entry-form).

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
per-file edit list. Nothing in prose keeps one: the **diff** is the source of truth for what
changed mechanically (`git show --stat`, immutable, naturally scoped to the commit), the **commit
body** states the problem and the solution in broad terms, and the **chores section** is the
source of truth for the design thinking. Each of the three cross-links to the others; none
restates another.

The section is **not built up here**: it is created at close-out by moving the cycle's
`## In Progress` block, which was its single home while the cycle ran. So a rung is appended, and
narrative written, in `TODO.md` as each step lands, and close-out moves the finished block rather
than assembling a second copy of it. Full when-in-the-cycle timing lives in cycle-protocol.md
[Chores sections](../notes/cycle-protocol.md#chores-sections); this note is the pointer, so the
two don't drift.

**Why one home:** the alternative keeps a working ladder in `TODO.md` and an as-built ladder in
chores, so every rung is written twice and every backfill applied twice. Detail written twice
drifts, which is the same argument that keeps the edit list out of the commit body.

When the intro starts wanting to explain a mechanism, hypothesis, or wrinkle, don't inflate it;
promote that to its own `###` subsection inside the same `chores-NN.md`. If the wrinkle is a
live design concern (something that *should* change, not just be recorded), also add a
`TODO.md` item with a `[N]` ref pointing at that subsection (todo->chores is the normal ref
direction).

**Why:** a chores edit list and the commit body were specified to be the same content in two
places, and detail written twice drifts. Git owns the mechanical record; the body owns the problem
and the solution; chores owns the narrative; the ladder's commit refs link them.

### Chores commit references

The first content under a chores section header is the **as-built ladder**: one rung per
commit the section records, in landing order, each rung carrying its own `[N]` citation slot.
The same form is used for every cycle, single- or multi-commit; a single-commit cycle is a
one-rung ladder whose one step is the close-out:

```
## refactor: extract config loader

- [[2]] 0.42.0-1 refactor: split loader from parser
- [[N]] refactor: extract config loader

<intro paragraph...>
```

- The rung form is `- [[N]] <title>` while the commit is unlanded, becoming
  `- [[2]] X.Y.Z[-n] <title>` at backfill, as the first rung above shows. No step number and no
  `(current)` / `(done)` marker, since as-built implies done (the in-flight markers live in
  `TODO.md > ## In Progress`). Landing order is the list order. See
  [Steps are named, not numbered](prose.md#steps-are-named-not-numbered).
- **The version arrives with the SHA, not before it.** It is the one version written into prose
  (see
  [Versions live in the version-of-record only](prose.md#versions-live-in-the-version-of-record-only)),
  because here it records what a landed commit carried rather than naming a step, and together
  with the SHA it decodes an old `-V` banner. On an unlanded branch it is as rewritable as the
  SHA, so it waits for the same moment (Timing, below). The version is knowable earlier, from the
  manifest; holding it back is deliberate, buying one timing rule and a record a rebase cannot
  falsify.
- Two headers in one chores file must not share a title, or their anchors collide and the
  duplicate silently resolves to the first. Within a file that is the whole uniqueness
  requirement; titles may repeat across the repo's history.
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
long-lived release/patch branch), because a rebase or squash rewrites it on the way, and neither is
its version. A commit can't record its own SHA, so the fill lands one push later: every rung opens
with the literal `[[N]]` placeholder and no version, and each push backfills the rungs of the
commits the previous push made permanent. On a topic branch a section waits until the branch lands.
The commit itself is the record, and `git log --grep "<title>"` finds it. See cycle-protocol
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
