//! Replication across processes: each bench runs `runs` times back to back, every run a fresh
//! child, and the bench's mean, stdev, CI95, and LSC come from the runs' means.
//!
//! Blocks inside one process share the placement the process drew, so their CI95 and LSC are
//! lower bounds on the spread a fresh process shows. A run is one process, and a series of run
//! means is the first replicate that re-rolls placement, so its error bars are the first that are
//! not lower bounds ([`crate::series`] owns the arithmetic).
//!
//! - A bench's runs go back to back, not interleaved with another bench's (wink, at the cycle's
//!   opening): a bench is compared by its own error bars, and a drift of the host between two
//!   benches' runs is a bias neither bench's CI95 contains.
//! - A run sleep, a time or a random range, goes before every run after the invocation's first.
//! - One run prints the child's report as a single process did. Several runs print a line per
//!   run as each child finishes, then the bench's summary, and `-v` shows every child's report
//!   as well.

use std::path::Path;

use crate::child::{self, RecordSpec, Spec};
use crate::dither::Dither;
use crate::harness::RunCfg;
use crate::record::{self, RunSummary};
use crate::report::{fmt_claim, fmt_commas_f64, print_summary_rows};
use crate::series::Series;

/// How an invocation's runs are spawned and shown.
pub struct Plan<'a> {
    /// The binary each child runs.
    pub exe: &'a Path,
    /// The invocation's scratch directory, for spec and result files.
    pub scratch: &'a Path,
    /// Runs per bench, one or more.
    pub runs: u64,
    /// The sleep before every run after the first, `(min_s, max_s)` seconds.
    pub run_sleep_s: (f64, f64),
    /// Pass `-v` to the children and show their reports beside the run lines.
    pub verbose: bool,
    /// Decimal digits on the time columns, the report's `--decimals`.
    pub decimals: usize,
}

/// Spawns the runs of an invocation's benches in order, carrying the run count across benches
/// so the sleep and the scratch file names span the whole invocation.
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
        let mut means = Vec::with_capacity(self.plan.runs as usize);
        for run in 1..=self.plan.runs {
            if self.spawned > 0 && self.plan.run_sleep_s.1 > 0.0 {
                let s = self.dither.span_s(self.plan.run_sleep_s);
                std::thread::sleep(std::time::Duration::from_secs_f64(s));
            }
            let n = self.spawned;
            self.spawned += 1;
            let result = self.plan.scratch.join(format!("run-{n}.jsonl"));
            let spec = Spec::new(bench, cfg, record, result.clone());
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
            means.extend(summaries.iter().map(|s| s.mean_ns));
        }
        if several {
            println!();
            print_summary_rows(&summary_rows(&means, self.plan.decimals));
            println!();
        }
        Ok(())
    }
}

/// The run table's header, over [`run_line`]'s columns.
fn run_header() -> String {
    format!(
        "{:>5}  {:>8}  {:>14}  {:>14}  {:>14}",
        "run", "pid", "mean", "CI95 blocks", "LSC blocks"
    )
}

/// One run's line: its index, the child's pid, the run's mean, and its within-process CI95 and
/// LSC, a withheld claim printing `-`.
fn run_line(run: u64, s: &RunSummary, decimals: usize) -> String {
    let claim = |v: Option<f64>| match v {
        Some(x) => format!("{} ns", fmt_claim(x, decimals.max(1))),
        None => "-".to_string(),
    };
    format!(
        "{run:>5}  {:>8}  {:>14}  {:>14}  {:>14}",
        s.pid,
        format!("{} ns", fmt_commas_f64(s.mean_ns, decimals)),
        claim(s.block_ci95_ns),
        claim(s.block_lsc_ns),
    )
}

/// A bench's summary rows over its run means: the plain mean, the run-to-run stdev, and the CI95
/// and LSC across runs, each `-` below two runs, where no spread exists.
fn summary_rows(means: &[f64], decimals: usize) -> Vec<(String, String)> {
    let dash = || "-".to_string();
    let (mean, stdev, ci95, lsc) = match Series::of(means) {
        Some(s) => (
            fmt_commas_f64(s.mean, decimals),
            fmt_commas_f64(s.stdev, decimals),
            fmt_claim(s.ci95(), decimals.max(1)),
            fmt_claim(s.lsc(), decimals.max(1)),
        ),
        None => (dash(), dash(), dash(), dash()),
    };
    vec![
        ("mean".to_string(), mean),
        ("stdev".to_string(), stdev),
        ("CI95 runs".to_string(), ci95),
        ("LSC runs".to_string(), lsc),
    ]
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
    fn one_run_has_no_spread() {
        let rows = summary_rows(&[10.0], 1);
        assert!(rows.iter().all(|(_, v)| v == "-"), "{rows:?}");
    }

    #[test]
    fn a_run_line_lines_up_under_its_header() {
        let s = RunSummary {
            bench: "min-now".to_string(),
            pid: 4242,
            mean_ns: 24.64,
            block_ci95_ns: Some(0.04),
            block_lsc_ns: None,
        };
        let line = run_line(3, &s, 1);
        assert_eq!(line.len(), run_header().len());
        assert!(line.contains("24.6 ns"), "{line}");
        assert!(line.contains("0.04 ns"), "{line}");
        assert!(line.trim_end().ends_with('-'), "{line}");
    }
}
