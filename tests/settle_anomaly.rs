//! Settle-anomaly acceptance test (3900X, 2026-07-27): the box
//! is bistable (~0.445 vs ~0.489 ns/iter, ~9%), and grades
//! degrade whenever a run's window straddles a state
//! transition — not in either steady state. One loop of N
//! back-to-back runs reproduces it: the loop's own load
//! provokes the transition, whichever run straddles it lights
//! up a transition detector (drift within a window, repeat
//! between attempts), and the later runs ride the state the
//! early runs forced — involuntary warmup, the very service
//! the dynamic-warmup fix (TODO.md "Dynamic startup warmup")
//! makes deliberate. Accepted when no run grades D/F on
//! drift/repeat and the median reaches B. The assertion is
//! cause-aware on purpose: disturbed/dirty C's are ambient
//! contamination (e.g. a concurrent build) and resid is a
//! machine trait — neither is the anomaly, and the warmup fix
//! removes neither. Grades, not values, are the criterion.
//!
//! - `#[ignore]`d: machine-specific physics — meaningful only on
//!   a box that exhibits the two-state relaxation (the 3900X;
//!   the 7600x is single-state and passes vacuously, guarding
//!   the warm-exit path post-fix), run alone on a quiet system:
//!   `cargo test --test settle_anomaly -- --ignored`
//! - Spawns the real binary per run (`CARGO_BIN_EXE_iiac-perf`) —
//!   a fresh process per run, like terminal use; `main` is not
//!   callable from an integration test and in-process repeats
//!   would share warmed state anyway.
//! - Env knobs: `IIAC_PERF_BIN` (test another build, e.g. a saved
//!   failing 0.23.0-1 once `calibrate` is gone from the tree),
//!   `SETTLE_N` (runs, default 10), `SETTLE_GAP` (seconds
//!   between runs, default 0 — zero-gap sustains the duty that
//!   provokes the transition; rerun with a nonzero gap to probe
//!   another duty cycle), `SETTLE_PRINT_ONLY=1` (table only,
//!   skip the assertion — for smoke-testing the harness itself).
//! - The observable is the calibrate environment letter; when
//!   the batch gauge replaces it (0.23.0-4), the parse migrates
//!   to the gauge's grade.

use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

/// One parsed calibrate run: environment letter, loop/iter ns,
/// and the letter's signal breakdown (the parenthesized
/// disturbed / dirty win / drift / repeat blob) so a stray C/D
/// in the table explains itself.
struct CalRun {
    letter: char,
    loop_iter: String,
    signals: String,
}

impl CalRun {
    /// Letter of one named signal from the parenthesized blob
    /// (each `, `-separated segment ends with its letter).
    fn signal_letter(&self, name: &str) -> Option<char> {
        self.signals
            .trim_start_matches('(')
            .trim_end_matches(')')
            .split(", ")
            .find(|s| s.starts_with(name))
            .and_then(|s| s.chars().last())
    }

    /// True when this run shows the anomaly's own signature: a
    /// state transition graded D/F on either transition
    /// detector — drift (within a window) or repeat (between
    /// attempts). C-level wobble and the other signals
    /// (disturbed / dirty / resid: contamination and machine
    /// traits the warmup fix cannot remove) don't count.
    fn transition_degraded(&self) -> bool {
        ["drift", "repeat"]
            .into_iter()
            .any(|s| self.signal_letter(s).is_some_and(|l| l == 'D' || l == 'F'))
    }
}

/// Path of the binary under test: `IIAC_PERF_BIN` override or
/// the freshly built one.
fn bin() -> String {
    std::env::var("IIAC_PERF_BIN").unwrap_or_else(|_| env!("CARGO_BIN_EXE_iiac-perf").to_string())
}

/// Env-var integer knob with default.
fn knob(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Run `<bin> calibrate` once and parse the environment letter
/// and loop/iter value from its report.
fn calibrate_once() -> CalRun {
    let out = Command::new(bin())
        .arg("calibrate")
        .output()
        .expect("spawn calibrate");
    let text =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
    let env_line = text
        .lines()
        .find_map(|l| l.strip_prefix("  environment"))
        .expect("environment line in calibrate output");
    let letter = env_line
        .trim_start()
        .chars()
        .next()
        .expect("environment letter");
    let signals = env_line
        .find('(')
        .map(|i| env_line[i..].to_string())
        .unwrap_or_default();
    let loop_iter = text
        .lines()
        .find_map(|l| {
            let rest = l.strip_prefix("  loop/iter")?;
            Some(rest.split_whitespace().next()?.to_string())
        })
        .expect("loop/iter in calibrate output");
    CalRun {
        letter,
        loop_iter,
        signals,
    }
}

/// Letter -> ordinal score for the cadence comparison (A best).
fn score(letter: char) -> i64 {
    match letter {
        'A' => 5,
        'B' => 4,
        'C' => 3,
        'D' => 2,
        _ => 1,
    }
}

/// Median score of a cadence's runs.
fn median_score(runs: &[CalRun]) -> i64 {
    let mut scores: Vec<i64> = runs.iter().map(|r| score(r.letter)).collect();
    scores.sort_unstable();
    scores[scores.len() / 2]
}

/// Run one cadence: `n` calibrates each preceded by `gap` sleep.
fn cadence(tag: &str, n: u64, gap: Duration) -> Vec<CalRun> {
    (0..n)
        .map(|i| {
            sleep(gap);
            let run = calibrate_once();
            println!(
                "{tag}-{idx}  {letter}  loop/iter {li}  {sig}",
                idx = i + 1,
                letter = run.letter,
                li = run.loop_iter,
                sig = run.signals
            );
            run
        })
        .collect()
}

#[test]
#[ignore = "machine-specific: run alone on a quiet box that shows the relaxation (see module doc)"]
fn settle_anomaly_runs_steady() {
    let n = knob("SETTLE_N", 10);
    let gap = Duration::from_secs(knob("SETTLE_GAP", 0));
    println!("bin: {}", bin());
    println!(
        "knobs (env): SETTLE_N={n} SETTLE_GAP={}s SETTLE_PRINT_ONLY={} IIAC_PERF_BIN={}",
        gap.as_secs(),
        std::env::var("SETTLE_PRINT_ONLY").is_ok(),
        if std::env::var("IIAC_PERF_BIN").is_ok() {
            "set"
        } else {
            "unset (CARGO_BIN_EXE)"
        },
    );
    println!("expect a drift/repeat D/F on the transition run while unfixed:");

    let runs = cadence("run", n, gap);

    let median = median_score(&runs);
    let transitions = runs.iter().filter(|r| r.transition_degraded()).count();
    println!("median score: {median}; transition-degraded (drift/repeat at D/F): {transitions}");
    if std::env::var("SETTLE_PRINT_ONLY").is_ok() {
        return;
    }
    assert!(
        median >= 4,
        "settle anomaly: median grade {median} below B (4) \
         — runs are measuring mid-transition machine states"
    );
    assert!(
        transitions == 0,
        "settle anomaly: {transitions} run(s) with drift or repeat at D/F \
         — a state transition landed inside or between measurement windows"
    );
}
