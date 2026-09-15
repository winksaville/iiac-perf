mod band_table;
mod bands;
mod benches;
mod child;
mod config;
mod dither;
mod freq;
mod freqctl;
mod gauge;
mod harness;
mod host;
mod inhibit;
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

use clap::{CommandFactory, Parser};
use clap_complete::{ArgValueCompleter, CompleteEnv, CompletionCandidate};
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

/// Default seconds per `qualify-environment` child run. Short on
/// purpose: the selftest wants many fresh processes rather than a
/// few long ones, since a respawn is what re-rolls the box's
/// state.
const QUALIFY_CHILD_SECONDS: f64 = 1.0;

/// Default `qualify-environment` child runs, where a bench's `--runs` defaults to
/// [`DEFAULT_RUNS`]: the selftest wants many short processes.
const QUALIFY_RUNS: u64 = 10;

/// Default runs per bench: enough fresh processes for an across-process CI95 and LSC whose
/// t multiplier (2.776 at four degrees of freedom) is not dominated by its own uncertainty.
const DEFAULT_RUNS: u64 = 5;

/// The reserved-word commands block, `--help`'s after-help. The
/// no-benches listing points at `-h` rather than repeating it.
const COMMANDS_HELP: &str = concat!(
    "Commands:\n",
    "  all        run every registered bench\n",
    "  qualify-environment\n",
    "             is this machine fit to measure on? Respawns this binary\n",
    "             --runs times after --run-sleep, collects each run's environment\n",
    "             grade, prints the table and a verdict: QUALIFIED when the median\n",
    "             grade is B or better and no run's drift or step reached D/F.\n",
    "             Exits nonzero when not. Grades the environment, not the run:\n",
    "             the machine is the subject, not a workload. Must stand alone;\n",
    "             -d sets each child's duration (default 1s), --pin-cpus\n",
    "             passes through, --print-only skips the verdict.\n",
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
    "  pin-freq [MHZ|pin_mhz|min_mhz|max_mhz]\n",
    "             hold the clock still until restore-freq: min = max at MHZ,\n",
    "             or at the config [freq] value named (default: pin_mhz,\n",
    "             else the base clock), boost off. The target must fit under\n",
    "             the ceiling with boost off. Needs root or setup's\n",
    "             permissions, and refuses without a declared [freq] steady\n",
    "             state in the config - the way home. Must stand alone.\n",
    "  restore-freq\n",
    "             converge the box to the config's declared [freq] steady\n",
    "             state (governor, EPP, boost, clamps), from any starting\n",
    "             point, including after an unclean death. Needs root or\n",
    "             setup's permissions. Must stand alone.\n",
    "  setup      make this host ready: print the [freq] steady state it would\n",
    "             write to ~/.config/iiac-perf/config.md from the live state,\n",
    "             clamp limits included, and the udev rule that lets you\n",
    "             pin-freq and restore-freq without sudo. --apply writes the\n",
    "             config and calls sudo once for the rule; --uninstall\n",
    "             plans removing the rule instead. Creates a missing config,\n",
    "             appends to one without [freq], and leaves one that declares\n",
    "             [freq] alone, checking it. Run as your user, not under\n",
    "             sudo. Must stand alone.\n",
    "  suggest-freq BENCH\n",
    "             measure the best pin frequency: descend from\n",
    "             max-with-boost-off, pin each candidate, drive BENCH (the\n",
    "             real workload, with this command line's -d/--pin-cpus),\n",
    "             and report the highest frequency the box held, ending\n",
    "             with the pin_mhz line to paste. The suggestion is per\n",
    "             bench, duration, and pin layout: a schedule selects the\n",
    "             state it can hold. Needs root or setup's permissions, and\n",
    "             a declared [freq] steady state, restores on exit like\n",
    "             pin-freq.",
);

