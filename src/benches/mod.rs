//! Bench registry. Each bench module exposes `NAME` (CLI id) and
//! `run` (entry point). Add a bench by creating a module and
//! appending it to [`REGISTRY`].

pub mod min_now;
pub mod mpsc_1t;
pub mod mpsc_2t;
pub mod probe_mpsc_2t;
pub mod producer_consumer;
pub mod std_now;
pub mod tp_pc;

use crate::harness::RunCfg;

/// Bench entry-point signature.
pub type RunFn = fn(&RunCfg);

/// Static list of every registered bench, in display order.
pub const REGISTRY: &[(&str, RunFn)] = &[
    (min_now::NAME, min_now::run),
    (std_now::NAME, std_now::run),
    (mpsc_1t::NAME, mpsc_1t::run),
    (mpsc_2t::NAME, mpsc_2t::run),
    (probe_mpsc_2t::NAME, probe_mpsc_2t::run),
    (producer_consumer::NAME, producer_consumer::run),
    (tp_pc::NAME, tp_pc::run),
];

/// All registered bench names, in [`REGISTRY`] order. Used for CLI
/// help and the `all` resolution.
pub fn names() -> Vec<&'static str> {
    REGISTRY.iter().map(|(n, _)| *n).collect()
}

/// Resolve a list of CLI-requested names (or the literal `"all"`)
/// to an ordered list of [`RunFn`]s. Returns an error on any
/// unknown name.
pub fn resolve(requested: &[String]) -> Result<Vec<RunFn>, String> {
    let want_all = requested.iter().any(|n| n == "all");
    let names: Vec<&str> = if want_all {
        names()
    } else {
        requested.iter().map(|s| s.as_str()).collect()
    };

    let mut runners = Vec::with_capacity(names.len());
    for name in names {
        match REGISTRY.iter().find(|(n, _)| *n == name) {
            Some((_, run)) => runners.push(*run),
            None => {
                return Err(format!(
                    "unknown bench '{name}'. valid: all, {}",
                    self::names().join(", ")
                ));
            }
        }
    }
    Ok(runners)
}
