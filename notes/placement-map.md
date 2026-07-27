# Thread-placement map: zcr-mpsc-2t round trip

Measured 2026-07-27 with iiac-perf 0.22.0-5, one command shape
per cell: `iiac-perf zcr-mpsc-2t [--pin A,B] -d 1 --decimals 3`.
The bench is a two-software-thread spin-wait round trip over two
zc-ring-x1 MPSC rings; the number reported here is the adjusted
trimmed mean (`mean z..n2`, i.e. mean below p99) and the run's
worst sample. Placement — which logical CPUs the two threads
own — is the only variable swept.

## Topology (lscpu -e)

- **3900X** (Zen 2): 12 physical cores, 24 logical CPUs,
  siblings N/N+12. Four 3-core CCXs — L3 domains {0,1,2},
  {3,4,5}, {6,7,8}, {9,10,11} — two per CCD, two CCDs.
- **7600X** (Zen 4): 6 physical cores, 12 logical CPUs,
  siblings N/N+6. One CCD, one CCX: all cores share a single
  L3.

## 3900X

| placement          | pin   | trimmed mean | worst sample |
|--------------------|-------|--------------|--------------|
| SMT siblings       | 2,14  | 51.8 ns      | 22.5 us      |
| same CCX           | 3,4   | 98.3 ns      | 36.1 us      |
| cross-CCX same CCD | 8,9   | 401.7 ns     | 17.7 us      |
| cross-CCD          | 3,9   | 394.6 ns     | 111.1 us     |
| unpinned           | —     | 98.4 ns      | 31.5 us      |

- The three zones are ~1 : 2 : 8 — share an L1/L2 (SMT), share
  an L3 (CCX), or cross the fabric.
- Crossing the die boundary costs the same as crossing CCXs on
  one die (394.6 vs 401.7, within run noise). Consistent with
  Zen 2's design: all CCX-to-CCX traffic routes through the IO
  die, so there is no "nearer" remote CCX. We think this is the
  mechanism; the map only shows the equality.
- Unpinned lands on the same-CCX number: the scheduler spreads
  onto separate physical cores (avoiding SMT sharing). The 2x
  SMT-sibling latency win is only reachable by explicit pinning.
- Neighborhood matters for the tail, not the mean: an SMT pair
  on core 0 (`--pin 0,12`, run 2026-07-27 00:53) matched the
  ~51 ns mean but paid a ~981 us worst sample and graded F —
  we think core 0's IRQ/housekeeping load is the cause. The
  same experiment on cores 2,14 or 3,4 keeps worst samples in
  the tens of us.
- The unpinned cell above is from a quiet by-hand run
  (2026-07-27 00:48). A same-day scripted rerun with a bot
  session live on the machine graded F (19.25% disturbed) and
  read ~107 ns — an unpinned run competes with everything
  running; the environment grade caught it.
- Every 3900X calibration that day carried the persistent
  "loop ladder deviates ~9%" warning (machine trait, present
  at every placement; see TODO).

## 7600X

| placement       | pin  | trimmed mean | worst sample |
|-----------------|------|--------------|--------------|
| SMT siblings    | 2,8  | 41.2 ns      | 3.3 us       |
| separate cores  | 2,3  | 69.8 ns      | 9.5 us       |
| unpinned        | —    | 59.6 ns      | 5.3 us       |

- The single-CCX part has no fabric zone to measure: the whole
  map spans 41–70 ns (1.7x), against the 3900X's 8x. The ~390
  ns zone isn't improved on the 7600X — it's absent, a fact of
  topology, not code.
- Unpinned sits *between* the two pinned cells (59.6), unlike
  the 3900X where it matched separate-cores. We think the
  scheduler sometimes co-locates the pair on siblings here;
  not verified.
- All three cells graded environment A with worst samples
  under 10 us — the machine is both faster and far quieter.

## Cross-machine reading

- Same code, same experiment: 41–70 ns everywhere on the
  7600X, 52–402 ns on the 3900X depending on placement. For a
  latency-sensitive pair of threads, placement on a multi-CCX
  part is worth more than the generation gap — the 3900X
  pinned to an SMT pair (51.8) beats the 7600X's scheduler
  default (59.6).
- The spin-wait design assumes each spinning software thread
  owns a logical CPU; a pool smaller than the placements
  requested livelocks through preemption (bug #1 in
  [bugs.md](bugs.md#bugs)).
