//! Apparatus-overhead calibration, paired and amortized.
//!
//! Three constants, all from paired window passes over one
//! shared [`run_inner`]:
//!
//! - **Loop-only ladder** ([`LOOP_LADDER`], min over amortized
//!   window means at each N, samples-per-window scaled so window
//!   duration stays constant): the Theil-Sen slope across the
//!   points is `loop_per_iter_ns`, subtracted per call.
//! - **Window pass minus the ladder's `N_LOW` point** (identical
//!   but for the per-sample timer pair) is `frame_call_ns`, the
//!   full call-to-call cost of taking a sample — sizes the
//!   experiment, never subtracted (most of it sits outside
//!   recorded intervals).
//! - **Dither `N_LOW` window minimum minus the same point**
//!   gives `frame_sample_ns`, the in-interval timer-pair slice a
//!   recorded sample actually contains, subtracted per sample.
//!
//! One shared compiled loop makes the loop term cancel exactly
//! in both differences, and pairing (never extrapolating) keeps
//! any one contaminated point from levering a constant negative.
//!
//! The slope needs neither dither nor quantiles: a loop-only
//! window has no per-sample timer reads, so the ~10 ns clock
//! quantum lands once per window (not once per sample) and the
//! per-iteration quantization error is ~0.0001 ns as measured.
//! The dithered two-point fit that used to produce the slope is
//! retained as a diagnostic only — its divergence from the
//! loop-only slope checks the one assumption the pairing leans
//! on (that the loop costs the same with and without timer pairs
//! interleaved). See
//! notes/chores/chores-04.md#replanning-slope-dither-and-self-checks.
//!
//! Dither makes the ~10 ns clock quantum a zero-mean error that
//! averages away in means (validated on r5-7600x: `frame_sample`
//! 8.23 / 8.25 / 8.29 ns, a 0.062 ns spread, within a frequency
//! regime); its one production consumer is the window *means*
//! behind `frame_sample_ns`. See
//! notes/design.md#dithering-random-phase-injection,
//! notes/design.md#timer-overhead-in-interval-vs-call-to-call and
//! notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit.
//!
//! Every calibration also self-checks and grades its
//! environment ([`CalGrade`]): physical invariants drive
//! retries, statistical signals (interference census, drift
//! bracket, linearity, slope cross-check, repeatability across
//! two clean attempts) feed an always-printed letter grade,
//! with plain-language warnings only at D or worse. The user is
//! never assumed to know the diagnostics exist.

use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::harness::Bench;

/// Calibration warmup iterations — long enough for CPU frequency
/// boost to ramp before the first measurement.
pub const CAL_WARMUP: u64 = 100_000;

/// Attempts before a physically impossible calibration is
/// reported rather than retried. Interference is transient, so a
/// retry usually lands on a quieter moment; a persistent
/// violation is a real problem the user needs to see.
pub const CAL_MAX_ATTEMPTS: u32 = 3;

/// Clean (violation-free) attempts calibration collects before
/// publishing — the second exists to *measure repeatability*,
/// the environment grade's headline number: two attempts that
/// disagree mean the constants don't transfer to the bench run
/// that follows, whatever each attempt's internal statistics
/// say. The published constants come from the last clean
/// attempt (the most warmed-up machine state).
pub const CAL_CLEAN_ATTEMPTS: usize = 2;

/// Inner-loop count for the low-N calibration point.
pub const N_LOW: u64 = 100;

/// Inner-loop count for the high-N calibration point. A wide spread
/// (`N_HIGH / (N_HIGH - N_LOW) ≈ 1.01`) keeps noise amplification on
/// the fitted intercept small.
pub const N_HIGH: u64 = 10_000;

/// Inner-loop counts for the loop-only slope ladder, geometric
/// from [`N_LOW`] to [`N_HIGH`].
///
/// - The slope needs the extremes; the interior points exist so
///   a violated model is *visible* — two points always fit a
///   line perfectly, however corrupted. Their residuals feed the
///   linearity self-check.
/// - Theil-Sen across all pairs (see [`theil_sen_slope`]) keeps
///   one contaminated point from steering the production slope.
pub const LOOP_LADDER: [u64; 5] = [100, 300, 1_000, 3_000, 10_000];
const _: () = assert!(LOOP_LADDER[0] == N_LOW);
const _: () = assert!(LOOP_LADDER[LOOP_LADDER.len() - 1] == N_HIGH);

/// Iterations per window in a loop-only ladder pass:
/// [`loop_samples`] scales samples-per-window as ~1/N against
/// this budget, holding window *duration* roughly constant — the
/// property that lets min-over-windows find windows that slip
/// between preemptions at every N, and that keeps the ladder's
/// points comparable to the [`N_LOW`] passes they pair with.
pub const LOOP_WINDOW_ITERS: u64 = N_LOW * W_LOW_SAMPLES;

/// Samples per window for a loop-only ladder pass at inner-loop
/// count `n`, from the [`LOOP_WINDOW_ITERS`] budget.
///
/// - At small `samples` the window's own framing (one timer pair
///   per window) amortizes over fewer samples; at the N_HIGH
///   point that is ~2 ns against a ~5,000 ns sample, ~0.0003
///   ns/iter on the slope — accepted, not corrected.
pub fn loop_samples(n: u64) -> u64 {
    (LOOP_WINDOW_ITERS / n).max(1)
}

/// Samples per window in the call-to-call and loop-only passes at
/// [`N_LOW`]. The window's ±1-quantum error divides by this:
/// ~0.01 ns at a ~10 ns quantum.
///
/// - Lowered from 10,000 with the window count raised to match,
///   leaving the total budget unchanged. Window *duration* is
///   what decides whether min-over-windows can find a clean
///   window: at 10,000 samples an unoptimized build ran ~7.9 ms
///   per window, so under a continuous competitor every window
///   was contaminated and `l_low` came out *above* the interval
///   containing it. Shorter windows fit between preemptions.
/// - The two passes are differenced, so their windows must be
///   comparable in duration or the difference is meaningless.
pub const W_LOW_SAMPLES: u64 = 1_000;

