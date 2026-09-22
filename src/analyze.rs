//! `analyze`: whether a claim held, read back from records across invocations.
//!
//! Every number a run prints is a claim about that invocation alone, and only repetition shows
//! whether it holds. This module reads records back and makes the claim across invocations, on
//! [`crate::record`]'s struct and [`crate::series`]'s arithmetic, so no second copy of either
//! exists.
//!
//! - **Three units.** A run is one record. An invocation is a series' runs of one bench, with a
//!   trimmed mean and the `LSC trimmed` it claims. A group is the invocations that share a bench,
//!   a host, and the value of every `--by` tag.
//! - **A group against itself** is the first report: how far its invocations' trimmed means
//!   spread, against what each claimed, and so what change the group could really detect.
//! - **Two sides compared** is the second, `--compare KEY=A,B`: the sides are the invocations
//!   whose `KEY` is `A` and `B`, `KEY` any tag or `bench`, `host`, or `file`, and more than two
//!   values make a ladder, each against the first and against the one before. Invocations that
//!   share a series pair by it, sides that alternate in time pair as neighbours, and anything
//!   else compares the two groups whole.
//! - **Session order** is series order, since a series id starts with its UTC start.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::record::{self, AnalyzedRun, Skipped};
use crate::series::{Series, Trim, Trimmed, t975};

/// One invocation of one bench: its runs' means in run order and what they claim.
#[derive(Debug, Clone)]
struct Invocation {
    series: String,
    bench: String,
    host: String,
    /// The name of the file it was read from.
    file: String,
    tags: BTreeMap<String, String>,
    /// Its first run's `t_start`, RFC3339.
    t_start: String,
    /// Its first run's parameter values.
    params: BTreeMap<String, String>,
    /// Run means, ns, in run order.
    means: Vec<f64>,
    /// Each run's mean delivered clock, GHz, for the runs that read one.
    ghz: Vec<f64>,
    /// Each run's block-mean lag-1 autocorrelation, for the runs that have one.
    lag1: Vec<f64>,
}

impl Invocation {
    /// The trimmed pair, `None` when the invocation is too short to trim.
    fn trimmed(&self, trim: Trim) -> Option<Trimmed> {
        Trimmed::of(&self.means, trim)
    }
}

/// Every record under `paths` with the name of the file it came from, a directory being its
/// `*.jsonl` in name order, with what was skipped and how many files were read.
type Collected = (Vec<(String, AnalyzedRun)>, Skipped, usize);

/// Read [`Collected`] from `paths`.
fn collect(paths: &[PathBuf]) -> Result<Collected, String> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            let listing =
                std::fs::read_dir(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
            let mut found: Vec<PathBuf> = listing
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
                .collect();
            found.sort();
            files.extend(found);
        } else {
            files.push(path.clone());
        }
    }
    let mut skipped = Skipped::default();
    let mut runs = Vec::new();
    for file in &files {
        let name = match file.file_name() {
            Some(n) => n.to_string_lossy().into_owned(),
            None => file.display().to_string(),
        };
        for run in record::read_analyzed(file, &mut skipped)? {
            runs.push((name.clone(), run));
        }
    }
    Ok((runs, skipped, files.len()))
}

/// The runs gathered into invocations, a series' runs of one bench each, in run order, and the
/// invocations in session order, which is the map's, keyed by series first.
fn invocations(runs: Vec<(String, AnalyzedRun)>) -> Vec<Invocation> {
    let mut by_key: BTreeMap<(String, String), (String, Vec<AnalyzedRun>)> = BTreeMap::new();
    for (file, run) in runs {
        by_key
            .entry((run.series.clone(), run.bench.clone()))
            .or_insert_with(|| (file, Vec::new()))
            .1
            .push(run);
    }
    by_key
        .into_values()
        .map(|(file, mut runs)| {
            runs.sort_by_key(|r| r.run);
            let first = &runs[0];
            Invocation {
                series: first.series.clone(),
                bench: first.bench.clone(),
                host: first.host.clone(),
                file,
                tags: first.tags.clone(),
                t_start: first.t_start.clone(),
                params: first.params.clone(),
                means: runs.iter().map(|r| r.mean_ns).collect(),
                ghz: runs
                    .iter()
                    .filter(|r| !r.clock_khz.is_empty())
                    .map(|r| {
                        r.clock_khz.iter().sum::<u64>() as f64 / r.clock_khz.len() as f64 / 1e6
                    })
                    .collect(),
                lag1: runs.iter().filter_map(|r| lag1(&r.block_mean_ns)).collect(),
            }
        })
        .collect()
}

/// The lag-1 autocorrelation of a series, `None` below two points or with no spread.
fn lag1(xs: &[f64]) -> Option<f64> {
    if xs.len() < 2 {
        return None;
    }
    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
    let den: f64 = xs.iter().map(|x| (x - mean) * (x - mean)).sum();
    if den == 0.0 {
        return None;
    }
    let num: f64 = xs.windows(2).map(|w| (w[0] - mean) * (w[1] - mean)).sum();
    Some(num / den)
}

/// An invocation's value of `key`: `bench`, `host`, and `file` are its own, and any other key
/// is a tag's, `-` when the records lack it.
fn value(inv: &Invocation, key: &str) -> String {
    match key {
        "bench" => inv.bench.clone(),
        "host" => inv.host.clone(),
        "file" => inv.file.clone(),
        tag => match inv.tags.get(tag) {
            Some(v) => v.clone(),
            None => "-".to_string(),
        },
    }
}

/// A group's key: its value of each dimension, in the dimensions' order.
type Key = Vec<String>;

/// The invocations grouped by their values of `dims`, the groups in key order and each group's
/// invocations in session order.
fn groups(invs: Vec<Invocation>, dims: &[String]) -> Vec<(Key, Vec<Invocation>)> {
    let mut out: BTreeMap<Key, Vec<Invocation>> = BTreeMap::new();
    for inv in invs {
        let key = dims.iter().map(|d| value(&inv, d)).collect();
        out.entry(key).or_default().push(inv);
    }
    out.into_iter().collect()
}

