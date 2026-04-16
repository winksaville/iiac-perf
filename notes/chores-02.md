# Chores-02

Continuation of `chores-01.md`, which crossed 500 lines. Same format;
see [Chores format](README.md#chores-format).

## Calibration robustness (0.6.0)

Calibration in `overhead.rs` occasionally produces a framing value
exactly 2× the typical — e.g. 11.11 ns one run, 22.22 ns the next,
on `iiac-perf mpsc-2t --pin 5,10 -D 1` (3900X). Not a physical 2×
effect — it's the arithmetic of the two-point fit amplifying ~10 ns
of endpoint noise.

### Analysis

Two-point fit:

```
framing  = min_low − N_LOW · (min_high − min_low) / (N_HIGH − N_LOW)
d(framing)/d(min_low)  = N_HIGH / (N_HIGH − N_LOW) = 1000 / 900 ≈ 1.11
d(framing)/d(min_high) = −N_LOW / (N_HIGH − N_LOW) = −100 / 900 ≈ −0.11
```

Reverse-engineering two observed runs (3900X, `--pin 5,10`, `-D 1`):

- Run A: `min_low = 60 ns, min_high = 500 ns` → framing 11.11 ns,
  loop/iter 0.4889 ns.
- Run B: `min_low = 70 ns, min_high = 500 ns` → framing 22.22 ns,
  loop/iter 0.4778 ns.

Only `min_low` moved (+10 ns). The fit amplified that by 10/9 = 1.11
→ +11.11 ns on framing. 11.11 + 11.11 = 22.22. The "exact 2×" is
numerology; the real defect is ~10 ns of slop on `min_low`.

Why `min_low` floats but `min_high` is stable: `measure(N_LOW=100)`
runs first, right after a mere 1,000-iter warmup (~500 µs of work).
That's too short to guarantee the CPU has reached peak boost. By the
time `measure(N_HIGH=1000)` runs, boost is fully ramped, so
`min_high` reliably hits its ~500 ns floor.

### --pin vs unpinned main thread

`main.rs` already pins the main thread to `pin_cores.first()` before
calibration. Measured on 3900X, `-d 3`:

| configuration             | framing    | stdev      | p99.99 tail |
|---------------------------|-----------:|-----------:|------------:|
| `--pin 5,10`              |  11.11 ns  |    787 ns  |    272 µs   |
| no `--pin` (main unpinned)|  27.78 ns  | 12,967 ns  |    5.5 ms   |

So when `--pin` is omitted, calibration itself inherits the unpinned
jitter (migrations, frequency wobble, CCX changes), and the framing
number is clearly elevated beyond the true floor. The adjustment we
subtract is then derived from a different measurement regime than the
one we want.

### Should main be unpinned after calibration?

Functionally irrelevant. When bench threads run, the main thread
blocks in `pthread_join` — it's parked, not runnable, so its affinity
doesn't compete with anything. Keeping or dropping the pin changes
nothing observable.

The cleaner contract is:

- Calibration always runs pinned (stable environment, reliable
  framing/loop numbers).
- Bench runs per `--pin` (user's requested environment).
- Main's pinning during the bench phase is a don't-care.

So: leave main's pin state alone after calibration; don't add a
ceremonial unpin step unless a concrete reason emerges later.

### Recommended changes

1. **Pin main for calibration regardless of `--pin`.** Even when the
   user didn't specify `--pin`, briefly pin main (to `pin_cores[0]`
   if set, otherwise a fixed default such as core 0, or a new
   `--cal-pin` flag) so calibration always runs in a stable
   environment. Biggest coherence win — calibration stops depending
   on how the bench is configured.

   Add a `--no-pin-cal` flag that reverts to the pre-0.6.0 behavior
   (skip the calibration-time pin, so main stays unpinned when
   `--pin` is absent). Makes A/B comparison easy — run the same
   bench with and without `--no-pin-cal` to see the calibration
   difference directly, and gives a quick escape hatch if the new
   default ever misbehaves on someone else's box.

   **Open question — semantics of `--pin` + `--no-pin-cal`.** As
   shipped in dev2, `--pin` wins: with `--pin 5,10 --no-pin-cal`
   main is still pinned to core 5 for calibration. Alternative
   reading: `--no-pin-cal` should *always* skip the cal pin, so
   `--pin 5,10 --no-pin-cal` → bench threads pinned to 5,10,
   calibration runs unpinned. User flagged this as a likely future
   preference but fine as-is for now. Revisit once we have more
   data on which framing is more useful in that combo.
2. **Longer warmup.** Bump `CAL_WARMUP` from 1,000 to ~100,000
   iterations so boost ramp completes before `measure(N_LOW)` runs.
3. **More samples.** Bump `CAL_SAMPLES` from 10,000 to ~100,000 so
   the min-of-samples estimator reliably hits the theoretical floor.
4. **Wider N spread.** Raise `N_HIGH` to ~10,000 (or drop `N_LOW` to
   ~10). This drops the noise-amplification coefficient on `min_low`
   from 1.11× to ~1.01×, so any remaining endpoint noise barely
   perturbs framing.
5. *(Optional)* **Sanity-check retry.** Run calibration 3–5 times;
   require framing/loop to agree within a tolerance. Retry or warn
   otherwise. Belt-and-suspenders for the rare bad run.

### Interpretation note: the low min-p1 / p1-p10 bands are real

Unpinned `mpsc-2t` runs show a very low `min-p1` band — e.g. `first
= 240 ns, last = 6,003 ns` in the 3s unpinned run above, while the
pinned `--pin 5,10` run's `min-p1` sits at `7,935…8,175 ns`. That
gap isn't calibration noise or a measurement artifact.

When both round-trip endpoints stay hot (scheduler co-locates them
on the same CCX, neither parks in a futex, both spin on the
channel), `std::sync::mpsc` — `crossbeam-channel` under the hood on
modern Rust — can round-trip in well under a microsecond. Pinning
to `5,10` on a 3900X likely forces cross-CCX placement (cores 5 and
10 are on different chiplets), which adds cache-coherence latency
and raises the floor. So the low bands in unpinned runs are a real
operating regime — "both ends hot and spinning" — not a glitch, and
the calibration rework here should not distort them.

Multi-step probably fits better because (1)–(4) are independent and
we'll want a before/after measurement at each step. Rough order:

- `0.6.0-dev1` ✅ chore marker: bump version, write this plan,
  update todo.
- `0.6.0-dev2` ✅ pin main for calibration regardless of `--pin`;
  add `--no-pin-cal` opt-out that restores pre-0.6.0 behavior.
  Relabel startup banner: `pinning` → `bench pin`, and add a new
  `cal pin` line. Dev2 fixes coherence only — framing/loop values
  still show run-to-run variance (expected; dev3/dev4 address
  stability).
- `0.6.0-dev3` — longer warmup + more samples.
- `0.6.0-dev4` — widen N spread.
- `0.6.0-dev5` *(optional)* — sanity-check retry loop.
- `0.6.0` final — remove `-devN`, update todo/chores.
