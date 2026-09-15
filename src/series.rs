//! Statistics over a series of replicate means, the one owner of the CI95 and LSC arithmetic.
//!
//! A replicate is whatever the caller treats as one independent draw: a block within a process,
//! an aggregation level's group in the resolution curve, or a whole process across runs. The
//! arithmetic is the same at every tier, so it lives here once.
//!
//! - [`weighted_mean`] pools means by their sample counts, which makes a pooled mean exact
//! - [`Series`] takes the replicates' plain mean and sample stdev, and from them the CI95
//!   half-width and the LSC against an equal-sized series of something else
//! - [`t975`] is the Student-t quantile both formulas use
//!
//! Method and worked numbers:
//! notes/design.md#comparing-implementations-least-significant-change.

/// Two-sided 95% Student-t quantile (`t(0.975, df)`), a table for df <= 30, then the
/// conservative 2.0, since the true value falls from 2.042 toward the normal 1.96.
pub(crate) fn t975(df: u64) -> f64 {
    const TABLE: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
        2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
        2.052, 2.048, 2.045, 2.042,
    ];
    match df {
        0 => f64::INFINITY,
        1..=30 => TABLE[(df - 1) as usize],
        _ => 2.0,
    }
}

/// The mean of `(mean, count)` points weighted by their counts, the exact mean of every sample
/// behind them.
///
/// - Equal counts give the plain mean, and a short point keeps only its own weight.
/// - With no counts at all it is the plain mean of the points, and an empty slice is NaN.
pub fn weighted_mean(points: &[(f64, u64)]) -> f64 {
    let total: u64 = points.iter().map(|p| p.1).sum();
    if total > 0 {
        points.iter().map(|p| p.0 * p.1 as f64).sum::<f64>() / total as f64
    } else {
        points.iter().map(|p| p.0).sum::<f64>() / points.len() as f64
    }
}

/// The 95% confidence half-width on the mean of `n` replicates with sample stdev `stdev`:
/// `t(0.975, n-1) * stdev / sqrt(n)`.
pub fn ci95(stdev: f64, n: u64) -> f64 {
    t975(n.saturating_sub(1)) * stdev / (n as f64).sqrt()
}

/// The least significant change between two series of `n` replicates each with pooled stdev
/// `stdev`: `t(0.975, 2n-2) * stdev * sqrt(2/n)`.
pub fn lsc(stdev: f64, n: u64) -> f64 {
    t975((2 * n).saturating_sub(2)) * stdev * (2.0 / n as f64).sqrt()
}

/// A series of replicate means: their count, plain mean, and sample stdev.
#[derive(Debug, Clone, Copy)]
pub struct Series {
    /// Replicates in the series.
    pub n: u64,
    /// Plain mean of the replicates, each one draw with equal weight.
    pub mean: f64,
    /// Sample standard deviation of the replicates, `n - 1` in the denominator.
    pub stdev: f64,
}

impl Series {
    /// Fit from the replicates' means. `None` below two, where no spread exists.
    pub fn of(means: &[f64]) -> Option<Series> {
        if means.len() < 2 {
            return None;
        }
        let n = means.len() as f64;
        let mean = means.iter().sum::<f64>() / n;
        let var = means.iter().map(|m| (m - mean) * (m - mean)).sum::<f64>() / (n - 1.0);
        Some(Series {
            n: means.len() as u64,
            mean,
            stdev: var.sqrt(),
        })
    }

    /// The 95% confidence half-width on [`Series::mean`].
    pub fn ci95(&self) -> f64 {
        ci95(self.stdev, self.n)
    }

    /// The least significant change against an equal-sized series of something else.
    pub fn lsc(&self) -> f64 {
        lsc(self.stdev, self.n)
    }
}

/// The share of a series trimmed from each end for the robust pair, the usual 20%: it tolerates
/// a fifth of the runs being disturbed, where a mean and its stdev tolerate none, and it costs
/// little efficiency on a clean series.
pub const TRIM_FRACTION: f64 = 0.2;

