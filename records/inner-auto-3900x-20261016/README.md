# Fixed auto against a span around it, 3900X, 2026-10-16

Eighteen invocations of ten runs each, taken at `feat: how inner is chosen and shown` to decide
whether auto should choose a span, and read in [Fixed auto against a span around
it](../../notes/random-inner.md#fixed-auto-against-a-span-around-it).

- Host: the 3900X, `ccx` pair, CPU-pinned with the clock unpinned (`--pin-freq=no`), since the
  session ran in a sandbox whose cpufreq files are read-only. The load average was 4.9 at the
  start, the desktop in use.
- Build: `iiac-perf-dev` 0.29.2-4.
- Benches: `min-now`, `zcr-spsc-v4-2t`, `zcr-mpsc-v4-2t-single-sos-futex-spntfe`.
- Arms, tagged `arm`: `fixed`, no `--inner`, auto choosing 23 for `min-now` and 6 to 7 for the
  rings, and `span`, `--inner 15-30` for `min-now` and `--inner 1-16` for the rings, auto's
  count about 8 either side, at least 16 counts wide, floored at 1.
- Rounds, tagged `round`: three, each bench's two arms back to back, fixed first in rounds 1 and
  3 and span first in round 2.

The session, 2026-10-16 23:35 to 23:44 UTC:

```bash
dir=records/inner-auto-3900x-20261016
common=(--runs 10 -d 1 --pin-cpus ccx --pin-freq=no --no-inhibit --record-dir "$dir")
declare -A span=([min-now]=15-30 [zcr-spsc-v4-2t]=1-16 [zcr-mpsc-v4-2t-single-sos-futex-spntfe]=1-16)
for round in 1 2 3; do
  for b in min-now zcr-spsc-v4-2t zcr-mpsc-v4-2t-single-sos-futex-spntfe; do
    if (( round % 2 )); then arms=(fixed span); else arms=(span fixed); fi
    for arm in "${arms[@]}"; do
      extra=()
      [[ $arm == span ]] && extra=(--inner "${span[$b]}")
      iiac-perf-dev run "$b" "${common[@]}" "${extra[@]}" --tag arm=$arm --tag round=$round
    done
  done
done
```

Read with `iiac-perf analyze records/inner-auto-3900x-20261016 --compare arm=fixed,span` and
`iiac-perf linear-regression records/inner-auto-3900x-20261016`.
