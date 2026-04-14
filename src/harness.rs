use hdrhistogram::Histogram;

use crate::overhead::Overhead;

pub const WARMUP: u64 = 10_000;
pub const INNER_ITERATIONS: u64 = 10;
pub const CALIBRATION_INNER: u64 = 100;

pub trait Bench {
    fn name(&self) -> &str;
    fn step(&mut self) -> u32;
}

pub fn run_bench<B: Bench>(bench: &mut B, iterations: u64) -> Histogram<u64> {
    let mut hist = Histogram::<u64>::new_with_bounds(1, 1_000_000_000, 3).unwrap();

    for _ in 0..WARMUP {
        bench.step();
    }

    for _ in 0..iterations {
        let start = minstant::Instant::now();
        for _ in 0..INNER_ITERATIONS {
            std::hint::black_box(bench.step());
        }
        let elapsed_ns = start.elapsed().as_nanos() as u64;
        hist.record(round_elapsed(elapsed_ns)).unwrap();
    }

    hist
}

fn round_elapsed(elapsed_ns: u64) -> u64 {
    (elapsed_ns + (INNER_ITERATIONS / 2)) / INNER_ITERATIONS
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

pub fn print_histogram(name: &str, iterations: u64, hist: &Histogram<u64>, overhead: &Overhead) {
    let adj = overhead.per_call_ns();
    println!(
        "{name} ({} calls)",
        fmt_commas(iterations * INNER_ITERATIONS)
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
