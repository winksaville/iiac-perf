# Chores 04

Continuation of [chores-03](../chores-03.md). Records landed work;
conventions in [AGENTS.md](../../AGENTS.md#chores-conventions) and
[cycle-protocol.md](../cycle-protocol.md#chores-sections).

## feat: zcr bench family (raw/with/spin, 1t/2t)

Commits: [[1]],[[2]],[[3]],[[4]],[[5]]

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

Commits:

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

Commits:

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

# References

[1]: https://github.com/winksaville/iiac-perf/commit/8aaccf8518c4 "8aaccf8518c4cb46bcc2fbf96a317d5d4c962f68"
[2]: https://github.com/winksaville/iiac-perf/commit/1043a8c53feb "1043a8c53feb0e9a10bafa0cff68eb23e13b181f"
[3]: https://github.com/winksaville/iiac-perf/commit/3fc6b48b61b1 "3fc6b48b61b1b3dd6764717ab4855f0e14429f5f"
[4]: https://github.com/winksaville/iiac-perf/commit/7251ad8e8e65 "7251ad8e8e65ad7d67883f15f7c32d4650b45c48"
[5]: https://github.com/winksaville/iiac-perf/commit/e7f138342c58 "e7f138342c58b73daf4545846644b0ecfcbc625a"