/// A series with its ends trimmed: the mean of what is kept, and the winsorized stdev of the
/// whole, which is the spread the trimmed mean's error is built from.
///
/// The pairing is Yuen's and is deliberate. Dropping the extremes is what keeps a disturbed
/// replicate out of the value, while the trimmed mean's uncertainty comes from a series where
/// those replicates are pulled in to the boundary rather than deleted, since their absence is
/// itself uncertainty. See notes/design.md#comparing-implementations-least-significant-change.
#[derive(Debug, Clone, PartialEq)]
pub struct Trimmed {
    /// Replicates in the whole series.
    pub n: u64,
    /// Replicates trimmed from each end.
    pub per_end: u64,
    /// Mean of the kept replicates.
    pub mean: f64,
    /// Sample standard deviation of the winsorized series.
    pub winsorized_stdev: f64,
    /// Positions of the trimmed replicates in the series as given, low end then high end,
    /// each 0-based.
    pub trimmed: Vec<usize>,
}

impl Trimmed {
    /// Trim [`TRIM_FRACTION`] of `means` from each end. `None` when the series is too short to
    /// trim anything, under five replicates, where the plain mean and stdev are the whole story.
    pub fn of(means: &[f64]) -> Option<Trimmed> {
        let n = means.len();
        let per_end = (TRIM_FRACTION * n as f64) as usize;
        if per_end == 0 || n - 2 * per_end < 2 {
            return None;
        }
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| means[a].total_cmp(&means[b]));
        let (lo, hi) = (means[order[per_end]], means[order[n - per_end - 1]]);
        let kept: Vec<f64> = order[per_end..n - per_end]
            .iter()
            .map(|&i| means[i])
            .collect();
        let mean = kept.iter().sum::<f64>() / kept.len() as f64;
        let winsorized: Vec<f64> = means.iter().map(|&m| m.clamp(lo, hi)).collect();
        let wmean = winsorized.iter().sum::<f64>() / n as f64;
        let var = winsorized
            .iter()
            .map(|w| (w - wmean) * (w - wmean))
            .sum::<f64>()
            / (n as f64 - 1.0);
        let mut trimmed: Vec<usize> = order[..per_end].to_vec();
        trimmed.extend_from_slice(&order[n - per_end..]);
        trimmed.sort_unstable();
        Some(Trimmed {
            n: n as u64,
            per_end: per_end as u64,
            mean,
            winsorized_stdev: var.sqrt(),
            trimmed,
        })
    }

    /// Replicates kept, `n - 2 * per_end`, which sets the degrees of freedom.
    pub fn kept(&self) -> u64 {
        self.n - 2 * self.per_end
    }

    /// The standard error of [`Trimmed::mean`]: the winsorized stdev over `(1 - 2 * trim) * sqrt(n)`,
    /// Yuen's estimate.
    fn se(&self) -> f64 {
        self.winsorized_stdev / ((1.0 - 2.0 * TRIM_FRACTION) * (self.n as f64).sqrt())
    }

    /// The 95% confidence half-width on [`Trimmed::mean`], at `kept - 1` degrees of freedom.
    pub fn ci95(&self) -> f64 {
        t975(self.kept().saturating_sub(1)) * self.se()
    }

    /// The least significant change against an equal-sized trimmed series of something else,
    /// the two-sample form of [`Trimmed::ci95`] with equally sized arms.
    pub fn lsc(&self) -> f64 {
        t975((2 * self.kept()).saturating_sub(2)) * self.se() * 2f64.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The six-run tp-pc series on r5-7600x (2026-07-13), `mean min-p99` per run, ns, from the
    /// design notes' least-significant-change section.
    const TP_PC_RUNS: [f64; 6] = [4752.5, 4852.1, 4763.7, 4834.0, 4726.6, 4863.2];

    #[test]
    fn the_design_notes_worked_lsc_reproduces() {
        let s = Series::of(&TP_PC_RUNS).expect("six runs fit");
        assert!((s.stdev - 58.0).abs() < 0.5, "stdev {}", s.stdev);
        for (n, want) in [(3, 131.0), (5, 85.0), (10, 55.0)] {
            let got = lsc(s.stdev, n);
            assert!(
                (got - want).abs() < 1.0,
                "n={n}: lsc {got}, want about {want}"
            );
        }
    }

    #[test]
    fn ci95_and_lsc_follow_their_formulas() {
        let s = Series::of(&[10.0, 12.0, 14.0]).expect("three fit");
        assert_eq!(s.n, 3);
        assert!((s.mean - 12.0).abs() < 1e-12);
        assert!((s.stdev - 2.0).abs() < 1e-12);
        assert!((s.ci95() - 4.303 * 2.0 / 3f64.sqrt()).abs() < 1e-9);
        assert!((s.lsc() - 2.776 * 2.0 * (2.0f64 / 3.0).sqrt()).abs() < 1e-9);
    }

    #[test]
    fn fewer_than_two_means_have_no_spread() {
        assert!(Series::of(&[]).is_none());
        assert!(Series::of(&[5.0]).is_none());
    }

    #[test]
    fn identical_means_have_zero_spread() {
        let s = Series::of(&[7.0, 7.0, 7.0, 7.0]).expect("four fit");
        assert_eq!(s.stdev, 0.0);
        assert_eq!(s.lsc(), 0.0);
    }

    #[test]
    fn weighted_mean_weights_by_count_and_falls_back_to_plain() {
        assert!((weighted_mean(&[(20.0, 900), (30.0, 100)]) - 21.0).abs() < 1e-12);
        assert!((weighted_mean(&[(20.0, 0), (30.0, 0)]) - 25.0).abs() < 1e-12);
        assert!(weighted_mean(&[]).is_nan());
    }

    #[test]
    fn t975_covers_the_table_and_its_tail() {
        assert_eq!(t975(0), f64::INFINITY);
        assert_eq!(t975(1), 12.706);
        assert_eq!(t975(30), 2.042);
        assert_eq!(t975(31), 2.0);
    }

    /// The 3900X's twenty `zcr-mpsc-v1-2t` run means, 2026-09-15, clock free on a busy desktop:
    /// four runs were disturbed, the last three badly.
    const BUSY_RUNS: [f64; 20] = [
        385.0, 382.7, 378.6, 387.0, 377.4, 382.7, 377.8, 385.5, 381.0, 377.6, 391.2, 381.5, 393.2,
        386.8, 384.8, 411.3, 385.2, 540.6, 772.0, 766.3,
    ];

    #[test]
    fn trimming_answers_where_the_plain_mean_cannot() {
        let plain = Series::of(&BUSY_RUNS).expect("twenty runs");
        // The plain pair is dominated by the four disturbed runs.
        assert!((plain.mean - 431.41).abs() < 0.01, "mean {}", plain.mean);
        assert!(plain.ci95() > 50.0, "ci95 {}", plain.ci95());
        let t = Trimmed::of(&BUSY_RUNS).expect("twenty runs trim");
        assert_eq!((t.n, t.per_end, t.kept()), (20, 4, 12));
        assert!((t.mean - 385.55).abs() < 0.01, "trimmed mean {}", t.mean);
        assert!(
            (t.winsorized_stdev - 4.908).abs() < 0.01,
            "winsorized stdev {}",
            t.winsorized_stdev
        );
        assert!((t.ci95() - 4.03).abs() < 0.01, "ci95 {}", t.ci95());
        assert!((t.lsc() - 5.37).abs() < 0.01, "lsc {}", t.lsc());
        // The four disturbed runs are trimmed from the high end, and four fast ones from the low.
        assert!(t.trimmed.contains(&17) && t.trimmed.contains(&18) && t.trimmed.contains(&19));
        assert_eq!(t.trimmed.len(), 8);
    }

    #[test]
    fn a_trim_keeps_the_middle_and_winsorizes_the_ends() {
        let x = [10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 100.0];
        let t = Trimmed::of(&x).expect("ten values trim");
        assert_eq!((t.per_end, t.kept()), (2, 6));
        // The kept middle is 12..17, and the winsorized series clamps to 12 and 17.
        assert!((t.mean - 14.5).abs() < 1e-12);
        assert!((t.winsorized_stdev - 2.173_067).abs() < 1e-6);
        assert_eq!(t.trimmed, vec![0, 1, 8, 9]);
    }

    #[test]
    fn a_short_series_trims_nothing() {
        assert!(Trimmed::of(&[]).is_none());
        assert!(Trimmed::of(&[1.0, 2.0, 3.0, 4.0]).is_none());
        // Five replicates trim one from each end and keep three.
        let t = Trimmed::of(&[1.0, 2.0, 3.0, 4.0, 5.0]).expect("five trim");
        assert_eq!((t.per_end, t.kept()), (1, 3));
        assert!((t.mean - 3.0).abs() < 1e-12);
    }
}
