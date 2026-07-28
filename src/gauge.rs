//! Grading. Today the run grade ([`RunGrade`]) — a measurement
//! run scored from its own time-ordered [`BatchSummary`] series.
//! The environment grade lands here too at 0.23.0-4, measured
//! from the warmup micro-probe; the two answer different
//! questions and print as separate letters.
//!
//! The calibration grade certified a ~1 s window *before* the run
//! — the room, not the exam. The run grade scores the exam: every
//! signal is computed from the batches the run itself produced,
//! so the letter describes the data being reported.
//!
//! - Four signals, each scored against its own ascending cutoffs
//!   (A..F); the composite letter is the worst of them, and each
//!   signal prints its own letter beside its value so the
//!   composite always names its cause. See
//!   [`RunGrade::scores`] for the arithmetic and
//!   [`thresholds`] for the cutoffs.
//! - **Four, where the calibration grade has six.** The mapping,
//!   and why two have no run-side analog:
//!   - `disturbed` → `interference` — the census rate, same
//!     definition rebased on the batch's own floor.
//!   - `dirty windows` → `bursts` — window becomes batch.
//!   - `drift` → `drift` — floor movement across the
//!     measurement, unchanged in spirit.
//!   - `repeat` → `step` — `repeat` compares constants between
//!     two clean calibration attempts, a transition detector at
//!     attempt-to-attempt scale where `drift` is the same
//!     detector within one window. A single run has no second
//!     attempt, so the equivalent question inside one run is
//!     whether the floor shifted partway through: the split
//!     detector.
//!   - `resid` and `cross` — **no analog, deliberately.** Both
//!     grade the quality of a *fit*: the worst residual of a
//!     ladder point against the Theil-Sen line, and the
//!     loop-only slope against the dithered two-point fit. They
//!     exist because calibration fits a line through a multi-N
//!     ladder. A bench run fits nothing — no line for a point
//!     to sit off, no second estimator to cross-check — so
//!     run-side versions would mean inventing the fit first.
//! - Two of the four watch the run's floor move under the
//!   measurement: `drift` asks whether the run ended where it
//!   began, `step` looks for a shift anywhere inside it and says
//!   when. A run that shifts and shifts back is invisible to the
//!   first and obvious to the second.
//! - The other two split contamination by how it is distributed:
//!   `interference` is the run-wide census rate (how many samples
//!   sat above their batch's floor), `bursts` the fraction of
//!   batches whose mean sits above the typical one (whether that
//!   interference was localized in time or spread evenly).
//!
//! The run grade **reports; it does not warn**. Its signals are facts
//! about the run, and a run's character is largely the workload's:
//! on a quiet 7600x the blocking mpsc round-trips read step F
//! whether pinned or not, while the same round-trip spinning reads
//! A — we think that is park/unpark bimodality, which belongs to
//! the bench, not the box. An F there is a true description of
//! what was measured, not a fault to raise, and the report's job
//! is a faithful histogram rather than a verdict. The verdict on
//! the box is a separate grade measured during warmup, where the
//! workload's character hasn't entered the numbers yet.

use crate::harness::BatchSummary;

/// Grade thresholds, one array per signal: the ascending cutoffs
/// a signal crosses to score B, C, D, F (below the first is A).
///
/// - **Provisional** — seeded from the 3900X's quiet-run spread
///   on 2026-07-27 and checked against a quiet 7600x on
///   2026-07-28, where ten back-to-back `min-now` runs all read A
///   and 12 of 16 benches in one sweep read A.
/// - Calibrated on a fast single-threaded bench, where the run's
///   letter is mostly the box. A blocking multi-threaded bench
///   carries OS involvement in its own numbers and reads worse by
///   nature — correctly, since it *is* less steady. Nothing here
///   raises a warning, so a low letter costs a reader nothing
///   beyond the fact itself.
pub mod thresholds {
    /// Run-wide census rate: samples above their batch's
    /// over-floor cut, as a fraction of all samples.
    pub const INTERFERENCE: [f64; 4] = [0.02, 0.05, 0.12, 0.30];
    /// Fraction of batches whose mean sits [`BURST_TOL`] above the
    /// run's median batch mean.
    pub const BURSTS: [f64; 4] = [0.25, 0.50, 0.75, 0.90];
    /// Floor movement from the run's first quarter to its last.
    pub const DRIFT: [f64; 4] = [0.01, 0.02, 0.05, 0.10];
    /// Largest floor shift across any split of the run.
    pub const STEP: [f64; 4] = [0.01, 0.02, 0.05, 0.10];
}

