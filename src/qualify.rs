//! The `qualify-environment` selftest: is this machine fit to
//! measure on?
//!
//! Respawns our own binary `--runs` times at `--gap`, collects
//! each run's environment grade, prints the table, and returns a
//! verdict. It tests the *machine*, not a workload, so it reads
//! the environment stretches rather than the run grade — see
//! [`crate::gauge::EnvGrade`].
//!
//! - **Why respawn** rather than loop in-process: a fresh process
//!   per run is what terminal use looks like, and in-process
//!   repeats would share warmed state — the very thing under
//!   test.
//! - **Why `min-now`** as the child workload: the box is the
//!   subject, so the leanest available bench is right. It also
//!   measures nearly what the probe measures, which keeps the run
//!   grade and the environment grade commensurable.
//! - **The verdict** is grades, not values: median environment
//!   grade at B or better, and no run whose `drift` or `step`
//!   reached D/F in either stretch. Those two are the transition
//!   detectors, so a D/F there is a state change landing inside a
//!   measurement window — the anomaly this test exists to catch.
//!   Wobble on `spread` or `interference` is ambient
//!   contamination and does not fail the run.

use std::process::Command;
use std::time::Duration;

use crate::gauge::Settle;

/// Child bench: the leanest registered workload, so the table
/// reflects the box rather than a workload's character.
const CHILD_BENCH: &str = "min-now";

/// Knobs for one selftest, from the CLI.
pub struct QualifyCfg {
    /// Child runs to spawn.
    pub runs: u64,
    /// Sleep before each child. Zero sustains the duty cycle that
    /// provokes a transition; a nonzero gap probes a quieter one.
    pub gap_s: f64,
    /// Wall-clock seconds per child run.
    pub duration_s: f64,
    /// `--pin-cpus` spec to pass through, if any.
    pub pin_cpus: Option<String>,
    /// Print the table and skip the verdict.
    pub print_only: bool,
    /// `--settle-time` to pass each child, when the parent was
    /// given one. Absent leaves each child on its own default, so
    /// the table reflects what a plain run of this binary does.
    pub settle_time: Option<f64>,
}

/// One environment stretch parsed from a child's grade-block
/// row: its composite (`worst` column) and the two transition
/// detectors' letters.
struct Stretch {
    letter: char,
    drift: Option<char>,
    step: Option<char>,
}

