//! Replication across processes: each bench runs `runs` times back to back, every run a fresh
//! child, and the bench's mean, stdev, CI95, and LSC come from the runs' means.
//!
//! Blocks inside one process share the placement the process drew, so their CI95 and LSC are
//! lower bounds on the spread a fresh process shows. A run is one process, and a series of run
//! means is the first replicate that re-rolls placement, so its error bars are the first that
//! cover it ([`crate::series`] owns the arithmetic).
//!
//! - Runs back to back share the host's state for their stretch, its clock above all, so where
//!   the clock drifts the run error bars are still a lower bound on another invocation's reading.
//!   Two unpinned 3900X invocations of `min-now` differed by three times their `LSC runs` while
//!   two pinned ones agreed, so a comparison across invocations wants the clock pinned.
//!
//! - A bench's runs go back to back, not interleaved with another bench's (wink, at the cycle's
//!   opening): a bench is compared by its own error bars, and a drift of the host between two
//!   benches' runs is a bias neither bench's CI95 contains.
//! - A run sleep, a time or a random range, goes before every run, the first included, so no run
//!   starts differently from the others: without one, the first run starts from whatever the
//!   host did before the invocation and the rest start hot from the run before.
//! - One run prints the child's report as a single process did. Several runs print a line per
//!   run as each child finishes, then the bench's summary, and `-v` shows every child's report
//!   as well. The summary's counters come from the runs' records, since a child's report, where
//!   one run shows them, is not shown.

use std::io::Write as _;
use std::path::Path;

use crate::child::{self, RecordSpec, Spec};
use crate::dither::Dither;
use crate::harness::RunCfg;
use crate::inner::InnerSpan;
use crate::record::{self, RunSummary};
use crate::report::{claim_precision, fmt_claim, fmt_commas_f64, print_summary_rows};
use crate::series::{Series, Trim, Trimmed};

/// The run sleep when neither `--run-sleep` nor the config sets one, `(min_s, max_s)` seconds:
/// 100 ms before every run, the project config's and the default since 2026-10-01, the clock pin
/// holding the host still between runs where a second or two of sleep once let it settle.
pub const DEFAULT_RUN_SLEEP_S: (f64, f64) = (0.1, 0.1);

/// How an invocation's runs are spawned and shown.
pub struct Plan<'a> {
    /// The binary each child runs.
    pub exe: &'a Path,
    /// The invocation's scratch directory, for spec and result files.
    pub scratch: &'a Path,
    /// Runs per bench, one or more.
    pub runs: u64,
    /// The sleep before every run, `(min_s, max_s)` seconds.
    pub run_sleep_s: (f64, f64),
    /// Pass `-v` to the children and show their reports beside the run lines.
    pub verbose: bool,
    /// Decimal digits on the time columns, the report's `--decimals`.
    pub decimals: usize,
    /// The band of the run means the trimmed rows keep, `--trim-runs`.
    pub trim_runs: Trim,
    /// Under a span, graph each run's residuals beside the largest, unless `--no-graph`.
    pub graph: bool,
}

/// Spawns the runs of an invocation's benches in order, carrying the run count across benches
/// so the scratch file names span the whole invocation.
pub struct Runner<'a> {
    /// The invocation's plan.
    plan: Plan<'a>,
    /// Draws the run sleeps.
    dither: Dither,
    /// Runs spawned so far in this invocation.
    spawned: u64,
}