/// A batch is "hot" when its mean exceeds the run's *median*
/// batch mean by this fraction — it carried something the typical
/// batch did not. The fraction of hot batches says whether the
/// contamination was localized in time or spread over the run.
///
/// - The reference is the median, not the quietest batch: the
///   minimum is an extreme, and against it a bench with any real
///   spread reads ~100% hot (mpsc-2t read 98% on a run whose
///   batch means varied normally).
pub const BURST_TOL: f64 = 0.05;

/// Minimum batches on each side of a candidate split point for
/// the `step` detector to consider it. Below this a "transition"
/// is one or two batches — a burst, which the `bursts` signal
/// already counts.
pub const MIN_SPLIT_BATCHES: usize = 4;

/// Score one signal against its [`thresholds`] array: 0 (A)
/// through 4 (F) — the count of cutoffs crossed.
fn score(x: f64, cutoffs: [f64; 4]) -> u8 {
    cutoffs.iter().filter(|&&c| x > c).count() as u8
}

/// Map a 0..=4 score to its letter (no 'E': 4 is 'F').
fn score_letter(s: u8) -> char {
    match s {
        0 => 'A',
        1 => 'B',
        2 => 'C',
        3 => 'D',
        _ => 'F',
    }
}

/// The run grade: signals are facts about the run's batch series,
/// `letter` is the worst signal's grade.
#[derive(Debug)]
pub struct RunGrade {
    /// Samples above their batch's over-floor cut, as a fraction
    /// of all samples in the run.
    pub interference_frac: f64,
    /// Fraction of batches whose mean is [`BURST_TOL`] above the
    /// run's median batch mean.
    pub burst_frac: f64,
    /// End to end: median batch floor of the run's last quarter
    /// against its first, relative.
    pub drift_frac: f64,
    /// The largest floor shift any split of the run divides,
    /// relative — median floor before against median floor after.
    pub step_frac: f64,
    /// Where that split fell — seconds from run start.
    pub step_at_s: f64,
    /// Overall letter, worst signal wins: A, B, C, D, or F.
    pub letter: char,
}

impl RunGrade {
    /// Grade a run from its time-ordered batch summaries; `None`
    /// when the run produced no batches (nothing to grade).
    ///
    /// - Both floor signals read `floor_q_ps`, the batch's robust
    ///   low-quantile floor, never its raw min — see
    ///   [`crate::harness::BATCH_FLOOR_Q`] for the measurement
    ///   that settled it.
    /// - `drift` compares the median floor of the run's first
    ///   quarter against its last: the plain end-to-end question,
    ///   did the run finish where it started. Movement is
    ///   reported, not attributed — see the module doc.
    /// - `step` is the transition detector — the split point that
    ///   most divides the floor series, scored on the medians of
    ///   the two sides. A run that shifts and shifts back reads
    ///   low on drift and high on step; the reported time says
    ///   when it happened.
    /// - Both are medians, not extremes, on purpose: a single hot
    ///   batch is a burst, not a transition, and an adjacent-pair
    ///   detector graded every quiet 3900X run D/F on exactly
    ///   those isolated batches.
    /// - Runs shorter than [`MIN_SPLIT_BATCHES`] batches per side
    ///   score 0 on both — too few floors to say anything.
    pub fn from_batches(batches: &[BatchSummary]) -> Option<Self> {
        if batches.is_empty() {
            return None;
        }

        let total: u64 = batches.iter().map(|b| b.count).sum();
        let over: u64 = batches.iter().map(|b| b.over_floor).sum();
        let interference_frac = if total == 0 {
            0.0
        } else {
            over as f64 / total as f64
        };

        let means: Vec<f64> = batches.iter().map(|b| b.mean_ps).collect();
        let typical = median(&means).unwrap_or(0.0); // OK: `batches` is non-empty
        let hot = means
            .iter()
            .filter(|&&m| m > typical * (1.0 + BURST_TOL))
            .count();
        let burst_frac = hot as f64 / batches.len() as f64;

        let floors: Vec<f64> = batches.iter().map(|b| b.floor_q_ps as f64).collect();
        let n = floors.len();

        let quarter = (n / 4).max(1);
        let drift_frac = split_change(&floors[..quarter], &floors[n - quarter..]);

        // Every interior split point, keeping MIN_SPLIT_BATCHES on
        // each side. n is the batch count (~20/s), so the O(n^2 log
        // n) scan costs microseconds against a run's seconds.
        //
        // Ranked by change x the split's balance, `n1 * n2 / n^2`:
        // a run's floor series has a plateau of splits reading the
        // same change (any cut inside the earlier level sees the
        // same two medians), and the balance term picks the one
        // nearest the middle of that plateau — the transition
        // itself — instead of whichever tie came first.
        let mut step_frac = 0.0f64;
        let mut step_at_s = batches[0].t_start_s;
        let mut best_rank = 0.0f64;
        if n >= 2 * MIN_SPLIT_BATCHES {
            for t in MIN_SPLIT_BATCHES..=(n - MIN_SPLIT_BATCHES) {
                let change = split_change(&floors[..t], &floors[t..]);
                let balance = (t * (n - t)) as f64 / (n * n) as f64;
                let rank = change * balance;
                if rank > best_rank {
                    best_rank = rank;
                    step_frac = change;
                    step_at_s = batches[t].t_start_s;
                }
            }
        }

        let mut grade = Self {
            interference_frac,
            burst_frac,
            drift_frac,
            step_frac,
            step_at_s,
            letter: 'A',
        };
        grade.letter = score_letter(grade.scores().into_iter().fold(0, u8::max));
        Some(grade)
    }

