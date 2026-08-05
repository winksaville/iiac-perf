# Prose and durable text

How long-lived text is written on this project: the prose shape, the punctuation rules, and the
commit-title identity. Read this before writing durable text (notes files, commit messages, doc
comments, chores sections).

Universal file, shared with the template repository; a proposed change is edited here and
converges at the template ([Changing the agent-files](../AGENTS.md#changing-the-agent-files)).
Project-local content goes in [custom.md](../custom.md).

## Prose form

Long-lived prose on this project follows one basic shape: a short intro that explains the *why*
or the high-level *what*, then a `-` bullet list for the details. Wrap lines at <=100 cols,
commit titles and bodies at <=72 (bullet continuations indent two spaces); existing text
re-wraps when touched, no mass sweeps. Avoid wall-of-prose paragraphs: they hide the structure
that bullets make scannable. Punctuation that joins clauses without naming their relationship is
the same failure at sentence scale; see [Semicolons inside bullets](#semicolons-inside-bullets)
and [Typeable punctuation only](#typeable-punctuation-only).

Surfaces that use this shape:

- Module / function / struct / field doc comments in `.rs` files; see
  [Doc comments](code.md#doc-comments-on-every-file-function-and-method).
- Commit message bodies (both work-repo and bot-repo). The <=72-col title is the
  commit-specific add-on; see [Per-commit flow](../notes/cycle-protocol.md#per-commit-flow).
- Chore descriptions in `notes/chores/chores-NN.md`; see
  [Chores section content](notes.md#chores-section-content-no-edit-list-git-is-the-record).
- Todo and Done entries in `TODO.md` when an entry needs more than one line of detail. Pure
  one-liners are still fine.

Bullet *content* differs by surface:

- **Commit bodies**: one bullet per *distinct change*, not per file, with detail as sub-bullets
  in short sentence fragments (full sentences are discouraged). Source of truth for the
  mechanical edit list.
  - a file earns its own bullet when its gist is its own: `README.md: new Overview intro`
  - files sharing one gist share one bullet, named by common path or glob, and by count once
    enumerating stops informing: `agent-data/*.md: "do not edit here" replaced`,
    `vendor/: upstream v1.2.3, 1043 files`
  - the body is a record, never a directory listing. An import of a thousand files has one
    change in it, and restating what `git show --stat` already prints is a second copy that
    can drift from the first
- **Chores / todo / done**: bullets are conceptual (design points, structural notes, the "what
  landed and why" at a notch above file-list granularity). Never a copy of the commit's edit
  list; see
  [Chores section content](notes.md#chores-section-content-no-edit-list-git-is-the-record).
- **Doc comments**: bullets are whatever structure fits (fields, cases, invariants).

### Problem + plan shape

`## In Progress` cycle blocks, chores section intros, and `## Todo` entries use a sharper form
of the same shape:

- **Problem statement** (the why): one or two sentences; don't pad with intent, don't restate
  the plan.
- **Plan bullets** (the what/when): formality differs by surface:
  - In Progress / chores: a committed ladder, one step per commit; see
    [Conventional-commit shape](#conventional-commit-shape-ladder--chores--commit) for the
    per-step title + `(current)` / `(done)` form.
  - Todo entries: rough informal bullets, no numbering; formalized only when the entry is
    picked up into a cycle.

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
file-by-file, ladder / chores conceptual (see "Bullet *content* differs by surface" above). The
shared template:

```
<optional - X.Y.Z[-N]> <title>   # <title> is the commit's `<type>: <desc>`
<optional prose intro>
  - <optional item>
    <optional prose intro>
      - <optional sub-item>
      ...
```

The three surfaces apply it as:

- **Ladder step** (`TODO.md` `## In Progress`): the title is prefixed with the version
  (`X.Y.Z-N <title>`) and carries a `(current)` / `(done)` marker. The bare three-element
  `X.Y.Z` (no `-N`) is the close-out step. Detail is bulleted, never `;`-joined inline.
- **Chores section** (`notes/chores/chores-NN.md`): no version prefix, since the `##` header
  *is* the bare title, with a `Commits: [[ref]]` line first under it (empty until backfilled;
  see [Chores commit references](notes.md#chores-commit-references)).
- **Commit description**: no version prefix, and the title is the <=72-col first line; the body
  is the prose (file-by-file for the work repo, per
  [Per-commit flow](../notes/cycle-protocol.md#per-commit-flow)).

The title is **identical** across all three for a given step, so a step's ladder entry, its
chores `##` header, and its commit title line up verbatim; pick the commit title first and
reuse it.

That identity is **per step**, not per cycle: each step in a cycle gets its own distinct
descriptive title, never one shared cycle title uniquified by a step marker. The cycle's chores
section header carries the anticipated *close-out* title. To keep a cycle's commits collectable
with one `git log --grep`, give the step titles a common greppable stem (e.g. `config loader`).

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
