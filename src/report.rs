//! The bench report: everything that turns a finished [`crate::harness::RunOutput`] into text.
//!
//! Split out of `harness.rs` so measuring and rendering stop sharing a file. The seam is
//! `RunOutput` itself, which the harness produces and this module consumes; nothing here feeds
//! back into a measurement.
//!
//! - **The band table and its summary rows** ([`print_report`]): per-quantile-band
//!   first/last/range/count/mean, then whole-histogram mean/stdev, the trimmed pair, the clock's
//!   `quantum`, and the block-replication rows.
//! - **The grade block**: the `env` rows grading the box across warmup and bench, over the `run`
//!   row grading the numbers themselves.
//! - **Formatting primitives** ([`fmt_commas`], [`fmt_commas_f64`], [`display_cols`],
//!   [`decimal_align`]) that `probe.rs` and `band_table.rs` share.
//!
//! Column alignment here assumes a terminal's uniform character cell. That assumption lives in
//! [`display_cols`] and nowhere else, so a renderer that measures text instead (a GUI) replaces
//! this module rather than editing it.

use hdrhistogram::Histogram;

use crate::bands::{self, BandLabels};
use crate::harness::{
    HIST_HIGH_PS, PS_PER_NS, ProbeSummary, RunCfg, RunOutput, WarmClock, WarmExit,
};
use crate::ticks;

/// Grade-block column widths, sized to each column's widest
/// realistic cell so right-aligned cells always leave the
/// two-space gap `qualify-environment`'s parser splits on.
///
/// - label columns (`grade`, `phase`) are left-aligned
/// - value columns are right-aligned; a signal that does not
///   apply to a row prints [`GB_BLANK`]
const GB_GRADE_W: usize = 5;
/// `phase` column: `warmup` is the widest phase.
const GB_PHASE_W: usize = 6;
/// `settle` column: a journey-carrying cell with its signal letter, like
/// `4.84->5.24GHz 100% +-0.2% A`, is the widest.
const GB_SETTLE_W: usize = 27;
/// `worst` column: the composite letter under its header.
const GB_WORST_W: usize = 5;
/// Percentage signal cells (`spread`, `drift`): `100.00% A`.
const GB_PCT_W: usize = 9;
/// `bursts` cells: `100% B`.
const GB_BURSTS_W: usize = 6;
/// `interference` column: its own header is the widest cell.
const GB_INT_W: usize = 12;
/// `step` cells carry the timestamp: `100.00% @99.99s F`.
const GB_STEP_W: usize = 17;
/// Blank grade-block cell: this signal does not apply to this
/// row. A plain typeable hyphen, not an em dash.
const GB_BLANK: &str = "-";

/// `CLOCK_BOOTTIME` minus `CLOCK_MONOTONIC` elapsed divergence
/// (seconds) at or above which [`warn_invalid`] reports that the
/// system suspended during the run.
const SUSPEND_WARN_S: f64 = 1.0;

/// The clock's per-sample resolution: one tick spread across the `inner` calls it brackets.
///
/// A sample times `inner` calls in one bracket, so the smallest per-call step the clock can
/// express is a tick divided by `inner`. It is the number that says whether a printed spread
/// describes the workload or the clock's lattice: a core spread well below `q` is the lattice
/// (rpi5-20cd's 54 MHz timer puts `q` at ~2% of the value), well above it is the workload.
///
/// - Sizing does not chase it. `pick_inner` targets framing domination, which fixes the
///   *relative* quantum at `tick / (RATIO * frame)`, a property of the box rather than the
///   bench. Chasing it instead would smear the tail linearly for precision the dither and a
///   few million samples already provide.
/// - Degenerate inputs yield 0 rather than an infinity in a report column; neither is reachable
///   (`pick_inner` clamps `inner` to at least 1).
fn quantum_ns(ticks_per_ns: f64, inner: u64) -> f64 {
    if ticks_per_ns <= 0.0 || inner == 0 {
        return 0.0;
    }
    1.0 / ticks_per_ns / inner as f64
}

/// Split an environment probe series into the two stretches the environment grade scores: the
/// warmup's trailing window and the probes taken while the bench ran.
///
/// - `warmup` is how many leading probes came from warmup (see [`RunOutput::warmup_probes`]);
///   it is clamped to the series length, so a truncated series can't panic.
/// - The warmup side's graded tail is the last `tail_len` probes: the warm exit window the
///   stopping rule graded ([`RunOutput::warm_tail`]), so the letter printed is the letter the
///   exit saw, one computation rather than two windows that can disagree. Absorbing a ramp is
///   warmup's job, so the stretch before the window is deliberately ungraded.
/// - Grading the two separately also stops the boundary between them from reading as a step,
///   which a blended series invents whenever warmup starts colder than the run.
///
/// Returns `(warm, tail, during)`: the whole warmup stretch (the one
/// `RunOutput::warm_settle` was scored over), its tail window (what the warmup grade reads),
/// and the bench stretch.
pub(crate) fn env_stretches(
    probes: &[ProbeSummary],
    warmup: usize,
    tail_len: usize,
) -> (&[ProbeSummary], &[ProbeSummary], &[ProbeSummary]) {
    let (warm, during) = probes.split_at(warmup.min(probes.len()));
    let tail = &warm[warm.len() - tail_len.min(warm.len())..];
    (warm, tail, during)
}