/// A group against itself: the invocations' trimmed means and what they say together.
#[derive(Debug, Clone, PartialEq)]
struct Qualified {
    /// Invocations trimmed, the ones every other number is over.
    inv: usize,
    /// Invocations too short to trim, left out.
    untrimmed: usize,
    /// The mean of the invocations' trimmed means, ns.
    grand: f64,
    /// The invocations' trimmed means, in session order.
    trimmed: Vec<f64>,
    /// Their sample stdev as a percent of `grand`, `None` below two.
    sd_pct: Option<f64>,
    /// Their range as a percent of `grand`.
    range_pct: f64,
    /// The mean claimed `LSC trimmed` as a percent of `grand`.
    lsc_pct: f64,
    /// Pairs of invocations further apart than their claim, and the pairs.
    exceed: (usize, usize),
    /// The spread of the trimmed means over the mean standard error one invocation claimed:
    /// about 1 is an honest claim, well above 1 one that misses what moves between invocations.
    /// `None` below two.
    calib: Option<f64>,
    /// The mean stdev of an invocation's run means as a percent of `grand`.
    run_sd_pct: Option<f64>,
    /// The mean delivered clock over every run, GHz.
    ghz: Option<f64>,
    /// The mean block lag-1 over every run.
    lag1: Option<f64>,
    /// The smallest change one invocation against one would call real, from the spread between
    /// invocations, as a percent of `grand`, `None` below two.
    detect_pct: Option<f64>,
    /// The fitted change over the session, first invocation to last, as a percent of `grand`,
    /// `None` below three.
    trend_pct: Option<f64>,
}

/// The mean of a slice, `None` when it is empty.
fn mean(xs: &[f64]) -> Option<f64> {
    (!xs.is_empty()).then(|| xs.iter().sum::<f64>() / xs.len() as f64)
}

/// Qualify a group against itself under `trim`. `None` when no invocation could be trimmed.
fn qualify(invs: &[Invocation], trim: Trim) -> Option<Qualified> {
    let pairs: Vec<(&Invocation, Trimmed)> = invs
        .iter()
        .filter_map(|inv| inv.trimmed(trim).map(|t| (inv, t)))
        .collect();
    let k = pairs.len();
    let grand = mean(&pairs.iter().map(|(_, t)| t.mean).collect::<Vec<_>>())?;
    let trimmed: Vec<f64> = pairs.iter().map(|(_, t)| t.mean).collect();
    let lscs: Vec<f64> = pairs.iter().map(|(_, t)| t.lsc()).collect();
    let claimed_se = mean(&pairs.iter().map(|(_, t)| t.se()).collect::<Vec<_>>())?;
    let pct = |v: f64| 100.0 * v / grand;
    let spread = Series::of(&trimmed);
    let mut exceed = (0, 0);
    for i in 0..k {
        for j in i + 1..k {
            exceed.1 += 1;
            let claim = ((lscs[i] * lscs[i] + lscs[j] * lscs[j]) / 2.0).sqrt();
            if (trimmed[i] - trimmed[j]).abs() > claim {
                exceed.0 += 1;
            }
        }
    }
    let run_sds: Vec<f64> = pairs
        .iter()
        .filter_map(|(inv, _)| Series::of(&inv.means).map(|s| s.stdev))
        .collect();
    let ghz: Vec<f64> = pairs.iter().flat_map(|(inv, _)| inv.ghz.clone()).collect();
    let lag: Vec<f64> = pairs.iter().flat_map(|(inv, _)| inv.lag1.clone()).collect();
    let (min, max) = trimmed
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &t| {
            (lo.min(t), hi.max(t))
        });
    Some(Qualified {
        inv: k,
        untrimmed: invs.len() - k,
        grand,
        sd_pct: spread.map(|s| pct(s.stdev)),
        range_pct: pct(max - min),
        lsc_pct: pct(mean(&lscs)?),
        exceed,
        calib: spread.map(|s| s.stdev / claimed_se),
        run_sd_pct: mean(&run_sds).map(pct),
        ghz: mean(&ghz),
        lag1: mean(&lag),
        detect_pct: spread.map(|s| pct(t975(s.n - 1) * s.stdev * 2f64.sqrt())),
        trend_pct: trend(&trimmed).map(pct),
        trimmed,
    })
}

/// The least-squares change over a series in order, first point to last, `None` below three
/// points, where a line through two is the two.
fn trend(xs: &[f64]) -> Option<f64> {
    if xs.len() < 3 {
        return None;
    }
    let n = xs.len() as f64;
    let xbar = (n - 1.0) / 2.0;
    let ybar = xs.iter().sum::<f64>() / n;
    let (mut num, mut den) = (0.0, 0.0);
    for (i, y) in xs.iter().enumerate() {
        let dx = i as f64 - xbar;
        num += dx * (y - ybar);
        den += dx * dx;
    }
    Some(num / den * (n - 1.0))
}

/// What the group table's columns are, printed above it.
const HEADER: &str = "
Per group: the grand trimmed mean, the spread and range of the invocations' trimmed means, the
mean claimed LSC trimmed, the pairs further apart than their claim (about 1 in 20 when it holds),
that spread over the error each claimed (calib, about 1 when honest), a run's spread, the clock,
the block lag-1, the change one invocation against one could detect, and the drift over the
session.

";

/// A cell printed to `digits`, `-` when absent.
fn cell(v: Option<f64>, digits: usize) -> String {
    match v {
        Some(v) => format!("{v:.digits$}"),
        None => "-".to_string(),
    }
}

/// The values of one key that are the sides of `--compare`: `KEY=A,B` names them, and `KEY`
/// alone, or a bare `--compare`, which is `bench`, takes every value the records hold.
pub struct Sides {
    /// The key the sides differ in.
    pub key: String,
    /// Its values, the first side first, empty for every value the records hold.
    pub values: Vec<String>,
}

impl Sides {
    /// Parse `KEY`, or `KEY=A,B[,C...]`.
    pub fn parse(spec: &str) -> Result<Sides, String> {
        let bad = || format!("'{spec}' is not KEY or KEY=A,B, as in condition=sleep,nosleep");
        let (key, values) = match spec.split_once('=') {
            Some((key, values)) => (key, Some(values)),
            None => (spec, None),
        };
        let values: Vec<String> = match values {
            None => Vec::new(),
            Some(values) => values
                .split(',')
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .collect(),
        };
        if key.trim().is_empty() || values.len() == 1 || (spec.contains('=') && values.is_empty()) {
            return Err(bad());
        }
        Ok(Sides {
            key: key.trim().to_string(),
            values,
        })
    }
}

