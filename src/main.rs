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

use clap::{CommandFactory, Parser};
use log::{debug, info};

/// One-line name + version banner, shared by clap's `about` and
/// every runtime entry (bench runs, the calibrate command, the
/// no-benches listing), so the header is identical everywhere.
const ABOUT: &str = concat!(
    "iiac-perf ",
    env!("CARGO_PKG_VERSION"),
    " — Rust latency microbenchmark harness",
);

/// The reserved-word commands block, shared by `--help`'s
/// after-help and the no-benches listing so the two stay in sync.
const COMMANDS_HELP: &str = concat!(
    "Commands:\n",
    "  all        run every registered bench\n",
    "  calibrate  run calibration only and print the constants plus the raw\n",
    "             fit inputs (dithered points, alternative fits, ticks/ns).\n",
    "             Must stand alone; --pin, --no-pin-cal, and -v apply as usual.",
);

#[derive(Parser)]
#[command(version, about = ABOUT, max_term_width = 80, after_help = COMMANDS_HELP)]
struct Cli {
    /// Benches to run, or a command word ('all', 'calibrate').
    ///
    /// Pass 'all' for every registered bench, or one or more
    /// names; a name matching no bench exactly runs every bench
    /// it is a prefix of (e.g. 'ice', 'mpsc'). Pass 'calibrate'
    /// (alone) to run calibration only and print the constants
    /// plus raw fit inputs — no bench runs. Run with no args to
    /// see the available list.
    benches: Vec<String>,

    /// Target wall-clock seconds per bench.
    ///
    /// Default 5.0, or the config `duration`; auto-sizes outer
    /// and inner loop counts. Mutually exclusive with -D.
    #[arg(short = 'd', long, conflicts_with = "total_duration")]
    duration: Option<f64>,

    /// Target total wall-clock seconds across all benches.
    ///
    /// The budget is split equally per bench. Mutually exclusive
    /// with -d.
    #[arg(short = 'D', long)]
    total_duration: Option<f64>,

    /// Override outer loop count (skips auto-sizing; inner still adapts).
    #[arg(short, long)]
    outer: Option<u64>,

    /// Override inner loop count (skips auto-sizing).
    ///
    /// inner=1 measures single-call latency (each sample = one
    /// step); higher inner measures back-to-back/burst rate
    /// (each sample = N steps averaged).
    #[arg(short, long)]
    inner: Option<u64>,

    /// Pin bench threads to logical CPUs (comma-separated, ranges OK).
    ///
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

    /// Skip pinning the main thread for calibration.
    ///
    /// Always takes effect, including when `--pin` is set — bench
    /// threads still pin per `--pin`, but main stays on whatever
    /// affinity mask the process launched with. By default
    /// (without this flag), calibration runs with main pinned to
    /// `pin[0]` if set, else core 0, so framing/loop come from a
    /// stable environment. Pass this flag to decouple the
    /// calibration environment from the bench pinning pool (or to
    /// reproduce pre-0.6.0 behavior when `--pin` is absent).
    #[arg(long)]
    no_pin_cal: bool,

    /// Enable verbose internals on stderr (like `RUST_LOG=debug`).
    ///
    /// Shows affinity mask, cal parameters and raw fit inputs,
    /// calibration wall time. Default is `warn` (silent unless
    /// something's wrong). `RUST_LOG` overrides this flag when
    /// set, so per-module filtering still works.
    #[arg(short, long)]
    verbose: bool,

    /// Show tprobe results in raw TSC ticks, not nanoseconds.
    ///
    /// Only affects `TProbe` output; `Probe` results are always
    /// in nanoseconds.
    #[arg(short = 't', long)]
    ticks: bool,

    /// Band label style for the report's histogram rows.
    ///
    /// 'zpn': nines/zeros + decile names (z3, p50, n4).
    /// 'frac': literal boundary fractions with '_' grouping
    /// (0.001, 0.50, 0.999_9). 'both': zpn and fraction
    /// side by side — the juxtaposition teaches the zpn
    /// vocabulary; switch to 'zpn' once fluent. Overrides the
    /// config `band_labels`; both absent defaults to 'both'.
    #[arg(long, value_enum)]
    band_labels: Option<bands::BandLabels>,

