//! Bench registry. Each bench module exposes `NAME` (CLI id) and
//! `run` (entry point). Add a bench by creating a module and
//! appending it to [`REGISTRY`].

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
pub mod zcr_mpsc_1t;
pub mod zcr_mpsc_2t;
pub mod zcr_with_1t;
pub mod zcr_with_2t;

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
    (ice_ps_1t::NAME, ice_ps_1t::run),
    (ice_ps_2t::NAME, ice_ps_2t::run),
    (ice_rr_1t::NAME, ice_rr_1t::run),
    (ice_rr_2t::NAME, ice_rr_2t::run),
    (zcr_with_1t::NAME, zcr_with_1t::run),
    (zcr_with_2t::NAME, zcr_with_2t::run),
    (zcr_mpsc_1t::NAME, zcr_mpsc_1t::run),
    (zcr_mpsc_2t::NAME, zcr_mpsc_2t::run),
];

/// All registered bench names, in [`REGISTRY`] order. Used for CLI
/// help and the `all` resolution.
pub fn names() -> Vec<&'static str> {
    REGISTRY.iter().map(|(n, _)| *n).collect()
}

/// Resolve a list of CLI-requested names (or the literal `"all"`)
/// to an ordered list of [`RunFn`]s. A name that matches no bench
/// exactly runs every bench it is a prefix of (`ice` → all four
/// ice benches, `mpsc` → both mpsc benches), in [`REGISTRY`]
/// order. Returns an error on any name matching nothing.
pub fn resolve(requested: &[String]) -> Result<Vec<RunFn>, String> {
    if requested.iter().any(|n| n == "all") {
        return Ok(REGISTRY.iter().map(|(_, run)| *run).collect());
    }

    let mut runners = Vec::with_capacity(requested.len());
    for name in requested {
        if let Some((_, run)) = REGISTRY.iter().find(|(n, _)| n == name) {
            runners.push(*run);
            continue;
        }
        let prefixed: Vec<RunFn> = REGISTRY
            .iter()
            .filter(|(n, _)| n.starts_with(name.as_str()))
            .map(|(_, run)| *run)
            .collect();
        if prefixed.is_empty() {
            return Err(format!(
                "unknown bench '{name}'. valid: all, {}",
                self::names().join(", ")
            ));
        }
        runners.extend(prefixed);
    }
    Ok(runners)
}