/// What to say when `--compare`'s value is not `KEY=A,B` but a path, the records given where the
/// sides belong: the shape with that path in its place, and the benches it holds, so the line
/// can be copied. `None` when `spec` names nothing on disk.
pub fn compare_hint(spec: &str) -> Option<String> {
    let path = PathBuf::from(spec);
    if spec.contains('=') || !path.exists() {
        return None;
    }
    let bin = crate::BIN_NAME;
    let mut hint = format!(
        "'{spec}' is records, and --compare takes the sides: name the records first, as in\n  \
         {bin} analyze {spec} --compare\n\
         which compares every bench, or --compare bench=A,B for two of them"
    );
    if let Ok((runs, _, _)) = collect(&[path]) {
        let benches: std::collections::BTreeSet<String> =
            runs.into_iter().map(|(_, r)| r.bench).collect();
        if !benches.is_empty() {
            let list: Vec<String> = benches.into_iter().collect();
            hint.push_str(&format!("\nIts benches: {}", list.join(", ")));
        }
    }
    Some(hint)
}

/// What the comparisons print: the key, every side in the order first named, and the pairs,
/// indices into `values`, in the order given.
#[derive(Debug)]
struct Plan {
    key: String,
    values: Vec<String>,
    steps: Vec<(usize, usize)>,
}

impl Plan {
    /// The plan of every `--compare`, each adding its pairs: two values one comparison, three
    /// or more a ladder, each against the first and against the one before, and a key alone
    /// every value `invs` holds, in the order each first ran. A pair given twice prints once.
    fn of(specs: &[Sides], invs: &[Invocation]) -> Result<Plan, String> {
        let key = specs[0].key.clone();
        if let Some(other) = specs.iter().find(|s| s.key != key) {
            return Err(format!(
                "--compare {key} and --compare {} compare on different keys: one analysis \
                 compares on one",
                other.key
            ));
        }
        let mut plan = Plan {
            key,
            values: Vec::new(),
            steps: Vec::new(),
        };
        for spec in specs {
            let values = if spec.values.is_empty() {
                let all = values_in_order(invs, &plan.key);
                if all.len() < 2 {
                    return Err(format!(
                        "the records hold {} value{} of {}, {}, and a comparison needs two",
                        all.len(),
                        if all.len() == 1 { "" } else { "s" },
                        plan.key,
                        all.join(", ")
                    ));
                }
                all
            } else {
                spec.values.clone()
            };
            let at: Vec<usize> = values
                .iter()
                .map(|v| match plan.values.iter().position(|x| x == v) {
                    Some(i) => i,
                    None => {
                        plan.values.push(v.clone());
                        plan.values.len() - 1
                    }
                })
                .collect();
            for i in 1..at.len() {
                let mut pairs = vec![(at[0], at[i])];
                if i >= 2 {
                    pairs.push((at[i - 1], at[i]));
                }
                for pair in pairs {
                    if !plan.steps.contains(&pair) {
                        plan.steps.push(pair);
                    }
                }
            }
        }
        Ok(plan)
    }
}

/// The `analyze` command: read `paths`, group by bench, host, and each key in `by`, and print
/// each group against itself under `trim`, or, with `compares`, the sides compared. Returns the
/// exit code.
pub fn run(paths: &[PathBuf], by: &[String], compares: &[Sides], trim: Trim) -> i32 {
    if paths.is_empty() {
        eprintln!("error: analyze: name the record files or directories to read");
        return 2;
    }
    let (runs, skipped, files) = match collect(paths) {
        Ok(read) => read,
        Err(e) => {
            eprintln!("error: analyze: {e}");
            return 1;
        }
    };
    let n_runs = runs.len();
    let invs = invocations(runs);
    let n_invs = invs.len();
    let many_hosts = invs.iter().any(|i| i.host != invs[0].host);
    let mut dims = vec!["bench".to_string(), "host".to_string()];
    dims.extend(
        by.iter()
            .filter(|d| !dims.contains(d))
            .cloned()
            .collect::<Vec<_>>(),
    );
    let plan = if compares.is_empty() {
        None
    } else {
        match Plan::of(compares, &invs) {
            Ok(plan) => {
                dims.retain(|d| *d != plan.key);
                Some(plan)
            }
            Err(e) => {
                eprintln!("error: analyze: --compare: {e}");
                return 2;
            }
        }
    };
    let groups = groups(invs, &dims);
    println!(
        "{n_runs} runs in {files} file{}, {n_invs} invocations, {} groups, trim {trim}",
        if files == 1 { "" } else { "s" },
        groups.len()
    );
    if skipped.unreadable > 0 || skipped.no_series > 0 {
        println!(
            "skipped: {} unreadable lines, {} records with no series",
            skipped.unreadable, skipped.no_series
        );
    }
    let text = match &plan {
        None => report(&groups, &dims, many_hosts, trim),
        Some(plan) => comparison(&groups, &dims, many_hosts, plan, trim),
    };
    print!("{text}");
    0
}

/// Every value of `key` among `invs`, in the order each first ran.
fn values_in_order(invs: &[Invocation], key: &str) -> Vec<String> {
    let mut first: BTreeMap<String, &str> = BTreeMap::new();
    for inv in invs {
        let v = value(inv, key);
        let at = first.entry(v).or_insert(inv.t_start.as_str());
        if inv.t_start.as_str() < *at {
            *at = inv.t_start.as_str();
        }
    }
    let mut values: Vec<(&str, String)> = first.into_iter().map(|(v, t)| (t, v)).collect();
    values.sort();
    values.into_iter().map(|(_, v)| v).collect()
}

/// Column labels for `dims`, the host left out when every invocation is one host's.
fn shown(dims: &[String], hosts: bool) -> Vec<usize> {
    (0..dims.len())
        .filter(|&i| hosts || dims[i] != "host")
        .collect()
}

