# How the build moves a bench

A bench's level belongs to the binary that measured it, not to its source alone. This file holds
what showed it, from `feat: spsc v4 benches` (2026-09-30 and 10-01), and the rules the repo took
from it. The guide's [Results by placement](../docs/report-guide.md#results-by-placement) and the
README's Building section are the user-facing side.

## What moved `zcr-mpsc-v2-2t`

Trimmed means at `smt`, ten runs an invocation, builds alternated in rounds and compared with
`analyze --compare build`. The noise floor, one binary in two sweeps an hour apart, is a median
0.06% (3900X) and 0.08% (7600X).

| change | 3900X | 7600X |
|---|---|---|
| rebuild of the same inputs | 0, byte-identical | 0 |
| version string only | +0.79%, not seen | +1.49% |
| an edit off the hot path, 16 codegen units | +7.61% | +1.08% |
| the same edit, one codegen unit | +0.25% | -0.90% |
| the same edit, fat LTO | +4.69% | -0.47% |
| the same edit, one codegen unit and forced alignment | +0.97% | -1.57% |
| rustc 1.98.1 against 1.98.0, same source | - | +2.04% |

- Under 16 codegen units an edit anywhere can move a hot function into another unit, where
  nothing inlines across: the `analyze` fix, which touched no bench, took the bench's `step` out of
  `run_adaptive` (10,819 bytes became 729 and 9,230), and the round trip rose 7.6%.
- One codegen unit and fat LTO give two trees one code shape, so what remains is where the hot loop
  falls against a cache line. LTO's pair differed only in `step`'s alignment (mod 64: 16 against
  0) and read 4.7% apart.
- Across every bench, the old tree against the new, the median change was 1.47% (3900X) and 0.82%
  (7600X) at the defaults then, 0.65% and 0.56% with one codegen unit, and 0.39% and 0.13% with
  forced alignment, which cost nothing overall (a median +0.04%) though single benches moved up to
  8%. Zen 2 is the more sensitive host.
- One unchanged binary drifts 1 to 2% between sessions days apart (the 7600X's 44.89, 45.04, and
  45.88 ns), where it repeats to 0.1% within an hour.
- The same commit, rustc, and flags built different binaries on the two x86 hosts at `b6b6404`,
  `924e69da617b5928` and `48512fa4f131f48e`: the 3900X's carries std source paths under
  `~/.rustup/toolchains/stable-…/lib/rustlib/src`, which we think its `rust-src` component puts
  there. Earlier both used an auto-installed toolchain without it and matched.

## What the repo does about it

- `.cargo/config.toml` builds with one codegen unit, incremental off, and functions and branch
  targets aligned to 64 bytes. Its rustflags come after a host's own, and rustc takes the last
  `codegen-units`, so the repo's settings win over a host's `~/.cargo/config.toml`.
- No toolchain file: each host builds with what it has installed, kept in step by hand. An exact
  pin made every update a commit, and `stable` moved the build without a word.
- Every record and the banner name the binary by SHA-256 and its build's inputs, the commit,
  whether the tree was dirty, the profile, and the rustflags. Numbers compare only within one hash,
  and `analyze` warns when a comparison or a group spans more than one.
- A comparison between hosts runs one binary, built once and copied, since only the same hash
  makes it a comparison of hosts, and the same commit does not guarantee the same hash.
- A difference between two benches in one binary can still be layout, each loop placed apart, so a
  claim under a few percent between benches wants a `-nop` twin or several builds.

## The records

`records/layout-2026-09-30/` holds the `zcr-mpsc-v2-2t` comparisons tagged by `build`: `runs.jsonl`
(3900X, the old tree, the new tree, and the version-only build), `runs6.jsonl` (3900X, one codegen
unit and fat LTO for both trees), `runs-al.jsonl` (3900X, forced alignment), and the 7600X's
`runs-7600x.jsonl`, `runs8-7600x.jsonl` (eight builds, the 1.98.1 session binary among them), and
`runs-al-7600x.jsonl`. The four every-bench sweeps, about 28 MB, stayed out of the repo, in the
3900X's `tmp/layout/allbench*.jsonl` and the 7600X's `~/tmp/layout/`.
