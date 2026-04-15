mod benches;
mod harness;
mod overhead;

use clap::Parser;

#[derive(Parser)]
#[command(version, about = "IIAC performance measurement", max_term_width = 80)]
struct Cli {
    /// Benches to run. Pass 'all' for every registered bench, or
    /// one or more names. Run with no args to see the available list.
    benches: Vec<String>,

    /// Target wall-clock seconds per bench (default 5.0). Auto-sizes
    /// iterations and INNER. Mutually exclusive with -D.
    #[arg(short = 'd', long, conflicts_with = "total_duration")]
    duration: Option<f64>,

    /// Target total wall-clock seconds across all requested benches;
    /// budget is split equally per bench. Mutually exclusive with -d.
    #[arg(short = 'D', long)]
    total_duration: Option<f64>,

    /// Override iterations (skips auto-sizing of total count; INNER still adapts).
    #[arg(short, long)]
    iterations: Option<u64>,

    /// Override INNER (skips auto-sizing of inner-loop count).
    /// INNER=1 measures single-call latency (each sample = one step); higher
    /// INNER measures back-to-back/burst rate (each sample = N steps averaged).
    #[arg(short = 'I', long)]
    inner: Option<u64>,
}

const DEFAULT_DURATION: f64 = 5.0;

fn main() {
    let cli = Cli::parse();

    if cli.benches.is_empty() {
        println!("no benches specified. use -h or --help for more info.");
        println!("available: all, {}", benches::names().join(", "));
        return;
    }

    println!(
        "iiac-perf {} — IIAC performance measurement\n",
        env!("CARGO_PKG_VERSION")
    );

    let overhead = overhead::calibrate();
    println!("Calibration:");
    println!(
        "  framing/sample    {:>7} ns  (timer pair, two-point fit)",
        harness::fmt_commas_f64(overhead.framing_per_sample_ns, 2)
    );
    println!(
        "  loop/iter         {:>7} ns  (per inner-loop iteration)",
        harness::fmt_commas_f64(overhead.loop_per_iter_ns, 2)
    );
    println!();

    let runners = match benches::resolve(&cli.benches) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };

    let target_seconds = match (cli.duration, cli.total_duration) {
        (Some(d), _) => d,
        (None, Some(t)) => t / runners.len() as f64,
        (None, None) => DEFAULT_DURATION,
    };

    let cfg = harness::RunCfg {
        overhead: &overhead,
        target_seconds,
        iterations_override: cli.iterations,
        inner_override: cli.inner,
    };

    for run in runners {
        run(&cfg);
    }
}