#[derive(Parser)]
#[command(version, about = ABOUT, max_term_width = 80, after_help = COMMANDS_HELP)]
struct Cli {
    /// Benches to run, or a command word ('all',
    /// 'qualify-environment', 'describe-record', 'read-freq',
    /// 'pin-freq', 'restore-freq', 'setup', 'suggest-freq').
    ///
    /// Pass 'all' for every registered bench, or one or more
    /// names. A name matching no bench exactly runs every bench
    /// it is a prefix of (e.g. 'ice', 'mpsc'). Pass
    /// 'qualify-environment' (alone) to ask whether this machine
    /// is fit to measure on. Pass 'describe-record' (alone) to
    /// print the --record field dictionary. Pass 'read-freq',
    /// 'pin-freq [MHZ]', or 'restore-freq' (alone) to read, pin,
    /// or restore the CPU clock. Pass 'setup' (alone) to make this
    /// host ready for them. Pass 'suggest-freq BENCH' to
    /// measure the best pin frequency under that bench's load.
    /// With no bench names, --benches or the config `benches`
    /// names the benches, and with none of them either, the
    /// available list prints.
    #[arg(value_name = "BENCH", add = ArgValueCompleter::new(complete_positional))]
    benches: Vec<String>,

    /// Benches to run, comma-separated or repeated.
    ///
    /// The flag form of the bench names above, for a line that
    /// reads better with every input named: names, prefixes, or
    /// 'all', never a command word. Overrides the config
    /// `benches`. Conflicts with bench names given positionally.
    #[arg(
        long = "benches",
        value_name = "BENCH",
        value_delimiter = ',',
        conflicts_with = "benches"
    )]
    benches_flag: Vec<String>,

    /// Target wall-clock seconds per bench.
    ///
    /// Default 5.0, or the config `duration`. Auto-sizes the sample
    /// and inner loop counts. Mutually exclusive with -D.
    #[arg(short = 'd', long, conflicts_with = "total_duration")]
    duration: Option<f64>,

    /// Target total wall-clock seconds across all benches.
    ///
    /// The budget is split equally over every run of every bench,
    /// benches times --runs. Mutually exclusive with -d.
    #[arg(short = 'D', long)]
    total_duration: Option<f64>,

    /// Override the sample count (skips auto-sizing, and inner still
    /// adapts), rounded up to whole blocks so every block runs
    /// the same count, never cut by the blocks' time cap. `-o` / `--outer`, the count's old name,
    /// still work.
    #[arg(short, long, short_alias = 'o', alias = "outer")]
    samples: Option<u64>,

    /// Override inner loop count (skips auto-sizing).
    ///
    /// inner=1 measures single-call latency (each sample = one
    /// step). Higher inner measures back-to-back/burst rate
    /// (each sample = N steps averaged).
    #[arg(short, long)]
    inner: Option<u64>,

    /// Pin bench threads to CPUs (comma-separated, ranges OK).
    ///
    /// A CPU is the kernel's schedulable unit (sysfs cpuN, one
    /// affinity-mask bit). A physical core hosts two of them when
    /// SMT is on. The list is a CPU *pool*: thread `i` of a bench
    /// is pinned to `pool[i % pool.len()]`, so shorter pools
    /// oversubscribe by wrap. Examples: `--pin-cpus 0,1` (2
    /// threads -> 2 CPUs), `--pin-cpus 0-5` (6-thread pool),
    /// `--pin-cpus 0,0` (two threads on the same CPU). On 3900X,
    /// CPUs N and N+12 are SMT siblings of the same physical core:
    /// `--pin-cpus 0,12` pairs siblings (max contention),
    /// `--pin-cpus 0,1` gives independent cores. A value naming a
    /// `[profiles]` entry in the config file expands to that
    /// profile's CPU spec (e.g. `--pin-cpus smt`). Omit to leave
    /// threads unpinned. `--pin` is a hidden alias.
    #[arg(long, alias = "pin", value_name = "CPUS")]
    pin_cpus: Option<String>,

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
    /// Only affects `TProbe` output. `Probe` results are always
    /// in nanoseconds.
    #[arg(short = 't', long)]
    ticks: bool,

    /// Runs of each bench, each a fresh process (default 5).
    ///
    /// A process start re-rolls where a bench's memory lands, and
    /// that sets its level, so the runs' means are the replicates
    /// the bench's `CI95 runs` and `LSC runs` come from. A bench's
    /// runs go back to back. One run prints its report as a single
    /// process does, and several print a line per run and the
    /// bench's summary, `-v` adding every run's report. Overrides
    /// the config `runs`. For 'qualify-environment', the child runs
    /// to spawn (default 10).
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u64).range(1..=1000))]
    runs: Option<u64>,

    /// Sleep before each run after the first: a duration or range with unit (us, ms, s).
    ///
    /// A range re-rolls per run. 0 (the default) starts each run
    /// as the last one ends, and the process start, the tick
    /// calibration, and the warm already stand in front of every
    /// run, so a sleep is how a colder start is asked for.
    /// Overrides the config `run_sleep`. For
    /// 'qualify-environment', the sleep before each child run,
    /// where 0 sustains the duty cycle that provokes a state
    /// transition and a sleep probes a quieter one.
    #[arg(long, value_name = "SPAN")]
    run_sleep: Option<String>,

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

    /// `setup` only: do what the plain command prints.
    ///
    /// Without it, setup changes nothing and shows the config it
    /// would write and the permissions it would install. With it,
    /// setup writes the config and calls sudo once for the
    /// permissions.
    #[arg(long)]
    apply: bool,

    /// `setup` only: plan removing the permissions instead.
    ///
    /// Shows the udev rule and file ownership it would give back
    /// to root, and does it with --apply. The config is left
    /// alone.
    #[arg(long)]
    uninstall: bool,

    /// Pin the CPU clock for this run, restoring on exit.
    ///
    /// Engages before the warmup, exactly like 'pin-freq': min =
    /// max at MHZ (--pin-freq=3800), or at the config [freq]
    /// value named (--pin-freq=min_mhz or max_mhz), bare meaning pin_mhz,
    /// else the discovered base clock, with boost off. The
    /// declared [freq] steady state is restored on normal exit,
    /// panic, SIGINT, and SIGTERM. After SIGKILL or power loss,
    /// run 'restore-freq'. --pin-freq=no cancels a config file's
    /// pin_freq for this run. Overrides the config `pin_freq`. Needs root, or the permissions 'setup --apply'
    /// grants, and a declared [freq] steady state.
    #[arg(
        long,
        value_name = "MHZ|pin_mhz|min_mhz|max_mhz|no",
        num_args = 0..=1,
        require_equals = true
    )]
    pin_freq: Option<Option<String>>,

    /// Stop probing the environment at block seams.
    ///
    /// The environment grade normally samples the box at every
    /// block boundary, so its letter covers the whole run. This
    /// limits it to the warmup probes, which cover only the few
    /// ms before the bench starts. Use it when the seam probes
    /// disturb the workload (a spinning multi-threaded bench
    /// keeps running through a probe, so its queues drain), or
    /// to A/B whether they do.
    #[arg(long)]
    no_env_probe: bool,

    /// Seconds to warm the box before a bench measures.
    ///
    /// The first bench of a process otherwise reports a cold
    /// machine's numbers - measured at ~8.6% slow on a 7600x.
    /// The warm is paid once per process, and every bench runs in
    /// a process of its own, so every bench pays it. The
    /// grade block's `settle` cell says how long the box actually
    /// took to settle. 0 skips it, which is how you measure what
    /// the warm is worth on a given box. Overrides the config
    /// `settle_time`, and both absent defaults to 1.5.
    #[arg(long, value_name = "SECONDS", allow_negative_numbers = true)]
    settle_time: Option<f64>,

    /// Cap on each run's warm-until-stable stretch (seconds).
    ///
    /// Every run warms until the trailing probe window grades A
    /// (and the delivered clock holds still, where readable), or
    /// until this cap. A settled box exits in ~50 ms, so the cap
    /// prices only the disturbed case, and hitting it is
    /// reported in the grade block (a "00%" settle cell with an
    /// F, or "uncertified"), never silently absorbed. 0 caps
    /// immediately, which is how you measure what the warm is
    /// worth. Overrides the config `warm_cap`, and both absent
    /// defaults to 1.5.
    #[arg(long, value_name = "SECONDS", allow_negative_numbers = true)]
    warm_cap: Option<f64>,

    /// Band label style for the report's histogram rows.
    ///
    /// 'zpn': nines/zeros + decile names (z3, p50, n4).
    /// 'frac': literal boundary fractions with '_' grouping
    /// (0.001, 0.50, 0.999_9). 'both': zpn and fraction
    /// side by side: the juxtaposition teaches the zpn
    /// vocabulary. Switch to 'zpn' once fluent. Overrides the
    /// config `band_labels`, and both absent defaults to 'both'.
    #[arg(long, value_enum)]
    band_labels: Option<bands::BandLabels>,

    /// Decimal digits on the report's time columns (0-3).
    ///
    /// 1 shows the sub-ns precision picosecond recording
    /// captures, 0 restores integer ns, and 3 is the recording
    /// floor - more digits would be artifacts. Overrides the
    /// config `decimals`, and both absent defaults to 1.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=3))]
    decimals: Option<u8>,

    /// Measurement blocks per run (default 100).
    ///
    /// Every run is N blocks sized to one sample count from the
    /// budget and the warmup's typical sample cost, so
    /// `--blocks 10 -d 10` is 10 blocks of ~1 s. A block that
    /// reaches twice its share of the budget stops there. The blocks are
    /// the run's time axis (the grades and the resolution curve
    /// read the block series) and its replicates (each block's
    /// mean is one point of the series behind mean, CI95 blocks,
    /// and LSC blocks). 1 is a
    /// plain run, and 8 is the suggested minimum: below it the
    /// stats that need more blocks print '-' and the report says
    /// so. Blocks
    /// sleep and re-warm between one another as --block-sleep /
    /// --block-warmup ask (1-10 ms and 0 by default, and neither is
    /// counted in the budget): the sleep makes the blocks genuine
    /// replicates, and '--block-sleep 0' leaves them partitions
    /// of one continuous run, where CI95 blocks / LSC blocks print
    /// '-'. Bench-driven benches only. Probe benches
    /// ignore it. Overrides the config `blocks`.
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u64).range(1..=1000))]
    blocks: Option<u64>,

    /// Sleep between blocks: a duration or range with unit (us, ms, s).
    ///
    /// E.g. '--block-sleep 1-10ms' re-rolls a random sleep per
    /// block (re-rolls scheduler and frequency state, and a range
    /// avoids phase-locking with kernel ticks), '--block-sleep 1s'
    /// sleeps exactly 1 s (a long sleep reaches deep C-states, so
    /// wakes start colder). Default 1-10ms, so every run's blocks
    /// are replicates. 0 never sleeps: the blocks are partitions of
    /// one continuous run and the replication rows print '-'.
    /// Overrides the config `block_sleep`.
    #[arg(long, value_name = "SPAN")]
    block_sleep: Option<String>,

    /// Unrecorded post-wake warmup per block: a duration with unit.
    ///
    /// Steps the bench unrecorded after each block sleep, keeping
    /// the frequency ramp and cache refill out of the samples. 0
    /// (the default) records from the first post-wake call, which
    /// is how cold-wake behavior is seen. Overrides the config
    /// `block_warmup`.
    #[arg(long, value_name = "DUR")]
    block_warmup: Option<String>,

    /// Append one JSONL record per bench result to PATH.
    ///
    /// A side channel, never a mode: the display is unchanged, and
    /// the record is what survives the session (fixed quantile
    /// ladder, block means, seam clock, power policy). The
    /// 'describe-record' command lists every field. The path's
    /// shape picks the mode: end it with '/' (or name an existing
    /// directory) for one file per run, stamped
    /// <ts>-<host>-<bench>.jsonl so a rerun can't clobber
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

    /// Print the registered bench names, one per line, and exit.
    ///
    /// No bench runs. Machine-readable, for scripts to iterate.
    /// The command words are not bench names and are not listed.
    #[arg(long)]
    list_benches: bool,

    /// Run as a bench child: the spec file the parent wrote. Internal, so hidden: every bench runs
    /// in a child of its own, spawned by the parent with this flag.
    #[arg(long, hide = true, value_name = "PATH")]
    child_spec: Option<std::path::PathBuf>,
}

