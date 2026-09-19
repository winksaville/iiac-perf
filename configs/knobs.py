#!/usr/bin/env python3
"""The knob search's analysis: is a config's number worth trusting, and what would cost less?

usage: python3 configs/knobs.py records/knobs.jsonl

A config is trustworthy on two counts, and tightness alone is not one of them.

- Precision: the `LSC trimmed` it claims, the smallest change between two such series that would
  register, as a percent of the trimmed mean. The trimmed pair is the cycle's statistic, since a
  host that disturbs a run in six leaves the plain pair reporting the disturbance rather than the
  workload. The plain pair is printed beside it, and the gap between them is the tail.
- Calibration: whether that claim holds up. Each invocation claims a standard error of
  `stdev / sqrt(runs)` for its own mean. Run the same config several times and the spread of
  those invocation means should match it. The ratio of the two is the calibration number: about
  1 is honest, well above 1 means the invocation is claiming a precision its runs cannot see,
  because something moves between invocations that does not move between the runs inside one.

The allocation model is the Todo entry's, `Allocate runs and duration for a fixed wall time`. A
run mean's variance is `s_p^2 + a/d` for between-process spread `s_p` and within-run noise `a/d`
at measured length `d`, so `R` runs in wall time `T = R * (o + d)` at per-run overhead `o` give
`(s_p^2 + a/d) / R`. Least variance at a fixed `T`, and least `T` at a fixed variance, both fall
at `d* = sqrt(a * o / s_p^2)`. Since `d*` goes as the square root of `o`, cutting the overhead
moves the optimum for everything after it.

Everything here is read from the records: `a` from each run's block series, `s_p` from the spread
of the run means less that within-run part, and `o` from the wall clock between run starts less
the measured stretch, so no knob's cost is assumed.
"""
import glob
import json
import math
import os
import sys
from collections import defaultdict
from datetime import datetime

T975 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
        2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064,
        2.060, 2.056, 2.052, 2.048, 2.045, 2.042]


def record_files(paths):
    """Each path as the record files it names: a directory is its `*.jsonl`, in name order, and
    anything else is that file. Every record is one line, so a file holds any number of them."""
    out = []
    for p in paths:
        out += sorted(glob.glob(f"{p}/*.jsonl")) if os.path.isdir(p) else [p]
    return out


def t975(df):
    """The project's own table, so these numbers match what a report prints."""
    if df <= 0:
        return float("inf")
    return T975[df - 1] if df <= 30 else 2.0


def mean(xs):
    xs = list(xs)
    return sum(xs) / len(xs)


def var(xs):
    xs = list(xs)
    if len(xs) < 2:
        return 0.0
    m = mean(xs)
    return sum((x - m) ** 2 for x in xs) / (len(xs) - 1)


def stdev(xs):
    return math.sqrt(var(xs))


def ci95(sd, n):
    return t975(n - 1) * sd / math.sqrt(n)


TRIM = 0.2


def trimmed(xs):
    """`Trimmed` from src/series.rs: a fifth off each end, Yuen's standard error, so these match
    the report's `trimmed mean`, `CI95 trimmed`, and `LSC trimmed` rows."""
    xs = sorted(xs)
    n = len(xs)
    per_end = int(TRIM * n)
    if per_end == 0 or n - 2 * per_end < 2:
        return None
    lo, hi = xs[per_end], xs[n - per_end - 1]
    kept = xs[per_end:n - per_end]
    wins = [min(max(x, lo), hi) for x in xs]
    wsd = stdev(wins)
    se = wsd / ((1 - 2 * TRIM) * math.sqrt(n))
    k = len(kept)
    return {
        "mean": mean(kept), "wsd": wsd, "se": se, "kept": k, "per_end": per_end,
        "ci95": t975(k - 1) * se, "lsc": t975(2 * k - 2) * se * math.sqrt(2),
    }


def lsc(sd, n):
    return t975(2 * n - 2) * sd * math.sqrt(2.0 / n)


def load(dirs):
    """Every record under `dirs`, grouped by host, condition, and invocation."""
    out = defaultdict(lambda: defaultdict(list))
    for f in record_files(dirs):
        for line in open(f):
            r = json.loads(line)
            cond = r.get("tags", {}).get("condition", "-")
            out[(r["host"]["name"], cond, r["bench"])][r["series"]].append(r)
    return out


