mod band_table;
mod bands;
mod benches;
mod config;
mod dither;
mod freq;
mod freqctl;
mod gauge;
mod harness;
mod inhibit;
mod md_fence;
mod pin;
mod probe;
mod qualify;
mod record;
mod report;
mod ticks;
mod timespec;
mod tprobe;
mod tprobe2;

use clap::{CommandFactory, Parser};
use log::{debug, info};

/// One-line name + version banner, shared by clap's `about` and
/// every runtime entry (bench runs, the no-benches listing), so
/// the header is identical everywhere.
const ABOUT: &str = concat!(
    "iiac-perf ",
    env!("CARGO_PKG_VERSION"),
    " — Rust latency microbenchmark harness",
);

/// Default seconds per `qualify-environment` child run. Short on
/// purpose: the selftest wants many fresh processes rather than a
/// few long ones, since a respawn is what re-rolls the box's
/// state.
const QUALIFY_CHILD_SECONDS: f64 = 1.0;

/// The reserved-word commands block, shared by `--help`'s
/// after-help and the no-benches listing so the two stay in sync.
const COMMANDS_HELP: &str = concat!(
    "Commands:\n",
    "  all        run every registered bench\n",
    "  qualify-environment\n",
    "             is this machine fit to measure on? Respawns this binary\n",
    "             --runs times at --gap, collects each run's environment grade,\n",
    "             prints the table and a verdict: QUALIFIED when the median\n",
    "             grade is B or better and no run's drift or step reached D/F.\n",
    "             Exits nonzero when not. Grades the environment, not the run —\n",
    "             the machine is the subject, not a workload. Must stand alone;\n",
    "             -d sets each child's duration (default 1s), --pin passes\n",
    "             through, --print-only skips the verdict.\n",
    "  describe-record\n",
    "             print the --record field dictionary: every record key with\n",
    "             its unit and one-line meaning, plus the schema_version the\n",
    "             dictionary describes. --help documents inputs; this documents\n",
    "             the recorded output. Must stand alone.\n",
    "  read-freq  print the clock state, one line per policy group: governor,\n",
    "             EPP, boost, clamp, current frequency, and the base clock\n",
    "             with its source. No root needed; shaped for a prompt or a\n",
    "             status bar. --as-config prints it as a config [freq]\n",
    "             section instead, ready to paste. Must stand alone.\n",
    "  pin-freq [MHZ]\n",
    "             hold the clock still until restore-freq: min = max at MHZ\n",
    "             (default: the config pin_mhz, else the base clock), boost\n",
    "             off. Needs root, and refuses without a declared [freq]\n",
    "             steady state in the config - the way home. Must stand\n",
    "             alone.\n",
    "  restore-freq\n",
    "             converge the box to the config's declared [freq] steady\n",
    "             state (governor, EPP, boost, clamps), from any starting\n",
    "             point, including after an unclean death. Needs root. Must\n",
    "             stand alone.\n",
    "  add-completion-yaml\n",
    "             install Tab completion (bench names, command words, flags)\n",
    "             for any carapace-served shell: generate the carapace spec\n",
    "             and write it to the specs dir as iiac-perf.yaml (see\n",
    "             --completion-dir), creating the dir and overwriting any\n",
    "             existing spec. Run once after install, again after an\n",
    "             upgrade that changes flags or command words (bench names\n",
    "             complete dynamically and never need a re-run). Must stand\n",
    "             alone.",
);

#[derive(Parser)]
#[command(version, about = ABOUT, max_term_width = 80, after_help = COMMANDS_HELP)]
struct Cli {
    /// Benches to run, or a command word ('all',
    /// 'qualify-environment', 'describe-record', 'read-freq',
    /// 'pin-freq', 'restore-freq', 'add-completion-yaml').
    ///
    /// Pass 'all' for every registered bench, or one or more
    /// names; a name matching no bench exactly runs every bench
    /// it is a prefix of (e.g. 'ice', 'mpsc'). Pass
    /// 'qualify-environment' (alone) to ask whether this machine
    /// is fit to measure on. Pass 'describe-record' (alone) to
    /// print the --record field dictionary. Pass 'read-freq',
    /// 'pin-freq [MHZ]', or 'restore-freq' (alone) to read, pin,
    /// or restore the CPU clock. Pass 'add-completion-yaml'
    /// (alone) to write the carapace completion spec to the specs
    /// dir (see --completion-dir). Run with no args to see the
    /// available list.
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