/// Print `WARNING` lines when the finished run's tail-sensitive
/// stats — `max` and the untrimmed mean/stdev — are poisoned:
///
/// - the system suspended during the run (clock divergence — a
///   mid-sample suspend inflates that sample by the sleep gap,
///   even under the histogram bound);
/// - one or more samples clamped at [`HIST_HIGH_PS`] (a wedged or
///   suspend-inflated sample with no detected suspend).
///
/// A few inflated samples out of millions land in the extreme
/// tail band: percentile boundaries and the trimmed non-tail
/// stats are unaffected, so the flag names what died rather than
/// condemning the whole report. Called at the end of
/// [`print_report`] so the flag is the last thing in the bench's
/// report, where it can't scroll out of mind. Prints one
/// `WARNING {name}:` header with each finding indented below it,
/// keeping the findings visible next to the long bench name.
///
/// Warnings are for stats that are *invalid* — poisoned by a
/// suspend or a clamp. The run gauge never routes here: its
/// signals describe the run truthfully, and much of what they
/// describe belongs to the workload rather than the machine.
pub(crate) fn warn_invalid(name: &str, hist: &Histogram<u64>, suspended_s: f64) {
    let mut findings: Vec<String> = Vec::new();
    if suspended_s >= SUSPEND_WARN_S {
        findings.push(format!(
            "system suspended ~{suspended_s:.1}s during the run; max/mean/stdev poisoned"
        ));
    }
    if !hist.is_empty() && hist.max() >= HIST_HIGH_PS {
        findings.push(format!(
            "sample(s) clamped at the {}s histogram bound; max/mean/stdev poisoned",
            HIST_HIGH_PS / 1_000_000_000_000
        ));
    }
    if !findings.is_empty() {
        println!("WARNING {name}:");
        for finding in &findings {
            println!("  {finding}");
        }
    }
}

/// Format an integer with thousands separators, e.g.
/// `12345` → `"12,345"`.
pub fn fmt_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format a step signal's ` @<time>s` suffix, empty when the
/// signal did not fire.
///
/// One place, used by both the environment and run grades: the two
/// call sites previously carried their own format strings and had
/// drifted to 3 and 1 decimals, printing the same quantity at
/// different precision on adjacent lines.
///
/// - Two decimals (10 ms) because both series locate a step to
///   within one batch or seam, and [`crate::harness::BATCH_TARGET_SECONDS`] plus
///   [`crate::harness::BATCH_SAMPLES`] put that at ~15-50 ms. Finer would claim
///   resolution neither series has; coarser would lose the grid.
fn step_at_suffix(step_frac: f64, step_at_s: f64) -> String {
    if step_frac > 0.0 {
        format!(" @{step_at_s:.2}s")
    } else {
        String::new()
    }
}

/// One percentage signal cell of the grade block: `value letter`
/// at the block's own fixed two-decimal precision (`--decimals`
/// governs the report's time columns, never these ratios).
fn pct_cell(frac: f64, letter: char) -> String {
    format!("{:.2}% {letter}", frac * 100.0)
}

/// A `bursts` cell: whole percent, since the signal counts
/// batches and finer digits would be false precision.
fn burst_cell(frac: f64, letter: char) -> String {
    format!("{:.0}% {letter}", frac * 100.0)
}

/// A `step` cell: the one signal carrying a timestamp, at the
/// 10 ms precision batches can actually locate a shift to.
fn step_cell(step_frac: f64, step_at_s: f64, letter: char) -> String {
    format!(
        "{:.2}%{} {letter}",
        step_frac * 100.0,
        step_at_suffix(step_frac, step_at_s)
    )
}

/// Print one grade-block line: the header or a row, nine cells
/// in the shared column layout the widths (`GB_*`) define.
fn print_grade_line(cells: [&str; 9]) {
    const INDENT: &str = "  ";
    let [
        grade,
        phase,
        settle,
        worst,
        spread,
        bursts,
        interference,
        drift,
        step,
    ] = cells;
    println!(
        "{INDENT}{grade:<GB_GRADE_W$}  {phase:<GB_PHASE_W$}  {settle:>GB_SETTLE_W$}  \
         {worst:>GB_WORST_W$}  {spread:>GB_PCT_W$}  {bursts:>GB_BURSTS_W$}  \
         {interference:>GB_INT_W$}  {drift:>GB_PCT_W$}  {step:>GB_STEP_W$}"
    );
}

/// Format a float with `decimals` fractional digits and thousands
/// separators on the integer part.
/// Width of a rendered cell in terminal columns.
///
/// Three quantities coincide for the ASCII this report prints and diverge otherwise:
/// `str::len()` counts *bytes*, `{:>n$}` padding counts *chars*, and a terminal renders
/// *columns* (CJK is two, combining marks are zero). Counting chars is what agrees with the
/// padder, so a column stays square; the point of routing every measurement through one function
/// is that swapping in a width-aware crate later means changing this body and nothing else.
pub fn display_cols(s: &str) -> usize {
    s.chars().count()
}

