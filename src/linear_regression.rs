//! `linear-regression`: what one step costs, read off a line through sample time against `inner`.
//!
//! A sample times `inner` steps back to back inside one timer frame, and a bench's level is
//! that time divided by `inner`, the frame's cost included. This module regresses sample time
//! on `inner` instead, a simple linear regression by ordinary least squares:
//!
//! ```text
//! sample time = intercept + slope x inner
//! ```
//!
//! - **The slope** is the regression coefficient, what one more step adds to a sample.
//! - **The intercept** is what a sample of no steps would take, the frame.
//! - **A residual** is how far a length's sample time sits from the line.
//!
//! The model is tested, not assumed. The regression's two numbers mean what they say only when
//! every step costs the same whatever the sample's length, so the line is refitted over each
//! half of the lengths and the two slopes compared, and the residuals are searched for a
//! period and for slow structure. Where the line does not hold, no cost is named.
//!
//! Two kinds of records feed it, and a group holds one kind:
//!
//! - Runs at fixed counts, `--inner 7`, one length a run, the line fitted across runs.
//! - Runs over a span, `--inner 1-16`, each sample drawing its length, whose totals by length
//!   give every run all the lengths.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use crate::record::{self, RegressionRun, Skipped};
use crate::series::{Trim, Trimmed};

/// The fewest lengths a line is fitted through.
const MIN_LENGTHS: usize = 3;

/// The fewest lengths in each half for the halves to be compared.
const HALF_MIN: usize = 3;

/// The halves' slopes agree when they sit within this percent of the whole line's slope.
const HOLDS_PCT: f64 = 1.0;

/// The halves' slopes are told apart only beyond this many standard errors of their
/// difference, as the runs' scatter at each length gives it.
const NOISE_MULT: f64 = 3.0;

/// The share of the other fold's residual scatter a period must account for to be named.
const PERIOD_MIN_EXPLAINED: f64 = 0.5;

/// The smallest period scoring within this of the best is the one named, since a multiple of
/// the true period scores as well as it does.
const PERIOD_NEAR_BEST: f64 = 0.1;

/// Residuals at neighbouring lengths agree, slow structure, from this lag-1 autocorrelation up.
const SLOW_LAG1: f64 = 0.5;

/// Residuals this small against the sample times are arithmetic rounding, not scatter.
const ROUNDING: f64 = 1e-9;

/// A run's own slope is a stray beyond this percent from the median of its group's.
const STRAY_PCT: f64 = 1.0;

/// The columns the plain version's sentence wraps to, inside its two of indent.
const PLAIN_WIDTH: usize = 96;

/// The most rows a group's table prints, longer spans being thinned evenly.
const MAX_ROWS: usize = 40;

/// A fitted line: sample time = `intercept` + `slope` x length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    /// What one more step adds, ns.
    pub slope: f64,
    /// What a sample of no steps would take, ns.
    pub intercept: f64,
}

impl Line {
    /// The line's sample time at length `x`.
    fn at(&self, x: f64) -> f64 {
        self.intercept + self.slope * x
    }
}

/// The mean of `xs`, 0 for none.
fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

/// The sample standard deviation of `xs`, `None` under two values.
fn stdev(xs: &[f64]) -> Option<f64> {
    if xs.len() < 2 {
        return None;
    }
    let m = mean(xs);
    let var = xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (xs.len() as f64 - 1.0);
    Some(var.sqrt())
}

/// The median of `xs`, `None` for none.
fn median(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    let mut sorted = xs.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    Some(if sorted.len().is_multiple_of(2) {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    })
}

/// The sum of squared distances of the points' lengths from their mean, the regression's
/// `Sxx`.
fn sxx(points: &[(f64, f64)]) -> f64 {
    let mx = mean(&points.iter().map(|p| p.0).collect::<Vec<_>>());
    points.iter().map(|p| (p.0 - mx) * (p.0 - mx)).sum()
}

/// The ordinary least squares line through `points`, each a length and a sample time. `None`
/// under two points or when every length is the same, where no slope exists.
pub fn ols(points: &[(f64, f64)]) -> Option<Line> {
    if points.len() < 2 {
        return None;
    }
    let mx = mean(&points.iter().map(|p| p.0).collect::<Vec<_>>());
    let my = mean(&points.iter().map(|p| p.1).collect::<Vec<_>>());
    let sxx = sxx(points);
    if sxx <= 0.0 {
        return None;
    }
    let sxy: f64 = points.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    let slope = sxy / sxx;
    Some(Line {
        slope,
        intercept: my - slope * mx,
    })
}

/// Each point's residual, its sample time less the line's.
fn residuals(line: &Line, points: &[(f64, f64)]) -> Vec<f64> {
    points.iter().map(|p| p.1 - line.at(p.0)).collect()
}

