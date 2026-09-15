# Ops notes

Durable machine/session facts that outlive any one cycle,
migrated from `TODO.md > ## In Progress` blocks at close-out.

- **7600x is reachable** (host renamed from r5-7600x
  2026-07-27): `ssh 7600x` and `scp` both work (scp verified
  2026-07-27, and a pre-rename "Network is unreachable" scp
  failure no longer reproduces). No `target-cpu=native`
  anywhere, so one release build is valid on both boxes.
  Non-interactive ssh has no `~/.cargo/bin` on PATH. Use the
  full binary path in `ssh 7600x '...'` commands.
- **Bot-sandbox measurement gotcha**: the bot's sandbox uses
  `--unshare-pid`, so a background spinner started in one
  shell is invisible to every other one: `pgrep`/`pkill`
  silently find nothing and cannot stop it. The only reliable
  "machine is quiet again" signal is the `timeout` expiring.
  Two rounds of measurements were taken under contention
  before this was understood (2026-07-25). Related: an
  unpinned bench run on the machine hosting the bot session
  competes with the session itself: a 2026-07-27 run graded
  F at 19.25% disturbed from exactly this.
- **Installed and configured hosts** (2026-09-15): the plain
  0.28.13 is on the 3900X, installed at `feat: config and setup`'s
  Land, and the 7600x still has the plain 0.28.11, owed a copy,
  and the stale `iiac-perf-dev` 0.28.13-9. The 3900X rebooted
  2026-09-15 and its cpufreq files read owned by `wink` after it,
  a sudo-free pin there not yet run since. Both have
  `~/.config/iiac-perf/config.md` written by `setup --apply` (the
  3900X 1745-4673 MHz, the 7600x 2991-5457 with `pin_mhz = 4701`)
  and the udev permissions, so pins and restores run without sudo,
  the 7600x's shown to survive a reboot. The 7600x's
  `~/iiac-perf-example.md`, a rename of the old `~/iiac-perf.md`, is
  redundant.
- **The 7600x's declared clamp** (corrected 2026-09-14): its
  `~/.config/iiac-perf/config.md`, rewritten 2026-09-12 and
  recorded then as declaring `min_mhz = 2991` and
  `max_mhz = 5457`, held the 3900X's table instead, prose and
  all, `1745` and `4673`, so a restore there would have capped
  the clock at 4.67 GHz. `setup` did not catch it, both numbers
  fitting the hardware range. `setup --apply` rewrote it from the
  live state on 2026-09-14 to `2991` and `5457`, the wrong file
  kept as `~/iiac-perf-data/config.md-3900x-clamp-20260914`.
- **Kept run records**: the 7600x keeps its in
  `~/iiac-perf-data/<series>-<date>/`: `blocks-1s-20260905`,
  `placement-20260905`, `v1v2-20260908`, and `warmup-20260913`,
  seen 2026-09-14. The 3900X's lived in this repo's ignored `tmp/`
  (`placement-20260905`, `v1v2-20260908`, `v2rows`, `mpscv1rows`,
  `blockval-20260912`, `capfix-20260912`, `accept-20260912`, and
  the `warmup-7600x-20260913` copy) and were found gone on
  2026-09-14, removed between that morning and the afternoon by
  nothing the session ran. The 3900X halves of those series exist
  now only as the numbers the Todo entries and the landed cycle
  records quote. `tmp/` is scratch: a series worth keeping goes to
  a directory outside the repo, as the 7600x's do.
- **Agent-files diff**: `vc-x1 agent-files diff` compares against
  `../vc-x1-template`, stale since 2026-08-31, so use
  `vc-x1 agent-files diff ../vc-x1 -c` (0 of 11 on 2026-09-12).
- **Messages write guard**: list `open/` whole before every write
  to `../vc-x1-messages`, since a message once arrived between two
  reads of the threads being answered (m-5, 2026-09-12).