    /// Enable verbose internals on stderr (like `RUST_LOG=debug`).
    ///
    /// Shows the affinity mask, the pin lifecycle, and the TSC
    /// tick rate. Default is `warn` (silent unless something's
    /// wrong). `RUST_LOG` overrides this flag when set, so
    /// per-module filtering still works.
    #[arg(short, long)]
    verbose: bool,

    /// Show tprobe results in raw TSC ticks, not nanoseconds.
    ///
    /// Only affects `TProbe` output; `Probe` results are always
    /// in nanoseconds.
    #[arg(short = 't', long)]
    ticks: bool,

    /// `qualify-environment` only: child runs to spawn.
    #[arg(long, value_name = "N", default_value_t = 10, value_parser = clap::value_parser!(u64).range(1..))]
    runs: u64,

    /// `qualify-environment` only: seconds to sleep before each
    /// child run.
    ///
    /// Zero (the default) sustains the duty cycle that provokes a
    /// state transition; a nonzero gap probes a quieter one.
    #[arg(long, value_name = "SECONDS", default_value_t = 0.0)]
    gap: f64,

    /// `qualify-environment` only: print the table and skip the
    /// verdict.
    #[arg(long)]
    print_only: bool,

    /// `read-freq` only: print the state as a config `[freq]`
    /// section.
    ///
    /// Ready to paste into a `toml` fence of the config file
    /// (usually ~/.config/iiac-perf/config.md), which is how the
    /// steady state that pin-freq and restore-freq need gets
    /// declared.
    #[arg(long)]
    as_config: bool,

    /// Pin the CPU clock for this run, restoring on exit.
    ///
    /// Engages before the warmup, exactly like 'pin-freq': min =
    /// max at MHZ (--pin-freq=3800), else the config pin_mhz,
    /// else the discovered base clock, with boost off. The
    /// declared [freq] steady state is restored on normal exit,
    /// panic, SIGINT, and SIGTERM; after SIGKILL or power loss,
    /// run 'restore-freq'. Needs root and a declared [freq]
    /// steady state.
    #[arg(long, value_name = "MHZ", num_args = 0..=1, require_equals = true)]
    pin_freq: Option<Option<u64>>,

    /// Stop probing the environment at batch seams.
    ///
    /// The environment grade normally samples the box at every
    /// batch boundary, so its letter covers the whole run. This
    /// limits it to the warmup probes, which cover only the few
    /// ms before the bench starts. Use it when the seam probes
    /// disturb the workload — a spinning multi-threaded bench
    /// keeps running through a probe, so its queues drain — or
    /// to A/B whether they do.
    #[arg(long)]
    no_env_probe: bool,

    /// Seconds to warm the box before the first bench measures.
    ///
    /// The first bench of a process otherwise reports a cold
    /// machine's numbers - measured at ~8.6% slow on a 7600x -
    /// while every later bench inherits the boosted state. The
    /// warm is paid once per process, not per bench, and the
    /// grade block's `settle` cell says how long the box actually
    /// took to settle. 0 skips it, which is how you measure what
    /// the warm is worth on a given box. Overrides the config
    /// `settle_time`; both absent defaults to 1.5.
    #[arg(long, value_name = "SECONDS", allow_negative_numbers = true)]
    settle_time: Option<f64>,

    /// Cap on each run's warm-until-stable stretch (seconds).
    ///
    /// Every run warms until the trailing probe window grades A
    /// (and the delivered clock holds still, where readable), or
    /// until this cap. A settled box exits in ~50 ms; the cap
    /// prices only the disturbed case, and hitting it is
    /// reported in the grade block (a "00%" settle cell with an
    /// F, or "uncertified"), never silently absorbed. 0 caps
    /// immediately, which is how you measure what the warm is
    /// worth. Overrides the config `warm_cap`; both absent
    /// defaults to 1.5.
    #[arg(long, value_name = "SECONDS", allow_negative_numbers = true)]
    warm_cap: Option<f64>,

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

