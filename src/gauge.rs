//! Grading — two letters, answering two different questions.
//!
//! - [`RunGrade`] scores a measurement run from its own
//!   time-ordered [`BatchSummary`] series: how steady were *these
//!   numbers*, the ones being reported.
//! - [`EnvGrade`] scores the **box** from the warmup micro-probe
//!   series ([`ProbeSummary`]): how steady was the machine,
//!   measured on the apparatus alone before any workload entered
//!   the numbers.
//!
//! They share their arithmetic (signals scored against ascending
//! cutoffs, worst signal wins, every signal printing its own
//! letter) and deliberately not their thresholds or their signal
//! sets. A bench that reads F on the run grade and A on the
//! environment grade is a true and useful statement: the workload
//! is bursty on a quiet machine.
//!
//! Every signal is computed from the batches the run itself
//! produced, so the letter describes the data being reported
//! rather than a window measured beforehand.
//!
//! - Four signals, each scored against its own ascending cutoffs
//!   (A..F); the composite letter is the worst of them, and each
//!   signal prints its own letter beside its value so the
//!   composite always names its cause. See
//!   [`RunGrade::scores`] for the arithmetic and
//!   [`thresholds`] for the cutoffs.
//! - Four, where the startup calibration grade this replaced had
//!   six. Two of those six graded how well a *fit* held (a
//!   ladder point's residual against the Theil-Sen line, and the
//!   loop-only slope against the dithered two-point fit), and a
//!   bench run fits nothing: no line for a point to sit off, no
//!   second estimator to cross-check. Inventing a fit just to
//!   grade it would have been the wrong direction. The full
//!   mapping is recorded in
//!   notes/chores/chores-05.md#six-calibration-signals-four-run-signals.
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

use crate::harness::{BatchSummary, ProbeSummary};

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

/// Environment-grade thresholds — the same shape as
/// [`thresholds`], applied to the warmup micro-probe series.
///
/// - **Provisional**, and deliberately *not* shared with the run
///   grade even where the number happens to match: the two grade
///   different populations. A run's signals carry the workload's
///   character, a probe's carry only the box's, so the cutoffs
///   are free to diverge as measurement says they should.
/// - `drift` and `step` keep the run's cutoffs as a starting
///   point: both measure the same physical thing (floor movement
///   under the measurement), and the 3900X's ~9% bistable shift
///   sits well above the D cutoff either way.
pub mod env_thresholds {
    /// Probe spread: the p90-over-floor width of a typical probe.
    pub const SPREAD: [f64; 4] = [0.02, 0.05, 0.10, 0.25];
    /// Census rate across the probe series: individual timer
    /// pairs above their probe's over-floor cut, as a fraction of
    /// all pairs. Counted per pair rather than per timed group
    /// because a group mean hides anything smaller than ~800 ns.
    ///
    /// - **The weakest of the four, and uncalibrated.** Set one
    ///   order of magnitude above the measured quiet baseline
    ///   (0.01% on a quiet 3900X) because there is no good upper
    ///   anchor: sharing the measured core with one spinner moved
    ///   it only to 0.04%, and with three spinners it read 0.01%
    ///   again. See
    ///   [`EnvGrade`]'s note on what the probe cannot see.
    pub const INTERFERENCE: [f64; 4] = [0.001, 0.005, 0.02, 0.05];
    /// Floor movement from the first quarter of warmup to the
    /// last — the frequency-ramp detector.
    pub const DRIFT: [f64; 4] = [0.01, 0.02, 0.05, 0.10];
    /// Largest floor shift across any split of the warmup window.
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

