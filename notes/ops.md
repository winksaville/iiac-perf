# Ops notes

Durable machine/session facts that outlive any one cycle,
migrated from `TODO.md > ## In Progress` blocks at close-out.

- **7600x is reachable** (host renamed from r5-7600x
  2026-07-27): `ssh 7600x` and `scp` both work (scp verified
  2026-07-27; a pre-rename "Network is unreachable" scp
  failure no longer reproduces). No `target-cpu=native`
  anywhere, so one release build is valid on both boxes.
  Non-interactive ssh has no `~/.cargo/bin` on PATH — use the
  full binary path in `ssh 7600x '...'` commands.
- **Bot-sandbox measurement gotcha**: the bot's sandbox uses
  `--unshare-pid`, so a background spinner started in one
  shell is invisible to every other one — `pgrep`/`pkill`
  silently find nothing and cannot stop it. The only reliable
  "machine is quiet again" signal is the `timeout` expiring.
  Two rounds of measurements were taken under contention
  before this was understood (2026-07-25). Related: an
  unpinned bench run on the machine hosting the bot session
  competes with the session itself — a 2026-07-27 run graded
  F at 19.25% disturbed from exactly this.
