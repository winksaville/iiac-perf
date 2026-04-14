use std::hint::black_box;

use clap::Parser;
use hdrhistogram::Histogram;

const DEFAULT_ITERATIONS: u64 = 10_000_000;
const WARMUP: u64 = 10_000;
const INNER_ITERATIONS: u64 = 10;

#[derive(Parser)]
#[command(version, about = "Timer overhead measurement")]
struct Cli {
    /// Number of outer iterations
    #[arg(short, long, default_value_t = DEFAULT_ITERATIONS)]
    iterations: u64,
}

fn fmt_commas(n: u64) -> String {
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

fn round_elapsed_time(elapsed_ns: u64) -> u64 {
    (elapsed_ns + (INNER_ITERATIONS / 2)) / INNER_ITERATIONS
}

fn measure_minstant(iterations: u64, hist: &mut Histogram<u64>) {
    // Warmup
    for _ in 0..WARMUP {
        black_box(minstant::Instant::now());
    }

    // Measure
    for _ in 0..iterations {
        let start = minstant::Instant::now();
        for _ in 0..INNER_ITERATIONS {
            black_box(minstant::Instant::now());
        }
        let elapsed_ns = start.elapsed().as_nanos() as u64;
        hist.record(round_elapsed_time(elapsed_ns)).unwrap();
    }
}

fn measure_instant(iterations: u64, hist: &mut Histogram<u64>) {
    // Warmup
    for _ in 0..WARMUP {
        black_box(std::time::Instant::now());
    }

    // Measure using minstant as the reference timer
    for _ in 0..iterations {
        let start = minstant::Instant::now();
        for _ in 0..INNER_ITERATIONS {
            black_box(std::time::Instant::now());
        }
        let elapsed_ns = start.elapsed().as_nanos() as u64;
        hist.record(round_elapsed_time(elapsed_ns)).unwrap();
    }
}

fn print_histogram(name: &str, iterations: u64, hist: &Histogram<u64>) {
    println!(
        "{name} ({} calls)",
        fmt_commas(iterations * INNER_ITERATIONS)
    );
    println!("  min      {:>10} ns", fmt_commas(hist.min()));
    println!(
        "  p1       {:>10} ns",
        fmt_commas(hist.value_at_quantile(0.01))
    );
    println!(
        "  p10      {:>10} ns",
        fmt_commas(hist.value_at_quantile(0.10))
    );
    println!(
        "  p50      {:>10} ns",
        fmt_commas(hist.value_at_quantile(0.50))
    );
    println!(
        "  p90      {:>10} ns",
        fmt_commas(hist.value_at_quantile(0.90))
    );
    println!(
        "  p99      {:>10} ns",
        fmt_commas(hist.value_at_quantile(0.99))
    );
    println!(
        "  p99.9    {:>10} ns",
        fmt_commas(hist.value_at_quantile(0.999))
    );
    println!(
        "  p99.99   {:>10} ns",
        fmt_commas(hist.value_at_quantile(0.9999))
    );
    println!("  max      {:>10} ns", fmt_commas(hist.max()));
    println!("  mean     {:>10.1} ns", hist.mean());
    println!("  stdev    {:>10.1} ns", hist.stdev());
    println!();
}

fn main() {
    let cli = Cli::parse();

    println!(
        "iiac-perf {} — timer overhead measurement\n",
        env!("CARGO_PKG_VERSION")
    );

    // 1ns to 1s range, 3 significant digits
    let mut minstant_hist = Histogram::<u64>::new_with_bounds(1, 1_000_000_000, 3).unwrap();
    let mut instant_hist = Histogram::<u64>::new_with_bounds(1, 1_000_000_000, 3).unwrap();

    measure_minstant(cli.iterations, &mut minstant_hist);
    measure_instant(cli.iterations, &mut instant_hist);

    print_histogram("minstant::Instant::now()", cli.iterations, &minstant_hist);
    print_histogram("std::time::Instant::now()", cli.iterations, &instant_hist);
}
