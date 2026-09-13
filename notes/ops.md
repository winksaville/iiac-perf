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
- **Installed and configured hosts** (2026-09-12): the plain
  0.28.11 is on both hosts, the 7600x's copied from the 3900X's
  build, and both dev builds are removed. The 7600x's
  `~/.config/iiac-perf/config.md` holds the `[freq]` table alone
  with `min_mhz = 2991` and `max_mhz = 5457`, and its
  `~/iiac-perf-example.md`, a rename of the old `~/iiac-perf.md`,
  is redundant. The 3900X has no XDG config yet (the `restore-freq`
  entry in `bugs.md`).
- **Kept run records**: the 7600x keeps its in
  `~/iiac-perf-data/<series>-<date>/`, and this repo's ignored
  `tmp/` holds the 3900X's and copies: `placement-20260905`,
  `v1v2-20260908`, `v2rows`, `mpscv1rows`, `blockval-20260912`,
  `capfix-20260912`, `accept-20260912` (the merge-batches-into-blocks
  cycle's validation), and `warmup-7600x-20260913`. The Todo entry
  citing a series names what it showed.
- **Agent-files diff**: `vc-x1 agent-files diff` compares against
  `../vc-x1-template`, stale since 2026-08-31, so use
  `vc-x1 agent-files diff ../vc-x1 -c` (0 of 11 on 2026-09-12).
- **Messages write guard**: list `open/` whole before every write
  to `../vc-x1-messages`, since a message once arrived between two
  reads of the threads being answered (m-5, 2026-09-12).