impl<'a> Runner<'a> {
    /// A runner for `plan`, nothing spawned yet.
    pub fn new(plan: Plan<'a>) -> Runner<'a> {
        Runner {
            plan,
            dither: Dither::new(),
            spawned: 0,
        }
    }

    /// Run `bench` `runs` times under `cfg`, each run a fresh child recording with `record`, and
    /// print its runs and summary when there are several. Returns the runs' means, empty for a
    /// probe bench, which records nothing.
    pub fn bench(
        &mut self,
        bench: &str,
        cfg: &RunCfg,
        record: &RecordSpec,
    ) -> Result<Vec<f64>, String> {
        let several = self.plan.runs > 1;
        // The span the run table graphs, none for a fixed count or under `--no-graph`.
        let span = cfg
            .inner_override
            .filter(|s| self.plan.graph && !s.is_fixed());
        let roles = crate::benches::find(bench)
            .map(|e| e.roles.threads(0))
            .unwrap_or_default(); // OK: a name no bench has fails in the child, and the line falls back to the pool
        let show_report = !several || self.plan.verbose;
        if several {
            let at = if cfg.pin_cpus.is_empty() { "" } else { "at " };
            println!(
                "{bench}: {} runs, each in a fresh process, {at}{}",
                self.plan.runs,
                crate::pin::threads_placement(&roles, cfg.pin_cpus)
            );
            println!();
        }
        let mut all: Vec<RunSummary> = Vec::with_capacity(self.plan.runs as usize);
        // Each run's number and its summaries, for the table after the last run.
        let mut table: Vec<(u64, Vec<RunSummary>)> = Vec::with_capacity(self.plan.runs as usize);
        for run in 1..=self.plan.runs {
            if self.plan.run_sleep_s.1 > 0.0 {
                let s = self.dither.span_s(self.plan.run_sleep_s);
                std::thread::sleep(std::time::Duration::from_secs_f64(s));
            }
            let n = self.spawned;
            self.spawned += 1;
            let result = self.plan.scratch.join(format!("run-{n}.jsonl"));
            let spec = Spec::new(bench, run, cfg, record, result.clone());
            let spec_path = self.plan.scratch.join(format!("run-{n}.json"));
            child::spawn(
                self.plan.exe,
                &spec_path,
                &spec,
                show_report,
                self.plan.verbose,
            )?;
            let summaries = record::read_summaries(&result)?;
            // A child's report says a run is done under `-v`, and the progress line otherwise.
            if several && !show_report {
                print!("{}", progress_mark(run));
                // A mark that cannot be flushed shows with the next line.
                let _ = std::io::stdout().flush();
            }
            all.extend(summaries.iter().cloned());
            table.push((run, summaries));
        }
        // The table waits for the last run: a run line drawn between runs, its graph's block
        // characters above all, costs the terminal enough to disturb the next run's measurement.
        if several {
            // The progress line wants ending and a blank, and a child's report ends on its own.
            if !show_report {
                println!();
                println!();
            }
            println!("{}", run_header(span));
            for (run, summaries) in &table {
                if summaries.is_empty() {
                    println!("{run:>5}  no record: a probe bench records nothing");
                }
                for s in summaries {
                    println!("{}", run_line(*run, s, self.plan.decimals, span));
                }
            }
        }
        if several {
            let means: Vec<f64> = all.iter().map(|s| s.mean_ns).collect();
            let rows = summary_rows(&means, self.plan.decimals, self.plan.trim_runs);
            println!();
            print_summary_rows(&rows);
            if let Some(t) = Trimmed::of(&means, self.plan.trim_runs) {
                println!(
                    "{}",
                    aux_line(&rows, "trimmed", &trimmed_runs(&t, self.plan.trim_runs))
                );
            }
            println!(
                "{}",
                aux_line(&rows, "clock", &clock_cell(clock_across(&all)))
            );
            if let Some((inner, samples)) = sizing_across(&all) {
                println!("{}", aux_line(&rows, "inner", &inner));
                println!("{}", aux_line(&rows, "samples", &samples));
            }
            if let Some(cell) = counters_across(&all) {
                println!("{}", aux_line(&rows, "counters", &cell));
            }
            println!();
        }
        Ok(all.iter().map(|s| s.mean_ns).collect())
    }
}

/// What one bench read at one placement in a run of several placements.
#[derive(Debug, Clone, PartialEq)]
enum Cell {
    /// Not reached, which a finished invocation never leaves.
    Pending,
    /// The runs' means.
    Ran(Vec<f64>),
    /// Not run, since its threads would use the CPUs they used at the placement of this column.
    Same(usize),
}

/// The benches by placements table a run of several placements ends with: a row per bench, a
/// column per placement, each cell the mean of the runs' means and its CI95, so the menu of what
/// each ring costs at each placement is one table.
pub struct PlacementTable {
    /// The column heads, each placement's label.
    placements: Vec<String>,
    /// The row heads, each bench's name.
    benches: Vec<&'static str>,
    /// The cells, row-major.
    cells: Vec<Vec<Cell>>,
}

impl PlacementTable {
    /// An empty table over `placements` and `benches`.
    pub fn new(placements: &[crate::pin::Placement], benches: &[&crate::benches::Entry]) -> Self {
        PlacementTable {
            placements: placements.iter().map(|p| p.label()).collect(),
            benches: benches.iter().map(|e| e.name).collect(),
            cells: vec![vec![Cell::Pending; placements.len()]; benches.len()],
        }
    }

    /// Record the runs' means of the bench at `row` at the placement at `col`.
    pub fn ran(&mut self, row: usize, col: usize, means: Vec<f64>) {
        self.cells[row][col] = Cell::Ran(means);
    }

    /// Record that the bench at `row` was not run at `col`, its threads using the CPUs they
    /// used at `earlier`.
    pub fn same(&mut self, row: usize, col: usize, earlier: usize) {
        self.cells[row][col] = Cell::Same(earlier);
    }

