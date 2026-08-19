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
    /// Fraction of batches whose mean sits [`super::BURST_TOL`] above the
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
    ///   [`super::EnvGrade`]'s note on what the probe cannot see.
    pub const INTERFERENCE: [f64; 4] = [0.001, 0.005, 0.02, 0.05];
    /// Floor movement from the first quarter of warmup to the
    /// last — the frequency-ramp detector.
    pub const DRIFT: [f64; 4] = [0.01, 0.02, 0.05, 0.10];
    /// Largest floor shift across any split of the warmup window.
    pub const STEP: [f64; 4] = [0.01, 0.02, 0.05, 0.10];
    /// Unsettled share of the warm (1 minus the settle cell's settled share): the settle
    /// signal's cutoffs, so the letters read A at >=25% settled, B at >=10%, C at >=5%, D
    /// below that, and F within 2% of never (never itself included).
    ///
    /// - **Provisional**, anchored on the floor: the earliest certifiable settle is the exit
    ///   window, ~3% of a default warm, and a box that repeatedly lands there settled at the
    ///   buzzer and would not have settled on any smaller budget, so the floor lands D. A box
    ///   that spent a quarter of its warm settled is comfortably certified (a healthy 7600x
    ///   ramp reads ~49%).
    pub const UNSETTLED: [f64; 4] = [0.75, 0.90, 0.95, 0.98];
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
///   - `bursts` — **dropped.** The stretch carries too few
///     probes for a hot-fraction to say much (granular to 1/16
///     back when the probe count was a fixed 16), and the
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

/// The clock's journey through the warmup stretch: where it started, the state it settled
/// into (the earliest suffix that grades A on timing *and* held its delivered clock still),
/// how long that state held before measurement began, and how still it held. `None` when
/// there is nothing to measure (an empty stretch or a `tail_len` the stretch cannot hold).
///
/// - **Why report it at all.** Once warmup deliberately spans the ramp, the grade stops seeing
///   the ramp, which is warmup working, and would otherwise leave the report saying nothing
///   about a box that took a second to come up to speed. The letter answers "was it settled
///   when measurement started"; this answers "at what state, and settled for how long".
/// - **Settled means "graded A from here on, at one clock"**: the timing test is the letter's
///   own computation, and the clock gate is [`filtered_clock_stable`], the same 1% band as
///   the warm exit's [`crate::freq::clock_stable`] but ranging over the dominant core's
///   readable samples rather than bailing on gaps. Timing alone was the whole cell until
///   0.26.0-1, and a box parked flat on its un-boosted base clock graded A immediately, so
///   the 3900X under `powersave` read `0.01s` and then transitioned mid-bench to an F
///   (2026-08-15).
/// - **The named state is the point.** A settle time alone cannot distinguish "settled at the
///   top" from "parked at base clock", and the parked case is the one that scatters later, so
///   the cell carries the settled suffix's median GHz whenever the driver exposed one.
/// - **Every clock statistic reads the dominant CPU only** ([`dominant_cpu_only`]): on an
///   unpinned run the sampler reads whichever CPU the scheduler placed the thread on, and the
///   mixed series rates scheduler placement rather than the clock (measured 2026-08-18:
///   +-11.9% mixed against +-0.2% for the same box pinned). The gate's own
///   missing-sample fallback still applies to the filtered series.
/// - `clock` is the delivered-frequency series sampled beside `warm`, index-parallel
///   ([`crate::freq::FreqSample`] per probe); a shorter series is aligned at the end, and
///   missing samples fall back to timing-only inside the gate.
/// - `tail_len` is the exit window's probe count ([`ProbeSummary`] count, the caller's
///   `RunOutput::warm_tail`): the shortest suffix the exit condition certified. The scan never
///   considers a shorter one, so a settled hold is always backed by at least the window the
///   letter was scored over, and a settled exit always finds one ([`Settle::Never`] can only
///   reach the report on a cap exit, both gates having passed on the exit window itself).
pub fn settle(
    warm: &[ProbeSummary],
    clock: &[Option<crate::freq::FreqSample>],
    tail_len: usize,
) -> Option<Settle> {
    if warm.is_empty() || tail_len == 0 || tail_len > warm.len() {
        return None;
    }
    let clock = dominant_cpu_only(clock);
    let start_ghz = clock.iter().flatten().next().map(|f| f.khz as f64 / 1e6);
    let offset = clock.len().saturating_sub(warm.len());
    let end_s = warm[warm.len() - 1].t_start_s;
    for start in 0..=(warm.len() - tail_len) {
        let clock_suffix = &clock[(offset + start).min(clock.len())..];
        // On a box with a readable clock, a suffix with no evidence does not certify: a
        // settled claim the clock cannot verify is worth less than a verified bad one, and
        // it was grading better (an evidence-free `18%` read B beside a verified `04%`'s D,
        // wink, 2026-08-19). Only a wholly clock-less box falls back to timing alone.
        let certified = match filtered_clock_stable(clock_suffix) {
            Some(stable) => stable,
            None => start_ghz.is_none(),
        };
        if certified && EnvGrade::from_probes(&warm[start..]).is_some_and(|g| g.letter == 'A') {
            let t_s = warm[start].t_start_s;
            return Some(Settle::At {
                t_s,
                // A one-probe stretch has no span to divide by, and settled at its only
                // probe is settled throughout.
                settled_frac: if end_s > 0.0 {
                    (end_s - t_s) / end_s
                } else {
                    1.0
                },
                start_ghz,
                ghz: median_ghz(clock_suffix),
                rating: rel_stdev(clock_suffix),
            });
        }
    }
    let tail_clock = &clock[clock.len().saturating_sub(tail_len)..];
    Some(Settle::Never {
        start_ghz,
        // Where the journey had got to: the exit window's median, or the last readable
        // sample when the thread was sampled elsewhere through the whole window.
        end_ghz: median_ghz(tail_clock)
            .or_else(|| clock.iter().flatten().last().map(|f| f.khz as f64 / 1e6)),
    })
}

