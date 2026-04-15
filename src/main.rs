mod benches;
mod harness;
mod overhead;
mod pin;

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

    /// Pin bench threads to logical CPUs (comma-separated, ranges OK).
    /// The list is a core *pool*: thread `i` of a bench is pinned to
    /// `pool[i % pool.len()]`, so shorter pools oversubscribe by wrap.
    /// Examples: `--pin 0,1` (2 threads → 2 CPUs), `--pin 0-5` (6-thread
    /// pool), `--pin 0,0` (two threads on the same CPU). On 3900X,
    /// logical CPUs N and N+12 are SMT siblings of the same physical
    /// core — `--pin 0,12` pairs siblings (max contention), `--pin 0,1`
    /// gives independent cores. Omit to leave threads unpinned.
    #[arg(long, value_name = "CORES")]
    pin: Option<String>,
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

    let pin_cores: Vec<usize> = match cli.pin.as_deref() {
        None => Vec::new(),
        Some(spec) => match pin::parse_cores(spec) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: --pin: {e}");
                std::process::exit(2);
            }
        },
    };

    // Pin the main (measurement) thread before any calibration or bench
    // work so TSC reads stay on the same CPU across start/elapsed.
    if let Some(&cpu) = pin_cores.first() {
        pin::pin_current(Some(cpu));
    }

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
    println!("  pinning           {}", pin::plan_summary(&pin_cores));
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
        pin_cores: &pin_cores,
    };

    for run in runners {
        run(&cfg);
    }
}
