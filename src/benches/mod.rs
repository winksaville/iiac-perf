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
pub mod zcr_mpsc_v3_1t;
pub mod zcr_mpsc_v3_2t;
pub mod zcr_mpsc_v4_1t;
pub mod zcr_mpsc_v4_2t;
pub mod zcr_mpsc_v4_2t_wait;
pub mod zcr_spsc_v0_1t;
pub mod zcr_spsc_v0_2t;
pub mod zcr_spsc_v1_1t;
pub mod zcr_spsc_v1_2t;
pub mod zcr_spsc_v2_1t;
pub mod zcr_spsc_v2_2t;
pub mod zcr_spsc_v3_1t;
pub mod zcr_spsc_v3_2t;
pub mod zcr_spsc_v4_1t;
pub mod zcr_spsc_v4_2t;

use crate::harness::RunCfg;

/// Bench entry-point signature.
pub type RunFn = fn(&RunCfg);

/// A bench's threads in order, each by its role: the fixed roles, thread 0 first, then one
/// role repeated for however many more threads a thread count asks for. Thread `i` is pinned
/// to slot `i` of the pin pool and a thread past the pool's end runs unpinned, so the order is
/// what a placement's CPU list is read against.
#[derive(Debug, Clone, Copy)]
pub struct Roles {
    /// The roles every run has, thread 0 first.
    pub fixed: &'static [&'static str],
    /// The role of every thread past the fixed ones, `None` for a bench whose thread count is
    /// its fixed roles alone, which is every bench so far.
    pub repeated: Option<&'static str>,
}

impl Roles {
    /// One thread, the bench's main, timing its own work.
    pub const MAIN: Roles = Roles {
        fixed: &["main"],
        repeated: None,
    };
    /// A round trip: main times it and the worker it spawns answers.
    pub const MAIN_WORKER: Roles = Roles {
        fixed: &["main", "worker"],
        repeated: None,
    };
    /// A producer and a consumer, each timing its own loop, main only orchestrating.
    pub const PRODUCER_CONSUMER: Roles = Roles {
        fixed: &["producer", "consumer"],
        repeated: None,
    };

    /// The role of each thread, in order, for a run with `extra` threads past the fixed ones.
    /// A bench with no repeated role has its fixed roles whatever `extra` says.
    pub fn threads(&self, extra: usize) -> Vec<&'static str> {
        let mut roles = self.fixed.to_vec();
        if let Some(r) = self.repeated {
            roles.extend(std::iter::repeat_n(r, extra));
        }
        roles
    }
}

/// One registered bench: its CLI name, its entry point, and its threads' roles.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    /// The name the CLI and a record use.
    pub name: &'static str,
    /// The entry point.
    pub run: RunFn,
    /// Its threads in pin order.
    pub roles: Roles,
}

impl Entry {
    const fn new(name: &'static str, run: RunFn, roles: Roles) -> Entry {
        Entry { name, run, roles }
    }
}

