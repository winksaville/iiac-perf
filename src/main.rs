mod band_table;
mod bands;
mod benches;
mod config;
mod harness;
mod inhibit;
mod overhead;
mod pin;
mod probe;
mod ticks;
mod tprobe;
mod tprobe2;

use clap::Parser;
use log::{debug, info};

#[derive(Parser)]
#[command(
    version,
    about = concat!(
        "iiac-perf ",
        env!("CARGO_PKG_VERSION"),
        " — Rust latency microbenchmark harness",
    ),
    max_term_width = 80,
)]
struct Cli {
    /// Benches to run. Pass 'all' for every registered bench, or
    /// one or more names; a name matching no bench exactly runs
    /// every bench it is a prefix of (e.g. 'ice', 'mpsc').
    /// Run with no args to see the available list.
    benches: Vec<String>,

    /// Target wall-clock seconds per bench (default 5.0, or the
    /// config `duration`). Auto-sizes outer and inner loop counts.
    /// Mutually exclusive with -D.
    #[arg(short = 'd', long, conflicts_with = "total_duration")]
    duration: Option<f64>,

    /// Target total wall-clock seconds across all requested benches;
    /// budget is split equally per bench. Mutually exclusive with -d.
    #[arg(short = 'D', long)]
    total_duration: Option<f64>,

    /// Override outer loop count (skips auto-sizing; inner still adapts).
    #[arg(short, long)]
    outer: Option<u64>,

    /// Override inner loop count (skips auto-sizing).
    /// inner=1 measures single-call latency (each sample = one step); higher
    /// inner measures back-to-back/burst rate (each sample = N steps averaged).
    #[arg(short, long)]
    inner: Option<u64>,

    /// Pin bench threads to logical CPUs (comma-separated, ranges OK).
    /// The list is a core *pool*: thread `i` of a bench is pinned to
    /// `pool[i % pool.len()]`, so shorter pools oversubscribe by wrap.
    /// Examples: `--pin 0,1` (2 threads → 2 CPUs), `--pin 0-5` (6-thread
    /// pool), `--pin 0,0` (two threads on the same CPU). On 3900X,
    /// logical CPUs N and N+12 are SMT siblings of the same physical
    /// core — `--pin 0,12` pairs siblings (max contention), `--pin 0,1`
    /// gives independent cores. A value naming a `[profiles]` entry in
    /// the config file expands to that profile's core spec (e.g.
    /// `--pin smt`). Omit to leave threads unpinned.
    #[arg(long, value_name = "CORES")]
    pin: Option<String>,

    /// Skip pinning the main thread for calibration. Always takes
    /// effect, including when `--pin` is set — bench threads still
    /// pin per `--pin`, but main stays on whatever affinity mask
    /// the process launched with. By default (without this flag),
    /// calibration runs with main pinned to `pin[0]` if set, else
    /// core 0, so framing/loop come from a stable environment.
    /// Pass this flag to decouple the calibration environment from
    /// the bench pinning pool (or to reproduce pre-0.6.0 behavior
    /// when `--pin` is absent).
    #[arg(long)]
    no_pin_cal: bool,

    /// Enable verbose internals on stderr: affinity mask, cal
    /// parameters and raw fit inputs, calibration wall time.
    /// Equivalent to `RUST_LOG=debug`. Default is `warn` (silent
    /// unless something's wrong). `RUST_LOG` overrides this flag
    /// when set, so per-module filtering still works.
    #[arg(short, long)]
    verbose: bool,

    /// Show tprobe results in raw TSC ticks instead of converting
    /// to nanoseconds. Only affects `TProbe` output; `Probe`
    /// results are always in nanoseconds.
    #[arg(short = 't', long)]
    ticks: bool,

    /// Band label style for the report's histogram rows.
    /// 'zpn': nines/zeros + decile names (z3, p50, n4).
    /// 'frac': literal boundary fractions with '_' grouping
    /// (0.001, 0.50, 0.999_9). 'both': zpn and fraction
    /// side by side — the juxtaposition teaches the zpn
    /// vocabulary; switch to 'zpn' once fluent. Overrides the
    /// config `band_labels`; both absent defaults to 'both'.
    #[arg(long, value_enum)]
    band_labels: Option<bands::BandLabels>,

