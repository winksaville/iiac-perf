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
  `block_sleep=1-10ms`, `block_warmup=0`, and `blocks=100`,
  which makes a five-second run's blocks about 50 ms, each a
  replicate.
- **XDG file**: `$XDG_CONFIG_HOME/iiac-perf/config.md` (or
  `.toml`), falling back to `$HOME/.config/iiac-perf/` when
  `XDG_CONFIG_HOME` is unset. The per-user home for defaults,
  profiles, and the box's `[freq]` steady state.
- **project-local file**: `iiac-perf.md` (or `iiac-perf.toml`)
  in the current directory (no upward walk). It overrides the XDG
  file field by field, profiles merging by key and the `[freq]`
  table replacing whole (half of one box's declaration on top of
  half of another's would be a state nobody declared).
- **CLI flags**: always win.

The report's `Config:` list names the files that were loaded (or
`none (built-in defaults)`), then every run parameter with its
value and source: `(default)`, the file that set it, or the flag,
and `same as default` when a file or flag restates the built-in.
A present-but-malformed
file is a hard error rather than a silent fallback, so a typo
surfaces. Every key is optional, and
[`iiac-perf.example.md`](../iiac-perf.example.md) is a
ready-to-copy sample in the markdown carrier, explaining each key
between its fences.

## Keys

```toml
duration     = 10.0     # default -d seconds
band_labels  = "zpn"    # zpn | frac | both
decimals     = 2        # 0-3
settle_time  = 3.0      # default --settle-time seconds; 0 skips the warm
warm_cap     = 1.5      # default --warm-cap seconds; 0 caps immediately
blocks       = 10       # default --blocks count, 1-1000; 100 when absent
block_sleep  = "1-10ms" # default --block-sleep span; 0 = partitions
block_warmup = "2ms"    # default --block-warmup; 0 records post-wake calls

[profiles]              # named --pin-cpus CPU specs
smt = "0,12"           # SMT siblings of one physical core (contention)
ccx = "0,1"            # independent cores, same CCX (best channel latency)
ccd = "0,6"            # cross-CCD
```

## The [freq] steady state

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
from the CPU, once every file the restore wrote reads back what was
written. The kernel can apply a limit change after the write
returns, so the report waits up to a second for that, and says
`not settled` naming the file when it has not. A restore on Ctrl-C
or SIGTERM prints the declared
values instead, marked as not read back, since a signal handler
cannot read the state. `pin-freq` names the file its later
`restore-freq` will use from the same directory. The report's
`Config:` list shows the declared table and its file on a `freq`
line.