    /// Per-signal scores in print order: interference, bursts,
    /// drift, step.
    ///
    /// Each is the count of its [`thresholds`] cutoffs crossed —
    /// 0 (below all four) through 4 — and the composite is
    /// `fold(0, u8::max)` over them: the worst signal wins
    /// outright, so one F makes the letter F and no number of
    /// A's pulls it back. Printing every signal's letter beside
    /// the composite makes that visible: the overall letter is
    /// always one of the four shown.
    fn scores(&self) -> [u8; 4] {
        [
            score(self.interference_frac, thresholds::INTERFERENCE),
            score(self.burst_frac, thresholds::BURSTS),
            score(self.drift_frac, thresholds::DRIFT),
            score(self.step_frac, thresholds::STEP),
        ]
    }

    /// Letters for the gauge line's printed signals, in print
    /// order — all four composite inputs, so the worst letter on
    /// the line *is* the composite.
    pub fn signal_letters(&self) -> [char; 4] {
        self.scores().map(score_letter)
    }
}

/// Relative change between two runs of floors, each summarized by
/// its median: `|median(b) - median(a)| / median(a)`. Zero when
/// either side is empty or the earlier median is not positive.
fn split_change(a: &[f64], b: &[f64]) -> f64 {
    match (median(a), median(b)) {
        (Some(early), Some(late)) if early > 0.0 => (late - early).abs() / early,
        _ => 0.0,
    }
}

