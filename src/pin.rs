//! Thread CPU-pinning helpers: `--pin` parsing, affinity snapshot
//! and restore, and human-readable mask/plan summaries.

use std::collections::BTreeSet;

/// Parse a `--pin` value (comma-separated list with optional ranges) into
/// an ordered vector of logical CPU IDs. Accepts forms like `"0,1"`,
/// `"0-11"`, `"0,3-5,7"`. Duplicates are preserved in the given order so
/// oversubscription (`"0,0,0"`) works.
pub fn parse_cores(spec: &str) -> Result<Vec<usize>, String> {
    let mut out = Vec::new();
    for part in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        match part.split_once('-') {
            None => out.push(
                part.parse::<usize>()
                    .map_err(|e| format!("invalid core id {part:?}: {e}"))?,
            ),
            Some((lo, hi)) => {
                let lo = lo
                    .trim()
                    .parse::<usize>()
                    .map_err(|e| format!("invalid range start {lo:?}: {e}"))?;
                let hi = hi
                    .trim()
                    .parse::<usize>()
                    .map_err(|e| format!("invalid range end {hi:?}: {e}"))?;
                if hi < lo {
                    return Err(format!("range {lo}-{hi} is empty"));
                }
                out.extend(lo..=hi);
            }
        }
    }
    Ok(out)
}

/// Print the current thread's CPU id to stderr with a label.
/// Useful for debugging pinning — not called in normal paths.
#[allow(dead_code)]
pub fn print_core_id(prompt: &str) {
    let cid = unsafe { libc::sched_getcpu() };
    eprintln!("{prompt}: cid={cid}");
}

/// Pin the current thread to `logical_cpu`. No-op if `None`.
///
/// Constructs a `CoreId` directly rather than querying
/// `get_core_ids()`, which only returns cores in the caller's
/// current affinity mask — after the first pin that mask is
/// narrowed and subsequent lookups for other cores would fail.
pub fn pin_current(logical_cpu: Option<usize>) {
    let Some(target) = logical_cpu else { return };
    core_affinity::set_for_current(core_affinity::CoreId { id: target });
}

/// Read the current thread's CPU affinity mask, or `None` if the
/// syscall fails (very unusual on Linux).
pub fn current_affinity() -> Option<libc::cpu_set_t> {
    let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    let ret = unsafe { libc::sched_getaffinity(0, size_of::<libc::cpu_set_t>(), &mut set) };
    (ret == 0).then_some(set)
}

/// Snapshot the current thread's affinity mask so it can be restored
/// later. Logs the captured mask at `info` level. Returns `None` on
/// syscall failure (very unusual on Linux).
pub fn save_affinity() -> Option<libc::cpu_set_t> {
    match current_affinity() {
        Some(set) => {
            log::info!("save_affinity: mask={}", affinity_summary(&set));
            Some(set)
        }
        None => {
            log::warn!("save_affinity: sched_getaffinity failed");
            None
        }
    }
}

/// Restore a previously-saved affinity mask. Used after calibration
/// to widen the mask back to what we started with (typically "all
/// CPUs the process was launched with", preserving any outer
/// `taskset` restrictions). Logs the restored mask at `info` level.
pub fn restore_affinity(set: &libc::cpu_set_t) {
    let ret = unsafe { libc::sched_setaffinity(0, size_of::<libc::cpu_set_t>(), set) };
    if ret == 0 {
        log::info!("restore_affinity: mask={}", affinity_summary(set));
    } else {
        log::warn!("restore_affinity: sched_setaffinity failed");
    }
}

/// Format a CPU set as a compact range list with a count suffix,
/// e.g. `"0-11,13-15 (15 cpus)"` or `"5 (1 cpu)"`.
pub fn affinity_summary(set: &libc::cpu_set_t) -> String {
    let cpus: Vec<usize> = (0..libc::CPU_SETSIZE as usize)
        .filter(|&i| unsafe { libc::CPU_ISSET(i, set) })
        .collect();
    if cpus.is_empty() {
        return "<empty>".to_string();
    }
    let mut ranges: Vec<String> = Vec::new();
    let mut start = cpus[0];
    let mut prev = cpus[0];
    for &c in &cpus[1..] {
        if c == prev + 1 {
            prev = c;
        } else {
            ranges.push(if start == prev {
                start.to_string()
            } else {
                format!("{start}-{prev}")
            });
            start = c;
            prev = c;
        }
    }
    ranges.push(if start == prev {
        start.to_string()
    } else {
        format!("{start}-{prev}")
    });
    format!(
        "{} ({} cpu{})",
        ranges.join(","),
        cpus.len(),
        if cpus.len() == 1 { "" } else { "s" }
    )
}

/// Report the pinning plan (what the user asked for) as a human-readable
/// summary for the startup banner.
pub fn plan_summary(cores: &[usize]) -> String {
    if cores.is_empty() {
        return "none (unpinned)".to_string();
    }
    let unique: BTreeSet<usize> = cores.iter().copied().collect();
    format!(
        "{cores:?} ({} slot{}, {} unique CPU{})",
        cores.len(),
        if cores.len() == 1 { "" } else { "s" },
        unique.len(),
        if unique.len() == 1 { "" } else { "s" },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_list() {
        assert_eq!(parse_cores("0,1,2").unwrap(), vec![0, 1, 2]);
    }

    #[test]
    fn parse_range() {
        assert_eq!(parse_cores("0-3").unwrap(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn parse_mixed() {
        assert_eq!(parse_cores("0,3-5,7").unwrap(), vec![0, 3, 4, 5, 7]);
    }

    #[test]
    fn parse_duplicates_preserved() {
        assert_eq!(parse_cores("0,0,0").unwrap(), vec![0, 0, 0]);
    }

    #[test]
    fn parse_empty_string_ok() {
        assert_eq!(parse_cores("").unwrap(), Vec::<usize>::new());
    }

    #[test]
    fn parse_reverse_range_errs() {
        assert!(parse_cores("5-3").is_err());
    }

    #[test]
    fn parse_garbage_errs() {
        assert!(parse_cores("abc").is_err());
        assert!(parse_cores("1-x").is_err());
    }

    #[test]
    fn pin_current_can_switch_cores() {
        let mut set = unsafe { std::mem::zeroed::<libc::cpu_set_t>() };
        let ret = unsafe { libc::sched_getaffinity(0, size_of::<libc::cpu_set_t>(), &mut set) };
        if ret != 0 {
            eprintln!("can't query affinity");
            return; // can't query affinity
        }
        let available = unsafe { libc::CPU_COUNT(&set) } as usize;
        if available < 2 {
            eprintln!("single-core or restricted by taskset");
            return; // single-core or restricted by taskset
        }
        let a = 0;
        let b = 1;

        pin_current(Some(a));
        super::print_core_id("after pin to a");
        let cpu_a = unsafe { libc::sched_getcpu() } as usize;
        assert_eq!(cpu_a, a);

        pin_current(Some(b));
        super::print_core_id("after pin to b");
        let cpu_b = unsafe { libc::sched_getcpu() } as usize;
        assert_eq!(cpu_b, b);
    }
}
