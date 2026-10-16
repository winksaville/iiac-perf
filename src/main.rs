mod analyze;
mod band_table;
mod bands;
mod benches;
mod binary;
mod child;
mod cli;
mod config;
mod dither;
mod figures;
mod freq;
mod freqctl;
mod gauge;
mod harness;
mod host;
mod inhibit;
mod init_config;
mod inner;
mod linear_regression;
mod md_fence;
mod pin;
mod probe;
mod qualify;
mod record;
mod report;
mod resolution;
mod run_config;
mod runs;
mod series;
mod setup;
mod ticks;
mod timespec;
mod tprobe;
mod tprobe2;
mod wrap;

use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;
use log::{debug, info};
use run_config::{Param, Source, layered};

/// The binary's own name, the package name at build time, so a
/// build under the dev name (`iiac-perf-dev`, per the cycle's
/// rename) names itself that way everywhere it names itself: the
/// banner, the shell completion hook, and every "run this" hint. Config paths and service identifiers stay `iiac-perf`,
/// since both builds share one config and one namespace.
pub const BIN_NAME: &str = env!("CARGO_PKG_NAME");

/// One-line name + version banner, shared by clap's `about` and
/// every runtime entry (bench runs, the no-benches listing), so
/// the header is identical everywhere.
const ABOUT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    " ",
    env!("CARGO_PKG_VERSION"),
    " - Rust latency microbenchmark harness",
);

/// Default seconds per `qualify` child run. Short on
/// purpose: the selftest wants many fresh processes rather than a
/// few long ones, since a respawn is what re-rolls the box's
/// state.
const QUALIFY_CHILD_SECONDS: f64 = 1.0;

/// Default `qualify` child runs, where a bench's `--runs` defaults to
/// [`DEFAULT_RUNS`]: the selftest wants many short processes.
const QUALIFY_RUNS: u64 = 10;

/// Default runs per bench: ten fresh processes, what the `10-50` trim needs to keep four, the
/// project config's and the default since 2026-10-01.
const DEFAULT_RUNS: u64 = 10;

/// Default seconds of measurement per run: 0.25 s, the project config's and the default since
/// 2026-10-01, ten runs of it costing about 17 s a bench with the sleeps.
const DEFAULT_DURATION: f64 = 0.25;
const DEFAULT_BAND_LABELS: bands::BandLabels = bands::BandLabels::Both;
/// Default decimals on the time columns: 3, the picosecond recording floor, which a 2 ns ring
/// needs to show a difference.
const DEFAULT_DECIMALS: u8 = 3;
/// Default placement: the nearest the host declares, `smt` where a core has two threads and
/// `ccx` where it has one, since a comparison pins and a name suits every host.
const DEFAULT_PIN_CPUS: &str = "nearest";
/// Default clock pin: the host's own `pin_mhz`, else its base clock, since a pinned clock is what
/// makes a claim honest rather than tight.
const DEFAULT_PIN_FREQ: config::PinFreq = config::PinFreq::PinMhz;

/// What this host lacks for a run that pins the clock when `pin_clock` and pins `spec`'s
/// placements: the `[freq]` steady state a pin returns to, the permission to write the clock,
/// and the profiles the placements name, each a line saying what is missing. Empty when the
/// host is ready, and when the run pins nothing.
fn readiness_gaps(config: &config::Config, pin_clock: bool, spec: Option<&str>) -> Vec<String> {
    let first = |e: String| e.lines().next().unwrap_or_default().to_string();
    let mut gaps = Vec::new();
    if pin_clock {
        if let Err(e) = freqctl::check_steady(config.freq.as_ref()) {
            gaps.push(format!("the clock: {}", first(e)));
        }
        if !setup::permissions_ready() {
            gaps.push(
                "the permissions: the cpufreq files a pin writes are not yours to write"
                    .to_string(),
            );
        }
    }
    // Only a host with no profiles at all lacks setup's: a name a set-up host does not declare, or
    // one misspelled, is the ordinary placement error, which setup could not fix.
    if config.profiles.is_empty()
        && let Some(s) =
            spec.filter(|s| !s.is_empty() && !s.starts_with(|c: char| c.is_ascii_digit()))
        && let Err(e) = config.placements(s)
    {
        gaps.push(format!("the placement: {}", first(e)));
    }
    gaps
}

/// The line an error that stops a run before its banner ends with: the binary's short hash and
/// version, which the banner would have shown, so a failure from a stale install names the build
/// that failed. Hashed only when a run fails this early.
fn binary_line() -> String {
    match binary::Binary::this() {
        Ok(b) => format!(
            "  binary {}, {BIN_NAME} {}",
            b.short(),
            env!("CARGO_PKG_VERSION")
        ),
        Err(e) => format!("  binary unreadable: {e}"),
    }
}

/// A host not set up for the run: name what it lacks, and on a terminal offer `setup --apply`,
/// then run the same command again once it is set up. Off a terminal, or declined, exit 2 naming
/// the command and the opt-outs.
fn not_set_up(gaps: &[String]) -> ! {
    eprintln!("This host is not set up for this run:");
    for g in gaps {
        eprintln!("  - {g}");
    }
    eprintln!("{}", binary_line());
    let terminal = unsafe { libc::isatty(0) == 1 && libc::isatty(2) == 1 };
    if terminal {
        eprint!("Set it up now with `{BIN_NAME} setup --apply`? [y/N] ");
        let mut answer = String::new();
        // OK: an unreadable answer is no answer, which declines.
        std::io::stdin().read_line(&mut answer).unwrap_or_default();
        if matches!(answer.trim(), "y" | "Y" | "yes") {
            eprintln!();
            let code = setup::run_setup(true);
            if code != 0 {
                std::process::exit(code);
            }
            eprintln!();
            eprintln!("Set up. Running the command again.");
            eprintln!();
            // The same command from the start, reading the config setup just wrote.
            use std::os::unix::process::CommandExt;
            let err = match std::env::current_exe() {
                Ok(exe) => std::process::Command::new(exe)
                    .args(std::env::args_os().skip(1))
                    .exec(),
                Err(e) => e,
            };
            eprintln!("error: running the command again: {err}");
            std::process::exit(1);
        }
    }
    eprintln!(
        "Run `{BIN_NAME} setup` to see what it would do and `{BIN_NAME} setup --apply` to do it, \
         or run unpinned with --pin-cpus \"\" --pin-freq=no."
    );
    std::process::exit(2);
}