        let times: Vec<f64> = batches.iter().map(|b| b.t_start_s).collect();
        let (step_frac, step_at_s) = best_split(&floors, &times);

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

/// The transition detector, shared by both grades: the split
/// point that most divides a floor series, returned as
/// `(relative change, time of the split)`.
///
/// - Scans every interior split keeping [`MIN_SPLIT_BATCHES`] on
///   each side, scoring each on the medians of the two sides. `n`
///   is a batch or probe count (tens), so the O(n^2 log n) scan
///   costs microseconds against a run's seconds.
/// - Ranked by change x the split's balance, `n1 * n2 / n^2`: a
///   floor series has a plateau of splits reading the same change
///   (any cut inside the earlier level sees the same two
///   medians), and the balance term picks the one nearest the
///   middle of that plateau — the transition itself — instead of
///   whichever tie came first.
/// - Series with fewer than `2 * MIN_SPLIT_BATCHES` points score
///   0 at the first timestamp: too few floors to say anything.
/// - `floors` and `times` are parallel; a short `times` only
///   costs the reported timestamp, never the change.
fn best_split(floors: &[f64], times: &[f64]) -> (f64, f64) {
    let n = floors.len();
    let mut step_frac = 0.0f64;
    let mut step_at_s = times.first().copied().unwrap_or(0.0);
    let mut best_rank = 0.0f64;
    if n >= 2 * MIN_SPLIT_BATCHES {
        for t in MIN_SPLIT_BATCHES..=(n - MIN_SPLIT_BATCHES) {
            let change = split_change(&floors[..t], &floors[t..]);
            let balance = (t * (n - t)) as f64 / (n * n) as f64;
            let rank = change * balance;
            if rank > best_rank {
                best_rank = rank;
                step_frac = change;
                step_at_s = times.get(t).copied().unwrap_or(step_at_s);
            }
        }
    }
    (step_frac, step_at_s)
}

/// The environment grade: a verdict on the **box**, scored from
/// the warmup micro-probe series.
///
/// - The run grade's signals carry the workload's character — a
///   blocking round-trip is genuinely less steady than a spinning
///   one, so its letter describes the bench as much as the
///   machine. The probes touch no bench at all, so this letter is
///   the machine alone. That separation is the whole point:
///   warmup is the only workload-independent window a run has.
/// - Four signals, worst wins, same arithmetic as [`RunGrade`],
///   scored against [`env_thresholds`].
/// - Signal choice against the run grade's four:
///   - `interference` and `drift` and `step` — the same
///     questions, same definitions, different population.
///   - `spread` — **new here.** How wide a probe's bulk sits
///     above its own floor. There is no run-side analog worth
///     having: a bench's spread is mostly its workload (park /
///     unpark bimodality is a fact about `mpsc`, not the box),
///     while a timer pair has no character of its own, so its
///     width is the machine's.
///   - `bursts` — **dropped.** With [`crate::harness::ENV_PROBES`]
///     probes the hot-fraction is granular to 1/16, and the
///     contamination it would count is already counted by
///     `interference` at group resolution.
/// - **What the probe cannot see.** It measures the box only
///   while the measuring thread is running, so time the thread
///   spends descheduled is largely invisible: a ~256 µs probe
///   usually fits inside one scheduling quantum and completes
///   without being preempted at all. Measured on the 3900X,
///   sharing the pinned core with one spinner moved
///   `interference` from 0.01% to 0.04% and with three spinners
///   it read 0.01% — while `spread` (0.31% to 2.02%) and
///   `drift`/`step` (A to D) responded properly. So this grade
///   detects frequency and state movement well and preemption
///   poorly; `interference` catches only the rare large
///   intrusion that lands inside a probe.
/// - This is the grade that could earn warnings later: an F here
///   is a statement about the machine, which is actionable in a
///   way that an F on a blocking bench is not. Nothing warns yet.
#[derive(Debug)]
pub struct EnvGrade {
    /// Median over probes of each probe's spread: its upper
    /// quantile over its floor, relative.
    pub spread_frac: f64,
    /// Individual timer pairs above their probe's over-floor
    /// cut, as a fraction of all pairs in the series.
    pub interference_frac: f64,
    /// Floor movement from the first quarter of the probe series
    /// to the last, relative.
    pub drift_frac: f64,
    /// The largest floor shift any split of the series divides,
    /// relative.
    pub step_frac: f64,
    /// Where that split fell — seconds from warmup start.
    pub step_at_s: f64,
    /// Overall letter, worst signal wins: A, B, C, D, or F.
    pub letter: char,
}

impl EnvGrade {
    /// Grade the environment from the warmup probe series; `None`
    /// when warmup produced no probes.
    pub fn from_probes(probes: &[ProbeSummary]) -> Option<Self> {
        if probes.is_empty() {
            return None;
        }

        let spreads: Vec<f64> = probes
            .iter()
            .map(|p| {
                if p.floor_q_ps == 0 {
                    0.0
                } else {
                    (p.spread_q_ps as f64 - p.floor_q_ps as f64) / p.floor_q_ps as f64
                }
            })
            .collect();
        let spread_frac = median(&spreads).unwrap_or(0.0); // OK: `probes` is non-empty

        let total: u64 = probes.iter().map(|p| p.pairs).sum();
        let over: u64 = probes.iter().map(|p| p.over_pairs).sum();
        let interference_frac = if total == 0 {
            0.0
        } else {
            over as f64 / total as f64
        };

        let floors: Vec<f64> = probes.iter().map(|p| p.floor_q_ps as f64).collect();
        let n = floors.len();
        let quarter = (n / 4).max(1);
        let drift_frac = split_change(&floors[..quarter], &floors[n - quarter..]);

        let times: Vec<f64> = probes.iter().map(|p| p.t_start_s).collect();
        let (step_frac, step_at_s) = best_split(&floors, &times);

        let mut grade = Self {
            spread_frac,
            interference_frac,
            drift_frac,
            step_frac,
            step_at_s,
            letter: 'A',
        };
        grade.letter = score_letter(grade.scores().into_iter().fold(0, u8::max));
        Some(grade)
    }

