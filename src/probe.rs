//! Free-form measurement probe: a named, single-writer histogram
//! of sample durations in nanoseconds.
//!
//! The caller captures endpoints with any time source (typically
//! `minstant::Instant::now()`) and feeds the elapsed delta to
//! [`Probe::record`]. At report time [`Probe::report`] renders a
//! band-table summary under the enclosing bench's report, using
//! the same percentile boundaries so columns line up visually.

use hdrhistogram::Histogram;

use crate::harness::{fmt_commas, fmt_commas_f64};

const BOUNDARY_PCTS: &[f64] = &[
    0.0, 0.01, 0.10, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70, 0.80, 0.90, 0.99, 1.0,
];
const BOUNDARY_NAMES: &[&str] = &[
    "min", "p1", "p10", "p20", "p30", "p40", "p50", "p60", "p70", "p80", "p90", "p99", "max",
];

/// A named, single-writer nanosecond histogram. Not `Sync`;
/// cross-thread *sharing* is out of scope. `Send` so probes can
/// be moved between threads (e.g. returned via a
/// `JoinHandle<Probe>` on shutdown).
pub struct Probe {
    name: String,
    hist: Histogram<u64>,
}

impl Probe {
    /// Create an empty probe. Histogram bounds (1 ns — 60 s,
    /// 3 significant figures) mirror the harness.
    ///
    /// Exits the process (code 1) if the hardware tick counter
    /// isn't usable — see [`crate::ticks::require_ok`].
    pub fn new(name: &str) -> Self {
        crate::ticks::require_ok();
        Self {
            name: name.to_string(),
            hist: Histogram::<u64>::new_with_bounds(1, 60_000_000_000, 3).unwrap(),
        }
    }

    /// Record a single sample. Values of 0 are clamped to 1 since
    /// the histogram's lower bound is 1 ns; sub-ns deltas from
    /// coarse timers round up rather than panic.
    pub fn record(&mut self, ns: u64) {
        self.hist.record(ns.max(1)).unwrap();
    }

    /// Render a band-table report for this probe, indented one
    /// level deeper than the enclosing bench's report. No
    /// `adjusted` column — probe overhead subtraction is a
    /// separate concern.
    pub fn report(&self) {
        let sample_count = self.hist.len();
        println!(
            "  probe: {} [count={}]",
            self.name,
            fmt_commas(sample_count)
        );
        if sample_count == 0 {
            println!();
            return;
        }

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
            rows.push(BandRow {
                label: format!("{}-{}", BOUNDARY_NAMES[i], BOUNDARY_NAMES[i + 1]),
                first: fmt_commas(band_first[i]),
                last: fmt_commas(band_last[i]),
                range: fmt_commas(band_last[i] - band_first[i] + 1),
                count: fmt_commas(band_count[i]),
                mean: fmt_commas_f64(mean_val, 0),
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
        let last_gap = " ns".len() + GAP.len() + last_w;
        let range_gap = " ns".len() + GAP.len() + range_w;
        let count_gap = " ns".len() + GAP.len() + count_w;
        let mean_gap = GAP.len() + mean_w;
        println!(
            "{:>first_col$}{:>last_gap$}{:>range_gap$}{:>count_gap$}{:>mean_gap$}",
            "first", "last", "range", "count", "mean",
        );

        for r in &rows {
            println!(
                "{INDENT}{:<label_w$} {:>first_w$} ns{GAP}{:>last_w$} ns{GAP}{:>range_w$} ns{GAP}{:>count_w$}{GAP}{:>mean_w$} ns",
                r.label, r.first, r.last, r.range, r.count, r.mean,
            );
        }

        let hist_mean = self.hist.mean();
        let skip = first_w
            + " ns".len()
            + GAP.len()
            + last_w
            + " ns".len()
            + GAP.len()
            + range_w
            + " ns".len()
            + GAP.len()
            + count_w;
        println!(
            "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} ns",
            "mean",
            "",
            fmt_commas_f64(hist_mean, 0),
        );
        println!(
            "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} ns",
            "stdev",
            "",
            fmt_commas_f64(self.hist.stdev(), 0),
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
                "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} ns",
                "mean min-p99",
                "",
                fmt_commas_f64(trim_mean, 0),
            );
            println!(
                "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} ns",
                "stdev min-p99",
                "",
                fmt_commas_f64(trim_stdev, 0),
            );
        }
        println!();
    }
}