/// The residual standard deviation, the residuals' root mean square over `n - 2` degrees of
/// freedom, and from it the standard errors of the slope and the intercept. `None` under
/// three points.
fn standard_errors(points: &[(f64, f64)], res: &[f64]) -> Option<(f64, f64, f64)> {
    let n = points.len();
    if n < 3 {
        return None;
    }
    let s = (res.iter().map(|r| r * r).sum::<f64>() / (n as f64 - 2.0)).sqrt();
    let sxx = sxx(points);
    let mx = mean(&points.iter().map(|p| p.0).collect::<Vec<_>>());
    let se_slope = s / sxx.sqrt();
    let se_intercept = s * (1.0 / n as f64 + mx * mx / sxx).sqrt();
    Some((s, se_slope, se_intercept))
}

/// The lag-1 autocorrelation of `res`, whether neighbours agree. `None` under three or when
/// they are all zero.
fn lag1(res: &[f64]) -> Option<f64> {
    if res.len() < 3 {
        return None;
    }
    let den: f64 = res.iter().map(|r| r * r).sum();
    if den <= 0.0 {
        return None;
    }
    let num: f64 = res.windows(2).map(|w| w[0] * w[1]).sum();
    Some(num / den)
}

/// One length's runs: each run's mean sample time at it, ns, and the fold the run is in.
#[derive(Debug, Clone)]
struct Point {
    length: u64,
    runs: Vec<(f64, usize)>,
}

impl Point {
    /// Every run's value.
    fn values(&self) -> Vec<f64> {
        self.runs.iter().map(|r| r.0).collect()
    }

    /// The length's level over its runs: the trimmed mean, how many runs it kept, and its
    /// standard error, or the plain mean and its own where the runs are too few to trim.
    fn level(&self, trim: Trim) -> (f64, usize, f64) {
        let values = self.values();
        match Trimmed::of(&values, trim) {
            Some(t) => (t.mean, t.kept() as usize, t.se()),
            None => {
                let se = match stdev(&values) {
                    Some(sd) => sd / (values.len() as f64).sqrt(),
                    None => 0.0,
                };
                (mean(&values), values.len(), se)
            }
        }
    }

    /// The plain mean over the runs in `fold`, `None` when it has none.
    fn fold_mean(&self, fold: usize) -> Option<f64> {
        let values: Vec<f64> = self
            .runs
            .iter()
            .filter(|r| r.1 == fold)
            .map(|r| r.0)
            .collect();
        (!values.is_empty()).then(|| mean(&values))
    }
}

/// What a group is of: a bench at a host and a placement, over fixed counts or one span.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GroupKey {
    bench: String,
    host: String,
    placement: String,
    /// The span its runs drew from, `None` for runs at fixed counts.
    span: Option<(u64, u64)>,
}

/// A group's runs gathered by length.
#[derive(Debug, Clone)]
struct Group {
    key: GroupKey,
    points: Vec<Point>,
    n_runs: usize,
    /// Each fixed run's standard error of its own sample time, from its blocks, ns.
    block_se: Vec<f64>,
    /// Each spanned run's own slope, ns a step.
    run_slopes: Vec<f64>,
}

/// The fold of each run: by its `pass` tag where the runs carry one, the first value against
/// the rest, and by the run number's parity where they do not.
fn folds(runs: &[&RegressionRun]) -> Vec<usize> {
    let mut passes: Vec<&str> = runs
        .iter()
        .filter_map(|r| r.tags.get("pass").map(String::as_str))
        .collect();
    passes.sort_unstable();
    passes.dedup();
    runs.iter()
        .map(|r| match r.tags.get("pass") {
            Some(p) if passes.len() > 1 => usize::from(p.as_str() != passes[0]),
            _ => (r.run % 2) as usize,
        })
        .collect()
}

/// Gather `runs` into groups, each group's points in length order.
fn groups(runs: &[RegressionRun]) -> Vec<Group> {
    let mut by_key: BTreeMap<GroupKey, Vec<&RegressionRun>> = BTreeMap::new();
    for run in runs {
        let span = (run.inner_lo != run.inner_hi).then_some((run.inner_lo, run.inner_hi));
        by_key
            .entry(GroupKey {
                bench: run.bench.clone(),
                host: run.host.clone(),
                placement: run.placement.clone(),
                span,
            })
            .or_default()
            .push(run);
    }
    by_key
        .into_iter()
        .map(|(key, runs)| {
            let fold = folds(&runs);
            let mut by_length: BTreeMap<u64, Vec<(f64, usize)>> = BTreeMap::new();
            let mut block_se = Vec::new();
            let mut run_slopes = Vec::new();
            for (run, &fold) in runs.iter().zip(&fold) {
                if key.span.is_none() {
                    let length = run.inner_lo as f64;
                    by_length
                        .entry(run.inner_lo)
                        .or_default()
                        .push((run.mean_ns * length, fold));
                    if let Some(sd) = stdev(&run.block_mean_ns) {
                        block_se.push(sd / (run.block_mean_ns.len() as f64).sqrt() * length);
                    }
                } else {
                    let mut own = Vec::new();
                    for ((length, &n), &sum) in (run.inner_lo..)
                        .zip(&run.inner_samples)
                        .zip(&run.inner_sum_ns)
                    {
                        if n > 0 {
                            let sample = sum / n as f64;
                            by_length.entry(length).or_default().push((sample, fold));
                            own.push((length as f64, sample));
                        }
                    }
                    if let Some(line) = ols(&own) {
                        run_slopes.push(line.slope);
                    }
                }
            }
            Group {
                key,
                points: by_length
                    .into_iter()
                    .map(|(length, runs)| Point { length, runs })
                    .collect(),
                n_runs: runs.len(),
                block_se,
                run_slopes,
            }
        })
        .collect()
}

