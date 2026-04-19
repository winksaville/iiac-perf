//! Free-form measurement probe: a named, single-writer histogram
//! of hardware tick-counter deltas.
//!
//! Same shape as [`crate::probe::Probe`], but the caller records
//! tick deltas (`ticks::read_ticks() − ticks::read_ticks()`)
//! rather than nanoseconds. Skipping the tick→ns conversion at
//! record time trims a mul-shift from the hot path; conversion
//! to nanoseconds, if desired, is deferred to the report phase
//! using [`crate::ticks::ticks_per_ns`].

use hdrhistogram::Histogram;

use crate::harness::{fmt_commas, fmt_commas_f64};
use crate::ticks;

const BOUNDARY_PCTS: &[f64] = &[
    0.0, 0.01, 0.10, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70, 0.80, 0.90, 0.99, 1.0,
];
const BOUNDARY_NAMES: &[&str] = &[
    "min", "p1", "p10", "p20", "p30", "p40", "p50", "p60", "p70", "p80", "p90", "p99", "max",
];

/// Opaque handle returned by [`TProbe::start`], consumed by
/// [`TProbe::end`]. Carries the caller-supplied `site_id` and
/// the start-time tick reading; no probe-internal allocation
/// happens at `start` time.
///
/// `#[must_use]` — dropping the id without passing it to
/// [`TProbe::end`] leaks the scope (no record is appended).
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct TProbeRecId {
    site_id: u64,
    start_tsc: u64,
}

/// A complete scope record: `(site_id, start_tsc, end_tsc)`.
/// Appended at [`TProbe::end`] time; the record buffer only
/// ever holds complete records. Delta and histogram ingestion
/// are deferred to [`TProbe::report`].
#[allow(dead_code)] // consumed by report ingestion in dev3.
#[derive(Clone, Copy, Debug)]
struct Record {
    site_id: u64,
    start_tsc: u64,
    end_tsc: u64,
}

/// A named, single-writer histogram of hardware tick-counter
/// deltas plus a scope-record buffer. Not `Sync`; cross-thread
/// *sharing* is out of scope. `Send` so probes can be moved
/// between threads (e.g. returned via a `JoinHandle<TProbe>`
/// on shutdown).
pub struct TProbe {
    name: String,
    hist: Histogram<u64>,
    #[allow(dead_code)] // drained by report ingestion in dev3.
    records: Vec<Record>,
}

impl TProbe {
    /// Create an empty probe. Histogram upper bound is 1e12
    /// ticks (~250 s at 4 GHz, ~100 s at 10 GHz), 3 significant
    /// figures.
    ///
    /// Exits the process (code 1) if the hardware tick counter
    /// isn't usable — see [`crate::ticks::require_ok`].
    pub fn new(name: &str) -> Self {
        ticks::require_ok();
        // Trigger calibration eagerly so the first report() doesn't
        // pay for it.
        let _ = ticks::ticks_per_ns();
        Self {
            name: name.to_string(),
            hist: Histogram::<u64>::new_with_bounds(1, 1_000_000_000_000, 3).unwrap(),
            records: Vec::new(),
        }
    }

    /// Record a single sample, in tick-counter deltas. Values
    /// of 0 are clamped to 1 since the histogram's lower bound
    /// is 1; back-to-back tick reads can produce 0 on fast cores.
    pub fn record(&mut self, ticks: u64) {
        self.hist.record(ticks.max(1)).unwrap();
    }

    /// Begin a scope. Reads the hardware tick counter and
    /// returns an opaque [`TProbeRecId`] carrying `(site_id,
    /// start_tsc)`. The id must eventually be passed to
    /// [`TProbe::end`]; a dropped id leaves no record.
    #[allow(dead_code)] // first non-test caller lands in dev4.
    #[inline]
    pub fn start(&mut self, site_id: u64) -> TProbeRecId {
        TProbeRecId {
            site_id,
            start_tsc: ticks::read_ticks(),
        }
    }

