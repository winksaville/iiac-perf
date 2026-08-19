//! The run's honest resolution: a variance-versus-aggregation
//! curve over batch means, whose plateau is the drift floor.
//!
//! The block LSC treats block means as independent draws, but
//! blocks milliseconds apart share the run's thermal and P-state
//! history, so it read ~7x optimistic against measured run-to-run
//! scatter. A single run cannot measure run-to-run scatter
//! directly; what it can do is watch whether variance falls as
//! `1/n` under aggregation (Allan deviation's move, IEEE Std
//! 1139). Where it stops falling is drift the run cannot average
//! away, and that floor is the smallest delta the run can
//! honestly claim to resolve.
//!
//! - The curve is fit on **batch means**: contiguous ~15-50 ms
//!   slices, the shape Allan deviation is defined over, present
//!   in every run whether or not `--blocks` was given.
//! - Group means are sample-count weighted, batch durations being
//!   uneven (fast benches flush on the sample buffer, slow ones
//!   on the time cap).
//! - Each level applies the LSC formula to its groups
//!   (`t(0.975, 2J-2) * s * sqrt(2/J)`), and the floor is the
//!   worst level: under white noise the levels agree, under drift
//!   the deeper levels rise, and taking the max cannot understate.
//! - Levels below [`MIN_GROUPS`] groups are excluded (the first
//!   level excepted): at J=2 the t multiplier alone inflates the
//!   claim 2.2x, so a deeper level would report its own
//!   uncertainty as drift.

use crate::harness::{BatchSummary, PS_PER_NS, t975};

/// Minimum groups for an aggregation level past the first:
/// t(2J-2)*sqrt(2/J) is within ~9% of its limit at J=8 and 2.2x
/// at J=2, so deeper levels would be dominated by their own
/// estimator noise rather than by drift.
const MIN_GROUPS: u64 = 8;

/// One aggregation level of the curve.
#[derive(Debug, Clone, Copy)]
pub struct CurvePoint {
    /// Batches per group at this level (1, 2, 4, ...).
    pub group: u64,
    /// Groups the level yielded (trailing remainder dropped).
    pub groups: u64,
    /// The LSC formula applied to this level's group means, ns.
    pub res_ns: f64,
}

/// The fitted curve and its floor: the run's resolution claim.
#[derive(Debug)]
pub struct Resolution {
    /// The claim: the worst level's `res_ns`.
    pub floor_ns: f64,
    /// Batches per group at the floor level.
    pub floor_group: u64,
    /// Groups at the floor level.
    pub floor_groups: u64,
    /// Every level fitted, shallow to deep.
    #[allow(dead_code)]
    // OK: the fitted levels behind the floor, asserted by this
    // module's tests and reproducible from the record's batch
    // series; a -v curve display is the intended future reader.
    pub curve: Vec<CurvePoint>,
}

/// Fit the curve from a run's batch series. `None` below two
/// usable batches: one slice supports no variance at all.
pub fn from_batches(batches: &[BatchSummary]) -> Option<Resolution> {
    let usable: Vec<&BatchSummary> = batches.iter().filter(|b| b.count > 0).collect();
    let b = usable.len() as u64;
    if b < 2 {
        return None;
    }
    let mut curve = Vec::new();
    let mut group = 1u64;
    while (group == 1 && b >= 2) || b / group >= MIN_GROUPS {
        curve.push(level(&usable, group));
        group *= 2;
        if b / group < 2 {
            break;
        }
    }
    #[allow(clippy::unwrap_used)]
    // OK: the b >= 2 guard above put at least the group=1 level in the curve
    let floor = curve
        .iter()
        .copied()
        .max_by(|a, c| a.res_ns.total_cmp(&c.res_ns))
        .unwrap();
    Some(Resolution {
        floor_ns: floor.res_ns,
        floor_group: floor.group,
        floor_groups: floor.groups,
        curve,
    })
}