    /// Decimal digits on the report's time columns (0-3).
    /// 1 shows the sub-ns precision picosecond recording
    /// captures; 0 restores integer ns; 3 is the recording
    /// floor - more digits would be artifacts. Overrides the
    /// config `decimals`; both absent defaults to 1.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=3))]
    decimals: Option<u8>,

    /// Do not inhibit system sleep for the run. By default the
    /// process re-execs itself under `systemd-inhibit --what=sleep`
    /// so an idle-suspend can't poison a long measurement. Pass
    /// this to keep the process image untouched (strace/gdb/perf
    /// wrappers), to let the machine sleep on purpose, or to test
    /// the suspend-detection WARNING path (a sleep inhibitor also
    /// blocks manual `systemctl suspend`).
    #[arg(long)]
    no_inhibit: bool,
}

const DEFAULT_DURATION: f64 = 5.0;
const DEFAULT_BAND_LABELS: bands::BandLabels = bands::BandLabels::Both;
const DEFAULT_DECIMALS: u8 = 1;

/// Banner text listing which config files were loaded, or
/// `"none (built-in defaults)"` when neither file exists.
fn config_summary(files: &[std::path::PathBuf]) -> String {
    if files.is_empty() {
        "none (built-in defaults)".to_string()
    } else {
        files
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn main() {
    let cli = Cli::parse();

    // Default filter is `warn`; `-v` bumps to `debug`. `RUST_LOG`
    // (if set) always wins — so users can still do fine-grained
    // per-module filtering without fighting the flag.
    let mut builder = env_logger::Builder::from_default_env();
    if std::env::var_os("RUST_LOG").is_none() {
        builder.filter_level(if cli.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Warn
        });
    }
    builder.format_timestamp(None).init();

    if cli.benches.is_empty() {
        println!("no benches specified. use -h or --help for more info.");
        println!("available: all, {}", benches::names().join(", "));
        return;
    }

    // Re-exec under systemd-inhibit (unless --no-inhibit or
    // already inhibited) before any output, so the banner prints
    // once, from the inhibited child.
    let inhibit_status = inhibit::ensure(cli.no_inhibit);

    // Layered defaults (built-in < XDG file < project-local file <
    // CLI). A malformed config is fatal so a typo surfaces.
    let (config, config_files) = match config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: config: {e}");
            std::process::exit(2);
        }
    };

    println!(
        "iiac-perf {} — Rust latency microbenchmark harness\n",
        env!("CARGO_PKG_VERSION")
    );

    if let Some(mask) = pin::current_affinity() {
        info!("startup affinity: {}", pin::affinity_summary(&mask));
    }

    let pin_cores: Vec<usize> = match cli.pin.as_deref() {
        None => Vec::new(),
        // A spec naming a config profile expands to its core list;
        // anything else parses as a raw core spec.
        Some(spec) => match pin::parse_cores(config.resolve_pin(spec)) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: --pin: {e}");
                std::process::exit(2);
            }
        },
    };

    // Pin main for calibration. Always pinned by default so
    // framing/loop numbers come from a stable environment
    // regardless of --pin. --no-pin-cal restores pre-0.6.0
    // behavior (pin iff --pin given).
    //
    // When we're doing the cal-only pin (--pin absent, --no-pin-cal
    // absent), snapshot the pre-cal affinity and restore it after
    // calibrate() so the scheduler regains freedom to co-locate
    // bench threads — otherwise main's pin leaks into the
    // scheduler's placement decisions and suppresses the
    // "both ends hot and spinning" fast path.
    let cal_pin: Option<usize> = if cli.no_pin_cal {
        None
    } else {
        pin_cores.first().copied().or(Some(0))
    };
    // Save/restore pre-cal affinity whenever we're about to set a
    // cal pin that wasn't the user's explicit bench-pin request —
    // i.e. the cal-only pin (--pin absent, --no-pin-cal absent).
    // --pin ⇒ user already wants main on pin[0]; leave alone.
    // --no-pin-cal ⇒ we aren't pinning at all; nothing to save.
    let saved_affinity = if pin_cores.is_empty() && !cli.no_pin_cal {
        pin::save_affinity()
    } else {
        None
    };
    pin::pin_current(cal_pin);

    if let Some(cpu) = cal_pin {
        info!("pinned main to core {cpu} for calibration");
    } else {
        info!("cal pin skipped");
    }
    if let Some(mask) = pin::current_affinity() {
        debug!("affinity during cal: {}", pin::affinity_summary(&mask));
    }

    let amp_coeff = overhead::N_HIGH as f64 / (overhead::N_HIGH - overhead::N_LOW) as f64;
    info!(
        "calibration params: warmup={}, N_LOW={} ({}x{} windows), \
         N_HIGH={} ({}x{} windows), noise_amp={:.4}",
        overhead::CAL_WARMUP,
        overhead::N_LOW,
        overhead::W_LOW_WINDOWS,
        overhead::W_LOW_SAMPLES,
        overhead::N_HIGH,
        overhead::W_HIGH_WINDOWS,
        overhead::W_HIGH_SAMPLES,
        amp_coeff,
    );

    let overhead = overhead::calibrate();

    // Warm the one-time TSC tick-rate calibration (a ~10 ms spin
    // behind a OnceLock) here on the pinned main thread. Without
    // this the first TProbe::new in a bench thread pays it inside
    // the measurement window — a short -d (e.g. 0.01) was consumed
    // entirely by calibration and recorded zero samples.
    let ticks_per_ns = ticks::ticks_per_ns();
    debug!("ticks_per_ns: {ticks_per_ns:.6}");

    debug!(
        "calibration raw: w_low={:.4} ns, w_high={:.4} ns",
        overhead.cal_w_low_ns, overhead.cal_w_high_ns,
    );
    debug!(
        "calibration fit: frame_call={:.4} ns, loop_per_iter={:.4} ns",
        overhead.frame_call_ns, overhead.loop_per_iter_ns,
    );
    info!(
        "calibration wall time: {:.2} ms",
        overhead.cal_duration.as_secs_f64() * 1000.0
    );

    if let Some(set) = saved_affinity.as_ref() {
        pin::restore_affinity(set);
    }

    let cal_pin_display = match (pin_cores.first(), cli.no_pin_cal) {
        (_, true) => "none (--no-pin-cal)".to_string(),
        (Some(c), false) => format!("core {c} (from --pin)"),
        (None, false) => "core 0 (unpinned after cal; --no-pin-cal to skip)".to_string(),
    };
    println!("Calibration:");
    println!(
        "  frame/sample      {:>7} ns  (call-to-call, amortized; sizes inner)",
        harness::fmt_commas_f64(overhead.frame_call_ns, 3)
    );
    println!(
        "  loop/iter         {:>7} ns  (per inner-loop iteration; subtracted)",
        harness::fmt_commas_f64(overhead.loop_per_iter_ns, 3)
    );
    println!("  cal pin           {cal_pin_display}");
    println!("  bench pin         {}", pin::plan_summary(&pin_cores));
    println!("  sleep inhibit     {inhibit_status}");
    println!("  config            {}", config_summary(&config_files));
    println!();

    let runners = match benches::resolve(&cli.benches) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };

    // Duration precedence: CLI -d / -D win, then the config
    // `duration`, then the built-in default.
    let target_seconds = match (cli.duration, cli.total_duration) {
        (Some(d), _) => d,
        (None, Some(t)) => t / runners.len() as f64,
        (None, None) => config.duration.unwrap_or(DEFAULT_DURATION),
    };

    let cfg = harness::RunCfg {
        overhead: &overhead,
        target_seconds,
        outer_override: cli.outer,
        inner_override: cli.inner,
        pin_cores: &pin_cores,
        report_ticks: cli.ticks,
        band_labels: cli
            .band_labels
            .or(config.band_labels)
            .unwrap_or(DEFAULT_BAND_LABELS),
        decimals: cli.decimals.or(config.decimals).unwrap_or(DEFAULT_DECIMALS) as usize,
    };

    for run in runners {
        run(&cfg);
    }
}
