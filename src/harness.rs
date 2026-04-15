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

    // All stats displayed as whole ns — timer resolution is ~1 ns and
    // cross-run duration variance dwarfs sub-ns structure, so extra
    // decimals would overclaim precision. stdev has no adjusted column
    // since subtracting overhead from a spread is meaningless.
    let rows: Vec<Row> = vec![
        Row::with_adj("min", hist.min() as f64, adj),
        Row::with_adj("p1", hist.value_at_quantile(0.01) as f64, adj),
        Row::with_adj("p10", hist.value_at_quantile(0.10) as f64, adj),
        Row::with_adj("p50", hist.value_at_quantile(0.50) as f64, adj),
        Row::with_adj("p90", hist.value_at_quantile(0.90) as f64, adj),
        Row::with_adj("p99", hist.value_at_quantile(0.99) as f64, adj),
        Row::with_adj("p99.9", hist.value_at_quantile(0.999) as f64, adj),
        Row::with_adj("p99.99", hist.value_at_quantile(0.9999) as f64, adj),
        Row::with_adj("max", hist.max() as f64, adj),
        Row::with_adj("mean", hist.mean(), adj),
        Row::raw_only("stdev", hist.stdev()),
    ];

    // Render each row to its final strings so we can size columns from
    // the actual widths.
    let rendered: Vec<(&str, String, Option<String>)> = rows
        .iter()
        .map(|r| {
            (
                r.label,
                fmt_commas_f64(r.raw, 0),
                r.adj.map(|a| fmt_commas_f64(a, 0)),
            )
        })
        .collect();

    // Column widths: widest rendered string in each column.
    let raw_w = rendered.iter().map(|(_, r, _)| r.len()).max().unwrap_or(0);
    let adj_w = rendered
        .iter()
        .filter_map(|(_, _, a)| a.as_ref().map(String::len))
        .max()
        .unwrap_or(0);

    // Fixed layout pieces: 2-space indent, 8-wide label, 4-space gap
    // between the raw and adjusted columns.
    const INDENT: &str = "  ";
    const LABEL_W: usize = 8;
    const GAP: &str = "    ";

    // "raw"/"adjusted" header sits over the numeric columns. We right-
    // align each label to the right-edge of its column (just before the
    // " ns" suffix).
    let raw_col_end = INDENT.len() + LABEL_W + 1 + raw_w;
    let adj_col_end = " ns".len() + GAP.len() + adj_w;
    println!("{:>raw_col_end$}{:>adj_col_end$}", "raw", "adjusted",);

    // Data rows: label left-aligned, numbers right-aligned in their
    // data-driven widths. Rows with no adjusted value skip that column.
    for (label, raw_s, adj_s) in &rendered {
        match adj_s {
            Some(a) => println!("{INDENT}{label:<LABEL_W$} {raw_s:>raw_w$} ns{GAP}{a:>adj_w$} ns"),
            None => println!("{INDENT}{label:<LABEL_W$} {raw_s:>raw_w$} ns"),
        }
    }
    println!();
}

struct Row {
    label: &'static str,
    raw: f64,
    adj: Option<f64>,
}

impl Row {
    fn with_adj(label: &'static str, raw: f64, overhead: f64) -> Self {
        Self {
            label,
            raw,
            adj: Some((raw - overhead).max(0.0)),
        }
    }

    fn raw_only(label: &'static str, raw: f64) -> Self {
        Self {
            label,
            raw,
            adj: None,
        }
    }
}