/// Whether a line describes a group, from the slopes of its two halves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// The halves' slopes agree: the slope is a step's cost.
    Holds,
    /// They differ by more than the runs' scatter allows: no cost is named.
    Fails,
    /// The runs' scatter is wider than the agreement asked for and than the difference seen, so
    /// the halves can neither hold the line nor refuse it.
    Noisy,
    /// Too few lengths for two halves.
    Untested,
}

/// The line through each half of the lengths, and how far apart their slopes are.
#[derive(Debug, Clone, Copy)]
struct Halves {
    lower: Line,
    upper: Line,
    /// The slopes' difference as a percent of the whole line's slope.
    apart_pct: f64,
    /// The standard error of the slopes' difference, from the runs' scatter at each length,
    /// as the same percent.
    noise_pct: f64,
}

/// The variance of a least squares slope through `points` whose sample times each carry the
/// standard error beside it.
fn slope_var(points: &[(f64, f64)], se: &[f64]) -> f64 {
    let mx = mean(&points.iter().map(|p| p.0).collect::<Vec<_>>());
    let sxx = sxx(points);
    points
        .iter()
        .zip(se)
        .map(|(p, s)| (p.0 - mx) * (p.0 - mx) * s * s)
        .sum::<f64>()
        / (sxx * sxx)
}

/// Fit each half of `points`, `None` when a half would hold under [`HALF_MIN`] lengths.
fn halves(points: &[(f64, f64)], se: &[f64], whole: &Line) -> Option<Halves> {
    let cut = points.len().div_ceil(2);
    if cut < HALF_MIN || points.len() - cut < HALF_MIN {
        return None;
    }
    let (lo, hi) = points.split_at(cut);
    let (lower, upper) = (ols(lo)?, ols(hi)?);
    let var = slope_var(lo, &se[..cut]) + slope_var(hi, &se[cut..]);
    let scale = 100.0 / whole.slope.abs();
    Some(Halves {
        lower,
        upper,
        apart_pct: (lower.slope - upper.slope).abs() * scale,
        noise_pct: var.sqrt() * scale,
    })
}

/// The halves' verdict.
fn verdict(halves: Option<&Halves>) -> Verdict {
    match halves {
        None => Verdict::Untested,
        // Scatter wider than the agreement asked for could hide a difference or make one.
        Some(h) if NOISE_MULT * h.noise_pct > HOLDS_PCT.max(h.apart_pct) => Verdict::Noisy,
        Some(h) if h.apart_pct <= HOLDS_PCT => Verdict::Holds,
        Some(_) => Verdict::Fails,
    }
}

/// What the search of the residuals for a period found.
#[derive(Debug, Clone, PartialEq)]
enum Period {
    /// The lengths are not consecutive counts, so a length modulo k says nothing.
    NotConsecutive,
    /// Too few lengths for any period to come round three times.
    TooFew,
    /// The runs do not fall into two folds that each hold every length.
    NoFolds,
    /// No period accounts for enough of the other fold. The best is kept, with its share.
    NotFound { best: u64, explained: f64 },
    /// A period, the share of the other fold's residual scatter it accounts for, and the mean
    /// residual at each length modulo it, ns a sample.
    Found {
        k: u64,
        explained: f64,
        offsets: Vec<f64>,
    },
}

/// The mean of `res` at each length modulo `k`.
fn offsets(lengths: &[u64], res: &[f64], k: u64) -> Vec<f64> {
    (0..k)
        .map(|m| {
            let at: Vec<f64> = lengths
                .iter()
                .zip(res)
                .filter(|(l, _)| *l % k == m)
                .map(|(_, r)| *r)
                .collect();
            mean(&at)
        })
        .collect()
}

