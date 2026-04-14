use hdrhistogram::Histogram;

pub const WARMUP: u64 = 10_000;
pub const INNER_ITERATIONS: u64 = 10;

pub trait Bench {
    fn name(&self) -> &str;
    fn step(&mut self);
}

pub fn run_bench<B: Bench>(bench: &mut B, iterations: u64) -> Histogram<u64> {
    let mut hist = Histogram::<u64>::new_with_bounds(1, 1_000_000_000, 3).unwrap();

    for _ in 0..WARMUP {
        bench.step();
    }

    for _ in 0..iterations {
        let start = minstant::Instant::now();
        for _ in 0..INNER_ITERATIONS {
            bench.step();
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

pub fn print_histogram(name: &str, iterations: u64, hist: &Histogram<u64>) {
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