/// Static list of every registered bench, in display order.
pub const REGISTRY: &[Entry] = &[
    Entry::new(min_now::NAME, min_now::run, Roles::MAIN),
    Entry::new(std_now::NAME, std_now::run, Roles::MAIN),
    Entry::new(mpsc_1t::NAME, mpsc_1t::run, Roles::MAIN),
    Entry::new(mpsc_2t::NAME, mpsc_2t::run, Roles::MAIN_WORKER),
    Entry::new(mpsc_2t_spin::NAME, mpsc_2t_spin::run, Roles::MAIN_WORKER),
    Entry::new(probe_mpsc_2t::NAME, probe_mpsc_2t::run, Roles::MAIN_WORKER),
    Entry::new(
        producer_consumer::NAME,
        producer_consumer::run,
        Roles::PRODUCER_CONSUMER,
    ),
    Entry::new(tp_pc::NAME, tp_pc::run, Roles::PRODUCER_CONSUMER),
    Entry::new(tp2_pc::NAME, tp2_pc::run, Roles::PRODUCER_CONSUMER),
    Entry::new(cb_chan_1t::NAME, cb_chan_1t::run, Roles::MAIN),
    Entry::new(cb_chan_2t::NAME, cb_chan_2t::run, Roles::MAIN_WORKER),
    Entry::new(cb_seg_1t::NAME, cb_seg_1t::run, Roles::MAIN),
    Entry::new(cb_seg_2t::NAME, cb_seg_2t::run, Roles::MAIN_WORKER),
    Entry::new(ice_ps_1t::NAME, ice_ps_1t::run, Roles::MAIN),
    Entry::new(ice_ps_2t::NAME, ice_ps_2t::run, Roles::MAIN_WORKER),
    Entry::new(ice_rr_1t::NAME, ice_rr_1t::run, Roles::MAIN),
    Entry::new(ice_rr_2t::NAME, ice_rr_2t::run, Roles::MAIN_WORKER),
    Entry::new(zcr_spsc_v0_1t::NAME, zcr_spsc_v0_1t::run, Roles::MAIN),
    Entry::new(
        zcr_spsc_v0_2t::NAME,
        zcr_spsc_v0_2t::run,
        Roles::MAIN_WORKER,
    ),
    Entry::new(zcr_mpsc_v0_1t::NAME, zcr_mpsc_v0_1t::run, Roles::MAIN),
    Entry::new(
        zcr_mpsc_v0_2t::NAME,
        zcr_mpsc_v0_2t::run,
        Roles::MAIN_WORKER,
    ),
    Entry::new(zcr_mpsc_v1_1t::NAME, zcr_mpsc_v1_1t::run, Roles::MAIN),
    Entry::new(
        zcr_mpsc_v1_2t::NAME,
        zcr_mpsc_v1_2t::run,
        Roles::MAIN_WORKER,
    ),
    Entry::new(zcr_mpsc_v2_1t::NAME, zcr_mpsc_v2_1t::run, Roles::MAIN),
    Entry::new(
        zcr_mpsc_v2_2t::NAME,
        zcr_mpsc_v2_2t::run,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v2_2t_ops::NAME_NOP,
        zcr_mpsc_v2_2t_ops::run_nop,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v2_2t_ops::NAME_STORE_SEQCST,
        zcr_mpsc_v2_2t_ops::run_store_seqcst,
        Roles::MAIN_WORKER,
    ),
    Entry::new(zcr_mpsc_v3_1t::NAME, zcr_mpsc_v3_1t::run, Roles::MAIN),
    Entry::new(
        zcr_mpsc_v3_2t::NAME,
        zcr_mpsc_v3_2t::run,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v3_1t::NAME_1SEG,
        zcr_mpsc_v3_1t::run_1seg,
        Roles::MAIN,
    ),
    Entry::new(
        zcr_mpsc_v3_2t::NAME_1SEG,
        zcr_mpsc_v3_2t::run_1seg,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v3_1t::NAME_SINGLE,
        zcr_mpsc_v3_1t::run_single,
        Roles::MAIN,
    ),
    Entry::new(
        zcr_mpsc_v3_2t::NAME_SINGLE,
        zcr_mpsc_v3_2t::run_single,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v3_1t::NAME_FUTEX,
        zcr_mpsc_v3_1t::run_futex,
        Roles::MAIN,
    ),
    Entry::new(
        zcr_mpsc_v3_2t::NAME_FUTEX,
        zcr_mpsc_v3_2t::run_futex,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v3_1t::NAME_1SEG_FUTEX,
        zcr_mpsc_v3_1t::run_1seg_futex,
        Roles::MAIN,
    ),
    Entry::new(
        zcr_mpsc_v3_2t::NAME_1SEG_FUTEX,
        zcr_mpsc_v3_2t::run_1seg_futex,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v3_1t::NAME_SINGLE_FUTEX,
        zcr_mpsc_v3_1t::run_single_futex,
        Roles::MAIN,
    ),
    Entry::new(
        zcr_mpsc_v3_2t::NAME_SINGLE_FUTEX,
        zcr_mpsc_v3_2t::run_single_futex,
        Roles::MAIN_WORKER,
    ),
    Entry::new(zcr_mpsc_v4_1t::NAME, zcr_mpsc_v4_1t::run, Roles::MAIN),
    Entry::new(
        zcr_mpsc_v4_2t::NAME,
        zcr_mpsc_v4_2t::run,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v4_1t::NAME_1SEG,
        zcr_mpsc_v4_1t::run_1seg,
        Roles::MAIN,
    ),
    Entry::new(
        zcr_mpsc_v4_2t::NAME_1SEG,
        zcr_mpsc_v4_2t::run_1seg,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v4_1t::NAME_SINGLE,
        zcr_mpsc_v4_1t::run_single,
        Roles::MAIN,
    ),
    Entry::new(
        zcr_mpsc_v4_2t::NAME_SINGLE,
        zcr_mpsc_v4_2t::run_single,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v4_1t::NAME_SOS,
        zcr_mpsc_v4_1t::run_sos,
        Roles::MAIN,
    ),
    Entry::new(
        zcr_mpsc_v4_2t::NAME_SOS,
        zcr_mpsc_v4_2t::run_sos,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v4_1t::NAME_1SEG_SOS,
        zcr_mpsc_v4_1t::run_1seg_sos,
        Roles::MAIN,
    ),
    Entry::new(
        zcr_mpsc_v4_2t::NAME_1SEG_SOS,
        zcr_mpsc_v4_2t::run_1seg_sos,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v4_1t::NAME_SINGLE_SOS,
        zcr_mpsc_v4_1t::run_single_sos,
        Roles::MAIN,
    ),
    Entry::new(
        zcr_mpsc_v4_2t::NAME_SINGLE_SOS,
        zcr_mpsc_v4_2t::run_single_sos,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v4_2t_wait::NAME_SLEEP,
        zcr_mpsc_v4_2t_wait::run_sleep,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v4_2t_wait::NAME_SPIN_SLEEP,
        zcr_mpsc_v4_2t_wait::run_spin_sleep,
        Roles::MAIN_WORKER,
    ),
    Entry::new(
        zcr_mpsc_v4_2t_wait::NAME_TIMED_SPIN,
        zcr_mpsc_v4_2t_wait::run_timed_spin,
        Roles::MAIN_WORKER,
    ),
    Entry::new(zcr_spsc_v1_1t::NAME, zcr_spsc_v1_1t::run, Roles::MAIN),
    Entry::new(
        zcr_spsc_v1_2t::NAME,
        zcr_spsc_v1_2t::run,
        Roles::MAIN_WORKER,
    ),
    Entry::new(zcr_spsc_v2_1t::NAME, zcr_spsc_v2_1t::run, Roles::MAIN),
    Entry::new(
        zcr_spsc_v2_2t::NAME,
        zcr_spsc_v2_2t::run,
        Roles::MAIN_WORKER,
    ),
    Entry::new(zcr_spsc_v3_1t::NAME, zcr_spsc_v3_1t::run, Roles::MAIN),
    Entry::new(
        zcr_spsc_v3_2t::NAME,
        zcr_spsc_v3_2t::run,
        Roles::MAIN_WORKER,
    ),
    Entry::new(zcr_spsc_v4_1t::NAME, zcr_spsc_v4_1t::run, Roles::MAIN),
    Entry::new(
        zcr_spsc_v4_2t::NAME,
        zcr_spsc_v4_2t::run,
        Roles::MAIN_WORKER,
    ),
];