/// The share of `test`'s scatter the offsets learned from `train` account for, one fold's
/// residuals predicting the other's. Cross-validation: a period that is chance in one fold
/// accounts for nothing in the other, and scores zero or below.
fn explained(lengths: &[u64], train: &[f64], test: &[f64], k: u64) -> f64 {
    let learned = offsets(lengths, train, k);
    let total: f64 = test.iter().map(|r| r * r).sum();
    if total <= 0.0 {
        return 0.0;
    }
    let left: f64 = lengths
        .iter()
        .zip(test)
        .map(|(l, r)| {
            let miss = r - learned[(l % k) as usize];
            miss * miss
        })
        .sum();
    1.0 - left / total
}

/// Search for a period in the residuals of two folds, each fold's sample times given by
/// length and fitted by its own line.
fn period(lengths: &[u64], a: &[f64], b: &[f64]) -> Period {
    if lengths.windows(2).any(|w| w[1] != w[0] + 1) {
        return Period::NotConsecutive;
    }
    let longest = lengths.len() as u64 / 3;
    if longest < 2 {
        return Period::TooFew;
    }
    let fold_residuals = |ys: &[f64]| {
        let points: Vec<(f64, f64)> = lengths.iter().map(|&l| l as f64).zip(ys.to_vec()).collect();
        ols(&points).map(|line| residuals(&line, &points))
    };
    let (Some(ra), Some(rb)) = (fold_residuals(a), fold_residuals(b)) else {
        return Period::NoFolds;
    };
    // Residuals that are rounding alone hold no period, and scoring them would find one.
    let scale = mean(&a.iter().chain(b).map(|y| y.abs()).collect::<Vec<_>>());
    let rms = |res: &[f64]| mean(&res.iter().map(|r| r * r).collect::<Vec<_>>()).sqrt();
    if rms(&ra).max(rms(&rb)) <= ROUNDING * scale {
        return Period::NotFound {
            best: 2,
            explained: 0.0,
        };
    }
    let scores: Vec<(u64, f64)> = (2..=longest)
        .map(|k| {
            let score = (explained(lengths, &ra, &rb, k) + explained(lengths, &rb, &ra, k)) / 2.0;
            (k, score)
        })
        .collect();
    let best =
        scores.iter().copied().fold(
            (2, f64::NEG_INFINITY),
            |acc, s| if s.1 > acc.1 { s } else { acc },
        );
    if best.1 < PERIOD_MIN_EXPLAINED {
        return Period::NotFound {
            best: best.0,
            explained: best.1,
        };
    }
    // OK: `best` is in `scores`, so the search finds it at the latest.
    let (k, explained) = scores
        .iter()
        .copied()
        .find(|s| s.1 >= best.1 - PERIOD_NEAR_BEST)
        .unwrap_or(best);
    let both: Vec<f64> = ra.iter().zip(&rb).map(|(x, y)| (x + y) / 2.0).collect();
    Period::Found {
        k,
        explained,
        offsets: offsets(lengths, &both, k),
    }
}

/// One length's row of a group's table.
#[derive(Debug, Clone)]
struct Row {
    length: u64,
    kept: usize,
    of: usize,
    sample_ns: f64,
    line_ns: f64,
}

/// Everything the regression says of one group.
#[derive(Debug, Clone)]
struct Fitted {
    line: Line,
    /// The residual standard deviation and the standard errors of the slope and the intercept.
    errors: Option<(f64, f64, f64)>,
    /// The line through every run's plain mean, untrimmed.
    plain: Option<Line>,
    /// The median over lengths of the runs' standard deviation at one length, ns a sample.
    run_sd: Option<f64>,
    /// The median over runs of a run's standard error of its own sample time, ns.
    block_se: Option<f64>,
    halves: Option<Halves>,
    verdict: Verdict,
    period: Period,
    lag1: Option<f64>,
    /// The spanned runs' own slopes: lowest, highest, and how many stray from the median.
    run_slopes: Option<(f64, f64, usize, usize)>,
    rows: Vec<Row>,
}