/// Load the layered config, exiting with the usage status on any
/// error: a malformed config is fatal so a typo surfaces. Shared by
/// the bench path and the freq command words, which need the
/// declared `[freq]` steady state.
fn load_config_or_exit(named: Option<&std::path::Path>) -> config::Config {
    match config::load(named) {
        Ok((c, _)) => c,
        Err(e) => {
            eprintln!("error: config: {e}");
            std::process::exit(2);
        }
    }
}

/// Banner text listing which config files were loaded, the highest priority first, where
/// `files` is in load order, or `"none (built-in defaults)"` when no file exists.
fn config_summary(files: &[std::path::PathBuf]) -> String {
    if files.is_empty() {
        "none (built-in defaults)".to_string()
    } else {
        files
            .iter()
            .rev()
            .map(|p| run_config::display_path(p))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Wrap a name list into comma-separated lines of at most `width`
/// columns, each line indented two spaces: the no-benches
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

fn main() {
    // Shell completion: when the shell set COMPLETE, answer with
    // the candidates and exit before anything else runs.
    CompleteEnv::with_factory(cli::Cli::command).complete();
    let cli = cli::Cli::parse();

    // The bench-name listing is a pure print-and-exit path: no
    // logging, no config, no setup.
    if cli.list_benches {
        for name in benches::names() {
            println!("{name}");
        }
        return;
    }

    // A bench child runs its one bench from the parent's spec and exits: no config, no inhibit,
    // no clock pin, and no banner, all of which the parent owns, its `-v` included.
    if let Some(spec) = &cli.child_spec {
        init_logger(cli.verbose == Some(true));
        std::process::exit(child::child_main(spec));
    }

    match cli.command {
        None => print!("{}", bench_list("-h, --help, or run")),
        Some(cli::Command::Run(mut run)) => {
            if let Err(e) = run.take_config_arg() {
                eprintln!("error: {e}");
                std::process::exit(2);
            }
            bench_run(BenchLine {
                benches: std::mem::take(&mut run.benches),
                config: run.config.as_deref(),
                measure: &run.measure,
                verbose: cli.verbose,
                suggest: None,
            });
        }
        // 'qualify' respawns --runs times and grades the box across those runs. It re-execs
        // children itself, so it takes neither the inhibit nor the banner path a bench run needs.
        Some(cli::Command::Qualify(q)) => {
            let config = match config::load(None) {
                Ok((c, _)) => c,
                Err(e) => {
                    eprintln!("error: config: {e}");
                    eprintln!("{}", binary_line());
                    std::process::exit(2);
                }
            };
            let (verbose, _) = layered(
                cli.verbose,
                "--verbose",
                config.verbose,
                "verbose",
                &config,
                false,
            );
            init_logger(verbose);
            println!("{ABOUT}\n");
            std::process::exit(qualify::run(&qualify::QualifyCfg {
                runs: q.runs,
                run_sleep_s: q.run_sleep,
                duration_s: q.duration,
                pin_cpus: q.pin_cpus,
                print_only: q.print_only,
                settle_time: q.settle_time,
            }));
        }
        // The record commands read records and print: no config, no setup, nothing measured.
        Some(cli::Command::Analyze(a)) => {
            // Records where the sides belong get the line to run, before a bare key could take
            // the path for one.
            let mut compares = Vec::new();
            for spec in &a.compare {
                if let Some(hint) = analyze::compare_hint(spec) {
                    eprintln!("error: analyze: --compare: {hint}");
                    std::process::exit(2);
                }
                match analyze::Sides::parse(spec) {
                    Ok(sides) => compares.push(sides),
                    Err(e) => {
                        eprintln!("error: analyze: --compare: {e}");
                        std::process::exit(2);
                    }
                }
            }
            std::process::exit(analyze::run(
                &a.records.paths,
                &a.by,
                &compares,
                a.records.trim_runs,
            ));
        }
        Some(cli::Command::LinearRegression(a)) => std::process::exit(linear_regression::run(
            &a.records.paths,
            a.records.trim_runs,
            a.overhead,
        )),
        Some(cli::Command::Figures(a)) => {
            let plan = figures::Plan {
                benches: a.bench,
                show: a.show,
                x: a.x_axis,
                series: a.series,
                trim: a.records.trim_runs,
            };
            std::process::exit(figures::run(&a.records.paths, &plan, &a.out));
        }
        // The record's field dictionary, documenting outputs the way --help documents inputs.
        Some(cli::Command::DescribeRecord) => {
            println!("{ABOUT}\n");
            record::describe();
        }
        Some(cli::Command::Freq(command)) => freq(command, cli.verbose),
        // Setup reads the live clock state and the XDG config itself, so it needs neither the
        // layered config nor the banner.
        Some(cli::Command::Setup(setup)) => std::process::exit(match setup.part {
            None => setup::run_setup(setup.apply),
            Some(cli::SetupCommand::Freq) => setup::run(setup.apply),
            Some(cli::SetupCommand::Uninstall) => setup::run_uninstall(setup.apply),
        }),
        Some(cli::Command::Config(command)) => config_command(command, cli.verbose),
        Some(cli::Command::Unknown(words)) => {
            eprintln!("error: {}", cli::unknown_word(&words));
            std::process::exit(2);
        }
    }
}

/// What a bare `iiac-perf`, and a `run` that names no benches anywhere, print: the banner, what
/// to try, `try_` first, the line that runs benches, and the benches it takes.
fn bench_list(try_: &str) -> String {
    format!(
        "{ABOUT}\n\nTry {try_}:\n\n  {BIN_NAME} run BENCH...\n\nBENCH...:\n{}\n",
        wrap_names(&benches::names(), 72)
    )
}

/// Refuse the retired `--record` before any setup, since an exit past the clock pin skips its
/// restore.
fn refuse_retired_record(m: &cli::MeasureArgs) {
    if m.record.is_some() {
        eprintln!(
            "error: --record is gone, since a path's trailing '/' picked its mode: \
             --record-dir DIR writes a file per invocation, --record-file PATH appends every \
             record to one file"
        );
        std::process::exit(2);
    }
}

/// The `freq` commands: read, pin, or restore the clock and exit, or suggest a pin from a bench's
/// load.
fn freq(command: cli::FreqCommand, verbose: Option<bool>) {
    match command {
        // Prints and exits: no root, no config, no banner, so a prompt or status bar can call it
        // every few seconds.
        cli::FreqCommand::Read { as_config } => {
            std::process::exit(freqctl::cmd_read_freq(as_config));
        }
        // Pin and restore mutate the box on request and exit, both reading the config for the
        // declared [freq] steady state.
        cli::FreqCommand::Pin { target, config } => {
            let target = match config::PinFreq::from_flag(target.as_deref()) {
                Ok(config::PinFreq::Off) => {
                    eprintln!("error: freq pin: \"no\" pins nothing; use freq restore to unpin");
                    std::process::exit(2);
                }
                Ok(t) => t,
                Err(e) => {
                    eprintln!("error: freq pin: {e}");
                    std::process::exit(2);
                }
            };
            let config = load_config_or_exit(config.as_deref());
            std::process::exit(freqctl::cmd_pin_freq(
                config.freq.as_ref(),
                target,
                config.source("freq"),
            ));
        }
        cli::FreqCommand::Restore { config } => {
            let config = load_config_or_exit(config.as_deref());
            std::process::exit(freqctl::cmd_restore_freq(
                config.freq.as_ref(),
                config.source("freq"),
            ));
        }
        // The candidate descent replaces the bench loop, driving the one bench through the run
        // a bench line gets: inhibit, config, Setup block, knobs.
        cli::FreqCommand::Suggest(s) => {
            let cli::SuggestArgs {
                bench,
                config,
                measure,
            } = s;
            if measure.pin_freq.is_some() {
                eprintln!("error: freq suggest pins for itself; drop --pin-freq");
                std::process::exit(2);
            }
            bench_run(BenchLine {
                benches: Vec::new(),
                config: config.as_deref(),
                measure: &measure,
                verbose,
                suggest: Some(bench),
            });
        }
    }
}

/// The `config` commands: write a starting config, or rewrite one in place, the run parameters on
/// the line setting their keys.
fn config_command(command: cli::ConfigCommand, verbose: Option<bool>) {
    let line = |m: &cli::MeasureArgs, name: &str| {
        refuse_retired_record(m);
        match cli::line_values(m, verbose) {
            Ok(table) => table,
            Err(e) => {
                eprintln!("error: config {name}: {e}");
                std::process::exit(2);
            }
        }
    };
    match command {
        cli::ConfigCommand::Init(i) => {
            let start = if let Some(old) = i.from.as_deref() {
                init_config::Start::From(old)
            } else if let Some(name) = i.config.as_deref() {
                init_config::Start::Named(name)
            } else if let Some(file) = i.from_record.as_deref() {
                init_config::Start::Record(file, i.series.as_deref())
            } else {
                init_config::Start::Host
            };
            let existing = match (i.backup, i.overwrite) {
                (true, _) => init_config::Existing::Backup,
                (false, true) => init_config::Existing::Overwrite,
                (false, false) => init_config::Existing::Refuse,
            };
            std::process::exit(init_config::run(
                i.path.as_deref(),
                start,
                line(&i.measure, "init"),
                existing,
            ));
        }
        cli::ConfigCommand::Update(u) => {
            std::process::exit(init_config::update(
                &u.file,
                u.backup,
                line(&u.measure, "update"),
            ));
        }
    }
}

/// What a bench run is asked for: the benches named on the line, the run's config file, the run
/// parameters, `--verbose`, and for freq suggest the one bench whose clock it suggests.
struct BenchLine<'a> {
    benches: Vec<String>,
    config: Option<&'a std::path::Path>,
    measure: &'a cli::MeasureArgs,
    verbose: Option<bool>,
    suggest: Option<String>,
}

/// Run the benches a line names, each in fresh processes, under the layered config, or
/// freq suggest's descent over its one bench.
fn bench_run(line: BenchLine) {
    let m = line.measure;
    refuse_retired_record(m);
    // Layered defaults (built-in < XDG file < project-local file <
    // CLI). A malformed config is fatal so a typo surfaces. Loaded
    // before the logger and the inhibit, since `verbose` and
    // `inhibit` are keys.
    let (config, config_files) = match config::load(line.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: config: {e}");
            eprintln!("{}", binary_line());
            std::process::exit(2);
        }
    };
    let (verbose, verbose_src) = layered(
        line.verbose,
        "--verbose",
        config.verbose,
        "verbose",
        &config,
        false,
    );
    init_logger(verbose);

    // Bench names on the line and --benches both name the benches, so a run takes one.
    if (!line.benches.is_empty() || line.suggest.is_some()) && !m.benches_flag.is_empty() {
        eprintln!("error: --benches and bench names on the line both name the benches: keep one");
        std::process::exit(2);
    }

    if line.suggest.is_none()
        && line.benches.is_empty()
        && m.benches_flag.is_empty()
        && config.benches.is_none()
    {
        print!("{}", bench_list("run -h, or"));
        return;
    }

    // The bench list: the positional names, else --benches, else the config's `benches`, checked
    // before anything prints. freq suggest's one bench is its own, checked where it resolves.
    let suggesting = line.suggest.is_some();
    let (bench_list, benches_src) = match (&line.suggest, line.benches.is_empty()) {
        (Some(name), _) => (vec![name.clone()], Source::Flag("command line".to_string())),
        (None, false) => (
            line.benches.clone(),
            Source::Flag("command line".to_string()),
        ),
        (None, true) => {
            let flag = if m.benches_flag.is_empty() {
                None
            } else {
                Some(m.benches_flag.clone())
            };
            layered(
                flag,
                "--benches",
                config.benches.clone(),
                "benches",
                &config,
                Vec::new(),
            )
        }
    };
    if !suggesting && let Err(e) = cli::check_bench_words(&bench_list) {
        eprintln!("error: {e}");
        std::process::exit(2);
    }

    // Pin the clock before anything measures or prints, so the
    // Setup block and the warm loop both see the pinned state. The
    // flag wins, then the config's pin_freq, and freq suggest never
    // takes a config pin, since it pins for itself. The guard
    // restores the declared steady state on drop (normal exit and
    // panic) and via the signal path on SIGINT/SIGTERM.
    let cli_pin = match &m.pin_freq {
        None => None,
        Some(value) => match config::PinFreq::from_flag(value.as_deref()) {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("error: --pin-freq: {e}");
                std::process::exit(2);
            }
        },
    };
    let (pin_setting, pin_freq_src) = layered(
        cli_pin,
        "--pin-freq",
        config.pin_freq,
        "pin_freq",
        &config,
        DEFAULT_PIN_FREQ,
    );
    let pin_target = match (suggesting, pin_setting) {
        (true, _) | (false, config::PinFreq::Off) => None,
        (false, target) => Some(target),
    };
    // The placement spec, the flag's, a file's, or the default `nearest`. `--pin-cpus ""` runs
    // unpinned.
    let (pin_cpus_spec, pin_cpus_src) = match run_config::layered_opt(
        m.pin_cpus.clone(),
        "--pin-cpus",
        config.pin_cpus.clone(),
        "pin_cpus",
        &config,
    ) {
        (None, _) => (Some(DEFAULT_PIN_CPUS.to_string()), Source::Default),
        given => given,
    };
    // A host not set up for what this run pins is asked to be, all at once, before anything
    // pins or prints.
    let gaps = readiness_gaps(&config, pin_target.is_some(), pin_cpus_spec.as_deref());
    if !gaps.is_empty() {
        not_set_up(&gaps);
    }

    // Re-exec under systemd-inhibit (unless turned off or already inhibited) before any output,
    // so the banner prints once, from the inhibited child, and after the readiness check, so a
    // run refused for setup neither takes the inhibit nor reports its refusal through it.
    let (inhibit, inhibit_src) = layered(
        m.no_inhibit.map(|no| !no),
        "--no-inhibit",
        config.inhibit,
        "inhibit",
        &config,
        true,
    );
    let inhibit_status = inhibit::ensure(inhibit, &run_config::source_name(&inhibit_src));
    // Resolved before the clock pins, so a placement that fails exits with no pin to restore.
    // The placements the invocation runs at: one for a CPU list or one name, several for a name
    // list or `all`, each profile checked against the host.
    let placements: Vec<pin::Placement> = match pin_cpus_spec.as_deref() {
        None => vec![pin::Placement {
            name: None,
            cpus: Vec::new(),
        }],
        Some(spec) => match config.placements(spec).and_then(|ps| {
            for p in ps.iter().filter(|p| p.is_profile()) {
                pin::check_named(&p.label(), &p.cpus)?;
            }
            Ok(ps)
        }) {
            Ok(ps) => ps,
            Err(e) => {
                eprintln!("error: pin_cpus: {e}");
                eprintln!("{}", binary_line());
                std::process::exit(2);
            }
        },
    };
    // Several placements pin nothing in this process: each child pins its own main, and this
    // one only spawns and waits.
    let several_placements = placements.len() > 1;
    let pin_cpus: Vec<usize> = if several_placements {
        Vec::new()
    } else {
        placements[0].cpus.clone()
    };

    let freq_pin = match pin_target {
        None => None,
        Some(target) => {
            match freqctl::RunPin::engage(config.freq.as_ref(), target, config.source("freq")) {
                Ok(g) => Some(g),
                Err(e) => {
                    eprintln!("error: pin_freq: {e}");
                    eprintln!("{}", binary_line());
                    std::process::exit(2);
                }
            }
        }
    };

    println!("{ABOUT}\n");

    if let Some(mask) = pin::current_affinity() {
        info!("startup affinity: {}", pin::affinity_summary(&mask));
    }

    // Pin main to the pool's first slot when --pin-cpus is given: thread 0 of a bench measures
    // on main, and the warm loop is a real timing phase converging on per-CPU frequency state,
    // so it must run where measurement will run. Without --pin-cpus, main stays wherever the
    // scheduler has it: a busy thread stays put, and the warm state lands on the CPU that
    // measures. The retired CPU0-default warm pin parked the warm on the kernel's busiest CPU
    // for no measured benefit: the tick-rate read is a ratio that cancels interruptions (~8e-7
    // spread across CPUs), and nothing else ran pinned.
    pin::remember_startup_affinity();
    if let Some(&cpu) = pin_cpus.first() {
        pin::pin_current(Some(cpu));
        info!("pinned main to CPU {cpu} (bench pin pool slot 0)");
    }
    if let Some(mask) = pin::current_affinity() {
        debug!("affinity for warm + run: {}", pin::affinity_summary(&mask));
    }

    // Warm the one-time TSC tick-rate calibration (a ~10 ms spin behind a OnceLock) here on
    // main. Without this the first TProbe::new in a bench thread pays it inside the measurement
    // window: a short -d (e.g. 0.01) was consumed entirely by that spin and recorded zero
    // samples.
    let ticks_per_ns = ticks::ticks_per_ns();
    debug!("ticks_per_ns: {ticks_per_ns:.6}");

    // Every layered knob: the flag wins, then the config file, then the built-in, each value
    // keeping its source for the Config: list. Negative is rejected rather than clamped: it
    // means the caller expected something we don't do.
    let (settle_time, settle_time_src) = layered(
        m.settle_time,
        "--settle-time",
        config.settle_time,
        "settle_time",
        &config,
        harness::DEFAULT_SETTLE_TIME_S,
    );
    if settle_time < 0.0 {
        eprintln!("error: --settle-time must be zero or more, got {settle_time}");
        std::process::exit(2);
    }
    let (warm_cap, warm_cap_src) = layered(
        m.warm_cap,
        "--warm-cap",
        config.warm_cap,
        "warm_cap",
        &config,
        harness::DEFAULT_WARM_CAP_S,
    );
    if warm_cap < 0.0 {
        eprintln!("error: --warm-cap must be zero or more, got {warm_cap}");
        std::process::exit(2);
    }
    // Every run has blocks, so the sleep and warmup knobs below need no gate.
    let (blocks, blocks_src) = layered(
        m.blocks,
        "--blocks",
        config.blocks,
        "blocks",
        &config,
        harness::DEFAULT_BLOCKS,
    );
    // The sleep defaults to a short range so every run's blocks are replicates, and the warmup
    // to zero so no sample is discarded unless asked.
    let cli_block_sleep = match m.block_sleep.as_deref() {
        None => None,
        Some(s) => match timespec::parse_span(s) {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("error: --block-sleep: {e}");
                std::process::exit(2);
            }
        },
    };
    let (block_sleep_s, block_sleep_src) = layered(
        cli_block_sleep,
        "--block-sleep",
        config.block_sleep,
        "block_sleep",
        &config,
        harness::DEFAULT_BLOCK_SLEEP_S,
    );
    let cli_block_warmup = match m.block_warmup.as_deref() {
        None => None,
        Some(s) => match timespec::parse_scalar(s) {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("error: --block-warmup: {e}");
                std::process::exit(2);
            }
        },
    };
    let (block_warmup_s, block_warmup_src) = layered(
        cli_block_warmup,
        "--block-warmup",
        config.block_warmup,
        "block_warmup",
        &config,
        harness::DEFAULT_BLOCK_WARMUP_S,
    );
    // Runs per bench, each a fresh process, and the sleep before each run after the first.
    let (runs, runs_src) = layered(m.runs, "--runs", config.runs, "runs", &config, DEFAULT_RUNS);
    let cli_run_sleep = match m.run_sleep.as_deref() {
        None => None,
        Some(s) => match timespec::parse_span(s) {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("error: --run-sleep: {e}");
                std::process::exit(2);
            }
        },
    };
    let (run_sleep_s, run_sleep_src) = layered(
        cli_run_sleep,
        "--run-sleep",
        config.run_sleep,
        "run_sleep",
        &config,
        runs::DEFAULT_RUN_SLEEP_S,
    );
    let cli_trim_runs = match m.trim_runs.as_deref() {
        None => None,
        Some(s) => match series::Trim::parse(s) {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("error: --trim-runs: {e}");
                std::process::exit(2);
            }
        },
    };
    let (trim_runs, trim_runs_src) = layered(
        cli_trim_runs,
        "--trim-runs",
        config.trim_runs,
        "trim_runs",
        &config,
        series::Trim::DEFAULT,
    );

    // Main's placement covers the warm loop and thread 0 of every bench, so the cell names
    // both.
    // The binary, hashed once here and handed to every child, so each record names the build that
    // measured it rather than a version two builds can share.
    let binary = match binary::Binary::this() {
        Ok(b) => Some(b),
        Err(e) => {
            eprintln!("warning: binary: {e}");
            None
        }
    };
    let main_pin_display = match pin_cpus.first() {
        Some(c) => format!("CPU {c} (pool slot 0; warm + run)"),
        None if several_placements => {
            "none (several placements, each child pins its own)".to_string()
        }
        None => "none (scheduler placement)".to_string(),
    };
    // The box's clock and power policy, printed before any bench so every archived report says
    // what machine produced it. No report before 0.25.0 recorded the policy, which left an 8.9%
    // governor delta indistinguishable from a code change in any A/B spanning one.
    let policy = freq::policy();
    let boost = policy.boost.as_ref().map(|f| freq::PolicyField {
        value: boost_word(&f.value).to_string(),
        uniform: f.uniform,
    });
    println!("Setup:");
    match &binary {
        Some(b) => println!("  binary            {}", b.summary()),
        None => println!("  binary            unreadable"),
    }
    println!("  ticks/ns          {ticks_per_ns:.6}");
    println!("  tick period       {:.3} ns", 1.0 / ticks_per_ns);
    println!(
        "  cpufreq driver    {}",
        policy_cell(policy.driver.as_ref())
    );
    println!(
        "  governor          {}",
        policy_cell(policy.governor.as_ref())
    );
    println!("  EPP               {}", policy_cell(policy.epp.as_ref()));
    println!("  boost             {}", policy_cell(boost.as_ref()));
    println!("  main pin          {main_pin_display}");
    if several_placements {
        for (i, p) in placements.iter().enumerate() {
            let head = if i == 0 { "  bench pin" } else { "" };
            println!("{head:<20}{:<9} {}", p.label(), pin::plan_summary(&p.cpus));
        }
    } else {
        println!("  bench pin         {}", pin::plan_summary(&pin_cpus));
    }
    if let Some(g) = &freq_pin {
        println!(
            "  freq pin          {} MHz ({}; min = max, boost off; restores on exit)",
            g.khz / 1000,
            g.source
        );
    }
    println!("  sleep inhibit     {inhibit_status}");
    println!();

    let suggest = line.suggest.clone();

    let resolve_args: Vec<String> = match &suggest {
        Some(name) => vec![name.clone()],
        None => bench_list.clone(),
    };
    let runners = match benches::resolve(&resolve_args) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };
    if suggest.is_some() && runners.len() != 1 {
        eprintln!(
            "error: freq suggest names one bench exactly (a prefix matching several \
             does not say which schedule the suggestion serves)"
        );
        std::process::exit(2);
    }

    // Duration precedence: CLI -d / -D win, then the config's
    // `duration` or `total_duration`, of which the files leave one
    // at most, then the built-in default. A total is split over
    // every run, and the duration row says so.
    let (total_duration, total_duration_src) = match m.duration {
        Some(_) => (None, Source::Default),
        None => run_config::layered_opt(
            m.total_duration,
            "--total-duration",
            config.total_duration,
            "total_duration",
            &config,
        ),
    };
    let (target_seconds, duration_src) = match total_duration {
        Some(t) => (
            t / (runners.len() as u64 * runs) as f64,
            Source::Flag(format!(
                "total_duration over {} benches x {runs} runs",
                runners.len()
            )),
        ),
        None => layered(
            m.duration,
            "-d",
            config.duration,
            "duration",
            &config,
            DEFAULT_DURATION,
        ),
    };
    let (band_labels, band_labels_src) = layered(
        m.band_labels,
        "--band-labels",
        config.band_labels,
        "band_labels",
        &config,
        DEFAULT_BAND_LABELS,
    );
    let (decimals, decimals_src) = layered(
        m.decimals,
        "--decimals",
        config.decimals,
        "decimals",
        &config,
        DEFAULT_DECIMALS,
    );

    // Every run parameter with its value and source, so a reader can tell a default from a
    // file's value from a flag, and a source restating the default is marked. The block knobs
    // print zeros included: an invisible sleep shaping results is the failure mode the knobs
    // replaced.
    let (samples, samples_src) =
        run_config::layered_opt(m.samples, "--samples", config.samples, "samples", &config);
    let (inner, inner_src) =
        run_config::layered_opt(m.inner, "--inner", config.inner, "inner", &config);
    let (env_probe, env_probe_src) = layered(
        m.no_env_probe.map(|no| !no),
        "--no-env-probe",
        config.env_probe,
        "env_probe",
        &config,
        true,
    );
    let (report_ticks, ticks_src) =
        layered(m.ticks, "--ticks", config.ticks, "ticks", &config, false);
    let cli_record = match (&m.record_dir, &m.record_file) {
        (Some(dir), _) => Some((record::Target::Dir(dir.clone()), "--record-dir")),
        (None, Some(file)) => Some((record::Target::File(file.clone()), "--record-file")),
        (None, None) => None,
    };
    let (record_target, record_src) = match cli_record {
        Some((target, flag)) => (Some(target), Source::Flag(flag.to_string())),
        None => run_config::layered_opt(None, "", config.record.clone(), "record", &config),
    };
    // The label names a --record-dir file, and a --record-file target is named by its path, so
    // the flag would promise a name it cannot give. A `label` tag stays a plain tag in either.
    if m.record_label.is_some()
        && let Some(record::Target::File(path)) = &record_target
    {
        eprintln!(
            "error: --record-label names a --record-dir file, and {} is named already: drop \
             --record-label, or keep the label in the records with --tag label=NAME",
            run_config::display_path(path)
        );
        drop(freq_pin);
        std::process::exit(2);
    }
    // The files' tags first and the line's after, so the line wins on a shared key, each as
    // the `KEY=VALUE` the recorder reads.
    let mut tag_map = config.tags.clone();
    for tag in &m.tag {
        match tag.split_once('=') {
            Some((k, v)) if !k.is_empty() => tag_map.insert(k.to_string(), v.to_string()),
            _ => {
                eprintln!("error: --tag '{tag}' is not key=value");
                std::process::exit(2);
            }
        };
    }
    if let Some(label) = &m.record_label {
        if m.tag.iter().any(|t| t.starts_with("label=")) {
            eprintln!("error: --record-label and --tag label= both name the label: keep one");
            drop(freq_pin);
            std::process::exit(2);
        }
        tag_map.insert(record::LABEL_TAG.to_string(), label.clone());
    }
    let tags: Vec<String> = tag_map.iter().map(|(k, v)| format!("{k}={v}")).collect();
    let tag_flag = match (m.tag.is_empty(), m.record_label.is_some()) {
        (true, false) => None,
        (true, true) => Some("--record-label"),
        (false, _) => Some("--tag"),
    };
    let tags_src = match (config.source("tags"), tag_flag) {
        (None, None) => Source::Default,
        (None, Some(flag)) => Source::Flag(flag.to_string()),
        (Some(path), None) => Source::File(path.to_path_buf()),
        (Some(path), Some(flag)) => {
            Source::Flag(format!("{} and {flag}", run_config::display_path(path)))
        }
    };
    if record_target.is_none() && !tags.is_empty() {
        eprintln!(
            "error: tags: a tag needs a record target, from --record-dir, --record-file, or the \
             config's record_dir or record_file"
        );
        std::process::exit(2);
    }
    // A profile name shows what it resolved to, so the banner says which CPUs a name meant. The
    // spec resolved above, so an error here cannot happen and reads as the spec itself.
    let pin_cpus_value = match pin_cpus_spec.as_deref() {
        None => "none".to_string(),
        Some(spec) if several_placements => {
            let each: Vec<String> = placements
                .iter()
                .map(|p| {
                    if p.is_profile() {
                        format!("{} {}", p.label(), cpus_list(&p.cpus))
                    } else {
                        p.label()
                    }
                })
                .collect();
            format!("{spec} = {}", each.join(", "))
        }
        Some("") => "none".to_string(),
        // A name stands for its CPUs, and `nearest` for the profile it chose as well.
        Some(spec) => match &placements[0] {
            p if p.is_profile() && p.label() != spec => {
                format!("{spec} = {} {}", p.label(), cpus_list(&p.cpus))
            }
            p if p.is_profile() => format!("{spec} = {}", cpus_list(&p.cpus)),
            _ => spec.to_string(),
        },
    };
    // The pin's resolved target, whichever layer named it, so a record says what clock the run
    // held rather than that a pin was asked for.
    let pin_freq_value = match (&freq_pin, pin_setting) {
        (Some(g), _) => format!("{} MHz ({})", g.khz / 1000, g.source),
        (None, config::PinFreq::Off) => "no".to_string(),
        (None, _) => "not engaged: freq suggest pins for itself".to_string(),
    };
    let params = [
        Param::new(
            "benches",
            match &suggest {
                Some(name) => name.clone(),
                None => bench_list.join(", "),
            },
            "none",
            benches_src,
        ),
        Param::new(
            "runs",
            runs.to_string(),
            &DEFAULT_RUNS.to_string(),
            runs_src,
        ),
        Param::new(
            "run_sleep",
            span_value(run_sleep_s),
            &span_value(runs::DEFAULT_RUN_SLEEP_S),
            run_sleep_src,
        ),
        Param::new(
            "trim_runs",
            trim_runs.to_string(),
            &series::Trim::DEFAULT.to_string(),
            trim_runs_src,
        ),
        Param::new(
            "duration",
            seconds_value(target_seconds),
            &seconds_value(DEFAULT_DURATION),
            duration_src,
        ),
        Param::new(
            "total_duration",
            total_duration.map_or("none".to_string(), seconds_value),
            "none",
            total_duration_src,
        ),
        Param::new(
            "samples",
            samples.map_or("auto".to_string(), |n| n.to_string()),
            "auto",
            samples_src,
        ),
        Param::new(
            "inner",
            inner.map_or("auto".to_string(), |n| n.to_string()),
            "auto",
            inner_src,
        ),
        Param::new("pin_cpus", pin_cpus_value, DEFAULT_PIN_CPUS, pin_cpus_src),
        Param::new("pin_freq", pin_freq_value, "pin_mhz", pin_freq_src),
        // The declared [freq] steady state, which no run reads unless it pins but every pin and
        // restore returns to, so a table set in a config shows where it came from.
        Param::new(
            "freq",
            match &config.freq {
                Some(f) => f.summary(),
                None => "none declared".to_string(),
            },
            "none declared",
            match config.source("freq") {
                Some(path) => Source::File(path.to_path_buf()),
                None => Source::Default,
            },
        ),
        Param::new(
            "blocks",
            blocks.to_string(),
            &harness::DEFAULT_BLOCKS.to_string(),
            blocks_src,
        ),
        Param::new(
            "block_sleep",
            span_value(block_sleep_s),
            &span_value(harness::DEFAULT_BLOCK_SLEEP_S),
            block_sleep_src,
        ),
        Param::new(
            "block_warmup",
            seconds_value(block_warmup_s),
            &seconds_value(harness::DEFAULT_BLOCK_WARMUP_S),
            block_warmup_src,
        ),
        Param::new(
            "settle_time",
            seconds_value(settle_time),
            &seconds_value(harness::DEFAULT_SETTLE_TIME_S),
            settle_time_src,
        ),
        Param::new(
            "warm_cap",
            seconds_value(warm_cap),
            &seconds_value(harness::DEFAULT_WARM_CAP_S),
            warm_cap_src,
        ),
        Param::new(
            "band_labels",
            band_labels.as_str().to_string(),
            DEFAULT_BAND_LABELS.as_str(),
            band_labels_src,
        ),
        Param::new(
            "decimals",
            decimals.to_string(),
            &DEFAULT_DECIMALS.to_string(),
            decimals_src,
        ),
        Param::new(
            "env_probe",
            if env_probe { "on" } else { "off" }.to_string(),
            "on",
            env_probe_src,
        ),
        Param::new(
            "ticks",
            if report_ticks { "ticks" } else { "ns" }.to_string(),
            "ns",
            ticks_src,
        ),
        Param::new(
            "inhibit",
            if inhibit { "on" } else { "off" }.to_string(),
            "on",
            inhibit_src,
        ),
        Param::new(
            "verbose",
            if verbose { "on" } else { "off" }.to_string(),
            "off",
            verbose_src,
        ),
        Param::new(
            "record",
            record_target
                .as_ref()
                .map_or("none".to_string(), record::Target::describe),
            "none",
            record_src,
        ),
        Param::new(
            "tags",
            if tags.is_empty() {
                "none".to_string()
            } else {
                tags.join(", ")
            },
            "none",
            tags_src,
        ),
    ];
    println!("Config:");
    println!("  files             {}", config_summary(&config_files));
    for line in run_config::lines(&params) {
        println!("{line}");
    }
    println!();

    // The record sink resolves before any bench runs, so a bad
    // path or tag fails in milliseconds rather than after minutes
    // of measuring.
    // The run as a config file spells it, from the files that gave it its keys, the named file
    // alone under --config NAME, and the line's flags. A failure drops the clock pin first, since
    // an exit runs no destructor.
    let run_files = match (line.config, config_files.last()) {
        (Some(_), Some(named)) => std::slice::from_ref(named),
        _ => config_files.as_slice(),
    };
    // The benches are the ones the run resolved, positional words included, which `line_values`
    // does not carry, since a positional on a `config init` line is its path.
    let run_benches = match &suggest {
        Some(name) => vec![name.clone()],
        None => bench_list.clone(),
    };
    let run_table = match cli::line_values(m, line.verbose).and_then(|mut line| {
        let names = run_benches.into_iter().map(toml::Value::String).collect();
        line.insert("benches".to_string(), toml::Value::Array(names));
        init_config::run_table(run_files, &line)
    }) {
        Ok(table) => table,
        Err(e) => {
            eprintln!("error: record: the run's config: {e}");
            drop(freq_pin);
            std::process::exit(2);
        }
    };
    let record_config = record::RecordConfig::new(&config_files, &params).with_run(run_table);
    let recorder = match &record_target {
        None => None,
        Some(target) => match record::Recorder::new(target.clone(), &tags, record_config.clone()) {
            Ok(mut r) => {
                r.set_binary(binary.clone());
                Some(r)
            }
            Err(e) => {
                eprintln!("error: record: {e}");
                std::process::exit(2);
            }
        },
    };

    // freq suggest runs its one bench in this process, so its record reads the pool against that
    // bench's threads. A child looks up its own bench's.
    let roles = match &suggest {
        Some(_) => runners[0].roles.threads(0),
        None => Vec::new(),
    };
    let cfg = harness::RunCfg {
        target_seconds,
        samples_override: samples,
        inner_override: inner,
        pin_cpus: &pin_cpus,
        pin_name: if several_placements {
            None
        } else {
            placements[0].name.as_deref()
        },
        roles: &roles,
        report_ticks,
        seam_probes: env_probe,
        band_labels,
        decimals: decimals as usize,
        settle_time_s: settle_time,
        warm_cap_s: warm_cap,
        blocks,
        block_sleep_s,
        block_warmup_s,
        record: recorder.as_ref(),
    };

    if let Some(name) = &suggest {
        if several_placements {
            eprintln!(
                "error: freq suggest runs at one placement, and pin_cpus names {}",
                placements.len()
            );
            std::process::exit(2);
        }
        std::process::exit(freqctl::cmd_suggest_freq(
            config.freq.as_ref(),
            config.source("freq"),
            name,
            runners[0].run,
            &cfg,
        ));
    }

    // Every bench runs in a child of its own, so none inherits the placement another drew. The
    // children get the resolved knobs and the record sink, and this process holds the sleep
    // inhibit and the clock pin for all of them.
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: current_exe: {e}");
            std::process::exit(1);
        }
    };
    let scratch = match child::ScratchDir::new() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };
    // The record target goes to the children absolute.
    let record_spec = child::RecordSpec {
        target: record_target.as_ref().map(record::Target::absolute),
        tags,
        config: record_config,
        series: record::new_series_id(),
        binary: binary.clone(),
    };
    let mut runner = runs::Runner::new(runs::Plan {
        exe: &exe,
        scratch: scratch.path(),
        runs,
        run_sleep_s,
        verbose,
        decimals: decimals as usize,
        trim_runs,
    });
    // Placements outer and benches inner, so each placement's rows run back to back as one run
    // of a single placement would. A bench whose threads would use the CPUs an earlier
    // placement's run used, a 1t bench at every pair sharing a first CPU, is not run again.
    let mut table = runs::PlacementTable::new(&placements, &runners);
    let mut ran: Vec<(&str, Vec<Option<usize>>, usize)> = Vec::new();
    for (col, p) in placements.iter().enumerate() {
        let pcfg = harness::RunCfg {
            pin_cpus: &p.cpus,
            pin_name: p.name.as_deref(),
            ..cfg
        };
        if several_placements {
            println!("Placement {}: {}\n", p.label(), pin::placement(&p.cpus));
        }
        for (row, entry) in runners.iter().enumerate() {
            let used = pin::thread_cpus(&p.cpus, entry.roles.threads(0).len());
            if let Some(&(_, _, earlier)) = ran
                .iter()
                .find(|(name, cpus, at)| *name == entry.name && *cpus == used && *at != col)
            {
                println!(
                    "{}: at {}, not run: its threads use the CPUs they used at {}\n",
                    entry.name,
                    p.label(),
                    placements[earlier].label()
                );
                table.same(row, col, earlier);
                continue;
            }
            ran.push((entry.name, used, col));
            match runner.bench(entry.name, &pcfg, &record_spec) {
                Ok(means) => table.ran(row, col, means),
                Err(e) => {
                    eprintln!("error: {e}");
                    drop(scratch);
                    drop(freq_pin);
                    std::process::exit(1);
                }
            }
        }
    }
    if several_placements {
        table.print(decimals as usize);
    }
}

