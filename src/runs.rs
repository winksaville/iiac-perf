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
//!   as well.

use std::path::Path;

use crate::child::{self, RecordSpec, Spec};
use crate::dither::Dither;
use crate::harness::RunCfg;
use crate::record::{self, RunSummary};
use crate::report::{claim_precision, fmt_claim, fmt_commas_f64, print_summary_rows};
use crate::series::{Series, Trimmed};

/// The run sleep when neither `--run-sleep` nor the config sets one, `(min_s, max_s)` seconds: a
/// second or two before every run, drawn per run so the starts do not lock to anything periodic.
pub const DEFAULT_RUN_SLEEP_S: (f64, f64) = (1.0, 2.0);

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
    /// print its runs and summary when there are several.
    pub fn bench(&mut self, bench: &str, cfg: &RunCfg, record: &RecordSpec) -> Result<(), String> {
        let several = self.plan.runs > 1;
        let show_report = !several || self.plan.verbose;
        if several {
            println!("{bench}: {} runs, each in a fresh process", self.plan.runs);
            println!();
            if !show_report {
                println!("{}", run_header());
            }
        }
        let mut all: Vec<RunSummary> = Vec::with_capacity(self.plan.runs as usize);
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
            if several {
                if show_report {
                    println!("{}", run_header());
                }
                if summaries.is_empty() {
                    println!("{run:>5}  no record: a probe bench records nothing");
                }
                for s in &summaries {
                    println!("{}", run_line(run, s, self.plan.decimals));
                }
            }
            all.extend(summaries);
        }
        if several {
            let means: Vec<f64> = all.iter().map(|s| s.mean_ns).collect();
            let rows = summary_rows(&means, self.plan.decimals);
            println!();
            print_summary_rows(&rows);
            if let Some(t) = Trimmed::of(&means) {
                println!("{}", aux_line(&rows, "trimmed", &trimmed_runs(&t)));
            }
            println!(
                "{}",
                aux_line(&rows, "clock", &clock_cell(clock_across(&all)))
            );
            println!();
        }
        Ok(())
    }
}

/// The run table's header, over [`run_line`]'s columns.
fn run_header() -> String {
    format!(
        "{:>5}  {:>8}  {:>14}  {:>14}  {:>14}  {:>15}",
        "run", "pid", "mean", "stdev blocks", "resolution", "clock"
    )
}

/// One run's line: its index, the child's pid, the run's mean, the stdev of its block means, its
/// resolution, and its delivered clock. The stdev says how far the run's blocks wandered and the
/// resolution whether they drifted, so a run on another level reads as an off mean with a small
/// stdev and a run that moved as a resolution well above its neighbours'. The mean prints at
/// least as precisely as the two.
fn run_line(run: u64, s: &RunSummary, decimals: usize) -> String {
    let claim = |v: Option<f64>| match v {
        Some(x) => fmt_claim(x, decimals.max(1)),
        None => "-".to_string(),
    };
    let (stdev, resolution) = (claim(s.block_stdev_ns), claim(s.resolution_ns));
    let mean_decimals = claim_precision(decimals, &[&stdev, &resolution]);
    format!(
        "{run:>5}  {:>8}  {:>14}  {:>14}  {:>14}  {:>15}",
        s.pid,
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

/// A summary line that carries no ns value, its label padded to `rows`' labels: the clock range,
/// and which runs the trim dropped.
fn aux_line(rows: &[(String, String)], label: &str, value: &str) -> String {
    let width = rows.iter().map(|(l, _)| l.len()).fold(0, usize::max);
    format!("  {label:<width$}  {value}")
}

/// Which runs the trim dropped, as 1-based run numbers, for the line under the trimmed rows.
fn trimmed_runs(t: &Trimmed) -> String {
    let runs: Vec<String> = t.trimmed.iter().map(|i| (i + 1).to_string()).collect();
    format!("{} at each end: runs {}", t.per_end, runs.join(", "))
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
fn summary_rows(means: &[f64], decimals: usize) -> Vec<(String, String)> {
    let dash = || "-".to_string();
    let claim = |v: f64| fmt_claim(v, decimals.max(1));
    let plain = Series::of(means);
    let trimmed = Trimmed::of(means);
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
    fn summary_rows_come_from_the_run_means() {
        let rows = summary_rows(&[10.0, 12.0, 14.0], 1);
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
        let rows = summary_rows(&[16.355, 16.354, 16.353, 16.356, 16.354], 1);
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
        let rows = summary_rows(&means, 1);
        let by = |label: &str| {
            rows.iter()
                .find(|(l, _)| l == label)
                .map(|(_, v)| v.clone())
                .expect(label)
        };
        // The plain pair follows the disturbed runs, the trimmed pair the other sixteen.
        assert_eq!(by("mean"), "431.4");
        assert_eq!(by("CI95 runs"), "56.5");
        assert_eq!(by("trimmed mean"), "385.5");
        assert_eq!(by("winsorized stdev"), "4.9");
        assert_eq!(by("CI95 trimmed"), "4.0");
        assert_eq!(by("LSC trimmed"), "5.4");
        let t = Trimmed::of(&means).expect("twenty trim");
        let line = aux_line(&rows, "trimmed", &trimmed_runs(&t));
        assert!(line.contains("4 at each end: runs "), "{line}");
        assert!(line.ends_with("18, 19, 20"), "{line}");
    }

    #[test]
    fn a_short_series_has_no_trimmed_rows() {
        let rows = summary_rows(&[10.0, 12.0, 14.0], 1);
        assert_eq!(rows.len(), 4);
        assert!(rows.iter().all(|(l, _)| !l.contains("trimmed")), "{rows:?}");
    }

    #[test]
    fn one_run_has_no_spread() {
        let rows = summary_rows(&[10.0], 1);
        assert!(rows.iter().all(|(_, v)| v == "-"), "{rows:?}");
    }

    /// A run summary with the given mean, block stdev, and clock range.
    fn run(mean_ns: f64, stdev: f64, clock_ghz: Option<(f64, f64)>) -> RunSummary {
        RunSummary {
            bench: "min-now".to_string(),
            pid: 4242,
            mean_ns,
            block_stdev_ns: Some(stdev),
            resolution_ns: None,
            clock_ghz,
        }
    }

    #[test]
    fn a_run_line_lines_up_under_its_header() {
        let line = run_line(3, &run(24.64, 0.04, Some((4.35, 5.44))), 1);
        assert!(line.len() <= run_header().len(), "{line}");
        assert!(line.contains("24.64 ns"), "{line}");
        assert!(line.contains("0.04 ns"), "{line}");
        assert!(line.ends_with("4.35-5.44 GHz"), "{line}");
    }

    #[test]
    fn run_line_points_line_up_across_precisions() {
        let a = run_line(1, &run(27.9, 0.02, None), 1);
        let b = run_line(2, &run(25.9, 0.5, None), 1);
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
        let rows = summary_rows(&[64.2, 66.5, 64.3], 1);
        assert_eq!(
            aux_line(&rows, "clock", &clock_cell(clock_across(&runs))),
            "  clock      4.90-5.44 GHz"
        );
        assert_eq!(clock_across(&[run(1.0, 0.1, None)]), None);
    }
}