/// Fit `group` under `trim`, `None` when it has under [`MIN_LENGTHS`] lengths.
fn fit(group: &Group, trim: Trim) -> Option<Fitted> {
    if group.points.len() < MIN_LENGTHS {
        return None;
    }
    let levels: Vec<(f64, usize, f64)> = group.points.iter().map(|p| p.level(trim)).collect();
    let points: Vec<(f64, f64)> = group
        .points
        .iter()
        .zip(&levels)
        .map(|(p, l)| (p.length as f64, l.0))
        .collect();
    let se: Vec<f64> = levels.iter().map(|l| l.2).collect();
    let line = ols(&points)?;
    let res = residuals(&line, &points);
    let plain_points: Vec<(f64, f64)> = group
        .points
        .iter()
        .map(|p| (p.length as f64, mean(&p.values())))
        .collect();
    let halves = halves(&points, &se, &line);
    let lengths: Vec<u64> = group.points.iter().map(|p| p.length).collect();
    let fold =
        |f: usize| -> Option<Vec<f64>> { group.points.iter().map(|p| p.fold_mean(f)).collect() };
    let period = match (fold(0), fold(1)) {
        (Some(a), Some(b)) => period(&lengths, &a, &b),
        _ if lengths.windows(2).any(|w| w[1] != w[0] + 1) => Period::NotConsecutive,
        _ => Period::NoFolds,
    };
    let consecutive = period != Period::NotConsecutive;
    let run_sds: Vec<f64> = group
        .points
        .iter()
        .filter_map(|p| stdev(&p.values()))
        .collect();
    let run_slopes = median(&group.run_slopes).map(|mid| {
        let strays = group
            .run_slopes
            .iter()
            .filter(|s| (*s - mid).abs() * 100.0 / mid.abs() > STRAY_PCT)
            .count();
        let lo = group
            .run_slopes
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        let hi = group
            .run_slopes
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        (lo, hi, strays, group.run_slopes.len())
    });
    Some(Fitted {
        line,
        errors: standard_errors(&points, &res),
        plain: ols(&plain_points),
        run_sd: median(&run_sds),
        block_se: median(&group.block_se),
        verdict: verdict(halves.as_ref()),
        halves,
        period,
        lag1: if consecutive { lag1(&res) } else { None },
        run_slopes,
        rows: group
            .points
            .iter()
            .zip(&levels)
            .map(|(p, l)| Row {
                length: p.length,
                kept: l.1,
                of: p.runs.len(),
                sample_ns: l.0,
                line_ns: line.at(p.length as f64),
            })
            .collect(),
    })
}

/// A group's heading: what it is of and how its runs took their lengths.
fn heading(group: &Group) -> String {
    let lengths = group.points.len();
    let (first, last) = match (group.points.first(), group.points.last()) {
        (Some(a), Some(b)) => (a.length, b.length),
        _ => (0, 0),
    };
    let how = match group.key.span {
        Some((lo, hi)) => format!("inner drawn per sample from {lo}-{hi}"),
        None => format!("inner fixed at {lengths} counts, {first} to {last}"),
    };
    format!(
        "bench {}  host {}  placement {}\n  {how}, {} runs\n",
        group.key.bench, group.key.host, group.key.placement, group.n_runs
    )
}

/// The verdict's line: the halves' slopes and what they say.
fn halves_line(f: &Fitted) -> String {
    let Some(h) = &f.halves else {
        return "  halves       not compared, under six lengths, so the line is untested\n"
            .to_string();
    };
    let says = match f.verdict {
        Verdict::Holds => "a line holds".to_string(),
        // Runs that agree exactly at every length have no scatter to be a multiple of.
        Verdict::Fails if h.noise_pct <= 0.0 => "a line does not hold".to_string(),
        Verdict::Fails => format!(
            "a line does not hold, {:.1} times their scatter",
            h.apart_pct / h.noise_pct
        ),
        Verdict::Noisy => "inside the runs' scatter, the line neither held nor refused".to_string(),
        Verdict::Untested => String::new(),
    };
    format!(
        "  halves       {:.3} | {:.3} ns/step, lower | upper lengths, {:.2}% apart\n\
         \x20              {says}\n",
        h.lower.slope, h.upper.slope, h.apart_pct
    )
}

/// The period's line, with its offsets when one is found.
fn period_line(period: &Period) -> String {
    match period {
        Period::NotConsecutive => {
            "  period       not looked for, the lengths are not consecutive\n".to_string()
        }
        Period::TooFew => "  period       not looked for, under six lengths\n".to_string(),
        Period::NoFolds => {
            "  period       not looked for, the runs do not make two folds\n".to_string()
        }
        Period::NotFound { best, explained } => format!(
            "  period       none: the best, {best}, accounts for {:.0}% of the other fold's \
             residual scatter\n",
            explained.max(0.0) * 100.0
        ),
        Period::Found {
            k,
            explained,
            offsets,
        } => {
            let by: Vec<String> = offsets
                .iter()
                .enumerate()
                .map(|(m, o)| format!("{m}: {o:+.1}"))
                .collect();
            format!(
                "  period       {k}, accounting for {:.0}% of the other fold's residual scatter\n\
                 \x20              inner mod {k}, ns a sample:  {}\n",
                explained * 100.0,
                by.join("  ")
            )
        }
    }
}