/// Windows in the call-to-call and loop-only passes; the minimum
/// is kept. The min sheds windows inflated by preemption or a
/// frequency dip while staying amortized (each candidate is
/// already a window mean).
pub const W_LOW_WINDOWS: u64 = 1_000;

/// Dither span in neutral spin iterations (~0.4-0.5 ns each →
/// ~26-32 ns, spanning ~3 clock quanta). A random 0..span delay
/// before each sample randomizes its phase on the ~10 ns clock
/// lattice, making the quantization error zero-mean (see
/// notes/design.md#dithering-random-phase-injection).
pub const DITHER_SPAN: u64 = 64;

/// Windows in the dithered [`N_HIGH`] pass (each yields one
/// window mean; the spread across windows is the dispersion
/// signal). The dithered [`N_LOW`] pass instead uses the
/// [`W_LOW_WINDOWS`] x [`W_LOW_SAMPLES`] shape: its
/// `min_window_ns` is differenced against the loop-only pass
/// for `frame_sample`, and differenced passes need identical
/// window shapes or their order statistics aren't comparable
/// (the d_low/l_low shape mismatch measurably destabilized
/// `min_window_ns` by ~8% between attempts).
pub const DITHER_WINDOWS: u64 = 40;

/// Dithered-fit samples per window at [`N_HIGH`] (samples are
/// ~100× longer than at [`N_LOW`], so few keep the wall cost
/// bounded).
pub const DITHER_HIGH_SAMPLES: u64 = 250;

/// Fastest window means averaged for the `fast` diagnostic.
///
/// - **Not** the production estimator: window-level tail
///   selection assumes interference is sporadic enough that some
///   window escapes it. Under a continuous competitor no
///   `N_HIGH` window does (measured: fastest window 142,644 ns
///   against a 76,573 ns sample floor), so this reads as
///   contaminated as the means. Kept as a logged comparison
///   because its *divergence* from the sample quantiles is a
///   contamination signal. See
///   notes/chores/chores-04.md#one-sided-contamination-and-the-two-point-fit.
pub const DITHER_FAST_WINDOWS: u64 = 4;

/// Candidate discard fractions for the low-quantile estimator:
/// drop this fraction of the fastest samples, then take the
/// minimum of what remains (so `0.0` is the strict minimum).
///
/// - Scheduler interference is one-sided, so the *low* tail is
///   the uncontaminated part of the distribution — but its very
///   bottom is where samples that rounded down on the ~10 ns
///   clock lattice collect. Discarding a slice sheds those
///   without giving up the robustness.
/// - The aim is a *stable* estimate, not an unbiased one: a
///   small repeatable bias is subtracted consistently, whereas
///   contamination is neither small nor repeatable.
pub const DITHER_LOW_Q: [f64; 4] = [0.0, 0.001, 0.01, 0.05];

/// Index into [`DITHER_LOW_Q`] selecting the discard fraction
/// the diagnostic two-point fit reads (see the module doc — the
/// production slope is the loop-only Theil-Sen; this fit is the
/// cross-check on the with/without-timer-pairs assumption).
pub const DITHER_PROD_Q: usize = 2;

/// A sample is "disturbed" when it exceeds the low-quantile
/// floor by this multiple *and* by [`DISTURBED_ABS_NS`] — the
/// calibration passes double as a census of scheduler
/// interference, and `max(floor x mult, floor + abs)` is the
/// census line. Interference is one-sided, so everything sits
/// between the floor and the spikes.
pub const DISTURBED_MULT: f64 = 1.5;

/// Absolute part of the disturbed-sample bound, in ns. The
/// multiplicative part alone breaks down when the floor is only
/// a few clock quanta: at a 60 ns d_low floor, 1.5x lands at
/// 90 ns — *inside* the legitimate lattice+dither range (~10 ns
/// quantum), and read 6.3% "disturbed" on a quiet machine.
/// ~5 quanta clears the lattice spread at short samples while
/// staying far below any preemption; at long samples the
/// multiplicative part dominates anyway.
pub const DISTURBED_ABS_NS: f64 = 50.0;

/// A window is "dirty" when its mean exceeds the minimum window
/// mean by this fraction — it contained interference the
/// min-over-windows estimator had to shed. The *fraction* of
/// dirty windows measures how hard clean windows were to find.
pub const DIRTY_WINDOW_TOL: f64 = 0.05;

/// Environment-grade thresholds, one array per signal: the
/// ascending cutoffs a signal crosses to score B, C, D, F (below
/// the first is A). The overall letter is the worst signal.
///
/// - **Provisional** — seeded from quiet-3900X spread on
///   2026-07-25; the -5 validation pass on both boxes is meant
///   to tune them. The design constraint: a quiet release-build
///   machine should essentially never leave A/B, because a
///   false alarm every third run destroys trust in the warning.
/// - WARNINGs print only at D or worse; C is visible in the
///   always-printed grade line without shouting. First
///   measurements: a quiet 3900X graded B (debug included), a
///   restless one C, and a cross-attempt frequency-regime shift
///   D with its warning — the intended spread.
pub mod grade_thresholds {
    /// Disturbed-sample fraction (worst of d_low / d_high).
    pub const DISTURBED: [f64; 4] = [0.005, 0.02, 0.05, 0.15];
    /// Dirty-window fraction (worst of d_low / d_high).
    pub const DIRTY_WINDOWS: [f64; 4] = [0.25, 0.50, 0.75, 0.90];
    /// Loop-only floor drift across the calibration bracket.
    pub const DRIFT: [f64; 4] = [0.01, 0.02, 0.05, 0.10];
    /// Worst relative residual of a ladder point vs the line.
    pub const RESID: [f64; 4] = [0.01, 0.02, 0.05, 0.10];
    /// Loop-only vs dithered-fit slope divergence.
    pub const CROSS: [f64; 4] = [0.02, 0.05, 0.10, 0.20];
    /// Worst relative constant change between clean attempts.
    pub const REPEAT: [f64; 4] = [0.025, 0.05, 0.10, 0.20];
}

