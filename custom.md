# custom.md - iiac-perf's project layer

The one agent-editable instruction file (see [AGENTS.md](AGENTS.md#custommd-the-project-layer)).
Loaded after AGENTS.md; on conflict, this file wins.

## Medium and validation

The artifact is the `iiac-perf` CLI, a Rust crate (manifest `Cargo.toml`, package name
`iiac-perf`); versioning specifics in [versioning.md](notes/versioning.md).

- **Full validation**
  - when: per-commit checklist step 4; skip-able for notes-only commits, mandatory at close-out
  - run as separate invocations, each exit status checked:
    1. `cargo fmt`
    2. `cargo clippy --all-targets -- -D warnings`
    3. `cargo test`
    4. `cargo install --path . --locked`
    5. (re-test if anything substantive changed)
- **Fast validation**
  - when: ladder checklist step 3
  - `cargo test --bins`
- **Pipelines hide failures**
  - never pipe a validating command into `tail`/`grep`
  - never `&&` after a piped stage: a pipeline's status is the last command's
  - `${PIPESTATUS[0]}`: the escape hatch when a pipe is genuinely wanted

## Project conventions and overrides

- **Installed vc-x1 predates the `code` -> `work` scope rename** (checked 2026-07-31)
  - overrides the scope note in [AGENTS.md Terminology](AGENTS.md#the-dual-repo-model): use
    `--scope=code|bot|code,bot` with the installed binary
  - retire this entry when vc-x1 is upgraded past the rename
- **Acquaint routine addition: check the mailbox** (adopted 2026-07-31)
  - on acquaint, check `../vc-x1-template/messages/iiac-perf.md`; an absent file means no mail
  - the message protocol is `../vc-x1-template/MESSAGES.md`
- **Non-top-commit ochid exception dropped pending verification** (2026-07-31)
  - the pre-restructure AGENTS.md permitted `jj commit` plus a hand-written `ochid:` for a
    commit pushed later as a non-top commit, on the ground that push stamps only the topmost
    commit
  - hard rule 5 (never hand-write ochids) governs until `vc-x1 push`'s actual stamping
    behavior on multi-commit pushes is verified; if push stamps every commit it creates, the
    exception stays dead, otherwise raise it as a template finding

## Dogfood log

Dated entries on where these instructions chafed, failed, or got amended; the evidence base
for the promotion decision in the template repository (vc-x1-template).

- 2026-07-31: adopted mid-dogfood-window
  - adoption base: the template's `AGENTS-vc-x1-f5-20260730-snapshot/` (AGENTS.md, CLAUDE.md,
    agent-data/), created the same day from vc-x1's live copy, the window's authority
  - local copies verified byte-identical to the snapshot at adoption (`diff -r`)
- 2026-07-31: adoption base amended before first commit
  - the snapshot's AGENTS.md gained rule 0 (read custom.md before acting) and hard-rules-first
    ordering; authored snapshot-side while vc-x1's session was live, with the pending sync to
    vc-x1 tracked in the template's snapshots.md
  - local AGENTS.md re-copied from the snapshot; pin set re-verified byte-identical
- 2026-07-31: template restructured into vc-x1-template; pin lines made generic
  - the template + coordination point is now the vc-x1-template repo: init payload in `work/`
    and `work.claude/`, discussion artifacts in `agents-protocol/`, mailboxes in `messages/`;
    the old vc-x1-work-repo-template and vc-x1-bot-repo-template are untouched pending wink's
    discussion with vc-x1
  - every "pinned to vc-x1-work-repo-template" line in the pin set became "the template
    repository" (snapshot-side amendment, same pending-sync flow as rule 0)
  - local pin set re-copied; snapshot, `work/`, and this repo verified three-way byte-identical
- 2026-07-31: first push under the new rules; two checklist gaps found (the 0.23.1 push)
  - given "desc and push", the agent jumped straight to the description: cycle.md's per-commit
    checklist has no step for backfilling the previous push's chores refs, bumping the
    version-of-record, or appending the chores record, so following it verbatim skips all
    three. Proposed template fix: add them as explicit steps before "write the description"
  - rule adopted (wink): **every commit belongs to a cycle, single- or multi-commit; there is
    no out-of-cycle push.** Mid-ladder the cycle is implied; otherwise ask "single- or
    multi-step cycle?" before preparing the commit. A single-commit cycle is a bare `X.Y.Z`
    close-out, so its validation is mandatory, and with no planning phase it skips
    `## In Progress` and goes straight to chores + Done
  - convention adopted (wink): chores commit references use the **as-built ladder form** for
    every cycle (rung `- [[N]] X.Y.Z[-n] <title>`), replacing the `Commits:` line; codified in
    agent-data/notes.md + cycle.md (snapshot-side, pending vc-x1 sync) and both
    cycle-protocol.md copies (this repo's and the template payload's); pre-existing
    `Commits:` lines are grandfathered
  - convention adopted (wink): chores files carry a **title-only `## Table of Contents`**, one
    `- [<title>](#<anchor>)` entry per commit-recording section; no versions or refs, so it
    never needs backfill (the TOC navigates, the ladder records); codified beside the ladder
    form, first instance in chores-05.md
  - the pre-restructure AGENTS.md is replaced (preserved in jj history, and verbatim as the
    template's AGENTS-iiac-perf.md)
  - the parked `punctuation-sweep` branch (TODO.md Todo #2) edited the old AGENTS.md; its
    AGENTS.md hunks are obsolete now that the typeable-punctuation rule ships as hard rule 8
    and prose.md, so the branch needs re-scoping to its README / TODO.md /
    notes/cycle-protocol.md conversions before landing
  - remaining pre-restructure local deltas (e.g. clap `verbatim_doc_comment` guidance) still
    need distilling into this file or proposing into the template; review findings recorded
    in the template's AGENTS-vc-x1-f5-20260730-review-iiac-perf-f5-20260731.md
