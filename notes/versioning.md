# Versioning

How this project versions its commits and the running artifact. The scheme, meaning the manifest's
suffix spelling and the unique-per-commit aim, is generic and shared across projects: this file is
copied **verbatim**, with
[Recording the version-of-record](#recording-the-version-of-record) covering each medium by
conditional rather than per-project edits.

## Terms

Three names, used as defined here across
[AGENTS.md](../AGENTS.md),
[cycle-protocol.md](cycle-protocol.md), and the notes files:

- **version**: the per-commit version (e.g. `0.3.0-5.3.0`). It lives in the manifest. No
  ladder, todo entry, commit title, or commit body writes one, the one exception being the
  chores as-built rung recording a landed commit (see
  [Versions live in the version-of-record only](../agent-data/prose.md#versions-live-in-the-version-of-record-only)),
  and its suffix encodes the cycle phase for whoever inspects the manifest (see
  [Suffix scheme](#suffix-scheme)).
- **version-of-record**: the authoritative stored copy of the version, in the project's manifest
  (see [Recording the version-of-record](#recording-the-version-of-record)); a built or running
  artifact derives from it.
- **versioning**: the topic, this scheme as a whole.

## A stamp, not a name

The version answers "which commit produced this artifact". It is not a name for a step: a step is
named by its title and located by its position in the ladder list (see
[Steps are named, not numbered](../agent-data/prose.md#steps-are-named-not-numbered)). So the
suffix below is the only number in the system, and since nothing dereferences it, reordering or
inserting a step leaves the versions already committed alone.

## Advancing X.Y.Z: scope decides

Which of the three numbers moves is decided by the **scope of the change**, not by whether it
touched code: **minor for architecture, patch for everything else.**

- The test is whether the *shape* of the system changed or only its contents. Minor when the
  structure moves: a subsystem added or removed, a pipeline reshaped, an output contract
  redesigned. Patch for work within the existing shape: incremental features, new cases,
  presentation, docs, notes.
- **A docs-only change can be a minor** and a large code change can be a patch. Volume is not
  scope, and the earlier functional-versus-docs test kept getting this backwards.
- **Major is a project's own call**, since what `X` promises depends on the artifact and its
  users; the project records that promise in [custom.md](../custom.md).

**Why:** the suffix already encodes a commit's phase, so `X.Y.Z` is free to encode the only other
thing a reader wants from a version at a glance, how big a change they are looking at.

## Recording the version-of-record

Where the version-of-record lives, how it's stored and surfaced, and how often it changes. Pick
the case that fits your medium:

- **Manifest**, where the version-of-record is stored:
  - if Rust, `Cargo.toml` `[package].version`
  - if Python, `pyproject.toml` `[project].version` (or the committed config it's sourced from)
  - otherwise wherever the medium records it (a generic `version.toml`, a book's frontmatter,
    ...); add the case as needed
- **Notation**, how the `-` form is stored:
  - if the format allows `-` (TOML `version.toml`, `Cargo.toml`), store it verbatim
  - if it bars `-` (PEP 440's local segment, e.g. a Python project), remap to `+`, so
    `0.3.0-5.3.0` becomes `0.3.0+5.3.0`: same version, just the stored spelling
- **Reporter**, how a built artifact surfaces the version-of-record:
  - if a CLI app, `<cli-app> -V`
  - if a TUI/GUI, add to Help/About or display on the title
- **Cadence**, how often to bump: see
  [Unique per commit](#unique-per-commit-preference-not-requirement); this project follows the
  per-commit preference.

## Unique per commit (preference, not requirement)

Our general notion is that the version-of-record should change on **every commit**, so a built or
running artifact identifies the exact commit it came from.

- This is a preference, **not** a hard requirement. A project following the cycle protocol may
  bump less often: once per cycle, only at release, and so on. Record the choice in
  [Recording the version-of-record](#recording-the-version-of-record) if it differs.
- It is achievable because the cycle's versions (below) are **pre-assignable**, unlike the git
  SHA, which a commit cannot contain (see the cycle protocol's
  [Commits backfill](cycle-protocol.md#commits-backfill)).

## Suffix scheme

The cycle (Preparation -> Work -> Close-out; see [cycle-protocol.md](cycle-protocol.md)) encodes
each commit's phase in the version suffix, the **final identifier `0` marking a Preparation**.

This is the manifest's own spelling, read by whoever inspects `Cargo.toml` or `-V` output. It is
not a name for a step: a step is identified by its title, and an in-flight ladder rung carries
neither a number nor a version (see
[Steps are named, not numbered](../agent-data/prose.md#steps-are-named-not-numbered)). The one
prose surface that records a version is a chores as-built rung, where it is a property of a landed
commit sitting beside that commit's SHA. The identifiers below count commits within a phase;
nothing dereferences one.

- `X.Y.Z-0`: Preparation
- `X.Y.Z-1`, `X.Y.Z-2`, ...: Work commits
- `X.Y.Z`: Close-out (bare version, no suffix)

**Preparation is optional.** A lightweight cycle, with no ladder and no setup commit, skips `-0`
and starts at `-1` (its first Work commit). The same holds at every level: a sub-cycle needing no
Preparation omits its `.0` (see Nesting). One that grows a Preparation later adds the `0` step
without renumbering siblings.

Disambiguation:

- `-10`: Work commit #10 (final identifier `10`), not a Preparation.
- `-1.0`: Preparation of the `-1` sub-cycle (final identifier `0`).

**Nesting.** Sub-cycles append another level, recursively:

- `X.Y.Z-3.0`: Preparation of the `-3` sub-cycle
- `X.Y.Z-3.1`, `X.Y.Z-3.2`: its Work
- `X.Y.Z-3`: its Close-out
- `X.Y.Z-3.1.0`: Preparation of the `-3.1` sub-sub-cycle

Bump the version-of-record at the start of each phase, so the active phase is recorded and, per
the preference above, every commit carries a distinct version.
