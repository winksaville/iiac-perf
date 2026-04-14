mod benches;
mod harness;

use clap::Parser;

const DEFAULT_ITERATIONS: u64 = 10_000_000;

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

    benches::timer::run(cli.iterations);
}