    /// Per-signal scores in print order: spread, interference,
    /// drift, step — each the count of its [`env_thresholds`]
    /// cutoffs crossed, composite is the worst.
    fn scores(&self) -> [u8; 4] {
        [
            score(self.spread_frac, env_thresholds::SPREAD),
            score(self.interference_frac, env_thresholds::INTERFERENCE),
            score(self.drift_frac, env_thresholds::DRIFT),
            score(self.step_frac, env_thresholds::STEP),
        ]
    }

    /// Letters for the gauge line's printed signals, in print
    /// order — all four composite inputs, so the worst letter on
    /// the line *is* the composite.
    pub fn signal_letters(&self) -> [char; 4] {
        self.scores().map(score_letter)
    }
}

/// When the warmup stretch settled: seconds from warmup start to the start of the earliest
/// suffix of the stretch that grades A. `None` when there is nothing to measure (an empty
/// stretch or a `tail_len` the stretch cannot hold).
///
/// - **Why report it at all.** Once warmup deliberately spans the ramp, the grade stops seeing
///   the ramp, which is warmup working, and would otherwise leave the report saying nothing
///   about a box that took a second to come up to speed. The letter answers "was it settled
///   when measurement started"; this answers "how long did that take".
/// - **Settled means "graded A from here on"**, the same computation as the letter, so the two
///   cannot disagree: the retired median-band rule read `A` beside `not settled` on the 3900X's
///   bimodal flicker, a window whose grade accepted movement the 1% band did not (2026-08-02).
///   The scan walks suffix starts from the front, so the reported time is the earliest moment
///   from which the rest of the warm reads clean; a stretch settled from its first probe reads
///   `0.00s`.
/// - `tail_len` is the exit window's probe count ([`ProbeSummary`] count, the caller's
///   `RunOutput::warm_tail`): the shortest suffix the exit condition certified. The scan never
///   considers a shorter one, so a settle time is always backed by at least the window the
///   letter was scored over, and a settled exit always finds one ([`Settle::Never`] can only
///   reach the report on a cap exit).
pub fn settle(warm: &[ProbeSummary], tail_len: usize) -> Option<Settle> {
    if warm.is_empty() || tail_len == 0 || tail_len > warm.len() {
        return None;
    }
    for start in 0..=(warm.len() - tail_len) {
        if EnvGrade::from_probes(&warm[start..]).is_some_and(|g| g.letter == 'A') {
            return Some(Settle::At(warm[start].t_start_s));
        }
    }
    Some(Settle::Never)
}

/// What [`settle`] found: when the stretch's trailing suffix first grades A, or that none did.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Settle {
    /// Seconds from warmup start to the earliest A-grading suffix.
    At(f64),
    /// No suffix down to the exit window graded A: still moving when the stretch ended.
    Never,
}