/// All registered bench names, in [`REGISTRY`] order. Used for CLI
/// help and the `all` resolution.
pub fn names() -> Vec<&'static str> {
    REGISTRY.iter().map(|e| e.name).collect()
}

/// The registered bench named exactly `name`, the lookup a child process runs its one bench by.
pub fn find(name: &str) -> Option<&'static Entry> {
    REGISTRY.iter().find(|e| e.name == name)
}

/// Resolve a list of CLI-requested names (or the literal `"all"`)
/// to an ordered list of registered [`Entry`]s. A
/// name that matches no bench exactly runs every bench it is a
/// prefix of (`ice` -> all four ice benches, `mpsc` -> both mpsc
/// benches), and one that is no prefix either runs every bench
/// it matches as a regular expression (`zcr-[sm]psc-v[23]` -> the
/// v2 and v3 pairs of both rings), in [`REGISTRY`] order. Returns
/// an error on any name matching nothing, and on a pattern that
/// does not parse, since a name that is neither a bench nor a
/// pattern has no other reading.
pub fn resolve(requested: &[String]) -> Result<Vec<&'static Entry>, String> {
    if requested.iter().any(|n| n == "all") {
        return Ok(REGISTRY.iter().collect());
    }

    let mut runners = Vec::with_capacity(requested.len());
    for name in requested {
        if let Some(entry) = find(name) {
            runners.push(entry);
            continue;
        }
        let mut matched: Vec<&'static Entry> = REGISTRY
            .iter()
            .filter(|e| e.name.starts_with(name.as_str()))
            .collect();
        if matched.is_empty() {
            let re = regex::Regex::new(name).map_err(|e| {
                format!(
                    "unknown bench '{name}', and as a pattern it does not parse: {e}\nvalid: all, {}",
                    self::names().join(", ")
                )
            })?;
            matched = REGISTRY.iter().filter(|e| re.is_match(e.name)).collect();
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
        resolve(&requested).map(|v| v.into_iter().map(|e| e.name).collect())
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
            vec![
                "zcr-mpsc-v3-1t-multi-2seg-stfe-wtnone",
                "zcr-mpsc-v3-2t-multi-2seg-stfe-wtnone",
                "zcr-mpsc-v3-1t-multi-1seg-stfe-wtnone",
                "zcr-mpsc-v3-2t-multi-1seg-stfe-wtnone",
                "zcr-mpsc-v3-1t-single-stfe-wtnone",
                "zcr-mpsc-v3-2t-single-stfe-wtnone",
                "zcr-mpsc-v3-1t-multi-2seg-stfe-wtnone-futex",
                "zcr-mpsc-v3-2t-multi-2seg-stfe-wtnone-futex",
                "zcr-mpsc-v3-1t-multi-1seg-stfe-wtnone-futex",
                "zcr-mpsc-v3-2t-multi-1seg-stfe-wtnone-futex",
                "zcr-mpsc-v3-1t-single-stfe-wtnone-futex",
                "zcr-mpsc-v3-2t-single-stfe-wtnone-futex",
                "zcr-spsc-v3-1t",
                "zcr-spsc-v3-2t"
            ]
        );
        assert_eq!(names_of(&["-v3-.*1t$"]).unwrap(), vec!["zcr-spsc-v3-1t"]);
        assert_eq!(
            names_of(&["zcr-mpsc-v4"]).unwrap(),
            vec![
                "zcr-mpsc-v4-1t-multi-2seg-stfe-wtnone",
                "zcr-mpsc-v4-2t-multi-2seg-stfe-wtnone",
                "zcr-mpsc-v4-1t-multi-1seg-stfe-wtnone",
                "zcr-mpsc-v4-2t-multi-1seg-stfe-wtnone",
                "zcr-mpsc-v4-1t-single-stfe-wtnone",
                "zcr-mpsc-v4-2t-single-stfe-wtnone",
                "zcr-mpsc-v4-1t-multi-2seg-stfe-wtnone-spinorsleep-futex",
                "zcr-mpsc-v4-2t-multi-2seg-stfe-wtnone-spinorsleep-futex",
                "zcr-mpsc-v4-1t-multi-1seg-stfe-wtnone-spinorsleep-futex",
                "zcr-mpsc-v4-2t-multi-1seg-stfe-wtnone-spinorsleep-futex",
                "zcr-mpsc-v4-1t-single-stfe-wtnone-spinorsleep-futex",
                "zcr-mpsc-v4-2t-single-stfe-wtnone-spinorsleep-futex",
                "zcr-mpsc-v4-2t-single-st0-wtfe-sleep-futex",
                "zcr-mpsc-v4-2t-single-st1us-wtfe-sleep-futex",
                "zcr-mpsc-v4-2t-single-st1us-wtnone"
            ]
        );
        assert_eq!(
            names_of(&["mpsc-v3-2t-.*-wtnone$"]).unwrap(),
            vec![
                "zcr-mpsc-v3-2t-multi-2seg-stfe-wtnone",
                "zcr-mpsc-v3-2t-multi-1seg-stfe-wtnone",
                "zcr-mpsc-v3-2t-single-stfe-wtnone"
            ]
        );
    }

    #[test]
    fn a_benchs_roles_agree_with_its_thread_count() {
        for e in REGISTRY {
            let n = e.roles.threads(0).len();
            if e.name.ends_with("-1t") || e.name.contains("-1t-") || e.name.ends_with("-now") {
                assert_eq!(n, 1, "{}", e.name);
            } else {
                assert_eq!(n, 2, "{}", e.name);
            }
        }
    }

    #[test]
    fn a_repeated_role_fills_the_extra_threads() {
        let mpsc = Roles {
            fixed: &["consumer"],
            repeated: Some("producer"),
        };
        assert_eq!(mpsc.threads(2), ["consumer", "producer", "producer"]);
        assert_eq!(Roles::MAIN_WORKER.threads(3), ["main", "worker"]);
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