    /// End the scope started by [`TProbe::start`]. Reads the
    /// hardware tick counter and appends a complete record
    /// `(site_id, start_tsc, end_tsc)` to the probe's record
    /// buffer. Delta and histogram ingestion are deferred to
    /// [`TProbe::report`].
    #[allow(dead_code)] // first non-test caller lands in dev4.
    #[inline]
    pub fn end(&mut self, tpri: TProbeRecId) {
        let end_tsc = ticks::read_ticks();
        self.records.push(Record {
            site_id: tpri.site_id,
            start_tsc: tpri.start_tsc,
            end_tsc,
        });
    }

    /// Render a band-table report for this probe. `as_ticks`
    /// controls the display unit: `false` converts stored tick
    /// deltas to nanoseconds (default for the CLI); `true` shows
    /// raw ticks (`-t`/`--ticks`).
    pub fn report(&self, as_ticks: bool) {
        let sample_count = self.hist.len();
        println!(
            "  tprobe: {} [count={}]",
            self.name,
            fmt_commas(sample_count)
        );
        if sample_count == 0 {
            println!();
            return;
        }

        let unit = if as_ticks { "tk" } else { "ns" };
        let tpn = ticks::ticks_per_ns();
        let conv = |v: u64| -> f64 { if as_ticks { v as f64 } else { v as f64 / tpn } };
        let conv_f = |v: f64| -> f64 { if as_ticks { v } else { v / tpn } };

        let n_bands = BOUNDARY_PCTS.len() - 1;
        let mut band_first = vec![u64::MAX; n_bands];
        let mut band_last = vec![0u64; n_bands];
        let mut band_count = vec![0u64; n_bands];
        let mut band_sum = vec![0u128; n_bands];

        let mut cumulative = 0u64;
        for iv in self.hist.iter_recorded() {
            let value = iv.value_iterated_to();
            let count = iv.count_at_value();
            let mid_rank = (cumulative as f64 + count as f64 / 2.0) / sample_count as f64;
            let idx = BOUNDARY_PCTS[1..]
                .iter()
                .position(|&b| mid_rank < b)
                .unwrap_or(n_bands - 1);
            band_first[idx] = band_first[idx].min(value);
            band_last[idx] = band_last[idx].max(value);
            band_count[idx] += count;
            band_sum[idx] += value as u128 * count as u128;
            cumulative += count;
        }

        struct BandRow {
            label: String,
            first: String,
            last: String,
            range: String,
            count: String,
            mean: String,
        }

        let mut rows: Vec<BandRow> = Vec::new();
        for i in 0..n_bands {
            if band_count[i] == 0 {
                continue;
            }
            let mean_val = band_sum[i] as f64 / band_count[i] as f64;
            let range_raw = band_last[i] - band_first[i] + 1;
            rows.push(BandRow {
                label: format!("{}-{}", BOUNDARY_NAMES[i], BOUNDARY_NAMES[i + 1]),
                first: fmt_commas_f64(conv(band_first[i]), 0),
                last: fmt_commas_f64(conv(band_last[i]), 0),
                range: fmt_commas_f64(conv(range_raw), 0),
                count: fmt_commas(band_count[i]),
                mean: fmt_commas_f64(conv_f(mean_val), 0),
            });
        }

        let label_w = rows
            .iter()
            .map(|r| r.label.len())
            .max()
            .unwrap_or(0)
            .max("stdev min-p99".len());
        let first_w = rows.iter().map(|r| r.first.len()).max().unwrap_or(0);
        let last_w = rows.iter().map(|r| r.last.len()).max().unwrap_or(0);
        let range_w = rows.iter().map(|r| r.range.len()).max().unwrap_or(0);
        let count_w = rows.iter().map(|r| r.count.len()).max().unwrap_or(0);
        let mean_w = rows.iter().map(|r| r.mean.len()).max().unwrap_or(0);

        const INDENT: &str = "    ";
        const GAP: &str = "    ";

        let first_col = INDENT.len() + label_w + 1 + first_w;
        let unit_len = 1 + unit.len(); // " ns" or " tk"
        let last_gap = unit_len + GAP.len() + last_w;
        let range_gap = unit_len + GAP.len() + range_w;
        let count_gap = unit_len + GAP.len() + count_w;
        let mean_gap = GAP.len() + mean_w;
        println!(
            "{:>first_col$}{:>last_gap$}{:>range_gap$}{:>count_gap$}{:>mean_gap$}",
            "first", "last", "range", "count", "mean",
        );

        for r in &rows {
            println!(
                "{INDENT}{:<label_w$} {:>first_w$} {unit}{GAP}{:>last_w$} {unit}{GAP}{:>range_w$} {unit}{GAP}{:>count_w$}{GAP}{:>mean_w$} {unit}",
                r.label, r.first, r.last, r.range, r.count, r.mean,
            );
        }

        let hist_mean = self.hist.mean();
        let skip = first_w
            + unit_len
            + GAP.len()
            + last_w
            + unit_len
            + GAP.len()
            + range_w
            + unit_len
            + GAP.len()
            + count_w;
        println!(
            "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} {unit}",
            "mean",
            "",
            fmt_commas_f64(conv_f(hist_mean), 0),
        );
        println!(
            "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} {unit}",
            "stdev",
            "",
            fmt_commas_f64(conv_f(self.hist.stdev()), 0),
        );

        let trim_count: u64 = band_count[..n_bands - 1].iter().sum();
        if trim_count > 0 {
            let trim_sum: u128 = band_sum[..n_bands - 1].iter().sum();
            let trim_mean = trim_sum as f64 / trim_count as f64;

            let mut trim_var_sum = 0.0f64;
            let mut trim_var_count = 0u64;
            let mut cum = 0u64;
            for iv in self.hist.iter_recorded() {
                let value = iv.value_iterated_to();
                let count = iv.count_at_value();
                let mid_rank = (cum as f64 + count as f64 / 2.0) / sample_count as f64;
                let idx = BOUNDARY_PCTS[1..]
                    .iter()
                    .position(|&b| mid_rank < b)
                    .unwrap_or(n_bands - 1);
                if idx < n_bands - 1 {
                    let diff = value as f64 - trim_mean;
                    trim_var_sum += diff * diff * count as f64;
                    trim_var_count += count;
                }
                cum += count;
            }
            let trim_stdev = if trim_var_count > 1 {
                (trim_var_sum / trim_var_count as f64).sqrt()
            } else {
                0.0
            };

            println!(
                "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} {unit}",
                "mean min-p99",
                "",
                fmt_commas_f64(conv_f(trim_mean), 0),
            );
            println!(
                "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} {unit}",
                "stdev min-p99",
                "",
                fmt_commas_f64(conv_f(trim_stdev), 0),
            );
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_end_appends_one_record() {
        let mut p = TProbe::new("t");
        let id = p.start(42);
        p.end(id);
        assert_eq!(p.records.len(), 1);
        let r = &p.records[0];
        assert_eq!(r.site_id, 42);
        assert!(r.end_tsc >= r.start_tsc);
    }

    #[test]
    fn start_end_preserves_start_tsc() {
        let mut p = TProbe::new("t");
        let id = p.start(7);
        let saved_start = id.start_tsc;
        p.end(id);
        let r = &p.records[0];
        assert_eq!(r.site_id, 7);
        assert_eq!(r.start_tsc, saved_start);
    }

    #[test]
    fn start_end_interleaved_non_stack() {
        let mut p = TProbe::new("t");
        let a = p.start(1);
        let b = p.start(2);
        p.end(a);
        p.end(b);
        assert_eq!(p.records.len(), 2);
        assert_eq!(p.records[0].site_id, 1);
        assert_eq!(p.records[1].site_id, 2);
    }

    #[test]
    fn record_and_start_end_are_independent() {
        let mut p = TProbe::new("t");
        let id = p.start(1);
        p.end(id);
        assert_eq!(p.hist.len(), 0);
        assert_eq!(p.records.len(), 1);

        p.record(100);
        assert_eq!(p.hist.len(), 1);
        assert_eq!(p.records.len(), 1);
    }
}