/// A CPU list as a spec spells it, `11,23`.
fn cpus_list(cpus: &[usize]) -> String {
    cpus.iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// Start the logger. The default filter is `warn`, and verbose bumps it to `debug`. `RUST_LOG`
/// (if set) always wins, so users can still do fine-grained per-module filtering without
/// fighting the flag.
fn init_logger(verbose: bool) {
    let mut builder = env_logger::Builder::from_default_env();
    if std::env::var_os("RUST_LOG").is_none() {
        builder.filter_level(if verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Warn
        });
    }
    builder.format_timestamp(None).init();
}

/// Render one `Setup:` policy cell: the token, marked when CPUs disagree, or why it is absent.
///
/// - `None` prints `not exposed` and never a default. A box without the file has no such
///   policy, and inventing one is exactly what makes an archived number ambiguous.
/// - A non-uniform field carries `(mixed across CPUs)`: one CPU's token is not the box's policy
///   when the policy groups were set separately.
fn policy_cell(field: Option<&freq::PolicyField>) -> String {
    match field {
        None => "not exposed".to_string(),
        Some(f) if f.uniform => f.value.clone(),
        Some(f) => format!("{} (mixed across CPUs)", f.value),
    }
}

/// A span of seconds as the `Config:` list prints it: `0`, one duration, or a range.
fn span_value(span: (f64, f64)) -> String {
    if span.1 <= 0.0 {
        "0".to_string()
    } else if span.0 == span.1 {
        timespec::display(span.0)
    } else {
        let (lo, hi) = (timespec::display(span.0), timespec::display(span.1));
        // One unit for both ends reads as the flag is typed, `1-10 ms`.
        match (lo.split_once(' '), hi.split_once(' ')) {
            (Some((lo_n, lo_u)), Some((hi_n, hi_u))) if lo_u == hi_u => {
                format!("{lo_n}-{hi_n} {hi_u}")
            }
            _ => format!("{lo}-{hi}"),
        }
    }
}

