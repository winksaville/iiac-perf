mod benches;
mod harness;
mod overhead;

use clap::Parser;

#[derive(Parser)]
#[command(version, about = "IIAC performance measurement")]
struct Cli {
    /// Benches to run. Pass 'all' for every registered bench, or
    /// one or more names. Run with no args to see the available list.
    benches: Vec<String>,

    /// Target wall-clock seconds per bench (auto-sizes iterations and INNER).
    #[arg(short = 'd', long, default_value_t = 5.0)]
    duration: f64,

    /// Override iterations (skips auto-sizing of total count; INNER still adapts).
    #[arg(short, long)]
    iterations: Option<u64>,

    /// Override INNER (skips auto-sizing of inner-loop count).
    /// INNER=1 measures single-call latency (each sample = one step); higher
    /// INNER measures back-to-back/burst rate (each sample = N steps averaged).
    #[arg(short = 'I', long)]
    inner: Option<u64>,
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

    let overhead = overhead::calibrate();
    println!("calibration:");
    println!(
        "  framing/sample    {:>7} ns  (timer pair, two-point fit)",
        harness::fmt_commas_f64(overhead.framing_per_sample_ns, 2)
    );
    println!(
        "  loop/iter         {:>7} ns  (per inner-loop iteration)",
        harness::fmt_commas_f64(overhead.loop_per_iter_ns, 2)
    );
    println!();

    let cfg = harness::RunCfg {
        overhead: &overhead,
        target_seconds: cli.duration,
        iterations_override: cli.iterations,
        inner_override: cli.inner,
    };

    for run in runners {
        run(&cfg);
    }
}