/// The settle scan's clock gate over the dominant-CPU series: the readable samples' range
/// inside [`crate::freq::FREQ_STABLE_TOL`], skipping the gaps, `None` when the window holds
/// fewer than two readable samples, since one reading has zero range and can attest nothing
/// about stillness (the caller decides what insufficient evidence means for its box).
/// `clock_stable`'s missing-sample fallback is wrong here: post-filter a `None` sample means
/// the thread was sampled on another core at that probe, and the readable samples still form
/// an honest same-core series. Bailing on the gaps disabled the gate for every unpinned run,
/// measured as a `+-7.0%` rating inside a 1% gate (2026-08-18).
fn filtered_clock_stable(clock: &[Option<crate::freq::FreqSample>]) -> Option<bool> {
    let mut min = u64::MAX;
    let mut max = 0u64;
    let mut readable = 0usize;
    for f in clock.iter().flatten() {
        min = min.min(f.khz);
        max = max.max(f.khz);
        readable += 1;
    }
    if readable < 2 || max == 0 {
        return None;
    }
    Some((max - min) as f64 / max as f64 <= crate::freq::FREQ_STABLE_TOL)
}

/// The series reduced to its dominant CPU: samples from other CPUs become `None`, positions
/// preserved so probe-index alignment survives. On an unpinned run the sampler reads whichever
/// CPU the scheduler placed the thread on, so the raw series is a tour of placements and its
/// statistics rate the scheduler rather than the clock. The most-sampled core's readings are
/// the one honest per-core story the series holds. Ties break to the lowest CPU id.
fn dominant_cpu_only(
    clock: &[Option<crate::freq::FreqSample>],
) -> Vec<Option<crate::freq::FreqSample>> {
    let mut counts: std::collections::BTreeMap<usize, usize> = std::collections::BTreeMap::new();
    for f in clock.iter().flatten() {
        *counts.entry(f.cpu).or_insert(0) += 1;
    }
    let dominant = counts
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&cpu, _)| cpu);
    clock
        .iter()
        .copied()
        .map(|s| s.filter(|f| Some(f.cpu) == dominant))
        .collect()
}

/// Median delivered frequency (GHz) of a clock-sample window, `None` when no sample in it was
/// readable. The median rather than an endpoint, so one odd read cannot name the state.
fn median_ghz(clock: &[Option<crate::freq::FreqSample>]) -> Option<f64> {
    let mut khz: Vec<u64> = clock.iter().flatten().map(|f| f.khz).collect();
    if khz.is_empty() {
        return None;
    }
    khz.sort_unstable();
    Some(khz[khz.len() / 2] as f64 / 1_000_000.0)
}

