#!/usr/bin/env python3
"""Draw the figures of docs/statistics.md from the tracked records.

usage: python3 docs/figures/make.py          (from the repository's root; writes docs/figures/*.svg)

Every figure is a standalone SVG drawn from `records/knobs.jsonl`, so a figure can be checked
against the data and redrawn when the data grows. Each carries its own style, an opaque surface,
and a dark variant under `prefers-color-scheme`, so it reads on a light page or a dark one. The
numbers a figure shows are also in the document's tables, since a static image has no hover.

One colour plan throughout: gray is a single run, blue the trimmed pair, orange the plain pair.
The statistics come from `configs/knobs.py` until the `analyze` command replaces it.
"""
import importlib.util
import itertools
import json
import math
import os
import statistics as st
from collections import defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "docs/figures")
spec = importlib.util.spec_from_file_location("knobs", os.path.join(ROOT, "configs/knobs.py"))
K = importlib.util.module_from_spec(spec)
spec.loader.exec_module(K)

STYLE = """<style>
svg{--surface:#fcfcfb;--ink:#0b0b0b;--ink2:#52514e;--muted:#898781;--grid:#e1e0d9;--axis:#c3c2b7;
--s1:#2a78d6;--s2:#eb6834;font-family:system-ui,-apple-system,"Segoe UI",sans-serif}
@media (prefers-color-scheme:dark){svg{--surface:#1a1a19;--ink:#fff;--ink2:#c3c2b7;--muted:#898781;
--grid:#2c2c2a;--axis:#383835;--s1:#3987e5;--s2:#d95926}}
.bg{fill:var(--surface)}.grid{stroke:var(--grid);stroke-width:1}.axis{stroke:var(--axis);stroke-width:1}
.tick{fill:var(--muted);font-size:11px}.axlabel{fill:var(--muted);font-size:11px}
.cap{fill:var(--muted);font-size:11px}.lab{fill:var(--ink2);font-size:12px;font-weight:600}
.ring{fill:var(--surface)}.run{fill:var(--muted)}.s1{fill:var(--s1)}.s2{fill:var(--s2)}
.e1{stroke:var(--s1);stroke-width:2;stroke-linecap:round}.e2{stroke:var(--s2);stroke-width:2;stroke-linecap:round}
.wash{fill:var(--s1);opacity:.10}.washg{fill:var(--muted);opacity:.14}.b1{fill:var(--s1)}.b2{fill:var(--s2)}
.l1{fill:none;stroke:var(--muted);stroke-width:2;stroke-linejoin:round}
.l2{fill:none;stroke:var(--ink2);stroke-width:2;stroke-linejoin:round}
.l3{fill:none;stroke:var(--s1);stroke-width:2;stroke-linejoin:round}
.rule{stroke:var(--axis);stroke-width:1}
</style>"""

_ids = itertools.count()


