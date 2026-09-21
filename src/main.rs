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
mod init_config;
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
    "             print the record field dictionary: every record key with\n",
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
    "             the ceiling with boost off. Needs root or setup-freq's\n",
    "             permissions, and refuses without a declared [freq] steady\n",
    "             state in the config - the way home. Must stand alone.\n",
    "  restore-freq\n",
    "             converge the box to the config's declared [freq] steady\n",
    "             state (governor, EPP, boost, clamps), from any starting\n",
    "             point, including after an unclean death. Needs root or\n",
    "             setup-freq's permissions. Must stand alone.\n",
    "  setup-freq\n",
    "             make this host ready: print the [freq] steady state it would\n",
    "             write to ~/.config/iiac-perf/config.md from the live state,\n",
    "             clamp limits included, and the udev rule that lets you\n",
    "             pin-freq and restore-freq without sudo. --apply writes the\n",
    "             config and calls sudo once for the rule; --uninstall\n",
    "             plans removing the rule instead. Creates a missing config,\n",
    "             appends to one without [freq], and leaves one that declares\n",
    "             [freq] alone, checking it. Run as your user, not under\n",
    "             sudo. Must stand alone.\n",
    "  init-config [PATH]\n",
    "             print a starting config: every key, commented out at its\n",
    "             default, with the prose that explains it. With PATH, write\n",
    "             it there, as TOML when PATH ends in .toml, and never over\n",
    "             a file unless --backup (keeps PATH.bak) or --overwrite\n",
    "             (keeps nothing) says so. The old file's values are not\n",
    "             kept: that is update-config. --from OLD sets every key\n",
    "             OLD sets at OLD's value,\n",
    "             which brings an older file up to date: OLD is not touched,\n",
    "             and a key no longer known fails by name. --config NAME\n",
    "             starts from a file found by name instead. With neither,\n",
    "             the start is the run keys this host's config files set,\n",
    "             and run flags on the line go over any of the three, so\n",
    "             the file is the run that line makes here: --benches\n",
    "             names the benches, PATH being the one positional. [freq]\n",
    "             and [profiles] stay the host's. --from /dev/null is the\n",
    "             bare template.\n",
    "  update-config FILE\n",
    "             rewrite FILE in place: the starting config with FILE's own\n",
    "             values set and the line's run flags over them, so\n",
    "             'update-config queue.md --blocks 20' changes one key. With\n",
    "             no flags it brings an older file up to date. FILE is\n",
    "             checked, filled, and checked again before it is touched.\n",
    "             Prose and comments its author added are lost: --backup\n",
    "             keeps the old file as FILE.bak. Must stand alone.\n",
    "  suggest-freq BENCH\n",
    "             measure the best pin frequency: descend from\n",
    "             max-with-boost-off, pin each candidate, drive BENCH (the\n",
    "             real workload, with this command line's -d/--pin-cpus),\n",
    "             and report the highest frequency the box held, ending\n",
    "             with the pin_mhz line to paste. The suggestion is per\n",
    "             bench, duration, and pin layout: a schedule selects the\n",
    "             state it can hold. Needs root or setup-freq's permissions, and\n",
    "             a declared [freq] steady state, restores on exit like\n",
    "             pin-freq.",
);

#[derive(Parser)]
#[command(version, about = ABOUT, max_term_width = 80, after_help = COMMANDS_HELP)]
struct Cli {
    /// Benches to run, a config file, or a command word ('all',
    /// 'qualify-environment', 'describe-record', 'read-freq',
    /// 'pin-freq', 'restore-freq', 'setup-freq', 'init-config',
    /// 'update-config', 'suggest-freq').
    ///
    /// Pass 'all' for every registered bench, or one or more
    /// names. A name matching no bench exactly runs every bench
    /// it is a prefix of (e.g. 'ice', 'mpsc'), and one that is no
    /// prefix either runs every bench it matches as a regular
    /// expression (e.g. 'zcr-[sm]psc-v[23]'). Pass
    /// 'qualify-environment' (alone) to ask whether this machine
    /// is fit to measure on. Pass 'describe-record' (alone) to
    /// print the record field dictionary. Pass 'read-freq',
    /// 'pin-freq [MHZ]', or 'restore-freq' (alone) to read, pin,
    /// or restore the CPU clock. Pass 'setup-freq' (alone) to make this
    /// host ready for them. Pass 'init-config [PATH]' to print or
    /// write a starting config, and 'update-config FILE' to
    /// rewrite one in place. Pass 'suggest-freq BENCH' to
    /// measure the best pin frequency under that bench's load.
    /// A word ending in .md or .toml is the run's config file,
    /// as --config with it: 'iiac-perf queue.md'. Bench names
    /// beside it win over the file's `benches`. With no bench
    /// names, --benches or the config `benches` names the benches,
    /// and with none of them either, the available list prints.
    #[arg(value_name = "BENCH", add = ArgValueCompleter::new(complete_positional))]
    benches: Vec<String>,

