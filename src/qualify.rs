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
    /// `--pin` spec to pass through, if any.
    pub pin: Option<String>,
    /// Print the table and skip the verdict.
    pub print_only: bool,
}

/// One environment stretch parsed from a child's report: its
/// composite letter and the per-signal blob behind it.
struct Stretch {
    letter: char,
    signals: String,
}

impl Stretch {
    /// Letter of one named signal — each `, `-separated segment
    /// ends with its own letter.
    fn signal_letter(&self, name: &str) -> Option<char> {
        self.signals
            .split(", ")
            .find(|s| s.trim().starts_with(name))
            .and_then(|s| s.trim().chars().last())
    }

    /// True when a transition detector reached D/F here.
    fn transition_degraded(&self) -> bool {
        ["drift", "step"]
            .into_iter()
            .any(|s| self.signal_letter(s).is_some_and(|l| l == 'D' || l == 'F'))
    }
}

/// One child run's parsed result.
struct QualifyRun {
    warmup: Option<Stretch>,
    during: Option<Stretch>,
    /// Environment composite — the worse of the two stretches, as
    /// the child computed it.
    worst: char,
    /// The run's mean, for the value column: the number that makes
    /// a two-state box visible at a glance.
    mean: String,
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

/// Parse one `env <name>:` row into a [`Stretch`]: the composite
/// is carried separately, so this keeps the signal blob and the
/// row's own worst letter.
fn parse_stretch(line: &str) -> Stretch {
    // Every signal segment ends with its letter; the row's worst
    // is the worst of them, which is what the child's composite
    // is computed from.
    let letter = line
        .split(", ")
        .filter_map(|s| s.trim().chars().last())
        .min_by_key(|&c| score(c))
        .unwrap_or('?'); // OK: empty row can't happen, '?' scores worst if it does
    Stretch {
        letter,
        signals: line.to_string(),
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
    if let Some(pin) = &cfg.pin {
        cmd.arg("--pin").arg(pin);
    }
    let out = cmd.output().map_err(|e| format!("spawn child: {e}"))?;
    let text =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);

    let mut warmup = None;
    let mut during = None;
    let mut worst = '?';
    let mut mean = String::new();
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("env warmup:") {
            warmup = Some(parse_stretch(rest.trim()));
        } else if let Some(rest) = t.strip_prefix("env run:") {
            during = Some(parse_stretch(rest.trim()));
        } else if let Some(rest) = t.strip_prefix("env worst case:") {
            if let Some(c) = rest.trim().chars().next() {
                worst = c;
            }
        } else {
            // The plain `mean` row — not `mean z3..n2` (trimmed)
            // or `mean blocks`, whose second token isn't a number.
            let mut tok = t.split_whitespace();
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
    Ok(QualifyRun {
        warmup,
        during,
        worst,
        mean,
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
        match &cfg.pin {
            Some(p) => format!(", --pin {p}"),
            None => String::new(),
        }
    );
    println!("  the box is the subject: grades are the environment's, not the run's\n");
    println!("  run   warmup  env-run  worst   mean");

    let gap = Duration::from_secs_f64(cfg.gap_s.max(0.0));
    let mut runs: Vec<QualifyRun> = Vec::with_capacity(cfg.runs as usize);
    for i in 0..cfg.runs {
        std::thread::sleep(gap);
        match run_once(cfg) {
            Ok(r) => {
                let (w, d) = r.letters();
                println!("  {:<5} {w:<7} {d:<8} {:<7} {}", i + 1, r.worst, r.mean);
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

    println!("\n  median environment grade: {}", letter_for(median));
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
            println!("    median grade below B — runs are measuring a moving box");
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
    fn stretch_reads_its_worst_signal() {
        let s = parse_stretch("spread 0.30% A, interference 0.00% B, drift 0.00% A, step 0.00% A");
        assert_eq!(s.letter, 'B');
        assert_eq!(s.signal_letter("drift"), Some('A'));
        assert_eq!(s.signal_letter("interference"), Some('B'));
        assert!(!s.transition_degraded());
    }

    #[test]
    fn transition_detectors_are_drift_and_step_only() {
        // An F on interference is contamination, not a transition.
        let noisy =
            parse_stretch("spread 0.30% A, interference 40.0% F, drift 0.00% A, step 0.00% A");
        assert!(!noisy.transition_degraded());
        // A D on step is.
        let moved = parse_stretch(
            "spread 0.30% A, interference 0.00% A, drift 0.10% A, step 6.50% @1.097s D",
        );
        assert!(moved.transition_degraded());
    }

    #[test]
    fn unknown_letter_scores_worst() {
        assert_eq!(score('?'), score('F'));
        assert!(score('A') > score('B'));
    }
}
