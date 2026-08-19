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
file to its reader. `.md` is the recommended form; plain `.toml`
stays accepted. A directory holding both is a hard error naming
both paths, because the one you edit could otherwise be the one
the loader ignores.

Precedence, lowest to highest:

- **built-in defaults**: `duration=5.0`, `band_labels=both`,
  `decimals=1`, `settle_time=1.5`, `warm_cap=1.5`,
  `block_sleep=0`, `block_warmup=0`;
- **XDG file**: `$XDG_CONFIG_HOME/iiac-perf/config.md` (or
  `.toml`), falling back to `$HOME/.config/iiac-perf/` when
  `XDG_CONFIG_HOME` is unset; the per-user home for defaults,
  profiles, and the box's `[freq]` steady state;
- **project-local file**: `iiac-perf.md` (or `iiac-perf.toml`)
  in the current directory (no upward walk); overrides the XDG
  file field by field, profiles merging by key and the `[freq]`
  table replacing whole (half of one box's declaration on top of
  half of another's would be a state nobody declared);
- **CLI flags**: always win.

The startup banner's `config` line names the files that were
loaded (or `none (built-in defaults)`). A present-but-malformed
file is a hard error rather than a silent fallback, so a typo
surfaces. Every key is optional;
[`iiac-perf.toml.example`](../iiac-perf.toml.example) is a
ready-to-copy sample.

## Keys

```toml
duration     = 10.0     # default -d seconds
band_labels  = "zpn"    # zpn | frac | both
decimals     = 2        # 0-3
settle_time  = 3.0      # default --settle-time seconds; 0 skips the warm
warm_cap     = 1.5      # default --warm-cap seconds; 0 caps immediately
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

`iiac-perf read-freq --as-config` prints the current state in
this form, ready to paste into a `toml` fence:

```toml
[freq]
governor = "powersave"
epp      = "balance_performance"  # required exactly when the box has EPP
boost    = true                   # required exactly when the box has a boost knob
# min_mhz / max_mhz omitted: the hardware range
pin_mhz  = 3801                   # pin target; omitted = the discovered base clock
```

A knob the box exposes must be declared (restoring around it
would leave a pin's residue), and a knob the box lacks must not
be. `suggest-freq` measures the best `pin_mhz` for a workload
and ends with the line to paste.
