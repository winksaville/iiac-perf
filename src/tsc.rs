//! TSC availability gate shared by [`crate::probe::Probe`] and
//! [`crate::tprobe::TProbe`].
//!
//! We require two properties:
//!
//! 1. **`constant_tsc`** — the TSC runs at a fixed frequency
//!    regardless of CPU P-state changes. Without this, tick
//!    deltas are not comparable across frequency transitions.
//! 2. **`minstant::is_tsc_available()`** — the Linux kernel has
//!    accepted TSC as an available clocksource, which also
//!    implies `nonstop_tsc` and monotonicity across cores.
//!
//! The gate lives outside both probe modules so a single error
//! message and exit path covers every probe construction site.

use std::sync::OnceLock;

/// `true` iff `/proc/cpuinfo` advertises `constant_tsc` on this
/// CPU. Cached; the first call reads the file, subsequent calls
/// return the cached answer.
pub fn has_constant_tsc() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| {
        let cpuinfo = match std::fs::read_to_string("/proc/cpuinfo") {
            Ok(s) => s,
            Err(_) => return false,
        };
        cpuinfo
            .lines()
            .filter(|l| l.starts_with("flags"))
            .any(|l| l.split_whitespace().any(|t| t == "constant_tsc"))
    })
}

/// Verify TSC is usable for probes: kernel-accepted clocksource
/// (`minstant::is_tsc_available`) and fixed-frequency
/// (`constant_tsc`). On failure, prints a diagnostic and exits
/// the process with code 1 — probes are meaningless without both
/// properties, so there's no graceful recovery.
pub fn require_tsc_ok() {
    if !minstant::is_tsc_available() {
        eprintln!(
            "error: TSC not available as a kernel clocksource. \
             iiac-perf probes require a stable, kernel-accepted TSC; \
             refusing to run."
        );
        std::process::exit(1);
    }
    if !has_constant_tsc() {
        eprintln!(
            "error: `constant_tsc` flag missing from /proc/cpuinfo. \
             iiac-perf probes require a fixed-frequency TSC so tick \
             deltas are comparable across P-state transitions; \
             refusing to run."
        );
        std::process::exit(1);
    }
}
