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

/// Pin the current thread to `logical_cpu`. No-op if `None` or if the CPU
/// id isn't present on this machine.
pub fn pin_current(logical_cpu: Option<usize>) {
    let Some(target) = logical_cpu else { return };
    let Some(ids) = core_affinity::get_core_ids() else {
        return;
    };
    if let Some(id) = ids.into_iter().find(|c| c.id == target) {
        core_affinity::set_for_current(id);
    }
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
}