/// A left-aligned label table: the widths of `heads` and every row of `labels`, two spaces
/// apart.
fn widths(heads: &[String], labels: &[Vec<String>]) -> Vec<usize> {
    (0..heads.len())
        .map(|c| {
            labels
                .iter()
                .map(|l| l[c].len())
                .fold(heads[c].len(), usize::max)
                + 2
        })
        .collect()
}

/// `values` padded to `widths`.
fn padded(values: &[String], widths: &[usize]) -> String {
    values
        .iter()
        .zip(widths)
        .map(|(v, w)| format!("{v:<w$}"))
        .collect()
}

/// The group table and each group's trimmed means in session order.
fn report(groups: &[(Key, Vec<Invocation>)], dims: &[String], hosts: bool, trim: Trim) -> String {
    let qualified: Vec<(&Key, Qualified)> = groups
        .iter()
        .filter_map(|(key, invs)| qualify(invs, trim).map(|q| (key, q)))
        .collect();
    let cols = shown(dims, hosts);
    let heads: Vec<String> = cols.iter().map(|&i| dims[i].clone()).collect();
    let labels: Vec<Vec<String>> = qualified
        .iter()
        .map(|(key, _)| cols.iter().map(|&i| key[i].clone()).collect())
        .collect();
    let widths = widths(&heads, &labels);
    let label = |l: &[String]| padded(l, &widths);
    let mut out = String::new();
    out.push_str(HEADER);
    out.push_str(&format!(
        "{}{:>4}{:>11}{:>8}{:>8}{:>8}{:>8}{:>7}{:>9}{:>7}{:>7}{:>9}{:>8}\n",
        label(&heads),
        "inv",
        "trimmed",
        "sd%",
        "range%",
        "LSC%",
        "exceed",
        "calib",
        "run sd%",
        "GHz",
        "lag1",
        "detect%",
        "trend%"
    ));
    // A blank line parts one bench's rows from the next, where a bench has several.
    let benches = qualified
        .iter()
        .map(|(key, _)| &key[0])
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let part = qualified.len() > benches;
    let mut last: Option<&str> = None;
    for (l, (_, q)) in labels.iter().zip(&qualified) {
        if part && last.is_some_and(|b| b != l[0]) {
            out.push('\n');
        }
        last = Some(&l[0]);
        out.push_str(&format!(
            "{}{:>4}{:>11.3}{:>8}{:>8.2}{:>8.3}{:>5}/{:<2}{:>7}{:>9}{:>7}{:>7}{:>9}{:>8}\n",
            label(l),
            q.inv,
            q.grand,
            cell(q.sd_pct, 3),
            q.range_pct,
            q.lsc_pct,
            q.exceed.0,
            q.exceed.1,
            cell(q.calib, 2),
            cell(q.run_sd_pct, 2),
            cell(q.ghz, 3),
            cell(q.lag1, 2),
            cell(q.detect_pct, 3),
            match q.trend_pct {
                Some(t) => format!("{t:+.2}"),
                None => "-".to_string(),
            },
        ));
    }
    let untrimmed: usize = qualified.iter().map(|(_, q)| q.untrimmed).sum();
    if untrimmed > 0 {
        out.push_str(&format!(
            "\n{untrimmed} invocations too short to trim are left out.\n"
        ));
    }
    out.push_str("\nEach group's trimmed means in session order:\n\n");
    for (l, (_, q)) in labels.iter().zip(&qualified) {
        let means: Vec<String> = q.trimmed.iter().map(|t| format!("{t:.3}")).collect();
        out.push_str(&format!("{}{}\n", label(l), means.join(" ")));
    }
    out
}

/// How a comparison paired the two sides' invocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pairing {
    /// Invocations sharing a series, `n` pairs: a ladder of benches in one invocation.
    Series(usize),
    /// Neighbours in time, `n` pairs, the sides having alternated.
    Alternating(usize),
    /// The two groups whole, `a` and `b` invocations.
    Groups(usize, usize),
    /// One invocation a side, the claim the two made together.
    OneEach,
}

impl std::fmt::Display for Pairing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pairing::Series(n) => write!(f, "series x{n}"),
            Pairing::Alternating(n) => write!(f, "alternating x{n}"),
            Pairing::Groups(a, b) => write!(f, "groups {a}/{b}"),
            Pairing::OneEach => write!(f, "one each"),
        }
    }
}

/// Side B against side A.
#[derive(Debug, Clone, PartialEq)]
struct Compared {
    /// Side A's trimmed mean, over the invocations compared, ns.
    a: f64,
    /// Side B's, ns.
    b: f64,
    /// B less A, ns.
    diff: f64,
    /// The 95% half-width on `diff`, ns: a larger difference is called real.
    claim: f64,
    pairing: Pairing,
}

/// Each invocation's trimmed mean and claim, the untrimmable left out.
fn trimmed_of(invs: &[Invocation], trim: Trim) -> Vec<(&Invocation, Trimmed)> {
    invs.iter()
        .filter_map(|inv| inv.trimmed(trim).map(|t| (inv, t)))
        .collect()
}

/// B against A. Invocations sharing a series pair by it, sides that alternate in time pair as
/// neighbours, and otherwise the two groups are compared whole. `None` when a side has nothing
/// trimmable.
fn compare(a: &[Invocation], b: &[Invocation], trim: Trim) -> Option<Compared> {
    let ta = trimmed_of(a, trim);
    let tb = trimmed_of(b, trim);
    if ta.is_empty() || tb.is_empty() {
        return None;
    }
    // One each: the claim the two invocations make together, as a pair of equal series would.
    if ta.len() == 1 && tb.len() == 1 {
        let (x, y) = (&ta[0].1, &tb[0].1);
        return Some(Compared {
            a: x.mean,
            b: y.mean,
            diff: y.mean - x.mean,
            claim: ((x.lsc() * x.lsc() + y.lsc() * y.lsc()) / 2.0).sqrt(),
            pairing: Pairing::OneEach,
        });
    }
    let shared: Vec<(f64, f64)> = ta
        .iter()
        .filter_map(|(ia, x)| {
            tb.iter()
                .find(|(ib, _)| ib.series == ia.series)
                .map(|(_, y)| (x.mean, y.mean))
        })
        .collect();
    if shared.len() >= 2 {
        return paired(&shared, Pairing::Series(shared.len()));
    }
    if let Some(pairs) = alternating(&ta, &tb) {
        return paired(&pairs, Pairing::Alternating(pairs.len()));
    }
    let xs: Vec<f64> = ta.iter().map(|(_, t)| t.mean).collect();
    let ys: Vec<f64> = tb.iter().map(|(_, t)| t.mean).collect();
    let (ma, mb) = (mean(&xs)?, mean(&ys)?);
    let (ka, kb) = (xs.len() as f64, ys.len() as f64);
    let (se, df) = match (Series::of(&xs), Series::of(&ys)) {
        (Some(x), Some(y)) => {
            let (va, vb) = (x.stdev * x.stdev / ka, y.stdev * y.stdev / kb);
            let se2 = va + vb;
            // Welch-Satterthwaite, rounded down.
            let df = se2 * se2 / (va * va / (ka - 1.0) + vb * vb / (kb - 1.0));
            (se2.sqrt(), if df.is_finite() { df as u64 } else { 1 })
        }
        (Some(s), None) | (None, Some(s)) => {
            ((s.stdev * s.stdev * (1.0 / ka + 1.0 / kb)).sqrt(), s.n - 1)
        }
        (None, None) => return None,
    };
    Some(Compared {
        a: ma,
        b: mb,
        diff: mb - ma,
        claim: t975(df) * se,
        pairing: Pairing::Groups(xs.len(), ys.len()),
    })
}

