pub mod min_now;
pub mod mpsc_1t;
pub mod mpsc_2t;
pub mod std_now;

use crate::overhead::Overhead;

pub type RunFn = fn(u64, &Overhead);

pub const REGISTRY: &[(&str, RunFn)] = &[
    (min_now::NAME, min_now::run),
    (std_now::NAME, std_now::run),
    (mpsc_1t::NAME, mpsc_1t::run),
    (mpsc_2t::NAME, mpsc_2t::run),
];

pub fn names() -> Vec<&'static str> {
    REGISTRY.iter().map(|(n, _)| *n).collect()
}

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