    /// Decimal digits on the report's time columns (0-3).
    ///
    /// 1 shows the sub-ns precision picosecond recording
    /// captures; 0 restores integer ns; 3 is the recording
    /// floor - more digits would be artifacts. Overrides the
    /// config `decimals`; both absent defaults to 1.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=3))]
    decimals: Option<u8>,

    /// Divide the run into N sleep-separated measurement blocks.
    ///
    /// E.g. `--blocks 10 -d 10` = 10 blocks of ~1 s each, with
    /// block-replication stats reported (mean blocks / CI95 /
    /// LSC). Each block sleeps a random 1-10 ms (re-rolls
    /// scheduler and frequency state) and warms up unrecorded
    /// before measuring — neither is counted in the budget. The
    /// spread of block means yields a per-run 95% confidence
    /// interval and least-significant-change estimate.
    /// Bench-driven benches only; probe benches ignore it.
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u64).range(2..=1000))]
    blocks: Option<u64>,

    /// Do not inhibit system sleep for the run.
    ///
    /// By default the process re-execs itself under
    /// `systemd-inhibit --what=sleep` so an idle-suspend can't
    /// poison a long measurement. Pass this to keep the process
    /// image untouched (strace/gdb/perf wrappers), to let the
    /// machine sleep on purpose, or to test the suspend-detection
    /// WARNING path (a sleep inhibitor also blocks manual
    /// `systemctl suspend`).
    #[arg(long)]
    no_inhibit: bool,

    /// Print a shell-completion artifact to stdout and exit.
    ///
    /// No bench runs: a static script for bash, zsh, fish,
    /// elvish, or powershell, or a spec for the carapace-bin
    /// multi-shell engine. Install e.g. `iiac-perf --completions
    /// bash > ~/.local/share/bash-completion/completions/iiac-perf`,
    /// or `iiac-perf --completions carapace >
    /// ~/.config/carapace/specs/iiac-perf.yaml`.
    #[arg(long, value_enum, value_name = "SHELL")]
    completions: Option<CompletionShell>,

    /// Print the registered bench names, one per line, and exit.
    ///
    /// No bench runs. Machine-readable: the carapace spec's
    /// exec-macro calls this at completion time for dynamic
    /// bench-name candidates (see --completions), and scripts can
    /// iterate it. The 'all' / 'calibrate' command words are not
    /// bench names and are not listed.
    #[arg(long, conflicts_with = "completions")]
    list_benches: bool,
}