class Plot:
    """One panel: linear scales, recessive grid, direct labels in a column at the right."""

    def __init__(self, w, h, x0, x1, y0, y1, left=58, right=150, top=16, bottom=36, logx=False):
        self.w, self.h, self.l, self.r, self.t, self.b = w, h, left, right, top, bottom
        self.x0, self.x1, self.y0, self.y1, self.logx = x0, x1, y0, y1, logx
        self.body, self.id = [], f"c{next(_ids)}"

    def X(self, v):
        f = ((math.log10(v) - math.log10(self.x0)) / (math.log10(self.x1) - math.log10(self.x0))
             if self.logx else (v - self.x0) / (self.x1 - self.x0))
        return self.l + f * (self.w - self.l - self.r)

    def Y(self, v):
        return self.h - self.b - (v - self.y0) / (self.y1 - self.y0) * (self.h - self.t - self.b)

    def add(self, s):
        self.body.append(s)

    def yaxis(self, ticks, label, fmt="{:g}"):
        for v in ticks:
            y = self.Y(v)
            self.add(f'<line class="grid" x1="{self.l}" x2="{self.w - self.r}" y1="{y:.1f}" y2="{y:.1f}"/>'
                     f'<text class="tick" x="{self.l - 8}" y="{y + 4:.1f}" text-anchor="end">{fmt.format(v)}</text>')
        mid = self.t + (self.h - self.t - self.b) / 2
        self.add(f'<text class="axlabel" transform="translate(14 {mid:.0f}) rotate(-90)" text-anchor="middle">{label}</text>')

    def xaxis(self, ticks, label=None, names=None):
        yb = self.h - self.b
        self.add(f'<line class="axis" x1="{self.l}" x2="{self.w - self.r}" y1="{yb}" y2="{yb}"/>')
        for i, v in enumerate(ticks):
            self.add(f'<text class="tick" x="{self.X(v):.1f}" y="{yb + 15}" text-anchor="middle">{names[i] if names else f"{v:g}"}</text>')
        if label:
            self.add(f'<text class="axlabel" x="{(self.l + self.w - self.r) / 2:.0f}" y="{self.h - 4}" text-anchor="middle">{label}</text>')

    def dot(self, x, y, cls, r=4):
        if self.y0 <= y <= self.y1:
            px, py = self.X(x), self.Y(y)
            self.add(f'<circle class="ring" cx="{px:.1f}" cy="{py:.1f}" r="{r + 2}"/><circle class="{cls}" cx="{px:.1f}" cy="{py:.1f}" r="{r}"/>')

    def arrow(self, x, cls):
        px, py = self.X(x), self.t + 7
        self.add(f'<path class="{cls}" d="M{px - 5:.1f},{py + 5:.1f} L{px:.1f},{py - 5:.1f} L{px + 5:.1f},{py + 5:.1f} Z"/>')

    def bar(self, x, lo, hi, cls):
        lo, hi = max(lo, self.y0), min(hi, self.y1)
        if hi > lo:
            self.add(f'<line class="{cls}" x1="{self.X(x):.1f}" x2="{self.X(x):.1f}" y1="{self.Y(lo):.1f}" y2="{self.Y(hi):.1f}"/>')

    def line(self, pts, cls):
        self.add(f'<polyline class="{cls}" points="' + " ".join(f"{self.X(x):.1f},{self.Y(y):.1f}" for x, y in pts) + '"/>')

    def side(self, lines, y=None, key=None):
        """Direct labels in the right-hand column: the first line strong, the rest muted.

        `key` is a line class, drawn as a short swatch above the label so it names its line.
        """
        y = self.t + 12 if y is None else y
        if key:
            x = self.w - self.r + 10
            self.add(f'<polyline class="{key}" points="{x},{y - 16:.1f} {x + 24},{y - 16:.1f}"/>')
        for i, s in enumerate(lines):
            self.add(f'<text class="{"lab" if i == 0 else "cap"}" x="{self.w - self.r + 10}" y="{y + 14 * i:.1f}">{s}</text>')

    def clip(self):
        return (f'<clipPath id="{self.id}"><rect x="{self.l}" y="{self.t}" width="{self.w - self.l - self.r}" '
                f'height="{self.h - self.t - self.b}"/></clipPath>')


def write(name, panels, title):
    w, h, y, parts = panels[0].w, sum(p.h for p in panels), 0, []
    for p in panels:
        parts.append(f'<g transform="translate(0 {y})"><defs>{p.clip()}</defs>{"".join(p.body)}</g>')
        y += p.h
    svg = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" role="img">'
           f"<title>{title}</title>{STYLE}<rect class=\"bg\" width=\"{w}\" height=\"{h}\"/>{''.join(parts)}</svg>\n")
    open(os.path.join(OUT, name), "w").write(svg)
    print(f"  {name}  {len(svg) // 1024} KB")


# ---------------------------------------------------------------- the data
runs_by = defaultdict(list)
for line in open(os.path.join(ROOT, "records/knobs.jsonl")):
    r = json.loads(line)
    runs_by[r["series"]].append(r)