    /// The table as lines: a heading, the column heads, and a row per bench. A cell is the mean
    /// and `±` its CI95 over the runs, the mean alone for one run, `= smt` for a bench not run
    /// again, and `-` for a probe bench, which records nothing.
    fn lines(&self, decimals: usize) -> Vec<String> {
        let cell = |c: &Cell| match c {
            Cell::Pending => "-".to_string(),
            Cell::Same(at) => format!("= {}", self.placements[*at]),
            Cell::Ran(means) => match (Series::of(means), means.as_slice()) {
                (None, [one]) => fmt_commas_f64(*one, decimals),
                (None, _) => "-".to_string(),
                (Some(s), _) => {
                    let ci = fmt_claim(s.ci95(), decimals.max(1));
                    let d = claim_precision(decimals, &[&ci]);
                    format!("{} ±{ci}", fmt_commas_f64(s.mean, d))
                }
            },
        };
        let rows: Vec<Vec<String>> = self
            .cells
            .iter()
            .map(|r| r.iter().map(cell).collect())
            .collect();
        // Widths in chars, since `±` is two bytes and `format!` pads by chars.
        let name_w = self.benches.iter().map(|b| b.len()).fold(5, usize::max);
        let widths: Vec<usize> = (0..self.placements.len())
            .map(|c| {
                rows.iter()
                    .map(|r| r[c].chars().count())
                    .fold(self.placements[c].chars().count(), usize::max)
            })
            .collect();
        let line = |head: &str, cells: &[String]| {
            let cols: Vec<String> = cells
                .iter()
                .zip(&widths)
                .map(|(v, w)| format!("{v:>w$}"))
                .collect();
            format!("  {head:<name_w$}  {}", cols.join("  "))
                .trim_end()
                .to_string()
        };
        let mut out = vec![
            "Placements: the mean of the runs' means and its CI95, ns".to_string(),
            String::new(),
            line("bench", &self.placements),
        ];
        for (b, r) in self.benches.iter().zip(&rows) {
            out.push(line(b, r));
        }
        out
    }

    /// Print the table.
    pub fn print(&self, decimals: usize) {
        for l in self.lines(decimals) {
            println!("{l}");
        }
        println!();
    }
}

/// The run table's header, over [`run_line`]'s columns. The ns headers carry [`point_cell`]'s
/// own pad, two columns for the usual one or two decimals, so a header sits over its values
/// rather than over the padding that lines their decimal points up.
fn run_header(span: Option<InnerSpan>) -> String {
    let (title, width, residual) = inner_columns(span);
    let residual = residual.map_or(String::new(), |w| format!("  {:>w$}", RESIDUAL_TITLE));
    format!(
        "{:>5}  {:>8}  {title:>width$}{residual}  {:>10}  {:>12}    {:>12}    {:>12}    {:>15}",
        "run", "pid", "samples", "mean", "stdev blocks", "resolution", "clock"
    )
}

/// The title over a span's largest residual.
const RESIDUAL_TITLE: &str = "residual";

/// The `inner` column's title and width, and the `residual` column's width when there is one:
/// `inner` over each run's count, or `inner LO-HI` over each run's graph of how its lengths leave
/// its line and `residual` over the largest distance, when `--inner` gave a span.
fn inner_columns(span: Option<InnerSpan>) -> (String, usize, Option<usize>) {
    match span {
        Some(s) if !s.is_fixed() => {
            let title = format!("inner {s}");
            let width = title.len().max(graph_width(s.hi() - s.lo() + 1));
            // Room for `+99.999 ns at 16`.
            (title, width, Some(RESIDUAL_TITLE.len().max(16)))
        }
        _ => ("inner".to_string(), 7, None),
    }
}

/// The most bars a run's length graph draws.
const GRAPH_BARS: u64 = 16;

/// The lengths each bar of a span of `lengths` covers, all bars alike but the last, so no more
/// than [`GRAPH_BARS`] bars draw it.
fn graph_step(lengths: u64) -> u64 {
    lengths.div_ceil(GRAPH_BARS).max(1)
}

/// The bars a span of `lengths` draws.
fn graph_width(lengths: u64) -> usize {
    lengths.div_ceil(graph_step(lengths)) as usize
}

/// How a run's lengths leave its own straight line, the lowest length first: each length's mean
/// sample time less the least squares line's through them, as `linear-regression` fits it, and
/// the largest such distance, signed and at its length, `+0.412 ns at 8`, the graph's cell and
/// the residual's.
///
/// - Each bar is a length, or a few alike when the span is wider than [`GRAPH_BARS`], at the mean
///   of their residuals.
/// - The bars span the run's own lowest residual to its highest, so the shape shows whatever its
///   size, and the residual beside it says that size: a bench on its line draws noise at a small
///   residual, and one whose cost repeats every few lengths draws the repeat.
/// - The residual is the largest by size, `+` where that length ran slower than the line and `-`
///   where it ran faster, a least squares line always having some of each.
/// - A bar no sample drew is a space, and a run with under two lengths drawn has no line, `-` in
///   both cells.
fn line_graph(lo: u64, counts: &[u64], sums_ns: &[f64], decimals: usize) -> (String, String) {
    const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let points: Vec<(f64, f64)> = counts
        .iter()
        .zip(sums_ns)
        .enumerate()
        .filter(|(_, (n, _))| **n > 0)
        .map(|(i, (n, sum))| ((lo + i as u64) as f64, sum / *n as f64))
        .collect();
    let Some(line) = crate::linear_regression::ols(&points) else {
        return ("-".to_string(), "-".to_string());
    };
    let res = crate::linear_regression::residuals(&line, &points);
    // Each length's residual, `None` where no sample drew it.
    let mut by_length: Vec<Option<f64>> = vec![None; counts.len()];
    for ((x, _), r) in points.iter().zip(&res) {
        by_length[(*x as u64 - lo) as usize] = Some(*r);
    }
    let step = graph_step(counts.len() as u64) as usize;
    let bars: Vec<Option<f64>> = by_length
        .chunks(step)
        .map(|c| {
            let drawn: Vec<f64> = c.iter().flatten().copied().collect();
            (!drawn.is_empty()).then(|| drawn.iter().sum::<f64>() / drawn.len() as f64)
        })
        .collect();
    let (low, high) = bars
        .iter()
        .flatten()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), &r| {
            (l.min(r), h.max(r))
        });
    let graph: String = bars
        .iter()
        .map(|bar| match bar {
            None => ' ',
            Some(_) if high <= low => BARS[3],
            Some(r) => BARS[(((r - low) / (high - low)) * 7.0).round() as usize],
        })
        .collect();
    // The largest residual by size, signed, and the length it is at.
    let (at, worst) = points
        .iter()
        .zip(&res)
        .map(|((x, _), r)| (*x as u64, *r))
        .fold((lo, 0.0_f64), |best, p| {
            if p.1.abs() > best.1.abs() { p } else { best }
        });
    let sign = if worst < 0.0 { '-' } else { '+' };
    let size = fmt_claim(worst.abs(), decimals.max(1));
    // The length in two columns, so the cells line up on `ns` up to a span of 99.
    (graph, format!("{sign}{size} ns at {at:>2}"))
}

