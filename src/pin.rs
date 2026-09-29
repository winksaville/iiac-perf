//! Thread CPU-pinning helpers: `--pin-cpus` parsing, affinity
//! snapshot and restore, human-readable mask/plan summaries, and
//! the pool's placement, what its CPUs share.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::OnceLock;

/// Parse a `--pin-cpus` value (comma-separated list with optional ranges)
/// into an ordered vector of CPU ids (the kernel's schedulable unit, sysfs
/// `cpuN`). Accepts forms like `"0,1"`, `"0-11"`, `"0,3-5,7"`. Duplicates
/// are preserved in the given order so oversubscription (`"0,0,0"`) works.
pub fn parse_cpus(spec: &str) -> Result<Vec<usize>, String> {
    let mut out = Vec::new();
    for part in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        match part.split_once('-') {
            None => out.push(
                part.parse::<usize>()
                    .map_err(|e| format!("invalid CPU id {part:?}: {e}"))?,
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
pub fn print_cpu_id(prompt: &str) {
    let cid = unsafe { libc::sched_getcpu() };
    eprintln!("{prompt}: cid={cid}");
}

/// The process's affinity before anything pinned, what an unpinned
/// thread returns to.
static STARTUP_AFFINITY: OnceLock<libc::cpu_set_t> = OnceLock::new();

/// Remember the process's affinity as it started, called before
/// main pins itself. A thread inherits its spawner's affinity, so
/// without this a worker the pool leaves unpinned would run on
/// main's CPU, two spinners sharing one CPU. Later calls keep the
/// first.
pub fn remember_startup_affinity() {
    if let Some(set) = current_affinity() {
        // A second call finds the first's set, which is the one wanted.
        let _ = STARTUP_AFFINITY.set(set);
    }
}

/// The remembered startup affinity as a CPU list, the current
/// affinity when none was remembered, and empty when neither reads.
/// A parent hands it to its children, which inherit the parent's
/// pinned main and so cannot read the invocation's own.
pub fn startup_cpus() -> Vec<usize> {
    let Some(set) = STARTUP_AFFINITY.get().copied().or_else(current_affinity) else {
        return Vec::new();
    };
    (0..libc::CPU_SETSIZE as usize)
        .filter(|&i| unsafe { libc::CPU_ISSET(i, &set) })
        .collect()
}

/// Remember `cpus`, a parent's [`startup_cpus`], as this process's
/// startup affinity, falling back to [`remember_startup_affinity`]
/// when the list is empty.
pub fn set_startup_cpus(cpus: &[usize]) {
    if cpus.is_empty() {
        return remember_startup_affinity();
    }
    let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    for &c in cpus {
        unsafe { libc::CPU_SET(c, &mut set) };
    }
    // A second call finds the first's set, which is the one wanted.
    let _ = STARTUP_AFFINITY.set(set);
}

/// Pin the current thread to `logical_cpu`, or with `None` return
/// it to the process's startup affinity, so an unpinned thread is
/// the scheduler's to place rather than its spawner's CPU. `None`
/// is a no-op when the startup affinity was never remembered.
///
/// Constructs a `CoreId` directly rather than querying
/// `get_core_ids()`, which only returns cores in the caller's
/// current affinity mask — after the first pin that mask is
/// narrowed and subsequent lookups for other cores would fail.
pub fn pin_current(logical_cpu: Option<usize>) {
    match logical_cpu {
        Some(target) => {
            core_affinity::set_for_current(core_affinity::CoreId { id: target });
        }
        None => {
            if let Some(set) = STARTUP_AFFINITY.get() {
                unsafe { libc::sched_setaffinity(0, size_of::<libc::cpu_set_t>(), set) };
            }
        }
    }
}

/// Read the current thread's CPU affinity mask, or `None` if the
/// syscall fails (very unusual on Linux).
pub fn current_affinity() -> Option<libc::cpu_set_t> {
    let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    let ret = unsafe { libc::sched_getaffinity(0, size_of::<libc::cpu_set_t>(), &mut set) };
    (ret == 0).then_some(set)
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
pub fn plan_summary(cpus: &[usize]) -> String {
    if cpus.is_empty() {
        return "none (unpinned)".to_string();
    }
    let unique: BTreeSet<usize> = cpus.iter().copied().collect();
    let label = match placement_label(cpus, &sysfs_topology(cpus[0])) {
        Some(l) => format!(", {l}"),
        None => String::new(),
    };
    format!(
        "{cpus:?} ({} slot{}, {} unique CPU{}{label})",
        cpus.len(),
        if cpus.len() == 1 { "" } else { "s" },
        unique.len(),
        if unique.len() == 1 { "" } else { "s" },
    )
}

/// What one CPU shares with the others, from sysfs: its SMT
/// siblings and the CPUs on its L3.
pub struct Topology {
    /// `topology/thread_siblings_list`: the CPUs of its core.
    pub siblings: Vec<usize>,
    /// `cache/index3/shared_cpu_list`: the CPUs of its L3, the
    /// CCX on AMD, and the sibling list when the index is absent.
    pub l3: Vec<usize>,
}

/// A sysfs CPU list, `None` when the file is unreadable.
fn sysfs_cpus(path: &Path) -> Option<Vec<usize>> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| parse_cpus(s.trim()).ok())
        .filter(|v| !v.is_empty())
}

/// `cpu`'s topology from `/sys/devices/system/cpu`, `None` when
/// its sibling list cannot be read, the L3 falling back to the
/// siblings, so a host without cache index 3 labels SMT and
/// x-CCX and never CCX.
pub fn sysfs_topology(cpu: usize) -> Option<Topology> {
    let dir = Path::new("/sys/devices/system/cpu").join(format!("cpu{cpu}"));
    let siblings = sysfs_cpus(&dir.join("topology/thread_siblings_list"))?;
    let l3 = sysfs_cpus(&dir.join("cache/index3/shared_cpu_list"))
        // OK: no cache index 3 means the L3 is unknown, and the
        // siblings are the smallest set the CPU surely shares, so
        // the label falls to SMT or x-CCX, never a false CCX.
        .unwrap_or_else(|| siblings.clone());
    Some(Topology { siblings, l3 })
}

/// The pool's placement, in zc-ring-x1's measurement tools' form,
/// judged from its first CPU's topology: `core` when the pool is
/// one CPU, `SMT` when every CPU is on its core, `CCX` when every
/// CPU is on its L3, and `x-CCX` otherwise. `None` for an empty
/// pool or an unreadable topology, so the caller prints nothing
/// rather than a guess.
pub fn placement_label(cpus: &[usize], topo: &Option<Topology>) -> Option<&'static str> {
    let unique: BTreeSet<usize> = cpus.iter().copied().collect();
    if unique.is_empty() {
        return None;
    }
    if unique.len() == 1 {
        return Some("core");
    }
    let topo = topo.as_ref()?;
    if unique.iter().all(|c| topo.siblings.contains(c)) {
        Some("SMT")
    } else if unique.iter().all(|c| topo.l3.contains(c)) {
        Some("CCX")
    } else {
        Some("x-CCX")
    }
}

/// The pool as a run line names it: `unpinned`, `core 11`, or the
/// CPUs then the label, `11,23 SMT`, the CPUs alone when the
/// topology is unreadable.
pub fn placement(cpus: &[usize]) -> String {
    if cpus.is_empty() {
        return "unpinned".to_string();
    }
    let list = cpus
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(",");
    match placement_label(cpus, &sysfs_topology(cpus[0])) {
        Some("core") => format!("core {list}"),
        Some(l) => format!("{list} {l}"),
        None => list,
    }
}

/// The CPU each of a bench's `threads` threads is pinned to, in
/// thread order: the pool's slot `i` for thread `i`, and `None`,
/// unpinned, for a thread past the pool's end.
pub fn thread_cpus(pool: &[usize], threads: usize) -> Vec<Option<usize>> {
    (0..threads).map(|i| pool.get(i).copied()).collect()
}

/// The placement of a bench's threads: [`placement_label`] over
/// their CPUs when every thread is pinned, `partial` when some are
/// and some are not, and `None` when none are or the topology is
/// unreadable.
pub fn threads_label(cpus: &[Option<usize>], topo: &Option<Topology>) -> Option<&'static str> {
    let pinned: Vec<usize> = cpus.iter().flatten().copied().collect();
    if pinned.is_empty() {
        None
    } else if pinned.len() < cpus.len() {
        Some("partial")
    } else {
        placement_label(&pinned, topo)
    }
}

