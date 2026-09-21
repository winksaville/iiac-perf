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
//! - **Session order** is series order, since a series id starts with its UTC start.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::record::{self, AnalyzedRun, Skipped};
use crate::series::{Series, Trim, Trimmed, t975};

/// One invocation of one bench: its runs' means in run order and what they claim.
#[derive(Debug, Clone)]
struct Invocation {
    bench: String,
    host: String,
    tags: BTreeMap<String, String>,
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

/// Every record under `paths`, a directory being its `*.jsonl` in name order, with what was
/// skipped and how many files were read.
fn collect(paths: &[PathBuf]) -> Result<(Vec<AnalyzedRun>, Skipped, usize), String> {
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
        runs.extend(record::read_analyzed(file, &mut skipped)?);
    }
    Ok((runs, skipped, files.len()))
}

/// The runs gathered into invocations, a series' runs of one bench each, in run order, and the
/// invocations in session order, which is the map's, keyed by series first.
fn invocations(runs: Vec<AnalyzedRun>) -> Vec<Invocation> {
    let mut by_key: BTreeMap<(String, String), Vec<AnalyzedRun>> = BTreeMap::new();
    for run in runs {
        by_key
            .entry((run.series.clone(), run.bench.clone()))
            .or_default()
            .push(run);
    }
    by_key
        .into_values()
        .map(|mut runs| {
            runs.sort_by_key(|r| r.run);
            let first = &runs[0];
            Invocation {
                bench: first.bench.clone(),
                host: first.host.clone(),
                tags: first.tags.clone(),
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

/// A group's key: its bench, its host, and its value of each `--by` tag, `-` when a record
/// lacks the tag.
type Key = (String, String, Vec<String>);

/// The invocations grouped by bench, host, and each tag in `by`, the groups in key order.
fn groups(invs: Vec<Invocation>, by: &[String]) -> Vec<(Key, Vec<Invocation>)> {
    let mut out: BTreeMap<Key, Vec<Invocation>> = BTreeMap::new();
    for inv in invs {
        let values = by
            .iter()
            .map(|tag| match inv.tags.get(tag) {
                Some(v) => v.clone(),
                None => "-".to_string(),
            })
            .collect();
        out.entry((inv.bench.clone(), inv.host.clone(), values))
            .or_default()
            .push(inv);
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

/// The `analyze` command: read `paths`, group by bench, host, and each tag in `by`, and print
/// each group against itself under `trim`. Returns the exit code.
pub fn run(paths: &[PathBuf], by: &[String], trim: Trim) -> i32 {
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
    let groups = groups(invs, by);
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
    print!("{}", report(&groups, by, many_hosts, trim));
    0
}

/// The group table and each group's trimmed means in session order.
fn report(groups: &[(Key, Vec<Invocation>)], by: &[String], hosts: bool, trim: Trim) -> String {
    let qualified: Vec<(&Key, Qualified)> = groups
        .iter()
        .filter_map(|(key, invs)| qualify(invs, trim).map(|q| (key, q)))
        .collect();
    let mut heads = vec!["bench".to_string()];
    if hosts {
        heads.push("host".to_string());
    }
    heads.extend(by.iter().cloned());
    let labels: Vec<Vec<String>> = qualified
        .iter()
        .map(|((bench, host, values), _)| {
            let mut l = vec![bench.clone()];
            if hosts {
                l.push(host.clone());
            }
            l.extend(values.iter().cloned());
            l
        })
        .collect();
    let widths: Vec<usize> = (0..heads.len())
        .map(|c| {
            labels
                .iter()
                .map(|l| l[c].len())
                .fold(heads[c].len(), usize::max)
                + 2
        })
        .collect();
    let label = |l: &[String]| -> String {
        l.iter()
            .zip(&widths)
            .map(|(v, w)| format!("{v:<w$}"))
            .collect()
    };
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
    let mut last: Option<&str> = None;
    for (l, (_, q)) in labels.iter().zip(&qualified) {
        if last.is_some_and(|b| b != l[0]) {
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
        groups(invocations(runs), &["condition".to_string()])
    }

    /// The group whose `condition` is `name`.
    fn condition<'a>(groups: &'a [(Key, Vec<Invocation>)], name: &str) -> &'a [Invocation] {
        let (_, invs) = groups
            .iter()
            .find(|((_, _, values), _)| values[0] == name)
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
                .any(|((_, _, values), invs)| values[0] == "-" && invs.len() == 1)
        );
        let text = report(&groups, &["condition".to_string()], false, Trim::DEFAULT);
        for name in ["baseline", "baseline-quiet", "untouched", "r100-d1s"] {
            assert!(
                text.lines().any(|l| l.contains(&format!(" {name} "))),
                "{name} missing:\n{text}"
            );
        }
        assert!(text.contains("exceed") && text.contains("detect%"));
    }
}