/// Median of a slice, `None` when empty. Copies to sort — the
/// slices are batch counts, not sample counts.
fn median(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    Some(s[s.len() / 2])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One batch with the given floor / mean / census counts,
    /// stamped at `at` seconds and 50 ms long.
    fn batch_at(at: f64, floor_ps: u64, mean_ps: f64, count: u64, over_floor: u64) -> BatchSummary {
        BatchSummary {
            t_start_s: at,
            t_end_s: at + 0.05,
            count,
            floor_ps,
            floor_q_ps: floor_ps,
            mean_ps,
            max_ps: floor_ps * 2,
            over_floor,
        }
    }

    /// A run of `n` identical batches, one every 50 ms.
    fn steady(n: usize, floor_ps: u64, mean_ps: f64, over_floor: u64) -> Vec<BatchSummary> {
        (0..n)
            .map(|i| batch_at(i as f64 * 0.05, floor_ps, mean_ps, 1000, over_floor))
            .collect()
    }

    #[test]
    fn empty_run_has_no_grade() {
        assert!(RunGrade::from_batches(&[]).is_none());
    }

    #[test]
    fn quiet_run_grades_a() {
        let g = RunGrade::from_batches(&steady(8, 1000, 1010.0, 2)).expect("graded");
        assert_eq!(g.letter, 'A');
        assert_eq!(g.signal_letters(), ['A', 'A', 'A', 'A']);
    }

    #[test]
    fn single_batch_cannot_move() {
        let g = RunGrade::from_batches(&steady(1, 1000, 1010.0, 0)).expect("graded");
        assert_eq!(g.drift_frac, 0.0);
        assert_eq!(g.step_frac, 0.0);
    }

    /// A run whose floor follows `segments` — `(batches, floor)`
    /// each, one batch every 50 ms from t=0.
    fn run_of(segments: &[(usize, u64)]) -> Vec<BatchSummary> {
        let mut out = Vec::new();
        for &(n, floor) in segments {
            for _ in 0..n {
                let at = out.len() as f64 * 0.05;
                out.push(batch_at(at, floor, floor as f64 + 10.0, 1000, 0));
            }
        }
        out
    }

    #[test]
    fn floor_shift_lights_drift_and_step() {
        // Halves at 1000 / 1200 ps: 20% end to end, and the split
        // at the seam finds the same 20%, at t = 8 x 50 ms.
        let g = RunGrade::from_batches(&run_of(&[(8, 1000), (8, 1200)])).expect("graded");
        assert!((g.drift_frac - 0.2).abs() < 1e-9);
        assert!((g.step_frac - 0.2).abs() < 1e-9);
        assert!((g.step_at_s - 0.4).abs() < 1e-9);
        assert_eq!(g.letter, 'F');
        assert_eq!(g.signal_letters(), ['A', 'A', 'F', 'F']);
    }

    #[test]
    fn returning_shift_hides_from_drift_only() {
        // Out and back: the run ends where it started, so drift
        // sees nothing and the split detector sees the departure.
        let g =
            RunGrade::from_batches(&run_of(&[(6, 1000), (8, 1200), (6, 1000)])).expect("graded");
        assert_eq!(g.drift_frac, 0.0);
        assert!((g.step_frac - 0.2).abs() < 1e-9);
        // No assertion on `step_at_s`: two transitions have no one
        // split point, and the balance term lands it between them.
    }

    #[test]
    fn transient_batch_is_not_a_transition() {
        // One hot batch in twenty: a burst, not a state change —
        // the medians on both sides of every split are unmoved.
        // (The adjacent-pair detector this replaced read 20%.)
        let g =
            RunGrade::from_batches(&run_of(&[(10, 1000), (1, 1200), (9, 1000)])).expect("graded");
        assert_eq!(g.drift_frac, 0.0);
        assert_eq!(g.step_frac, 0.0);
        assert_eq!(g.letter, 'A');
    }

    #[test]
    fn census_counts_drive_interference() {
        let mut batches = steady(2, 1000, 1010.0, 0);
        batches[0].over_floor = 100;
        let g = RunGrade::from_batches(&batches).expect("graded");
        assert!((g.interference_frac - 0.05).abs() < 1e-9);
        assert_eq!(g.signal_letters()[0], 'B');
    }

    #[test]
    fn hot_batches_drive_bursts() {
        // Three of ten means sit > 5% above the median batch.
        let mut batches = steady(10, 1000, 1000.0, 0);
        for b in batches.iter_mut().take(3) {
            b.mean_ps = 1200.0;
        }
        let g = RunGrade::from_batches(&batches).expect("graded");
        assert!((g.burst_frac - 0.3).abs() < 1e-9);
        assert_eq!(g.signal_letters()[1], 'B');
    }

    #[test]
    fn a_broad_bench_is_not_bursty() {
        // Batch means scattered ±4% around the median: spread,
        // not bursts. Against the *quietest* batch this read 100%.
        let mut batches = steady(10, 1000, 1000.0, 0);
        for (i, b) in batches.iter_mut().enumerate() {
            b.mean_ps = if i % 2 == 0 { 1000.0 } else { 1040.0 };
        }
        let g = RunGrade::from_batches(&batches).expect("graded");
        assert_eq!(g.burst_frac, 0.0);
    }
}
