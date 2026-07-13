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

/// Trimmed-stat range label from the populated bands below the
/// `p99-max` tail (`band_count`'s last band).
///
/// - Names the first..last populated non-tail band low-to-high like
///   the rows (`min-p99`, or `p10-p99` / `min-p90` when an end band
///   is empty), so the label tracks the real extent instead of a
///   fixed `min-p99` whose `min`/`p99` edges may not be present.
/// - Empty string when no non-tail band is populated (the caller
///   guards this with `trim_count > 0`, so the label goes unused).
fn trim_range_label(band_count: &[u64]) -> String {
    let trim_end = band_count.len() - 1; // exclude the p99-max tail band
    let first = (0..trim_end).find(|&i| band_count[i] > 0);
    let last = (0..trim_end).rev().find(|&i| band_count[i] > 0);
    match (first, last) {
        (Some(f), Some(l)) => format!("{}-{}", BOUNDARY_NAMES[f], BOUNDARY_NAMES[l + 1]),
        _ => String::new(),
    }
}

/// Index of the band containing `mid_rank`, over [`BOUNDARY_PCTS`]
/// (`n_bands = BOUNDARY_PCTS.len() - 1`).
///
/// Right-closed `(lower, upper]` — a rank exactly on a boundary falls
/// in the band that boundary caps, mirroring
/// [`crate::harness::print_report`]'s convention. `mid_rank` is the
/// Hazen plotting position `(i - 0.5) / n`.
fn band_index(mid_rank: f64) -> usize {
    let n_bands = BOUNDARY_PCTS.len() - 1;
    BOUNDARY_PCTS[1..]
        .iter()
        .position(|&b| mid_rank <= b)
        .unwrap_or(n_bands - 1)
}

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
    /// separate concern. Bands are right-closed `(lower, upper]`
    /// (see [`band_index`]), matching the harness report.
    /// `decimals` (`--decimals`) applies to the computed mean and
    /// stdev columns; `first`/`last`/`range` are recorded integer
    /// ns, so they stay integer — more digits would be artifacts.
    pub fn report(&self, decimals: usize) {
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
            let idx = band_index(mid_rank);
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
                mean: fmt_commas_f64(mean_val, decimals),
            });
        }

        // Trimmed-stat range label, derived from the populated bands
        // (empty when none — then `trim_count > 0` skips the rows).
        let trim_range = trim_range_label(&band_count);
        let mean_trim_label = format!("mean {trim_range}");
        let stdev_trim_label = format!("stdev {trim_range}");

        let label_w = rows
            .iter()
            .map(|r| r.label.len())
            .max()
            .unwrap_or(0)
            .max(stdev_trim_label.len());
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
            fmt_commas_f64(hist_mean, decimals),
        );
        println!(
            "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} ns",
            "stdev",
            "",
            fmt_commas_f64(self.hist.stdev(), decimals),
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
                let idx = band_index(mid_rank);
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
                mean_trim_label,
                "",
                fmt_commas_f64(trim_mean, decimals),
            );
            println!(
                "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} ns",
                stdev_trim_label,
                "",
                fmt_commas_f64(trim_stdev, decimals),
            );
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `band_count` vec (len = n_bands) with the given band
    /// indices marked populated.
    fn counts(populated: &[usize]) -> Vec<u64> {
        let mut c = vec![0u64; BOUNDARY_PCTS.len() - 1];
        for &i in populated {
            c[i] = 1;
        }
        c
    }

    #[test]
    fn trim_range_label_spans_populated_bands() {
        // Full: min-p1 band (0) through p90-p99 band (10) -> min-p99.
        assert_eq!(trim_range_label(&counts(&[0, 5, 10])), "min-p99");
        // The p99-max tail band (11) is excluded even when populated.
        assert_eq!(trim_range_label(&counts(&[2, 10, 11])), "p10-p99");
        // Empty p90-p99 band: upper end is the last populated band.
        assert_eq!(trim_range_label(&counts(&[0, 9])), "min-p90");
        // One populated band is just that band's own low-high label.
        assert_eq!(trim_range_label(&counts(&[6])), "p50-p60");
        // No populated non-tail band yields an empty label (unused).
        assert_eq!(trim_range_label(&counts(&[])), "");
    }

    #[test]
    fn band_index_right_closed_on_boundary() {
        // Band label for a rank (probe rows are low-high ranges).
        let label = |r: f64| {
            let i = band_index(r);
            format!("{}-{}", BOUNDARY_NAMES[i], BOUNDARY_NAMES[i + 1])
        };

        // Right-closed: a rank exactly on a boundary lands in the band
        // that boundary caps (upper edge), not the next one up.
        assert_eq!(label(0.5), "p40-p50"); // single-sample mid-rank
        assert_eq!(label(0.4), "p30-p40"); // exactly the p40 boundary
        assert_eq!(label(0.99), "p90-p99"); // exactly p99 → last non-tail band

        // Strictly-interior ranks are unaffected by the closed end.
        assert_eq!(label(0.55), "p50-p60");
        assert_eq!(label(0.05), "p1-p10");
    }
}
