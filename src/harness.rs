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
}

pub fn run_adaptive<B: Bench>(bench: &mut B, cfg: &RunCfg) -> (Histogram<u64>, u64, u64) {
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

    let hist = run_bench(bench, iterations, inner);
    (hist, iterations, inner)
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

fn run_bench<B: Bench>(bench: &mut B, iterations: u64, inner: u64) -> Histogram<u64> {
    // 1 ns to 60 s, 3 sig figs
    let mut hist = Histogram::<u64>::new_with_bounds(1, 60_000_000_000, 3).unwrap();

    for _ in 0..iterations {
        let start = minstant::Instant::now();
        for _ in 0..inner {
            black_box(bench.step());
        }
        let elapsed_ns = start.elapsed().as_nanos() as u64;
        hist.record(round_elapsed(elapsed_ns, inner)).unwrap();
    }

    hist
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
    hist: &Histogram<u64>,
    overhead: &Overhead,
) {
    let adj = overhead.per_call_ns(inner);
    let total = iterations * inner;
    println!(
        "{name} ({} iters × {} inner = {} calls; subtract {} ns/call)",
        fmt_commas(iterations),
        inner,
        fmt_commas(total),
        fmt_commas_f64(adj, 2),
    );
    println!("                      raw      adjusted");
    print_row("min", hist.min(), adj);
    print_row("p1", hist.value_at_quantile(0.01), adj);
    print_row("p10", hist.value_at_quantile(0.10), adj);
    print_row("p50", hist.value_at_quantile(0.50), adj);
    print_row("p90", hist.value_at_quantile(0.90), adj);
    print_row("p99", hist.value_at_quantile(0.99), adj);
    print_row("p99.9", hist.value_at_quantile(0.999), adj);
    print_row("p99.99", hist.value_at_quantile(0.9999), adj);
    print_row("max", hist.max(), adj);
    let mean = hist.mean();
    let adj_mean = (mean - adj).max(0.0);
    println!(
        "  mean       {:>10} ns    {:>10} ns",
        fmt_commas_f64(mean, 1),
        fmt_commas_f64(adj_mean, 1)
    );
    println!("  stdev      {:>10} ns", fmt_commas_f64(hist.stdev(), 1));
    println!();
}

fn print_row(label: &str, raw: u64, overhead: f64) {
    let adj = (raw as f64 - overhead).max(0.0);
    println!(
        "  {label:<8} {:>10} ns    {:>10} ns",
        fmt_commas(raw),
        fmt_commas_f64(adj, 1)
    );
}