NAMES = {"baseline": "baseline", "baseline-quiet": "baseline again", "untouched": "untouched", "-": "wink's run"}
inv, hundred = [], None
for sid in sorted(runs_by):
    rs = sorted(runs_by[sid], key=lambda r: r["run"])
    cond = rs[0]["tags"].get("condition", "-")
    if cond == "r100-d1s":
        hundred = rs
        continue
    m = [r["mean_ns"] for r in rs]
    inv.append(dict(sid=sid, cond=NAMES[cond], runs=rs, m=m, t=K.trimmed(m), pm=K.mean(m),
                    pci=K.ci95(K.stdev(m), len(m))))
W = 860

# ---------------------------------------------------------------- runs.svg
def runs_panel(y0, y1, ticks, note):
    x, xs, prev, groups = 0.0, [], None, []
    for v in inv:
        x += 1.0 if v["cond"] == prev else 1.9
        if v["cond"] != prev:
            groups.append([v["cond"], x, x])
        groups[-1][2], prev = x, v["cond"]
        xs.append(x)
    p = Plot(W, 290, 0.6, xs[-1] + 0.9, y0, y1)
    p.yaxis(ticks, "run mean, ns")
    p.xaxis([])
    for name, a, b in groups:
        p.add(f'<text class="tick" x="{(p.X(a) + p.X(b)) / 2:.1f}" y="{p.h - p.b + 15}" text-anchor="middle">{name}</text>')
    p.add(f'<g clip-path="url(#{p.id})">')
    for v, cx in zip(inv, xs):
        for i, r in enumerate(v["runs"]):
            p.dot(cx - 0.12 + 0.06 * (i % 5), r["mean_ns"], "run", r=3.5)
        p.bar(cx + 0.30, v["pm"] - v["pci"], v["pm"] + v["pci"], "e2")
        p.bar(cx - 0.30, v["t"]["mean"] - v["t"]["ci95"], v["t"]["mean"] + v["t"]["ci95"], "e1")
    p.add("</g>")
    for v, cx in zip(inv, xs):
        if v["pm"] > y1:
            p.arrow(cx + 0.30, "s2")
        p.dot(cx + 0.30, v["pm"], "s2", r=4.5)
        p.dot(cx - 0.30, v["t"]["mean"], "s1", r=4.5)
    p.side(note)
    return p


top = runs_panel(90, 156, [100, 120, 140], ["the whole range"])
top.add(f'<circle class="s1" cx="{top.w - top.r + 15}" cy="{top.t + 40}" r="4.5"/><text class="cap" x="{top.w - top.r + 25}" y="{top.t + 44}">trimmed mean ± CI95</text>'
        f'<circle class="s2" cx="{top.w - top.r + 15}" cy="{top.t + 58}" r="4.5"/><text class="cap" x="{top.w - top.r + 25}" y="{top.t + 62}">plain mean ± CI95</text>'
        f'<circle class="run" cx="{top.w - top.r + 15}" cy="{top.t + 76}" r="3.5"/><text class="cap" x="{top.w - top.r + 25}" y="{top.t + 80}">one run</text>')
write("runs.svg", [top, runs_panel(93.5, 98.0, [94, 95, 96, 97], ["zoomed on the core", "an arrow: a plain mean", "above this panel"])],
      "Every baseline run by invocation, with the trimmed and the plain mean and their error bars")

# ---------------------------------------------------------------- same-code.svg
def pairs(mean_of, se_of, df):
    return [(100 * abs(mean_of(a) - mean_of(b)) / mean_of(a),
             abs(mean_of(a) - mean_of(b)) > K.t975(df) * math.hypot(se_of(a), se_of(b)))
            for a, b in itertools.combinations(inv, 2)]


dt = pairs(lambda v: v["t"]["mean"], lambda v: v["t"]["se"], 6)
dp = pairs(lambda v: v["pm"], lambda v: K.stdev(v["m"]) / math.sqrt(len(v["m"])), 18)
STEP, TOP = 0.25, 9.5
tallest = max(max(sum(1 for d, _ in data if i * STEP <= d < (i + 1) * STEP) for i in range(int(TOP / STEP))) for data in (dt, dp))
YMAX = int(math.ceil(tallest * 1.05 / 20) * 20)
AA = {}


