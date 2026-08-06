# jj and cross-repo linking

How version control is driven on this project (jj, never raw git) and how the two repos'
commits point at each other (the `ochid:` trailer). Read this before any jj operation beyond
`st` / `log` / `diff`, and always before touching a commit description.

Universal file, shared with the template repository; a proposed change is edited here and
converges at the template ([Changing the agent-files](../AGENTS.md#changing-the-agent-files)).
Project-local content goes in [custom.md](../custom.md).

## jj Basics

**Use jj, not git, for version-control operations** (status, log, diff, commit, push, history
rewrite). jj coexists with the git backend, so the repo *can* be driven with raw `git`, but
this project's workflow (bookmarks, the working-copy `@` model, ochid trailers) is expressed in
jj terms; reaching for `git` invites state that doesn't match the jj documentation here. There
is no `jj mv`: to move/rename a tracked file, just `mv` it on disk and jj detects the rename by
content.

- `jj st -R .` / `jj st -R .claude`: show working copy status
- `jj log -R .` / `jj log -R .claude`: show commit log
- `jj commit -m "title" -m "body" -R <repo>`: finalize working copy into a commit
- `jj describe -m "title" -m "body" -R <repo>`: set description without committing
- `jj git push --bookmark <name> -R <repo>`: push a bookmark (no `--allow-new` flag; jj pushes
  new bookmarks without special flags)
- `jj bookmark set <name> -r <rev> -R <repo>`: create the bookmark, or move an existing one, at
  `<rev>`. Moving it backwards needs `--allow-backwards`
- `jj git push --named <name>=<rev> -R <repo>`: create *and* publish in one step, which is the
  usual way a cycle's bookmark is born (see [Cycle bookmarks](#cycle-bookmarks-create-and-land))
- In jj, the working copy (@) is always a mutable commit being edited. `jj commit` finalizes it
  and creates a new empty working copy on top.
- The bot repo always has uncommitted changes during an active session because session data
  updates continuously.
- `jj rebase` uses `--onto`/`-o` to name the destination(s).

## Cross-repo linking (ochid trailers)

The cross-reference between the work repo and the bot repo is what makes the dual-repo work:
every commit points at its counterpart in the other repo, so the "what" (code) and the
"why / how" (session) stay linked across time. That pointer is the **ochid** (Other Change ID)
git trailer.

A **chid** is jj's change ID, a permanent identifier that survives rebases and `describe`s
(unlike the commit ID / git SHA, which changes on rewrite). An **ochid** trailer carries the
counterpart commit's chid as a workspace-root-relative path:

- Paths start with `/`, the workspace root, i.e. the work repo (the project root). `/.claude`
  is the bot sub-repo.
- `ochid: /<chid>` references a change in the **work repo**.
- `ochid: /.claude/<chid>` references a change in the **bot repo**.

Trailers are blank-line-separated `key: value` lines at the end of the commit body, using the
chid's **12-character** prefix:

```
ochid: /.claude/xvzvruqowktp   # points to a bot-repo change
ochid: /wtpmottvxqzl           # points to a work-repo change
```

How many, and which direction:

- **Work-repo commits** each carry one `ochid: /.claude/<bot-chid>`, the bot repo's change ID.
- **The bot-repo commit** carries one `ochid: /<work-chid>` per work-repo commit in that push.
  The count is per *push*, not per cycle. A trapezoid close-out whose rungs were pushed 1:1 as
  they landed still carries exactly one; more than one occurs when a single push publishes
  several work-repo commits.

Use `vc-x1 chid -s work,bot -L` to capture the change IDs (first line work repo, second bot
repo).

`ochid:` trailers are **stamped by `vc-x1 push`**; never hand-write them into a commit body or
`--title`/`--body`.

## Re-describing: coordinate first, and keep the trailer

**Never `jj describe` a commit that is already published or already carries trailers without
coordinating with everyone involved first.** It is a history rewrite, and it silently drops the
cross-repo link. Describing a fresh local commit that has never been described and carries no
trailers is authoring a message rather than rewriting one, and is not covered; that is the
sub-cycle ladder's
[per-Work-commit contract](../notes/cycle-protocol.md#per-work-commit-contract-within-a-ladder)
step 4.

When a re-describe is agreed, copy any `ochid:` trailers into the new body by hand (the "don't
hand-write trailers" rule covers push authoring a message from scratch, not preserving one
already stamped). Hit at the 0.77.2 amend (2026-07-29), where the trailer survived only that
way; `vc-x1 fix-desc` repairs a dropped one by title match.

## Cycle bookmarks: create and land

The mechanics behind [Cycles run on a bookmark](cycle.md#cycles-run-on-a-bookmark); that section
holds the rule and when it applies, this one holds the commands.

**Create**, at the cycle's opening, with the bookmark named for the cycle rather than the step:

- `jj git push --named <bookmark>=@- -R .` is the common case: it creates the bookmark at the
  last committed change and publishes it in one invocation.
- Any other revision works as the `=<rev>` right-hand side. When `<rev>` is not `@-`, follow with
  `jj new <rev> -R .` so the working copy actually sits on the new line; otherwise the bookmark
  exists and the next commit lands somewhere else.
- `jj bookmark set <bookmark> -r <rev> -R .` creates it without publishing, for a line that is
  not ready to be seen.

**Land**, once the close-out is approved: fast-forward the permanent branch to the bookmark.

- `jj bookmark set main -r <bookmark> -R .`, then `jj git push --bookmark main -R .`.
- It is a fast-forward, so `--allow-backwards` is not wanted; needing it means the bookmark is
  not a descendant of `main` and the situation wants a look, not a flag.
- Landing is the moment the cycle's commits become permanent, so it triggers the records that
  wait on permanence: the chores as-built rungs take their SHAs and versions
  ([Chores commit references](notes.md#chores-commit-references)).
- The bookmark is redundant once landed and may be deleted (`jj bookmark delete <bookmark>`),
  the same disposal the long-lived case gets below.

We think a `vc-x1 start-change <bookmark>` will eventually own the create half; it would replace
the create bullets and nothing else, which is why the rule and the commands are separated.

## Long-lived bookmarks: merge-only by default, deletable once merged

A long-lived bookmark (a program line pushed rung by rung across cycles) is not a cycle bookmark:
it carries published history, and the discipline protects that history while the bookmark is its
only holder.

- conflicts with `main` are resolved in merge commits, never by rewriting published rungs
- a rebase is never required for correctness: it is a linearity preference
- coordinated rebases stay available (the bar is on unilateral rewrites, not rewrites), at the
  known cost of staling the git-SHA citations in the records; chids and ochid trailers survive
  a rebase
- once the bookmark's history is fully merged into `main` the bookmark is redundant and may be
  deleted

The contrast with a cycle bookmark is the whole point: that one is a draft and may be rewritten
freely until it lands (see [Topic bookmarks are drafts](cycle.md#topic-bookmarks-are-drafts)),
this one is published and may not. Refined 2026-08-03 in vc-x1 from the earlier "treated as
permanent, never rebased" wording, after the fully merged `refactor-vc-x1` bookmark was deleted.

## Resolvability

A change ID travels with its commit: a **pushed** commit resolves to the same chid in every
clone; cloning the bot repo gave the published `main` tip the same chid as an existing clone.
We think jj carries the change ID in the git commit object, so it survives `jj git clone` /
fetch.

The local-only case is the **working-copy `@`**: jj mints a fresh random chid for `@` in each
clone, so an unpushed `@` is never a stable ochid target. This is why a bot-repo ochid names
`@-` (the last committed change), not `@`.

## .vc-config.toml

Each repo contains a `.vc-config.toml` whose `[repos]` registry records the workspace layout.
Values are ordinary paths relative to the config file's directory (absolute allowed,
discouraged), so the two sides' blocks **differ**: the entry that resolves to the config's own
directory names its side, and the two sides must agree on the same resolved work/bot pair:

```toml
# work side          # bot side
[repos]              [repos]
work = "."           work = ".."
bot = ".claude"      bot = "."
```

Ochid trailer prefixes are fixed per-side labels (`/` work, `/.claude` bot) resolved by side
detection, not filesystem paths.
