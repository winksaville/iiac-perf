mod benches;
mod harness;
mod overhead;

use clap::Parser;

const DEFAULT_ITERATIONS: u64 = 10_000_000;
const CALIBRATION_SAMPLES: u64 = 100_000;

#[derive(Parser)]
#[command(version, about = "IIAC performance measurement")]
struct Cli {
    /// Number of outer iterations
    #[arg(short, long, default_value_t = DEFAULT_ITERATIONS)]
    iterations: u64,
}

fn main() {
    let cli = Cli::parse();

    println!(
        "iiac-perf {} — timer overhead measurement\n",
        env!("CARGO_PKG_VERSION")
    );

    let overhead = overhead::calibrate(CALIBRATION_SAMPLES);
    println!(
        "calibration ({} empty-bench samples):",
        harness::fmt_commas(CALIBRATION_SAMPLES)
    );
    println!(
        "  apparatus/sample  {:>7} ns  (timer framing + {} empty loop iters)",
        harness::fmt_commas(overhead.per_sample_min_ns),
        overhead.calls_per_sample
    );
    println!(
        "  apparatus/call    {:>7} ns  (subtracted from adjusted column)",
        harness::fmt_commas_f64(overhead.per_call_ns(), 2)
    );
    println!();

    benches::timer::run(cli.iterations, &overhead);
}