/// A comparison from `(a, b)` pairs of trimmed means: the mean difference and its 95%
/// half-width, at `pairs - 1` degrees of freedom.
fn paired(pairs: &[(f64, f64)], pairing: Pairing) -> Option<Compared> {
    let d: Vec<f64> = pairs.iter().map(|(x, y)| y - x).collect();
    let s = Series::of(&d)?;
    Some(Compared {
        a: mean(&pairs.iter().map(|p| p.0).collect::<Vec<_>>())?,
        b: mean(&pairs.iter().map(|p| p.1).collect::<Vec<_>>())?,
        diff: s.mean,
        claim: s.ci95(),
        pairing,
    })
}

/// The two sides as neighbour pairs when their invocations alternate in time, A B A B or
/// B A B A, a trailing unpaired one dropped. `None` when they do not alternate or give fewer
/// than two pairs.
fn alternating(
    ta: &[(&Invocation, Trimmed)],
    tb: &[(&Invocation, Trimmed)],
) -> Option<Vec<(f64, f64)>> {
    let mut all: Vec<(&str, bool, f64)> = ta
        .iter()
        .map(|(i, t)| (i.t_start.as_str(), false, t.mean))
        .chain(tb.iter().map(|(i, t)| (i.t_start.as_str(), true, t.mean)))
        .collect();
    all.sort_by(|x, y| x.0.cmp(y.0));
    if all.windows(2).any(|w| w[0].1 == w[1].1) {
        return None;
    }
    let pairs: Vec<(f64, f64)> = all
        .as_chunks::<2>()
        .0
        .iter()
        .map(|[x, y]| if x.1 { (y.2, x.2) } else { (x.2, y.2) })
        .collect();
    (pairs.len() >= 2).then_some(pairs)
}