def session(runs):
    """One invocation's numbers: its claim, its cost, and the model's three parameters."""
    runs = sorted(runs, key=lambda r: r["run"])
    means = [r["mean_ns"] for r in runs]
    n = len(means)
    d = mean(r["measured_s"] for r in runs)
    # The within-run part, a/d, is what each run's own blocks measure, over the blocks'
    # effective count rather than their raw one: correlated blocks carry less than they look.
    n_eff = eff_n([r["block_mean_ns"] for r in runs])
    within = mean(var(r["block_mean_ns"]) / n_eff for r in runs)
    within_white = mean(var(r["block_mean_ns"]) / len(r["block_mean_ns"]) for r in runs)
    r1 = mean(lag1(r["block_mean_ns"]) for r in runs)
    # Wall clock between run starts covers the warm, the sleep, the spawn, and the measuring.
    ts = [datetime.strptime(r["t_start"], "%Y-%m-%dT%H:%M:%S.%fZ") for r in runs]
    cycle = (ts[-1] - ts[0]).total_seconds() / (n - 1) if n > 1 else float("nan")
    sd = stdev(means)
    t = trimmed(means)
    # The model's between-process spread comes from the trimmed estimate, so a disturbed run
    # does not inflate `s_p` and through it `d*`.
    robust_sd = t["wsd"] if t else sd
    s_p2 = max(robust_sd ** 2 - within, 0.0)
    return {
        "n": n, "mean": mean(means), "sd": sd,
        "ci95": ci95(sd, n), "lsc": lsc(sd, n), "se": sd / math.sqrt(n),
        "t": t,
        "d": d, "cycle": cycle, "o": cycle - d,
        "a": d * within, "s_p": math.sqrt(s_p2), "within": within,
        "a_white": d * within_white, "r1": r1, "n_eff": n_eff,
        "n_blocks": len(runs[0]["block_mean_ns"]),
        "wall": cycle * n,
    }


def lag1(xs):
    """Lag-1 autocorrelation of a series."""
    m = mean(xs)
    den = sum((x - m) ** 2 for x in xs)
    if not den:
        return 0.0
    return sum((xs[i] - m) * (xs[i + 1] - m) for i in range(len(xs) - 1)) / den


def acf(xs, k):
    """The lag-`k` autocorrelation of a series."""
    m = mean(xs)
    den = sum((x - m) ** 2 for x in xs)
    if not den or k >= len(xs):
        return 0.0
    return sum((xs[i] - m) * (xs[i + k] - m) for i in range(len(xs) - k)) / den