/// One run's line: its index, the child's pid, the steps a sample timed and the samples, as the
/// run sized them, the run's mean, the stdev of its block means, its resolution, and its delivered
/// clock. Under a span the steps are the graph of how its lengths leave its line, and the largest
/// residual follows. A run that sized itself to another `inner` than its neighbours shows it
/// here. The stdev says how far the run's blocks wandered and the resolution whether they drifted,
/// so a run on another level reads as an off mean with a small stdev and a run that moved as a
/// resolution well above its neighbours'. The mean prints at least as precisely as the two.
fn run_line(run: u64, s: &RunSummary, decimals: usize, span: Option<InnerSpan>) -> String {
    let claim = |v: Option<f64>| match v {
        Some(x) => fmt_claim(x, decimals.max(1)),
        None => "-".to_string(),
    };
    let (stdev, resolution) = (claim(s.block_stdev_ns), claim(s.resolution_ns));
    let mean_decimals = claim_precision(decimals, &[&stdev, &resolution]);
    let (_, width, residual_width) = inner_columns(span);
    let (inner, residual) = match residual_width {
        Some(w) if s.inner.0 != s.inner.1 => {
            let (graph, reach) = line_graph(s.inner.0, &s.inner_samples, &s.inner_sum_ns, decimals);
            (graph, format!("  {reach:>w$}"))
        }
        Some(w) => (inner_cell(s.inner), format!("  {:>w$}", "-")),
        None => (inner_cell(s.inner), String::new()),
    };
    format!(
        "{run:>5}  {:>8}  {inner:>width$}{residual}  {:>10}  {:>14}  {:>14}  {:>14}  {:>15}",
        s.pid,
        crate::report::fmt_commas(s.samples),
        point_cell(fmt_commas_f64(s.mean_ns, mean_decimals)),
        point_cell(stdev),
        point_cell(resolution),
        clock_cell(s.clock_ghz),
    )
    .trim_end()
    .to_string()
}

/// A delivered clock range, `(min, max)` GHz, as a run line or the summary prints it: one number
/// when the range holds within the stability tolerance, as a pinned clock's should, the range
/// otherwise, and `-` when the host exposes no readable clock.
fn clock_cell(clock: Option<(f64, f64)>) -> String {
    match clock {
        None => "-".to_string(),
        Some((lo, hi)) if hi <= 0.0 || (hi - lo) / hi <= crate::freq::FREQ_STABLE_TOL => {
            format!("{:.2} GHz", (lo + hi) / 2.0)
        }
        Some((lo, hi)) => format!("{lo:.2}-{hi:.2} GHz"),
    }
}

/// The steps a run's samples timed: one count, or the span each sample drew from, `1-16`.
fn inner_cell((lo, hi): (u64, u64)) -> String {
    if lo == hi {
        lo.to_string()
    } else {
        format!("{lo}-{hi}")
    }
}

/// The inner counts and the sample counts across a bench's runs, each one value when every run
/// agrees and `lo to hi` when they differ, the way `counters` reads. `None` with no runs.
fn sizing_across(summaries: &[RunSummary]) -> Option<(String, String)> {
    let first = summaries.first()?;
    let inner = if summaries.iter().all(|s| s.inner == first.inner) {
        inner_cell(first.inner)
    } else {
        let lo = summaries.iter().map(|s| s.inner.0).fold(u64::MAX, u64::min);
        let hi = summaries.iter().map(|s| s.inner.1).fold(0, u64::max);
        format!("{lo} to {hi}")
    };
    let fewest = summaries.iter().map(|s| s.samples).fold(u64::MAX, u64::min);
    let most = summaries.iter().map(|s| s.samples).fold(0, u64::max);
    let commas = crate::report::fmt_commas;
    let samples = if fewest == most {
        commas(fewest)
    } else {
        format!("{} to {}", commas(fewest), commas(most))
    };
    Some((inner, samples))
}