/// The command words the positional accepts beside bench names,
/// each with the one-line help Tab shows: the completer's source,
/// and the list the positional's doc comment spells out.
const COMMAND_WORDS: &[(&str, &str)] = &[
    ("all", "run every registered bench"),
    ("qualify-environment", "is this machine fit to measure on?"),
    ("describe-record", "print the --record field dictionary"),
    ("read-freq", "print the CPU clock state"),
    (
        "pin-freq",
        "hold the CPU clock still (min = max, boost off)",
    ),
    (
        "restore-freq",
        "converge to the declared [freq] steady state",
    ),
    (
        "setup",
        "make this host ready for pin-freq and restore-freq",
    ),
    (
        "suggest-freq",
        "measure the best pin frequency under a bench's load",
    ),
];

/// Tab candidates for the positional: every registered bench name
/// and every command word starting with what is typed so far. The
/// shell calls the binary itself for these (`COMPLETE=bash`, see
/// `CompleteEnv`), so the list is always the running build's.
fn complete_positional(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let typed = current.to_string_lossy();
    let benches = benches::names().into_iter().map(CompletionCandidate::new);
    let words = COMMAND_WORDS
        .iter()
        .map(|(w, help)| CompletionCandidate::new(*w).help(Some((*help).into())));
    benches
        .chain(words)
        .filter(|c| c.get_value().to_string_lossy().starts_with(typed.as_ref()))
        .collect()
}

