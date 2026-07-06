//! Report band boundaries and label styles: the single source of
//! truth for the min/z/p/n/max ladder that
//! [`crate::harness::print_report`] renders, documented by the
//! README's boundary-ladder table (pinned by this module's tests).
//!
//! - Familiar deciles in the body; nines/zeros tails generated
//!   from [`Z_DEPTH`]/[`N_DEPTH`]. Fractions and names come from
//!   one structural description, so the label styles can never
//!   drift apart and deepening a tail is a one-constant change.
//! - `band_table.rs` and `probe.rs` still carry their own older
//!   percentile ladders; unifying them onto this module is part
//!   of the probe-based-harness conversion todo.

/// Fast-tail depth: the lowest tail boundary is `z{Z_DEPTH}`.
const Z_DEPTH: i32 = 4;
/// Slow-tail depth: the highest tail boundary is `n{N_DEPTH}`.
/// Deeper than [`Z_DEPTH`] because a latency distribution is
/// floored below (nothing beats the fast path) and open above.
const N_DEPTH: i32 = 10;

/// Band-label style for the report's histogram rows, selected by
/// the `--band-labels` CLI flag.
///
/// - `Zpn` — nines/zeros + decile names (`z3`, `p50`, `n4`).
/// - `Frac` — literal boundary fractions with `_` grouping
///   (`0.001`, `0.50`, `0.999_9`).
/// - `Both` — zpn and fraction side by side (`n4  0.999_9`);
///   the default: the juxtaposition teaches the zpn vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum BandLabels {
    Zpn,
    Frac,
    Both,
}

impl BandLabels {
    /// Lowercase name used on the CLI and in the report header's
    /// `labels=` metadata.
    pub fn as_str(self) -> &'static str {
        match self {
            BandLabels::Zpn => "zpn",
            BandLabels::Frac => "frac",
            BandLabels::Both => "both",
        }
    }

    /// Trimmed-stat row range label matching the style, so `n2`
    /// never appears in a frac-only table.
    pub fn trim_label(self) -> &'static str {
        match self {
            BandLabels::Frac => "min..0.99",
            BandLabels::Zpn | BandLabels::Both => "min..n2",
        }
    }
}

/// One report band boundary: its CDF fraction plus its rendered
/// name in each label style.
pub struct Boundary {
    /// Fraction of samples at or below the boundary (CDF).
    pub pct: f64,
    /// nines/zeros / decile name (`z3`, `p50`, `n4`).
    pub zpn: String,
    /// Literal-fraction name, `_`-grouped (`0.001`, `0.999_9`).
    pub frac: String,
}

impl Boundary {
    /// Row label for this boundary in the given style. min/max
    /// carry no fraction, so `both` prints them bare; elsewhere
    /// `both` pads zpn to its 3-char max so the fraction column
    /// aligns.
    pub fn label(&self, style: BandLabels) -> String {
        match style {
            BandLabels::Zpn => self.zpn.clone(),
            BandLabels::Frac => self.frac.clone(),
            BandLabels::Both if self.zpn == self.frac => self.zpn.clone(),
            BandLabels::Both => format!("{:<3} {}", self.zpn, self.frac),
        }
    }
}

/// Group `digits` in threes with `_` so deep nines stay
/// countable at a glance, e.g. `"99999"` → `"999_99"`.
fn group3(digits: &str) -> String {
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push('_');
        }
        out.push(c);
    }
    out
}

/// The report's band boundaries, generated from
/// [`Z_DEPTH`]/[`N_DEPTH`]: familiar deciles in the body,
/// nines/zeros notation in both tails.
///
/// - `nK`/`zK` mark the boundary with a fraction 10^-K of
///   samples above (n) or below (z): n2 ≡ p99, n3 ≡ p99.9, …
///   and z2 ≡ p1, z3 ≡ p0.1, z4. "K nines" is standard
///   engineering shorthand for proportions near one
///   (nines = -log10(1-x)); zK is this project's mirror of it
///   for the fast tail.
/// - Deep bands populate only when the run has enough samples;
///   empty bands are skipped in the output.
pub fn boundaries() -> Vec<Boundary> {
    let mut v = vec![Boundary {
        pct: 0.0,
        zpn: "min".to_string(),
        frac: "min".to_string(),
    }];
    for k in (2..=Z_DEPTH).rev() {
        v.push(Boundary {
            pct: 10f64.powi(-k),
            zpn: format!("z{k}"),
            frac: format!("0.{}", group3(&("0".repeat(k as usize - 1) + "1"))),
        });
    }
    for d in 1..=9 {
        v.push(Boundary {
            pct: d as f64 / 10.0,
            zpn: format!("p{d}0"),
            frac: format!("0.{d}0"),
        });
    }
    for k in 2..=N_DEPTH {
        v.push(Boundary {
            pct: 1.0 - 10f64.powi(-k),
            zpn: format!("n{k}"),
            frac: format!("0.{}", group3(&"9".repeat(k as usize))),
        });
    }
    v.push(Boundary {
        pct: 1.0,
        zpn: "max".to_string(),
        frac: "max".to_string(),
    });
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_boundaries_match_documented_ladder() {
        let b = boundaries();
        let zpn: Vec<&str> = b.iter().map(|x| x.zpn.as_str()).collect();
        assert_eq!(
            zpn,
            [
                "min", "z4", "z3", "z2", "p10", "p20", "p30", "p40", "p50", "p60", "p70", "p80",
                "p90", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9", "n10", "max",
            ]
        );
        let frac: Vec<&str> = b.iter().map(|x| x.frac.as_str()).collect();
        assert_eq!(
            frac,
            [
                "min",
                "0.000_1",
                "0.001",
                "0.01",
                "0.10",
                "0.20",
                "0.30",
                "0.40",
                "0.50",
                "0.60",
                "0.70",
                "0.80",
                "0.90",
                "0.99",
                "0.999",
                "0.999_9",
                "0.999_99",
                "0.999_999",
                "0.999_999_9",
                "0.999_999_99",
                "0.999_999_999",
                "0.999_999_999_9",
                "max",
            ]
        );
        // Endpoints exact; interior strictly increasing; the n2
        // trim anchor lands within a ulp of 0.99.
        assert_eq!(b[0].pct, 0.0);
        assert_eq!(b[b.len() - 1].pct, 1.0);
        assert!(b.windows(2).all(|w| w[0].pct < w[1].pct));
        let n2 = &b[13];
        assert!((n2.pct - 0.99).abs() < 1e-12, "n2 pct: {}", n2.pct);
    }
}