/// Split a rendered number at its decimal point: `"15,974.399"` gives `("15,974", "399")`, and a
/// number without one gives an empty fraction.
fn split_decimal(s: &str) -> (&str, &str) {
    match s.find('.') {
        Some(i) => (&s[..i], &s[i + 1..]),
        None => (s, ""),
    }
}

/// Pad a rendered number so a column of them lines up on the decimal point.
///
/// Right-justifying on the last character aligns the trailing unit but not the point, which
/// staggers as soon as one row carries more decimals than its neighbours: `quantum` prints at
/// least three (see [`quantum_ns`]) while every other row follows `--decimals`. Padding the
/// integer side to `int_cols` and the fraction to `frac_cols` puts every point in one column and
/// every following ` ns` in another.
///
/// - A value with no fraction in a column that has one is only reachable at `--decimals 0`,
///   where `quantum` still prints three. It pads across the point's column as well, so no row
///   renders a dangling `.`.
fn decimal_align(s: &str, int_cols: usize, frac_cols: usize) -> String {
    let (int, frac) = split_decimal(s);
    if frac_cols == 0 {
        return format!("{int:>int_cols$}");
    }
    if s.contains('.') {
        format!("{int:>int_cols$}.{frac:<frac_cols$}")
    } else {
        let blank = frac_cols + 1;
        format!("{int:>int_cols$}{:<blank$}", "")
    }
}

pub fn fmt_commas_f64(n: f64, decimals: usize) -> String {
    let s = format!("{n:.decimals$}");
    let (sign, body) = match s.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", s.as_str()),
    };
    let (int_part, frac_part) = match body.find('.') {
        Some(i) => (&body[..i], &body[i..]),
        None => (body, ""),
    };
    let int_num: u64 = int_part.parse().unwrap_or(0);
    format!("{sign}{}{frac_part}", fmt_commas(int_num))
}

/// Format a claim (resolution, CI95, LSC) that must never print as a
/// bare zero: extend decimals from `start` until the value rounds to
/// a nonzero digit, capped at 3 (the ps recording floor), then print
/// `<0.001`. A `0.00` claim reads as "nothing to distinguish", which
/// is the fiction the resolution row replaced; the dash for "no claim
/// exists" is the caller's, not this function's.
fn fmt_claim(v: f64, start: usize) -> String {
    for decimals in start..=3 {
        if v >= 0.5 * 10f64.powi(-(decimals as i32)) {
            return fmt_commas_f64(v, decimals);
        }
    }
    "<0.001".to_string()
}

/// Index of the band containing `mid_rank`, over the full boundary
/// ladder `bounds` (`n_bands = bounds.len() - 1`).
///
/// - Bands are **right-closed** `(lower, upper]`: a rank exactly on a
///   boundary falls in the band that boundary *caps* — a single
///   sample's mid-rank of 0.5 lands in `p50`, not `p60`. This matches
///   the upper-boundary row labels and the CDF reading of a
///   percentile (value at or below which that fraction of samples
///   falls). See [`print_report`] for the convention and references.
/// - `mid_rank` is the Hazen plotting position `(i - 0.5) / n` of the
///   sample's rank, computed by the caller.
fn band_index(mid_rank: f64, bounds: &[bands::Boundary]) -> usize {
    let n_bands = bounds.len() - 1;
    bounds[1..]
        .iter()
        .position(|b| mid_rank <= b.pct)
        .unwrap_or(n_bands - 1)
}

/// Build the trimmed-stat range label from the populated bands
/// below the n2 ≡ p99 tail cut.
///
/// - Names the first..last populated band in `band_count[..trim_bands]`
///   by its **upper** boundary (`bounds[i + 1]`), matching the row
///   labels — so the label tracks the real extent of the trimmed
///   data rather than asserting a `min` row (never printed — rows use
///   upper boundaries) or an `n2` band that can be empty.
/// - Collapses to a single name when one band holds all the trimmed
///   data (`p60`, not `p60..p60`).
/// - Empty string when no trimmed band is populated — only with no
///   samples at all, where the caller's `trim` is `None` and the
///   label goes unused.
fn trim_range_label(
    bounds: &[bands::Boundary],
    band_count: &[u64],
    trim_bands: usize,
    style: BandLabels,
) -> String {
    let first = (0..trim_bands).find(|&i| band_count[i] > 0);
    let last = (0..trim_bands).rev().find(|&i| band_count[i] > 0);
    match (first, last) {
        (Some(f), Some(l)) if f == l => bounds[f + 1].trim_name(style).to_string(),
        (Some(f), Some(l)) => format!(
            "{}..{}",
            bounds[f + 1].trim_name(style),
            bounds[l + 1].trim_name(style),
        ),
        _ => String::new(),
    }
}