impl std::fmt::Display for Settle {
    /// `0.84s` / `not settled`: the grade block's `settle` cell
    /// on the warmup row, parsed back by `qualify-environment`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Settle::At(s) => write!(f, "{s:.2}s"),
            Settle::Never => write!(f, "not settled"),
        }
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

    /// One probe with the given floor / upper quantile / census
    /// count, stamped at `at` seconds.
    fn probe_at(at: f64, floor_ps: u64, spread_q_ps: u64, over_pairs: u64) -> ProbeSummary {
        ProbeSummary {
            t_start_s: at,
            groups: 128,
            floor_q_ps: floor_ps,
            spread_q_ps,
            mean_ps: floor_ps as f64 + 10.0,
            pairs: 8192,
            over_pairs,
        }
    }

    /// A series of `n` identical probes, one every millisecond.
    fn probes(n: usize, floor_ps: u64, spread_q_ps: u64, over_pairs: u64) -> Vec<ProbeSummary> {
        (0..n)
            .map(|i| probe_at(i as f64 * 0.001, floor_ps, spread_q_ps, over_pairs))
            .collect()
    }

    #[test]
    fn warmup_with_no_probes_has_no_grade() {
        assert!(EnvGrade::from_probes(&[]).is_none());
    }

    #[test]
    fn quiet_box_grades_a() {
        // 25.0 ns floor, 25.2 ns p90: 0.8% spread, no census hits.
        let g = EnvGrade::from_probes(&probes(16, 25_000, 25_200, 0)).expect("graded");
        assert_eq!(g.letter, 'A');
        assert_eq!(g.signal_letters(), ['A', 'A', 'A', 'A']);
    }

    #[test]
    fn wide_probes_drive_spread() {
        // p90 sits 12% over the floor — past the C cutoff (0.10).
        let g = EnvGrade::from_probes(&probes(16, 25_000, 28_000, 0)).expect("graded");
        assert!((g.spread_frac - 0.12).abs() < 1e-9);
        assert_eq!(g.signal_letters()[0], 'D');
        assert_eq!(g.letter, 'D');
    }

    #[test]
    fn census_counts_drive_env_interference() {
        let mut ps = probes(16, 25_000, 25_200, 0);
        // 1,024 of 131,072 pairs over the cut: 0.78%, past the
        // 0.005 cutoff and short of 0.02.
        ps[0].over_pairs = 1_024;
        let g = EnvGrade::from_probes(&ps).expect("graded");
        assert!((g.interference_frac - 0.0078125).abs() < 1e-9);
        assert_eq!(g.signal_letters()[1], 'C');
    }

    #[test]
    fn a_ramping_box_lights_drift_and_step() {
        // The frequency ramp this grade exists to catch: the
        // floor settles 20% lower halfway through warmup.
        let mut ps = probes(16, 30_000, 30_200, 0);
        for (i, p) in ps.iter_mut().enumerate().skip(8) {
            p.floor_q_ps = 24_000;
            p.spread_q_ps = 24_160;
            p.t_start_s = i as f64 * 0.001;
        }
        let g = EnvGrade::from_probes(&ps).expect("graded");
        assert!((g.drift_frac - 0.2).abs() < 1e-9);
        assert!((g.step_frac - 0.2).abs() < 1e-9);
        assert!((g.step_at_s - 0.008).abs() < 1e-9);
        assert_eq!(g.signal_letters(), ['A', 'A', 'F', 'F']);
    }

    #[test]
    fn one_disturbed_probe_is_not_a_transition() {
        // A single slow probe among sixteen: the medians on both
        // sides of every split are unmoved.
        let mut ps = probes(16, 25_000, 25_200, 0);
        ps[9].floor_q_ps = 40_000;
        let g = EnvGrade::from_probes(&ps).expect("graded");
        assert_eq!(g.drift_frac, 0.0);
        assert_eq!(g.step_frac, 0.0);
        assert_eq!(g.letter, 'A');
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