def hist(data, cls, name):
    vals = sorted(d for d, _ in data)
    p = Plot(W, 215, 0, TOP, 0, YMAX, top=26)
    p.yaxis(list(range(20, YMAX + 1, 20)), "pairs")
    p.xaxis(list(range(10)), "difference between two invocations of the same code, % of the mean")
    wpx = p.X(STEP) - p.X(0) - 2
    for i in range(int(TOP / STEP)):
        c = sum(1 for v in vals if i * STEP <= v < (i + 1) * STEP)
        if c:
            x, y = p.X(i * STEP) + 1, p.Y(c)
            p.add(f'<path class="{cls}" d="M{x:.1f},{p.Y(0):.1f} V{y + 3:.1f} Q{x:.1f},{y:.1f} {x + 3:.1f},{y:.1f} '
                  f'H{x + wpx - 3:.1f} Q{x + wpx:.1f},{y:.1f} {x + wpx:.1f},{y + 3:.1f} V{p.Y(0):.1f} Z"/>')
    med, p95, alarms = st.median(vals), vals[int(0.95 * len(vals))], sum(f for _, f in data)
    for v, tag in ((med, "half"), (p95, "95%")):
        p.add(f'<line class="rule" x1="{p.X(v):.1f}" x2="{p.X(v):.1f}" y1="{p.t - 4}" y2="{p.Y(0):.1f}"/>'
              f'<text class="cap" x="{p.X(v):.1f}" y="{p.t - 8}" text-anchor="middle">{tag}</text>')
    p.side([name, f"half are within {med:.2f}%", f"95% are within {p95:.2f}%",
            f"{alarms} of {len(data)} beyond their LSC", f"= {100 * alarms / len(data):.1f}% (5% is honest)"])
    AA[name] = (len(data), med, p95, alarms)
    return p


write("same-code.svg", [hist(dt, "b1", "trimmed means"), hist(dp, "b2", "plain means")],
      "How far apart two invocations of the same code land, trimmed and plain")

# ---------------------------------------------------------------- levels.svg
hm = [r["mean_ns"] for r in hundred]
ht = K.trimmed(hm)
hs = sorted(hm)
lo, hi = hs[ht["low"]], hs[len(hs) - ht["high"] - 1]
p = Plot(W, 270, 0, 101, 88, 156)
p.yaxis([100, 120, 140], "run mean, ns")
p.xaxis([1, 20, 40, 60, 80, 100], "run, in the order it ran")
p.add(f'<rect class="wash" x="{p.l}" width="{p.w - p.l - p.r}" y="{p.Y(hi):.1f}" height="{p.Y(lo) - p.Y(hi):.1f}"/>'
      f'<line class="e1" x1="{p.l}" x2="{p.w - p.r}" y1="{p.Y(ht["mean"]):.1f}" y2="{p.Y(ht["mean"]):.1f}"/>')
for i, m in enumerate(hm, 1):
    p.dot(i, m, "run", r=3.5)
p.side([f"trimmed mean {ht['mean']:.2f} ns", "band: the runs the", f"trim keeps, {lo:.1f}–{hi:.1f}"], y=p.Y(ht["mean"]) - 16)
write("levels.svg", [p], "A hundred one-second runs in the order they ran, with the band the trim keeps")
SLOW100 = sum(m >= 97.5 for m in hm)

# ---------------------------------------------------------------- blocks.svg
allr = [r for v in inv for r in v["runs"]]
slowrun = max(allr, key=lambda r: r["mean_ns"])
normal = min((r for r in runs_by[slowrun["series"]] if r["mean_ns"] < 97), key=lambda r: abs(r["mean_ns"] - 95))


def blocks(y0, y1, ticks, rs, h, note=None):
    p = Plot(W, h, 0, 101, y0, y1)
    p.yaxis(ticks, "block mean, ns")
    p.xaxis([1, 20, 40, 60, 80, 100], "block, in order, about 55 ms each")
    for r, cls, name in rs:
        b = r["block_mean_ns"]
        p.line(list(enumerate(b, 1)), cls)
        p.side([name, f"mean {r['mean_ns']:.1f} ns"], y=p.Y(b[-1]) + 4)
    if note:
        p.side(note, y=p.h - p.b - 22)
    return p


