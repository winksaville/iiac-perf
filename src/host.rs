//! The host block of a record: what box produced it, read once per process without root.
//!
//! A record used to name its box by hostname alone, so a file read on another machine could not
//! say what CPU, topology, memory, kernel, or toolchain produced it. The block rides in every
//! record, as the policy fields do, so a line stands alone.
//!
//! - **Absent is null**: a field a box may not expose is an `Option`, never a default.
//! - **Units in the names**, as the rest of the record does.
//! - **Caches as the kernel says them**: one entry per sysfs cache index, so L1 appears twice
//!   (Data and Instruction) and the array's length is the depth. Each entry's sharing list is
//!   the topology: L1's names the SMT siblings, L3's the CCX.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Where the box produced a record: hostname, CPU, memory, caches, kernel, and toolchain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Host {
    /// The hostname, `unknown-host` when the syscall fails.
    pub name: String,
    /// `model name` from `/proc/cpuinfo`.
    pub cpu_model: Option<String>,
    /// `MemTotal` from `/proc/meminfo`, what the kernel has, less than what is installed.
    pub ram_bytes: Option<u64>,
    /// L1D's `coherency_line_size`, the false-sharing constant the rings are sized by.
    pub cache_line_bytes: Option<u32>,
    /// One entry per sysfs cache index of CPU 0, in index order.
    pub caches: Vec<Cache>,
    /// The kernel release from `uname`.
    pub kernel: Option<String>,
    /// The compiler that built this binary, baked in by `build.rs`.
    pub rustc: String,
}

/// One cache as sysfs describes it under `/sys/devices/system/cpu/cpu0/cache/indexN/`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cache {
    /// `level`: 1, 2, 3.
    pub level: u32,
    /// `type`: `Data`, `Instruction`, or `Unified`.
    #[serde(rename = "type")]
    pub kind: String,
    /// `size`, converted from its `K` / `M` spelling.
    pub size_bytes: u64,
    /// `shared_cpu_list`, verbatim: the CPUs this cache serves.
    pub shared_cpus: String,
}

/// Read the host block from the running box.
pub fn probe() -> Host {
    let caches = read_caches(Path::new("/sys/devices/system/cpu/cpu0/cache"));
    Host {
        name: hostname(),
        cpu_model: cpu_model(&read("/proc/cpuinfo")),
        ram_bytes: mem_total_bytes(&read("/proc/meminfo")),
        cache_line_bytes: cache_line_bytes(
            Path::new("/sys/devices/system/cpu/cpu0/cache"),
            &caches,
        ),
        caches,
        kernel: kernel_release(),
        rustc: env!("IIAC_PERF_RUSTC").to_string(),
    }
}

/// A file's text, empty when it cannot be read: every parser below treats empty as absent.
fn read(path: impl AsRef<Path>) -> String {
    // OK: an unreadable file is the absent case, and empty text is what every parser here
    // maps to None, so the default is the meaning, not a hidden failure.
    std::fs::read_to_string(path).unwrap_or_default()
}

/// The first `model name` line's value.
fn cpu_model(cpuinfo: &str) -> Option<String> {
    cpuinfo
        .lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// `MemTotal:  32767688 kB` as bytes.
fn mem_total_bytes(meminfo: &str) -> Option<u64> {
    let line = meminfo.lines().find(|l| l.starts_with("MemTotal:"))?;
    let mut parts = line.split_whitespace();
    parts.next();
    let kb: u64 = parts.next()?.parse().ok()?;
    Some(kb * 1024)
}

/// Every `indexN` under the cache directory that reads whole, in index order.
fn read_caches(dir: &Path) -> Vec<Cache> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("index"))
        .collect();
    names.sort_by_key(|n| n[5..].parse::<u32>().ok());
    names
        .iter()
        .filter_map(|n| read_cache(&dir.join(n)))
        .collect()
}

/// One cache entry, `None` when any of its four files is missing or malformed.
fn read_cache(index: &Path) -> Option<Cache> {
    let level = read(index.join("level")).trim().parse().ok()?;
    let kind = read(index.join("type")).trim().to_string();
    if kind.is_empty() {
        return None;
    }
    let size_bytes = parse_size(read(index.join("size")).trim())?;
    let shared_cpus = read(index.join("shared_cpu_list")).trim().to_string();
    if shared_cpus.is_empty() {
        return None;
    }
    Some(Cache {
        level,
        kind,
        size_bytes,
        shared_cpus,
    })
}