/// The runs the progress line counts on one line before it starts another.
const PROGRESS_PER_LINE: u64 = 20;

/// What the progress line prints as `run` finishes: `run 1`, then `, 2` and on, a new `run 21`
/// line every [`PROGRESS_PER_LINE`] runs.
fn progress_mark(run: u64) -> String {
    match (run - 1) % PROGRESS_PER_LINE {
        0 if run == 1 => "run 1".to_string(),
        0 => format!("\nrun {run}"),
        _ => format!(", {run}"),
    }
}

/// The clock range across a bench's runs: the lowest and highest any run's dominant core read.
fn clock_across(summaries: &[RunSummary]) -> Option<(f64, f64)> {
    summaries
        .iter()
        .filter_map(|s| s.clock_ghz)
        .reduce(|(lo, hi), (l, h)| (lo.min(l), hi.max(h)))
}

/// A run-line cell: the value and its unit, padded after the unit so a right-aligned column lines
/// up on the decimal point whether the value carries 0 or up to 3 decimals. A withheld `-` stands
/// where the point would be.
fn point_cell(v: String) -> String {
    if v == "-" {
        return "-      ".to_string();
    }
    let frac = match v.split_once('.') {
        Some((_, f)) => f.len() + 1,
        None => 0,
    };
    format!("{v} ns{}", " ".repeat(4usize.saturating_sub(frac)))
}

/// The runs' counters as one cell, `None` when no run counted anything: each counter's value
/// when every run read the same, its lowest and highest otherwise, a run that lacks a counter
/// reading 0, so a count that moves between runs shows as a range where a steady one is a number.
fn counters_across(summaries: &[RunSummary]) -> Option<String> {
    let names: std::collections::BTreeSet<&String> =
        summaries.iter().flat_map(|s| s.counters.keys()).collect();
    if names.is_empty() {
        return None;
    }
    let cells: Vec<String> = names
        .into_iter()
        .map(|name| {
            let values = summaries
                .iter()
                // OK: a run without the counter counted none of it, as a record from
                // before schema 9 reads.
                .map(|s| s.counters.get(name).copied().unwrap_or(0));
            let (lo, hi) = values.fold((u64::MAX, 0), |(lo, hi), v| (lo.min(v), hi.max(v)));
            if lo == hi {
                format!("{name} {lo}")
            } else {
                format!("{name} {lo} to {hi}")
            }
        })
        .collect();
    Some(cells.join(", "))
}

/// A summary line that carries no ns value, its label padded to `rows`' labels: the clock range,
/// and which runs the trim dropped.
fn aux_line(rows: &[(String, String)], label: &str, value: &str) -> String {
    let width = rows.iter().map(|(l, _)| l.len()).fold(0, usize::max);
    format!("  {label:<width$}  {value}")
}

/// The trim, how many runs it kept, and which it dropped from each end, as 1-based run numbers
/// in the order the runs ran, for the line under the trimmed rows.
fn trimmed_runs(t: &Trimmed, trim: Trim) -> String {
    let list = |runs: &[usize]| match runs {
        [] => "none".to_string(),
        _ => {
            let runs: Vec<String> = runs.iter().map(|i| (i + 1).to_string()).collect();
            runs.join(", ")
        }
    };
    format!(
        "{trim} keeps {} of {} runs, drops low: {}, high: {}",
        t.kept(),
        t.n,
        list(&t.dropped_low),
        list(&t.dropped_high)
    )
}

