#!/usr/bin/env python3
"""The clock experiment's analysis: reads the records of configs/clock-shift.md.

usage: python3 configs/clock-shift.py records/clock-shift.jsonl

Per host and condition: the runs' means, their spread, and the clock they ran at. Then the two
questions. Does a run's mean follow its clock: if it does, mean_ns x clock_GHz, the cycles a call
takes, is steadier across runs than mean_ns is, and mean_ns correlates with 1/clock. Does the
sleep before a run move its reading: the difference between the sleep and no-sleep conditions
against the spread of each.
"""
import glob
import json
import math
import os
import sys
from collections import defaultdict


def record_files(paths):
    """Each path as the record files it names: a directory is its `*.jsonl`, in name order, and
    anything else is that file. Every record is one line, so a file holds any number of them."""
    out = []
    for p in paths:
        out += sorted(glob.glob(f"{p}/*.jsonl")) if os.path.isdir(p) else [p]
    return out


def mean(xs):
    xs = list(xs)
    return sum(xs) / len(xs)


def stdev(xs):
    if len(xs) < 2:
        return 0.0
    m = mean(xs)
    return math.sqrt(sum((x - m) ** 2 for x in xs) / (len(xs) - 1))


def pearson(xs, ys):
    mx, my = mean(xs), mean(ys)
    sx = math.sqrt(sum((x - mx) ** 2 for x in xs))
    sy = math.sqrt(sum((y - my) ** 2 for y in ys))
    if sx == 0 or sy == 0:
        return float("nan")
    return sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / (sx * sy)


def t975(df):
    table = {1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365, 8: 2.306,
             9: 2.262, 10: 2.228, 15: 2.131, 20: 2.086, 25: 2.060, 30: 2.042, 40: 2.021, 60: 2.000}
    for k in sorted(table):
        if df <= k:
            return table[k]
    return 1.96


rows = []
for f in record_files(sys.argv[1:]):
    for line in open(f):
        r = json.loads(line)
        if r.get("tags", {}).get("experiment") != "clock-shift":
            continue
        khz = r.get("clock_khz") or []
        if not khz:
            continue
        rows.append({
            "host": r["host"]["name"],
            "cond": r["tags"].get("condition", "?"),
            "series": r["series"],
            "mean_ns": r["mean_ns"],
            "ghz": mean(khz) / 1e6,
            "ghz_lo": min(khz) / 1e6,
            "ghz_hi": max(khz) / 1e6,
        })

groups = defaultdict(list)
for r in rows:
    groups[(r["host"], r["cond"])].append(r)

print(f"{len(rows)} records\n")
print(f"{'host':6} {'condition':18} {'n':>3} {'series':>6} {'mean ns':>9} {'stdev':>7} "
      f"{'CI95':>6} {'clock GHz':>10} {'clock range':>13} {'cycles':>8} {'cv mean':>8} {'cv cyc':>7}")
summary = {}
for (host, cond), g in sorted(groups.items()):
    ms = [r["mean_ns"] for r in g]
    cyc = [r["mean_ns"] * r["ghz"] for r in g]
    n = len(ms)
    ci = t975(n - 1) * stdev(ms) / math.sqrt(n) if n > 1 else float("nan")
    summary[(host, cond)] = (mean(ms), stdev(ms), n)
    print(f"{host:6} {cond:18} {n:3d} {len({r['series'] for r in g}):6d} {mean(ms):9.2f} "
          f"{stdev(ms):7.2f} {ci:6.2f} {mean(r['ghz'] for r in g):10.3f} "
          f"{min(r['ghz_lo'] for r in g):6.2f}-{max(r['ghz_hi'] for r in g):<6.2f} "
          f"{mean(cyc):8.2f} {100 * stdev(ms) / mean(ms):7.2f}% {100 * stdev(cyc) / mean(cyc):6.2f}%")

print("\nDoes the mean follow the clock? Unpinned runs, per host:")
for host in sorted({r["host"] for r in rows}):
    g = [r for r in rows if r["host"] == host and r["cond"].startswith("unpinned")]
    if len(g) < 3:
        continue
    ms = [r["mean_ns"] for r in g]
    inv = [1.0 / r["ghz"] for r in g]
    cyc = [r["mean_ns"] * r["ghz"] for r in g]
    print(f"  {host}: n={len(g)}  r(mean_ns, 1/clock)={pearson(ms, inv):+.3f}  "
          f"clock {min(r['ghz'] for r in g):.3f}-{max(r['ghz'] for r in g):.3f} GHz "
          f"(spread {100 * stdev([r['ghz'] for r in g]) / mean([r['ghz'] for r in g]):.2f}%)  "
          f"cv mean_ns {100 * stdev(ms) / mean(ms):.2f}%  cv cycles {100 * stdev(cyc) / mean(cyc):.2f}%")

print("\nDoes the sleep move the reading? sleep minus nosleep, per host and pin:")
for host in sorted({r["host"] for r in rows}):
    for pin in ("unpinned", "pinned"):
        a = summary.get((host, f"{pin}-sleep"))
        b = summary.get((host, f"{pin}-nosleep"))
        if not a or not b:
            continue
        diff = a[0] - b[0]
        se = math.sqrt(a[1] ** 2 / a[2] + b[1] ** 2 / b[2])
        df = min(a[2], b[2]) - 1
        half = t975(df) * se
        verdict = "differs" if abs(diff) > half else "no difference shown"
        print(f"  {host} {pin:9}: {diff:+.2f} ns  (95% half-width {half:.2f} ns)  {verdict}")

print("\nPinned against unpinned, sleep conditions pooled, per host:")
for host in sorted({r["host"] for r in rows}):
    p = [r for r in rows if r["host"] == host and r["cond"].startswith("pinned")]
    u = [r for r in rows if r["host"] == host and r["cond"].startswith("unpinned")]
    if not p or not u:
        continue
    pm, um = mean([r["mean_ns"] for r in p]), mean([r["mean_ns"] for r in u])
    pg, ug = mean([r["ghz"] for r in p]), mean([r["ghz"] for r in u])
    print(f"  {host}: pinned {pm:.2f} ns at {pg:.3f} GHz, unpinned {um:.2f} ns at {ug:.3f} GHz  "
          f"mean ratio {pm / um:.3f}, clock ratio (unpinned/pinned) {ug / pg:.3f}")
