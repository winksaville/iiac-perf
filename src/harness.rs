use std::hint::black_box;

use hdrhistogram::Histogram;

use crate::overhead::Overhead;

const WARMUP: u64 = 10_000;
const ESTIMATE_STEPS: u64 = 1_000;
const ESTIMATE_SAMPLES: usize = 5;
const FRAMING_DOMINATION_RATIO: f64 = 10.0;
const MAX_INNER: u64 = 1_000;
const MIN_ITERATIONS: u64 = 1_000;
const MAX_ITERATIONS: u64 = 1_000_000_000;

pub trait Bench {
    fn name(&self) -> &str;
    fn step(&mut self) -> u64;
}

#[derive(Debug)]
pub struct RunCfg<'a> {
    pub overhead: &'a Overhead,
    pub target_seconds: f64,
    pub iterations_override: Option<u64>,
    pub inner_override: Option<u64>,
    /// Core pool for thread pinning. Indexed positionally with wrap-around
    /// via `core_for(thread_idx)`; empty means no pinning.
    pub pin_cores: &'a [usize],
}

impl RunCfg<'_> {
    /// CPU id for the bench's `thread_idx`-th thread, using wrap-around
    /// over the pool. Returns `None` when the pool is empty so callers
    /// can treat unpinned and pinned runs uniformly.
    pub fn core_for(&self, thread_idx: usize) -> Option<usize> {
        if self.pin_cores.is_empty() {
            None
        } else {
            Some(self.pin_cores[thread_idx % self.pin_cores.len()])
        }
    }
}

pub fn run_adaptive<B: Bench>(bench: &mut B, cfg: &RunCfg) -> (Histogram<u64>, u64, u64, f64) {
    for _ in 0..WARMUP {
        black_box(bench.step());
    }

    let step_cost_ns = estimate_step_cost(bench);
    let framing_ns = cfg.overhead.framing_per_sample_ns.max(1.0);
    let inner = cfg
        .inner_override
        .unwrap_or_else(|| pick_inner(step_cost_ns, framing_ns));
    let iterations = cfg
        .iterations_override
        .unwrap_or_else(|| pick_iterations(step_cost_ns, inner, cfg.target_seconds));

    let (hist, duration_s) = run_bench(bench, iterations, inner);
    (hist, iterations, inner, duration_s)
}

fn estimate_step_cost<B: Bench>(bench: &mut B) -> f64 {
    let mut samples: Vec<f64> = (0..ESTIMATE_SAMPLES)
        .map(|_| {
            let start = minstant::Instant::now();
            for _ in 0..ESTIMATE_STEPS {
                black_box(bench.step());
            }
            let total = start.elapsed().as_nanos() as u64;
            total as f64 / ESTIMATE_STEPS as f64
        })
        .collect();
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    samples[ESTIMATE_SAMPLES / 2]
}

fn pick_inner(step_cost_ns: f64, framing_ns: f64) -> u64 {
    let target = (FRAMING_DOMINATION_RATIO * framing_ns / step_cost_ns).ceil() as u64;
    target.clamp(1, MAX_INNER)
}

fn pick_iterations(step_cost_ns: f64, inner: u64, target_seconds: f64) -> u64 {
    let target_ns = target_seconds * 1e9;
    let per_sample_ns = inner as f64 * step_cost_ns;
    let raw = (target_ns / per_sample_ns).ceil() as u64;
    raw.clamp(MIN_ITERATIONS, MAX_ITERATIONS)
}

fn run_bench<B: Bench>(bench: &mut B, iterations: u64, inner: u64) -> (Histogram<u64>, f64) {
    // 1 ns to 60 s, 3 sig figs
    let mut hist = Histogram::<u64>::new_with_bounds(1, 60_000_000_000, 3).unwrap();

    let run_start = minstant::Instant::now();
    for _ in 0..iterations {
        let start = minstant::Instant::now();
        for _ in 0..inner {
            black_box(bench.step());
        }
        let elapsed_ns = start.elapsed().as_nanos() as u64;
        hist.record(round_elapsed(elapsed_ns, inner)).unwrap();
    }
    let duration_s = run_start.elapsed().as_nanos() as f64 / 1e9;

    (hist, duration_s)
}

fn round_elapsed(elapsed_ns: u64, inner: u64) -> u64 {
    (elapsed_ns + inner / 2) / inner
}

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