    /// Benches to run, comma-separated or repeated.
    ///
    /// The flag form of the bench names above, for a line that
    /// reads better with every input named: names, prefixes,
    /// patterns, or 'all', never a command word. Overrides the config
    /// `benches`. An error beside bench names given positionally.
    /// On an 'init-config' or 'update-config' line it sets the
    /// file's `benches`.
    #[arg(long = "benches", value_name = "BENCH", value_delimiter = ',')]
    benches_flag: Vec<String>,

    /// The run's config file, by name. A positional ending in
    /// .md or .toml is the same: 'iiac-perf queue.md'.
    ///
    /// The run keys come from this file and the built-in defaults
    /// alone, flags still winning, so one file is one run on every
    /// host. The XDG and project-local files give only [freq] and
    /// [profiles], under whatever this file sets of them. An
    /// absolute NAME is taken as given. A relative one is looked
    /// for in the current directory, each parent, then the XDG
    /// config directory, the first found winning, as NAME, NAME.md,
    /// or NAME.toml. Not found is an error.
    #[arg(long, value_name = "NAME")]
    config: Option<std::path::PathBuf>,

    /// Target wall-clock time per bench: seconds bare, or a
    /// duration with unit (us, ms, s).
    ///
    /// Default 5.0, or the config `duration`. Auto-sizes the sample
    /// and inner loop counts. Mutually exclusive with -D.
    #[arg(short = 'd', long, conflicts_with = "total_duration", value_name = "DUR", value_parser = timespec::parse_seconds)]
    duration: Option<f64>,

    /// Target total wall-clock time across all benches: seconds
    /// bare, or a duration with unit.
    ///
    /// The budget is split equally over every run of every bench,
    /// benches times --runs. Mutually exclusive with -d. Overrides
    /// the config `total_duration` and `duration`.
    #[arg(short = 'D', long, value_name = "DUR", value_parser = timespec::parse_seconds)]
    total_duration: Option<f64>,

    /// Override the sample count (skips auto-sizing, and inner still
    /// adapts), rounded up to whole blocks so every block runs
    /// the same count, never cut by the blocks' time cap. `-o` / `--outer`, the count's old name,
    /// still work. Overrides the config `samples`.
    #[arg(short, long, short_alias = 'o', alias = "outer")]
    samples: Option<u64>,

    /// Override inner loop count (skips auto-sizing).
    ///
    /// inner=1 measures single-call latency (each sample = one
    /// step). Higher inner measures back-to-back/burst rate
    /// (each sample = N steps averaged). Overrides the config
    /// `inner`.
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
    /// threads unpinned. `--pin` is a hidden alias. Overrides the
    /// config `pin_cpus`.
    #[arg(long, alias = "pin", value_name = "CPUS")]
    pin_cpus: Option<String>,

