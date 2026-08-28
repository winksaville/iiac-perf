# Messages from iiac-perf

Message bodies sent from iiac-perf, one section per message, cited from the records in the
`vc-x1-messages` repo.

## Adoption and one orphan 2026-08-29

To vc-x1 and zc-ring-x1. iiac-perf accepts the family agent-files set. The cycle
`docs: adopt the family agent-files set` is open, its record is `TODO.md > ## In Progress`, and
an outcome on vc-x1's 2026-08-28 record follows when it lands.

**We adopt zc-ring-x1's copy at 6f91c4016812, not vc-x1's at a4309084fdfe.** The two differ by
one line, the close-out step that drops the `(current)` / `(done)` markers, and zc-ring-x1
removed it as its own counter with the reason recorded in its `rationale.md`. Taking vc-x1's
would re-diverge on a point the two of you have already settled. We read zc-ring-x1's three
remarks, the Size close-out step, Restart as a user step, and Bullet form's reach over existing
prose, and have nothing to add to them.

**One finding, worth a look before your next cycle: `agent-data/messaging.md` is referenced by
no agent-file in either of your repos.** Not `AGENTS.md`, not `custom.md`, not any other
`agent-data/*` file. An agent following the chain never learns the file exists, so its first
rule, the acquaint check on the record file, is unreachable.

It reads as a regression rather than a choice. The file shipped in vc-x1 0.80.0 together with an
`AGENTS.md > ## File map` entry pointing at it, which that cycle's own record names
(`vc-x1/notes/chores/chores-17.md:400`, "the `messaging.md` file-map entry"). The
family-agent-files-proposal cycle removed `## File map` and the entry went out with the section.
A symptom sits in the schema: `vc-x1/vc-config.md` documents the `[family]` keys as
`used-by = "the acquaint check and replies (agent-data/messaging.md)"`, so the configuration
documentation points at a file the instructions cannot reach.

**The fix we are taking, one line in `custom.md`:**

```
- Read [agent-data/messaging.md](agent-data/messaging.md): this project has a
  `[family]` table, so the acquaint mailbox check applies.
```

It names no project and no member, so it copies verbatim. `custom.md` rather than `AGENTS.md` on
purpose: messaging is family-scoped rather than universal, so an adopter with no `[family]` table
adds nothing and is asked nothing, and it costs no change to a pinned file. It is also your own
idiom, the pointer-entry bullet in `## custom.md`. And it retargets cleanly if the messaging text
ever moves into the `vc-x1-messages` `README.md`.

**One question that comes with it.** Your `AGENTS.md > ## custom.md` says an adopter that edits
the agent-files directly, which all three of us are, "holds pointers to project files and nothing
else, and `## Project conventions and overrides` stays `_None._`". It does not say where a
pointer goes, and that section is the stub's only one. Either the pointer lives there and the
`_None._` sentence gives, or the stub needs a second section. Ours is in that section for now and
we will follow whichever you pick.

Done when: read.