/// Relative standard deviation (stdev over mean) of a clock window's readable samples, `None`
/// when none is readable: the settled state's steadiness rating, "how still is still" inside
/// the 1% stability gate. Under a pin it reads zero, and a wandering `powersave` box shows a
/// visibly fatter band.
fn rel_stdev(clock: &[Option<crate::freq::FreqSample>]) -> Option<f64> {
    let khz: Vec<f64> = clock.iter().flatten().map(|f| f.khz as f64).collect();
    if khz.is_empty() {
        return None;
    }
    let mean = khz.iter().sum::<f64>() / khz.len() as f64;
    if mean <= 0.0 {
        return None;
    }
    let var = khz.iter().map(|k| (k - mean).powi(2)).sum::<f64>() / khz.len() as f64;
    Some(var.sqrt() / mean)
}

/// What [`settle`] found: the clock's journey through the warmup stretch, ending either at the
/// earliest suffix that read settled (timing A with the clock held still) or still moving when
/// the stretch ended.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Settle {
    /// The earliest settled suffix: the journey that reached it, how much of the warm was
    /// settled, and how still the settled state held.
    At {
        /// Seconds from warmup start to the suffix's first probe: the ramp's end. The record's
        /// `settle_s` and never the cell's, whose number is `settled_frac`: a box already at
        /// speed reads `t_s` ~0, the first probe's timestamp, a measurement floor that dressed
        /// as a duration confused every reader it met (wink, 2026-08-18).
        t_s: f64,
        /// The settled share of the warm stretch, 0..1: the suffix's first probe to the
        /// stretch's last, over the whole stretch. The cell's number, as a percent: `100%` is
        /// a box settled throughout, small is one that settled at the last moment (the floor
        /// is the exit window as a share of the warm), and `not settled` maps to `0%`, since
        /// no share of the warm was certified. An absolute hold time was tried first and read
        /// as noise beside the warm budget (wink, 2026-08-18).
        settled_frac: f64,
        /// The journey's start: the series' first readable sample (GHz), `None` when the whole
        /// series was unreadable.
        start_ghz: Option<f64>,
        /// The settled state: the suffix's median delivered frequency (GHz), `None` when
        /// unreadable.
        ghz: Option<f64>,
        /// The settled state's steadiness ([`rel_stdev`] of the suffix), `None` when the clock
        /// was unreadable. On a fast exit the suffix is only the exit window, a handful of
        /// samples, so this is indicative rather than a precision instrument.
        rating: Option<f64>,
    },
    /// No suffix down to the exit window read settled: the journey it was still on when the
    /// stretch ended, first readable sample to the exit window's median.
    Never {
        /// The journey's start (GHz), `None` when unreadable.
        start_ghz: Option<f64>,
        /// Where the journey had got to: the exit window's median (GHz), `None` when
        /// unreadable.
        end_ghz: Option<f64>,
    },
}

impl std::fmt::Display for Settle {
    /// The grade block's `settle` cell: the clock's journey, parsed back by
    /// `qualify-environment` (format and parser move together):
    ///
    /// - `4.84->5.24GHz 49% +-0.0%`: start, settled state, the settled share of the warm, and
    ///   how still the settled state held. A journey that went nowhere still prints its arrow
    ///   (`4.09->4.09GHz`), keeping the column uniform, a box settled throughout reads
    ///   `100%`, and the percent is zero-padded to two digits so the column's digits align
    /// - `3.60->4.20GHz 00%`: never settled, the journey still under way when warmup gave up,
    ///   and no share of the warm certified. `00%` is reserved for it: a settled share rounds
    ///   up to at least `01%`
    /// - clock unreadable: the timing-only forms `49%`, `00%`
    ///
    /// The settle *letter* ([`settle_letter`]) is not part of this Display: the report
    /// appends it beside the cell like every other signal's letter.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Settle::At {
                settled_frac,
                ghz: Some(g),
                start_ghz,
                rating,
                ..
            } => {
                if let Some(s) = start_ghz {
                    write!(f, "{s:.2}->")?;
                }
                write!(
                    f,
                    "{g:.2}GHz {:02}%{}",
                    settled_pct(*settled_frac),
                    rating_suffix(*rating)
                )
            }
            Settle::At {
                settled_frac,
                ghz: None,
                ..
            } => write!(f, "{:02}%", settled_pct(*settled_frac)),
            Settle::Never {
                start_ghz: Some(s),
                end_ghz: Some(e),
            } => write!(f, "{s:.2}->{e:.2}GHz 00%"),
            Settle::Never { .. } => write!(f, "00%"),
        }
    }
}