/// Seconds as the `Config:` list prints them, zero as `0` rather than `0 us`.
fn seconds_value(s: f64) -> String {
    if s <= 0.0 {
        "0".to_string()
    } else {
        timespec::display(s)
    }
}

/// `boost`'s raw sysfs token as a word. Anything unrecognized passes through untranslated
/// rather than being guessed at.
fn boost_word(raw: &str) -> &str {
    match raw {
        "1" => "enabled",
        "0" => "disabled",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_names_the_placement_it_lacks_and_nothing_when_it_pins_none() {
        let path = std::path::Path::new("host.toml");
        let bare = config::parse_text(path, "blocks = 10\n").unwrap();
        let gaps = readiness_gaps(&bare, false, Some("nearest"));
        assert_eq!(gaps.len(), 1, "{gaps:?}");
        assert!(
            gaps[0].starts_with("the placement: \"nearest\""),
            "{gaps:?}"
        );
        // Unpinned, a CPU list, and a declared name need nothing of the host.
        assert!(readiness_gaps(&bare, false, Some("")).is_empty());
        assert!(readiness_gaps(&bare, false, Some("11,23")).is_empty());
        let host = config::parse_text(path, "[profiles]\nccx = \"3,2\"\n").unwrap();
        assert!(readiness_gaps(&host, false, Some("nearest")).is_empty());
        assert!(readiness_gaps(&host, false, Some("ccx")).is_empty());
        // A name a set-up host lacks is a placement error, not a setup gap.
        assert!(readiness_gaps(&host, false, Some("smt")).is_empty());
        assert!(readiness_gaps(&host, false, Some("nosuch")).is_empty());
    }

    fn policy_field(value: &str, uniform: bool) -> freq::PolicyField {
        freq::PolicyField {
            value: value.to_string(),
            uniform,
        }
    }

    #[test]
    fn policy_cell_says_absent_rather_than_defaulting() {
        assert_eq!(policy_cell(None), "not exposed");
    }

    #[test]
    fn policy_cell_marks_a_split_policy() {
        assert_eq!(
            policy_cell(Some(&policy_field("powersave", true))),
            "powersave"
        );
        assert_eq!(
            policy_cell(Some(&policy_field("powersave", false))),
            "powersave (mixed across CPUs)"
        );
    }

    #[test]
    fn boost_word_translates_only_the_known_tokens() {
        assert_eq!(boost_word("1"), "enabled");
        assert_eq!(boost_word("0"), "disabled");
        assert_eq!(boost_word("unexpected"), "unexpected");
    }

    #[test]
    fn config_values_read_as_typed() {
        assert_eq!(span_value((0.0, 0.0)), "0");
        assert_eq!(span_value((0.002, 0.002)), "2 ms");
        assert_eq!(span_value((0.001, 0.010)), "1-10 ms");
        assert_eq!(span_value((0.0005, 0.002)), "500 us-2 ms");
        assert_eq!(seconds_value(0.0), "0");
        assert_eq!(seconds_value(1.5), "1.5 s");
    }

    #[test]
    fn the_bench_list_says_what_to_type_and_lists_the_benches() {
        let text = bench_list("-h, --help, or run");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], ABOUT);
        assert_eq!(lines[2], "Try -h, --help, or run:");
        assert_eq!(lines[4], format!("  {BIN_NAME} run BENCH..."));
        assert_eq!(lines[6], "BENCH...:");
        assert!(lines[7].starts_with("  min-now, "), "{text}");
    }

    #[test]
    fn wrap_names_single_line() {
        assert_eq!(wrap_names(&["a", "b"], 72), "  a, b");
    }

    #[test]
    fn wrap_names_breaks_at_width() {
        // "ccc" would land past col 10, so it wraps, and the separator
        // comma stays on the prior line and the new line re-indents.
        assert_eq!(wrap_names(&["aaa", "bbb", "ccc"], 10), "  aaa, bbb,\n  ccc");
    }
}