/// Score one signal against its [`grade_thresholds`] array:
/// 0 (A) through 4 (F) — the count of cutoffs crossed.
fn grade_score(x: f64, cutoffs: [f64; 4]) -> u8 {
    cutoffs.iter().filter(|&&c| x > c).count() as u8
}

/// The environment grade: the always-printed synthesis of the
/// calibration self-checks. Signals are facts about the run;
/// `letter` is the worst signal's grade ('F' outright when the
/// published attempt carries physical violations).
///
/// - See notes/chores/chores-04.md#replanning-slope-dither-and-self-checks
///   for the design: every check runs every time, a passing
///   check is silent, and the one always-visible line carries
///   the letter plus the evidence behind it.
#[derive(Debug)]
pub struct CalGrade {
    /// Fraction of dithered samples above [`DISTURBED_MULT`] x
    /// the low-quantile floor (worst of the two points).
    pub disturbed_frac: f64,
    /// Fraction of dithered windows above [`DIRTY_WINDOW_TOL`]
    /// over the minimum window mean (worst of the two points).
    pub dirty_window_frac: f64,
    /// `|l_end - l_start| / l_start` over the loop-only bracket
    /// passes — machine-state drift across the calibration.
    pub drift_frac: f64,
    /// Worst `|residual| / value` of a ladder point against the
    /// Theil-Sen line (median-intercept anchored).
    pub max_resid_frac: f64,
    /// `|fit_slope - slope| / slope`, dithered two-point fit vs
    /// the production loop-only slope.
    pub slope_cross_frac: f64,
    /// Worst relative constant change between the last two clean
    /// attempts; `None` when fewer than two attempts came out
    /// clean (scored as C at best).
    pub repeat_rel: Option<f64>,
    /// Largest absolute change of frame/call or frame/sample
    /// between the last two clean attempts, in ns — the headline
    /// "constants repeat to ±X ns" number.
    pub repeat_ns: Option<f64>,
    /// Overall letter, worst signal wins: A, B, C, D, or F.
    pub letter: char,
}

impl CalGrade {
    /// Letters for the environment line's printed signals, in
    /// print order: disturbed, dirty windows, drift, resid,
    /// cross, repeat — all six composite inputs, so the worst
    /// letter on the line *is* the composite and every grade is
    /// self-explaining (unknown repeat floors at C).
    pub fn signal_letters(&self) -> [char; 6] {
        let repeat_score = match self.repeat_rel {
            Some(r) => grade_score(r, grade_thresholds::REPEAT),
            None => 2,
        };
        [
            score_letter(grade_score(
                self.disturbed_frac,
                grade_thresholds::DISTURBED,
            )),
            score_letter(grade_score(
                self.dirty_window_frac,
                grade_thresholds::DIRTY_WINDOWS,
            )),
            score_letter(grade_score(self.drift_frac, grade_thresholds::DRIFT)),
            score_letter(grade_score(self.max_resid_frac, grade_thresholds::RESID)),
            score_letter(grade_score(self.slope_cross_frac, grade_thresholds::CROSS)),
            score_letter(repeat_score),
        ]
    }

    /// Worst per-signal score (0=A .. 4=F), with an unknown
    /// repeatability floored at C — if two clean attempts never
    /// happened, the environment has already said something.
    fn score(&self) -> u8 {
        let repeat_score = match self.repeat_rel {
            Some(r) => grade_score(r, grade_thresholds::REPEAT),
            None => 2,
        };
        [
            grade_score(self.disturbed_frac, grade_thresholds::DISTURBED),
            grade_score(self.dirty_window_frac, grade_thresholds::DIRTY_WINDOWS),
            grade_score(self.drift_frac, grade_thresholds::DRIFT),
            grade_score(self.max_resid_frac, grade_thresholds::RESID),
            grade_score(self.slope_cross_frac, grade_thresholds::CROSS),
            repeat_score,
        ]
        .into_iter()
        .fold(0, u8::max)
    }
}

/// Map a 0..=4 score to its letter (no 'E': 4 is 'F').
fn score_letter(score: u8) -> char {
    match score {
        0 => 'A',
        1 => 'B',
        2 => 'C',
        3 => 'D',
        _ => 'F',
    }
}

/// Xorshift64* PRNG for dither lengths. No external dep; phase
/// randomization needs rough uniformity, not statistical rigor.
struct XorShift64(u64);

impl XorShift64 {
    /// Next pseudo-random u64.
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// Sub-quantum phase dither: a random 0..[`DITHER_SPAN`] neutral
/// spin, run *outside* the timed interval before each sample.
/// Used by calibration and by the harness sample seam.
pub struct Dither(XorShift64);

impl Dither {
    /// New dither source, seeded from wall-clock nanos (any
    /// per-invocation variation suffices for phase dither).
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64 | 1)
            .unwrap_or(0x9E37_79B9_7F4A_7C15); // OK: fixed fallback seed still dithers
        Dither(XorShift64(seed))
    }

    /// Spin a random 0..[`DITHER_SPAN`] iterations to re-roll the
    /// next sample's phase on the clock lattice.
    #[inline]
    pub fn spin(&mut self) {
        let r = self.0.next() % DITHER_SPAN;
        for _ in 0..r {
            black_box(1u64);
        }
    }

    /// Next raw pseudo-random u64, for callers needing coarser
    /// randomness (e.g. the harness's block sleep lengths).
    pub fn rand_u64(&mut self) -> u64 {
        self.0.next()
    }
}