/// Seconds since the epoch of an RFC3339 UTC time as a record writes it,
/// `2026-09-21T15:40:30.304Z`, `None` when it is not one.
fn epoch_s(t: &str) -> Option<f64> {
    let num = |r: std::ops::Range<usize>| t.get(r)?.parse::<i64>().ok();
    let (y, mo, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let (h, mi) = (num(11..13)?, num(14..16)?);
    let sec: f64 = t.get(17..t.len() - 1)?.parse().ok()?;
    // Howard Hinnant's days_from_civil.
    let y = if mo <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (mo + if mo > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some((days * 86_400 + h * 3_600 + mi * 60) as f64 + sec)
}

/// Hours between two sides that ran one after the other, `None` when their spans overlap.
fn apart_h(a: &[Invocation], b: &[Invocation]) -> Option<f64> {
    let span = |invs: &[Invocation]| {
        let ts: Vec<f64> = invs.iter().filter_map(|i| epoch_s(&i.t_start)).collect();
        let lo = ts.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = ts.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        (lo, hi)
    };
    let ((a0, a1), (b0, b1)) = (span(a), span(b));
    let gap = if a1 < b0 {
        b0 - a1
    } else if b1 < a0 {
        a0 - b1
    } else {
        return None;
    };
    gap.is_finite().then_some(gap / 3_600.0)
}

/// The run parameters the two sides ran differently, each with both sides' values, the keys
/// that name what was run rather than how left out.
fn differing(a: &[Invocation], b: &[Invocation]) -> Vec<String> {
    let values = |invs: &[Invocation], key: &str| {
        invs.iter()
            .filter_map(|i| i.params.get(key).cloned())
            .collect::<std::collections::BTreeSet<_>>()
    };
    let keys: std::collections::BTreeSet<&String> =
        a.iter().chain(b).flat_map(|i| i.params.keys()).collect();
    keys.into_iter()
        .filter(|k| !matches!(k.as_str(), "benches" | "record" | "tags"))
        .filter_map(|k| {
            let (va, vb) = (values(a, k), values(b, k));
            (va != vb).then(|| {
                let join = |v: std::collections::BTreeSet<String>| {
                    v.into_iter().collect::<Vec<_>>().join(" | ")
                };
                format!("{k} ({} / {})", join(va), join(vb))
            })
        })
        .collect()
}

/// One comparison row: its label, the comparison, the parameters the sides ran differently,
/// and the hours between them when they ran apart.
type Row = (Vec<String>, Compared, Vec<String>, Option<f64>);

/// What the comparison table's columns are, printed above it.
const COMPARE_HEADER: &str = "
Per group and pair of sides: each side's trimmed mean, the difference as a percent of A's, the
claim, the smallest difference the pairing could call real, as a percent of A's, the difference
over the claim, and the verdict. Pairing is by series where the sides share one, by neighbours in
time where they alternate, and otherwise of the two groups whole.

";

/// The plan's pairs compared in every group, each B against A.
fn comparison(
    groups: &[(Key, Vec<Invocation>)],
    dims: &[String],
    hosts: bool,
    sides: &Plan,
    trim: Trim,
) -> String {
    let steps = &sides.steps;
    // A side is named by its value, or by a letter and a legend when a value is too long to
    // repeat on every row, as a file name is.
    let long = sides.values.iter().any(|v| v.len() > 24);
    let names: Vec<String> = if long {
        (0..sides.values.len())
            .map(|i| char::from(b'A' + (i % 26) as u8).to_string())
            .collect()
    } else {
        sides.values.clone()
    };
    let cols = shown(dims, hosts);
    let mut heads: Vec<String> = cols.iter().map(|&i| dims[i].clone()).collect();
    heads.push(format!("{} A -> B", sides.key));
    let mut rows: Vec<Row> = Vec::new();
    let mut left_out = 0;
    for (key, invs) in groups {
        let side = |v: &str| -> Vec<Invocation> {
            invs.iter()
                .filter(|i| value(i, &sides.key) == v)
                .cloned()
                .collect()
        };
        for &(i, j) in steps {
            let (a, b) = (side(&sides.values[i]), side(&sides.values[j]));
            // A side with no invocation at all is a group the key does not reach, and one
            // whose invocations are all too short to trim is counted as left out.
            if a.is_empty() || b.is_empty() {
                continue;
            }
            let Some(c) = compare(&a, &b, trim) else {
                left_out += 1;
                continue;
            };
            let mut label: Vec<String> = cols.iter().map(|&c| key[c].clone()).collect();
            label.push(format!("{} -> {}", names[i], names[j]));
            let apart = match c.pairing {
                Pairing::Series(_) => None,
                _ => apart_h(&a, &b).filter(|h| *h >= 1.0),
            };
            rows.push((label, c, differing(&a, &b), apart));
        }
    }
    let labels: Vec<Vec<String>> = rows.iter().map(|r| r.0.clone()).collect();
    let widths = widths(&heads, &labels);
    let mut out = String::new();
    out.push_str(COMPARE_HEADER);
    if long {
        for (name, v) in names.iter().zip(&sides.values) {
            out.push_str(&format!("{name} = {} {v}\n", sides.key));
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "{}{:>11}{:>11}{:>8}{:>8}{:>9}  {:<16}verdict\n",
        padded(&heads, &widths),
        "A trim",
        "B trim",
        "d%",
        "claim%",
        "d/claim",
        "pairing"
    ));
    let mut beyond = 0;
    let mut notes: Vec<(String, String)> = Vec::new();
    for (label, c, differ, apart) in &rows {
        let pct = |v: f64| 100.0 * v / c.a;
        let q = if c.claim > 0.0 {
            c.diff.abs() / c.claim
        } else {
            f64::INFINITY
        };
        let verdict = if c.diff.abs() > c.claim {
            beyond += 1;
            "detected".to_string()
        } else {
            format!("not seen, below {:.2}%", pct(c.claim))
        };
        out.push_str(&format!(
            "{}{:>11.3}{:>11.3}{:>8.2}{:>8.2}{:>9.1}  {:<16}{verdict}\n",
            padded(label, &widths),
            c.a,
            c.b,
            pct(c.diff),
            pct(c.claim),
            q,
            c.pairing.to_string(),
        ));
        let name = label.join(" ");
        if !differ.is_empty() {
            notes.push((
                format!("the sides ran differently: {}", differ.join(", ")),
                name.clone(),
            ));
        }
        if let Some(h) = apart {
            notes.push((
                format!(
                    "the sides ran {h:.1} h apart, and sessions hours apart differ by 0.1 to 0.9% \
                     whatever each claims, so this is a difference between sessions as much as \
                     between the sides"
                ),
                name,
            ));
        }
    }
    out.push_str(&format!(
        "\n{beyond} of {} comparisons beyond their claim.\n",
        rows.len()
    ));
    if left_out > 0 {
        out.push_str(&format!(
            "{left_out} left out: a side had no invocation of five runs or more, the fewest the \
             trim takes.\n"
        ));
    }
    // A note that holds for several comparisons prints once, naming them, or "every
    // comparison" when it holds for all.
    let mut by_note: Vec<(String, Vec<String>)> = Vec::new();
    for (note, name) in notes {
        match by_note.iter_mut().find(|(n, _)| *n == note) {
            Some((_, names)) => names.push(name),
            None => by_note.push((note, vec![name])),
        }
    }
    if !by_note.is_empty() {
        out.push('\n');
        for (note, names) in by_note {
            let who = if names.len() == rows.len() && rows.len() > 1 {
                "every comparison".to_string()
            } else {
                names.join(", ")
            };
            out.push_str(&crate::wrap::wrap(&format!("{who}: {note}"), 100));
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tracked record file, the closed cycle's baseline among them.
    fn tracked(name: &str) -> PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("records")
            .join(name)
    }

    /// The 7600X baseline, from `records/knobs.jsonl`, grouped by condition.
    fn baseline() -> Vec<(Key, Vec<Invocation>)> {
        let (runs, skipped, files) = collect(&[tracked("knobs.jsonl")]).unwrap();
        assert_eq!((skipped, files), (Skipped::default(), 1));
        groups(invocations(runs), &dims(&["condition"]))
    }

    /// The default dimensions and `by`.
    fn dims(by: &[&str]) -> Vec<String> {
        ["bench", "host"]
            .iter()
            .chain(by)
            .map(|d| d.to_string())
            .collect()
    }

    /// The group whose `condition` is `name`.
    fn condition<'a>(groups: &'a [(Key, Vec<Invocation>)], name: &str) -> &'a [Invocation] {
        let (_, invs) = groups
            .iter()
            .find(|(key, _)| key[2] == name)
            .expect("the condition is recorded");
        invs
    }

    #[test]
    fn the_baseline_reproduces_the_closed_cycles_numbers() {
        // knobs.py on the same file: eight invocations of ten runs, their trimmed means 94.67 ns
        // apart by 0.296 ns, 0.31%, where the plain means are apart by 1.818 ns, 1.88%.
        let groups = baseline();
        let invs = condition(&groups, "baseline");
        let q = qualify(invs, Trim::DEFAULT).unwrap();
        assert_eq!((q.inv, q.untrimmed), (8, 0));
        assert!((q.grand - 94.67).abs() < 0.005, "grand {}", q.grand);
        let sd = q.sd_pct.unwrap() * q.grand / 100.0;
        assert!((sd - 0.296).abs() < 0.0005, "sd {sd}");
        let plain: Vec<f64> = invs
            .iter()
            .map(|i| i.means.iter().sum::<f64>() / i.means.len() as f64)
            .collect();
        let p = Series::of(&plain).unwrap();
        assert!((p.stdev - 1.818).abs() < 0.0005, "plain sd {}", p.stdev);
        assert!((100.0 * p.stdev / p.mean - 1.88).abs() < 0.005);
        // knobs.py's calibration: the spread over a claimed standard error of 0.119 ns, 2.49x.
        assert!(
            (q.calib.unwrap() - 2.49).abs() < 0.005,
            "calib {:?}",
            q.calib
        );
        // The first invocation's own claim, knobs.py's first row: 94.52 ns claiming 0.354 ns.
        let first = invs[0].trimmed(Trim::DEFAULT).unwrap();
        assert!((first.mean - 94.52).abs() < 0.005, "first {}", first.mean);
        assert!((first.lsc() - 0.354).abs() < 0.0005, "lsc {}", first.lsc());
    }

    #[test]
    fn the_other_conditions_agree_with_knobs_py_too() {
        let groups = baseline();
        for (name, grand, sd, calib) in [
            ("baseline-quiet", 94.58, 0.239, 1.30),
            ("untouched", 94.59, 0.148, 0.95),
        ] {
            let q = qualify(condition(&groups, name), Trim::DEFAULT).unwrap();
            assert!((q.grand - grand).abs() < 0.005, "{name}: grand {}", q.grand);
            let got = q.sd_pct.unwrap() * q.grand / 100.0;
            assert!((got - sd).abs() < 0.0005, "{name}: sd {got}");
            let got = q.calib.unwrap();
            assert!((got - calib).abs() < 0.005, "{name}: calib {got}");
        }
        // One invocation qualifies nothing beyond its own mean.
        let q = qualify(condition(&groups, "r100-d1s"), Trim::DEFAULT).unwrap();
        assert!((q.grand - 95.30).abs() < 0.005, "grand {}", q.grand);
        assert_eq!(
            (q.sd_pct, q.calib, q.detect_pct, q.trend_pct),
            (None, None, None, None)
        );
    }

    #[test]
    fn lag1_trend_and_exceed_on_known_series() {
        assert_eq!(lag1(&[1.0]), None);
        assert_eq!(lag1(&[2.0, 2.0, 2.0]), None);
        let alternating = lag1(&[1.0, -1.0, 1.0, -1.0]).unwrap();
        assert!((alternating + 0.75).abs() < 1e-12, "{alternating}");
        assert_eq!(trend(&[1.0, 2.0]), None);
        let rising = trend(&[10.0, 11.0, 12.0, 13.0]).unwrap();
        assert!((rising - 3.0).abs() < 1e-12, "{rising}");
        assert!(trend(&[5.0, 5.0, 5.0]).unwrap().abs() < 1e-12);
    }

    #[test]
    fn a_missing_tag_groups_as_a_dash_and_the_report_prints_every_group() {
        let groups = baseline();
        // The one invocation with no condition tag is its own group.
        assert!(
            groups
                .iter()
                .any(|(key, invs)| key[2] == "-" && invs.len() == 1)
        );
        let text = report(&groups, &dims(&["condition"]), false, Trim::DEFAULT);
        for name in ["baseline", "baseline-quiet", "untouched", "r100-d1s"] {
            assert!(
                text.lines().any(|l| l.contains(&format!(" {name} "))),
                "{name} missing:\n{text}"
            );
        }
        assert!(text.contains("exceed") && text.contains("detect%"));
    }

    /// An invocation of `bench` in `series` at `t_start`, its runs `means`, with `params`.
    fn inv(
        series: &str,
        bench: &str,
        t_start: &str,
        means: &[f64],
        params: &[(&str, &str)],
    ) -> Invocation {
        Invocation {
            series: series.to_string(),
            bench: bench.to_string(),
            host: "h".to_string(),
            file: "f.jsonl".to_string(),
            tags: BTreeMap::new(),
            t_start: t_start.to_string(),
            params: params
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            means: means.to_vec(),
            ghz: Vec::new(),
            lag1: Vec::new(),
        }
    }

    /// Ten run means centred on `at`, spread a little so the trim has something to cut.
    fn ten(at: f64) -> Vec<f64> {
        (0..10).map(|i| at + 0.01 * i as f64).collect()
    }

    #[test]
    fn sides_parse_a_key_and_two_or_more_values() {
        let s = Sides::parse("condition=sleep, nosleep").unwrap();
        assert_eq!((s.key.as_str(), s.values.len()), ("condition", 2));
        assert!(Sides::parse("condition=sleep").is_err());
        assert!(Sides::parse("condition=").is_err());
        assert!(Sides::parse("=a,b").is_err());
        // A key alone is every value the records hold.
        let all = Sides::parse("bench").unwrap();
        assert_eq!((all.key.as_str(), all.values.len()), ("bench", 0));
    }

    #[test]
    fn a_record_time_reads_as_epoch_seconds() {
        assert_eq!(epoch_s("1970-01-01T00:00:00.000Z"), Some(0.0));
        assert_eq!(epoch_s("2001-09-09T01:46:40.123Z"), Some(1_000_000_000.123));
        assert_eq!(epoch_s("not a time"), None);
    }

    #[test]
    fn one_each_uses_the_claim_the_two_make_together() {
        let a = [inv("s1", "x", "2026-01-01T00:00:00.000Z", &ten(100.0), &[])];
        let b = [inv("s2", "x", "2026-01-01T01:00:00.000Z", &ten(101.0), &[])];
        let c = compare(&a, &b, Trim::DEFAULT).unwrap();
        assert_eq!(c.pairing, Pairing::OneEach);
        let (x, y) = (
            a[0].trimmed(Trim::DEFAULT).unwrap(),
            b[0].trimmed(Trim::DEFAULT).unwrap(),
        );
        assert!((c.diff - 1.0).abs() < 1e-9);
        let want = ((x.lsc() * x.lsc() + y.lsc() * y.lsc()) / 2.0).sqrt();
        assert!((c.claim - want).abs() < 1e-12);
        assert_eq!(apart_h(&a, &b), Some(1.0));
    }

    #[test]
    fn shared_series_pair_by_series_and_alternating_sides_as_neighbours() {
        // A ladder: both benches in each of three invocations, B a steady 2 above A.
        let a: Vec<Invocation> = (0..3)
            .map(|i| {
                inv(
                    &format!("s{i}"),
                    "v0",
                    "2026-01-01T00:00:00.000Z",
                    &ten(100.0 + i as f64),
                    &[],
                )
            })
            .collect();
        let b: Vec<Invocation> = (0..3)
            .map(|i| {
                inv(
                    &format!("s{i}"),
                    "v1",
                    "2026-01-01T00:00:00.000Z",
                    &ten(102.0 + i as f64 + 0.001 * i as f64),
                    &[],
                )
            })
            .collect();
        let c = compare(&a, &b, Trim::DEFAULT).unwrap();
        assert_eq!(c.pairing, Pairing::Series(3));
        assert!((c.diff - 2.001).abs() < 1e-9, "diff {}", c.diff);
        assert!(
            c.diff.abs() > c.claim,
            "the drift between invocations cancels"
        );
        // Alternating in time, A B A B, drift and all: neighbours pair.
        let times = ["00", "01", "02", "03"];
        let a: Vec<Invocation> = [0, 2]
            .iter()
            .map(|&i| {
                inv(
                    &format!("a{i}"),
                    "x",
                    &format!("2026-01-01T00:{}:00.000Z", times[i]),
                    &ten(100.0 + i as f64),
                    &[],
                )
            })
            .collect();
        let b: Vec<Invocation> = [1, 3]
            .iter()
            .map(|&i| {
                inv(
                    &format!("b{i}"),
                    "x",
                    &format!("2026-01-01T00:{}:00.000Z", times[i]),
                    &ten(100.0 + i as f64),
                    &[],
                )
            })
            .collect();
        assert_eq!(
            compare(&a, &b, Trim::DEFAULT).unwrap().pairing,
            Pairing::Alternating(2)
        );
        // Not alternating, A A B B: the groups compare whole.
        let b2: Vec<Invocation> = b
            .iter()
            .map(|i| Invocation {
                t_start: "2026-01-01T01:00:00.000Z".to_string(),
                ..i.clone()
            })
            .collect();
        assert_eq!(
            compare(&a, &b2, Trim::DEFAULT).unwrap().pairing,
            Pairing::Groups(2, 2)
        );
    }

    #[test]
    fn the_sides_parameters_that_differ_are_named_and_the_bench_list_is_not() {
        let a = [inv(
            "s1",
            "x",
            "t",
            &ten(1.0),
            &[("blocks", "10"), ("benches", "x"), ("runs", "10")],
        )];
        let b = [inv(
            "s2",
            "x",
            "t",
            &ten(1.0),
            &[("blocks", "100"), ("benches", "y"), ("runs", "10")],
        )];
        assert_eq!(differing(&a, &b), ["blocks (10 / 100)"]);
    }

    #[test]
    fn clock_shifts_sleep_shows_no_difference_as_clock_shift_py_found() {
        let (runs, _, _) = collect(&[tracked("clock-shift.jsonl")]).unwrap();
        let groups = groups(invocations(runs), &dims(&[]));
        let sides = Sides::parse("condition=pinned-nosleep,pinned-sleep").unwrap();
        let invs: Vec<Invocation> = groups.iter().flat_map(|(_, i)| i.clone()).collect();
        let plan = Plan::of(&[sides], &invs).unwrap();
        let text = comparison(&groups, &dims(&[]), true, &plan, Trim::DEFAULT);
        let rows: Vec<&str> = text
            .lines()
            .filter(|l| l.contains("alternating x3"))
            .collect();
        assert_eq!(rows.len(), 2, "{text}");
        assert!(rows.iter().all(|r| r.contains("not seen")), "{text}");
    }

    #[test]
    fn records_given_as_the_sides_get_the_line_to_run() {
        let path = tracked("knobs.jsonl");
        let spec = path.to_string_lossy().into_owned();
        let hint = compare_hint(&spec).expect("a path is recognized");
        assert!(
            hint.contains(&format!("analyze {spec} --compare\n")),
            "{hint}"
        );
        assert!(hint.contains("zcr-spsc-v3-2t"), "{hint}");
        assert_eq!(compare_hint("condition=a,b"), None);
        assert_eq!(compare_hint("no/such/path"), None);
    }

    #[test]
    fn unnamed_sides_are_every_value_in_the_order_it_first_ran() {
        let invs = [
            inv("s1", "v1", "2026-01-01T00:00:02.000Z", &ten(1.0), &[]),
            inv("s1", "v0", "2026-01-01T00:00:01.000Z", &ten(1.0), &[]),
            inv("s2", "v0", "2026-01-01T00:01:01.000Z", &ten(1.0), &[]),
            inv("s1", "v2", "2026-01-01T00:00:03.000Z", &ten(1.0), &[]),
        ];
        assert_eq!(values_in_order(&invs, "bench"), ["v0", "v1", "v2"]);
    }

    #[test]
    fn repeated_compares_add_their_pairs_in_the_order_given() {
        let invs = [inv("s1", "a", "2026-01-01T00:00:01.000Z", &ten(1.0), &[])];
        let specs = |list: &[&str]| -> Vec<Sides> {
            list.iter().map(|s| Sides::parse(s).unwrap()).collect()
        };
        // A against B and A against C, and nothing else.
        let plan = Plan::of(&specs(&["bench=a,b", "bench=a,c"]), &invs).unwrap();
        assert_eq!(plan.values, ["a", "b", "c"]);
        assert_eq!(plan.steps, [(0, 1), (0, 2)]);
        // A ladder is each against the first and the one before, and a pair given twice is one.
        let plan = Plan::of(&specs(&["bench=a,b,c", "bench=a,b"]), &invs).unwrap();
        assert_eq!(plan.steps, [(0, 1), (0, 2), (1, 2)]);
        // One analysis compares on one key.
        let err = Plan::of(&specs(&["bench=a,b", "cond=x,y"]), &invs).unwrap_err();
        assert!(err.contains("different keys"), "{err}");
    }
}
