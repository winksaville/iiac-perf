//! Scope-based measurement probe: a named, single-writer
//! histogram plus a record buffer, populated via `start` /
//! `end` rather than `record(ticks)`.
//!
//! `start(site_id)` reads the hardware tick counter and returns
//! an opaque [`TProbe2RecId`] carrying `(site_id, start_tsc)`;
//! `end(id)` reads the tick counter again and appends a complete
//! `(site_id, start_tsc, end_tsc)` record to the probe's
//! internal buffer. No delta math, histogram ingestion, or
//! tick→ns conversion happens on the hot path — all of that is
//! deferred to [`TProbe2::report`], which drains pending records
//! into the histogram before rendering.
//!
//! This primitive preserves record-order information across
//! interleaved scopes and sites (non-stack nesting is supported
//! by construction; see ideas.md — Tprobe, Option B) and gives
//! future evolution space for per-site grouping, bounded buffers,
//! background drain threads, and long-term trace retention.
//!
//! The trade-off vs. [`crate::tprobe::TProbe`]: a growing
//! `Vec<Record>` in the hot path adds cache pressure and
//! reallocation cost in long, high-rate runs. For high-rate
//! single-histogram measurement prefer `TProbe`. Run
//! [`crate::benches::tp_pc`] and [`crate::benches::tp2_pc`]
//! back-to-back to see the hot-path cost of the scope API on a
//! matched workload.

use hdrhistogram::Histogram;

use crate::band_table;
use crate::ticks;

/// Opaque handle returned by [`TProbe2::start`], consumed by
/// [`TProbe2::end`]. Carries the caller-supplied `site_id` and
/// the start-time tick reading; no probe-internal allocation
/// happens at `start` time.
///
/// `#[must_use]` — dropping the id without passing it to
/// [`TProbe2::end`] leaks the scope (no record is appended).
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct TProbe2RecId {
    site_id: u64,
    start_tsc: u64,
}

/// A complete scope record: `(site_id, start_tsc, end_tsc)`.
/// Appended at [`TProbe2::end`] time; the record buffer only
/// ever holds complete records. Drained into the histogram at
/// [`TProbe2::report`] time.
#[derive(Clone, Copy, Debug)]
struct Record {
    #[allow(dead_code)] // read once per-site grouping lands.
    site_id: u64,
    start_tsc: u64,
    end_tsc: u64,
}

/// A named, single-writer histogram of hardware tick-counter
/// deltas plus a scope-record buffer. Not `Sync`; cross-thread
/// *sharing* is out of scope. `Send` so probes can be moved
/// between threads (e.g. returned via a `JoinHandle<TProbe2>`
/// on shutdown).
pub struct TProbe2 {
    name: String,
    hist: Histogram<u64>,
    records: Vec<Record>,
}

impl TProbe2 {
    /// Create an empty probe. Histogram upper bound is 1e12
    /// ticks (~250 s at 4 GHz, ~100 s at 10 GHz), 3 significant
    /// figures.
    ///
    /// Exits the process (code 1) if the hardware tick counter
    /// isn't usable — see [`crate::ticks::require_ok`].
    pub fn new(name: &str) -> Self {
        ticks::require_ok();
        let _ = ticks::ticks_per_ns();
        Self {
            name: name.to_string(),
            hist: Histogram::<u64>::new_with_bounds(1, 1_000_000_000_000, 3).unwrap(),
            records: Vec::new(),
        }
    }

    /// Begin a scope. Reads the hardware tick counter and
    /// returns an opaque [`TProbe2RecId`] carrying `(site_id,
    /// start_tsc)`. The id must eventually be passed to
    /// [`TProbe2::end`]; a dropped id leaves no record.
    #[inline]
    pub fn start(&mut self, site_id: u64) -> TProbe2RecId {
        TProbe2RecId {
            site_id,
            start_tsc: ticks::read_ticks(),
        }
    }

    /// End the scope started by [`TProbe2::start`]. Reads the
    /// hardware tick counter and appends a complete record
    /// `(site_id, start_tsc, end_tsc)` to the probe's record
    /// buffer. Delta and histogram ingestion are deferred to
    /// [`TProbe2::report`].
    #[inline]
    pub fn end(&mut self, tpri: TProbe2RecId) {
        let end_tsc = ticks::read_ticks();
        self.records.push(Record {
            site_id: tpri.site_id,
            start_tsc: tpri.start_tsc,
            end_tsc,
        });
    }

    /// Render a band-table report for this probe. `as_ticks`
    /// controls the display unit: `false` converts stored tick
    /// deltas to nanoseconds (default for the CLI); `true` shows
    /// raw ticks (`-t`/`--ticks`).
    ///
    /// Drains any pending `start`/`end` records into the histogram
    /// before rendering: `delta = end_tsc − start_tsc`, clamped to
    /// `1` since the histogram lower bound is 1.
    pub fn report(&mut self, as_ticks: bool) {
        for r in self.records.drain(..) {
            let delta = r.end_tsc.saturating_sub(r.start_tsc);
            self.hist.record(delta.max(1)).unwrap();
        }
        band_table::render("tprobe2", &self.name, &self.hist, as_ticks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_end_appends_one_record() {
        let mut p = TProbe2::new("t");
        let id = p.start(42);
        p.end(id);
        assert_eq!(p.records.len(), 1);
        let r = &p.records[0];
        assert_eq!(r.site_id, 42);
        assert!(r.end_tsc >= r.start_tsc);
    }

    #[test]
    fn start_end_preserves_start_tsc() {
        let mut p = TProbe2::new("t");
        let id = p.start(7);
        let saved_start = id.start_tsc;
        p.end(id);
        let r = &p.records[0];
        assert_eq!(r.site_id, 7);
        assert_eq!(r.start_tsc, saved_start);
    }

    #[test]
    fn start_end_interleaved_non_stack() {
        let mut p = TProbe2::new("t");
        let a = p.start(1);
        let b = p.start(2);
        p.end(a);
        p.end(b);
        assert_eq!(p.records.len(), 2);
        assert_eq!(p.records[0].site_id, 1);
        assert_eq!(p.records[1].site_id, 2);
    }

    #[test]
    fn report_drains_records_into_histogram() {
        let mut p = TProbe2::new("t");
        let id1 = p.start(1);
        p.end(id1);
        let id2 = p.start(2);
        p.end(id2);
        assert_eq!(p.hist.len(), 0);
        assert_eq!(p.records.len(), 2);

        p.report(false);
        assert_eq!(p.records.len(), 0);
        assert_eq!(p.hist.len(), 2);

        // Idempotent: a second report drains nothing, hist unchanged.
        p.report(false);
        assert_eq!(p.hist.len(), 2);
    }
}
