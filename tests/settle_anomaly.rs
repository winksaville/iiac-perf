//! Settle-anomaly acceptance test (3900X, 2026-07-27): the box
//! is bistable (~0.445 vs ~0.489 ns/iter, ~9%), and grades
//! degrade whenever a run's window straddles a transition —
//! not in either steady state. Three regimes observed:
//! continuous hammering locks the fast state and grades B
//! (after one F on the transition run); fully settled sits at
//! 0.489 and grades B; intermediate cadences (~1 s gaps, or
//! 8 s gaps on a previously-hot box) put a transition inside
//! every window and grade D/F. The dynamic-warmup fix (TODO.md
//! "Dynamic startup warmup") is accepted when every run
//! measures a steady state no matter when it launches: both
//! cadence medians reach B *and* no run grades D/F on a
//! transition detector (drift within a window, repeat between
//! attempts). The assertion is cause-aware on purpose:
//! disturbed/dirty C's are ambient contamination (e.g. a
//! concurrent build) and resid is a machine trait — neither is
//! the anomaly, and the warmup fix removes neither. Cadences
//! may still report different speeds (bistable machine,
//! honestly measured); grades, not values, are the criterion.
//!
//! - `#[ignore]`d: machine-specific physics — meaningful only on
//!   a box that exhibits the two-state relaxation (the 3900X),
//!   run alone on a quiet system:
//!   `cargo test --test settle_anomaly -- --ignored`
//! - Spawns the real binary per run (`CARGO_BIN_EXE_iiac-perf`) —
//!   a fresh process per run, like terminal use; `main` is not
//!   callable from an integration test and in-process repeats
//!   would share warmed state anyway.
//! - Env knobs: `IIAC_PERF_BIN` (test another build, e.g. a saved
//!   failing 0.23.0-1 once `calibrate` is gone from the tree),
//!   `SETTLE_N` (runs per cadence, default 5),
//!   `SETTLE_QUICK_GAP` / `SETTLE_WAIT_GAP` (seconds, defaults
//!   0 / 8 — zero-gap is what triggers the boost climb; a 2 s
//!   gap is ~18% duty and never shifts the core up),
//!   `SETTLE_PRINT_ONLY=1` (table only, skip the
//!   assertion — for smoke-testing the harness itself).
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
fn settle_anomaly_cadences_agree() {
    let n = knob("SETTLE_N", 5);
    let quick_gap = Duration::from_secs(knob("SETTLE_QUICK_GAP", 0));
    let wait_gap = Duration::from_secs(knob("SETTLE_WAIT_GAP", 8));
    println!("bin: {}", bin());
    println!("N={n} quick_gap={quick_gap:?} wait_gap={wait_gap:?}");

    println!("quick cadence (expect D/F while unfixed):");
    let quick = cadence("quick", n, quick_gap);
    println!("waited cadence (expect A/B):");
    let waited = cadence("wait", n, wait_gap);

    let (mq, mw) = (median_score(&quick), median_score(&waited));
    let transitions = |runs: &[CalRun]| runs.iter().filter(|r| r.transition_degraded()).count();
    let (tq, tw) = (transitions(&quick), transitions(&waited));
    println!(
        "median score: quick {mq} vs waited {mw}; \
         transition-degraded (drift/repeat at D/F): quick {tq} / waited {tw}"
    );
    if std::env::var("SETTLE_PRINT_ONLY").is_ok() {
        return;
    }
    assert!(
        mq >= 4 && mw >= 4,
        "settle anomaly: cadence medians quick {mq} / waited {mw} below B (4) \
         — runs are measuring mid-transition machine states"
    );
    assert!(
        tq == 0 && tw == 0,
        "settle anomaly: {tq} quick / {tw} waited runs with drift or repeat at D/F \
         — a state transition landed inside or between measurement windows"
    );
}
