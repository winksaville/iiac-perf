//! Bench registry. Each bench module exposes `NAME` (CLI id) and
//! `run` (entry point). Add a bench by creating a module and
//! appending it to [`REGISTRY`].

pub mod cb_chan_1t;
pub mod cb_chan_2t;
pub mod cb_seg_1t;
pub mod cb_seg_2t;
pub mod ice_ps_1t;
pub mod ice_ps_2t;
pub mod ice_rr_1t;
pub mod ice_rr_2t;
pub mod min_now;
pub mod mpsc_1t;
pub mod mpsc_2t;
pub mod mpsc_2t_spin;
pub mod probe_mpsc_2t;
pub mod producer_consumer;
pub mod std_now;
pub mod tp2_pc;
pub mod tp_pc;
pub mod zcr_common;
pub mod zcr_mpsc_v0_1t;
pub mod zcr_mpsc_v0_2t;
pub mod zcr_mpsc_v1_1t;
pub mod zcr_mpsc_v1_2t;
pub mod zcr_mpsc_v2_1t;
pub mod zcr_mpsc_v2_2t;
pub mod zcr_mpsc_v2_2t_ops;
pub mod zcr_spsc_v0_1t;
pub mod zcr_spsc_v0_2t;
pub mod zcr_spsc_v1_1t;
pub mod zcr_spsc_v1_2t;
pub mod zcr_spsc_v2_1t;
pub mod zcr_spsc_v2_2t;
pub mod zcr_spsc_v3_1t;
pub mod zcr_spsc_v3_2t;

use crate::harness::RunCfg;

/// Bench entry-point signature.
pub type RunFn = fn(&RunCfg);

/// Static list of every registered bench, in display order.
pub const REGISTRY: &[(&str, RunFn)] = &[
    (min_now::NAME, min_now::run),
    (std_now::NAME, std_now::run),
    (mpsc_1t::NAME, mpsc_1t::run),
    (mpsc_2t::NAME, mpsc_2t::run),
    (mpsc_2t_spin::NAME, mpsc_2t_spin::run),
    (probe_mpsc_2t::NAME, probe_mpsc_2t::run),
    (producer_consumer::NAME, producer_consumer::run),
    (tp_pc::NAME, tp_pc::run),
    (tp2_pc::NAME, tp2_pc::run),
    (cb_chan_1t::NAME, cb_chan_1t::run),
    (cb_chan_2t::NAME, cb_chan_2t::run),
    (cb_seg_1t::NAME, cb_seg_1t::run),
    (cb_seg_2t::NAME, cb_seg_2t::run),
    (ice_ps_1t::NAME, ice_ps_1t::run),
    (ice_ps_2t::NAME, ice_ps_2t::run),
    (ice_rr_1t::NAME, ice_rr_1t::run),
    (ice_rr_2t::NAME, ice_rr_2t::run),
    (zcr_spsc_v0_1t::NAME, zcr_spsc_v0_1t::run),
    (zcr_spsc_v0_2t::NAME, zcr_spsc_v0_2t::run),
    (zcr_mpsc_v0_1t::NAME, zcr_mpsc_v0_1t::run),
    (zcr_mpsc_v0_2t::NAME, zcr_mpsc_v0_2t::run),
    (zcr_mpsc_v1_1t::NAME, zcr_mpsc_v1_1t::run),
    (zcr_mpsc_v1_2t::NAME, zcr_mpsc_v1_2t::run),
    (zcr_mpsc_v2_1t::NAME, zcr_mpsc_v2_1t::run),
    (zcr_mpsc_v2_2t::NAME, zcr_mpsc_v2_2t::run),
    (zcr_mpsc_v2_2t_ops::NAME_NOP, zcr_mpsc_v2_2t_ops::run_nop),
    (
        zcr_mpsc_v2_2t_ops::NAME_STORE_SEQCST,
        zcr_mpsc_v2_2t_ops::run_store_seqcst,
    ),
    (zcr_spsc_v1_1t::NAME, zcr_spsc_v1_1t::run),
    (zcr_spsc_v1_2t::NAME, zcr_spsc_v1_2t::run),
    (zcr_spsc_v2_1t::NAME, zcr_spsc_v2_1t::run),
    (zcr_spsc_v2_2t::NAME, zcr_spsc_v2_2t::run),
    (zcr_spsc_v3_1t::NAME, zcr_spsc_v3_1t::run),
    (zcr_spsc_v3_2t::NAME, zcr_spsc_v3_2t::run),
];

