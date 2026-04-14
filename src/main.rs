mod benches;
mod harness;
mod overhead;

use clap::Parser;

const DEFAULT_ITERATIONS: u64 = 10_000_000;
const CALIBRATION_SAMPLES: u64 = 100_000;

#[derive(Parser)]
#[command(version, about = "IIAC performance measurement")]
struct Cli {
    /// Benches to run. Pass 'all' for every registered bench, or
    /// one or more names. Run with no args to see the available list.
    benches: Vec<String>,

    /// Number of outer iterations
    #[arg(short, long, default_value_t = DEFAULT_ITERATIONS)]
    iterations: u64,
}

fn main() {
    let cli = Cli::parse();

    if cli.benches.is_empty() {
        println!("no benches specified. use -h or --help for more info.");
        println!("available: all, {}", benches::names().join(", "));
        return;
    }

    let runners = match benches::resolve(&cli.benches) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };

    println!(
        "iiac-perf {} — IIAC performance measurement\n",
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

    for run in runners {
        run(cli.iterations, &overhead);
    }
}