    /// Divide the run into N measurement blocks.
    ///
    /// E.g. `--blocks 10 -d 10` = 10 blocks of ~1 s each, each
    /// block's mean one point of the block stats (mean blocks /
    /// CI95 / LSC). Blocks sleep and re-warm between one another
    /// only as --block-sleep / --block-warmup ask (both default
    /// 0; neither is counted in the budget): sleepless blocks are
    /// partitions of one continuous run, so CI95 / LSC print '-'
    /// unless a nonzero --block-sleep makes the blocks genuine
    /// replicates. Blocks nest above batches: each block is a
    /// contiguous stretch of whole batches (batch boundaries
    /// align to the block gaps), so batches stay the grade's
    /// time-series grain and blocks are the replication grain.
    /// Bench-driven benches only; probe benches ignore it.
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u64).range(2..=1000))]
    blocks: Option<u64>,

    /// Sleep between blocks: a duration or range with unit (us, ms, s).
    ///
    /// E.g. '--block-sleep 1-10ms' re-rolls a random sleep per
    /// block (re-rolls scheduler and frequency state; a range
    /// avoids phase-locking with kernel ticks), '--block-sleep 1s'
    /// sleeps exactly 1 s (a long sleep reaches deep C-states, so
    /// wakes start colder). 0 (the default) never sleeps: the
    /// blocks are partitions of one continuous run and the
    /// replication rows print '-'. Requires --blocks. Overrides
    /// the config `block_sleep`.
    #[arg(long, value_name = "SPAN", requires = "blocks")]
    block_sleep: Option<String>,

    /// Unrecorded post-wake warmup per block: a duration with unit.
    ///
    /// Steps the bench unrecorded after each block sleep, keeping
    /// the frequency ramp and cache refill out of the samples. 0
    /// (the default) records from the first post-wake call, which
    /// is how cold-wake behavior is seen. Requires --blocks.
    /// Overrides the config `block_warmup`.
    #[arg(long, value_name = "DUR", requires = "blocks")]
    block_warmup: Option<String>,

    /// Append one NDJSON record per bench result to PATH.
    ///
    /// A side channel, never a mode: the display is unchanged, and
    /// the record is what survives the session (fixed quantile
    /// ladder, block means, seam clock, power policy). The
    /// 'describe-record' command lists every field. The path's
    /// shape picks the mode: end it with '/' (or name an existing
    /// directory) for one file per run, stamped
    /// <ts>-<host>-<bench>.ndjson so a rerun can't clobber
    /// evidence, or name a file to append every record there. The
    /// open never truncates. Probe-style benches produce no
    /// harness result and record nothing.
    #[arg(long, value_name = "PATH")]
    record: Option<std::path::PathBuf>,

    /// Tag every record with KEY=VALUE (repeatable).
    ///
    /// Recorded verbatim, never interpreted: the caller, not the
    /// tool, knows which runs form one experiment, so e.g.
    /// '--tag series=20260816T09' labels a series and '--tag
    /// condition=pinned' a condition. Requires --record.
    #[arg(long, value_name = "KEY=VALUE", requires = "record")]
    tag: Vec<String>,

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
    /// bash > ~/.local/share/bash-completion/completions/iiac-perf`;
    /// for carapace prefer the 'add-completion-yaml' command,
    /// which writes the spec to the specs dir itself.
    #[arg(long, value_enum, value_name = "SHELL")]
    completions: Option<CompletionShell>,

    /// Directory for 'add-completion-yaml' to write iiac-perf.yaml.
    ///
    /// Defaults to $XDG_CONFIG_HOME/carapace/specs, falling back
    /// to ~/.config/carapace/specs — carapace-bin's own spec
    /// lookup dir. The dir is created if missing and an existing
    /// spec is overwritten.
    #[arg(long, value_name = "DIR")]
    completion_dir: Option<std::path::PathBuf>,

    /// Print the registered bench names, one per line, and exit.
    ///
    /// No bench runs. Machine-readable: the carapace spec's
    /// exec-macro calls this at completion time for dynamic
    /// bench-name candidates (see --completions), and scripts can
    /// iterate it. The 'all' command word is not
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

/// Load the layered config, exiting with the usage status on any
/// error: a malformed config is fatal so a typo surfaces. Shared by
/// the bench path and the freq command words, which need the
/// declared `[freq]` steady state.
fn load_config_or_exit() -> config::Config {
    match config::load() {
        Ok((c, _)) => c,
        Err(e) => {
            eprintln!("error: config: {e}");
            std::process::exit(2);
        }
    }
}

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
        CompletionShell::Carapace => print!("{}", carapace_spec()),
    }
}

/// Render the carapace YAML spec (clap-generated plus the dynamic
/// bench-name injection) as a string — shared by `--completions
/// carapace` (stdout) and the 'add-completion-yaml' command (file).
fn carapace_spec() -> String {
    let mut cmd = Cli::command();
    let mut buf = Vec::new();
    clap_complete::generate(carapace_spec_clap::Spec, &mut cmd, "iiac-perf", &mut buf);
    inject_bench_candidates(&String::from_utf8_lossy(&buf))
}

