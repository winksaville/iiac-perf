# Prose and durable text

How long-lived text is written on this project: the prose shape, the punctuation rules, and the
commit-title identity. Read this before writing durable text (notes files, commit messages, doc
comments, chores sections).

Universal file, shared with the template repository; a proposed change is edited here and
converges at the template ([Changing the agent-files](../AGENTS.md#changing-the-agent-files)).
Project-local content goes in [custom.md](../custom.md).

## Prose form

Long-lived prose on this project follows one basic shape: a short intro that explains the *why*
or the high-level *what*, sharpened to a *problem statement* where a surface calls for one (see
[Problem-first shape](#problem-first-shape)), then a `-` bullet list for the details. Wrap lines
at <=100 cols, commit titles at <=50 and commit bodies at <=72 (bullet continuations indent two
spaces); existing text re-wraps when touched, no mass sweeps. Write to the full width: wrap near
the limit rather than imitating the narrow wrap of older text. A default, not an absolute: a line
that reads better long stays long (an URL, a literal report row, indented code in a comment). One
fact per bullet or sub-bullet beats a paragraph packing several. Avoid wall-of-prose paragraphs:
they hide the structure that bullets make scannable. Punctuation that joins clauses without
naming their relationship is the same failure at sentence scale; see
[Semicolons inside bullets](#semicolons-inside-bullets) and
[Typeable punctuation only](#typeable-punctuation-only).

Surfaces that use this shape:

- Module / function / struct / field doc comments in `.rs` files; see
  [Doc comments](code.md#doc-comments-on-every-file-function-and-method).
- Commit message bodies (both work-repo and bot-repo). The <=50-col title is the
  commit-specific add-on; see [Per-commit flow](../notes/cycle-protocol.md#per-commit-flow).
- Chore descriptions in `notes/chores/chores-NN.md`; see
  [Chores section content](notes.md#chores-section-content-no-edit-list-git-is-the-record).
- Todo entries in `TODO.md` when an entry needs more than one line of detail; pure one-liners are
  still fine. Done entries take the same shape with the title bolded, detail always as
  sub-bullets rather than sentences trailing off the title line; see
  [Done entry form](notes.md#done-entry-form).

Bullet *content* differs by surface:

- **Commit bodies**: the [Problem-first shape](#problem-first-shape) for finished work, a problem
  statement then a solution statement, both broad. What is specific to a commit:
  - the problem statement defines any word the title assumes, since the title is what a reader
    meets first and it answers the problem
  - **no file list.** The diff and `git show --stat` are the mechanical record, so restating them
    is a second copy that can drift from the first. An import of a thousand files is one change
  - these are claims a reader has to follow, so they are sentences rather than fragments. A
    bullet wanting a paragraph belongs in the chores section instead
  - the **deliberation** stays out: alternatives weighed, evidence, dates, costs accepted. Those
    live in the chores section, the `## Todo` entry, and the session the `ochid:` trailer names,
    each reachable from the commit by construction. The problem itself is a *why* and belongs
    here
- **Chores / todo / done**: bullets are conceptual (design points, structural notes, the "what
  landed and why" at a notch above file-list granularity). Never a copy of the commit's edit
  list; see
  [Chores section content](notes.md#chores-section-content-no-edit-list-git-is-the-record).
- **Doc comments**: bullets are whatever structure fits (fields, cases, invariants).

### Problem-first shape

`## In Progress` cycle blocks, chores sections, `## Todo` entries, and commit bodies use a sharper
form of the same shape: a problem, then how it is answered, then the steps that get there.

- **Problem statement** (the why): one or two sentences; don't pad with intent, don't restate
  what follows it.
- **Solution statement** (the what/how): what is done about the problem, in broad terms,
  answering whatever question the problem statement raises. Surface-specific rules are in
  [Bullet content differs by surface](#prose-form) above.
- **Plan bullets** (the what/when), the steps. Formality differs by surface:
  - In Progress / chores: a committed ladder, one step per commit; see
    [Conventional-commit shape](#conventional-commit-shape-ladder--chores--commit) for the
    per-step title + `(current)` / `(done)` form.
  - Todo entries: rough informal bullets, no numbering; formalized only when the entry is
    picked up into a cycle.

**Timing decides whether the solution statement is provisional, not whether it is written.** A
cycle writes one at Preparation, before the work, and revises it as steps land; the close-out's
commit body carries the final one. A `## Todo` entry's is provisional in the same way. Only a
commit body's is settled, because a commit is finished by the time it has one. The earlier rule
here said a plan was for work not yet done and a solution for work already done, which left a
cycle unable to say at its opening what it intended to do.

### Semicolons inside bullets

A bullet that joins multiple clauses with semicolons (`A; B; C`) is a list hiding inside
running prose: break the clauses into sub-bullets so the structure shows. Semicolons in running
prose (intro paragraphs, sentence-joins) are fine. Not absolute: very short clauses or tight
pairs can stay joined inside a bullet when breaking would be more noise than signal.

### Typeable punctuation only

Durable text uses punctuation that can be typed at a terminal. Banned: `—`, `–`, `…`, `→`.
None can be entered without a compose key or a paste, so none can be grepped for, and an em
dash next to option syntax reads as another flag. Unlike the semicolon rule above this one is
absolute: they cost nothing to write and are paid on every read, so a soft rule accumulates
them.

`…` becomes `...` and `→` becomes `->`. The dashes have no single replacement, because an em
dash usually stands in for a structural decision that was not made. Make the decision:

- **A bullet's title and its body sharing a line** is a heading and a paragraph. Make the body
  sub-bullets.
- **A term and its definition** (`jj diff`, `<base>`, a flag) takes a colon, which keeps a
  glossary or a command list at one line per entry.
- **A prose aside** takes a comma, parentheses, or two sentences. Often the aside should just
  go.

Converting a heading moves its anchor. The em dash strips but the spaces on both sides survive,
so `## A — B` slugs to `#a--b` while its colon form slugs to `#a-b` (see
[Markdown anchor links](notes.md#markdown-anchor-links)). Re-point inbound links in the same
commit.

Scope is the same as [Speculation marker](#speculation-marker), plus commit titles and
everything under `src/`: doc comments, inline comments, and any user-visible string. Source is
the surface a human edits and greps most, so an untypeable character costs more there than in
prose. It applies going forward; existing text converts when touched. A code span is not exempt
by itself. Naming the character is a specimen and stays, which is how this section names them.
A banned character doing a job is a use and converts: `` `.expect(…)` `` becomes
`` `.expect(...)` ``.

Text quoted from outside this repo's prose (tool output, an error message, an already-published
commit title) is transcribed, not written, so it keeps its characters, whether or not it sits
in a code span. It matters most for commit titles: converting one stops it matching
`git log --grep` and breaks the verbatim identity that
[Conventional-commit shape](#conventional-commit-shape-ladder--chores--commit) requires between
a commit title, its chores header, and its `## Done` entry.

### Conventional-commit shape (ladder / chores / commit)

A ladder step, its chores section, and its commit description share a *title* shape, a
[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) title (`<type>: <desc>`,
an optional `(scope)` after the type: `feat(push): ...`) over [Prose form](#prose-form) detail.
They differ in the title's prefix / marker (below) and in bullet *content*: commit bodies are
problem-then-solution, ladder / chores conceptual (see "Bullet *content* differs by surface"). The
shared template:

```
<title>                          # <title> is the commit's `<type>: <desc>`
<optional prose intro>
  - <optional item>
    <optional prose intro>
      - <optional sub-item>
      ...
```

The three surfaces apply it as:

- **Ladder step** (`TODO.md` `## In Progress`): the rung is the bare title plus a `(current)` /
  `(done)` marker, and its position in the list is its position in the ladder. The last rung is
  the close-out and its text says so. Detail is bulleted, never `;`-joined inline.
- **Chores section** (`notes/chores/chores-NN.md`): no prefix, since the `##` header *is* the
  bare title; the as-built ladder is the first content under it (see
  [Chores commit references](notes.md#chores-commit-references)).
- **Commit description**: no prefix, and the title is the <=50-col first line; the body is the
  prose (see [Commit description](../notes/cycle-protocol.md#commit-description)).

The title is **identical** across all three for a given step, so a step's ladder entry, its
chores `##` header, and its commit title line up verbatim; pick the commit title first and
reuse it.

That identity is **per step**, not per cycle: each step in a cycle gets its own distinct
descriptive title, never one shared cycle title uniquified by a step marker. The cycle's chores
section header carries the anticipated *close-out* title. To keep a cycle's commits collectable
with one `git log --grep`, give the step titles a common greppable stem (e.g. `config loader`).

**Cycle bookend titles**: the opening commit's title is the close-out title plus " opening",
same type (`feat: dynamic warmup opening` / `feat: dynamic warmup`), so one
`git log --grep "<close-out title>"` returns exactly the pair that brackets the cycle. The
type repeats the close-out's even though an opening is mostly bookkeeping: identical prefixes
make the pair scannable. Rungs between keep their own titles on the stem.

### Steps are named, not numbered

A step has a title and no number. Nothing in a ladder rung, a chores as-built rung, a `## Done`
entry, or a commit gives a step an ordinal: a rung's place in the list already *is* its place in
the ladder, so a number beside it would restate the position and then have to be maintained.

- **The title is the identifier.** A record points at a step by its title, a plain greppable
  mention, which is why the title is verbatim-identical across the three surfaces.
- **Unambiguous, not globally unique.** Two titles must be distinguishable in the two places a
  title is resolved: within its own cycle, so a ladder rung names one step, and within its chores
  file, since a `##` header is also an anchor and a repeated slug silently dedupes to the first
  one. Across the repo's history a title may repeat.
- **Nothing renumbers.** Inserting, reordering or dropping a step edits the ladder list and
  nothing else. On an unlanded topic bookmark the rungs that already committed an older ladder
  come along; see
  [Topic bookmarks are drafts](../notes/cycle-protocol.md#topic-bookmarks-are-drafts).
- **`## Todo` ranks are the exception that stays numbered**, because a priority list has an order
  worth reading off (see [Todo format](notes.md#todo-format)). Those numbers are positional too,
  and are equally never used as references.

### Versions live in the version-of-record only

No version appears in durable prose: not in an in-flight ladder rung, a chores header, a commit
title, or a commit body. The manifest is the version's only written home (see
[versioning.md](../notes/versioning.md)), and a commit's version is read from that file at that
commit.

**Why:** the version is a build stamp answering "which commit produced this artifact", not a name
for a step. Written into prose it becomes a second identifier that history is free to invalidate:
one renumber of published versions turns every prose mention, transcript and pasted report into
residue that needs a decoder to read. A renumber cannot touch a title.

**Two surfaces record a version rather than name a step.** Both record a *commit*, never a step,
which is what keeps them outside the rule rather than exceptions to it:

- **A chores as-built rung** records a version alongside that commit's SHA once the commit is on
  a permanent branch. The pair decodes an old `-V` banner or a pasted report. It obeys the SHA's
  timing exactly, so a rung on an unlanded branch carries neither (see
  [Chores commit references](notes.md#chores-commit-references)).
- **A `## Done` entry**, in `TODO.md` and in `done.md`, carries the close-out's version ahead of
  its title (see [Done entry form](notes.md#done-entry-form)). Here the version is the search
  key: the question a reader arrives with is "what shipped in 0.78.2", and with no version
  written anywhere in the Done list that question has no answer.

The two differ in timing, and the reason is the SHA rather than the version. The rung waits
because a commit cannot record its own SHA; a Done entry has no SHA to wait for and its version
is already in the manifest of the commit it is written in, so it is written at close-out. On an
unlanded bookmark it is a draft like the rest of the line
([Topic bookmarks are drafts](cycle.md#topic-bookmarks-are-drafts)), and a renumber of published
versions rewrites it in the same sweep as the rungs.

**How to apply:** name the step by its title and the phase in words ("the close-out", "the
opening"). Writing *about* versioning is unaffected: a version named as a specimen, whether in the
scheme's own notation, a decoder table, or a narrative about a renumber, is a use of the word
rather than an identifier for a step, the same distinction
[Typeable punctuation only](#typeable-punctuation-only) draws between naming a character and using
one. Existing versioned prose is grandfathered and converts when touched, no sweep.

## Speculation marker

Durable text the bot writes (agent-files, `notes/`, commit bodies, chores sections)
should stick to observations and direct descriptions of the code or data. If a mechanism,
hypothesis, or causal claim enters the text, prefix it with "We think ..." (a royal "we") so a
reader can tell the measured from the inferred.

**Why:** unmarked speculation reads like evidence, and a future reader (or the bot on a later
session) can pick it up as a known fact when it's not. Measured / inferred is a distinction
worth keeping visible in the written record.

**How to apply:** observations and factual descriptions need no marker. Prefix with
"We think ..." (or a close variant like "Our guess is ...") when the claim is a mechanism
("X wins because Y caches better"), a cause ("the drift was due to thermal state"), a
prediction ("this should scale linearly"), or any reasoning not directly supported by the data
on hand.

## Plain synopsis after technical explanations

When a conversational reply centers on a technical explanation (measurement theory, statistics,
hardware behavior), end it with a short plain-language synopsis, no jargon and no symbols, so
the reader can check their understanding against the technical version.

**Why:** the technical form is precise but easy to misread; the plain form catches
misunderstandings early, when they are cheap.

**How to apply:** conversation only, not notes files (a notes entry should already lead with
the why). Mark it clearly (e.g. "The plain version:"). A reply that is already plain needs no
synopsis.
