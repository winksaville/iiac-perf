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
}
