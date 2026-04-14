mod benches;
mod harness;
mod overhead;

use clap::{Parser, Subcommand};

const DEFAULT_ITERATIONS: u64 = 10_000_000;
const CALIBRATION_SAMPLES: u64 = 100_000;

#[derive(Parser)]
#[command(version, about = "IIAC performance measurement")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Number of outer iterations
    #[arg(short, long, default_value_t = DEFAULT_ITERATIONS, global = true)]
    iterations: u64,
}

#[derive(Subcommand)]
enum Command {
    /// Run timer overhead benches (minstant vs std::time::Instant)
    Timer,
    /// Run channel benches (std::sync::mpsc round-trip, single thread)
    Channel,
    /// Run all benches (default)
    All,
}

fn main() {
    let cli = Cli::parse();
    let cmd = cli.command.unwrap_or(Command::All);

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

    match cmd {
        Command::Timer => benches::timer::run(cli.iterations, &overhead),
        Command::Channel => benches::channel::run(cli.iterations, &overhead),
        Command::All => {
            benches::timer::run(cli.iterations, &overhead);
            benches::channel::run(cli.iterations, &overhead);
        }
    }
}