/// Refuse a bench list holding a command word other than `all`. A command word runs alone and
/// positionally, so in `--benches` or the config `benches` it would otherwise reach bench
/// resolution and read as an unknown bench.
fn check_bench_words(words: &[String]) -> Result<(), String> {
    match words
        .iter()
        .find(|w| *w != "all" && COMMAND_WORDS.iter().any(|(c, _)| c == w))
    {
        Some(word) => Err(format!(
            "benches: '{word}' is a command word, not a bench: run it as '{BIN_NAME} {word}'"
        )),
        None => Ok(()),
    }
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
    CompleteEnv::with_factory(Cli::command).complete();
    let cli = Cli::parse();

    // The bench-name listing is a pure print-and-exit path: no
    // logging, no config, no setup.
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
    // state, and pin-freq additionally takes one optional MHZ arg
    // (`pin-freq 3800`).
    if cli.benches.iter().any(|b| b == "pin-freq") {
        if cli.benches[0] != "pin-freq" || cli.benches.len() > 2 {
            eprintln!(
                "error: 'pin-freq' runs alone, with at most one MHZ, pin_mhz, min_mhz, or max_mhz arg"
            );
            std::process::exit(2);
        }
        let target = match config::PinFreq::from_flag(cli.benches.get(1).map(String::as_str)) {
            Ok(config::PinFreq::Off) => {
                eprintln!("error: pin-freq: \"no\" pins nothing; use restore-freq to unpin");
                std::process::exit(2);
            }
            Ok(t) => t,
            Err(e) => {
                eprintln!("error: pin-freq: {e}");
                std::process::exit(2);
            }
        };
        let config = load_config_or_exit();
        std::process::exit(freqctl::cmd_pin_freq(
            config.freq.as_ref(),
            target,
            config.source("freq"),
        ));
    }
    if cli.benches.iter().any(|b| b == "restore-freq") {
        if cli.benches.len() > 1 {
            eprintln!("error: 'restore-freq' runs alone; drop the other bench args");
            std::process::exit(2);
        }
        let config = load_config_or_exit();
        std::process::exit(freqctl::cmd_restore_freq(
            config.freq.as_ref(),
            config.source("freq"),
        ));
    }

    // 'setup' prepares the host and exits: it reads the live clock
    // state and the XDG config itself, so it needs neither the
    // layered config nor the banner.
    if cli.benches.iter().any(|b| b == "setup") {
        if cli.benches.len() > 1 {
            eprintln!("error: 'setup' runs alone; drop the other bench args");
            std::process::exit(2);
        }
        std::process::exit(setup::run(cli.apply, cli.uninstall));
    }

    // Default filter is `warn`. `-v` bumps to `debug`. `RUST_LOG`
    // (if set) always wins, so users can still do fine-grained
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

    // A bench child runs its one bench from the parent's spec and exits: no config, no inhibit,
    // no clock pin, and no banner, all of which the parent owns.
    if let Some(spec) = &cli.child_spec {
        std::process::exit(child::child_main(spec));
    }

    if cli.benches.is_empty()
        && cli.benches_flag.is_empty()
        && load_config_or_exit().benches.is_none()
    {
        println!("{ABOUT}\n");
        println!("no benches specified. use -h or --help for more info.\n");
        println!("Benches:");
        println!("{}", wrap_names(&benches::names(), 72));
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
        let run_sleep_s = match cli.run_sleep.as_deref() {
            None => (0.0, 0.0),
            Some(s) => match timespec::parse_span(s) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("error: --run-sleep: {e}");
                    std::process::exit(2);
                }
            },
        };
        let code = qualify::run(&qualify::QualifyCfg {
            runs: match cli.runs {
                Some(n) => n,
                None => QUALIFY_RUNS,
            },
            run_sleep_s,
            duration_s: cli.duration.unwrap_or(QUALIFY_CHILD_SECONDS),
            pin_cpus: cli.pin_cpus.clone(),
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

    // The bench list: the positional names, else --benches, else the config's `benches`, checked
    // before anything prints. The listing check above already sent a line with no list anywhere
    // to the listing, and suggest-freq's positional words are checked where it is resolved.
    let suggesting = cli.benches.first().is_some_and(|w| w == "suggest-freq");
    let (bench_list, benches_src) = if cli.benches.is_empty() {
        let flag = if cli.benches_flag.is_empty() {
            None
        } else {
            Some(cli.benches_flag.clone())
        };
        layered(
            flag,
            "--benches",
            config.benches.clone(),
            "benches",
            &config,
            Vec::new(),
        )
    } else {
        (
            cli.benches.clone(),
            Source::Flag("command line".to_string()),
        )
    };
    if !suggesting && let Err(e) = check_bench_words(&bench_list) {
        eprintln!("error: {e}");
        std::process::exit(2);
    }

    // Pin the clock before anything measures or prints, so the
    // Setup block and the warm loop both see the pinned state. The
    // flag wins, then the config's pin_freq, and suggest-freq never
    // takes a config pin, since it pins for itself. The guard
    // restores the declared steady state on drop (normal exit and
    // panic) and via the signal path on SIGINT/SIGTERM.
    let cli_pin = match &cli.pin_freq {
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
        config::PinFreq::Off,
    );
    let pin_target = match (suggesting, pin_setting) {
        (true, _) | (false, config::PinFreq::Off) => None,
        (false, target) => Some(target),
    };
    let freq_pin = match pin_target {
        None => None,
        Some(target) => {
            match freqctl::RunPin::engage(config.freq.as_ref(), target, config.source("freq")) {
                Ok(g) => Some(g),
                Err(e) => {
                    eprintln!("error: pin_freq: {e}");
                    std::process::exit(2);
                }
            }
        }
    };

    println!("{ABOUT}\n");

    if let Some(mask) = pin::current_affinity() {
        info!("startup affinity: {}", pin::affinity_summary(&mask));
    }

    let pin_cpus: Vec<usize> = match cli.pin_cpus.as_deref() {
        None => Vec::new(),
        // A spec naming a config profile expands to its CPU list.
        // Anything else parses as a raw CPU spec.
        Some(spec) => match pin::parse_cpus(config.resolve_pin(spec)) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: --pin-cpus: {e}");
                std::process::exit(2);
            }
        },
    };

    // Pin main to the pool's first slot when --pin-cpus is given: thread 0 of a bench measures
    // on main, and the warm loop is a real timing phase converging on per-CPU frequency state,
    // so it must run where measurement will run. Without --pin-cpus, main stays wherever the
    // scheduler has it: a busy thread stays put, and the warm state lands on the CPU that
    // measures. The retired CPU0-default warm pin parked the warm on the kernel's busiest CPU
    // for no measured benefit: the tick-rate read is a ratio that cancels interruptions (~8e-7
    // spread across CPUs), and nothing else ran pinned.
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
        cli.settle_time,
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
        cli.warm_cap,
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
        cli.blocks,
        "--blocks",
        config.blocks,
        "blocks",
        &config,
        harness::DEFAULT_BLOCKS,
    );
    // The sleep defaults to a short range so every run's blocks are replicates, and the warmup
    // to zero so no sample is discarded unless asked.
    let cli_block_sleep = match cli.block_sleep.as_deref() {
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
    let cli_block_warmup = match cli.block_warmup.as_deref() {
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
        0.0,
    );
    // Runs per bench, each a fresh process, and the sleep before each run after the first.
    let (runs, runs_src) = layered(
        cli.runs,
        "--runs",
        config.runs,
        "runs",
        &config,
        DEFAULT_RUNS,
    );
    let cli_run_sleep = match cli.run_sleep.as_deref() {
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
        (0.0, 0.0),
    );

    // Main's placement covers the warm loop and thread 0 of every bench, so the cell names
    // both.
    let main_pin_display = match pin_cpus.first() {
        Some(c) => format!("CPU {c} (pool slot 0; warm + run)"),
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
    println!("  bench pin         {}", pin::plan_summary(&pin_cpus));
    if let Some(g) = &freq_pin {
        println!(
            "  freq pin          {} MHz ({}; min = max, boost off; restores on exit)",
            g.khz / 1000,
            g.source
        );
    }
    println!("  sleep inhibit     {inhibit_status}");
    println!();

    // 'suggest-freq BENCH' replaces the bench loop with the
    // candidate descent, driving that one bench through the same
    // run configuration. Resolved here rather than with the other
    // command words because it wants the whole setup a bench run
    // gets: inhibit, config, Setup block, knobs.
    let suggest = match cli.benches.first() {
        Some(w) if w == "suggest-freq" => {
            if cli.benches.len() != 2 {
                eprintln!(
                    "error: 'suggest-freq' takes exactly one bench word, e.g. \
                     'suggest-freq zcr-mpsc-v0-2t'"
                );
                std::process::exit(2);
            }
            if cli.pin_freq.is_some() {
                eprintln!("error: suggest-freq pins for itself; drop --pin-freq");
                std::process::exit(2);
            }
            Some(cli.benches[1].clone())
        }
        _ => {
            if cli.benches.iter().any(|b| b == "suggest-freq") {
                eprintln!("error: 'suggest-freq' leads: 'suggest-freq BENCH'");
                std::process::exit(2);
            }
            None
        }
    };

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
            "error: suggest-freq names one bench exactly (a prefix matching several \
             does not say which schedule the suggestion serves)"
        );
        std::process::exit(2);
    }

    // Duration precedence: CLI -d / -D win, then the config
    // `duration`, then the built-in default.
    let (target_seconds, duration_src) = match cli.total_duration {
        Some(t) if cli.duration.is_none() => (
            t / (runners.len() as u64 * runs) as f64,
            Source::Flag(format!(
                "--total-duration {} over {} benches x {runs} runs",
                seconds_value(t),
                runners.len()
            )),
        ),
        _ => layered(
            cli.duration,
            "-d",
            config.duration,
            "duration",
            &config,
            DEFAULT_DURATION,
        ),
    };
    let (band_labels, band_labels_src) = layered(
        cli.band_labels,
        "--band-labels",
        config.band_labels,
        "band_labels",
        &config,
        DEFAULT_BAND_LABELS,
    );
    let (decimals, decimals_src) = layered(
        cli.decimals,
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
    let flag_or_default = |set: bool, flag: &str| {
        if set {
            Source::Flag(flag.to_string())
        } else {
            Source::Default
        }
    };
    let pin_cpus_value = match cli.pin_cpus.as_deref() {
        None => "none".to_string(),
        Some(spec) if config.resolve_pin(spec) != spec => {
            format!("{spec} = {}", config.resolve_pin(spec))
        }
        Some(spec) => spec.to_string(),
    };
    // The pin's resolved target, whichever layer named it, so a record says what clock the run
    // held rather than that a pin was asked for.
    let pin_freq_value = match (&freq_pin, pin_setting) {
        (Some(g), _) => format!("{} MHz ({})", g.khz / 1000, g.source),
        (None, config::PinFreq::Off) => "no".to_string(),
        (None, _) => "not engaged: suggest-freq pins for itself".to_string(),
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
            &span_value((0.0, 0.0)),
            run_sleep_src,
        ),
        Param::new(
            "duration",
            seconds_value(target_seconds),
            &seconds_value(DEFAULT_DURATION),
            duration_src,
        ),
        Param::new(
            "samples",
            cli.samples.map_or("auto".to_string(), |n| n.to_string()),
            "auto",
            flag_or_default(cli.samples.is_some(), "--samples"),
        ),
        Param::new(
            "inner",
            cli.inner.map_or("auto".to_string(), |n| n.to_string()),
            "auto",
            flag_or_default(cli.inner.is_some(), "--inner"),
        ),
        Param::new(
            "pin_cpus",
            pin_cpus_value,
            "none",
            flag_or_default(cli.pin_cpus.is_some(), "--pin-cpus"),
        ),
        Param::new("pin_freq", pin_freq_value, "no", pin_freq_src),
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
            &seconds_value(0.0),
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
            if cli.no_env_probe { "off" } else { "on" }.to_string(),
            "on",
            flag_or_default(cli.no_env_probe, "--no-env-probe"),
        ),
        Param::new(
            "ticks",
            if cli.ticks { "ticks" } else { "ns" }.to_string(),
            "ns",
            flag_or_default(cli.ticks, "--ticks"),
        ),
        Param::new(
            "inhibit",
            if cli.no_inhibit { "off" } else { "on" }.to_string(),
            "on",
            flag_or_default(cli.no_inhibit, "--no-inhibit"),
        ),
        Param::new(
            "record",
            cli.record
                .as_deref()
                .map_or("none".to_string(), run_config::display_path),
            "none",
            flag_or_default(cli.record.is_some(), "--record"),
        ),
        Param::new(
            "tag",
            if cli.tag.is_empty() {
                "none".to_string()
            } else {
                cli.tag.join(", ")
            },
            "none",
            flag_or_default(!cli.tag.is_empty(), "--tag"),
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
    let record_config = record::RecordConfig::new(&config_files, &params);
    let recorder = match cli.record.as_deref() {
        None => None,
        Some(path) => match record::Recorder::new(path, &cli.tag, record_config.clone()) {
            Ok(r) => Some(r),
            Err(e) => {
                eprintln!("error: --record: {e}");
                std::process::exit(2);
            }
        },
    };

    let cfg = harness::RunCfg {
        target_seconds,
        samples_override: cli.samples,
        inner_override: cli.inner,
        pin_cpus: &pin_cpus,
        report_ticks: cli.ticks,
        seam_probes: !cli.no_env_probe,
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
        std::process::exit(freqctl::cmd_suggest_freq(
            config.freq.as_ref(),
            config.source("freq"),
            name,
            runners[0].1,
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
    // The record path goes to the children absolute, and as given when that fails, which still
    // resolves since a child inherits this directory.
    let record_spec = child::RecordSpec {
        path: cli
            .record
            .as_deref()
            .map(|path| match std::path::absolute(path) {
                Ok(abs) => abs,
                Err(_) => path.to_path_buf(),
            }),
        tags: cli.tag.clone(),
        config: record_config,
        series: record::new_series_id(),
    };
    let mut runner = runs::Runner::new(runs::Plan {
        exe: &exe,
        scratch: scratch.path(),
        runs,
        run_sleep_s,
        verbose: cli.verbose,
        decimals: decimals as usize,
    });
    for (name, _) in &runners {
        if let Err(e) = runner.bench(name, &cfg, &record_spec) {
            eprintln!("error: {e}");
            drop(scratch);
            drop(freq_pin);
            std::process::exit(1);
        }
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
    fn wrap_names_single_line() {
        assert_eq!(wrap_names(&["a", "b"], 72), "  a, b");
    }

    #[test]
    fn a_bench_list_refuses_command_words_but_all() {
        let words = |w: &[&str]| w.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(check_bench_words(&words(&["all"])).is_ok());
        assert!(check_bench_words(&words(&["min-now", "zcr"])).is_ok());
        let err = check_bench_words(&words(&["min-now", "setup"])).unwrap_err();
        assert!(err.contains("'setup'"), "unexpected error: {err}");
    }

    #[test]
    fn benches_flag_splits_on_commas_and_conflicts_with_names() {
        let cli = Cli::try_parse_from([
            "iiac-perf",
            "--benches",
            "min-now,std-now",
            "--benches",
            "zcr",
        ])
        .expect("parses");
        assert_eq!(cli.benches_flag, ["min-now", "std-now", "zcr"]);
        assert!(cli.benches.is_empty());
        assert!(Cli::try_parse_from(["iiac-perf", "min-now", "--benches", "std-now"]).is_err());
    }

    #[test]
    fn complete_positional_offers_benches_and_words_by_prefix() {
        let values = |typed: &str| -> Vec<String> {
            complete_positional(std::ffi::OsStr::new(typed))
                .iter()
                .map(|c| c.get_value().to_string_lossy().into_owned())
                .collect()
        };
        assert_eq!(values("cb-chan"), ["cb-chan-1t", "cb-chan-2t"]);
        assert_eq!(values("qual"), ["qualify-environment"]);
        let all = values("");
        assert_eq!(all.len(), benches::names().len() + COMMAND_WORDS.len());
        assert!(all.contains(&"suggest-freq".to_string()));
    }

    #[test]
    fn wrap_names_breaks_at_width() {
        // "ccc" would land past col 10, so it wraps, and the separator
        // comma stays on the prior line and the new line re-indents.
        assert_eq!(wrap_names(&["aaa", "bbb", "ccc"], 10), "  aaa, bbb,\n  ccc");
    }
}