    /// Enable verbose internals on stderr (like `RUST_LOG=debug`).
    ///
    /// Shows the affinity mask, the pin lifecycle, and the TSC
    /// tick rate. Default is `warn` (silent unless something's
    /// wrong). `RUST_LOG` overrides this flag when set, so
    /// per-module filtering still works. Overrides the config
    /// `verbose`, and --verbose=no cancels a config file's.
    #[arg(
        short,
        long,
        value_name = "yes|no",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "yes",
        value_parser = parse_yes_no
    )]
    verbose: Option<bool>,

    /// Show tprobe results in raw TSC ticks, not nanoseconds.
    ///
    /// Only affects `TProbe` output. `Probe` results are always
    /// in nanoseconds. Overrides the config `ticks`, and
    /// --ticks=no cancels a config file's.
    #[arg(
        short = 't',
        long,
        value_name = "yes|no",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "yes",
        value_parser = parse_yes_no
    )]
    ticks: Option<bool>,

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

    /// Sleep before each run: a duration or range with unit (us, ms, s).
    ///
    /// Default 1-2s, re-rolled per run and drawn before the first
    /// run too, so every run starts alike: without it the first
    /// run starts from whatever the host did before and the rest
    /// start hot from the run before. 0 starts each run as the
    /// last one ends. Overrides the config `run_sleep`. For
    /// 'qualify-environment', the sleep before each child run,
    /// default 0, which sustains the duty cycle that provokes a
    /// state transition, where a sleep probes a quieter one.
    #[arg(long, value_name = "SPAN")]
    run_sleep: Option<String>,

    /// The band of the run means the trimmed rows keep, FROM-TO in whole percents: `10-50`
    /// keeps the 10th to the 50th percentile, cutting mostly high since a run can land slow and
    /// never fast. `20-80` is the symmetric middle 60%, `0-100` no trim. Set it once for a
    /// project, never per comparison: a trim picked after seeing the numbers flatters them.
    /// Overrides the config `trim_runs`.
    #[arg(long, value_name = "FROM-TO")]
    trim_runs: Option<String>,

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

    /// `setup-freq` only: do what the plain command prints.
    ///
    /// Without it, setup-freq changes nothing and shows the config it
    /// would write and the permissions it would install. With it,
    /// setup-freq writes the config and calls sudo once for the
    /// permissions.
    #[arg(long)]
    apply: bool,

    /// `setup-freq` only: plan removing the permissions instead.
    ///
    /// Shows the udev rule and file ownership it would give back
    /// to root, and does it with --apply. The config is left
    /// alone.
    #[arg(long)]
    uninstall: bool,

    /// `init-config` only: carry this config file's values over.
    ///
    /// The new file is the starting config with every key OLD
    /// sets uncommented at OLD's value. OLD is read, never written.
    /// --config NAME does the same for a file found by its search,
    /// and run flags on the line set their keys over either.
    #[arg(long, value_name = "OLD")]
    from: Option<std::path::PathBuf>,

    /// `init-config` and `update-config`: keep the old file as
    /// FILE.bak.
    ///
    /// For 'update-config', without it nothing of the old file is
    /// kept, and prose and comments its author added are gone. For
    /// 'init-config' it is what lets PATH be a file that exists:
    /// the file is replaced and the old one kept.
    #[arg(long)]
    backup: bool,

    /// `init-config` only: replace PATH when it exists, keeping
    /// nothing.
    ///
    /// The old file's values are not carried over, which is what
    /// 'update-config' is for. --backup replaces it too and keeps
    /// the old file.
    #[arg(long)]
    overwrite: bool,

    /// Pin the CPU clock for this run, restoring on exit.
    ///
    /// Engages before the warmup, exactly like 'pin-freq': min =
    /// max at MHZ (--pin-freq=3800), or at the config [freq]
    /// value named (--pin-freq=min_mhz or max_mhz), bare meaning pin_mhz,
    /// else the discovered base clock, with boost off. The
    /// declared [freq] steady state is restored on normal exit,
    /// panic, SIGINT, and SIGTERM. After SIGKILL or power loss,
    /// run 'restore-freq'. --pin-freq=no cancels a config file's
    /// pin_freq for this run. Overrides the config `pin_freq`. Needs root, or the permissions 'setup-freq --apply'
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
    /// to A/B whether they do. Overrides the config `env_probe`,
    /// and --no-env-probe=no turns the probes back on over a
    /// config file's `env_probe = false`.
    #[arg(
        long,
        value_name = "yes|no",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "yes",
        value_parser = parse_yes_no
    )]
    no_env_probe: Option<bool>,

    /// Time to warm the box before a bench measures: seconds
    /// bare, or a duration with unit.
    ///
    /// The first bench of a process otherwise reports a cold
    /// machine's numbers - measured at ~8.6% slow on a 7600x.
    /// The warm is paid once per process, and every bench runs in
    /// a process of its own, so every bench pays it. The
    /// grade block's `settle` cell says how long the box actually
    /// took to settle. 0 skips it, which is how you measure what
    /// the warm is worth on a given box. Overrides the config
    /// `settle_time`, and both absent defaults to 1.5.
    #[arg(long, value_name = "DUR", allow_negative_numbers = true, value_parser = timespec::parse_seconds)]
    settle_time: Option<f64>,

    /// Cap on each run's warm-until-stable stretch: seconds bare,
    /// or a duration with unit.
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
    #[arg(long, value_name = "DUR", allow_negative_numbers = true, value_parser = timespec::parse_seconds)]
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

    /// Record one JSONL line per run to a file per invocation in DIR.
    ///
    /// A side channel, never a mode: the display is unchanged, and
    /// the record is what survives the session (fixed quantile
    /// ladder, block means, seam clock, power policy). The
    /// 'describe-record' command lists every field. The file is
    /// named <label>-<series>-<host>.jsonl, so a rerun can't clobber
    /// evidence, and every run and bench of one command goes to it.
    /// DIR is created. Probe-style benches produce no harness
    /// result and record nothing. Overrides the config `record_dir`
    /// or `record_file`.
    #[arg(long, value_name = "DIR", conflicts_with = "record_file")]
    record_dir: Option<std::path::PathBuf>,

    /// Append every run's JSONL record to the one file PATH.
    ///
    /// As --record-dir, but every invocation appends to PATH, a
    /// line a run, and the open never truncates. PATH's directory
    /// is created. Overrides the config `record_dir` or
    /// `record_file`.
    #[arg(long, value_name = "PATH")]
    record_file: Option<std::path::PathBuf>,

    /// Name a --record-dir file: its name leads with NAME.
    ///
    /// Sugar for '--tag label=NAME', so every record carries the
    /// label and a renamed file still knows it. Without it the
    /// label is the bench selector as typed: 'ice-rr-2t --record-dir
    /// runs' writes runs/ice-rr-2t-<series>-<host>.jsonl, a list
    /// joins its names with '_', and past three names it is their
    /// count. Refused with a --record-file target, whose path is its
    /// name: '--tag label=NAME' labels those records. The config
    /// spells it as `label` in `[tags]`.
    #[arg(long, value_name = "NAME")]
    record_label: Option<String>,

    /// Retired: refused with a message naming --record-dir and
    /// --record-file, since a path's trailing '/' picked its mode.
    #[arg(long, hide = true, value_name = "PATH")]
    record: Option<std::path::PathBuf>,

    /// Tag every record with KEY=VALUE (repeatable).
    ///
    /// Recorded verbatim, never interpreted: the caller, not the
    /// tool, knows which runs form one experiment, so e.g.
    /// '--tag series=20260816T09' labels a series and '--tag
    /// condition=pinned' a condition. Adds to the config `[tags]`
    /// table, winning on a shared key. Needs a record target, from
    /// --record-dir, --record-file, or the config.
    #[arg(long, value_name = "KEY=VALUE")]
    tag: Vec<String>,

    /// Do not inhibit system sleep for the run.
    ///
    /// By default the process re-execs itself under
    /// `systemd-inhibit --what=sleep` so an idle-suspend can't
    /// poison a long measurement. Pass this to keep the process
    /// image untouched (strace/gdb/perf wrappers), to let the
    /// machine sleep on purpose, or to test the suspend-detection
    /// WARNING path (a sleep inhibitor also blocks manual
    /// `systemctl suspend`). Overrides the config `inhibit`, and
    /// --no-inhibit=no inhibits over a config file's
    /// `inhibit = false`.
    #[arg(
        long,
        value_name = "yes|no",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "yes",
        value_parser = parse_yes_no
    )]
    no_inhibit: Option<bool>,

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
    ("describe-record", "print the record field dictionary"),
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
        "setup-freq",
        "make this host ready for pin-freq and restore-freq",
    ),
    ("init-config", "print or write a starting config"),
    ("update-config", "rewrite a config in place"),
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
    let configs = config_files(&typed)
        .into_iter()
        .map(|f| CompletionCandidate::new(f).help(Some("run this config".into())));
    let benches = benches::names().into_iter().map(CompletionCandidate::new);
    let words = COMMAND_WORDS
        .iter()
        .map(|(w, help)| CompletionCandidate::new(*w).help(Some((*help).into())));
    benches
        .chain(words)
        .filter(|c| c.get_value().to_string_lossy().starts_with(typed.as_ref()))
        .chain(configs)
        .collect()
}