/// The settled share as a whole percent, floored at 1 so a settled cell can never collide
/// with `00%`, which is reserved for never-settled.
fn settled_pct(frac: f64) -> u32 {
    ((frac * 100.0).round() as u32).clamp(1, 100)
}

/// The settle signal's letter: the unsettled share of the warm scored against
/// [`env_thresholds::UNSETTLED`], so a buzzer-beater settle reads D and never-settled reads
/// F. Printed by the report beside the settle cell, and folded into the warmup row's `worst`:
/// this is the one place the clock decides a letter, because a fast late ramp can finish
/// inside the bench's first batches where no timing detector sees it, and the settle scan is
/// then the only witness that the box was not fit (wink, 2026-08-19).
pub fn settle_letter(s: &Settle) -> char {
    let unsettled = match s {
        Settle::At { settled_frac, .. } => 1.0 - settled_frac,
        Settle::Never { .. } => 1.0,
    };
    score_letter(score(unsettled, env_thresholds::UNSETTLED))
}

/// The cell's steadiness suffix (` +-0.1%`), empty when the clock was unreadable. Shared with
/// the report's `-v` clock line, so the two spell the rating identically.
pub(crate) fn rating_suffix(rating: Option<f64>) -> String {
    match rating {
        Some(r) => format!(" +-{:.1}%", r * 100.0),
        None => String::new(),
    }
}

/// The warmup clock series compressed for the `-v` profile line: extremes plus one tick per
/// step between readable samples, the stability gate's own view of the series.
///
/// - `^`/`v` is a step the stability band would flag, `-` a hold within it. The deadband is
///   [`crate::freq::FREQ_STABLE_TOL`], shared with `clock_stable`, so the settled suffix reads
///   all `-` by construction and settle's start is visible as the point the ticks go quiet.
/// - The tick characters are typeable by design: hard rule 8 bars arrow glyphs from
///   user-visible strings.
/// - An unreadable sample produces no tick of its own: the step bridges to the next readable
///   sample, so a gappy series shortens the line rather than inventing holds.
#[derive(Debug, Clone, PartialEq)]
pub struct ClockProfile {
    /// Lowest readable sample (GHz).
    pub min_ghz: f64,
    /// Highest readable sample (GHz).
    pub max_ghz: f64,
    /// One tick per step between readable samples: `^` up, `v` down, `-` hold.
    pub ticks: String,
}