/// A bench's summary rows over its run means: the plain mean, stdev, CI95, and LSC, then the
/// trimmed pair when the series is long enough to trim, each `-` below two runs, where no spread
/// exists. Every mean and stdev prints at least as precisely as the claims beside it.
///
/// - The plain rows answer what a run of this bench costs, a disturbed run included, since a
///   disturbance a host really produces is part of what a run draws.
/// - The trimmed rows answer whether a change moved the bench, where a run the host disturbed is
///   noise about the code ([`Trimmed`] for the arithmetic, and why the stdev is winsorized while
///   the mean is trimmed).
fn summary_rows(means: &[f64], decimals: usize, trim: Trim) -> Vec<(String, String)> {
    let dash = || "-".to_string();
    let claim = |v: f64| fmt_claim(v, decimals.max(1));
    let plain = Series::of(means);
    let trimmed = Trimmed::of(means, trim);
    let claims: Vec<String> = plain
        .iter()
        .flat_map(|s| [claim(s.ci95()), claim(s.lsc())])
        .chain(
            trimmed
                .iter()
                .flat_map(|t| [claim(t.ci95()), claim(t.lsc())]),
        )
        .collect();
    let refs: Vec<&str> = claims.iter().map(String::as_str).collect();
    let d = claim_precision(decimals, &refs);
    let mut rows = match &plain {
        Some(s) => vec![
            ("mean".to_string(), fmt_commas_f64(s.mean, d)),
            ("stdev".to_string(), fmt_commas_f64(s.stdev, d)),
            ("CI95 runs".to_string(), claim(s.ci95())),
            ("LSC runs".to_string(), claim(s.lsc())),
        ],
        None => vec![
            ("mean".to_string(), dash()),
            ("stdev".to_string(), dash()),
            ("CI95 runs".to_string(), dash()),
            ("LSC runs".to_string(), dash()),
        ],
    };
    if let Some(t) = &trimmed {
        rows.extend([
            ("trimmed mean".to_string(), fmt_commas_f64(t.mean, d)),
            (
                "winsorized stdev".to_string(),
                fmt_commas_f64(t.winsorized_stdev, d),
            ),
            ("CI95 trimmed".to_string(), claim(t.ci95())),
            ("LSC trimmed".to_string(), claim(t.lsc())),
        ]);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_placement_table_names_a_bench_not_run_again_by_its_placement() {
        let t = PlacementTable {
            placements: vec!["smt".to_string(), "ccx".to_string()],
            benches: vec!["zcr-spsc-v4-1t", "zcr-spsc-v4-2t", "producer-consumer"],
            cells: vec![
                vec![Cell::Ran(vec![12.5]), Cell::Same(0)],
                vec![Cell::Ran(vec![56.0, 56.2]), Cell::Ran(vec![104.0, 106.0])],
                vec![Cell::Ran(vec![]), Cell::Ran(vec![])],
            ],
        };
        let lines = t.lines(1);
        let words = |l: &str| l.split_whitespace().map(str::to_string).collect::<Vec<_>>();
        assert_eq!(words(&lines[2]), ["bench", "smt", "ccx"]);
        assert_eq!(words(&lines[3]), ["zcr-spsc-v4-1t", "12.5", "=", "smt"]);
        let row = words(&lines[4]);
        assert!(
            row[1].starts_with("56.1") && row[2].starts_with('±'),
            "{}",
            lines[4]
        );
        assert!(
            row[3].starts_with("105.0") && row[4].starts_with('±'),
            "{}",
            lines[4]
        );
        assert_eq!(words(&lines[5]), ["producer-consumer", "-", "-"]);
        // A row of `±` cells ends where the column heads do, `±` counting as one column.
        assert_eq!(lines[2].chars().count(), lines[4].chars().count());
    }

    #[test]
    fn summary_rows_come_from_the_run_means() {
        let rows = summary_rows(&[10.0, 12.0, 14.0], 1, Trim::DEFAULT);
        assert_eq!(rows[0], ("mean".to_string(), "12.0".to_string()));
        assert_eq!(rows[1], ("stdev".to_string(), "2.0".to_string()));
        // t(0.975, 2) * 2 / sqrt(3) and t(0.975, 4) * 2 * sqrt(2/3).
        assert_eq!(rows[2], ("CI95 runs".to_string(), "5.0".to_string()));
        assert_eq!(rows[3], ("LSC runs".to_string(), "4.5".to_string()));
    }

    #[test]
    fn a_mean_prints_as_precisely_as_its_claims() {
        // The 7600x's min-now run means at --decimals 1: the claims extend to 3 decimals, and
        // the mean follows them rather than rounding to 16.4.
        let rows = summary_rows(&[16.355, 16.354, 16.353, 16.356, 16.354], 1, Trim::DEFAULT);
        assert_eq!(rows[0], ("mean".to_string(), "16.354".to_string()));
        assert_eq!(rows[3].1, "0.002");
    }

    #[test]
    fn a_long_series_adds_the_trimmed_pair_and_names_what_it_dropped() {
        // The 3900X's twenty clock-free run means, four of them disturbed.
        let means = [
            385.0, 382.7, 378.6, 387.0, 377.4, 382.7, 377.8, 385.5, 381.0, 377.6, 391.2, 381.5,
            393.2, 386.8, 384.8, 411.3, 385.2, 540.6, 772.0, 766.3,
        ];
        let rows = summary_rows(&means, 1, Trim::DEFAULT);
        let by = |label: &str| {
            rows.iter()
                .find(|(l, _)| l == label)
                .map(|(_, v)| v.clone())
                .expect(label)
        };
        // The plain pair follows the disturbed runs, the trimmed pair the low side of the rest.
        assert_eq!(by("mean"), "431.4");
        assert_eq!(by("CI95 runs"), "56.5");
        assert_eq!(by("trimmed mean"), "381.8");
        assert_eq!(by("winsorized stdev"), "2.8");
        assert_eq!(by("CI95 trimmed"), "3.8");
        assert_eq!(by("LSC trimmed"), "4.8");
        let t = Trimmed::of(&means, Trim::DEFAULT).expect("twenty trim");
        let line = aux_line(&rows, "trimmed", &trimmed_runs(&t, Trim::DEFAULT));
        assert!(
            line.contains("10-50 keeps 8 of 20 runs, drops low: 5, 10, high: "),
            "{line}"
        );
        assert!(line.ends_with("18, 19, 20"), "{line}");
    }

    #[test]
    fn a_short_series_has_no_trimmed_rows() {
        let rows = summary_rows(&[10.0, 12.0, 14.0], 1, Trim::DEFAULT);
        assert_eq!(rows.len(), 4);
        assert!(rows.iter().all(|(l, _)| !l.contains("trimmed")), "{rows:?}");
    }

    #[test]
    fn one_run_has_no_spread() {
        let rows = summary_rows(&[10.0], 1, Trim::DEFAULT);
        assert!(rows.iter().all(|(_, v)| v == "-"), "{rows:?}");
    }

    /// A run summary with the given mean, block stdev, and clock range.
    fn run(mean_ns: f64, stdev: f64, clock_ghz: Option<(f64, f64)>) -> RunSummary {
        RunSummary {
            bench: "min-now".to_string(),
            pid: 4242,
            samples: 1000,
            inner: (20, 20),
            inner_samples: Vec::new(),
            inner_sum_ns: Vec::new(),
            mean_ns,
            block_stdev_ns: Some(stdev),
            resolution_ns: None,
            clock_ghz,
            counters: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn a_run_line_lines_up_under_its_header() {
        let line = run_line(3, &run(24.64, 0.04, Some((4.35, 5.44))), 1, None);
        assert!(line.len() <= run_header(None).len(), "{line}");
        assert!(line.contains("24.64 ns"), "{line}");
        assert!(line.contains("0.04 ns"), "{line}");
        assert!(line.ends_with("4.35-5.44 GHz"), "{line}");
    }

    #[test]
    fn every_header_ends_where_its_cells_do() {
        let header = run_header(None);
        let line = run_line(1, &run(29.3, 0.7, Some((4.08, 4.08))), 1, None);
        for (label, cell) in [
            ("mean", "29.3 ns"),
            ("stdev blocks", "0.7 ns"),
            ("clock", "4.08 GHz"),
        ] {
            let head_end = header.find(label).expect(label) + label.len();
            let cell_end = line.find(cell).expect(cell) + cell.len();
            assert_eq!(head_end, cell_end, "{label} over {cell}\n{header}\n{line}");
        }
    }

    #[test]
    fn run_line_points_line_up_across_precisions() {
        let a = run_line(1, &run(27.9, 0.02, None), 1, None);
        let b = run_line(2, &run(25.9, 0.5, None), 1, None);
        assert_eq!(
            a.find("27.90").unwrap() + 2,
            b.find("25.9").unwrap() + 2,
            "{a}\n{b}"
        );
        assert_eq!(
            a.find("0.02").unwrap() + 1,
            b.find("0.5").unwrap() + 1,
            "{a}\n{b}"
        );
    }

    #[test]
    fn the_sizing_reads_as_one_value_or_its_range_and_a_span_as_written() {
        let sized = |inner: (u64, u64), samples: u64| RunSummary {
            inner,
            samples,
            ..run(10.0, 0.1, None)
        };
        assert_eq!(sizing_across(&[]), None);
        assert_eq!(
            sizing_across(&[sized((22, 22), 27_980), sized((22, 22), 27_980)]),
            Some(("22".to_string(), "27,980".to_string()))
        );
        assert_eq!(
            sizing_across(&[sized((22, 22), 27_930), sized((24, 24), 27_980)]),
            Some(("22 to 24".to_string(), "27,930 to 27,980".to_string()))
        );
        assert_eq!(
            sizing_across(&[sized((1, 16), 62_296)]).map(|(i, _)| i),
            Some("1-16".to_string())
        );
        // The run line carries both, under the header's columns.
        let line = run_line(1, &sized((22, 22), 27_980), 1, None);
        let header = run_header(None);
        assert_eq!(
            line.find("27,980").map(|i| i + 6),
            header.find("samples").map(|i| i + 7),
            "{header}\n{line}"
        );
        assert_eq!(
            run_line(1, &sized((1, 16), 62_296), 1, None).find("   1-16  "),
            header.find("inner").map(|i| i - 2),
            "{header}"
        );
    }

    #[test]
    fn a_span_titles_its_column_and_draws_how_each_run_leaves_its_line() {
        // Sample time 10 + 2 x length, every fourth length 1 ns slow: the repeat draws, and the
        // reach is the largest residual.
        let lengths = 1..=8u64;
        let counts = vec![100; 8];
        let sums: Vec<f64> = lengths
            .map(|l| 100.0 * (10.0 + 2.0 * l as f64 + if l % 4 == 0 { 1.0 } else { 0.0 }))
            .collect();
        let (graph, reach) = line_graph(1, &counts, &sums, 3);
        let cell = format!("{graph} {reach}");
        let bars: Vec<char> = graph.chars().collect();
        assert_eq!(bars.len(), 8, "{cell}");
        // The two slow lengths are the two highest bars, the fit tilting a little toward the
        // one at the end.
        let mut ranked: Vec<(char, usize)> = bars.iter().copied().zip(0..).collect();
        ranked.sort_by(|a, b| b.cmp(a));
        let mut top = [ranked[0].1, ranked[1].1];
        top.sort();
        assert_eq!(top, [3, 7], "{cell}");
        // The largest is the slow length 4, above the line.
        assert_eq!(reach.chars().next(), Some('+'), "{cell}");
        assert!(reach.ends_with(" ns at  4"), "{cell}");
        // A length that runs fast is a residual below the line, signed `-`.
        let mut fast = sums.clone();
        fast[5] -= 100.0 * 3.0;
        let (_, below) = line_graph(1, &counts, &fast, 3);
        assert!(
            below.starts_with('-') && below.ends_with(" ns at  6"),
            "{below}"
        );
        // On its line it draws level at no reach, an undrawn length is a space, and under two
        // lengths drawn there is no line.
        let straight: Vec<f64> = (1..=4).map(|l| 100.0 * (10.0 + 2.0 * l as f64)).collect();
        let (_, none_off) = line_graph(1, &[100; 4], &straight, 1);
        assert!(none_off.contains("<0.001 ns at "), "{none_off}");
        let (gap, _) = line_graph(1, &[100, 0, 100, 100], &[1200.0, 0.0, 1600.0, 1800.0], 1);
        assert_eq!(gap.chars().nth(1), Some(' '), "{gap}");
        let none = ("-".to_string(), "-".to_string());
        assert_eq!(line_graph(1, &[100, 0], &[1200.0, 0.0], 1), none);
        // Up to sixteen lengths a bar is a length, `1-16` the span the regression usually draws,
        // and past them the bars take two or more alike.
        assert_eq!(graph_width(16), 16);
        assert_eq!(graph_width(17), 9);
        assert_eq!(graph_width(120), 15);
        let span: InnerSpan = "1-8".parse().expect("a span");
        let header = run_header(Some(span));
        let line = run_line(
            1,
            &RunSummary {
                inner: (1, 8),
                inner_samples: counts,
                inner_sum_ns: sums,
                ..run(10.0, 0.1, None)
            },
            3,
            Some(span),
        );
        // Each title ends where its cells do, the graph under `inner 1-8` and the reach under
        // `residual`.
        let cols =
            |text: &str, at: &str| text[..text.find(at).expect(at) + at.len()].chars().count();
        assert_eq!(
            cols(&header, "inner 1-8"),
            cols(&line, &graph),
            "{header}\n{line}"
        );
        assert_eq!(
            cols(&header, "residual"),
            cols(&line, &reach),
            "{header}\n{line}"
        );
        assert!(!run_header(None).contains("residual"));
    }

    #[test]
    fn the_progress_line_counts_twenty_runs_a_line() {
        let line: String = (1..=22).map(progress_mark).collect();
        let lines: Vec<&str> = line.lines().collect();
        assert_eq!(lines.len(), 2, "{line}");
        assert!(lines[0].starts_with("run 1, 2, 3"), "{line}");
        assert!(lines[0].ends_with(", 19, 20"), "{line}");
        assert_eq!(lines[1], "run 21, 22");
    }

    #[test]
    fn counters_read_a_steady_count_as_a_number_and_a_moving_one_as_its_range() {
        let with = |pairs: &[(&str, u64)]| RunSummary {
            counters: pairs.iter().map(|&(k, v)| (k.to_string(), v)).collect(),
            ..run(10.0, 0.1, None)
        };
        assert_eq!(counters_across(&[run(10.0, 0.1, None)]), None);
        let runs = [
            with(&[("switches.consumer", 0), ("switches.producer", 0)]),
            with(&[("switches.consumer", 3), ("switches.producer", 0)]),
            with(&[("switches.producer", 0)]),
        ];
        assert_eq!(
            counters_across(&runs).as_deref(),
            Some("switches.consumer 0 to 3, switches.producer 0")
        );
    }

    #[test]
    fn a_held_clock_prints_one_number_and_a_moving_one_its_range() {
        assert_eq!(clock_cell(Some((4.70, 4.70))), "4.70 GHz");
        assert_eq!(clock_cell(Some((4.69, 4.71))), "4.70 GHz");
        assert_eq!(clock_cell(Some((4.62, 5.44))), "4.62-5.44 GHz");
        assert_eq!(clock_cell(None), "-");
    }

    #[test]
    fn the_summary_clock_spans_every_run() {
        let runs = [
            run(64.2, 0.1, Some((5.40, 5.44))),
            run(66.5, 0.1, Some((4.90, 5.44))),
            run(64.3, 0.1, None),
        ];
        assert_eq!(clock_across(&runs), Some((4.90, 5.44)));
        let rows = summary_rows(&[64.2, 66.5, 64.3], 1, Trim::DEFAULT);
        assert_eq!(
            aux_line(&rows, "clock", &clock_cell(clock_across(&runs))),
            "  clock      4.90-5.44 GHz"
        );
        assert_eq!(clock_across(&[run(1.0, 0.1, None)]), None);
    }
}