/// Resolve the default carapace specs dir from the environment:
/// `$XDG_CONFIG_HOME/carapace/specs`, falling back to
/// `$HOME/.config/carapace/specs` — mirroring carapace-bin's own
/// spec lookup. `None` when neither variable yields a base.
fn default_completion_dir() -> Option<std::path::PathBuf> {
    completion_dir_from(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

/// Env-free core of [`default_completion_dir`], split out so unit
/// tests can drive it without mutating the process environment.
///
/// - a relative `$XDG_CONFIG_HOME` is ignored, per the XDG spec;
/// - an unset or empty `$HOME` yields `None` rather than a
///   relative `.config` path.
fn completion_dir_from(
    xdg: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
) -> Option<std::path::PathBuf> {
    let base = match xdg.map(std::path::PathBuf::from) {
        Some(p) if p.is_absolute() => p,
        _ => {
            let home = home.filter(|h| !h.is_empty())?;
            std::path::PathBuf::from(home).join(".config")
        }
    };
    Some(base.join("carapace").join("specs"))
}

/// Write the carapace spec to `dir/iiac-perf.yaml`, creating the
/// dir if needed and overwriting any existing spec; returns the
/// written path.
fn write_completion_yaml(dir: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join("iiac-perf.yaml");
    std::fs::write(&path, carapace_spec())?;
    Ok(path)
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
        "  - \"qualify-environment\\tis this machine fit to measure on?\"\n",
        "  - \"describe-record\\tprint the --record field dictionary\"\n",
        "  - \"read-freq\\tprint the CPU clock state\"\n",
        "  - \"pin-freq\\thold the CPU clock still (min = max, boost off)\"\n",
        "  - \"restore-freq\\tconverge to the declared [freq] steady state\"\n",
        "  - \"add-completion-yaml\\twrite the carapace completion spec to the specs dir\"\n",
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

fn main() {
    let cli = Cli::parse();

    // Completion generation and the bench-name listing are pure
    // print-and-exit paths: no logging, no config, no setup.
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

    // 'describe-record' is a pure print-and-exit command word: the
    // record's field dictionary, documenting outputs the way
    // --help documents inputs.
    if cli.benches.iter().any(|b| b == "describe-record") {
        if cli.benches.len() > 1 {
            eprintln!("error: 'describe-record' runs alone; drop the other bench args");
            std::process::exit(2);
        }
        println!("{ABOUT}\n");
        record::describe();
        return;
    }

    // 'add-completion-yaml' is a command word like 'all':
    // self-install the carapace spec, print the path, and exit.
    if cli.benches.iter().any(|b| b == "add-completion-yaml") {
        if cli.benches.len() > 1 {
            eprintln!("error: 'add-completion-yaml' runs alone; drop the other bench args");
            std::process::exit(2);
        }
        let dir = match cli.completion_dir.clone().or_else(default_completion_dir) {
            Some(d) => d,
            None => {
                eprintln!(
                    "error: no specs dir ($XDG_CONFIG_HOME and $HOME unset); pass --completion-dir"
                );
                std::process::exit(2);
            }
        };
        match write_completion_yaml(&dir) {
            Ok(path) => println!("{}", path.display()),
            Err(e) => {
                eprintln!(
                    "error: writing {}: {e}",
                    dir.join("iiac-perf.yaml").display()
                );
                std::process::exit(1);
            }
        }
        return;
    }

    // 'read-freq' prints and exits: no root, no config, no banner,
    // so a prompt or status bar can call it every few seconds.
    if cli.benches.iter().any(|b| b == "read-freq") {
        if cli.benches.len() > 1 {
            eprintln!("error: 'read-freq' runs alone; drop the other bench args");
            std::process::exit(2);
        }
        std::process::exit(freqctl::cmd_read_freq(cli.as_config));
    }

    // 'pin-freq' and 'restore-freq' mutate the box on request and
    // exit. Both read the config for the declared [freq] steady
    // state; pin-freq additionally takes one optional MHZ arg
    // (`pin-freq 3800`).
    if cli.benches.iter().any(|b| b == "pin-freq") {
        if cli.benches[0] != "pin-freq" || cli.benches.len() > 2 {
            eprintln!("error: 'pin-freq' runs alone, with at most one MHZ arg");
            std::process::exit(2);
        }
        let mhz = match cli.benches.get(1) {
            None => None,
            Some(s) => match s.parse::<u64>() {
                Ok(v) => Some(v),
                Err(_) => {
                    eprintln!("error: pin-freq: {s:?} is not a frequency in MHz");
                    std::process::exit(2);
                }
            },
        };
        let config = load_config_or_exit();
        std::process::exit(freqctl::cmd_pin_freq(config.freq.as_ref(), mhz));
    }
    if cli.benches.iter().any(|b| b == "restore-freq") {
        if cli.benches.len() > 1 {
            eprintln!("error: 'restore-freq' runs alone; drop the other bench args");
            std::process::exit(2);
        }
        let config = load_config_or_exit();
        std::process::exit(freqctl::cmd_restore_freq(config.freq.as_ref()));
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
        // Nudge toward completion setup until the spec exists at
        // the default path; silent once installed (or if a custom
        // --completion-dir was used — only the default is checked).
        let spec_missing =
            default_completion_dir().is_some_and(|d| !d.join("iiac-perf.yaml").exists());
        if spec_missing {
            println!("\nFor command completion execute 'iiac-perf add-completion-yaml -h'");
        }
        return;
    }

    // 'qualify-environment' is a command, not a bench: it respawns
    // --runs times and grades the box across those runs. It stands
    // alone, and it re-execs children itself, so it runs before
    // the inhibit/config/banner path a bench run needs.
    if cli.benches.iter().any(|b| b == "qualify-environment") {
        if cli.benches.len() > 1 {
            eprintln!("error: 'qualify-environment' runs alone; drop the other bench args");
            std::process::exit(2);
        }
        println!("{ABOUT}\n");
        let code = qualify::run(&qualify::QualifyCfg {
            runs: cli.runs,
            gap_s: cli.gap,
            duration_s: cli.duration.unwrap_or(QUALIFY_CHILD_SECONDS),
            pin: cli.pin.clone(),
            print_only: cli.print_only,
            settle_time: cli.settle_time,
        });
        std::process::exit(code);
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

    // Pin the clock before anything measures or prints, so the
    // Setup block and the warm loop both see the pinned state. The
    // guard restores the declared steady state on drop (normal
    // exit and panic) and via the signal path on SIGINT/SIGTERM.
    let freq_pin = match cli.pin_freq {
        None => None,
        Some(mhz) => match freqctl::RunPin::engage(config.freq.as_ref(), mhz) {
            Ok(g) => Some(g),
            Err(e) => {
                eprintln!("error: --pin-freq: {e}");
                std::process::exit(2);
            }
        },
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

    // Pin main to the pool's first slot when --pin is given: thread 0 of a bench measures on
    // main, and the warm loop is a real timing phase converging on per-core frequency state, so
    // it must run where measurement will run. Without --pin, main stays wherever the scheduler
    // has it: a busy thread stays put, and the warm state lands on the core that measures. The
    // retired CPU0-default warm pin parked the warm on the kernel's busiest core for no
    // measured benefit: the tick-rate read is a ratio that cancels interruptions (~8e-7 spread
    // across cores), and nothing else ran pinned.
    if let Some(&cpu) = pin_cores.first() {
        pin::pin_current(Some(cpu));
        info!("pinned main to core {cpu} (bench pin pool slot 0)");
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

    // Same precedence as duration: CLI, then config, then the
    // built-in. Negative is rejected rather than clamped: it
    // means the caller expected something we don't do.
    let settle_time = cli
        .settle_time
        .or(config.settle_time)
        .unwrap_or(harness::DEFAULT_SETTLE_TIME_S);
    if settle_time < 0.0 {
        eprintln!("error: --settle-time must be zero or more, got {settle_time}");
        std::process::exit(2);
    }

    // Same precedence as settle time: CLI, then config, then the built-in.
    let warm_cap = cli
        .warm_cap
        .or(config.warm_cap)
        .unwrap_or(harness::DEFAULT_WARM_CAP_S);
    if warm_cap < 0.0 {
        eprintln!("error: --warm-cap must be zero or more, got {warm_cap}");
        std::process::exit(2);
    }

    // Block knobs: CLI wins, then config, then zero. Zero is the
    // neutral setting: a run never sleeps or discards samples
    // unless asked to.
    let block_sleep_s = match cli.block_sleep.as_deref() {
        Some(s) => match timespec::parse_span(s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: --block-sleep: {e}");
                std::process::exit(2);
            }
        },
        None => config.block_sleep.unwrap_or((0.0, 0.0)),
    };
    let block_warmup_s = match cli.block_warmup.as_deref() {
        Some(s) => match timespec::parse_scalar(s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: --block-warmup: {e}");
                std::process::exit(2);
            }
        },
        None => config.block_warmup.unwrap_or(0.0),
    };

    // Main's placement covers the warm loop and thread 0 of every bench, so the cell names
    // both.
    let main_pin_display = match pin_cores.first() {
        Some(c) => format!("core {c} (pool slot 0; warm + run)"),
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
    println!("  bench pin         {}", pin::plan_summary(&pin_cores));
    if let Some(g) = &freq_pin {
        println!(
            "  freq pin          {} MHz ({}; min = max, boost off; restores on exit)",
            g.khz / 1000,
            g.source
        );
    }
    // The block knobs print whenever blocks run, zeros included:
    // an invisible sleep shaping results is the failure mode the
    // knobs replaced.
    if cli.blocks.is_some() {
        println!("  block sleep       {}", sleep_cell(block_sleep_s));
        println!("  block warmup      {}", warmup_cell(block_warmup_s));
    }
    // The budgets, not the spend: each run's report brackets carry
    // its own warm=used/cap, and the grade block's settle cell says
    // when the box settled.
    println!("  warm budget       settle {settle_time}s once + cap {warm_cap}s per run");
    println!("  sleep inhibit     {inhibit_status}");
    println!("  config            {}", config_summary(&config_files));
    println!();

    // The record sink resolves before any bench runs, so a bad
    // path or tag fails in milliseconds rather than after minutes
    // of measuring.
    let recorder = match cli.record.as_deref() {
        None => None,
        Some(path) => match record::Recorder::new(path, &cli.tag) {
            Ok(r) => Some(r),
            Err(e) => {
                eprintln!("error: --record: {e}");
                std::process::exit(2);
            }
        },
    };

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
        target_seconds,
        outer_override: cli.outer,
        inner_override: cli.inner,
        pin_cores: &pin_cores,
        report_ticks: cli.ticks,
        seam_probes: !cli.no_env_probe,
        band_labels: cli
            .band_labels
            .or(config.band_labels)
            .unwrap_or(DEFAULT_BAND_LABELS),
        decimals: cli.decimals.or(config.decimals).unwrap_or(DEFAULT_DECIMALS) as usize,
        settle_time_s: settle_time,
        warm_cap_s: warm_cap,
        blocks: cli.blocks,
        block_sleep_s,
        block_warmup_s,
        record: recorder.as_ref(),
    };

    for run in runners {
        run(&cfg);
    }
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

/// Render the Setup `block sleep` cell from the resolved span (seconds).
fn sleep_cell(span: (f64, f64)) -> String {
    if span.1 <= 0.0 {
        "none (blocks are partitions; CI95/LSC print '-')".to_string()
    } else if span.0 == span.1 {
        format!("{} fixed", timespec::display(span.0))
    } else {
        format!(
            "{}-{} random per block",
            timespec::display(span.0),
            timespec::display(span.1)
        )
    }
}

/// Render the Setup `block warmup` cell from the resolved seconds.
fn warmup_cell(s: f64) -> String {
    if s <= 0.0 {
        "none (records from the first post-wake call)".to_string()
    } else {
        format!("{} unrecorded post-wake", timespec::display(s))
    }
}

/// `boost`'s raw sysfs token as a word; anything unrecognized passes through untranslated
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
        assert!(out.ends_with(
            "- \"add-completion-yaml\\twrite the carapace completion spec to the specs dir\"\n"
        ));
        assert!(out.contains("completion:\n  positionalany:\n"));
    }

    #[test]
    fn completion_dir_prefers_absolute_xdg() {
        let dir = completion_dir_from(Some("/xdg".into()), Some("/home/u".into()));
        assert_eq!(dir, Some("/xdg/carapace/specs".into()));
    }

    #[test]
    fn completion_dir_ignores_relative_xdg() {
        let dir = completion_dir_from(Some("rel".into()), Some("/home/u".into()));
        assert_eq!(dir, Some("/home/u/.config/carapace/specs".into()));
    }

    #[test]
    fn completion_dir_none_without_home() {
        assert_eq!(completion_dir_from(None, None), None);
        assert_eq!(completion_dir_from(None, Some("".into())), None);
    }

    #[test]
    fn wrap_names_breaks_at_width() {
        // "ccc" would land past col 10, so it wraps; the separator
        // comma stays on the prior line and the new line re-indents.
        assert_eq!(wrap_names(&["aaa", "bbb", "ccc"], 10), "  aaa, bbb,\n  ccc");
    }
}