def eff_n(series_list):
    """Blocks worth of independent information in a run's correlated ones:
    `n / (1 + 2 * sum((1 - k/n) * rho_k))`, the correlations averaged over the runs given and
    summed out to the last lag before one turns non-positive, the initial positive sequence.

    The first version assumed the correlation dies away geometrically from its lag-1 value,
    `n * (1 - r) / (1 + r)`. It does not on these blocks: at lag 5 it is still +0.17 where that
    assumption has +0.01, so the geometric form put a hundred blocks at 41 where the measured
    correlations put them at 23. Without the correction the within-run term `a/d` is
    understated and `d*` with it."""
    n = min(len(b) for b in series_list)
    total = 0.0
    for k in range(1, n // 2):
        rho = mean(acf(b, k) for b in series_list)
        if rho <= 0:
            break
        total += (1 - k / n) * rho
    return max(n / (1 + 2 * total), 1.0)


def d_star(a, o, s_p):
    return math.sqrt(a * o / s_p ** 2) if s_p > 0 and a > 0 else float("nan")


def main(dirs):
    groups = load(dirs)
    if not groups:
        print("no records found")
        return
    for key in sorted(groups):
        host, cond, bench = key
        ss = [session(v) for v in groups[key].values() if len(v) > 1]
        if not ss:
            continue
        print(f"\n=== {host}  {bench}  condition={cond}  ({len(ss)} invocations "
              f"x {ss[0]['n']} runs) ===")
        print(f"{'':4}{'trim mean':>10} {'LSC trim':>9} {'LSC %':>7} | {'plain':>8} "
              f"{'LSC %':>7} | {'cut':>4} {'wall s':>7} {'d s':>6} {'o s':>6} "
              f"{'a ns2s':>8} {'s_p ns':>7}")
        for i, s_ in enumerate(ss, 1):
            t = s_["t"]
            tm = f"{t['mean']:10.2f} {t['lsc']:9.3f} {100*t['lsc']/t['mean']:6.2f}%" if t else \
                 f"{'-':>10} {'-':>9} {'-':>7}"
            print(f"{i:<4}{tm} | {s_['mean']:8.2f} {100*s_['lsc']/s_['mean']:6.2f}% | "
                  f"{2*t['per_end'] if t else 0:4d} {s_['wall']:7.1f} {s_['d']:6.2f} "
                  f"{s_['o']:6.2f} {s_['a']:8.4f} {s_['s_p']:7.3f}")
        print(f"{'':4}{'-'*78}")
        # The claim and the check, both on the trimmed pair.
        tms = [s_["t"]["mean"] for s_ in ss if s_["t"]]
        tse = mean(s_["t"]["se"] for s_ in ss if s_["t"])
        gm, claimed, observed = mean(tms), tse, stdev(tms)
        plain_obs = stdev(s_["mean"] for s_ in ss)
        print(f"     trimmed grand mean {gm:.2f} ns   between invocations {observed:.3f} ns "
              f"= {100*observed/gm:.2f}%   (plain: {plain_obs:.3f} ns "
              f"= {100*plain_obs/mean(s_['mean'] for s_ in ss):.2f}%)")
        if claimed > 0:
            ratio = observed / claimed
            if len(ss) < 5:
                verdict = f"too few invocations to judge, {len(ss)} of about 5 wanted"
            elif ratio <= 1.5:
                verdict = "calibrated"
            else:
                verdict = "OPTIMISTIC: the claim does not cover the spread between invocations"
            print(f"     claimed SE {claimed:.3f} ns   calibration {ratio:.2f}x  ({verdict})")
        a, o, s_p = (mean(s_[k] for s_ in ss) for k in ("a", "o", "s_p"))
        aw, r1 = (mean(s_[k] for s_ in ss) for k in ("a_white", "r1"))
        print(f"     model on the trimmed spread: a={a:.4f} ns^2 s  o={o:.2f} s  "
              f"s_p={s_p:.3f} ns  -> d*={d_star(a, o, s_p):.3f} s")
        ne, nb = mean(s_["n_eff"] for s_ in ss), ss[0]["n_blocks"]
        print(f"       a counts a run's {nb} blocks as {ne:.0f}, from their measured correlations "
              f"(lag-1 {r1:+.2f}); uncorrected it would read {aw:.4f} and d* {d_star(aw, o, s_p):.3f} s")
        for cut in (0.5, 0.2, 0.05):
            if cut < o:
                print(f"            if o were {cut:.2f} s: d*={d_star(a, cut, s_p):.3f} s")

        runs = [r for v in groups[key].values() for r in v]
        print(f"\n     how many runs? the LSC a k-run trimmed series would claim, "
              f"as % of the mean")
        row = "       "
        # The spread a k-run series would see is the robust one, so a disturbed run does not
        # decide the run count. Below five runs nothing is trimmed and the plain pair is all
        # there is, which is itself the argument for not going there on a host with a tail.
        wsd = mean(s_["t"]["wsd"] for s_ in ss if s_["t"])
        psd = mean(s_["sd"] for s_ in ss)
        for k in (3, 5, 10, 15, 20, 30):
            per_end = int(TRIM * k)
            kept = k - 2 * per_end
            if per_end == 0 or kept < 2:
                # Nothing can be trimmed, so what such a series would claim is the plain pair
                # off the contaminated spread. That is the honest figure, and the warning.
                row += f"k={k}: {100*lsc(psd, k)/gm:5.2f}%*  "
                continue
            se = wsd / ((1 - 2 * TRIM) * math.sqrt(k))
            row += f"k={k}: {100*t975(2*kept-2)*se*math.sqrt(2)/gm:5.2f}%   "
        print(row + "   (* untrimmable, plain)")
        print(f"     how many blocks? the within-run bar from the first k blocks, as % of the mean")
        row = "       "
        for k in (8, 16, 25, 50, 100):
            hs = [100 * ci95(stdev(r["block_mean_ns"][:k]), k) / mean(r["block_mean_ns"][:k])
                  for r in runs if len(r["block_mean_ns"]) >= k]
            if hs:
                row += f"k={k}: {mean(hs):5.3f}%   "
        print(row)
        r1 = []
        for r in runs:
            b = r["block_mean_ns"]
            m = mean(b)
            den = sum((x - m) ** 2 for x in b)
            if den:
                r1.append(sum((b[i] - m) * (b[i + 1] - m) for i in range(len(b) - 1)) / den)
        if r1:
            print(f"     blocks independent? lag-1 autocorrelation {mean(r1):+.3f} "
                  f"(0 = independent), and they are worth about "
                  f"{eff_n([r['block_mean_ns'] for r in runs]):.0f} independent ones")


if __name__ == "__main__":
    main(sys.argv[1:] or ["records/knobs.jsonl"])
