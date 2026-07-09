# Chores 04

Continuation of [chores-03](chores-03.md). Records landed work;
conventions in [AGENTS.md](../../AGENTS.md#chores-conventions) and
[cycle-protocol.md](../cycle-protocol.md#chores-sections).

## feat: zcr bench family (raw/with/spin, 1t/2t)

Commits: [[1]],[[2]],[[3]],[[4]],[[5]],[[6]]

`../zc-ring-x1` exposes three API tiers per endpoint — raw
`reserve_slot` (caller handles Full/Empty), `reserve_slot_with`
(injected wait-policy closure), and `reserve_slot_spin` (built-in
spin policy). Its docs claim the no-wait fast path does the same
loads at every tier; add six benches `zcr-{raw,with,spin}-{1t,2t}`
to verify the tiers perform basically the same.

### As-built ladder

- 0.13.0-0 `chore: adopt cycle protocol + open zcr cycle`
- 0.13.0-1 `feat: zcr-raw-1t/2t ring benches`
- 0.13.0-2 `feat: zcr-with-1t/2t ring benches`
- 0.13.0-3 `feat: zcr-spin-1t/2t ring benches`
- 0.13.0-4 `docs: zcr tier comparison results`
- 0.13.0 `feat: zcr bench family (raw/with/spin, 1t/2t)` —
  close-out

The cycle was planned as 0.12.0 but renumbered at `-0`: the
session started from a working copy based on 0.11.0, unaware
the aarch64 ticks cycle (0.12.0, pushed from another machine)
already held `main`. The `-0` commit was rebased onto it —
`jj new main` → `vc-x1 sync` (no-op, already fetched) →
`jj rebase -r <prep> -d main` → conflict resolution (Cargo
version line; union-merge of `.claude/settings.local.json`) —
and the cycle renumbered 0.12.0 → 0.13.0.

### Tier comparison (0.13.0-4)

Method: installed `iiac-perf 0.13.0-4`, Ryzen 9 3900X, idle
desktop, `iiac-perf zcr mpsc-2t-spin ice-ps-2t -d 5 --pin 0,1`;
values are the adjusted trimmed mean (min-p99). Run 2 repeats
the 2t trio to check repeatability.

| bench        | run 1  | run 2  |
|--------------|-------:|-------:|
| zcr-raw-1t   |   4 ns |      — |
| zcr-with-1t  |   4 ns |      — |
| zcr-spin-1t  |   5 ns |      — |
| zcr-raw-2t   | 119 ns | 120 ns |
| zcr-with-2t  | 101 ns | 101 ns |
| zcr-spin-2t  | 101 ns |  99 ns |
| mpsc-2t-spin | 117 ns |      — |
| ice-ps-2t    | 656 ns |      — |

Observations:

- 1t: all three tiers are identical within noise (4-5 ns).
  The no-wait fast path costs the same at every tier, matching
  the zc-ring-x1 doc claim that `_with`'s fast path does
  exactly the loads `reserve_slot` does.
- 2t: `with` and `spin` are identical (~100 ns) — expected,
  `reserve_slot_spin` *is* `reserve_slot_with(policy::spin)`.
  `raw` is consistently ~20 ns slower (119-120 ns) across
  runs.
- We think the raw 2t penalty is in the waiting path, not the
  fast path: the hand-written retry loop re-enters
  `reserve_slot` on every failed attempt — re-running the type
  check and re-loading the caller's own index each time — while
  `_with` hoists the own-index load out of the loop and re-reads
  only the peer index per attempt, so it reacts faster when the
  peer's index store lands.
- Transport context: the zcr 2t rows (~100-120 ns) sit at or
  under `mpsc-2t-spin` (117 ns) and ~6x under `ice-ps-2t`
  (656 ns) at the same wait policy.

Net: the ergonomic tiers cost nothing over raw — and under
cross-core traffic the DIY retry loop is the *slowest* way to
wait.

## fix: saturate hist records, flag suspended runs

Commits: [[7]]

The harness panicked at `hist.record().unwrap()` with
`ValueOutOfRangeResizeDisabled` when the desktop idle-suspended
during long runs: a sample that spans a suspend measures the
whole sleep gap, and at `inner=1` the raw gap exceeded the
histogram's 60 s high bound (journal `PM: suspend exit`
timestamps match both observed panics to the second). A
sub-bound gap is worse — divided down by `inner`, it records
silently and poisons `max` and the untrimmed mean/stdev
untraced (percentile bands and trimmed stats survive: a few
inflated samples out of millions land in the extreme tail).

- `saturating_record` replaces `record().unwrap()`: a sample
  above the bound clamps at 60 s instead of panicking the run;
  a clamp pileup stays visible in the `max` column.
- Suspend detection (`ClockPair`): capture `CLOCK_MONOTONIC` +
  `CLOCK_BOOTTIME` at run start; MONOTONIC freezes during
  suspend, BOOTTIME keeps counting, so their elapsed divergence
  is the suspended time. At ≥1 s a `WARNING` naming the bench
  and gap prints as the report's last line — after the table, so
  it can't scroll out of mind; a clamped `max` prints a second
  `WARNING` (covers a wedged sample with no suspend).
- The 60 s bound is a sane-world ceiling, not a type limit
  (u64 ns holds ~584 years); recording the "true" hours-long
  value would only distort mean/stdev further, so clamp + flag
  beats raising the bound.
- We think minstant's TSC keeps counting across s2idle, which
  is why the sleep gap appears in samples at all — detection
  therefore uses std `Instant`, not minstant.

## fix: report column alignment

Commits: [[8]]

Pre-existing alignment problems in `print_report` — two bugs
from format widths being minimums that wide content silently
overflows, plus header justification that read poorly:

- Column widths were computed from the band rows only, so a
  summary value wider than any band mean — typically the
  untrimmed stdev after a tail outlier — overflowed its field
  and shifted its line right. Summary values (whole-histogram
  and trimmed mean/stdev) are now rendered before the width
  pass and included in it.
- The `adjusted` header (8 chars) spans mean's ` ns` + gap +
  adj_w = 7 + adj_w header columns, so single-digit adjusted
  values (adj_w = 1) left the label flush against `mean`
  (`meanadjusted`), and even 3-digit values gave only 2 spaces.
  adj_w now floors at 5, keeping the full 4-space gap between
  the two headers that every other column pair gets.
- Headers right-justified to the digit end of their column,
  3 characters left of where the column visually ends (` ns`).
  Each label now right-justifies to the last character of its
  column's unit; unitless `count` still aligns to its digits.

## feat: finer report tail bands

Commits: [[9]]

The p99-max band lumped the top 1% of samples into one row —
~3M samples on a 5-minute 2t run — hiding tail structure and
letting a single outlier (e.g. a suspend-inflated sample)
dictate the band's `last` and `mean`. New p99.9 / p99.99 /
p99.999 boundaries split the tail four ways: rare outliers
isolate in `p99.999-max` while the lower tail bands show the
genuine scheduler-preemption structure.

- The trimmed mean/stdev stay anchored at the p99 boundary
  (label `min-p99` unchanged): the trim excludes every band at
  or above p99, independent of how many finer bands subdivide
  the tail.
- Empty bands were already skipped in the output, so runs with
  few samples print only the tail rows they can populate.

## feat: inhibit sleep during bench runs

Commits: [[11]]

Idle-suspend poisoned two long runs before the suspend detector
existed; detection flags a poisoned run but prevention is
better. The binary now re-execs itself under
`systemd-inhibit --what=sleep` before any output, so the machine
can't idle-suspend while a bench runs.

- Guard env var (`IIAC_PERF_INHIBITED`) marks the re-exec'd
  child; the wrapper holds the inhibitor lock for the child's
  lifetime and releases it on exit, crash included.
- `--no-inhibit` opts out: for strace/gdb/perf wrappers (the
  re-exec inserts `systemd-inhibit` into the exec chain), to
  let the machine sleep on purpose, or to test the
  suspend-detection path — a sleep inhibitor also blocks
  manual `systemctl suspend`.
- Where `systemd-inhibit` is unavailable (non-systemd box) the
  run continues uninhibited; the banner's new `sleep inhibit`
  line reports active / disabled / unavailable, and the
  suspend detector remains the backstop.
- Bench listing (no args) and `--help`/`-V` don't re-exec.

## feat: nines/zeros tail bands (z4..n10)

Commits: [[12]]

Exploring the tail needs bands beyond p99.999; percentile names
grow a digit per decade (`p99.99999999`), and bare exceedance
fractions (`1e-4`) scale but don't read as percentiles. Both
tails now use nines/zeros notation — `nK`/`zK` mark the boundary
with a fraction 10^-K of samples above (n) / below (z), so
n2 ≡ p99, n3 ≡ p99.9, … n10 and z2 ≡ p1, z3, z4 — while the body
keeps familiar deciles. "K nines" is standard engineering
shorthand (nines = −log10(1−x)) [[10]]; zK is our mirror of it
for the fast tail.

- Slow tail subdivides n2..n10; fast tail only z4..z2 — a
  latency distribution is floored below (nothing beats the
  fast path), open above.
- Rows are labeled by their upper boundary alone (`z3`,
  `p50`, `n4`); the lower boundary is the previous printed
  row, so skipped empty bands read correctly — a row's count
  is "samples since the last printed boundary".
- Trimmed stats renamed `min-p99` → `min..n2`; still exclude
  every band at or above the n2 boundary.
- Deep bands populate as run length earns them (n10 needs
  ~1e10 calls, ~20 min at 110 ns); empty bands already skip.

## fix: number todo entries per AGENTS todo format

Commits: [[13]]

`notes/todo.md` predates the vc-template-x1 todo header and the
AGENTS.md [Todo format] numbering rule: entries were `-`
bullets, so `vc-x1 validate-todo` recognized 0 entries and the
convention was never enforced (nothing to check, nothing to
flag).

- Port the template's section intros — the In Progress ladder
  note and the Todo strict-priority-rank note — adapted: this
  repo has no `todo-backlog.md`, so that sentence is dropped.
- Number the 20 `## Todo` entries in current order (rank
  preserved); `## Done` keeps `-` bullets per convention.
  `fix-todo`/`validate-todo` now see all 20 entries.
- AGENTS.md "Example shape": number the `# Todo` example
  entries so the example matches the normative text four
  paragraphs above it.
- Drop the stale "Foramt details" pointer (typo'd, and aimed at
  a README anchor that doesn't exist).

## feat: report options + ps recording

Commits: [[14]],[[15]],[[16]],[[17]],[[18]]

Band-label style, decimal display, and picosecond recording —
evaluated as CLI options ahead of the config file (options
first so we could see how they look; persistence afterwards).
Decimals needed ps recording to be uniform: mean/adjusted were
f64 already, but first/last/range were integer-ns recorded
values until the recording unit became ps.

- `--band-labels zpn|frac|both` (default both): labels and
  boundary fractions generated in the new `src/bands.rs` from
  one structural description (Z_DEPTH/N_DEPTH + deciles), so
  the styles can't drift; README's ladder table is pinned by
  the module's test. Header records `labels=<style>`.
- Picosecond recording: dividing a sample by `inner` in ps
  keeps true sub-ns per-call precision — zcr-spin-1t's body
  resolved into 4.8/5.2/5.5/5.9 ns rows that were two integer
  bins, and its trimmed adjusted mean dropped 5 → 4.6 ns
  (ns-rounding inflation removed).
- `--decimals N` (0–3, default 1) controls time-column digits;
  0 restores integers, 3 is the recording floor.

### As-built ladder

- 0.14.0-0 `chore: open report options cycle`
- 0.14.0-1 `feat: report option --band-labels`
- 0.14.0-2 `feat: report picosecond recording`
- 0.14.0-3 `feat: report option --decimals`
- 0.14.0 `feat: report options + ps recording` — close-out

The -2/-3 order was swapped from the original plan (decimals
were to precede ps) so the display default could be designed
against the real post-ps precision; the 1-digit default then
landed in -2 itself to make the ps gain visible immediately.

## feat: config file + pin profiles

Commits: [[19]]

Defaults and named pin profiles now come from a layered config —
built-in defaults < `$XDG_CONFIG_HOME/iiac-perf/config.toml` <
project-local `iiac-perf.toml` (cwd, no upward walk) < CLI. The
config is where the report options that landed the prior cycle
(`--band-labels`, `--decimals`) and `--duration` get their
persisted defaults, so common invocations stop repeating flags.

- `src/config.rs`: a `RawConfig` deserialize target (serde +
  toml, `deny_unknown_fields` so a typo is caught), overlaid
  XDG-then-local into a validated `Config` of `Option` scalars
  (`None` = "no opinion, use built-in") plus a flat profiles
  map. A present-but-malformed file is fatal, not a silent
  fallback.
- `--pin <name>` resolves a `[profiles]` entry to its core spec
  before parsing (`smt = "0,12"` ⇒ `--pin smt` is `--pin 0,12`);
  a non-profile value parses as raw cores, so existing specs are
  unaffected.
- `main` resolves precedence per field (`cli.x.or(config.x)
  .unwrap_or(built_in)`); `--band-labels`/`--decimals` dropped
  their clap `default_value_t` and became `Option` so
  CLI-vs-config-vs-default is distinguishable.
- Banner gains a `config` line naming the loaded files (or
  `none (built-in defaults)`).
- `iiac-perf.toml.example`: a ready-to-copy sample at every key's
  built-in default, documenting each key's meaning and possible
  values; `.example` suffix so it is never auto-loaded from cwd.
  README's Config file section links it.

### Why band_labels maps by string, not serde on the enum

`BandLabels` lives in `bands.rs` as a `clap::ValueEnum`; the
config deserializes it as a plain string and maps it in
`validate`, keeping serde out of `bands.rs`. The mapping is the
same three names clap already accepts, and a bad value gets a
config-specific error (`"nope" is not one of zpn, frac, both`)
rather than a serde enum-variant message.

## refactor: drop zcr raw/spin bench tiers

Commits: [[20]]

zc-ring-x1 simplified its API down to a single `reserve_slot_with`
per endpoint, removing `reserve_slot` and `reserve_slot_spin`. The
`zcr-raw-*` and `zcr-spin-*` benches only existed to measure those
two now-gone tiers, so they go with them; the `zcr-with-*` pair is
the whole zcr family now.

- Removed `zcr_raw_1t/2t.rs` + `zcr_spin_1t/2t.rs` and their
  `mod.rs` `pub mod` + `REGISTRY` entries; `zcr_common.rs` stays
  (shared by the `with` benches).
- Reworded the `zcr-with-*` doc comments that cross-referenced the
  deleted benches, and dropped the four rows + "three API tiers"
  phrasing from README's results table.
- Switched the `zc-ring-x1` dep from the local `../zc-ring-x1`
  path to the GitHub git source (Cargo.lock pins the resolved
  commit); trivially revertible to path.

The raw-tier finding (looping the fallible single-shot
`reserve_slot` re-reads the caller-owned index each spin → ~+30 ns
vs `with`/`spin`) stays recorded in the pinned tier comparison
above and this file's `zcr bench family` section — the benches are
gone but the measurement is preserved in prose.

## fix: trim label spans populated bands

Commits: [[21]]

The trimmed-stat summary rows (`mean`/`stdev`) were labeled with a
fixed `min..n2`. But band rows are named by their **upper** boundary,
so `min` never prints as a row, and the `n2` band can be empty (spiky
data with nothing in p90..p99) — the label asserted two rows that may
not exist. Now both ends are derived from the actual populated bands
within the trim range (indices `0..trim_bands`, the bands below the
n2 ≡ p99 tail cut), each named by its upper boundary like the rows
above.

- The lower end is the first populated non-tail band, the upper end
  the last; a run with the low tail empty reads `p20..n2`, one with
  the `n2` band empty reads `..p90`, and a single populated band
  collapses to one name (`p60`, not `p60..p60`).
- The trim *computation* is unchanged — still every band below the
  n2 cut. Only the label tracks the real extent instead of the fixed
  boundary pair.
- `BandLabels::trim_label` (the old static `min..n2` / `min..0.99`)
  is gone; a new `Boundary::trim_name` returns the bare style-name
  (`both` reuses `zpn`, as the trim label always did), and
  `harness::trim_range_label` assembles the range — unit-tested for
  the full-range, empty-`n2`, single-band, and no-samples cases.
- The sibling probe report path (`probe.rs`, used by the `tp`/`tp2`
  tprobe benches) had the same fixed `min-p99` label; it got the
  same fix, adapted to its `low-high` row convention — a
  `trim_range_label(band_count)` there reads `min-p99` normally,
  `p10-p99` / `min-p90` when an end band is empty, with its own
  unit test. The two report paths stay independent (the probe
  ladder is slated to unify onto `bands.rs` under the probe-based
  harness conversion todo).

### README example refresh

The `## Example runs` blocks had drifted badly — the `mpsc-2t -v`
and default-vs-`--pin` examples still showed 0.7.0-era output: the
old `min-p1 … p99-max` decile ladder and `min-p99` trim label,
predating both the nines/zeros `z4..n10` ladder and this cycle's
label change. We think stale example output is worse than none —
a reader copies the shape and expects it.

- Re-ran `mpsc-2t` (default, `--pin 0,1`, `-v`) on the 3900X and
  replaced the blocks with current-format output; the `--pin`
  Δ table and its prose move from `min-p99` to the live `z4..n2`
  rows (mean −10 %, stdev −25 %; the untrimmed stdev is wider
  pinned, 37 µs vs 10 µs, from a lone ms-scale outlier).
- Added a `### Label styles (--band-labels)` subsection showing
  `min-now` under `both` / `zpn` / `frac`, which doubles as a live
  demo of the new trim label — the runs have no `min` row, so it
  reads `p20..n2` (`0.20..0.99` in frac), exactly the case the old
  fixed label got wrong.

## fix: upper-closed band intervals

Commits: [[22]]

A single-sample run (`iiac-perf zcr -d 0.0000001`) put its lone
value in `p60`, not the `p50` a median reads as. The band-membership
test was half-open the *low* way — `[lower, upper)` via strict
`mid_rank < boundary` — so a rank landing exactly on a boundary fell
into the band that boundary *opens* rather than the one it *caps*.
Flip to right-closed `(lower, upper]` (`mid_rank <= boundary`) so a
boundary-exact rank counts in the band its label names. (The `-d`
that lands a single sample is machine-dependent — tune it to the
sample count you want; there are no timing guarantees.)

- `harness.rs` and `probe.rs` each grew a `band_index` helper (the
  membership test was duplicated at two sites per file — the band
  histogram pass and the trimmed-variance pass); both now call it, so
  the `<` → `<=` change lives in one place per file, each with a unit
  test pinning the boundary-exact cases.
- Only ranks landing *exactly* on a boundary move (down one band, into
  the capped band); every strictly-interior rank is unchanged, so
  normal multi-sample runs are unaffected — visible only in
  tiny/degenerate runs.

### Band membership

The mechanics — the Hazen `mid_rank = (i − 0.5) / n` formula, the
right-closed `(lower, upper]` convention (with the open/closed
definition and why it fits this report's upper-boundary labels), the
worked 10-value and single-sample tables, and the
interval-convention citations (Hazen, pandas, numpy, Dijkstra
EWD831) — live in the README's
[Reading a report](/README.md#reading-a-report) section, the single
source of truth. This note records only the decision to adopt
right-closed bands so a boundary-exact rank (a lone median sample)
reads `p50`; see the README for the full explanation.

## docs: add "Reading a report" to README

Commits: [[23]]

The report format was described under `## Usage`, but there was no
practical "how to read a row and poke at it" guide, and the
band-membership mechanics (the Hazen rank formula, the right-closed
interval convention, the citations) lived only in the
upper-closed-intervals chores note above — history, not where a user
looks. Promote them to a user-facing README section and make that the
single source of truth.

- New README `### Reading a report` (top of `## Example runs`): a
  column key, the whole-histogram-vs-trimmed distinction, the
  `mid_rank` → band mapping (Hazen formula + right-closed
  `(lower, upper]` with an explicit open/closed definition), the
  worked 10-value and single-sample tables, the interval-convention
  citations as inline links (Hazen 1914, pandas.cut, numpy.histogram,
  Dijkstra EWD831), and the `-d` investigation technique with real
  few-sample / single-sample output.
- The `### Band membership` note is trimmed to a pointer at that
  README section (SSOT); its four interval refs move to README inline
  links and are pruned here.
- A machine-dependence caveat on the `-d` reproducer rounds it out —
  the value that lands N samples is timing-specific.

## feat: zcr-mpsc-1t/2t benches

Commits:

zc-ring-x1 0.11.0 grew an MPSC sibling ring (CAS-claimed
producer index + per-slot seq array, closure `send_with`);
these benches are the A/B its design's measurement plan calls
for, mirroring the zcr-with pair.

- `zcr-mpsc-1t` — same-thread round-trip: the uncontended
  claim CAS + seq publish, against `zcr-with-1t`'s
  load/store-only SPSC pair.
- `zcr-mpsc-2t` — main → echo worker → main over two MPSC
  rings (one producer each), the same shape as `zcr-with-2t`:
  the "MPSC when you don't need it" number.
- `zcr_common` gained `leak_mpsc_ring()`; the `zcr` CLI
  prefix now resolves to all four zcr benches.
- First 300s numbers (recorded in zc-ring-x1's README and
  chores): mpsc-1t 4.4 ns vs with-1t 2.3 ns adjusted — and at
  2t the sign flips, mpsc 73.9 ns vs spsc 100.1 ns. We think
  SPSC bounces two index cache lines per handoff while MPSC's
  only shared hot word is the slot seq; the exploration is
  tracked in zc-ring-x1's todo.
- Cargo.lock: the zc-ring-x1 git dep advances to 0.11.1 (the
  MPSC release + backfill tip).

## docs: add notes/design.md (calibration accuracy)

Commits:

Repeated runs showed framing/sample jumping 1-21 ns while
loop/iter held steady — traced to ~10 ns TSC quantization of
the un-amortized `min_low` measurement, not sampling noise.
The analysis and the resulting design (amortized framing
measurement + cached calibration in the config file) are too
durable for a chores section, so this cycle opens
[notes/design.md](../design.md) as the home for
measurement-theory / error-model analyses, with this as its
first entry
([Calibration accuracy](../design.md#calibration-accuracy-framing-quantization)).

- Key findings recorded there: the min estimator can't
  resolve inside a quantum (true framing ∈ ~[1, 11] ns); the
  framing estimate sizes `inner` via `pick_inner`, so an
  under-read under-sizes the experiment (worst case ~50%
  apparatus contamination, invisible in the report); the TSC
  quantum is frequency-invariant but the framing *cost* is
  core-clocked.
- Design direction: amortized framing measurement (M timer
  pairs in one window, error q/M), constants cached in the
  config file with provenance and a cheap live validity
  check each run; duration-scaled calibration rejected.
- design.md also opens with an `## Architecture Overview` —
  the four layers (startup / environment control,
  measurement, benches, reporting), the two measurement
  styles (harness-driven `Bench` vs self-driven probes), and
  the bench-family map — so the error-model sections have a
  structural map to hang off.
- notes/README.md gains a pointer to design.md.

## refactor: move chores-01..03 into notes/chores/

Commits:

chores-04 opened the `notes/chores/` directory; the three
older chores files move in beside it so the family lives in
one place. File moves only — anchors are unchanged, so every
`/notes/chores-0N.md#anchor` reference rewrites to
`/notes/chores/chores-0N.md#anchor` mechanically.

- Reference rewrites: todo.md refs `[15]`-`[39]`, done.md
  refs `[2]`-`[14]`, and the moved files' sibling-relative
  links (`todo.md`, `README.md`, `ideas.md` gain `../`).
- jj-tips.md's four "full details" links were doubly broken
  (a stale `./notes/` prefix *and* the wrong file — the
  sections live in vc-notes.md, not chores-01); they now
  point at vc-notes.md directly.
- Prose examples (AGENTS.md ref-numbering example, the
  notes/README.md format snippets) and historical
  commit-body bullets inside chores-02/03 keep their old
  text — they describe the past, not live links. The
  notes/README.md chores-format filename now shows the
  `chores/` prefix.
- One code touch: the `probe_mpsc_2t.rs` module doc's path
  mention updates, so the commit runs the cargo cycle.

# References

[1]: https://github.com/winksaville/iiac-perf/commit/8aaccf8518c4 "8aaccf8518c4cb46bcc2fbf96a317d5d4c962f68"
[2]: https://github.com/winksaville/iiac-perf/commit/1043a8c53feb "1043a8c53feb0e9a10bafa0cff68eb23e13b181f"
[3]: https://github.com/winksaville/iiac-perf/commit/3fc6b48b61b1 "3fc6b48b61b1b3dd6764717ab4855f0e14429f5f"
[4]: https://github.com/winksaville/iiac-perf/commit/7251ad8e8e65 "7251ad8e8e65ad7d67883f15f7c32d4650b45c48"
[5]: https://github.com/winksaville/iiac-perf/commit/e7f138342c58 "e7f138342c58b73daf4545846644b0ecfcbc625a"
[6]: https://github.com/winksaville/iiac-perf/commit/de5dc57e8caf "de5dc57e8caf2ff90220ce4be22807d04d772aa5"
[7]: https://github.com/winksaville/iiac-perf/commit/639c3b712687 "639c3b712687e65e3c856e8a2c4d36423afc3a3d"
[8]: https://github.com/winksaville/iiac-perf/commit/6732298ddf2a "6732298ddf2ab4f76b47c4354bb654406316cd52"
[9]: https://github.com/winksaville/iiac-perf/commit/8fab65df3de7 "8fab65df3de74e0e05985e3ba395309b16aea447"
[10]: https://en.wikipedia.org/wiki/Nines_%28notation%29
[11]: https://github.com/winksaville/iiac-perf/commit/2b0472b1323d "2b0472b1323d652c6e590ff41573943ce1c7db85"
[12]: https://github.com/winksaville/iiac-perf/commit/7f586c5eac99 "7f586c5eac99c568750ad5702a8cd3d99f1d626d"
[13]: https://github.com/winksaville/iiac-perf/commit/e26cebdf4654 "e26cebdf46545862eedcac96ec269a21181e5440"
[14]: https://github.com/winksaville/iiac-perf/commit/a9946548c6b7 "a9946548c6b78cb3c8018a748fb0895d6f294e17"
[15]: https://github.com/winksaville/iiac-perf/commit/5dcd734fd2b0 "5dcd734fd2b0d8a46add370825fb156aa6034b2c"
[16]: https://github.com/winksaville/iiac-perf/commit/33a203254a91 "33a203254a91caec68c3cb9b96609c8d6a621e70"
[17]: https://github.com/winksaville/iiac-perf/commit/739675ad93bc "739675ad93bc438d1318f6f94369c0b598a60427"
[18]: https://github.com/winksaville/iiac-perf/commit/918035af8415 "918035af841582e0fb8243f2aa4257d72a9d9141"
[19]: https://github.com/winksaville/iiac-perf/commit/fb681f0620cc "fb681f0620cc023eb0c405de6418d60a8bfcb6b8"
[20]: https://github.com/winksaville/iiac-perf/commit/c3d19d9a3298 "c3d19d9a3298ba7e226facefb0e5348959e32604"
[21]: https://github.com/winksaville/iiac-perf/commit/a7fa81842cd8 "a7fa81842cd8610a26d55d10229185d1825db64e"
[22]: https://github.com/winksaville/iiac-perf/commit/c38201d8a687 "c38201d8a687b438e2c2d7a54f655a5631473a80"
[23]: https://github.com/winksaville/iiac-perf/commit/19ce29727ecf "19ce29727ecf0d0ea10d1d8494f069fa2c09f96e"