/// Compress `clock` into its [`ClockProfile`], `None` when no sample is readable. Reads the
/// dominant CPU only ([`dominant_cpu_only`]), like every other clock statistic.
pub fn clock_profile(clock: &[Option<crate::freq::FreqSample>]) -> Option<ClockProfile> {
    let clock = dominant_cpu_only(clock);
    let khz: Vec<f64> = clock.iter().flatten().map(|f| f.khz as f64).collect();
    let first = *khz.first()?;
    let (min, max) = khz
        .iter()
        .fold((first, first), |(lo, hi), &k| (lo.min(k), hi.max(k)));
    let ticks = khz
        .windows(2)
        .map(|w| {
            let step = if w[0] > 0.0 {
                (w[1] - w[0]) / w[0]
            } else {
                0.0
            };
            if step > crate::freq::FREQ_STABLE_TOL {
                '^'
            } else if step < -crate::freq::FREQ_STABLE_TOL {
                'v'
            } else {
                '-'
            }
        })
        .collect();
    Some(ClockProfile {
        min_ghz: min / 1e6,
        max_ghz: max / 1e6,
        ticks,
    })
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
    fn settle_waits_for_the_clock() {
        // Timing is flat from the first probe, but the clock climbs through the
        // first half of the stretch: settle must wait for the clock to hold, and
        // name the state it held at. This is the 3900X `powersave` defect in
        // miniature: flat timing alone used to read settled immediately.
        let warm = probes(16, 25_000, 25_200, 0);
        let khz = |k: u64| Some(crate::freq::FreqSample { cpu: 0, khz: k });
        let clock: Vec<_> = (0..8u64)
            .map(|i| khz(3_800_000 + i * 100_000))
            .chain((0..8).map(|_| khz(4_350_000)))
            .collect();
        match settle(&warm, &clock, 4) {
            Some(Settle::At {
                t_s,
                settled_frac,
                start_ghz,
                ghz,
                rating,
            }) => {
                assert!(
                    t_s >= warm[8].t_start_s,
                    "settled at {t_s}s before the clock held"
                );
                // The settled share is the settle point to the stretch's last probe, over
                // the whole stretch.
                let want = (warm[15].t_start_s - t_s) / warm[15].t_start_s;
                assert!(
                    (settled_frac - want).abs() < 1e-9,
                    "share {settled_frac} disagrees with the stretch"
                );
                assert_eq!(ghz, Some(4.35));
                // The journey: first readable sample to the settled median, a real move,
                // rated flat once there.
                assert_eq!(start_ghz, Some(3.8));
                assert_eq!(rating, Some(0.0), "a flat suffix rates +-0.0%");
            }
            other => panic!("expected a settled suffix, got {other:?}"),
        }
        // No clock series at all falls back to timing-only, settling immediately
        // with no state to name: the settled share is the whole stretch.
        match settle(&warm, &[], 4) {
            Some(Settle::At {
                ghz: None,
                settled_frac,
                ..
            }) => {
                assert!((settled_frac - 1.0).abs() < 1e-9);
            }
            other => panic!("expected an immediate timing-only settle, got {other:?}"),
        }
    }

    #[test]
    fn settle_cell_shows_the_journey() {
        // The four cell forms the Display contract promises, driven end to end from the
        // series: moved, no observed change, never settled, and timing-only.
        let warm = probes(16, 25_000, 25_200, 0);
        let khz = |k: u64| Some(crate::freq::FreqSample { cpu: 0, khz: k });
        let ramp: Vec<_> = (0..8u64)
            .map(|i| khz(3_600_000 + i * 100_000))
            .chain((0..8).map(|_| khz(4_090_000)))
            .collect();
        let cell = settle(&warm, &ramp, 4).expect("graded").to_string();
        assert!(
            cell.starts_with("3.60->4.09GHz ") && cell.ends_with("% +-0.0%"),
            "moved journey cell: {cell}"
        );

        // A journey that went nowhere still prints its arrow, keeping the column uniform,
        // and a box settled from the first probe was settled through the whole stretch.
        let flat: Vec<_> = (0..16).map(|_| khz(4_090_000)).collect();
        assert_eq!(
            settle(&warm, &flat, 4).expect("graded").to_string(),
            "4.09->4.09GHz 100% +-0.0%"
        );

        // A stretch whose timing never grades A: the journey it was still on, and the
        // reserved 00%.
        let noisy = probes(16, 25_000, 45_000, 40);
        assert_eq!(
            settle(&noisy, &ramp, 4).expect("graded").to_string(),
            "3.60->4.09GHz 00%"
        );
        assert_eq!(settle(&noisy, &[], 4).expect("graded").to_string(), "00%");
    }

    #[test]
    fn settle_letter_scores_the_settled_share() {
        let at = |frac| Settle::At {
            t_s: 0.0,
            settled_frac: frac,
            start_ghz: None,
            ghz: None,
            rating: None,
        };
        assert_eq!(settle_letter(&at(1.0)), 'A');
        assert_eq!(settle_letter(&at(0.25)), 'A');
        assert_eq!(settle_letter(&at(0.12)), 'B');
        assert_eq!(settle_letter(&at(0.07)), 'C');
        // The floor case: the exit window as a share of a default warm settles at the
        // buzzer.
        assert_eq!(settle_letter(&at(0.03)), 'D');
        assert_eq!(settle_letter(&at(0.01)), 'F');
        assert_eq!(
            settle_letter(&Settle::Never {
                start_ghz: None,
                end_ghz: None
            }),
            'F'
        );
    }

    #[test]
    fn clock_stats_read_the_dominant_cpu() {
        // An unpinned run's series mixes cores at different clocks (a quarter of the
        // samples land on a slow core here). Every statistic follows the most-sampled
        // core, so the cell rates the clock rather than the scheduler's placements.
        let warm = probes(16, 25_000, 25_200, 0);
        let s = |cpu: usize, k: u64| Some(crate::freq::FreqSample { cpu, khz: k });
        let clock: Vec<_> = (0..16)
            .map(|i| {
                if i % 4 == 0 {
                    s(7, 3_200_000)
                } else {
                    s(0, 4_090_000)
                }
            })
            .collect();
        match settle(&warm, &clock, 4) {
            Some(Settle::At {
                start_ghz,
                ghz,
                rating,
                ..
            }) => {
                assert_eq!(start_ghz, Some(4.09), "the slow core's samples are ignored");
                assert_eq!(ghz, Some(4.09));
                assert_eq!(rating, Some(0.0));
            }
            other => panic!("expected settled, got {other:?}"),
        }
        let p = clock_profile(&clock).expect("readable");
        assert_eq!((p.min_ghz, p.max_ghz), (4.09, 4.09));
        assert!(p.ticks.chars().all(|c| c == '-'));
    }

    #[test]
    fn gaps_do_not_disable_the_scan_gate() {
        // The unpinned defect the filtered gate exists for: the dominant core's samples
        // wander 12% with other-core samples between them. clock_stable's missing-sample
        // fallback would bail true at every gap and certify the whole stretch (measured
        // live as a +-7.0% rating inside the 1% gate). The scan must instead range over
        // the readable samples, land where the dominant core held still, and rate flat.
        let warm = probes(16, 25_000, 25_200, 0);
        let s = |cpu: usize, k: u64| Some(crate::freq::FreqSample { cpu, khz: k });
        let clock: Vec<_> = (0..16)
            .map(|i| {
                if i % 2 == 1 {
                    s(7, 3_200_000)
                } else if i < 8 {
                    s(0, 4_000_000 + i * 60_000)
                } else {
                    s(0, 4_490_000)
                }
            })
            .collect();
        match settle(&warm, &clock, 4) {
            Some(Settle::At {
                t_s, ghz, rating, ..
            }) => {
                assert!(
                    t_s >= warm[7].t_start_s,
                    "settled at {t_s}s while the dominant core still moved"
                );
                assert_eq!(ghz, Some(4.49));
                assert_eq!(rating, Some(0.0));
            }
            other => panic!("expected a settled suffix, got {other:?}"),
        }
    }

    #[test]
    fn an_unverifiable_settle_does_not_certify() {
        // The clock is readable on this box (the ramp's samples prove it), but the settled
        // stretch holds no dominant-core sample, so the settled claim cannot be verified:
        // Never with an F, not a timing-only share (an evidence-free `18%` graded B beside
        // a verified `04%`'s D until this rule).
        let warm = probes(16, 25_000, 25_200, 0);
        let s = |cpu: usize, k: u64| Some(crate::freq::FreqSample { cpu, khz: k });
        let clock: Vec<_> = (0..16)
            .map(|i| {
                if i < 8 {
                    s(0, 3_600_000 + i * 100_000)
                } else {
                    s(7, 4_500_000)
                }
            })
            .collect();
        let got = settle(&warm, &clock, 4).expect("graded");
        assert!(
            matches!(got, Settle::Never { .. }),
            "expected an uncertified Never, got {got:?}"
        );
        assert_eq!(settle_letter(&got), 'F');
    }

    #[test]
    fn clock_profile_ticks_are_the_gates_view() {
        let khz = |k: u64| Some(crate::freq::FreqSample { cpu: 0, khz: k });
        // Two >1% climbs, a >1% dip, then holds inside the band.
        let series = [
            khz(3_600_000),
            khz(3_800_000),
            khz(4_000_000),
            khz(3_900_000),
            khz(3_910_000),
            None,
            khz(3_905_000),
        ];
        let p = clock_profile(&series).expect("readable samples");
        // The None produces no tick of its own: 6 readable samples, 5 steps, the last one
        // bridging the gap.
        assert_eq!(p.ticks, "^^v--");
        assert_eq!(p.min_ghz, 3.6);
        assert_eq!(p.max_ghz, 4.0);
        assert_eq!(clock_profile(&[None, None]), None);
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