impl Default for Dither {
    /// Same as [`Dither::new`].
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregates of one dithered measurement point (all ns):
/// linear statistics that keep the dither win, plus min for
/// reference against the lattice floor.
#[derive(Debug)]
pub struct DitherPoint {
    /// Full mean over all samples — unbiased under dither but
    /// absorbs interrupt spikes.
    pub mean_ns: f64,
    /// Mean of samples ≤ p99 — sheds the top 1% of samples.
    /// Retained for comparison and regime fingerprinting; too
    /// little trim to survive contention, so no longer the fit
    /// input.
    pub mean_p99_ns: f64,
    /// Mean of the fastest [`DITHER_FAST_WINDOWS`] window means.
    /// A diagnostic, not the fit input — see
    /// [`DITHER_FAST_WINDOWS`].
    pub mean_fast_ns: f64,
    /// Low quantiles, one per [`DITHER_LOW_Q`] entry: the
    /// smallest sample remaining after discarding that fraction
    /// of the fastest. Index into it with the same position.
    pub low_q_ns: [f64; DITHER_LOW_Q.len()],
    /// Minimum window mean — the amortized in-interval cost per
    /// sample, least-disturbed window. Pairs with the loop-only
    /// pass to derive `frame_sample` without extrapolating.
    pub min_window_ns: f64,
    /// Median of per-window means — robust to a bad window
    /// without snapping (window means are not lattice-valued).
    pub median_window_ns: f64,
    /// Spread (max − min) of the window means — a dispersion
    /// signal for a CI and for regime-shift detection.
    pub window_spread_ns: f64,
    /// Minimum sample — the lattice floor, for comparison.
    pub min_ns: u64,
    /// Fraction of samples above [`DISTURBED_MULT`] x the
    /// low-quantile floor — the interference census.
    pub disturbed_frac: f64,
    /// Fraction of window means above [`DIRTY_WINDOW_TOL`] over
    /// the minimum window mean — how hard clean windows were to
    /// find.
    pub dirty_window_frac: f64,
}

/// Apparatus-overhead model fitted by [`calibrate`].
#[derive(Debug)]
pub struct Overhead {
    /// Call-to-call cost of taking one sample (full timer-pair
    /// apparatus cost, clock-read latencies included), in ns —
    /// the window pass minus the loop-only pass. Sizes the
    /// experiment ([`crate::harness`]'s `pick_inner`); most of it
    /// sits *outside* recorded intervals, so it is never
    /// subtracted from reported values. Clamped at 0, with a
    /// warning, if the two passes disagree.
    pub frame_call_ns: f64,
    /// In-interval timer-pair slice a recorded sample contains,
    /// in ns — the dithered `N_LOW` window minimum minus the
    /// loop-only pass, both amortized per-sample costs over the
    /// same [`run_inner`]. Subtracted per sample (amortized by
    /// `inner`); see [`Overhead::adjust_per_call_ns`].
    pub frame_sample_ns: f64,
    /// Per-inner-iteration loop overhead (branch + `black_box`),
    /// in ns — the Theil-Sen slope across the loop-only ladder
    /// ([`Overhead::cal_loop_ladder`]). Subtracted per call.
    /// Also the frequency-regime fingerprint.
    pub loop_per_iter_ns: f64,
    /// Raw call-to-call window minimum at [`N_LOW`] (ns).
    /// Preserved for `-v` logging and cache provenance.
    pub cal_w_low_ns: f64,
    /// Raw loop-only window minimum at [`N_LOW`] (ns) — the same
    /// pass without the per-sample timer pair (the ladder's
    /// first point). `cal_w_low_ns` minus this is
    /// [`Overhead::frame_call_ns`].
    pub cal_l_low_ns: f64,
    /// The loop-only ladder: `(inner, min window mean ns/sample)`
    /// per [`LOOP_LADDER`] point. The Theil-Sen slope across it
    /// is [`Overhead::loop_per_iter_ns`]; the points feed the
    /// linearity diagnostic.
    pub cal_loop_ladder: Vec<(u64, f64)>,
    /// Raw dithered point at [`N_LOW`].
    pub cal_d_low: DitherPoint,
    /// Raw dithered point at [`N_HIGH`].
    pub cal_d_high: DitherPoint,
    /// Loop-only [`N_LOW`] floor measured first (right after
    /// warmup) and last — the drift bracket. Their relative
    /// difference is [`CalGrade::drift_frac`]; diagnostic only,
    /// no constant is derived from them.
    pub cal_l_start_ns: f64,
    /// See [`Overhead::cal_l_start_ns`].
    pub cal_l_end_ns: f64,
    /// Wall-clock duration of the full calibration run.
    pub cal_duration: Duration,
    /// Physical impossibilities detected in this result, empty
    /// when the calibration is self-consistent. Non-empty means
    /// the constants were kept only so the run can continue —
    /// report them rather than presenting the values as measured.
    pub violations: Vec<String>,
    /// Statistical self-check failures (D-or-worse signals) in
    /// plain language, empty on a healthy run. Unlike
    /// [`Overhead::violations`] these do not drive retries — they
    /// describe the environment, not a broken measurement model.
    pub warnings: Vec<String>,
    /// The environment grade synthesized from the self-check
    /// signals; finalized by [`calibrate`] once repeatability
    /// across attempts is known.
    pub grade: CalGrade,
}

impl Overhead {
    /// Per-call apparatus overhead subtracted from reported values,
    /// in ns: the amortized loop cost plus the in-interval framing
    /// slice amortized by `inner`.
    ///
    /// - `frame_sample_ns` is the slice of timer cost recorded
    ///   intervals actually contain (±~0.1 ns run-to-run within
    ///   a frequency regime), not the call-to-call cost, most of
    ///   which falls outside them.
    pub fn adjust_per_call_ns(&self, inner: u64) -> f64 {
        self.frame_sample_ns / inner as f64 + self.loop_per_iter_ns
    }
}

struct EmptyBench;

impl Bench for EmptyBench {
    fn name(&self) -> &str {
        "empty"
    }