/// A bench's threads as a run line names them: the placement of
/// the CPUs they use, [`placement`]'s form, then each role and its
/// CPU when there are several, `11,23 SMT (main 11, worker 23)`,
/// and `11 partial (main 11, worker unpinned)` when the pool runs
/// out. A 1t bench on a pair's pool is `core 11`, since that is
/// all it uses. No roles, a parent's config for several benches,
/// names the pool.
pub fn threads_placement(roles: &[&str], pool: &[usize]) -> String {
    if roles.is_empty() || pool.is_empty() {
        return placement(pool);
    }
    let cpus = thread_cpus(pool, roles.len());
    let pinned: Vec<usize> = cpus.iter().flatten().copied().collect();
    let head = if pinned.len() < cpus.len() {
        let list = pinned
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        format!("{list} partial")
    } else {
        placement(&pinned)
    };
    if roles.len() < 2 {
        return head;
    }
    let each = roles
        .iter()
        .zip(&cpus)
        .map(|(role, cpu)| match cpu {
            Some(c) => format!("{role} {c}"),
            None => format!("{role} unpinned"),
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{head} ({each})")
}

/// The name a placement list uses for a pool of no CPUs, a run the
/// scheduler places, and which `all` ends with.
pub const UNPINNED: &str = "unpinned";

/// One placement a run pins at: the name it was given, a
/// `[profiles]` entry or [`UNPINNED`], `None` for a CPU list, and
/// the CPUs it resolved to, empty when unpinned.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    /// The name, `None` for a CPU list given as one.
    pub name: Option<String>,
    /// The pool, empty when unpinned.
    pub cpus: Vec<usize>,
}

impl Placement {
    /// How a run line and the table name it: its name, or the CPU
    /// list when it has none, `unpinned` for an empty one.
    pub fn label(&self) -> String {
        match &self.name {
            Some(n) => n.clone(),
            None if self.cpus.is_empty() => UNPINNED.to_string(),
            None => self
                .cpus
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(","),
        }
    }

    /// Whether the name is a `[profiles]` entry, which
    /// [`check_named`] checks against the host.
    pub fn is_profile(&self) -> bool {
        self.name.as_deref().is_some_and(|n| n != UNPINNED)
    }
}

/// Where `all` puts a profile name, nearest first: the names sysfs
/// can judge in the order their CPUs grow apart, then every other
/// name, `x-ccd` or the user's own, whose distance only its author
/// knows.
pub fn placement_rank(name: &str) -> usize {
    match promised_label(name) {
        Some("core") => 0,
        Some("SMT") => 1,
        Some("CCX") => 2,
        Some("x-CCX") => 3,
        _ => 4,
    }
}

/// The placement label a profile name promises, for the names
/// sysfs can judge, in any case: `core`, `smt`, `ccx`, and `x-ccx`.
/// Any other name, `x-ccd` or the user's own, promises nothing
/// sysfs can check.
fn promised_label(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "core" => Some("core"),
        "smt" => Some("SMT"),
        "ccx" => Some("CCX"),
        "x-ccx" => Some("x-CCX"),
        _ => None,
    }
}