/// sysfs cache sizes: `32K`, `16384K`, `1M`, or a bare byte count.
fn parse_size(s: &str) -> Option<u64> {
    let (digits, mult) = match s.as_bytes().last()? {
        b'K' => (&s[..s.len() - 1], 1024),
        b'M' => (&s[..s.len() - 1], 1024 * 1024),
        b'G' => (&s[..s.len() - 1], 1024 * 1024 * 1024),
        _ => (s, 1),
    };
    digits.parse::<u64>().ok().map(|n| n * mult)
}

/// The L1 data cache's line size, or the first index that reports one when no entry names L1
/// Data, or nothing when the directory is absent.
fn cache_line_bytes(dir: &Path, caches: &[Cache]) -> Option<u32> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return None;
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("index"))
        .collect();
    names.sort_by_key(|n| n[5..].parse::<u32>().ok());
    let l1d = caches
        .iter()
        .position(|c| c.level == 1 && c.kind == "Data")
        .and_then(|i| names.get(i));
    let pick = l1d.or_else(|| names.first())?;
    read(dir.join(pick).join("coherency_line_size"))
        .trim()
        .parse()
        .ok()
}

/// The kernel release via `uname`, `None` when the syscall fails.
fn kernel_release() -> Option<String> {
    let mut uts: libc::utsname = unsafe { std::mem::zeroed() };
    // SAFETY: uname writes only into `uts` and returns non-zero on failure.
    let rc = unsafe { libc::uname(&mut uts) };
    if rc != 0 {
        return None;
    }
    Some(c_field(&uts.release))
}

/// The writing box's hostname via `gethostname`, `unknown-host` when the syscall fails.
fn hostname() -> String {
    let mut buf = [0u8; 256];
    // SAFETY: gethostname writes at most buf.len() bytes into buf.
    let rc = unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) };
    if rc != 0 {
        return "unknown-host".to_string();
    }
    let end = match buf.iter().position(|&b| b == 0) {
        Some(e) => e,
        // A name that filled the buffer arrives untruncated-looking either way: take it whole.
        None => buf.len(),
    };
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

/// A NUL-terminated C char array as a `String`, lossily.
fn c_field(field: &[libc::c_char]) -> String {
    let bytes: Vec<u8> = field
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpuinfo_and_meminfo_parse_their_lines() {
        let cpuinfo = "processor\t: 0\nmodel name\t: AMD Ryzen 9 3900X 12-Core Processor\n";
        assert_eq!(
            cpu_model(cpuinfo).as_deref(),
            Some("AMD Ryzen 9 3900X 12-Core Processor")
        );
        assert_eq!(cpu_model(""), None);
        assert_eq!(
            mem_total_bytes("MemTotal:       32767688 kB\nMemFree: 1 kB\n"),
            Some(32_767_688 * 1024)
        );
        assert_eq!(mem_total_bytes("MemFree: 1 kB\n"), None);
    }

    #[test]
    fn sizes_convert_their_suffix() {
        assert_eq!(parse_size("32K"), Some(32 * 1024));
        assert_eq!(parse_size("16384K"), Some(16384 * 1024));
        assert_eq!(parse_size("1M"), Some(1024 * 1024));
        assert_eq!(parse_size("64"), Some(64));
        assert_eq!(parse_size(""), None);
        assert_eq!(parse_size("K"), None);
    }

    #[test]
    fn a_missing_cache_dir_is_empty_not_a_panic() {
        let dir = Path::new("/no/such/cache/dir");
        assert!(read_caches(dir).is_empty());
        assert_eq!(cache_line_bytes(dir, &[]), None);
    }

    #[test]
    fn the_running_box_probes_without_panicking() {
        let host = probe();
        assert!(!host.name.is_empty());
        assert!(host.rustc.starts_with("rustc") || host.rustc == "unknown");
        for c in &host.caches {
            assert!(c.level >= 1);
            assert!(c.size_bytes > 0);
        }
    }
}