    fn step(&mut self) -> u64 {
        black_box(1)
    }
}

/// The inner loop, as one compiled copy shared by every
/// calibration pass.
///
/// - `#[inline(never)]` is load-bearing: `frame_call_ns` is a
///   difference of two passes, so any per-call-site codegen
///   difference in this loop lands in the result at full size.
///   One copy makes it cancel instead. See
///   notes/chores/chores-04.md#call-site-codegen-and-the-frame_call-subtraction.
/// - The call itself costs one call per *sample*, not per
///   iteration, so it does not enter `loop_per_iter_ns`; it is a
///   constant present in every pass, and cancels too.
#[inline(never)]
fn run_inner(bench: &mut EmptyBench, inner: u64) {
    for _ in 0..inner {
        black_box(bench.step());
    }
}

/// Calibrate: collect [`CAL_CLEAN_ATTEMPTS`] violation-free
/// attempts (retrying dirty ones up to [`CAL_MAX_ATTEMPTS`]
/// total), publish the last clean one, and grade the environment.
///
/// - A violation (see [`Overhead::violations`]) means interference
///   corrupted the run, not that the apparatus is slow, so another
///   attempt on a quieter moment is the right response.
/// - The second clean attempt exists to measure repeatability —
///   the grade's headline number. Constants come from the last
///   clean attempt; the difference between the two is
///   [`CalGrade::repeat_ns`].
/// - When attempts run out, the best that exists is returned —
///   the last clean attempt (repeatability unknown, graded C at
///   best) or the last dirty one (graded F) — with its violations
///   attached for the caller to report. A clamped constant is
///   never passed off as a measurement.
pub fn calibrate() -> Overhead {
    let mut attempts: Vec<Overhead> = Vec::new();
    loop {
        let o = calibrate_once();
        if !o.violations.is_empty() {
            log::warn!(
                "calibration attempt {} of {CAL_MAX_ATTEMPTS} was physically impossible \
                 ({}); retrying",
                attempts.len() + 1,
                o.violations.join("; "),
            );
        }
        attempts.push(o);
        let clean = attempts.iter().filter(|a| a.violations.is_empty()).count();
        if clean >= CAL_CLEAN_ATTEMPTS || attempts.len() >= CAL_MAX_ATTEMPTS as usize {
            break;
        }
    }

    let clean_idx: Vec<usize> = (0..attempts.len())
        .filter(|&i| attempts[i].violations.is_empty())
        .collect();
    let repeat = match clean_idx.as_slice() {
        [.., a, b] => Some(repeat_between(&attempts[*a], &attempts[*b])),
        _ => None,
    };
    let pick = clean_idx.last().copied().unwrap_or(attempts.len() - 1);
    // OK: obvious — no clean attempt falls back to the last (dirty) one;
    // the loop body ran at least once, so attempts is non-empty.
    let mut o = attempts.swap_remove(pick);
    finalize_grade(&mut o, repeat);
    o
}

/// Worst relative and largest absolute (ns) constant change
/// between two clean attempts, as `(rel, ns)`.
///
/// - Relative covers all three constants (each against its own
///   scale); absolute covers the two ns-scale framing constants,
///   the "constants repeat to ±X ns" headline.
fn repeat_between(a: &Overhead, b: &Overhead) -> (f64, f64) {
    let rel = [
        (a.frame_call_ns, b.frame_call_ns),
        (a.frame_sample_ns, b.frame_sample_ns),
        (a.loop_per_iter_ns, b.loop_per_iter_ns),
    ]
    .into_iter()
    .map(|(x, y)| (x - y).abs() / y.abs().max(1e-9))
    .fold(0.0, f64::max);
    let ns = (a.frame_call_ns - b.frame_call_ns)
        .abs()
        .max((a.frame_sample_ns - b.frame_sample_ns).abs());
    (rel, ns)
}

/// Fill in the published attempt's repeatability, final letter,
/// and plain-language warnings for every D-or-worse signal.
fn finalize_grade(o: &mut Overhead, repeat: Option<(f64, f64)>) {
    if let Some((rel, ns)) = repeat {
        o.grade.repeat_rel = Some(rel);
        o.grade.repeat_ns = Some(ns);
    }
    o.grade.letter = if o.violations.is_empty() {
        score_letter(o.grade.score())
    } else {
        'F'
    };

    let g = &o.grade;
    let mut warn = |bad: bool, msg: String| {
        if bad {
            o.warnings.push(msg);
        }
    };
    warn(
        grade_score(g.disturbed_frac, grade_thresholds::DISTURBED) >= 3,
        format!(
            "environment noisy: {:.1}% of calibration samples were disturbed by \
             interference",
            g.disturbed_frac * 100.0
        ),
    );
    warn(
        grade_score(g.dirty_window_frac, grade_thresholds::DIRTY_WINDOWS) >= 3,
        format!(
            "environment noisy: {:.0}% of measurement windows ran slow (a continuous \
             competitor or heavy load)",
            g.dirty_window_frac * 100.0
        ),
    );
    warn(
        grade_score(g.drift_frac, grade_thresholds::DRIFT) >= 3,
        format!(
            "machine speed changed {:.1}% during calibration (frequency or thermal \
             shift); paired constants may not match each other",
            g.drift_frac * 100.0
        ),
    );
    warn(
        grade_score(g.max_resid_frac, grade_thresholds::RESID) >= 3,
        format!(
            "loop ladder deviates {:.1}% from the linear model; the slope is suspect",
            g.max_resid_frac * 100.0
        ),
    );
    warn(
        grade_score(g.slope_cross_frac, grade_thresholds::CROSS) >= 3,
        format!(
            "independent slope estimates differ {:.1}%; the calibration model is \
             suspect",
            g.slope_cross_frac * 100.0
        ),
    );
    match g.repeat_rel {
        Some(r) => warn(
            grade_score(r, grade_thresholds::REPEAT) >= 3,
            format!(
                "calibration constants differ {:.1}% between attempts; treat results \
                 as unreliable",
                r * 100.0
            ),
        ),
        None => warn(
            true,
            "repeatability unknown: fewer than two calibration attempts came out \
             clean"
                .to_string(),
        ),
    }
}

/// One calibration pass: the loop-only ladder, the call-to-call
/// window pass, the two dithered points, and a drift bracket
/// (loop-only [`N_LOW`] floor first and last). Blocks for
/// ~0.5-1 s release (several seconds unoptimized) on a typical
/// modern x86; logs raw points and the alternative fits at debug
/// level. [`calibrate`] runs it at least twice.
fn calibrate_once() -> Overhead {
    let mut bench = EmptyBench;
    let mut dither = Dither::new();
    let cal_start = Instant::now();
    for _ in 0..CAL_WARMUP {
        black_box(bench.step());
    }

    // Drift bracket, opening side: every paired difference below
    // assumes machine state holds across the pair, and a debug
    // run has been observed breaking that (a frequency-regime
    // shift mid-calibration drove frame/sample to 88 ns against
    // frame/call 39). The same measurement repeated first and
    // last makes the assumption checkable.
    let l_start = measure_loop_only(&mut bench, W_LOW_WINDOWS, W_LOW_SAMPLES, N_LOW);

    // d_low shares the loop-only pass's window shape: its
    // min_window_ns is differenced against l_low below, and
    // differenced passes need identical window shapes (count and
    // duration) or their min-over-windows aren't comparable.
    let d_low = dither_measure(&mut dither, W_LOW_WINDOWS, W_LOW_SAMPLES, N_LOW);
    let d_high = dither_measure(&mut dither, DITHER_WINDOWS, DITHER_HIGH_SAMPLES, N_HIGH);
    log_dither_point("d_low", &d_low);
    log_dither_point("d_high", &d_high);
    log_alt_fits(&d_low, &d_high);

    // Call-to-call: two window passes at N_LOW differing only in
    // the per-sample timer pair, so the loop term cancels in the
    // subtraction rather than being estimated from the dithered
    // slope (a different call site, and in an unoptimized build a
    // ~12% different loop — see
    // notes/chores/chores-04.md#call-site-codegen-and-the-frame_call-subtraction).
    let w_low = measure_window(&mut bench, W_LOW_WINDOWS, W_LOW_SAMPLES, N_LOW);
    let cal_loop_ladder: Vec<(u64, f64)> = LOOP_LADDER
        .iter()
        .map(|&n| {
            (
                n,
                measure_loop_only(&mut bench, W_LOW_WINDOWS, loop_samples(n), n),
            )
        })
        .collect();
    let l_low = cal_loop_ladder[0].1;
    let frame_call_raw = w_low - l_low;

    // Production slope: Theil-Sen across the loop-only ladder —
    // no per-sample timer reads, so no lattice to de-bias, and
    // the median of pairwise slopes resists a contaminated point.
    let loop_per_iter_ns = theil_sen_slope(&cal_loop_ladder);
    for &(n, per_sample) in &cal_loop_ladder {
        log::debug!(
            "loop ladder: N={n:<6} {per_sample:.4} ns/sample ({:.6} ns/iter, \
             resid={:+.4} ns vs slope from l_low)",
            per_sample / n as f64,
            per_sample - (l_low + (n - N_LOW) as f64 * loop_per_iter_ns),
        );
    }

    // The dithered two-point fit, demoted to a diagnostic: its
    // slope diverging from the loop-only slope is the check on
    // the assumption that the loop costs the same with and
    // without timer pairs interleaved.
    let (fit_intercept, fit_slope) = two_point_fit(
        d_low.low_q_ns[DITHER_PROD_Q],
        d_high.low_q_ns[DITHER_PROD_Q],
    );
    log::debug!(
        "slope cross-check: loop-only={loop_per_iter_ns:.6} ns/iter, \
         dithered-fit={fit_slope:.6} ns/iter"
    );

    // frame_sample is *not* taken from that intercept: it is a
    // difference of two same-N measurements over the same
    // run_inner, so no long point can lever it negative. Both
    // terms are amortized per-sample costs, min over windows.
    let frame_sample_raw = d_low.min_window_ns - l_low;
    log::debug!(
        "frame_sample: paired={frame_sample_raw:.4} ns (d_low_minwin={:.4}, l_low={l_low:.4}), \
         fit-intercept={fit_intercept:.4} ns",
        d_low.min_window_ns,
    );
    log::debug!(
        "frame_call: paired={frame_call_raw:.4} ns (w_low={w_low:.4}, l_low={l_low:.4}), \
         slope-based={:.4} ns",
        w_low - N_LOW as f64 * loop_per_iter_ns,
    );

    // Physical plausibility. These are not statistical tests: each
    // one is a statement that cannot be false of a real apparatus,
    // so a failure means the measurement is invalid, not unlucky.
    let mut violations = Vec::new();
    if loop_per_iter_ns <= 0.0 {
        violations.push(format!(
            "loop/iter is non-positive ({loop_per_iter_ns:.6} ns): an iteration \
             cannot cost nothing"
        ));
    }
    if frame_call_raw <= 0.0 {
        violations.push(format!(
            "frame/call is non-positive ({frame_call_raw:.4} ns; w_low={w_low:.4}, \
             l_low={l_low:.4}): taking a sample cannot cost nothing"
        ));
    }
    if frame_sample_raw < 0.0 {
        violations.push(format!(
            "frame/sample is negative ({frame_sample_raw:.4} ns; d_low_minwin={:.4}, \
             l_low={l_low:.4}): a timed interval cannot be shorter than the loop it contains",
            d_low.min_window_ns,
        ));
    }
    if frame_sample_raw > frame_call_raw {
        violations.push(format!(
            "frame/sample ({frame_sample_raw:.4} ns) exceeds frame/call \
             ({frame_call_raw:.4} ns): the timer cost inside the interval cannot \
             exceed the whole timer cost"
        ));
    }

    let frame_call_ns = frame_call_raw.max(0.0);
    let frame_sample_ns = frame_sample_raw.max(0.0);

    // Drift bracket, closing side, then the statistical
    // self-check signals. These are graded, not asserted: unlike
    // the violations above, a bad value describes the
    // environment, not an impossible measurement.
    let l_end = measure_loop_only(&mut bench, W_LOW_WINDOWS, W_LOW_SAMPLES, N_LOW);
    let drift_frac = (l_end - l_start).abs() / l_start.max(1e-9);

    // Linearity: residuals of the ladder points against the
    // Theil-Sen line, anchored at the median intercept so no one
    // point (not even l_low) is privileged.
    let mut intercepts: Vec<f64> = cal_loop_ladder
        .iter()
        .map(|&(n, p)| p - loop_per_iter_ns * n as f64)
        .collect();
    intercepts.sort_unstable_by(|a, b| a.total_cmp(b));
    let intercept = intercepts[intercepts.len() / 2];
    let max_resid_frac = cal_loop_ladder
        .iter()
        .map(|&(n, p)| {
            let predicted = intercept + loop_per_iter_ns * n as f64;
            (p - predicted).abs() / p.max(1e-9)
        })
        .fold(0.0, f64::max);

    let slope_cross_frac = (fit_slope - loop_per_iter_ns).abs() / loop_per_iter_ns.abs().max(1e-9);

    let grade = CalGrade {
        disturbed_frac: d_low.disturbed_frac.max(d_high.disturbed_frac),
        dirty_window_frac: d_low.dirty_window_frac.max(d_high.dirty_window_frac),
        drift_frac,
        max_resid_frac,
        slope_cross_frac,
        repeat_rel: None,
        repeat_ns: None,
        letter: 'F', // placeholder; finalize_grade() sets it
    };
    log::debug!(
        "self-checks: disturbed={:.4} dirty_win={:.4} drift={:.4} resid={:.4} cross={:.4} \
         (l_start={l_start:.4}, l_end={l_end:.4})",
        grade.disturbed_frac,
        grade.dirty_window_frac,
        grade.drift_frac,
        grade.max_resid_frac,
        grade.slope_cross_frac,
    );

    Overhead {
        frame_call_ns,
        frame_sample_ns,
        loop_per_iter_ns,
        cal_w_low_ns: w_low,
        cal_l_low_ns: l_low,
        cal_loop_ladder,
        cal_d_low: d_low,
        cal_d_high: d_high,
        cal_l_start_ns: l_start,
        cal_l_end_ns: l_end,
        cal_duration: cal_start.elapsed(),
        violations,
        warnings: Vec::new(),
        grade,
    }
}

/// Measure one dithered point: `windows` windows of `samples`
/// samples at the given `inner`, each sample preceded by a random
/// sub-quantum [`Dither::spin`] outside the timed interval.
fn dither_measure(dither: &mut Dither, windows: u64, samples: u64, inner: u64) -> DitherPoint {
    let mut bench = EmptyBench;
    let mut all: Vec<u64> = Vec::with_capacity((windows * samples) as usize);
    let mut window_means: Vec<f64> = Vec::with_capacity(windows as usize);
    for _ in 0..windows {
        let mut sum: u128 = 0;
        for _ in 0..samples {
            dither.spin();
            let start = Instant::now();
            run_inner(&mut bench, inner);
            let e = start.elapsed().as_nanos() as u64;
            sum += u128::from(e);
            all.push(e);
        }
        window_means.push(sum as f64 / samples as f64);
    }

    all.sort_unstable();
    let n = all.len();
    let mean_ns = all.iter().map(|&v| v as f64).sum::<f64>() / n as f64;
    let n99 = (n as f64 * 0.99).ceil() as usize;
    let mean_p99_ns = all[..n99].iter().map(|&v| v as f64).sum::<f64>() / n99 as f64;

    window_means.sort_unstable_by(|a, b| a.total_cmp(b));
    let median_window_ns = window_means[window_means.len() / 2];
    let window_spread_ns = window_means[window_means.len() - 1] - window_means[0];
    let fast = (DITHER_FAST_WINDOWS as usize).clamp(1, window_means.len());
    let mean_fast_ns = window_means[..fast].iter().sum::<f64>() / fast as f64;

    let mut low_q_ns = [0.0; DITHER_LOW_Q.len()];
    for (slot, frac) in low_q_ns.iter_mut().zip(DITHER_LOW_Q) {
        let idx = ((n as f64 * frac) as usize).min(n - 1);
        *slot = all[idx] as f64;
    }

    // The interference census, over the sorted samples: everything
    // above the census line was hit. See DISTURBED_ABS_NS for why
    // the bound has an absolute part.
    let floor = low_q_ns[DITHER_PROD_Q];
    let disturbed_bound = (floor * DISTURBED_MULT).max(floor + DISTURBED_ABS_NS);
    let first_disturbed = all.partition_point(|&v| (v as f64) <= disturbed_bound);
    let disturbed_frac = (n - first_disturbed) as f64 / n as f64;

    // Same census at window granularity (window_means is sorted).
    let dirty_bound = window_means[0] * (1.0 + DIRTY_WINDOW_TOL);
    let first_dirty = window_means.partition_point(|&w| w <= dirty_bound);
    let dirty_window_frac = (window_means.len() - first_dirty) as f64 / window_means.len() as f64;

    DitherPoint {
        mean_ns,
        mean_p99_ns,
        mean_fast_ns,
        low_q_ns,
        min_window_ns: window_means[0],
        median_window_ns,
        window_spread_ns,
        min_ns: all[0],
        disturbed_frac,
        dirty_window_frac,
    }
}

/// Amortized per-sample cost at a given `inner`: each window times
/// `samples` complete samples (timer pair around an `inner`-iteration
/// loop — the exact shape the harness takes one at a time) and
/// divides by the count, so quantization error is q/samples; the
/// minimum over `windows` windows is returned, in ns.
fn measure_window(bench: &mut EmptyBench, windows: u64, samples: u64, inner: u64) -> f64 {
    let mut min_ns = f64::INFINITY;
    for _ in 0..windows {
        let window = Instant::now();
        for _ in 0..samples {
            let start = Instant::now();
            run_inner(bench, inner);
            black_box(start.elapsed());
        }
        let per_sample = window.elapsed().as_nanos() as f64 / samples as f64;
        if per_sample < min_ns {
            min_ns = per_sample;
        }
    }
    min_ns
}

/// [`measure_window`] with the per-sample timer pair removed: the
/// loop's own cost per `inner` iterations, same window shape, same
/// min-over-windows estimator, same [`run_inner`].
///
/// - Paired with [`measure_window`], the difference is the
///   call-to-call framing cost with the loop term cancelled
///   exactly — no fitted slope, no second call site.
fn measure_loop_only(bench: &mut EmptyBench, windows: u64, samples: u64, inner: u64) -> f64 {
    let mut min_ns = f64::INFINITY;
    for _ in 0..windows {
        let window = Instant::now();
        for _ in 0..samples {
            run_inner(bench, inner);
        }
        let per_sample = window.elapsed().as_nanos() as f64 / samples as f64;
        if per_sample < min_ns {
            min_ns = per_sample;
        }
    }
    min_ns
}

/// Theil-Sen slope over `(inner, per-sample ns)` ladder points:
/// the median of all pairwise slopes, in ns per iteration.
///
/// - Robust to a contaminated point: with 5 ladder points (10
///   pairs), up to 4 pairs can be corrupted before the median
///   moves — a mean-based fit moves on the first.
/// - The per-sample constant term (the `run_inner` call, window
///   framing) cancels in every pairwise difference, so only the
///   per-iteration cost survives.
fn theil_sen_slope(points: &[(u64, f64)]) -> f64 {
    let mut slopes: Vec<f64> = Vec::with_capacity(points.len() * (points.len() - 1) / 2);
    for (i, &(n_i, p_i)) in points.iter().enumerate() {
        for &(n_j, p_j) in &points[i + 1..] {
            slopes.push((p_j - p_i) / (n_j - n_i) as f64);
        }
    }
    slopes.sort_unstable_by(|a, b| a.total_cmp(b));
    let mid = slopes.len() / 2;
    if slopes.len() % 2 == 1 {
        slopes[mid]
    } else {
        (slopes[mid - 1] + slopes[mid]) / 2.0
    }
}

/// Two-point linear fit over the [`N_LOW`] / [`N_HIGH`] pair,
/// shared by the diagnostic fits (the alternative-fit logging and
/// the `calibrate` command's diagnostic output). Returns
/// `(intercept_ns, slope_ns)`; the slope clamps to 0 when the
/// points invert, and the intercept is left unclamped (a negative
/// value is itself a diagnostic signal).
pub fn two_point_fit(low_ns: f64, high_ns: f64) -> (f64, f64) {
    let slope = if high_ns > low_ns {
        (high_ns - low_ns) / (N_HIGH - N_LOW) as f64
    } else {
        0.0
    };
    (low_ns - N_LOW as f64 * slope, slope)
}

/// Debug-log one dithered point's aggregates.
fn log_dither_point(name: &str, p: &DitherPoint) {
    log::debug!(
        "dither {name}: mean={:.4} p99mean={:.4} fast={:.4} medwin={:.4} spread={:.4} min={} ns",
        p.mean_ns,
        p.mean_p99_ns,
        p.mean_fast_ns,
        p.median_window_ns,
        p.window_spread_ns,
        p.min_ns,
    );
}

/// Debug-log the diagnostic two-point fits (every
/// [`fit_candidates`] entry) so estimator agreement stays
/// observable run to run — their divergence from each other and
/// from the loop-only slope is a contamination signal.
fn log_alt_fits(d_low: &DitherPoint, d_high: &DitherPoint) {
    for (kind, low, high) in fit_candidates(d_low, d_high) {
        let (intercept, slope) = two_point_fit(low, high);
        log::debug!(
            "dither fit({kind}): in-interval framing={intercept:.4} ns, loop_per_iter={slope:.6} ns"
        );
    }
}

/// Every aggregation the two points can be fitted through, as
/// `(label, low, high)` — the single list behind both the `-v`
/// debug log and the `calibrate` command's stdout block, so the
/// two can't drift.
///
/// - The means come first, then one entry per [`DITHER_LOW_Q`]
///   discard fraction. All are diagnostics: the production slope
///   is the loop-only Theil-Sen, and the cross-check reads the
///   `lowq` entry at [`DITHER_PROD_Q`].
pub fn fit_candidates(d_low: &DitherPoint, d_high: &DitherPoint) -> Vec<(String, f64, f64)> {
    let mut out = vec![
        ("full".to_string(), d_low.mean_ns, d_high.mean_ns),
        ("p99".to_string(), d_low.mean_p99_ns, d_high.mean_p99_ns),
        ("fast".to_string(), d_low.mean_fast_ns, d_high.mean_fast_ns),
        (
            "medwin".to_string(),
            d_low.median_window_ns,
            d_high.median_window_ns,
        ),
    ];
    for (i, frac) in DITHER_LOW_Q.iter().enumerate() {
        out.push((
            format!("lowq{:.1}%", frac * 100.0),
            d_low.low_q_ns[i],
            d_high.low_q_ns[i],
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theil_sen_is_median_of_pairwise_slopes() {
        // Perfect line: every pairwise slope is 2.0.
        let line: Vec<(u64, f64)> = vec![(100, 210.0), (300, 610.0), (1000, 2010.0)];
        assert!((theil_sen_slope(&line) - 2.0).abs() < 1e-12);

        // One wildly contaminated point out of five leaves the
        // median pairwise slope on the clean value.
        let dirty: Vec<(u64, f64)> = vec![
            (100, 210.0),
            (300, 610.0),
            (1000, 9999.0), // contaminated
            (3000, 6010.0),
            (10000, 20010.0),
        ];
        assert!((theil_sen_slope(&dirty) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn grade_score_counts_crossed_cutoffs() {
        let cutoffs = [0.01, 0.02, 0.05, 0.10];
        assert_eq!(grade_score(0.0, cutoffs), 0);
        assert_eq!(grade_score(0.01, cutoffs), 0); // boundary is inclusive-below
        assert_eq!(grade_score(0.015, cutoffs), 1);
        assert_eq!(grade_score(0.04, cutoffs), 2);
        assert_eq!(grade_score(0.07, cutoffs), 3);
        assert_eq!(grade_score(0.5, cutoffs), 4);
    }

    #[test]
    fn grade_letter_maps_and_worst_signal_wins() {
        assert_eq!(score_letter(0), 'A');
        assert_eq!(score_letter(4), 'F');
        let mut g = CalGrade {
            disturbed_frac: 0.0,
            dirty_window_frac: 0.0,
            drift_frac: 0.0,
            max_resid_frac: 0.0,
            slope_cross_frac: 0.0,
            repeat_rel: Some(0.0),
            repeat_ns: Some(0.0),
            letter: 'F',
        };
        assert_eq!(score_letter(g.score()), 'A');
        g.drift_frac = 0.06; // crosses A, B, C cutoffs -> D
        assert_eq!(score_letter(g.score()), 'D');
        g.drift_frac = 0.0;
        g.repeat_rel = None; // unknown repeatability floors at C
        assert_eq!(score_letter(g.score()), 'C');
    }
}