/// The config files `typed` could become: the `.md` and `.toml` files in the directory it
/// names so far whose names start as it does. None for an empty `typed`, where every README
/// in the directory would crowd the bench names.
fn config_files(typed: &str) -> Vec<String> {
    if typed.is_empty() {
        return Vec::new();
    }
    let (dir, prefix) = match typed.rsplit_once('/') {
        Some((dir, prefix)) => (format!("{dir}/"), prefix),
        None => (String::new(), typed),
    };
    let listing = match std::fs::read_dir(if dir.is_empty() { "." } else { &dir }) {
        Ok(listing) => listing,
        Err(_) => return Vec::new(),
    };
    let mut files: Vec<String> = listing
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter(|name| name.starts_with(prefix) && is_config_arg(name))
        .map(|name| format!("{dir}{name}"))
        .collect();
    files.sort();
    files
}

/// Whether a positional names a config file: it ends in a carrier's extension, which no bench
/// name does, so the two never collide. A bare name stays a bench, since falling back to a
/// config would turn a mistyped bench into a file lookup.
fn is_config_arg(word: &str) -> bool {
    std::path::Path::new(word)
        .extension()
        .is_some_and(|e| e == "md" || e == "toml")
}

/// Move a config file among the positionals into `--config`, so `iiac-perf queue.md` is
/// `iiac-perf --config queue.md`, the common line without the flag. `init-config` and
/// `update-config` keep theirs, their one positional being a file to write.
fn take_config_arg(cli: &mut Cli) -> Result<(), String> {
    if cli
        .benches
        .first()
        .is_some_and(|w| w == "init-config" || w == "update-config")
    {
        return Ok(());
    }
    let (files, names): (Vec<String>, Vec<String>) =
        cli.benches.drain(..).partition(|w| is_config_arg(w));
    cli.benches = names;
    let mut files = files.into_iter();
    let Some(file) = files.next() else {
        return Ok(());
    };
    if let Some(second) = files.next() {
        return Err(format!(
            "'{file}' and '{second}' both name the run's config: a run has one"
        ));
    }
    if let Some(flag) = &cli.config {
        return Err(format!(
            "'{file}' and --config {} both name the run's config: keep one",
            flag.display()
        ));
    }
    cli.config = Some(file.into());
    Ok(())
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

/// An on/off flag's optional value, `--verbose=no`, so the line can cancel what a config file
/// turned on. The bare flag is `yes`.
fn parse_yes_no(value: &str) -> Result<bool, String> {
    match value {
        "yes" => Ok(true),
        "no" => Ok(false),
        other => Err(format!("{other:?} is not yes or no")),
    }
}

const DEFAULT_DURATION: f64 = 5.0;
const DEFAULT_BAND_LABELS: bands::BandLabels = bands::BandLabels::Both;
const DEFAULT_DECIMALS: u8 = 1;

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
    CompleteEnv::with_factory(Cli::command).complete();
    let mut cli = Cli::parse();
    if let Err(e) = take_config_arg(&mut cli) {
        eprintln!("error: {e}");
        std::process::exit(2);
    }
    // Refused before any setup, since an exit past the clock pin skips its restore.
    if cli.record.is_some() {
        eprintln!(
            "error: --record is gone, since a path's trailing '/' picked its mode: \
             --record-dir DIR writes a file per invocation, --record-file PATH appends every \
             record to one file"
        );
        std::process::exit(2);
    }

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
        let config = load_config_or_exit(cli.config.as_deref());
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
        let config = load_config_or_exit(cli.config.as_deref());
        std::process::exit(freqctl::cmd_restore_freq(
            config.freq.as_ref(),
            config.source("freq"),
        ));
    }

    // 'update-config FILE' rewrites a config in place from its own values and the line's.
    if cli.benches.iter().any(|b| b == "update-config") {
        if cli.benches[0] != "update-config" || cli.benches.len() != 2 {
            eprintln!("error: 'update-config' runs alone, with the one FILE to rewrite");
            std::process::exit(2);
        }
        if cli.from.is_some() || cli.config.is_some() {
            eprintln!("error: update-config: FILE is the start, so drop --from and --config");
            std::process::exit(2);
        }
        let line = match line_values(&cli) {
            Ok(table) => table,
            Err(e) => {
                eprintln!("error: update-config: {e}");
                std::process::exit(2);
            }
        };
        std::process::exit(init_config::update(
            std::path::Path::new(&cli.benches[1]),
            cli.backup,
            line,
        ));
    }

    // 'init-config' prints or writes the starting config and exits, with one optional PATH arg.
    if cli.benches.iter().any(|b| b == "init-config") {
        if cli.benches[0] != "init-config" || cli.benches.len() > 2 {
            eprintln!("error: 'init-config' runs alone, with at most one PATH arg");
            std::process::exit(2);
        }
        let start = match (cli.from.as_deref(), cli.config.as_deref()) {
            (Some(_), Some(_)) => {
                eprintln!(
                    "error: init-config: --from and --config both name the file to start from: \
                     keep one"
                );
                std::process::exit(2);
            }
            (Some(old), None) => init_config::Start::From(old),
            (None, Some(name)) => init_config::Start::Named(name),
            (None, None) => init_config::Start::Host,
        };
        let line = match line_values(&cli) {
            Ok(table) => table,
            Err(e) => {
                eprintln!("error: init-config: {e}");
                std::process::exit(2);
            }
        };
        let existing = match (cli.backup, cli.overwrite) {
            (true, _) => init_config::Existing::Backup,
            (false, true) => init_config::Existing::Overwrite,
            (false, false) => init_config::Existing::Refuse,
        };
        std::process::exit(init_config::run(
            cli.benches.get(1).map(std::path::Path::new),
            start,
            line,
            existing,
        ));
    }
    if cli.backup || cli.overwrite {
        eprintln!("error: --backup and --overwrite belong to 'init-config' and 'update-config'");
        std::process::exit(2);
    }
    if cli.from.is_some() {
        eprintln!("error: --from belongs to 'init-config'");
        std::process::exit(2);
    }

    // 'setup-freq' prepares the host and exits: it reads the live clock
    // state and the XDG config itself, so it needs neither the
    // layered config nor the banner.
    if cli.benches.iter().any(|b| b == "setup-freq") {
        if cli.benches.len() > 1 {
            eprintln!("error: 'setup-freq' runs alone; drop the other bench args");
            std::process::exit(2);
        }
        std::process::exit(setup::run(cli.apply, cli.uninstall));
    }

    // A bench child runs its one bench from the parent's spec and exits: no config, no inhibit,
    // no clock pin, and no banner, all of which the parent owns, its `-v` included.
    if let Some(spec) = &cli.child_spec {
        init_logger(cli.verbose == Some(true));
        std::process::exit(child::child_main(spec));
    }

    // Layered defaults (built-in < XDG file < project-local file <
    // CLI). A malformed config is fatal so a typo surfaces. Loaded
    // before the logger and the inhibit, since `verbose` and
    // `inhibit` are keys.
    let (config, config_files) = match config::load(cli.config.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: config: {e}");
            std::process::exit(2);
        }
    };
    let (verbose, verbose_src) = layered(
        cli.verbose,
        "--verbose",
        config.verbose,
        "verbose",
        &config,
        false,
    );
    init_logger(verbose);

    // Checked here rather than by clap, since `init-config PATH --benches a` is a positional and
    // the flag together, and is how a written file gets its benches.
    if !cli.benches.is_empty() && !cli.benches_flag.is_empty() {
        eprintln!("error: --benches and bench names on the line both name the benches: keep one");
        std::process::exit(2);
    }

    if cli.benches.is_empty() && cli.benches_flag.is_empty() && config.benches.is_none() {
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

    // Re-exec under systemd-inhibit (unless turned off or already
    // inhibited) before any output, so the banner prints once,
    // from the inhibited child.
    let (inhibit, inhibit_src) = layered(
        cli.no_inhibit.map(|no| !no),
        "--no-inhibit",
        config.inhibit,
        "inhibit",
        &config,
        true,
    );
    let inhibit_status = inhibit::ensure(inhibit, &run_config::source_name(&inhibit_src));

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

    let (pin_cpus_spec, pin_cpus_src) = run_config::layered_opt(
        cli.pin_cpus.clone(),
        "--pin-cpus",
        config.pin_cpus.clone(),
        "pin_cpus",
        &config,
    );
    let pin_cpus: Vec<usize> = match pin_cpus_spec.as_deref() {
        None => Vec::new(),
        // A spec naming a config profile expands to its CPU list.
        // Anything else parses as a raw CPU spec.
        Some(spec) => match config.resolve_pin(spec).and_then(pin::parse_cpus) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: pin_cpus: {e}");
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
        runs::DEFAULT_RUN_SLEEP_S,
    );
    let cli_trim_runs = match cli.trim_runs.as_deref() {
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

    // Duration precedence: CLI -d / -D win, then the config's
    // `duration` or `total_duration`, of which the files leave one
    // at most, then the built-in default. A total is split over
    // every run, and the duration row says so.
    let (total_duration, total_duration_src) = match cli.duration {
        Some(_) => (None, Source::Default),
        None => run_config::layered_opt(
            cli.total_duration,
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
    let (samples, samples_src) =
        run_config::layered_opt(cli.samples, "--samples", config.samples, "samples", &config);
    let (inner, inner_src) =
        run_config::layered_opt(cli.inner, "--inner", config.inner, "inner", &config);
    let (env_probe, env_probe_src) = layered(
        cli.no_env_probe.map(|no| !no),
        "--no-env-probe",
        config.env_probe,
        "env_probe",
        &config,
        true,
    );
    let (report_ticks, ticks_src) =
        layered(cli.ticks, "--ticks", config.ticks, "ticks", &config, false);
    let cli_record = match (&cli.record_dir, &cli.record_file) {
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
    if cli.record_label.is_some()
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
    for tag in &cli.tag {
        match tag.split_once('=') {
            Some((k, v)) if !k.is_empty() => tag_map.insert(k.to_string(), v.to_string()),
            _ => {
                eprintln!("error: --tag '{tag}' is not key=value");
                std::process::exit(2);
            }
        };
    }
    if let Some(label) = &cli.record_label {
        if cli.tag.iter().any(|t| t.starts_with("label=")) {
            eprintln!("error: --record-label and --tag label= both name the label: keep one");
            drop(freq_pin);
            std::process::exit(2);
        }
        tag_map.insert(record::LABEL_TAG.to_string(), label.clone());
    }
    let tags: Vec<String> = tag_map.iter().map(|(k, v)| format!("{k}={v}")).collect();
    let tag_flag = match (cli.tag.is_empty(), cli.record_label.is_some()) {
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
        Some(spec) => match config.resolve_pin(spec) {
            Ok(cpus) if cpus != spec => format!("{spec} = {cpus}"),
            _ => spec.to_string(),
        },
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
        Param::new("pin_cpus", pin_cpus_value, "none", pin_cpus_src),
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
    let run_files = match (&cli.config, config_files.last()) {
        (Some(_), Some(named)) => std::slice::from_ref(named),
        _ => config_files.as_slice(),
    };
    // The benches are the ones the run resolved, positional words included, which `line_values`
    // does not carry, since a positional on an `init-config` line is its path.
    let run_benches = match &suggest {
        Some(name) => vec![name.clone()],
        None => bench_list.clone(),
    };
    let run_table = match line_values(&cli).and_then(|mut line| {
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
            Ok(r) => Some(r),
            Err(e) => {
                eprintln!("error: record: {e}");
                std::process::exit(2);
            }
        },
    };

    let cfg = harness::RunCfg {
        target_seconds,
        samples_override: samples,
        inner_override: inner,
        pin_cpus: &pin_cpus,
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
    // The record target goes to the children absolute.
    let record_spec = child::RecordSpec {
        target: record_target.as_ref().map(record::Target::absolute),
        tags,
        config: record_config,
        series: record::new_series_id(),
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
    for (name, _) in &runners {
        if let Err(e) = runner.bench(name, &cfg, &record_spec) {
            eprintln!("error: {e}");
            drop(scratch);
            drop(freq_pin);
            std::process::exit(1);
        }
    }
}

/// The run flags on the line as config keys, for `init-config` to set in the file it writes: a
/// flag's key and its value as the config spells it. A flag that is not a run parameter is an
/// error by name, since a flag this command ignored in silence once wrote a file of defaults.
fn line_values(cli: &Cli) -> Result<toml::Table, String> {
    use toml::Value;
    for (set, flag) in [
        (cli.print_only, "--print-only"),
        (cli.as_config, "--as-config"),
        (cli.apply, "--apply"),
        (cli.uninstall, "--uninstall"),
    ] {
        // --backup and --overwrite say what to do with the file, and their commands read them.
        if set {
            return Err(format!("{flag} is not a run parameter, so no key holds it"));
        }
    }
    let count = |n: u64, flag: &str| match i64::try_from(n) {
        Ok(n) => Ok(Value::Integer(n)),
        Err(_) => Err(format!("{flag}: {n} is too large for a config")),
    };
    let text = |s: &str| Value::String(s.to_string());
    let mut t = toml::Table::new();
    if !cli.benches_flag.is_empty() {
        let names = cli.benches_flag.iter().map(|b| text(b)).collect();
        t.insert("benches".to_string(), Value::Array(names));
    }
    // Seconds reach here parsed, so a `-d 250ms` is written `0.25`.
    for (key, value) in [
        ("duration", cli.duration),
        ("total_duration", cli.total_duration),
        ("settle_time", cli.settle_time),
        ("warm_cap", cli.warm_cap),
    ] {
        if let Some(seconds) = value {
            t.insert(key.to_string(), Value::Float(seconds));
        }
    }
    for (key, flag, value) in [
        ("samples", "--samples", cli.samples),
        ("inner", "--inner", cli.inner),
        ("runs", "--runs", cli.runs),
        ("blocks", "--blocks", cli.blocks),
        ("decimals", "--decimals", cli.decimals.map(u64::from)),
    ] {
        if let Some(n) = value {
            t.insert(key.to_string(), count(n, flag)?);
        }
    }
    for (key, value) in [
        ("pin_cpus", cli.pin_cpus.as_deref()),
        ("run_sleep", cli.run_sleep.as_deref()),
        ("trim_runs", cli.trim_runs.as_deref()),
        ("block_sleep", cli.block_sleep.as_deref()),
        ("block_warmup", cli.block_warmup.as_deref()),
        (
            "band_labels",
            cli.band_labels.map(bands::BandLabels::as_str),
        ),
    ] {
        if let Some(s) = value {
            t.insert(key.to_string(), text(s));
        }
    }
    for (key, value) in [
        ("record_dir", &cli.record_dir),
        ("record_file", &cli.record_file),
    ] {
        if let Some(path) = value {
            t.insert(key.to_string(), text(&path.to_string_lossy()));
        }
    }
    // The two `no-` flags are the key's opposite.
    for (key, value) in [
        ("verbose", cli.verbose),
        ("ticks", cli.ticks),
        ("env_probe", cli.no_env_probe.map(|no| !no)),
        ("inhibit", cli.no_inhibit.map(|no| !no)),
    ] {
        if let Some(on) = value {
            t.insert(key.to_string(), Value::Boolean(on));
        }
    }
    // A bare --pin-freq is the word it stands for, and a frequency is a number.
    let pin_freq = match &cli.pin_freq {
        None => None,
        Some(None) => Some(text("pin_mhz")),
        Some(Some(word)) => Some(match word.parse::<i64>() {
            Ok(mhz) => Value::Integer(mhz),
            Err(_) => text(word),
        }),
    };
    if let Some(value) = pin_freq {
        t.insert("pin_freq".to_string(), value);
    }
    if !cli.tag.is_empty() || cli.record_label.is_some() {
        let mut tags = toml::Table::new();
        if let Some(label) = &cli.record_label {
            tags.insert(record::LABEL_TAG.to_string(), text(label));
        }
        for tag in &cli.tag {
            match tag.split_once('=') {
                Some((k, v)) if !k.is_empty() => tags.insert(k.to_string(), text(v)),
                _ => return Err(format!("--tag '{tag}' is not key=value")),
            };
        }
        t.insert("tags".to_string(), Value::Table(tags));
    }
    Ok(t)
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
        let err = check_bench_words(&words(&["min-now", "setup-freq"])).unwrap_err();
        assert!(err.contains("'setup-freq'"), "unexpected error: {err}");
    }

    #[test]
    fn benches_flag_splits_on_commas_and_sits_beside_a_command_words_path() {
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
        // The flag beside a positional parses, since `init-config PATH --benches a` needs both,
        // and `main` refuses it on a bench line.
        let cli = Cli::try_parse_from(["iiac-perf", "init-config", "q.md", "--benches", "min-now"])
            .expect("parses");
        let line = line_values(&cli).expect("values");
        assert_eq!(line["benches"].as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn the_record_flags_become_their_keys_and_the_label_a_tag() {
        let cli = Cli::try_parse_from([
            "iiac-perf",
            "init-config",
            "q.toml",
            "--record-dir",
            "runs",
            "--record-label",
            "pins",
            "--tag",
            "host=a",
        ])
        .expect("parses");
        let line = line_values(&cli).expect("values");
        assert_eq!(line["record_dir"].as_str(), Some("runs"));
        assert_eq!(line["tags"]["label"].as_str(), Some("pins"));
        assert_eq!(line["tags"]["host"].as_str(), Some("a"));
        assert!(
            Cli::try_parse_from(["iiac-perf", "--record-dir", "a", "--record-file", "b"]).is_err()
        );
    }

    #[test]
    fn an_on_off_flag_is_bare_or_takes_yes_or_no() {
        let cli = Cli::try_parse_from(["iiac-perf", "min-now"]).expect("parses");
        assert_eq!(
            (cli.verbose, cli.ticks, cli.no_env_probe, cli.no_inhibit),
            (None, None, None, None)
        );
        let cli = Cli::try_parse_from(["iiac-perf", "-v", "-t", "--no-inhibit", "min-now"])
            .expect("parses");
        assert_eq!(
            (cli.verbose, cli.ticks, cli.no_inhibit),
            (Some(true), Some(true), Some(true))
        );
        // The bare flag takes no following word, so the bench name stays a bench name.
        assert_eq!(cli.benches, ["min-now"]);
        let cli = Cli::try_parse_from(["iiac-perf", "--verbose=no", "--no-env-probe=no"])
            .expect("parses");
        assert_eq!((cli.verbose, cli.no_env_probe), (Some(false), Some(false)));
        assert!(Cli::try_parse_from(["iiac-perf", "--ticks=maybe"]).is_err());
        // A tag needs a record, but the record may come from a file, so the line alone parses.
        assert!(Cli::try_parse_from(["iiac-perf", "--tag", "k=v", "min-now"]).is_ok());
    }

    #[test]
    fn a_positional_ending_in_a_carriers_extension_is_the_config() {
        let parse = |args: &[&str]| {
            let mut cli = Cli::try_parse_from(args).expect("parses");
            take_config_arg(&mut cli).map(|()| cli)
        };
        let cli = parse(&["iiac-perf", "min-now", "configs/queue.md", "zcr"]).unwrap();
        assert_eq!(cli.benches, ["min-now", "zcr"]);
        assert_eq!(
            cli.config.as_deref(),
            Some(std::path::Path::new("configs/queue.md"))
        );
        let cli = parse(&["iiac-perf", "queue.toml"]).unwrap();
        assert!(cli.benches.is_empty());
        // A bare name stays a bench, and a pattern with a dot in it is no file.
        let cli = parse(&["iiac-perf", "queue", "zcr-.psc"]).unwrap();
        assert_eq!((cli.benches.len(), cli.config.is_none()), (2, true));
        assert!(parse(&["iiac-perf", "a.md", "b.toml"]).is_err());
        assert!(parse(&["iiac-perf", "a.md", "--config", "b"]).is_err());
        // The two commands that write a file keep their positional.
        let cli = parse(&["iiac-perf", "init-config", "q.md", "--config", "base"]).unwrap();
        assert_eq!(cli.benches, ["init-config", "q.md"]);
        let cli = parse(&["iiac-perf", "update-config", "q.md"]).unwrap();
        assert_eq!(cli.benches.len(), 2);
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
        // A config file is offered once something is typed, a path's directory kept.
        assert_eq!(values("iiac-perf.ex"), ["iiac-perf.example.md"]);
        assert_eq!(values("docs/conf"), ["docs/config.md"]);
    }

    #[test]
    fn wrap_names_breaks_at_width() {
        // "ccc" would land past col 10, so it wraps, and the separator
        // comma stays on the prior line and the new line re-indents.
        assert_eq!(wrap_names(&["aaa", "bbb", "ccc"], 10), "  aaa, bbb,\n  ccc");
    }
}