/// Print the full bench report: header line (logfmt-style metadata),
/// per-band histogram, whole-histogram mean/stdev, and trimmed
/// mean/stdev (every band below the n2 ≡ p99 tail cut). The trimmed
/// rows are labeled by the span of populated non-tail bands (e.g.
/// `mean z4..n2`), so `min` — never a row — is not asserted and an
/// empty n2 band is not named; see the label derivation below.
///
/// Each histogram row is one band, labeled by its **upper**
/// boundary — deciles in the body (`p10` … `p90`), nines/zeros in
/// the tails (`zK`/`nK` = fraction 10^-K of samples below/above
/// the boundary) — the lower boundary being the previous printed
/// row (empty bands are skipped). Bands are **right-closed**
/// `(lower, upper]` (see [`band_index`]): a sample whose rank lands
/// exactly on a boundary counts in the band that boundary caps, so a
/// lone median sample reads `p50`, not `p60` — matching the
/// upper-boundary label and the CDF definition of a percentile. This
/// is `pandas.cut`'s `right=True` convention; the rank is the Hazen
/// plotting position `(i - 0.5) / n`. Label style comes from
/// `cfg.band_labels` and is recorded as `labels=` in the header
/// metadata so saved outputs are self-describing. Values are raw:
/// nothing is subtracted, so a column is what the apparatus
/// measured. The untrimmed `stdev` is the
/// hdrhistogram-native stdev, which includes the ms-scale outliers
/// in the tail band. Ends with `WARNING` lines flagging poisoned
/// stats when they apply — `suspended_s` comes from
/// [`crate::harness::run_adaptive`] (see [`warn_invalid`]).
pub fn print_report(name: &str, out: &RunOutput, cfg: &RunCfg) {
    let hist = &out.hist;
    let outer = out.outer;
    let inner = out.inner;
    let duration_s = out.duration_s;
    let suspended_s = out.suspended_s;
    let block_stats = out.block_stats.as_ref();
    // Header line: bench name + logfmt-style metadata.
    let total = outer * inner;
    let blocks_meta = match block_stats {
        Some(b) => format!(" blocks={}", b.blocks),
        None => String::new(),
    };
    let batches_meta = format!(" batches={}", out.batches.len());
    // The warm cell is this run's total spend over its total
    // allowance: settle budget (when this run ran the process
    // warm) plus the cap.
    println!(
        "{name} [duration={:.1}s warm={:.2}/{:.1}s outer={} inner={} calls={}{blocks_meta}{batches_meta} labels={}]:",
        duration_s,
        out.warm_used_s,
        out.warm_budget_s,
        fmt_commas(outer),
        inner,
        fmt_commas(total),
        cfg.band_labels.as_str(),
    );

    let bounds = bands::boundaries();

    // Trim anchor: bands at or above the n2 (p99) boundary are
    // the "tail" — excluded from the trimmed stats no matter how
    // many finer tail bands subdivide them.
    #[allow(clippy::unwrap_used)]
    // OK: boundaries() always emits n2 (N_DEPTH >= 2)
    let trim_bands = bounds.iter().position(|b| b.zpn == "n2").unwrap();

    let n_bands = bounds.len() - 1;
    let sample_count = hist.len();

    // Accumulate per-band stats by walking recorded histogram buckets.
    // Each bucket is assigned to the band containing its midpoint rank.
    let mut band_first = vec![u64::MAX; n_bands];
    let mut band_last = vec![0u64; n_bands];
    let mut band_count = vec![0u64; n_bands];
    let mut band_sum = vec![0u128; n_bands];

    let mut cumulative = 0u64;
    for iv in hist.iter_recorded() {
        let value = iv.value_iterated_to();
        let count = iv.count_at_value();
        let mid_rank = (cumulative as f64 + count as f64 / 2.0) / sample_count as f64;
        let idx = band_index(mid_rank, &bounds);
        band_first[idx] = band_first[idx].min(value);
        band_last[idx] = band_last[idx].max(value);
        band_count[idx] += count;
        band_sum[idx] += value as u128 * count as u128;
        cumulative += count;
    }

    // Trimmed-stat range label, derived from the populated bands.
    let trim_range = trim_range_label(&bounds, &band_count, trim_bands, cfg.band_labels);
    let mean_trim_label = format!("mean {trim_range}");
    let stdev_trim_label = format!("stdev {trim_range}");

    // Build rendered rows: (label, first, last, range, count, mean).
    struct BandRow {
        label: String,
        first: String,
        last: String,
        range: String,
        count: String,
        mean: String,
    }

    let mut rows: Vec<BandRow> = Vec::new();
    for i in 0..n_bands {
        if band_count[i] == 0 {
            continue;
        }
        let mean_ns = band_sum[i] as f64 / band_count[i] as f64 / PS_PER_NS;
        rows.push(BandRow {
            label: bounds[i + 1].label(cfg.band_labels),
            first: fmt_commas_f64(band_first[i] as f64 / PS_PER_NS, cfg.decimals),
            last: fmt_commas_f64(band_last[i] as f64 / PS_PER_NS, cfg.decimals),
            range: fmt_commas_f64(
                (band_last[i] - band_first[i] + 1) as f64 / PS_PER_NS,
                cfg.decimals,
            ),
            count: fmt_commas(band_count[i]),
            mean: fmt_commas_f64(mean_ns, cfg.decimals),
        });
    }

    // Whole-histogram and trimmed (every band below the n2 ≡ p99
    // tail cut) summary values, rendered before the width
    // pass so the widths account for them — the untrimmed stdev
    // is often wider than any band mean and would otherwise
    // overflow its column, shifting its line right.
    let hist_mean = hist.mean() / PS_PER_NS;
    let hist_mean_str = fmt_commas_f64(hist_mean, cfg.decimals);
    let hist_stdev_str = fmt_commas_f64(hist.stdev() / PS_PER_NS, cfg.decimals);

    let trim_count: u64 = band_count[..trim_bands].iter().sum();
    let trim = if trim_count > 0 {
        let trim_sum: u128 = band_sum[..trim_bands].iter().sum();
        let trim_mean = trim_sum as f64 / trim_count as f64 / PS_PER_NS;

        // Variance: walk histogram buckets, include only non-tail bands.
        let mut trim_var_sum = 0.0f64;
        let mut trim_var_count = 0u64;
        let mut cum = 0u64;
        for iv in hist.iter_recorded() {
            let value = iv.value_iterated_to();
            let count = iv.count_at_value();
            let mid_rank = (cum as f64 + count as f64 / 2.0) / sample_count as f64;
            let idx = band_index(mid_rank, &bounds);
            if idx < trim_bands {
                let diff = value as f64 / PS_PER_NS - trim_mean;
                trim_var_sum += diff * diff * count as f64;
                trim_var_count += count;
            }
            cum += count;
        }
        let trim_stdev = if trim_var_count > 1 {
            (trim_var_sum / trim_var_count as f64).sqrt()
        } else {
            0.0
        };

        Some((
            fmt_commas_f64(trim_mean, cfg.decimals),
            fmt_commas_f64(trim_stdev, cfg.decimals),
        ))
    } else {
        None
    };

    // Block summary strings, rendered before the width pass like
    // the other summary lines. CI95 / LSC are `-` for sleepless
    // blocks: partitions of one continuous run cannot pretend to
    // be independent replicates. Present values are claims and
    // never print as a bare zero ([`fmt_claim`]).
    let block_strs = block_stats.map(|b| {
        let opt = |v: Option<f64>| match v {
            Some(x) => fmt_claim(x, cfg.decimals.max(1)),
            None => "-".to_string(),
        };
        (
            fmt_commas_f64(b.mean_ns, cfg.decimals),
            opt(b.ci95_ns),
            opt(b.lsc_ns),
        )
    });

    // The clock's per-sample quantum, rendered next to the spread
    // rows because that is where it is needed: it says whether
    // those rows describe the workload or the lattice. At least 3
    // decimals regardless of `--decimals`, since a fine quantum
    // renders as `0.0` at the report's default precision and zero
    // is the one value it must never claim.
    let quantum_str = fmt_commas_f64(
        quantum_ns(ticks::ticks_per_ns(), inner),
        cfg.decimals.max(3),
    );

    // The resolution claim: the batch-curve drift floor
    // ([`crate::resolution`]), the smallest delta this run can
    // honestly distinguish, printed on every run. A claim never
    // prints as a bare zero ([`fmt_claim`]): this row replaced a
    // fictional 0.0, and "at least 2 decimals" lost to a quiet
    // 7600x whose true floor sat below 5 ps.
    let resolution_str = match &out.resolution {
        Some(r) => fmt_claim(r.floor_ns, cfg.decimals.max(2)),
        None => "-".to_string(),
    };

    // Column widths from rendered strings: band rows and the
    // summary lines that print in the mean column.
    let label_cols = rows
        .iter()
        .map(|r| display_cols(&r.label))
        .fold(0, usize::max);
    let first_cols = rows
        .iter()
        .map(|r| display_cols(&r.first))
        .fold(0, usize::max);
    let last_cols = rows
        .iter()
        .map(|r| display_cols(&r.last))
        .fold(0, usize::max);
    let range_cols = rows
        .iter()
        .map(|r| display_cols(&r.range))
        .fold(0, usize::max);
    let count_cols = rows
        .iter()
        .map(|r| display_cols(&r.count))
        .fold(0, usize::max);
    // The mean column aligns on the decimal point rather than on
    // its last character, so each side is measured separately
    // (see [`decimal_align`]). Band rows only: the summary rows
    // moved into their own left-aligned block below.
    let mean_values: Vec<&str> = rows.iter().map(|r| r.mean.as_str()).collect();
    let mean_int_cols = mean_values
        .iter()
        .map(|s| display_cols(split_decimal(s).0))
        .fold(0, usize::max);
    let mean_frac_cols = mean_values
        .iter()
        .map(|s| display_cols(split_decimal(s).1))
        .fold(0, usize::max);
    let mean_cols = mean_int_cols
        + if mean_frac_cols > 0 {
            mean_frac_cols + 1
        } else {
            0
        };

    const INDENT: &str = "  ";
    const GAP: &str = "    ";

    // Header row. Each label right-justifies to the last
    // character of its column's ` ns` unit; `count` is unitless
    // and right-justifies to its digits.
    const UNIT: usize = " ns".len();
    let first_col = INDENT.len() + label_cols + 1 + first_cols + UNIT;
    let last_gap = GAP.len() + last_cols + UNIT;
    let range_gap = GAP.len() + range_cols + UNIT;
    let count_gap = GAP.len() + count_cols;
    let mean_gap = GAP.len() + mean_cols + UNIT;
    println!(
        "{:>first_col$}{:>last_gap$}{:>range_gap$}{:>count_gap$}{:>mean_gap$}",
        "first", "last", "range", "count", "mean",
    );

    for r in &rows {
        println!(
            "{INDENT}{:<label_cols$} {:>first_cols$} ns{GAP}{:>last_cols$} ns{GAP}{:>range_cols$} ns{GAP}{:>count_cols$}{GAP}{} ns",
            r.label,
            r.first,
            r.last,
            r.range,
            r.count,
            decimal_align(&r.mean, mean_int_cols, mean_frac_cols),
        );
    }

    // The summary block: labels on the left beside a
    // decimal-aligned value column, fenced by blank lines, rather
    // than values right-aligned ~80 columns away under the band
    // table's mean column (wink's sketch, 2026-08-20, from live
    // 7600x runs).
    let mut summary: Vec<(String, String)> = vec![
        ("mean".to_string(), hist_mean_str),
        ("stdev".to_string(), hist_stdev_str),
    ];
    if let Some((trim_mean_str, trim_stdev_str)) = trim {
        summary.push((mean_trim_label, trim_mean_str));
        summary.push((stdev_trim_label, trim_stdev_str));
    }
    summary.push(("quantum".to_string(), quantum_str));
    summary.push(("resolution".to_string(), resolution_str));
    if let Some((block_mean_str, block_ci_str, block_lsc_str)) = block_strs {
        summary.push(("mean blocks".to_string(), block_mean_str));
        summary.push(("CI95".to_string(), block_ci_str));
        summary.push(("LSC".to_string(), block_lsc_str));
    }
    let sum_label_cols = summary
        .iter()
        .map(|(l, _)| display_cols(l))
        .fold(0, usize::max);
    let sum_int_cols = summary
        .iter()
        .map(|(_, v)| display_cols(split_decimal(v).0))
        .fold(0, usize::max);
    let sum_frac_cols = summary
        .iter()
        .map(|(_, v)| display_cols(split_decimal(v).1))
        .fold(0, usize::max);
    println!();
    for (label, v) in &summary {
        // A withheld value prints a bare `-`: a unit would dress
        // the absence up as a number.
        let unit = if v == "-" { "" } else { " ns" };
        let line = format!(
            "{INDENT}{label:<sum_label_cols$}  {}{unit}",
            decimal_align(v, sum_int_cols, sum_frac_cols),
        );
        println!("{}", line.trim_end());
    }
    // The grade block: one header over three rows, `env` grading
    // the *box* (two stretches: did warmup end settled, did the
    // bench stretch stay settled) above `run` grading *these*
    // numbers from the run's own batches. Each row's `worst` is
    // its own composite (worst signal wins), printed beside its
    // causes; a blank cell means the signal does not apply to
    // that row, which is the env/run signal mapping made
    // visible. Reported, never warned on — see [`crate::gauge`].
    let (warm, tail, during) = env_stretches(&out.probes, out.warmup_probes, out.warm_tail);
    let warm_grade = crate::gauge::EnvGrade::from_probes(tail);
    let bench_grade = crate::gauge::EnvGrade::from_probes(during);
    let run_grade = crate::gauge::RunGrade::from_batches(&out.batches);
    if warm_grade.is_some() || bench_grade.is_some() || run_grade.is_some() {
        println!();
        print_grade_line([
            "grade",
            "phase",
            "settle",
            "worst",
            "spread",
            "bursts",
            "interference",
            "drift",
            "step",
        ]);
    }
    // The settle cell rides the warmup row because it describes that stretch alone: the
    // ramp the tail window now sits after. The cell answers by exit verdict: a settled
    // exit reports the journey and the settled share (RunOutput::warm_settle, scored in
    // the harness while the clock series was alive), an unstable exit reports the
    // journey still under way and the reserved 0%, and an uncertified warm says so
    // instead of a number.
    let settled = match out.warm_exit {
        WarmExit::Uncertified => "uncertified".to_string(),
        // An unstable exit is the settle scan's Never with the journey it was still on;
        // the plain form is the defensive fallback should the two ever disagree.
        WarmExit::Unstable => match out.warm_settle {
            Some(s @ crate::gauge::Settle::Never { .. }) => s.to_string(),
            _ => "0%".to_string(),
        },
        WarmExit::Settled => match out.warm_settle {
            Some(s) => s.to_string(),
            None => GB_BLANK.to_string(),
        },
    };
    // The settle cell is a graded signal like the rest: its letter prints beside its value
    // (crate::gauge::settle_letter) and folds into the warmup row's worst below, so a
    // buzzer-beater settle reads D and a never-settled warmup reads F outright.
    let settle_l = match (out.warm_exit, out.warm_settle) {
        (WarmExit::Uncertified, _) => None,
        (WarmExit::Unstable, None) => Some('F'),
        (_, Some(s)) => Some(crate::gauge::settle_letter(&s)),
        (_, None) => None,
    };
    let settled = match settle_l {
        Some(l) => format!("{settled} {l}"),
        None => settled,
    };
    for (phase, grade, settle_cell) in [
        ("warmup", &warm_grade, settled.as_str()),
        ("bench", &bench_grade, GB_BLANK),
    ] {
        if let Some(g) = grade {
            let [sl_spread, sl_int, sl_drift, sl_step] = g.signal_letters();
            let worst = match (phase, settle_l) {
                ("warmup", Some(l)) => g.letter.max(l),
                _ => g.letter,
            };
            print_grade_line([
                "env",
                phase,
                settle_cell,
                &worst.to_string(),
                &pct_cell(g.spread_frac, sl_spread),
                GB_BLANK,
                &pct_cell(g.interference_frac, sl_int),
                &pct_cell(g.drift_frac, sl_drift),
                &step_cell(g.step_frac, g.step_at_s, sl_step),
            ]);
        }
    }
    if let Some(g) = run_grade {
        let [sl_int, sl_burst, sl_drift, sl_step] = g.signal_letters();
        print_grade_line([
            "run",
            "all",
            GB_BLANK,
            &g.letter.to_string(),
            GB_BLANK,
            &burst_cell(g.burst_frac, sl_burst),
            &pct_cell(g.interference_frac, sl_int),
            &pct_cell(g.drift_frac, sl_drift),
            &step_cell(g.step_frac, g.step_at_s, sl_step),
        ]);
    }
    // The complete warmup picture under -v: the per-probe table with the ramp's
    // shape, and where the exit window began.
    if log::log_enabled!(log::Level::Debug) && !warm.is_empty() {
        println!();
        let clock_cell = match out.warm_clock {
            Some(WarmClock {
                end_mhz,
                max_mhz: Some(max),
            }) => {
                format!(
                    ", clock {end_mhz:.0}/{max:.0} MHz ({:.1}%)",
                    end_mhz / max * 100.0
                )
            }
            Some(WarmClock {
                end_mhz,
                max_mhz: None,
            }) => format!(", clock {end_mhz:.0} MHz"),
            None => String::new(),
        };
        println!(
            "{INDENT}warmup probes ({} total, exit {:?}, window: last {} spanning \
             {:.1} ms{clock_cell}):",
            warm.len(),
            out.warm_exit,
            tail.len(),
            tail_span_ms(tail),
        );
        // The clock's profile: the journey, the tick-per-step shape the stability gate saw
        // (settle's start is where the ticks go quiet), and the series extremes.
        if let Some(p) = &out.warm_clock_profile {
            let (journey, state) = clock_journey_state(out.warm_settle.as_ref());
            println!(
                "{INDENT}clock: {journey}{} (min {:.2} max {:.2}, {state})",
                p.ticks, p.min_ghz, p.max_ghz,
            );
        }
        println!("{INDENT}      t ms   floor ns  spread ns   over %");
        for p in warm {
            let over_pct = if p.pairs == 0 {
                0.0
            } else {
                p.over_pairs as f64 / p.pairs as f64 * 100.0
            };
            println!(
                "{INDENT}{:>10.3} {:>10.3} {:>10.3} {:>8.2}",
                p.t_start_s * 1e3,
                p.floor_q_ps as f64 / PS_PER_NS,
                p.spread_q_ps as f64 / PS_PER_NS,
                over_pct,
            );
        }
    }
    warn_invalid(name, hist, suspended_s);
    println!();
}