/// All registered bench names, in [`REGISTRY`] order. Used for CLI
/// help and the `all` resolution.
pub fn names() -> Vec<&'static str> {
    REGISTRY.iter().map(|(n, _)| *n).collect()
}

/// The registered bench named exactly `name`, the lookup a child process runs its one bench by.
pub fn find(name: &str) -> Option<RunFn> {
    REGISTRY
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, run)| *run)
}

/// Resolve a list of CLI-requested names (or the literal `"all"`)
/// to an ordered list of registered names and their [`RunFn`]s. A
/// name that matches no bench exactly runs every bench it is a
/// prefix of (`ice` -> all four ice benches, `mpsc` -> both mpsc
/// benches), and one that is no prefix either runs every bench
/// it matches as a regular expression (`zcr-[sm]psc-v[23]` -> the
/// v2 and v3 pairs of both rings), in [`REGISTRY`] order. Returns
/// an error on any name matching nothing, and on a pattern that
/// does not parse, since a name that is neither a bench nor a
/// pattern has no other reading.
pub fn resolve(requested: &[String]) -> Result<Vec<(&'static str, RunFn)>, String> {
    if requested.iter().any(|n| n == "all") {
        return Ok(REGISTRY.to_vec());
    }

    let mut runners = Vec::with_capacity(requested.len());
    for name in requested {
        if let Some(entry) = REGISTRY.iter().find(|(n, _)| n == name) {
            runners.push(*entry);
            continue;
        }
        let mut matched: Vec<(&'static str, RunFn)> = REGISTRY
            .iter()
            .filter(|(n, _)| n.starts_with(name.as_str()))
            .copied()
            .collect();
        if matched.is_empty() {
            let re = regex::Regex::new(name).map_err(|e| {
                format!(
                    "unknown bench '{name}', and as a pattern it does not parse: {e}\nvalid: all, {}",
                    self::names().join(", ")
                )
            })?;
            matched = REGISTRY
                .iter()
                .filter(|(n, _)| re.is_match(n))
                .copied()
                .collect();
        }
        if matched.is_empty() {
            return Err(format!(
                "unknown bench '{name}'. valid: all, {}",
                self::names().join(", ")
            ));
        }
        runners.extend(matched);
    }
    Ok(runners)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names_of(requested: &[&str]) -> Result<Vec<&'static str>, String> {
        let requested: Vec<String> = requested.iter().map(|s| s.to_string()).collect();
        resolve(&requested).map(|v| v.into_iter().map(|(n, _)| n).collect())
    }

    #[test]
    fn a_pattern_resolves_after_exact_and_prefix() {
        assert_eq!(names_of(&["min-now"]).unwrap(), vec!["min-now"]);
        assert_eq!(
            names_of(&["zcr-spsc-v3"]).unwrap(),
            vec!["zcr-spsc-v3-1t", "zcr-spsc-v3-2t"]
        );
        assert_eq!(
            names_of(&["zcr-[sm]psc-v[23]-2t$"]).unwrap(),
            vec!["zcr-mpsc-v2-2t", "zcr-spsc-v2-2t", "zcr-spsc-v3-2t"]
        );
        assert_eq!(
            names_of(&["(s|m)psc-v3"]).unwrap(),
            vec!["zcr-spsc-v3-1t", "zcr-spsc-v3-2t"]
        );
        assert_eq!(names_of(&["-v3-.*1t$"]).unwrap(), vec!["zcr-spsc-v3-1t"]);
    }

    #[test]
    fn a_pattern_matching_nothing_is_unknown() {
        let e = names_of(&["zcr-v9"]).unwrap_err();
        assert!(e.starts_with("unknown bench 'zcr-v9'"), "{e}");
    }

    #[test]
    fn a_pattern_that_does_not_parse_says_so() {
        let e = names_of(&["zcr-{s|m}psc"]).unwrap_err();
        assert!(e.contains("as a pattern it does not parse"), "{e}");
        let e = names_of(&["zcr-("]).unwrap_err();
        assert!(e.contains("as a pattern it does not parse"), "{e}");
    }
}