/// Split one report line into grade-block cells: columns are
/// right-aligned with a two-space minimum gap, so two-or-more
/// spaces is the cell separator and single spaces stay inside a
/// cell (`0.30% A`, `4.84->5.24GHz 49% +-0.1%`, `9.37% @1.90s D`).
fn row_cells(line: &str) -> Vec<&str> {
    line.split("  ")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

/// A cell's trailing grade letter, `None` for a blank cell or
/// anything else that does not end in A-F.
fn cell_letter(cell: &str) -> Option<char> {
    cell.chars().last().filter(|c| ('A'..='F').contains(c))
}

/// A warmup row's `settle` cell: the journey forms the harness
/// prints (`4.84->5.24GHz 49% +-0.0%`, `3.60->4.20GHz 0%`, and
/// the timing-only `49%` / `0%`), or `None` for a blank or an
/// `uncertified`. Format and parser move together: the child
/// is this same binary. `0%` is reserved for never-settled,
/// and the ramp-end `t_s` never reaches the cell, so a parsed
/// `At` carries `t_s` as zero.
fn parse_settle(cell: &str) -> Option<Settle> {
    // The report appends the settle signal's letter after the cell. The values are what
    // qualify reads (the letter already folded into the row's worst), so strip it.
    let cell = match cell.rsplit_once(' ') {
        Some((head, l)) if l.len() == 1 && l.chars().all(|c| c.is_ascii_uppercase()) => head,
        _ => cell,
    };
    let (cell, rating) = match cell.rsplit_once(" +-") {
        Some((head, r)) => (
            head,
            Some(r.strip_suffix('%')?.parse::<f64>().ok()? / 100.0),
        ),
        None => (cell, None),
    };
    let (head, pct) = match cell.rsplit_once(' ') {
        Some((h, t)) => (h, t),
        None => ("", cell),
    };
    let pct: f64 = pct.strip_suffix('%')?.parse().ok()?;
    let (start_ghz, ghz) = parse_journey(head)?;
    if pct == 0.0 {
        return Some(Settle::Never {
            start_ghz,
            end_ghz: ghz,
        });
    }
    Some(Settle::At {
        t_s: 0.0,
        settled_frac: pct / 100.0,
        start_ghz,
        ghz,
        rating,
    })
}

/// A cell's journey prefix: `3.60->4.09GHz` -> both ends,
/// `4.09GHz` -> one clock at both ends, empty -> no clock.
/// `None` for anything else.
fn parse_journey(s: &str) -> Option<(Option<f64>, Option<f64>)> {
    if s.is_empty() {
        return Some((None, None));
    }
    let s = s.strip_suffix("GHz")?;
    match s.split_once("->") {
        Some((a, b)) => Some((Some(a.parse().ok()?), Some(b.parse().ok()?))),
        None => {
            let g: f64 = s.parse().ok()?;
            Some((Some(g), Some(g)))
        }
    }
}

/// Parse one grade-block row's cells into a [`Stretch`]. Cell
/// order is the block's column order: grade, phase, settle,
/// worst, spread, bursts, interference, drift, step.
fn parse_stretch(cells: &[&str]) -> Stretch {
    Stretch {
        letter: cells.get(3).and_then(|c| c.chars().next()).unwrap_or('?'), // OK: short row can't happen, '?' scores worst if it does
        drift: cells.get(7).copied().and_then(cell_letter),
        step: cells.get(8).copied().and_then(cell_letter),
    }
}

impl Stretch {
    /// True when a transition detector reached D/F here.
    fn transition_degraded(&self) -> bool {
        [self.drift, self.step]
            .into_iter()
            .flatten()
            .any(|l| l == 'D' || l == 'F')
    }
}

/// One child run's parsed result.
struct QualifyRun {
    warmup: Option<Stretch>,
    during: Option<Stretch>,
    /// Environment composite — the worse of the two stretches,
    /// computed here from their `worst` columns.
    worst: char,
    /// The run's mean, for the value column: the number that makes
    /// a two-state box visible at a glance.
    mean: String,
    /// The child warmup's settle cell: the clock's journey and
    /// how long the settled state held, read across respawns.
    /// Reported, not judged: the verdict stays grades, because a
    /// box still moving at the end of warmup already shows up as
    /// a drift/step D/F on the warmup stretch.
    settle: Option<Settle>,
}

impl QualifyRun {
    /// True when either stretch shows a transition at D/F.
    fn transition_degraded(&self) -> bool {
        [self.warmup.as_ref(), self.during.as_ref()]
            .into_iter()
            .flatten()
            .any(Stretch::transition_degraded)
    }

    /// Per-stretch letters for the table, `-` where a stretch is
    /// absent (`--no-env-probe` leaves no run stretch).
    fn letters(&self) -> (char, char) {
        (
            self.warmup.as_ref().map_or('-', |s| s.letter),
            self.during.as_ref().map_or('-', |s| s.letter),
        )
    }
}

/// One-line tally of the runs' composite grades, best first:
/// `7A 1B 1D 1F`, absent letters omitted.
///
/// - Printed beside the median because a median is a statement
///   about the typical run, and qualification is a question about
///   the bad one: a box clean on seven runs of ten reports a
///   median of A while three runs saw the machine move. The tally
///   cannot hide a tail.
fn grade_tally(runs: &[QualifyRun]) -> String {
    ['A', 'B', 'C', 'D', 'F']
        .into_iter()
        .filter_map(|l| match runs.iter().filter(|r| r.worst == l).count() {
            0 => None,
            n => Some(format!("{n}{l}")),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Letter -> ordinal score, A best. Unknown letters score worst,
/// so a parse miss can never flatter a run.
fn score(letter: char) -> i64 {
    match letter {
        'A' => 5,
        'B' => 4,
        'C' => 3,
        'D' => 2,
        _ => 1,
    }
}

/// Inverse of [`score`], for reporting a median back as a letter.
fn letter_for(score: i64) -> char {
    match score {
        5 => 'A',
        4 => 'B',
        3 => 'C',
        2 => 'D',
        _ => 'F',
    }
}

/// Spawn one child and parse its report.
fn run_once(cfg: &QualifyCfg) -> Result<QualifyRun, String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let mut cmd = Command::new(exe);
    cmd.arg(CHILD_BENCH)
        .arg("-d")
        .arg(cfg.duration_s.to_string())
        // The parent already holds the sleep lock; a child
        // re-exec per run would cost more than the run.
        .arg("--no-inhibit");
    if let Some(pin) = &cfg.pin_cpus {
        cmd.arg("--pin-cpus").arg(pin);
    }
    if let Some(t) = cfg.settle_time {
        cmd.arg("--settle-time").arg(t.to_string());
    }
    let out = cmd.output().map_err(|e| format!("spawn child: {e}"))?;
    let text =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);

    let mut warmup = None;
    let mut during = None;
    let mut mean = String::new();
    let mut settle = None;
    for line in text.lines() {
        let cells = row_cells(line);
        if cells.first() == Some(&"env") {
            match cells.get(1) {
                Some(&"warmup") => {
                    settle = cells.get(2).and_then(|c| parse_settle(c));
                    warmup = Some(parse_stretch(&cells));
                }
                Some(&"bench") => during = Some(parse_stretch(&cells)),
                _ => {}
            }
        } else {
            // The plain `mean` row — not `mean z3..n2` (trimmed)
            // or `mean blocks`, whose second token isn't a number.
            let mut tok = line.split_whitespace();
            if tok.next() == Some("mean")
                && let Some(v) = tok.next()
                && v.parse::<f64>().is_ok()
                && mean.is_empty()
            {
                mean = format!("{v} ns");
            }
        }
    }
    if warmup.is_none() && during.is_none() {
        return Err(format!(
            "no environment grade in child output ({} bytes); \
             child exited {}",
            text.len(),
            out.status
        ));
    }
    // The environment composite: the worse of the two stretch
    // letters, computed here now that the block prints no
    // composite line of its own (each row's `worst` is visible
    // beside its causes instead).
    let worst = [warmup.as_ref(), during.as_ref()]
        .into_iter()
        .flatten()
        .map(|s| s.letter)
        .min_by_key(|&c| score(c))
        .unwrap_or('?'); // OK: unreachable, the is_none guard above returned already
    Ok(QualifyRun {
        warmup,
        during,
        worst,
        mean,
        settle,
    })
}

/// Run the selftest and return the process exit code: 0 when the
/// machine qualifies, 1 when it does not, 2 on a spawn/parse
/// failure.
pub fn run(cfg: &QualifyCfg) -> i32 {
    println!(
        "qualify-environment: {} runs of `{CHILD_BENCH} -d {}`, gap {}s{}",
        cfg.runs,
        cfg.duration_s,
        cfg.gap_s,
        match &cfg.pin_cpus {
            Some(p) => format!(", --pin-cpus {p}"),
            None => String::new(),
        }
    );
    println!("  the box is the subject: grades are the environment's, not the run's\n");
    println!("  run   warmup  bench    worst   settle                      mean");

    let gap = Duration::from_secs_f64(cfg.gap_s.max(0.0));
    let mut runs: Vec<QualifyRun> = Vec::with_capacity(cfg.runs as usize);
    for i in 0..cfg.runs {
        std::thread::sleep(gap);
        match run_once(cfg) {
            Ok(r) => {
                let (w, d) = r.letters();
                // The state rides the settle column here too: which clock each
                // warmup certified is the fastest read on a two-state box, where
                // the settle times alone all look instant.
                let settle = match &r.settle {
                    Some(s) => s.to_string(),
                    None => "-".to_string(),
                };
                println!(
                    "  {:<5} {w:<7} {d:<8} {:<7} {settle:<27} {}",
                    i + 1,
                    r.worst,
                    r.mean
                );
                runs.push(r);
            }
            Err(e) => {
                eprintln!("error: qualify run {}: {e}", i + 1);
                return 2;
            }
        }
    }

    let mut scores: Vec<i64> = runs.iter().map(|r| score(r.worst)).collect();
    scores.sort_unstable();
    let median = scores.get(scores.len() / 2).copied().unwrap_or(0); // OK: runs >= 1 enforced by the CLI
    let degraded = runs.iter().filter(|r| r.transition_degraded()).count();

    // Never-settled runs enter as their reserved 0%, so the median is over every run that
    // reported a cell rather than only the ones that settled.
    let mut settles: Vec<f64> = runs
        .iter()
        .filter_map(|r| match r.settle {
            Some(Settle::At { settled_frac, .. }) => Some(settled_frac),
            Some(Settle::Never { .. }) => Some(0.0),
            None => None,
        })
        .collect();
    settles.sort_by(f64::total_cmp);
    let never = runs
        .iter()
        .filter(|r| matches!(r.settle, Some(Settle::Never { .. })))
        .count();

    println!("\n  environment grades: {}", grade_tally(&runs));
    println!("  median environment grade: {}", letter_for(median));
    if let Some(&s) = settles.get(settles.len() / 2) {
        println!(
            "  median settled: {:.0}% of warmup ({never} of {} never settled)",
            s * 100.0,
            runs.len()
        );
    }
    println!(
        "  transition-degraded (drift or step at D/F): {degraded} of {}",
        runs.len()
    );

    if cfg.print_only {
        println!("\n  verdict: skipped (--print-only)");
        return 0;
    }
    let pass = median >= 4 && degraded == 0;
    println!(
        "\n  verdict: {}",
        if pass { "QUALIFIED" } else { "NOT QUALIFIED" }
    );
    if !pass {
        if median < 4 {
            println!("    median grade below B: runs are measuring a moving box");
        }
        if degraded > 0 {
            println!("    a state transition landed inside a measurement window");
        }
        return 1;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stretch_reads_the_worst_column() {
        let cells = row_cells(
            "  env    warmup        0.86s      B    0.30% A       -       0.01% B   0.00% A            0.00% A",
        );
        let s = parse_stretch(&cells);
        assert_eq!(s.letter, 'B');
        assert_eq!(s.drift, Some('A'));
        assert_eq!(s.step, Some('A'));
        assert!(!s.transition_degraded());
    }

    #[test]
    fn transition_detectors_are_drift_and_step_only() {
        // An F on interference is contamination, not a transition.
        let noisy = parse_stretch(&row_cells(
            "  env    bench             -      F    0.30% A       -      40.00% F   0.00% A            0.00% A",
        ));
        assert!(!noisy.transition_degraded());
        // A D on step is.
        let moved = parse_stretch(&row_cells(
            "  env    bench             -      D    0.30% A       -       0.01% A   0.10% A     6.50% @1.10s D",
        ));
        assert!(moved.transition_degraded());
    }

    #[test]
    fn settle_cell_is_read_but_not_scored() {
        // The timing-only forms, 0% reserved for never-settled.
        assert_eq!(
            parse_settle("84%"),
            Some(Settle::At {
                t_s: 0.0,
                settled_frac: 0.84,
                start_ghz: None,
                ghz: None,
                rating: None
            })
        );
        assert_eq!(
            parse_settle("00%"),
            Some(Settle::Never {
                start_ghz: None,
                end_ghz: None
            })
        );
        // The journey forms: single spaces stay inside the cell.
        assert_eq!(
            parse_settle("4.84->5.24GHz 49% +-0.1%"),
            Some(Settle::At {
                t_s: 0.0,
                settled_frac: 0.49,
                start_ghz: Some(4.84),
                ghz: Some(5.24),
                rating: Some(0.1 / 100.0)
            })
        );
        assert_eq!(
            parse_settle("4.09->4.09GHz 100% +-0.1%"),
            Some(Settle::At {
                t_s: 0.0,
                settled_frac: 1.0,
                start_ghz: Some(4.09),
                ghz: Some(4.09),
                rating: Some(0.1 / 100.0)
            })
        );
        assert_eq!(
            parse_settle("3.60->4.20GHz 00%"),
            Some(Settle::Never {
                start_ghz: Some(3.6),
                end_ghz: Some(4.2)
            })
        );
        // The report appends the settle signal's letter; the parse strips it.
        assert_eq!(
            parse_settle("4.84->5.24GHz 49% +-0.1% D"),
            parse_settle("4.84->5.24GHz 49% +-0.1%")
        );
        // The bench and run rows carry a blank settle cell.
        assert_eq!(parse_settle("-"), None);
        // A never-settled warmup still scores from its own worst
        // column, never from the settle cell.
        let never = parse_stretch(&row_cells(
            "  env    warmup  3.65->4.53GHz 00% F      F    0.30% A       -       0.01% A   9.90% F            0.00% A",
        ));
        assert_eq!(never.letter, 'F');
        assert_eq!(never.drift, Some('F'));
    }

    #[test]
    fn header_and_run_rows_are_not_stretches() {
        // Neither the header nor the `run all` row starts with
        // `env`, so run_once's row filter passes them by; this
        // pins the cell shapes it filters on.
        let header = row_cells(
            "  grade  phase        settle  worst     spread  bursts  interference     drift               step",
        );
        assert_eq!(header.first(), Some(&"grade"));
        let run = row_cells(
            "  run    all               -      D          -   33% B       2.59% B   2.93% C     9.37% @1.90s D",
        );
        assert_eq!(run.first(), Some(&"run"));
        assert_eq!(run.len(), 9);
    }

    #[test]
    fn grade_tally_counts_every_letter_seen() {
        let run = |worst| QualifyRun {
            warmup: None,
            during: None,
            worst,
            mean: String::new(),
            settle: None,
        };
        let runs: Vec<QualifyRun> = "AAAAAAABDF".chars().map(run).collect();
        // The shape that motivated the line: a median of A over a
        // table showing three runs where the box moved.
        assert_eq!(grade_tally(&runs), "7A 1B 1D 1F");
        assert_eq!(grade_tally(&[]), "");
    }

    #[test]
    fn unknown_letter_scores_worst() {
        assert_eq!(score('?'), score('F'));
        assert!(score('A') > score('B'));
    }
}
