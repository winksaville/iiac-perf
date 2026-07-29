//! Environment-qualification acceptance test: a thin wrapper
//! around the `qualify-environment` subcommand, which owns the
//! logic.
//!
//! The box under test is bistable (3900X, 2026-07-27: ~9% between
//! states, measured at 0.23.0-4 to be a 4.09 -> 4.49 GHz clock
//! climb), and grades degrade whenever a run's window straddles a
//! transition — not in either steady state. Respawning the binary
//! N times back to back provokes it: the loop's own load drives
//! the climb, whichever run straddles it lights up a transition
//! detector, and later runs ride the state the early ones forced
//! — involuntary warmup, the service the dynamic-warmup fix
//! (TODO.md "Dynamic startup warmup") makes deliberate.
//!
//! - **The logic moved to `iiac-perf qualify-environment`** at
//!   0.23.0-6, so the knobs are real flags with real `--help`
//!   (`--runs`, `--gap`, `-d`, `--pin`, `--print-only`) instead
//!   of env vars only this file understood, and the selftest is
//!   runnable by hand on any box. This test asserts the verdict;
//!   the subcommand decides it.
//! - **The observable is the environment grade**, not the run
//!   grade: this is a test of the box, and the environment
//!   stretches are the workload-independent ones. It migrated
//!   there from the `calibrate` environment letter, which
//!   0.23.0-7 deletes.
//! - `#[ignore]`d: machine-specific physics, meaningful only on a
//!   box that shows the two-state relaxation, run alone on a
//!   quiet system:
//!   `cargo test --release --test qualify_environment -- --ignored`
//! - **Use `--release`.** `cargo test` otherwise builds a debug
//!   binary, and each child then spends ~20 s in unoptimized
//!   calibration and warmup against ~2 s optimized — 200 s for
//!   the default ten runs. It is also the less representative
//!   measurement: the child's own phases are what provoke the
//!   box's state change, so they should run at the speed a real
//!   run does.
//! - `IIAC_PERF_BIN` tests another build (e.g. a saved failing
//!   one, or an installed release binary) instead of the freshly
//!   built one.

use std::process::Command;

/// Path of the binary under test: `IIAC_PERF_BIN` override or the
/// freshly built one.
fn bin() -> String {
    std::env::var("IIAC_PERF_BIN").unwrap_or_else(|_| env!("CARGO_BIN_EXE_iiac-perf").to_string())
}

#[test]
#[ignore = "machine-specific: run alone on a quiet box that shows the relaxation (see module doc)"]
fn environment_qualifies() {
    let out = Command::new(bin())
        .arg("qualify-environment")
        .output()
        .expect("spawn qualify-environment");
    // The subcommand's table is the diagnostic; surface it either
    // way so a failure explains itself under --nocapture.
    print!("{}", String::from_utf8_lossy(&out.stdout));
    eprint!("{}", String::from_utf8_lossy(&out.stderr));
    assert!(
        out.status.success(),
        "environment NOT QUALIFIED ({}): a state transition landed inside a \
         measurement window, or the median environment grade fell below B",
        out.status
    );
}
