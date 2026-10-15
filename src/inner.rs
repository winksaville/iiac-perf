//! How many steps a sample times: one count, or a span each sample draws its count from.
//!
//! A fixed count repeats one burst length for a whole run, so a bench whose cost depends on the
//! length, or on its repetition, reads differently at each count. A span mixes the lengths
//! inside one run, and the run's totals by length ([`LengthTotals`]) are what a regression of
//! sample time on length reads.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// The widest span, in lengths, a run keeps totals for.
const MAX_SPAN_LENGTHS: u64 = 4096;

/// The steps a sample times: `lo` to `hi` inclusive, one count when they agree.
///
/// Written `7` or `1-16`, on the command line, in a config file, and in a child's spec, where a
/// single count may also be a bare number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Repr", into = "Repr")]
pub struct InnerSpan {
    lo: u64,
    hi: u64,
}

impl InnerSpan {
    /// The span of one count.
    pub fn fixed(n: u64) -> Self {
        InnerSpan { lo: n, hi: n }
    }

    /// A span from `lo` to `hi` inclusive, refused when it is empty, starts at zero, or is
    /// wider than a run keeps totals for.
    pub fn new(lo: u64, hi: u64) -> Result<Self, String> {
        if lo == 0 {
            return Err("an inner count is at least 1".to_string());
        }
        if hi < lo {
            return Err(format!("an inner span runs low to high, not {lo}-{hi}"));
        }
        if hi - lo >= MAX_SPAN_LENGTHS {
            return Err(format!(
                "an inner span holds at most {MAX_SPAN_LENGTHS} lengths"
            ));
        }
        Ok(InnerSpan { lo, hi })
    }

    /// The shortest sample, in steps.
    pub fn lo(self) -> u64 {
        self.lo
    }

    /// The longest sample, in steps.
    pub fn hi(self) -> u64 {
        self.hi
    }

    /// Whether every sample is one length.
    pub fn is_fixed(self) -> bool {
        self.lo == self.hi
    }

    /// How many lengths the span holds.
    pub fn lengths(self) -> u64 {
        self.hi - self.lo + 1
    }

    /// The mean length of a sample drawn uniformly from the span.
    pub fn mean(self) -> f64 {
        (self.lo + self.hi) as f64 / 2.0
    }

    /// The whole count nearest the mean length, what a record's `inner` holds for a span.
    pub fn middle(self) -> u64 {
        (self.lo + self.hi).div_ceil(2)
    }
}

impl fmt::Display for InnerSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_fixed() {
            write!(f, "{}", self.lo)
        } else {
            write!(f, "{}-{}", self.lo, self.hi)
        }
    }
}

impl FromStr for InnerSpan {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let count = |part: &str| {
            part.trim()
                .parse::<u64>()
                .map_err(|_| format!("`{s}` is not a count like 7 or a span like 1-16"))
        };
        match s.split_once('-') {
            Some((lo, hi)) => InnerSpan::new(count(lo)?, count(hi)?),
            None => {
                let n = count(s)?;
                InnerSpan::new(n, n)
            }
        }
    }
}

/// The serialized form: a bare count, or the span as text.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Repr {
    Count(u64),
    Span(String),
}

impl TryFrom<Repr> for InnerSpan {
    type Error = String;

    fn try_from(repr: Repr) -> Result<Self, String> {
        match repr {
            Repr::Count(n) => InnerSpan::new(n, n),
            Repr::Span(s) => s.parse(),
        }
    }
}

impl From<InnerSpan> for Repr {
    fn from(span: InnerSpan) -> Repr {
        if span.is_fixed() {
            Repr::Count(span.lo)
        } else {
            Repr::Span(span.to_string())
        }
    }
}

/// A run's samples and their summed time for each length of a span, the points a regression of
/// sample time on length is fitted through.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LengthTotals {
    /// The length the first entry holds.
    lo: u64,
    /// Samples taken at each length, from `lo` up.
    samples: Vec<u64>,
    /// Their summed time at each length, picoseconds.
    sum_ps: Vec<u128>,
}

impl LengthTotals {
    /// Empty totals over `span`.
    pub fn new(span: InnerSpan) -> Self {
        let n = span.lengths() as usize;
        LengthTotals {
            lo: span.lo,
            samples: vec![0; n],
            sum_ps: vec![0; n],
        }
    }

    /// Count one sample of `length` steps that took `elapsed_ps`.
    #[inline]
    pub fn add(&mut self, length: u64, elapsed_ps: u128) {
        let i = (length - self.lo) as usize;
        self.samples[i] += 1;
        self.sum_ps[i] += elapsed_ps;
    }

    /// Samples taken at each length, from the span's shortest up.
    pub fn samples(&self) -> &[u64] {
        &self.samples
    }

    /// The summed sample time at each length, nanoseconds.
    pub fn sums_ns(&self) -> Vec<f64> {
        self.sum_ps.iter().map(|&ps| ps as f64 / 1000.0).collect()
    }

    /// Steps measured in total: each length times its samples.
    pub fn steps(&self) -> u64 {
        self.samples
            .iter()
            .zip(self.lo..)
            .map(|(&n, length)| n * length)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_and_a_span_parse_and_print() {
        let seven: InnerSpan = "7".parse().unwrap();
        assert!(seven.is_fixed());
        assert_eq!(
            (seven.lo(), seven.hi(), seven.to_string()),
            (7, 7, "7".to_string())
        );
        let span: InnerSpan = "1-16".parse().unwrap();
        assert!(!span.is_fixed());
        assert_eq!((span.lo(), span.hi(), span.lengths()), (1, 16, 16));
        assert_eq!(span.to_string(), "1-16");
        assert_eq!(span.mean(), 8.5);
        assert_eq!(span.middle(), 9);
    }

    #[test]
    fn a_bad_span_is_refused() {
        for bad in ["0", "0-4", "9-3", "x", "1-", "-4", "1-99999"] {
            assert!(bad.parse::<InnerSpan>().is_err(), "{bad}");
        }
    }

    #[test]
    fn a_span_serializes_as_a_count_or_as_text() {
        let json = |s: InnerSpan| serde_json::to_string(&s).unwrap();
        assert_eq!(json(InnerSpan::fixed(7)), "7");
        assert_eq!(json("1-16".parse().unwrap()), "\"1-16\"");
        let back: InnerSpan = serde_json::from_str("\"1-16\"").unwrap();
        assert_eq!(back, "1-16".parse().unwrap());
        let back: InnerSpan = serde_json::from_str("7").unwrap();
        assert_eq!(back, InnerSpan::fixed(7));
        let back: InnerSpan = serde_json::from_str("\"7\"").unwrap();
        assert_eq!(back, InnerSpan::fixed(7));
        assert!(serde_json::from_str::<InnerSpan>("0").is_err());
    }

    #[test]
    fn totals_count_samples_and_steps_by_length() {
        let mut totals = LengthTotals::new("2-4".parse().unwrap());
        totals.add(2, 2_000);
        totals.add(4, 4_000);
        totals.add(4, 6_000);
        assert_eq!(totals.samples(), &[1, 0, 2]);
        assert_eq!(totals.sums_ns(), vec![2.0, 0.0, 10.0]);
        assert_eq!(totals.steps(), 2 + 4 + 4);
    }
}