write("blocks.svg", [blocks(90, 156, [100, 120, 140], [(slowrun, "l2", "a slow-level run"), (normal, "l1", "a normal run")], 230),
                     blocks(93.5, 96.5, [94, 95, 96], [(normal, "l1", "the normal run")], 200, ["", "the same run, zoomed"])],
      "The hundred blocks of a slow-level run and of a normal run, and the normal run zoomed")

# ---------------------------------------------------------------- within-between.svg
core = [r for r in allr if r["mean_ns"] < 97]
neff = K.eff_n([r["block_mean_ns"] for r in core])
b = normal["block_mean_ns"]
se_run = math.sqrt(st.mean(K.var(r["block_mean_ns"]) for r in core) / neff)
left = Plot(W // 2, 250, 0, 101, 93.0, 97.0, right=20)
left.yaxis([94, 95, 96], "ns")
left.xaxis([1, 50, 100], "one run: its 100 blocks")
m = normal["mean_ns"]
left.add(f'<rect class="washg" x="{left.l}" width="{left.w - left.l - left.r}" y="{left.Y(m + se_run):.1f}" height="{left.Y(m - se_run) - left.Y(m + se_run):.1f}"/>')
left.line(list(enumerate(b, 1)), "l1")
left.add(f'<text class="lab" x="{left.l + 8}" y="{left.t + 12}">within a run</text>'
         f'<text class="cap" x="{left.l + 8}" y="{left.t + 26}">band: how well the run knows its own mean, ±{se_run:.2f} ns</text>')
cm = [r["mean_ns"] for r in core]
right = Plot(W - W // 2, 250, 0, len(cm) + 1, 93.0, 97.0, left=20, right=150)
for v in (94, 95, 96):
    right.add(f'<line class="grid" x1="{right.l}" x2="{right.w - right.r}" y1="{right.Y(v):.1f}" y2="{right.Y(v):.1f}"/>')
right.xaxis([], f"{len(cm)} runs on a normal level, in order")
gm, gs = st.mean(cm), st.stdev(cm)
right.add(f'<rect class="washg" x="{right.l}" width="{right.w - right.l - right.r}" y="{right.Y(gm + gs):.1f}" height="{right.Y(gm - gs) - right.Y(gm + gs):.1f}"/>')
for i, v in enumerate(cm, 1):
    right.dot(i, v, "run", r=2.5)
right.add(f'<text class="lab" x="{right.l + 8}" y="{right.t + 12}">between runs</text>'
          f'<text class="cap" x="{right.l + 8}" y="{right.t + 26}">band: how much one run differs from the next, ±{gs:.2f} ns</text>')
right.side(["one scale, both panels", f"within  ±{se_run:.2f} ns", f"between ±{gs:.2f} ns", f"ratio {gs / se_run:.0f} to 1"], y=right.t + 60)
svg_wb = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} 250" width="{W}" height="250" role="img">'
          f"<title>Within-run noise beside between-run spread, on one scale</title>{STYLE}<rect class=\"bg\" width=\"{W}\" height=\"250\"/>"
          f'<g><defs>{left.clip()}</defs>{"".join(left.body)}</g>'
          f'<g transform="translate({W // 2} 0)"><defs>{right.clip()}</defs>{"".join(right.body)}</g></svg>\n')
open(os.path.join(OUT, "within-between.svg"), "w").write(svg_wb)
print(f"  within-between.svg  {len(svg_wb) // 1024} KB")

# ---------------------------------------------------------------- correlation.svg
lags = list(range(1, 31))
obs = [st.mean(K.acf(r["block_mean_ns"], k) for r in core) for k in lags]
p = Plot(W, 250, 0, 31, -0.1, 0.5)
p.yaxis([0, 0.1, 0.2, 0.3, 0.4], "correlation with the block k later")
p.xaxis([1, 5, 10, 15, 20, 25, 30], "k, blocks apart (a block is about 55 ms)")
p.add(f'<line class="axis" x1="{p.l}" x2="{p.w - p.r}" y1="{p.Y(0):.1f}" y2="{p.Y(0):.1f}"/>')
p.line([(k, obs[0] ** k) for k in lags], "l1")
p.line(list(zip(lags, obs)), "l3")
for k, v in zip(lags, obs):
    p.dot(k, v, "s1", r=3)