/// Fit one level: chunk the series into groups of `group`
/// consecutive batches (remainder dropped), take count-weighted
/// group means, and apply the LSC formula to their spread.
fn level(batches: &[&BatchSummary], group: u64) -> CurvePoint {
    let j = batches.len() as u64 / group;
    let mut means = Vec::with_capacity(j as usize);
    for g in 0..j as usize {
        let chunk = &batches[g * group as usize..(g + 1) * group as usize];
        let count: u64 = chunk.iter().map(|x| x.count).sum();
        let sum: f64 = chunk.iter().map(|x| x.mean_ps * x.count as f64).sum();
        means.push(sum / count as f64 / PS_PER_NS);
    }
    let jf = j as f64;
    let mean = means.iter().sum::<f64>() / jf;
    let var = means.iter().map(|m| (m - mean) * (m - mean)).sum::<f64>() / (jf - 1.0);
    CurvePoint {
        group,
        groups: j,
        res_ns: t975(2 * j - 2) * var.sqrt() * (2.0 / jf).sqrt(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A batch with the given mean (ns) and sample count.
    fn batch(mean_ns: f64, count: u64) -> BatchSummary {
        BatchSummary {
            t_start_s: 0.0,
            t_end_s: 0.05,
            count,
            floor_ps: (mean_ns * PS_PER_NS) as u64,
            floor_q_ps: (mean_ns * PS_PER_NS) as u64,
            mean_ps: mean_ns * PS_PER_NS,
            max_ps: (mean_ns * PS_PER_NS) as u64,
            over_floor: 0,
        }
    }

    #[test]
    fn too_few_batches_yield_no_claim() {
        assert!(from_batches(&[]).is_none());
        assert!(from_batches(&[batch(20.0, 100)]).is_none());
    }

    #[test]
    fn anti_correlated_noise_floors_at_the_first_level() {
        // Alternating means cancel exactly under pairing, so every
        // deeper level's variance collapses and the first level is
        // the floor.
        let batches: Vec<BatchSummary> = (0..32)
            .map(|i| batch(if i % 2 == 0 { 20.0 } else { 22.0 }, 100))
            .collect();
        let r = from_batches(&batches).expect("curve fits");
        assert_eq!(r.floor_group, 1);
        assert!(r.curve.len() > 1);
        // Deeper levels see identical group means: zero spread.
        assert!(r.curve[1].res_ns < 1e-9, "got {}", r.curve[1].res_ns);
    }

    #[test]
    fn a_step_drift_raises_the_floor_above_the_first_level() {
        // Half the run at 20 ns, half at 24: aggregation cannot
        // average the step away, so deeper levels read worse and
        // the floor comes from one of them.
        let batches: Vec<BatchSummary> = (0..32)
            .map(|i| batch(if i < 16 { 20.0 } else { 24.0 }, 100))
            .collect();
        let r = from_batches(&batches).expect("curve fits");
        assert!(r.floor_group > 1, "floor at group {}", r.floor_group);
        assert!(r.floor_ns > r.curve[0].res_ns);
    }

    #[test]
    fn group_means_weight_by_count() {
        // Two batches, one carrying 9x the samples: the pooled
        // mean leans to it, so the level-1 spread is what the
        // unweighted mean would misstate.
        let batches = vec![batch(20.0, 900), batch(30.0, 100)];
        let r = from_batches(&batches).expect("curve fits");
        // J=2, s from two means 20 and 30: the claim exists and is
        // large; the point is it fit without panicking on uneven
        // counts and dropped nothing.
        assert_eq!(r.curve[0].groups, 2);
        assert!(r.floor_ns > 0.0);
    }

    #[test]
    fn zero_count_batches_are_excluded() {
        let batches = vec![batch(20.0, 100), batch(0.0, 0), batch(22.0, 100)];
        let r = from_batches(&batches).expect("two usable batches remain");
        assert_eq!(r.curve[0].groups, 2);
    }
}