/// The `-v` clock line's journey prefix (trailing space included, empty when the settle
/// verdict carries no clock) and its parenthetical state (`settled +-0.1%` / `not settled`).
fn clock_journey_state(settle: Option<&crate::gauge::Settle>) -> (String, String) {
    use crate::gauge::Settle;
    match settle {
        Some(Settle::At {
            start_ghz,
            ghz: Some(g),
            rating,
            ..
        }) => {
            let journey = match start_ghz {
                Some(s) => format!("{s:.2}->{g:.2}GHz "),
                None => format!("{g:.2}GHz "),
            };
            (
                journey,
                format!("settled{}", crate::gauge::rating_suffix(*rating)),
            )
        }
        Some(Settle::At { ghz: None, .. }) => (String::new(), "settled".to_string()),
        Some(Settle::Never {
            start_ghz: Some(s),
            end_ghz: Some(e),
        }) => (format!("{s:.2}->{e:.2}GHz "), "not settled".to_string()),
        Some(Settle::Never { .. }) => (String::new(), "not settled".to_string()),
        None => (String::new(), GB_BLANK.to_string()),
    }
}

/// Wall span of a probe window (ms), first probe's start to the last's: the number the exit's
/// minimum-span rule constrains.
fn tail_span_ms(tail: &[ProbeSummary]) -> f64 {
    match (tail.first(), tail.last()) {
        (Some(first), Some(last)) => (last.t_start_s - first.t_start_s) * 1e3,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A probe at `at` seconds with a flat floor; only the series length matters to the
    /// stretch split, so the quantiles are arbitrary.
    fn probe(at: f64, floor_ps: u64) -> ProbeSummary {
        ProbeSummary {
            t_start_s: at,
            groups: 128,
            floor_q_ps: floor_ps,
            spread_q_ps: floor_ps * 1008 / 1000,
            mean_ps: floor_ps as f64,
            pairs: 8192,
            over_pairs: 0,
        }
    }

    #[test]
    fn fmt_claim_never_prints_a_bare_zero() {
        assert_eq!(fmt_claim(0.17, 2), "0.17");
        assert_eq!(fmt_claim(16.115, 1), "16.1");
        // Extends until the leading digit shows.
        assert_eq!(fmt_claim(0.04, 1), "0.04");
        assert_eq!(fmt_claim(0.004, 2), "0.004");
        // Below the recording floor: bounded, never zero.
        assert_eq!(fmt_claim(0.0004, 1), "<0.001");
        assert_eq!(fmt_claim(0.0, 2), "<0.001");
    }

    #[test]
    fn decimal_align_lines_up_the_point() {
        // The case the `quantum` row created: three decimals beside a column of one.
        assert_eq!(decimal_align("24.0", 5, 3), "   24.0  ");
        assert_eq!(decimal_align("0.011", 5, 3), "    0.011");
    }

    #[test]
    fn decimal_align_pads_a_fractionless_value_across_the_point() {
        // Reachable at `--decimals 0`, where `quantum` still prints three.
        assert_eq!(decimal_align("24", 5, 3), "   24    ");
    }

    #[test]
    fn decimal_align_leaves_an_integer_column_alone() {
        assert_eq!(decimal_align("24", 5, 0), "   24");
    }

    #[test]
    fn quantum_is_one_tick_over_inner() {
        // rpi5-20cd's 54 MHz Generic Timer: 18.5185 ns per tick, over the 23 calls a `min-now`
        // sample brackets, is the 0.805 ns lattice its band table shows.
        let q = quantum_ns(0.054, 23);
        assert!((q - 0.805).abs() < 0.001, "got {q}");
    }

    #[test]
    fn quantum_shrinks_with_a_faster_clock() {
        // 3900X TSC at 3.792855 ticks/ns, same `inner`: ~70x finer, and invisible in a report.
        let q = quantum_ns(3.792855, 23);
        assert!((q - 0.0115).abs() < 0.0001, "got {q}");
    }

    #[test]
    fn quantum_guards_degenerate_inputs() {
        assert_eq!(quantum_ns(0.0, 23), 0.0);
        assert_eq!(quantum_ns(3.792855, 0), 0.0);
    }

    #[test]
    fn env_stretches_survives_a_short_series() {
        // `--no-env-probe` leaves only warmup probes, and fewer of them than the
        // recorded tail claims.
        let probes: Vec<ProbeSummary> = (0..3).map(|i| probe(i as f64 * 0.001, 24_000)).collect();
        let (warm, tail, during) = env_stretches(&probes, 16, 16);
        assert_eq!(warm.len(), 3);
        assert_eq!(tail.len(), 3);
        assert!(during.is_empty());
    }

    /// A `band_count` vec (len = n_bands) with the given band
    /// indices marked populated.
    fn counts(n_bands: usize, populated: &[usize]) -> Vec<u64> {
        let mut c = vec![0u64; n_bands];
        for &i in populated {
            c[i] = 1;
        }
        c
    }

    #[test]
    fn trim_range_label_spans_populated_bands() {
        let bounds = bands::boundaries();
        let n_bands = bounds.len() - 1;
        // OK: boundaries() always emits n2 (N_DEPTH >= 2)
        let trim_bands = bounds.iter().position(|b| b.zpn == "n2").unwrap();

        // Full range: first band (label z4) through the n2 band.
        let c = counts(n_bands, &[0, 5, trim_bands - 1]);
        assert_eq!(
            trim_range_label(&bounds, &c, trim_bands, BandLabels::Zpn),
            "z4..n2"
        );
        assert_eq!(
            trim_range_label(&bounds, &c, trim_bands, BandLabels::Both),
            "z4..n2"
        );
        assert_eq!(
            trim_range_label(&bounds, &c, trim_bands, BandLabels::Frac),
            "0.000_1..0.99"
        );

        // n2 band empty: upper end is the last populated band (p90).
        let c = counts(n_bands, &[0, 11]);
        assert_eq!(
            trim_range_label(&bounds, &c, trim_bands, BandLabels::Zpn),
            "z4..p90"
        );

        // One populated band collapses to a single name.
        let c = counts(n_bands, &[8]);
        assert_eq!(
            trim_range_label(&bounds, &c, trim_bands, BandLabels::Zpn),
            "p60"
        );

        // No populated trimmed band yields an empty label (unused).
        let c = counts(n_bands, &[]);
        assert_eq!(
            trim_range_label(&bounds, &c, trim_bands, BandLabels::Zpn),
            ""
        );
    }

    #[test]
    fn band_index_right_closed_on_boundary() {
        let bounds = bands::boundaries();
        // Label of the band `mid_rank` falls in (bands labeled by
        // upper boundary → bounds[idx + 1]).
        let label = |r: f64| bounds[band_index(r, &bounds) + 1].zpn.as_str();

        // Right-closed: a rank exactly on a boundary lands in the band
        // that boundary caps, not the next one up.
        assert_eq!(label(0.5), "p50"); // single-sample mid-rank
        assert_eq!(label(0.4), "p40"); // exactly the p40 boundary
        assert_eq!(label(0.99), "n2"); // exactly p99 → last non-tail band
        assert_eq!(label(0.01), "z2"); // exactly the z2 boundary

        // Strictly-interior ranks are unaffected by the closed end.
        assert_eq!(label(0.45), "p50");
        assert_eq!(label(0.55), "p60");
        assert_eq!(label(0.05), "p10");
    }
}
