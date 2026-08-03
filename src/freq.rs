//! Delivered-frequency reads for the warm loop's clock gate: `cpuinfo_avg_freq` where the
//! driver provides it (amd-pstate), nothing otherwise.
//!
//! - `scaling_cur_freq` is deliberately not a fallback: some drivers report the *requested*
//!   rather than the delivered frequency, and a gate fed requested values would certify a dwell
//!   as the top. Read the honest file where it exists and fall back to timing-only everywhere
//!   else.

/// One delivered-frequency sample: which logical CPU it was read on, and the kHz value.
#[derive(Debug, Clone, Copy)]
pub struct FreqSample {
    /// Logical CPU the calling thread was on at the read.
    pub cpu: usize,
    /// Delivered frequency (kHz) as the driver reports it.
    pub khz: u64,
}

/// Sample the calling thread's current CPU and its delivered frequency; `None` when the driver
/// does not expose `cpuinfo_avg_freq` (see the module doc) or the read fails.
pub fn avg_freq() -> Option<FreqSample> {
    let cpu = current_cpu()?;
    let khz = read_khz(cpu, "cpuinfo_avg_freq")?;
    Some(FreqSample { cpu, khz })
}

/// The hardware maximum (kHz) for `cpu`, for the reported ratio; `None` when unavailable.
pub fn max_freq(cpu: usize) -> Option<u64> {
    read_khz(cpu, "cpuinfo_max_freq")
}

/// The calling thread's current logical CPU (`sched_getcpu`), `None` on syscall failure.
fn current_cpu() -> Option<usize> {
    let cpu = unsafe { libc::sched_getcpu() };
    usize::try_from(cpu).ok()
}

/// One cpufreq sysfs value (kHz) for `cpu`, `None` when the file is absent or unparsable.
fn read_khz(cpu: usize, name: &str) -> Option<u64> {
    let path = format!("/sys/devices/system/cpu/cpu{cpu}/cpufreq/{name}");
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_cpu_resolves() {
        assert!(current_cpu().is_some());
    }

    #[test]
    fn absent_file_reads_none() {
        assert_eq!(read_khz(usize::MAX, "cpuinfo_avg_freq"), None);
    }
}
