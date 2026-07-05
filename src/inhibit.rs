//! Automatic system-sleep inhibition: re-exec the process under
//! `systemd-inhibit` so an idle-suspend can't poison a long bench
//! run — a mid-sample suspend inflates that sample by the whole
//! sleep gap (see the harness suspend detection, which remains
//! the backstop when inhibition is off or unavailable).

use std::os::unix::process::CommandExt;
use std::process::Command;

use log::warn;

/// Env var marking the re-exec'd child so it doesn't recurse.
const GUARD: &str = "IIAC_PERF_INHIBITED";

/// Ensure the process holds a `systemd-inhibit --what=sleep` lock,
/// re-exec'ing itself once under the wrapper if needed. Returns
/// the status line for the startup banner:
///
/// - `--no-inhibit` passed → disabled by request;
/// - guard env var present → active (we are the re-exec'd child,
///   the wrapper holds the lock for our lifetime);
/// - re-exec failed (`systemd-inhibit` absent, non-systemd box)
///   → run continues uninhibited; suspend detection still flags
///   a poisoned run.
pub fn ensure(no_inhibit: bool) -> String {
    if no_inhibit {
        return "disabled (--no-inhibit)".to_string();
    }
    if std::env::var_os(GUARD).is_some() {
        return "active (systemd-inhibit --what=sleep)".to_string();
    }
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            warn!("sleep inhibit unavailable (current_exe: {e}); continuing uninhibited");
            return "unavailable (current_exe failed; run uninhibited)".to_string();
        }
    };
    let err = Command::new("systemd-inhibit")
        .arg("--what=sleep")
        .arg("--who=iiac-perf")
        .arg("--why=benchmark run in progress")
        .arg("--mode=block")
        .arg(exe)
        .args(std::env::args_os().skip(1))
        .env(GUARD, "1")
        .exec();
    // exec only returns on failure (e.g. systemd-inhibit absent).
    warn!("sleep inhibit unavailable ({err}); continuing uninhibited");
    "unavailable (systemd-inhibit failed; run uninhibited)".to_string()
}