/// The group's closing sentence in plain words.
fn plain_version(f: &Fitted) -> String {
    let strays = match f.run_slopes {
        Some((_, _, strays, of)) if strays > 0 => {
            format!(
                " {strays} of {of} runs have a slope of their own, so one run is not the answer."
            )
        }
        _ => String::new(),
    };
    let what = match f.verdict {
        Verdict::Holds => format!(
            "each extra step adds {:.3} ns, and a sample costs {:.2} ns beyond its steps.",
            f.line.slope, f.line.intercept
        ),
        Verdict::Fails => {
            let why = match (&f.period, f.lag1) {
                (Period::Found { k, .. }, _) => {
                    format!(" The samples repeat a pattern every {k} steps of length.")
                }
                (_, Some(r)) if r >= SLOW_LAG1 => {
                    " The cost of a step drifts as samples get longer.".to_string()
                }
                _ => String::new(),
            };
            format!(
                "a straight line does not describe these samples, so no cost per step is named.{why}"
            )
        }
        Verdict::Noisy => {
            "the runs scatter too much to say whether a straight line fits.".to_string()
        }
        Verdict::Untested => format!(
            "the line gives {:.3} ns a step, with too few lengths to test whether a line fits.",
            f.line.slope
        ),
    };
    let text = crate::wrap::wrap(&format!("The plain version: {what}{strays}"), PLAIN_WIDTH);
    text.lines().map(|line| format!("  {line}\n")).collect()
}

/// One group's report.
fn report(group: &Group, f: &Fitted) -> String {
    let mut out = heading(group);
    let named = f.verdict == Verdict::Holds;
    let (slope_is, intercept_is) = if named {
        (
            "regression coefficient: what one more step adds",
            "what a sample of no steps would take",
        )
    } else {
        (
            "the line's slope, not a cost",
            "the line's, not an overhead",
        )
    };
    // OK: writing to a String cannot fail, here and through this function.
    let _ = match f.errors {
        Some((_, se_slope, se_intercept)) => writeln!(
            out,
            "  slope        {:.3} ns/step  \u{b1}{:.3}   {slope_is}\n  intercept    {:.2} ns  \
             \u{b1}{:.2}   {intercept_is}",
            f.line.slope,
            2.0 * se_slope,
            f.line.intercept,
            2.0 * se_intercept
        ),
        None => writeln!(
            out,
            "  slope        {:.3} ns/step   {slope_is}\n  intercept    {:.2} ns   {intercept_is}",
            f.line.slope, f.line.intercept
        ),
    };
    if let Some(plain) = &f.plain {
        let _ = writeln!(
            out,
            "  untrimmed    {:.3} ns/step, {:.2} ns   the line through every run's plain mean",
            plain.slope, plain.intercept
        );
    }
    if let Some((s, _, _)) = f.errors {
        let _ = writeln!(
            out,
            "  residual sd  {s:.2} ns   scatter of the lengths about the line"
        );
    }
    if let Some(sd) = f.run_sd {
        let _ = writeln!(
            out,
            "  run sd       {sd:.2} ns   median scatter of the runs at one length"
        );
    }
    if let Some(se) = f.block_se {
        let _ = writeln!(
            out,
            "  block se     {se:.2} ns   median standard error of a run's own sample time, \
             from its blocks"
        );
    }
    if let Some((lo, hi, strays, of)) = f.run_slopes {
        let _ = writeln!(
            out,
            "  run slopes   {lo:.3} to {hi:.3} ns/step, {strays} of {of} more than \
             {STRAY_PCT:.0}% from their median"
        );
    }
    out.push_str(&halves_line(f));
    out.push_str(&period_line(&f.period));
    if let Some(r) = f.lag1 {
        let reads = if r >= SLOW_LAG1 {
            "neighbouring residuals agree, slow structure"
        } else if r <= -SLOW_LAG1 {
            "neighbouring residuals alternate"
        } else {
            "neighbouring residuals are unrelated"
        };
        let _ = writeln!(out, "  lag-1        {r:+.2}   autocorrelation: {reads}");
    }
    out.push_str(&plain_version(f));
    let step = f.rows.len().div_ceil(MAX_ROWS).max(1);
    let _ = writeln!(
        out,
        "\n  {:>5}  {:>7}  {:>11}  {:>11}  {:>9}  {:>9}",
        "inner", "runs", "sample ns", "line ns", "off ns", "level ns"
    );
    for row in f.rows.iter().step_by(step) {
        let _ = writeln!(
            out,
            "  {:>5}  {:>7}  {:>11.2}  {:>11.2}  {:>+9.2}  {:>9.3}",
            row.length,
            format!("{}/{}", row.kept, row.of),
            row.sample_ns,
            row.line_ns,
            row.sample_ns - row.line_ns,
            row.sample_ns / row.length as f64
        );
    }
    if step > 1 {
        let _ = writeln!(out, "  one length in {step} of {} shown", f.rows.len());
    }
    out
}

/// What the columns and the terms mean, printed once above the groups.
const LEGEND: &str = "\
A simple linear regression of sample time on inner, by ordinary least squares, for each bench,
host, placement, and way of taking lengths: sample time = intercept + slope x inner. A length's
sample time is the trimmed mean over its runs. The \u{b1} is two standard errors, from the
residuals, and is a rough guide where the residuals have structure. The halves are the same
regression over the lower and the upper lengths: where their slopes agree within 1% a line
holds, and the slope is a step's cost and the intercept a sample's overhead. A period is
searched for in the residuals by length modulo k, one fold of the runs predicting the other
(cross-validation), the folds being the runs' pass tags or their run numbers' parity. In a
table, runs is kept/of under the trim, off is the residual, and level is sample over inner,
what a run reports today.
";