/// Check a named placement, a `[profiles]` entry, against this
/// host: every CPU online, and a name sysfs can judge matching
/// sysfs's label for its CPUs, so `smt` naming two cores is an
/// error rather than a mislabeled run. A name sysfs cannot judge,
/// `x-ccd` on a 3900X whose CPUs all read one die, is checked for
/// its CPUs alone, and the banner's bench pin row shows sysfs's
/// label beside it.
pub fn check_named(name: &str, cpus: &[usize]) -> Result<(), String> {
    let online = std::fs::read_to_string("/sys/devices/system/cpu/online")
        .ok()
        .and_then(|s| parse_cpus(s.trim()).ok());
    let topo = cpus.first().and_then(|&c| sysfs_topology(c));
    check_named_on(name, cpus, online.as_deref(), &topo)
}

/// [`check_named`] over a given online list and topology, `None`
/// for either when sysfs could not be read, which skips that check
/// rather than refusing a run over a file the kernel did not offer.
fn check_named_on(
    name: &str,
    cpus: &[usize],
    online: Option<&[usize]>,
    topo: &Option<Topology>,
) -> Result<(), String> {
    let list = cpus
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(",");
    if let Some(online) = online {
        let off: Vec<String> = cpus
            .iter()
            .filter(|c| !online.contains(c))
            .map(|c| c.to_string())
            .collect();
        if !off.is_empty() {
            return Err(format!(
                "the profile {name} = \"{list}\" names CPU {} that this host does not have \
                 online. Fix the entry in [profiles] in ~/.config/iiac-perf/config.toml.",
                off.join(",")
            ));
        }
    }
    let (Some(promised), Some(actual)) = (promised_label(name), placement_label(cpus, topo)) else {
        return Ok(());
    };
    if promised == actual {
        return Ok(());
    }
    Err(format!(
        "the profile {name} = \"{list}\" is {actual} by this host's topology, not {promised}. \
         Fix the entry in [profiles] in ~/.config/iiac-perf/config.toml, and `lscpu -e` shows \
         which CPUs pair."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 3900X's CPU 11: siblings 11 and 23, L3 the CCX 9-11 and 21-23.
    fn cpu11() -> Option<Topology> {
        Some(Topology {
            siblings: vec![11, 23],
            l3: vec![9, 10, 11, 21, 22, 23],
        })
    }

    #[test]
    fn placement_labels_agree_with_the_demo() {
        assert_eq!(placement_label(&[11, 23], &cpu11()), Some("SMT"));
        assert_eq!(placement_label(&[11, 10], &cpu11()), Some("CCX"));
        assert_eq!(placement_label(&[11, 8], &cpu11()), Some("x-CCX"));
        assert_eq!(placement_label(&[11], &cpu11()), Some("core"));
        assert_eq!(placement_label(&[11, 11], &cpu11()), Some("core"));
        assert_eq!(placement_label(&[], &cpu11()), None);
    }

    #[test]
    fn placement_label_is_none_without_a_topology() {
        assert_eq!(placement_label(&[11, 23], &None), None);
        assert_eq!(placement_label(&[11], &None), Some("core"));
    }

    #[test]
    fn placement_mixes_the_pool_and_a_far_cpu_to_x_ccx() {
        assert_eq!(placement_label(&[11, 23, 10], &cpu11()), Some("CCX"));
        assert_eq!(placement_label(&[11, 23, 0], &cpu11()), Some("x-CCX"));
    }

    #[test]
    fn an_unpinned_thread_leaves_its_spawners_cpu() {
        remember_startup_affinity();
        let Some(start) = current_affinity() else {
            return; // can't query affinity
        };
        if unsafe { libc::CPU_COUNT(&start) } < 2 {
            return; // one CPU, nothing to leave
        }
        let first = (0..libc::CPU_SETSIZE as usize)
            .find(|&i| unsafe { libc::CPU_ISSET(i, &start) })
            .expect("a CPU in a non-empty set");
        pin_current(Some(first));
        let spawned = std::thread::spawn(|| {
            pin_current(None);
            current_affinity().map(|s| unsafe { libc::CPU_COUNT(&s) })
        })
        .join()
        .expect("the thread joins");
        assert!(spawned.is_some_and(|n| n > 1), "{spawned:?}");
    }

    #[test]
    fn threads_past_the_pool_run_unpinned() {
        assert_eq!(thread_cpus(&[11, 23], 2), [Some(11), Some(23)]);
        assert_eq!(thread_cpus(&[11, 23], 1), [Some(11)]);
        assert_eq!(thread_cpus(&[11], 2), [Some(11), None]);
        assert_eq!(thread_cpus(&[], 2), [None, None]);
    }

    #[test]
    fn threads_label_reads_only_the_cpus_the_threads_use() {
        assert_eq!(threads_label(&[Some(11), Some(23)], &cpu11()), Some("SMT"));
        assert_eq!(threads_label(&[Some(11)], &cpu11()), Some("core"));
        assert_eq!(threads_label(&[Some(11), None], &cpu11()), Some("partial"));
        assert_eq!(threads_label(&[None, None], &cpu11()), None);
    }

    #[test]
    fn a_run_line_names_each_role_and_its_cpu() {
        assert_eq!(threads_placement(&["main"], &[]), "unpinned");
        assert_eq!(threads_placement(&[], &[3, 5]), placement(&[3, 5]));
        assert_eq!(
            threads_placement(&["main", "worker"], &[3]),
            "3 partial (main 3, worker unpinned)"
        );
        let both = threads_placement(&["main", "worker"], &[3, 5]);
        assert!(both.ends_with(" (main 3, worker 5)"), "{both}");
    }

    #[test]
    fn a_name_sysfs_can_judge_must_match_the_topology() {
        let online: Vec<usize> = (0..24).collect();
        let on = Some(online.as_slice());
        assert!(check_named_on("smt", &[11, 23], on, &cpu11()).is_ok());
        assert!(check_named_on("SMT", &[11, 23], on, &cpu11()).is_ok());
        assert!(check_named_on("ccx", &[11, 10], on, &cpu11()).is_ok());
        assert!(check_named_on("x-ccx", &[11, 8], on, &cpu11()).is_ok());
        let e = check_named_on("smt", &[11, 10], on, &cpu11()).unwrap_err();
        assert!(e.contains("is CCX") && e.contains("not SMT"), "{e}");
        let e = check_named_on("ccx", &[11, 5], on, &cpu11()).unwrap_err();
        assert!(e.contains("is x-CCX"), "{e}");
    }

    #[test]
    fn a_name_sysfs_cannot_judge_is_checked_for_its_cpus_alone() {
        let online: Vec<usize> = (0..24).collect();
        let on = Some(online.as_slice());
        assert!(check_named_on("x-ccd", &[11, 5], on, &cpu11()).is_ok());
        assert!(check_named_on("XX", &[11, 23], on, &cpu11()).is_ok());
        let e = check_named_on("XX", &[11, 30], on, &cpu11()).unwrap_err();
        assert!(e.contains("CPU 30"), "{e}");
        // Unreadable sysfs skips the checks it would need.
        assert!(check_named_on("smt", &[11, 10], None, &None).is_ok());
    }

    #[test]
    fn parse_plain_list() {
        assert_eq!(parse_cpus("0,1,2").unwrap(), vec![0, 1, 2]);
    }

    #[test]
    fn parse_range() {
        assert_eq!(parse_cpus("0-3").unwrap(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn parse_mixed() {
        assert_eq!(parse_cpus("0,3-5,7").unwrap(), vec![0, 3, 4, 5, 7]);
    }

    #[test]
    fn parse_duplicates_preserved() {
        assert_eq!(parse_cpus("0,0,0").unwrap(), vec![0, 0, 0]);
    }

    #[test]
    fn parse_empty_string_ok() {
        assert_eq!(parse_cpus("").unwrap(), Vec::<usize>::new());
    }

    #[test]
    fn parse_reverse_range_errs() {
        assert!(parse_cpus("5-3").is_err());
    }

    #[test]
    fn parse_garbage_errs() {
        assert!(parse_cpus("abc").is_err());
        assert!(parse_cpus("1-x").is_err());
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
        super::print_cpu_id("after pin to a");
        let cpu_a = unsafe { libc::sched_getcpu() } as usize;
        assert_eq!(cpu_a, a);

        pin_current(Some(b));
        super::print_cpu_id("after pin to b");
        let cpu_b = unsafe { libc::sched_getcpu() } as usize;
        assert_eq!(cpu_b, b);
    }
}
