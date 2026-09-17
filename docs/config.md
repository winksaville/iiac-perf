# Config file

Defaults, named pin profiles, and the box's declared frequency
steady state can live in a config file, so common invocations
don't repeat flags and the frequency commands have a way home.
Moved from the README (whose [Usage](../README.md#usage) points
here) and refreshed for the markdown carrier, the block knobs,
and the `[freq]` table.

## Carriers and precedence

Two carriers, one per directory. A `.md` config is a markdown
document whose ` ```toml ` fences, concatenated in document
order, are the config, so the prose between them documents the
file to its reader. `.md` is the recommended form. Plain `.toml`
stays accepted. A directory holding both is a hard error naming
both paths, because the one you edit could otherwise be the one
the loader ignores.

Precedence, lowest to highest:

- **built-in defaults**: `duration=5.0`, `band_labels=both`,
  `decimals=1`, `settle_time=1.5`, `warm_cap=1.5`,
  `block_sleep=1-10ms`, `block_warmup=0`, `blocks=100`,
  which makes a five-second run's blocks about 50 ms, each a
  replicate, and `runs=5` with `run_sleep=1-2s`, five fresh
  processes per bench, back to back, each after a sleep.
- **XDG file**: `$XDG_CONFIG_HOME/iiac-perf/config.md` (or
  `.toml`), falling back to `$HOME/.config/iiac-perf/` when
  `XDG_CONFIG_HOME` is unset. The per-user home for defaults,
  profiles, and the box's `[freq]` steady state.
- **project-local file**: the nearest `iiac-perf.md` (or
  `iiac-perf.toml`), the current directory first and then each
  parent up to the root. The search stops at the first found and
  merges no level above it, so a file high in a tree is what the
  directories below fall back to, never a layer under a nearer
  one, and a stray `~/iiac-perf.md` reaches only directories with
  no file of their own nearer. The `files` line names one found in
  a parent by its full path. It overrides the XDG
  file field by field, profiles merging by key and the `[freq]`
  table replacing whole (half of one box's declaration on top of
  half of another's would be a state nobody declared).
- **CLI flags**: always win.

`--config NAME` names the run's file, and changes what the two
files above give. The run keys then come from the named file and
the built-in defaults alone, so one file is one run on every host:
a host whose XDG file says `blocks = 10` no longer reaches a run
that another host's file leaves alone. The XDG and project-local
files still give `[freq]` and `[profiles]`, the host's own facts,
and the named file's own `[freq]` or profile, being the nearest,
wins over them. Flags still win over all of it.

A positional ending in `.md` or `.toml` is the same as `--config`
with it, so the common line needs no flag: `iiac-perf queue.md`.
No bench name ends that way, so the two never collide, and bench
names beside it win over the file's `benches`:
`iiac-perf queue.md min-now`. A bare `queue` stays a bench name,
since falling back to a config would turn a mistyped bench into a
file lookup, and `--config queue` is the form that completes the
extension. Two config files on a line, or one beside `--config`,
is an error.

An absolute NAME is taken as given. A relative one, `queue`,
`queue.md`, or `configs/queue`, is looked for in the current
directory, then each parent up to the root, then the XDG
directory, and the first found wins, as NAME, `NAME.md`, or
`NAME.toml`, both carriers in one place an error. So a tree of
bench directories shares a parent's config by name, and a nearer
file of the same name overrides it. Not found is an error listing
the places tried. Sharing is one file found from several
directories: a config does not include another.

The report's `Config:` list names the files that were loaded,
the highest priority first (or
`none (built-in defaults)`), then every run parameter with its
value and source: `(default)`, the file that set it, or the flag,
and `same as default` when a file or flag restates the built-in.
A present-but-malformed
file is a hard error rather than a silent fallback, so a typo
surfaces. Every key is optional, and
[`iiac-perf.example.md`](../iiac-perf.example.md) is the
starting config in the markdown carrier: every key commented out
at its default, explained between its fences.

`iiac-perf init-config` prints that file, and `iiac-perf
init-config PATH` writes it, and never over an existing file
unless asked: `--backup` replaces the file and keeps the old one as
`PATH.bak`, and `--overwrite` replaces it and keeps nothing. Either
way the old file's values are gone, where `update-config` keeps
them. A PATH
ending in `.toml` gets the TOML carrier: the section headings and
the keys, without the prose, since as comments the prose and the
commented keys look alike and a set key is lost among them. `setup --apply` creates a missing
XDG file from it too, with the host's live `[freq]` set.

`iiac-perf init-config --from OLD PATH` brings a file up to date.
It writes the starting config with every key OLD sets uncommented
at OLD's value, so a key added since arrives commented out, and a
key no longer known stops it with the key's name. OLD is not
touched, so a diff shows the change. Prose the author added to OLD
is not carried over.

Run flags on an `init-config` line set their keys in the new file,
so a command line that worked becomes a file:

```
iiac-perf init-config quick.md --benches min-now --blocks 10 -d 0.5s --pin-freq
iiac-perf --config quick
```

The new file is the run that line makes on this host. A plain
line runs on what the XDG and project-local files set, so with
neither `--from` nor `--config` those run keys are the start,
layered as the loader layers them, and the flags go over them. The
command names what it took, a line per file:

```
init-config: from iiac-perf.md: block_sleep, block_warmup
init-config: wrote quick.md
```

Without that start a file written from a line that worked can run
differently from the line, for want of a key the host's file was
quietly giving it. `[freq]` and `[profiles]` are not copied: they
are the host's, a run under the new file still gets them from the
host's files, and one host's clamp is wrong on the next. Defaults
stay commented out. `--config NAME` starts from a file found by
its search instead, as `--from OLD` starts from a path, the host's
files then left out, as a run under `--config` leaves them out.
The two together are an error, and `--from /dev/null` is the bare
template. `--benches` names the
benches, PATH being the one positional. Seconds are written as the
number they parse to, so `-d 0.5s` is `duration = 0.5`, a bare
`--pin-freq` is `"pin_mhz"`, `-d` clears a `total_duration` the
file gave, and a `--tag` joins the file's `[tags]`. A flag that is no run parameter,
`--apply` say, is refused by name, and the finished text is checked
as a load checks it before anything is written.

`iiac-perf update-config FILE` is the same fill written back over
FILE: its own values, and the line's run flags over them.

```
iiac-perf update-config queue.md --blocks 20 --runs 3
iiac-perf update-config queue.md --backup          # no flags: bring it up to date
```

FILE is a path as given and must exist, never a searched name,
since a file should not be rewritten because a search found it.
It is read, filled, and checked before it is touched, and the new
text is written beside it and renamed over it, so a stale key, a
bad value, or an interrupted run leaves FILE whole. Prose and
comments its author added are lost: `--backup` keeps the old file
as `FILE.bak`, replacing an earlier one, and without it the command
says so when there was something to lose.

## Keys

```toml
benches      = ["zcr-mpsc-v0-2t", "zcr-mpsc-v1-2t"] # run with no bench names; or one name, "all"
duration     = 10.0     # default -d, seconds or "250ms"
band_labels  = "zpn"    # zpn | frac | both
decimals     = 2        # 0-3
settle_time  = 3.0      # default --settle-time, seconds or "250ms"; 0 skips the warm
warm_cap     = 1.5      # default --warm-cap, seconds or "250ms"; 0 caps immediately
blocks       = 10       # default --blocks count, 1-1000; 100 when absent
runs         = 5        # default --runs, each run a fresh process, 1-1000
run_sleep    = "1-3s"   # default --run-sleep span before each run; 1-2s when absent
block_sleep  = "1-10ms" # default --block-sleep span; 0 = partitions
block_warmup = "2ms"    # default --block-warmup; 0 records post-wake calls
pin_freq     = "min_mhz" # pin every run: MHz, "pin_mhz", "min_mhz", "max_mhz", or "no"
# total_duration = "60s" # default -D, split over every run; a file sets this or duration
samples      = 100000   # default --samples; auto-sized when absent
inner        = 1        # default --inner; auto-sized when absent
pin_cpus     = "0,1"    # default --pin-cpus: a CPU spec or a [profiles] name
record       = "records/" # default --record; relative to the current directory
env_probe    = true     # false is --no-env-probe
inhibit      = true     # false is --no-inhibit
ticks        = false    # true is --ticks
verbose      = false    # true is --verbose

[profiles]              # named --pin-cpus CPU specs
smt = "0,12"           # SMT siblings of one physical core (contention)
ccx = "0,1"            # independent cores, same CCX (best channel latency)
ccd = "0,6"            # cross-CCD

[tags]                  # each a --tag KEY=VALUE on every record; needs a record
experiment = "clock-shift"
```

Every run parameter has a key, so a file can say what a command line can. The words that say
what to do rather than how a run is shaped have none: `--print-only`, `--as-config`, `--apply`,
`--uninstall`, and `--list-benches`.

- `duration` and `total_duration` are one choice. A file sets one of them, and the nearer
  file's choice clears the other.
- `[tags]` merges by key across the files, as `[profiles]` does. A `--tag` on the line adds to
  the table and wins on a shared key. A tag with no record is an error.
- An on/off key is undone from the line by giving the flag a value: `--verbose=no`,
  `--ticks=no`, `--no-env-probe=no`, `--no-inhibit=no`. The bare flag means `yes`.
- `pin_cpus`, `record`, `samples`, and `inner` have no such undo: a run that wants none of a
  file's value runs without that file.

## The host: the [freq] steady state

The `[freq]` table declares the box's steady state: what
`restore-freq` converges to, from any starting point, and what a
pinned run (`--pin-freq`, `pin-freq`, `suggest-freq`) restores on
exit. Declared once by you rather than remembered from before a
pin, because a remembered state ratchets on back-to-back runs.
It normally lives in the XDG config, the steady state being the
box's rather than the project's.

`iiac-perf setup` writes it for you: it prints the declaration
it would add to `~/.config/iiac-perf/config.md` from the live
state, clamp limits included, and `iiac-perf setup --apply`
writes it. A missing file is created, a file without `[freq]`
gains the section at its end, and a file that already declares
`[freq]` is left alone and checked against the box: against its
ranges, and against the state it runs at, naming every declared
value the host does not hold, since a declaration copied from
another host can fit the ranges and still be wrong. It refuses
to write a declaration that would not pass the pin and restore
checks, a pinned clamp at setup time among them, and it runs as
your user, not under sudo, since the file belongs under your
home.

`setup` also removes the need for sudo. It prints a udev rule,
`/etc/udev/rules.d/70-iiac-perf.rules`, that hands you ownership
of the cpufreq files a pin or restore writes (each CPU's
governor, EPP, boost, and clamp files, and the global boost) and
of `/dev/cpu_dma_latency`, on every boot and CPU hotplug, and
the root script that installs it and takes ownership now.
`setup --apply` runs that script through one `sudo`, after which
`pin-freq`, `restore-freq`, `--pin-freq`, and `suggest-freq` run
as you, reading your own config. `setup --uninstall` prints the
removal, and `setup --uninstall --apply` removes the rule and
gives the files back to root. The grant is the point and also
the cost: any process you run can then move this box's clock.

`iiac-perf read-freq --as-config` prints the current state in
the same form, ready to paste into a `toml` fence:

```toml
[freq]
governor = "powersave"
epp      = "balance_performance"  # required exactly when the box has EPP
boost    = true                   # required exactly when the box has a boost knob
min_mhz  = 1745                   # the live clamp's floor, required
max_mhz  = 4673                   # the live clamp's ceiling, required
pin_mhz  = 3801                   # pin target; omitted = the discovered base clock
```

A knob the box exposes must be declared (restoring around it
would leave a pin's residue), and a knob the box lacks must not
be. `min_mhz` and `max_mhz` are required on every box: a restore
without them would fall to the hardware range, far below the
clamp the box runs at (the 7600x once dropped from 2.99 GHz to
427 MHz that way), so every command that pins or restores refuses
a declaration missing either. They must fit each CPU's hardware
range, and on a driver with a fixed frequency list, be in that
list. On amd-pstate the upper end of that range is the boosted
ceiling, `amd_pstate_max_freq`, since `cpuinfo_max_freq` falls to
the nominal frequency while a pin holds boost off. `min_mhz = max_mhz` is allowed, a steady state that holds
the clock at one frequency with boost as declared, and every
restore then returns there. `read-freq --as-config` prints the
limits from the live clamp, and comments them out when the clamp
is `min = max`, since from the live state alone a pin still
running looks the same.
`suggest-freq` measures the best `pin_mhz` for a workload
and ends with the line to paste.

Every restore says where the clock went back to: `restore-freq`,
and a pinned run or `suggest-freq` as it exits, print
`freq: restored the [freq] from <file>:` and the state read back
from the CPU, governor, EPP, boost, and clamp, once every file the
restore wrote reads back what was written. The line leaves out the
live average: read just after a run, it is an idle core at the
bottom of its clamp, whatever the run was pinned at. The kernel can apply a limit change after the write
returns, so the report waits up to a second for that, and says
`not settled` naming the file when it has not. A restore on Ctrl-C
or SIGTERM prints the declared
values instead, marked as not read back, since a signal handler
cannot read the state. `pin-freq` names the file its later
`restore-freq` will use from the same directory. The report's
`Config:` list shows the declared table and its file on a `freq`
line.

## A run: pin_freq

The `[freq]` table above describes the host, and `pin_freq` describes
a run: whether it pins the clock, and where. It is the config twin of
`--pin-freq`, and both take the same values:

| Value | Pins at |
|---|---|
| a number, `3801` | that many MHz |
| `"pin_mhz"`, and bare `--pin-freq` | the `[freq]` table's `pin_mhz`, else the base clock |
| `"min_mhz"` | the `[freq]` table's `min_mhz` |
| `"max_mhz"` | the `[freq]` table's `max_mhz` |
| `"no"` | nothing |

A word names the host's own `[freq]` value, so a benchmark
directory's config that says `pin_freq = "min_mhz"` pins each host at
its own floor and moves between hosts unchanged.

Every target must fit under the ceiling with boost off, since a pin
turns boost off. On amd-pstate that is the nominal frequency, the
`base` `read-freq` shows (3801 MHz on a 3900X), so a `max_mhz` of
4673, which needs boost, is refused as a pin target with the reason,
while `max_mhz = 3801` pins. The same check refuses a number above
it, which would otherwise pass while boost is still on and then be
capped by the kernel once the pin turns it off.

Where a value is written decides how long it lasts. In a file it
holds until the file changes, and `"no"` there is the same as leaving
the key out, so a lower file's `pin_freq` still applies. On the line
it lasts one run, and `--pin-freq=no` is the way to run once without a
file's pin. Each pinned run pins before its warmup and restores the
host's `[freq]` steady state when it exits, saying where the clock
went back to. `suggest-freq` pins for itself and never takes a
config's pin.

This is how a benchmark directory holds the clock: its `iiac-perf.md`
sets `pin_freq`, and the host's `[freq]` stays in the XDG config, so a
restore never depends on the directory. Declaring `min_mhz = max_mhz`
in a project-local `[freq]` also holds the clock, but it moves where
every restore from that directory returns, and a `restore-freq` there
leaves the host pinned after you leave.