pub fn print_histogram(
    name: &str,
    iterations: u64,
    inner: u64,
    duration_s: f64,
    hist: &Histogram<u64>,
    overhead: &Overhead,
) {
    // Header line: bench name + logfmt-style metadata. `adj` is the
    // apparatus overhead subtracted from each sample downstream.
    let adj = overhead.per_call_ns(inner);
    let total = iterations * inner;
    println!(
        "{name} [duration={:.1}s outer={} inner={} calls={} adj/call={}ns]:",
        duration_s,
        fmt_commas(iterations),
        inner,
        fmt_commas(total),
        fmt_commas_f64(adj, 2),
    );

    // Band boundaries defined by percentiles. Each consecutive pair
    // forms one band; we iterate the histogram to compute per-band
    // stats (first, last, count, mean).
    let boundary_pcts: &[f64] = &[
        0.0, 0.01, 0.10, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70, 0.80, 0.90, 0.99, 1.0,
    ];
    let boundary_names: &[&str] = &[
        "min", "p1", "p10", "p20", "p30", "p40", "p50", "p60", "p70", "p80", "p90", "p99", "max",
    ];

    let n_bands = boundary_pcts.len() - 1;
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
        let idx = boundary_pcts[1..]
            .iter()
            .position(|&b| mid_rank < b)
            .unwrap_or(n_bands - 1);
        band_first[idx] = band_first[idx].min(value);
        band_last[idx] = band_last[idx].max(value);
        band_count[idx] += count;
        band_sum[idx] += value as u128 * count as u128;
        cumulative += count;
    }

    // Build rendered rows: (label, first, last, count, mean, adj_mean).
    struct BandRow {
        label: String,
        first: String,
        last: String,
        count: String,
        mean: String,
        adj_mean: String,
    }

    let mut rows: Vec<BandRow> = Vec::new();
    for i in 0..n_bands {
        if band_count[i] == 0 {
            continue;
        }
        let mean_val = band_sum[i] as f64 / band_count[i] as f64;
        let adj_mean = (mean_val - adj).max(0.0);
        rows.push(BandRow {
            label: format!("{}-{}", boundary_names[i], boundary_names[i + 1]),
            first: fmt_commas(band_first[i]),
            last: fmt_commas(band_last[i]),
            count: fmt_commas(band_count[i]),
            mean: fmt_commas_f64(mean_val, 0),
            adj_mean: fmt_commas_f64(adj_mean, 0),
        });
    }

    // Column widths from rendered strings.
    let label_w = rows.iter().map(|r| r.label.len()).max().unwrap_or(0);
    let first_w = rows.iter().map(|r| r.first.len()).max().unwrap_or(0);
    let last_w = rows.iter().map(|r| r.last.len()).max().unwrap_or(0);
    let count_w = rows.iter().map(|r| r.count.len()).max().unwrap_or(0);
    let mean_w = rows.iter().map(|r| r.mean.len()).max().unwrap_or(0);
    let adj_w = rows.iter().map(|r| r.adj_mean.len()).max().unwrap_or(0);

    const INDENT: &str = "  ";
    const GAP: &str = "    ";

    // Header row.
    let first_col = INDENT.len() + label_w + 1 + first_w;
    let last_gap = " ns".len() + GAP.len() + last_w;
    let count_gap = " ns".len() + GAP.len() + count_w;
    let mean_gap = GAP.len() + mean_w;
    let adj_gap = " ns".len() + GAP.len() + adj_w;
    println!(
        "{:>first_col$}{:>last_gap$}{:>count_gap$}{:>mean_gap$}{:>adj_gap$}",
        "first", "last", "count", "mean", "adjusted",
    );

    for r in &rows {
        println!(
            "{INDENT}{:<label_w$} {:>first_w$} ns{GAP}{:>last_w$} ns{GAP}{:>count_w$}{GAP}{:>mean_w$} ns{GAP}{:>adj_w$} ns",
            r.label, r.first, r.last, r.count, r.mean, r.adj_mean,
        );
    }

    // Whole-histogram summary. Aligned to mean/adjusted columns.
    let hist_mean = hist.mean();
    let hist_adj = (hist_mean - adj).max(0.0);
    let skip = first_w + " ns".len() + GAP.len() + last_w + " ns".len() + GAP.len() + count_w;
    println!(
        "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} ns{GAP}{:>adj_w$} ns",
        "mean",
        "",
        fmt_commas_f64(hist_mean, 0),
        fmt_commas_f64(hist_adj, 0),
    );
    println!(
        "{INDENT}{:<label_w$} {:>skip$}{GAP}{:>mean_w$} ns",
        "stdev",
        "",
        fmt_commas_f64(hist.stdev(), 0),
    );
    println!();
}
