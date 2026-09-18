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
  and the stale `iiac-perf-dev` 0.28.13-9 (the dev build there is
  0.28.16-12 since 2026-09-17: wink installs on the 7600x by
  building on the 3900X and `scp` to `~/.cargo/bin`, and the agent
  did the same over ssh, the build being valid on both). The 3900X rebooted
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
- **A measuring host is not polled while it measures** (2026-09-18, at
  `feat: a shorter trustworthy run on the 7600x`): a watch that ssh'd
  the 7600x every 90 s ran against invocations of about 91 s, and the
  baseline came back with about 15% of its runs slow, 98 to 152 ns
  against a 94 to 96 ns core. The repeat meant to clear it was polled
  too: the first watch was still inside its half-hour and polled the
  repeat at the same rate, unnoticed until it expired. A watch outlives
  what it watches, so stop it or do not arm it. Compute the expected
  duration, add a tenth, wait, look once, and arm nothing against a
  host that is the subject.
- **The 7600x shows about one run in seven slow** (2026-09-18) on
  `zcr-spsc-v3-2t` pinned at `--pin-cpus 4,5`: 12 runs of 80 twice
  over, landing 98 to 152 ns against a 93.6 to 96.1 ns core whose own
  spread is 0.55%. It is what makes the plain `LSC runs` read 6.9%
  where the trimmed pair reads 0.9%. A third baseline with nothing
  watching it gave 10 of 80, so the tail is the host's. They are
  discrete levels rather than disturbance: each such run is flat
  across all 100 of its blocks, 1.04x to 1.59x the core, which is
  where the ring landed that process start. `suspended_s` is zero
  throughout and `inhibit` is on, so sleep is not in it. Whether the
  placement matters, two independent cores rather than an SMT pair,
  is untested, and the worst level at 1.59x has so far appeared only
  in polled runs, three of 160 against none of 80.
- **A record carries no environment grade** (2026-09-18): the gauge
  grades every run and the grade is display-only, so an analysis over
  records cannot filter disturbed runs by the tool's own judgement and
  has to infer them from the values. Noted where it cost work.
- **The agent's sandbox cannot pin the clock** (2026-09-17): `/sys`
  is read-only to its commands, so `--pin-freq` fails there with
  "Read-only file system", and pinned measurements on the 3900X are
  wink's to run. Over `ssh 7600x` the agent's commands are not
  sandboxed and the udev permissions apply, so it ran that host's
  pinned loop itself. Two more sandbox facts from the same day: a
  background command killed mid-run leaves zero-byte read-only stubs
  of protected dotfile names (`.bashrc`, `.gitconfig`, `.zshrc`, ...)
  in the working directory, which jj would commit and the agent
  cannot delete ("Device or resource busy"), so wink removes them;
  and `pkill -f` with a pattern that matches the ssh session's own
  command line kills that session.
- **Kept run records**: the clock experiment's 240 records are the
  first series tracked in the repo, `records/clock-shift/` (wink,
  2026-09-17), 2.9 MB, so the evidence travels with the finding. The
  7600x keeps its earlier ones in
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
- **Clock pins reach the child runs** (2026-09-15, at
  `feat: CI95 and LSC across processes`): the parent engages the pin
  and every child measures under it. Twenty runs on the 3900X pinned
  at 3801 MHz read 3.77 GHz each, and twenty on the 7600x at 4701 MHz
  read 4.67 GHz each, the report's per-run clock column showing one
  number where the clock held.
- **Pin pairs on these hosts** (2026-09-15): the 3900X's L3 groups are
  CPUs 0-2 and 3-5, so `--pin-cpus 2,3` straddles two CCXs and measures
  a cross-L3 round trip, about 380 ns on `zcr-mpsc-v1-2t` against about
  86 ns on `0,1`, and it does not speed up with a faster clock. Use
  `1,2` there for one CCX off CPU 0. On the 7600x `2,3` ran about 2 ns
  faster and tighter than `0,1`, which we think is CPU 0 carrying the
  kernel's housekeeping.
- **Agent-files diff**: `vc-x1 agent-files diff` compares against
  `../vc-x1-template`, stale since 2026-08-31, so use
  `vc-x1 agent-files diff ../vc-x1 -c` (0 of 11 on 2026-09-12).
- **Messages write guard**: list `open/` whole before every write
  to `../vc-x1-messages`, since a message once arrived between two
  reads of the threads being answered (m-5, 2026-09-12).