p.side(["measured", f"worth {neff:.0f} blocks of 100"], y=p.t + 30, key="l3")
p.side(["if it died away", "geometrically", f"worth {100 * (1 - obs[0]) / (1 + obs[0]):.0f} of 100"], y=p.t + 90, key="l1")
write("correlation.svg", [p], "How a block correlates with later blocks, measured and as a geometric decay would have it")

# ---------------------------------------------------------------- run-length.svg
# The model's two noise terms, from the runs on a normal level, the same ones the two figures above
# use: `a` is a run's own variance scaled to one second, `s_p` what is left of the spread between runs.
d_core = st.mean(r["measured_s"] for r in core)
a = d_core * se_run ** 2
s_p = math.sqrt(max(gs ** 2 - se_run ** 2, 0.0))
target_var = (0.005 * gm * 0.4 / (2.0 * math.sqrt(2))) ** 2  # the variance a 0.5% trimmed LSC needs, large-n form
p = Plot(W, 270, 0.05, 10, 0, 720, logx=True)
p.yaxis([120, 240, 360, 480, 600], "seconds to reach the precision")
p.xaxis([0.05, 0.1, 0.2, 0.5, 1, 2, 5, 10], "measured length of one run, seconds (log scale)")
DS = {}
for o, cls, name in ((3.6, "l2", "overhead 3.6 s a run"), (0.3, "l3", "overhead 0.3 s a run")):
    pts = [(d, (s_p ** 2 + a / d) / target_var * (o + d)) for d in (0.05 * 1.06 ** i for i in range(92)) if d <= 10]
    p.add(f'<g clip-path="url(#{p.id})">')
    p.line(pts, cls)
    p.add("</g>")
    ds = math.sqrt(a * o / s_p ** 2)
    t = (s_p ** 2 + a / ds) / target_var * (o + ds)
    p.dot(ds, t, "s1" if cls == "l3" else "run", r=5)
    p.side([name, f"least at {ds:.2f} s, {t:.0f} s in all"], y=p.t + (30 if cls == "l2" else 90), key=cls)
    DS[o] = (ds, t)
write("run-length.svg", [p], "Total time to reach a fixed precision against the length of one run, at two overheads")

# ---------------------------------------------------------------- the numbers the document quotes
print("\nnumbers for docs/statistics.md:")
tm = [v["t"]["mean"] for v in inv]
print(f"  invocations {len(inv)}, trimmed means {min(tm):.2f}-{max(tm):.2f}; plain {min(v['pm'] for v in inv):.2f}-{max(v['pm'] for v in inv):.2f}")
for k, (n, med, p95, al) in AA.items():
    print(f"  same-code, {k}: {n} pairs, median {med:.2f}%, 95th {p95:.2f}%, beyond LSC {al} = {100 * al / n:.1f}%")
print(f"  hundred 1 s runs: slow {SLOW100}, trimmed mean {ht['mean']:.2f}, LSC trimmed {100 * ht['lsc'] / ht['mean']:.2f}%, band {lo:.2f}-{hi:.2f}")
print(f"  within ±{se_run:.3f} ns, between ±{gs:.3f} ns, blocks worth {neff:.0f} of 100, lag-1 {obs[0]:+.2f}, lag-5 {obs[4]:+.2f} (geometric {obs[0] ** 5:+.3f})")
print(f"  model (normal-level runs): a={a:.4f} s_p={s_p:.3f}; d* {DS[3.6][0]:.2f} s at o=3.6 ({DS[3.6][1]:.0f} s), {DS[0.3][0]:.2f} s at o=0.3 ({DS[0.3][1]:.0f} s)")
print(f"  slow run {slowrun['mean_ns']:.1f} ns blocks {min(slowrun['block_mean_ns']):.1f}-{max(slowrun['block_mean_ns']):.1f}; normal {normal['mean_ns']:.1f}")