/// `--completions` output formats: the five clap_complete static
/// shells plus the carapace spec (one YAML consumed by
/// carapace-bin, which serves all of its shells from it).
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum CompletionShell {
    /// Static bash script.
    Bash,
    /// Static zsh script.
    Zsh,
    /// Static fish script.
    Fish,
    /// Static elvish script.
    Elvish,
    /// Static PowerShell script.
    Powershell,
    /// carapace-bin YAML spec (all carapace-supported shells).
    Carapace,
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

/// Print the `--completions` artifact for the chosen shell to
/// stdout: a clap_complete static script, or the carapace YAML
/// spec via the same `Generator` interface.
fn print_completions(shell: CompletionShell) {
    use clap_complete::{Shell, generate};
    let mut cmd = Cli::command();
    let out = &mut std::io::stdout();
    match shell {
        CompletionShell::Bash => generate(Shell::Bash, &mut cmd, "iiac-perf", out),
        CompletionShell::Zsh => generate(Shell::Zsh, &mut cmd, "iiac-perf", out),
        CompletionShell::Fish => generate(Shell::Fish, &mut cmd, "iiac-perf", out),
        CompletionShell::Elvish => generate(Shell::Elvish, &mut cmd, "iiac-perf", out),
        CompletionShell::Powershell => generate(Shell::PowerShell, &mut cmd, "iiac-perf", out),
        CompletionShell::Carapace => {
            let mut buf = Vec::new();
            generate(carapace_spec_clap::Spec, &mut cmd, "iiac-perf", &mut buf);
            print!(
                "{}",
                inject_bench_candidates(&String::from_utf8_lossy(&buf))
            );
        }
    }
}

/// Inject dynamic bench-name completion into the generated carapace
/// spec: a `positionalany` list whose exec-macro asks the installed
/// binary (`--list-benches`) on every Tab, plus the two static
/// command words. The generator emits only what clap knows, so the
/// positional would otherwise complete to nothing.
fn inject_bench_candidates(spec: &str) -> String {
    let block = concat!(
        "completion:\n",
        "  positionalany:\n",
        "  - \"$(iiac-perf --list-benches)\"\n",
        "  - \"all\\trun every registered bench\"\n",
        "  - \"calibrate\\tcalibration only; print constants + raw fit inputs\"\n",
    );
    if spec.contains("completion:\n") {
        spec.replacen("completion:\n", block, 1)
    } else {
        format!("{spec}{block}")
    }
}

/// Wrap a name list into comma-separated lines of at most `width`
/// columns, each line indented two spaces — the no-benches
/// listing's counterpart of clap's two-column help style.
fn wrap_names(names: &[&str], width: usize) -> String {
    let mut out = String::new();
    let mut col = 0;
    for name in names {
        if out.is_empty() {
            out.push_str("  ");
        } else if col + 2 + name.len() <= width {
            out.push_str(", ");
        } else {
            out.push_str(",\n  ");
            col = 0;
        }
        out.push_str(name);
        col += 2 + name.len();
    }
    out
}

/// Print the `calibrate` command's diagnostic block: raw fit
/// inputs (window minimum, both dithered points), the alternative
/// fits, the TSC tick rate, and the calibration wall time — the
/// stdout counterpart of the `-v` debug logs, for frequency-regime
/// fingerprinting without running a bench.
fn print_raw_calibration(o: &overhead::Overhead, ticks_per_ns: f64) {
    println!("Raw fit inputs:");
    println!("  ticks/ns          {ticks_per_ns:.6}");
    println!(
        "  w_low             {:.3} ns/sample  (min window mean, {}x{} @ N={})",
        o.cal_w_low_ns,
        overhead::W_LOW_WINDOWS,
        overhead::W_LOW_SAMPLES,
        overhead::N_LOW,
    );
    for (name, n, p) in [
        ("d_low ", overhead::N_LOW, &o.cal_d_low),
        ("d_high", overhead::N_HIGH, &o.cal_d_high),
    ] {
        println!(
            "  {name} @ N={n:<6} mean {:.3} | p99mean {:.3} | medwin {:.3} | spread {:.3} | min {} ns",
            p.mean_ns, p.mean_p99_ns, p.median_window_ns, p.window_spread_ns, p.min_ns,
        );
    }
    println!();
    println!("Alternative fits (production fit is p99):");
    for (kind, low, high) in [
        ("full  ", o.cal_d_low.mean_ns, o.cal_d_high.mean_ns),
        ("p99   ", o.cal_d_low.mean_p99_ns, o.cal_d_high.mean_p99_ns),
        (
            "medwin",
            o.cal_d_low.median_window_ns,
            o.cal_d_high.median_window_ns,
        ),
    ] {
        let (frame_sample, loop_per_iter) = overhead::two_point_fit(low, high);
        println!("  {kind}  frame/sample {frame_sample:.4} ns, loop/iter {loop_per_iter:.6} ns");
    }
    println!();
    println!(
        "  cal wall time     {:.2} ms",
        o.cal_duration.as_secs_f64() * 1000.0
    );
}

fn main() {
    let cli = Cli::parse();

    // Completion generation and the bench-name listing are pure
    // print-and-exit paths: no logging, no config, no calibration.
    if let Some(shell) = cli.completions {
        print_completions(shell);
        return;
    }
    if cli.list_benches {
        for name in benches::names() {
            println!("{name}");
        }
        return;
    }

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
        println!("{ABOUT}\n");
        println!("no benches specified. use -h or --help for more info.\n");
        println!("Benches:");
        println!("{}\n", wrap_names(&benches::names(), 72));
        println!("{COMMANDS_HELP}");
        return;
    }

    // 'calibrate' is a command, not a bench: calibration + the
    // diagnostic block below, no bench run. It stands alone so a
    // typo'd mix doesn't half-run something.
    let calibrate_cmd = cli.benches.iter().any(|b| b == "calibrate");
    if calibrate_cmd && cli.benches.len() > 1 {
        eprintln!("error: 'calibrate' runs alone; drop the other bench args");
        std::process::exit(2);
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

    println!("{ABOUT}\n");

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
        "calibration params: warmup={}, dither N_LOW={} ({}x{}), \
         N_HIGH={} ({}x{}), span={}, w_low {}x{}, noise_amp={:.4}",
        overhead::CAL_WARMUP,
        overhead::N_LOW,
        overhead::DITHER_WINDOWS,
        overhead::DITHER_LOW_SAMPLES,
        overhead::N_HIGH,
        overhead::DITHER_WINDOWS,
        overhead::DITHER_HIGH_SAMPLES,
        overhead::DITHER_SPAN,
        overhead::W_LOW_WINDOWS,
        overhead::W_LOW_SAMPLES,
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
        "calibration raw: w_low={:.4} ns, d_low_p99={:.4} ns, d_high_p99={:.4} ns",
        overhead.cal_w_low_ns, overhead.cal_d_low.mean_p99_ns, overhead.cal_d_high.mean_p99_ns,
    );
    debug!(
        "calibration fit: frame_call={:.4} ns, frame_sample={:.4} ns, loop_per_iter={:.4} ns",
        overhead.frame_call_ns, overhead.frame_sample_ns, overhead.loop_per_iter_ns,
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
        "  frame/call        {:>7} ns  (call-to-call, amortized; sizes inner)",
        harness::fmt_commas_f64(overhead.frame_call_ns, 3)
    );
    println!(
        "  frame/sample      {:>7} ns  (in-interval, dithered; subtracted /inner)",
        harness::fmt_commas_f64(overhead.frame_sample_ns, 3)
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

    if calibrate_cmd {
        print_raw_calibration(&overhead, ticks_per_ns);
        return;
    }

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
        blocks: cli.blocks,
    };

    for run in runners {
        run(&cfg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_names_single_line() {
        assert_eq!(wrap_names(&["a", "b"], 72), "  a, b");
    }

    #[test]
    fn inject_bench_candidates_precedes_flag_section() {
        let spec = "name: x\ncompletion:\n  flag:\n    a:\n    - one\n";
        let out = inject_bench_candidates(spec);
        // positionalany lands under the existing completion key,
        // before the flag section; the exec-macro is quoted.
        let pos = out.find("  positionalany:").expect("positionalany missing");
        let flag = out.find("  flag:").expect("flag section missing");
        assert!(pos < flag);
        assert!(out.contains("\"$(iiac-perf --list-benches)\""));
        assert_eq!(out.matches("completion:\n").count(), 1);
    }

    #[test]
    fn inject_bench_candidates_appends_when_no_completion_key() {
        let out = inject_bench_candidates("name: x\n");
        assert!(
            out.ends_with("- \"calibrate\\tcalibration only; print constants + raw fit inputs\"\n")
        );
        assert!(out.contains("completion:\n  positionalany:\n"));
    }

    #[test]
    fn wrap_names_breaks_at_width() {
        // "ccc" would land past col 10, so it wraps; the separator
        // comma stays on the prior line and the new line re-indents.
        assert_eq!(wrap_names(&["aaa", "bbb", "ccc"], 10), "  aaa, bbb,\n  ccc");
    }
}