/// The `linear-regression` command, `lr` for short: read `paths`, group the runs by bench, host,
/// placement, and how they took their lengths, and print each group's regression under `trim`.
/// Returns the exit code.
pub fn run(paths: &[PathBuf], trim: Trim) -> i32 {
    if paths.is_empty() {
        eprintln!("error: linear-regression: name the record files or directories to read");
        return 2;
    }
    let files = match crate::analyze::record_files(paths) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("error: linear-regression: {e}");
            return 1;
        }
    };
    let mut skipped = Skipped::default();
    let mut runs = Vec::new();
    for file in &files {
        match record::read_regression(file, &mut skipped) {
            Ok(read) => runs.extend(read),
            Err(e) => {
                eprintln!("error: linear-regression: {e}");
                return 1;
            }
        }
    }
    let groups = groups(&runs);
    let fitted: Vec<(&Group, Fitted)> = groups
        .iter()
        .filter_map(|g| fit(g, trim).map(|f| (g, f)))
        .collect();
    println!(
        "{} runs in {} file{}, {} groups fitted, trim {trim}",
        runs.len(),
        files.len(),
        if files.len() == 1 { "" } else { "s" },
        fitted.len()
    );
    if skipped.unreadable > 0 {
        println!("skipped: {} unreadable lines", skipped.unreadable);
    }
    let unfitted = groups.len() - fitted.len();
    if unfitted > 0 {
        println!(
            "not fitted: {unfitted} group{} with under {MIN_LENGTHS} values of inner",
            if unfitted == 1 { "" } else { "s" }
        );
    }
    if fitted.is_empty() {
        return 0;
    }
    println!("\n{LEGEND}");
    for (group, f) in &fitted {
        println!("{}", report(group, f));
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed-count run of `bench` whose sample time at `length` is `sample_ns`.
    fn fixed_run(length: u64, sample_ns: f64, run: u64) -> RegressionRun {
        let level = sample_ns / length as f64;
        RegressionRun {
            bench: "b".to_string(),
            host: "h".to_string(),
            placement: "p".to_string(),
            run,
            tags: BTreeMap::new(),
            inner_lo: length,
            inner_hi: length,
            mean_ns: level,
            block_mean_ns: vec![level; 4],
            inner_samples: Vec::new(),
            inner_sum_ns: Vec::new(),
        }
    }

    /// Two runs at each length of `lengths`, their sample times given by `sample`.
    fn fixed_runs(
        lengths: impl Iterator<Item = u64>,
        sample: impl Fn(u64) -> f64,
    ) -> Vec<RegressionRun> {
        lengths
            .flat_map(|l| [fixed_run(l, sample(l), 1), fixed_run(l, sample(l), 2)])
            .collect()
    }

    /// The one group of `runs`, fitted.
    fn fitted(runs: &[RegressionRun]) -> Fitted {
        let groups = groups(runs);
        assert_eq!(groups.len(), 1);
        fit(&groups[0], Trim::DEFAULT).expect("enough lengths to fit")
    }

    #[test]
    fn ols_recovers_a_known_line() {
        let points: Vec<(f64, f64)> = (1..=8)
            .map(|x| (x as f64, 18.4 + 18.27 * x as f64))
            .collect();
        let line = ols(&points).expect("eight points fit");
        assert!((line.slope - 18.27).abs() < 1e-9, "{line:?}");
        assert!((line.intercept - 18.4).abs() < 1e-9, "{line:?}");
        assert_eq!(ols(&points[..1]), None);
        assert_eq!(ols(&[(3.0, 1.0), (3.0, 2.0)]), None);
    }

    #[test]
    fn fixed_runs_on_a_line_hold_and_give_its_slope_and_intercept() {
        let f = fitted(&fixed_runs(1..=12, |l| 18.4 + 18.27 * l as f64));
        assert!((f.line.slope - 18.27).abs() < 1e-9);
        assert!((f.line.intercept - 18.4).abs() < 1e-9);
        assert_eq!(f.verdict, Verdict::Holds);
        assert!(
            matches!(f.period, Period::NotFound { .. }),
            "{:?}",
            f.period
        );
    }

    #[test]
    fn a_curve_fails_the_halves() {
        // A cost a step that falls as samples lengthen: the lower half's slope is steeper.
        let f = fitted(&fixed_runs(1..=12, |l| {
            20.0 + 70.0 * l as f64 - 1.0 * (l * l) as f64
        }));
        assert_eq!(f.verdict, Verdict::Fails);
        assert!(f.lag1.expect("consecutive lengths") >= SLOW_LAG1);
    }

    #[test]
    fn runs_far_apart_at_each_length_neither_hold_nor_refuse_the_line() {
        // Two runs a length, 1,000 ns apart about a line of 70 ns a step: their means sit on
        // the line, and their scatter could hide any difference between the halves.
        let runs: Vec<RegressionRun> = (1..=12u64)
            .flat_map(|l| {
                let line = 20.0 + 70.0 * l as f64;
                [fixed_run(l, line + 500.0, 1), fixed_run(l, line - 500.0, 2)]
            })
            .collect();
        assert_eq!(fitted(&runs).verdict, Verdict::Noisy);
    }

    #[test]
    fn a_period_in_both_folds_is_found_and_the_smallest_named() {
        let offset = |l: u64| [-30.0, 15.0, 0.0, 15.0][(l % 4) as usize];
        let f = fitted(&fixed_runs(1..=24, |l| 20.0 + 70.0 * l as f64 + offset(l)));
        match &f.period {
            Period::Found {
                k,
                explained,
                offsets,
            } => {
                assert_eq!(*k, 4);
                assert!(*explained > 0.9, "{explained}");
                assert!(offsets[0] < -20.0 && offsets[1] > 10.0, "{offsets:?}");
            }
            other => panic!("no period found: {other:?}"),
        }
    }

    #[test]
    fn a_pattern_in_one_fold_alone_is_no_period() {
        // Fold 1 carries a pattern fold 0 lacks, so neither predicts the other.
        let offset = |l: u64| [-30.0, 15.0, 0.0, 15.0][(l % 4) as usize];
        let runs: Vec<RegressionRun> = (1..=24u64)
            .flat_map(|l| {
                let line = 20.0 + 70.0 * l as f64;
                [fixed_run(l, line + offset(l), 1), fixed_run(l, line, 2)]
            })
            .collect();
        let f = fitted(&runs);
        assert!(
            matches!(f.period, Period::NotFound { .. }),
            "{:?}",
            f.period
        );
    }

    #[test]
    fn lengths_apart_are_not_searched_for_a_period() {
        let f = fitted(&fixed_runs((2..=12).map(|l| l * 10), |l| {
            18.4 + 18.27 * l as f64
        }));
        assert_eq!(f.period, Period::NotConsecutive);
        assert_eq!(f.lag1, None);
        assert_eq!(f.verdict, Verdict::Holds);
    }

    #[test]
    fn a_spanned_run_gives_every_length_and_its_own_slope() {
        let run = |n: u64, tag: &str| RegressionRun {
            tags: BTreeMap::from([("pass".to_string(), tag.to_string())]),
            inner_lo: 1,
            inner_hi: 8,
            inner_samples: vec![10; 8],
            inner_sum_ns: (1..=8).map(|l| 10.0 * (5.0 + 3.0 * l as f64)).collect(),
            ..fixed_run(1, 8.0, n)
        };
        let runs = [run(1, "a"), run(2, "b")];
        let groups = groups(&runs);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].key.span, Some((1, 8)));
        assert_eq!(groups[0].points.len(), 8);
        let f = fit(&groups[0], Trim::DEFAULT).expect("eight lengths fit");
        assert!((f.line.slope - 3.0).abs() < 1e-9 && (f.line.intercept - 5.0).abs() < 1e-9);
        let (lo, hi, strays, of) = f.run_slopes.expect("spanned runs have slopes");
        assert!((lo - 3.0).abs() < 1e-9 && (hi - 3.0).abs() < 1e-9);
        assert_eq!((strays, of), (0, 2));
    }

    #[test]
    fn fixed_and_spanned_runs_are_two_groups_and_two_lengths_are_not_fitted() {
        let mut runs = fixed_runs(1..=2, |l| l as f64);
        runs.push(RegressionRun {
            inner_hi: 4,
            inner_samples: vec![1; 4],
            inner_sum_ns: vec![1.0; 4],
            ..fixed_run(1, 1.0, 1)
        });
        let groups = groups(&runs);
        assert_eq!(groups.len(), 2);
        assert!(fit(&groups[0], Trim::DEFAULT).is_none());
    }

    #[test]
    fn a_report_names_no_cost_where_the_line_fails() {
        let runs = fixed_runs(1..=12, |l| 20.0 + 70.0 * l as f64 - 1.0 * (l * l) as f64);
        let groups = groups(&runs);
        let f = fit(&groups[0], Trim::DEFAULT).expect("twelve lengths fit");
        // The plain version wraps, so its sentence is looked for with the breaks taken out.
        let text = report(&groups[0], &f)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(text.contains("a line does not hold"), "{text}");
        assert!(!text.contains("inf"), "{text}");
        assert!(text.contains("no cost per step is named"), "{text}");
        assert!(text.contains("not a cost"), "{text}");
    }
}
