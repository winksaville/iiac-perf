//! The command line: the subcommands, each a struct of the flags it owns, and the helpers that
//! read them.
//!
//! - [`Cli`] is the top level: the subcommand, and the few flags that come before one.
//! - [`MeasureArgs`] holds the run parameters, the flags a config file has a key for, so every
//!   command that measures or writes a config flattens the one struct and [`line_values`] reads
//!   it as the keys.
//! - A first word that is no subcommand reaches [`Command::Unknown`], so an old command word or
//!   a bench without `run` is answered with the line to type.

use clap::{Args, Parser, Subcommand};
use clap_complete::{ArgValueCompleter, CompletionCandidate};

use crate::{ABOUT, BIN_NAME, bands, benches, figures, inner, record, series, timespec};

#[derive(Parser)]
#[command(version, about = ABOUT, max_term_width = 80)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

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
        global = true,
        value_name = "yes|no",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "yes",
        value_parser = parse_yes_no
    )]
    pub verbose: Option<bool>,

    /// Print the registered bench names, one per line, and exit.
    ///
    /// No bench runs. Machine-readable, for scripts to iterate.
    /// The subcommands are not bench names and are not listed.
    #[arg(long)]
    pub list_benches: bool,

    /// Run as a bench child: the spec file the parent wrote. Internal, so hidden: every bench runs
    /// in a child of its own, spawned by the parent with this flag.
    #[arg(long, hide = true, value_name = "PATH")]
    pub child_spec: Option<std::path::PathBuf>,
}

/// The subcommands. A bare `iiac-perf` prints the bench list.
// The variants differ in size by their flags, and one is built per process, so boxing buys
// nothing.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
pub enum Command {
    /// Run benches, each in fresh processes, and report each one.
    Run(RunArgs),

    /// Is this machine fit to measure on?
    ///
    /// Respawns this binary --runs times after --run-sleep, each child a one-run `run min-now`,
    /// collects each run's environment grade, and prints the table and a verdict: QUALIFIED when
    /// the median grade is B or better and no run's drift or step reached D or F, and a nonzero
    /// exit when not. It grades the environment, not the run: the machine is the subject, not a
    /// workload.
    #[command(visible_alias = "qualify-environment")]
    Qualify(QualifyArgs),

    /// Check a claim across invocations, from records.
    ///
    /// Reads record files, and directories of them, and qualifies each group of invocations
    /// against itself: the spread of their trimmed means against the LSC trimmed each claimed,
    /// the pairs beyond their claim, and the change one invocation against one could detect. A
    /// group is a bench, a host, a placement when the records name one, and the value of each
    /// --by TAG.
    Analyze(AnalyzeArgs),

    /// What one step costs, from samples of many lengths.
    ///
    /// Reads record files, and directories of them, taken at several values of --inner, and
    /// regresses sample time on inner: the slope is what one more step adds and the intercept
    /// what a sample of no steps would take. The line is tested, its halves refitted and its
    /// residuals searched for a period, and where it does not hold no cost is named. Runs at
    /// fixed counts and runs over a span (--inner 1-16) are fitted apart. The harness's overhead
    /// a sample is taken as known and taken off each length's sample: what is left over inner is
    /// the step there, which a group too short to regress is given too.
    #[command(visible_alias = "lr")]
    LinearRegression(LinearRegressionArgs),

    /// Draw records as a PNG or SVG.
    ///
    /// Draws the block means of record files, and directories of them, as one image: a panel
    /// per bench and invocation, each run a line of its block means against time, the trimmed
    /// mean dashed.
    Figures(FiguresArgs),

    /// Print the record field dictionary.
    ///
    /// Every record key with its unit and one-line meaning, and the schema_version the dictionary
    /// describes: --help documents the inputs, and this the recorded output.
    DescribeRecord,

    /// The CPU clock: read it, pin it, restore it, or find the pin a bench holds.
    #[command(subcommand)]
    Freq(FreqCommand),

    /// Make this host ready for every run: placements, clock, and permissions.
    ///
    /// Plans setup freq's [freq] and permissions, and the host's [profiles], the placements read
    /// from sysfs by the base-CPU rule (smt, ccx, x-ccx as the host can form them), and prints
    /// the plan. --apply writes the config and calls sudo once for the permissions. A declared
    /// [profiles] is left alone and checked. Run as your user, not under sudo.
    Setup(SetupArgs),

    /// Write a config file: a starting one, or one rewritten in place.
    #[command(subcommand)]
    Config(ConfigCommand),

    /// A first word that is no subcommand: an old command word, or a bench without `run`, each
    /// answered with the line to type.
    #[command(external_subcommand)]
    Unknown(Vec<String>),
}

/// `run`: the benches, the run's config file, and the run parameters.
#[derive(Args)]
pub struct RunArgs {
    /// Benches to run, or the run's config file.
    ///
    /// Pass 'all' for every registered bench, or one or more
    /// names. A name matching no bench exactly runs every bench
    /// it is a prefix of (e.g. 'ice', 'mpsc'), and one that is no
    /// prefix either runs every bench it matches as a regular
    /// expression (e.g. 'zcr-[sm]psc-v[23]'). A word ending in .md
    /// or .toml is the run's config file, as --config with it:
    /// 'iiac-perf run queue.md'. Bench names beside it win over the
    /// file's `benches`. With no bench names, --benches or the
    /// config `benches` names the benches, and with none of them
    /// either, the available list prints.
    #[arg(value_name = "BENCH", add = ArgValueCompleter::new(complete_bench))]
    pub benches: Vec<String>,

    /// The run's config file, by name. A positional ending in
    /// .md or .toml is the same: 'iiac-perf run queue.md'.
    ///
    /// The run keys come from this file and the built-in defaults
    /// alone, flags still winning, so one file is one run on every
    /// host. The XDG and project-local files give only [freq] and
    /// [profiles], under whatever this file sets of them. An
    /// absolute NAME is taken as given. A relative one is looked
    /// for in the current directory, each parent, then the XDG
    /// config directory, the first found winning, as NAME, NAME.md,
    /// or NAME.toml. Not found is an error.
    #[arg(long, value_name = "NAME", add = ArgValueCompleter::new(complete_config))]
    pub config: Option<std::path::PathBuf>,

    #[command(flatten)]
    pub measure: MeasureArgs,

    /// Under a span, keep the run table to its plain columns.
    ///
    /// The `inner` column then shows the span, `5-12`, as it does a fixed count's number, and the
    /// `residual` column is dropped, so no line is fitted and no graph drawn.
    #[arg(long)]
    pub no_graph: bool,
}

/// `qualify`: the children it spawns, and what each passes on.
#[derive(Args)]
pub struct QualifyArgs {
    /// Each child's measuring time: seconds bare, or a duration with unit (us, ms, s).
    ///
    /// Short on purpose: the qualification wants many fresh processes rather than a few long
    /// ones, since a respawn is what re-rolls the box's state.
    #[arg(
        short = 'd',
        long,
        value_name = "DUR",
        default_value_t = crate::QUALIFY_CHILD_SECONDS,
        value_parser = timespec::parse_seconds
    )]
    pub duration: f64,

    /// Child runs to spawn.
    #[arg(
        long,
        value_name = "N",
        default_value_t = crate::QUALIFY_RUNS,
        value_parser = clap::value_parser!(u64).range(1..=1000)
    )]
    pub runs: u64,

    /// Sleep before each child run: a duration or range with unit (us, ms, s).
    ///
    /// 0 sustains the duty cycle that provokes a state transition, where a sleep probes a quieter
    /// one. A range is re-rolled per child.
    #[arg(long, value_name = "SPAN", default_value = "0", value_parser = timespec::parse_span)]
    pub run_sleep: (f64, f64),

    /// Each child's --pin-cpus, passed through.
    #[arg(long, alias = "pin", value_name = "CPUS", add = ArgValueCompleter::new(complete_placements))]
    pub pin_cpus: Option<String>,

    /// Each child's --settle-time, passed through: seconds bare, or a duration with unit.
    ///
    /// Without it each child takes its own default, so the table reflects what a plain run of
    /// this binary does.
    #[arg(long, value_name = "DUR", allow_negative_numbers = true, value_parser = timespec::parse_seconds)]
    pub settle_time: Option<f64>,

    /// Print the table and skip the verdict.
    #[arg(long)]
    pub print_only: bool,
}

/// The record files a record command reads: files, and directories of them.
#[derive(Args)]
pub struct RecordPaths {
    /// Record files, and directories of them.
    #[arg(value_name = "PATH", required = true)]
    pub paths: Vec<std::path::PathBuf>,

    /// The band of the run means the trimmed rows keep, FROM-TO in whole percents.
    ///
    /// `10-50` keeps the 10th to the 50th percentile, cutting mostly high since a run can land
    /// slow and never fast. `20-80` is the symmetric middle 60%, `0-100` no trim. Set it once for
    /// a project, never per comparison: a trim picked after seeing the numbers flatters them.
    #[arg(
        long,
        value_name = "FROM-TO",
        default_value_t = series::Trim::DEFAULT,
        value_parser = series::Trim::parse
    )]
    pub trim_runs: series::Trim,
}

/// `analyze`: the records, and how to group and compare their invocations.
#[derive(Args)]
pub struct AnalyzeArgs {
    #[command(flatten)]
    pub records: RecordPaths,

    /// Group invocations by this tag too (repeatable).
    ///
    /// Invocations always group by bench and host, and each --by
    /// adds a tag's value, `-` where a record lacks it, so two make
    /// a grid: '--by cpus --by freq'.
    #[arg(long, value_name = "TAG")]
    pub by: Vec<String>,

    /// Compare the invocations whose KEY is A with those whose KEY
    /// is B (repeatable, each adding its pairs).
    ///
    /// KEY is any tag, or `bench`, `host`, `placement`, `binary`, or
    /// `file`. KEY alone compares every value the records hold, in
    /// the order each first ran, and a bare --compare every bench.
    /// More than two values make a ladder, each against the first
    /// and against the one before: '--compare bench=zcr-spsc-v0-2t,
    /// zcr-spsc-v1-2t,zcr-spsc-v2-2t'. Invocations sharing a series
    /// pair by it, sides alternating in time pair as neighbours, and
    /// otherwise the two groups compare whole. '--compare bench=a,b
    /// --compare bench=a,c' is a against b and a against c alone.
    #[arg(
        long,
        value_name = "KEY=A,B",
        num_args = 0..=1,
        default_missing_value = "bench",
        add = ArgValueCompleter::new(complete_compare)
    )]
    pub compare: Vec<String>,
}

/// `linear-regression`: the records, and the harness overhead when it is known.
#[derive(Args)]
pub struct LinearRegressionArgs {
    #[command(flatten)]
    pub records: RecordPaths,

    /// The harness's overhead a sample, in ns, taken as known.
    ///
    /// Without it the overhead is the intercept of `min-now` on the same host, from the files
    /// read, where its line holds. It is taken off each length's sample, and what is left over
    /// `inner` is the cost of a step there.
    #[arg(long, value_name = "NS", allow_negative_numbers = true, value_parser = parse_overhead)]
    pub overhead: Option<f64>,
}

/// `figures`: the records, what each panel draws, and the file to write.
#[derive(Args)]
pub struct FiguresArgs {
    #[command(flatten)]
    pub records: RecordPaths,

    /// The benches to draw, comma-separated or repeated (default all).
    #[arg(long, value_name = "BENCH", value_delimiter = ',', add = ArgValueCompleter::new(complete_bench_list))]
    pub bench: Vec<String>,

    /// The one invocation to draw, by its series id (default every one).
    #[arg(long, value_name = "ID")]
    pub series: Option<String>,

    /// The runs each panel draws.
    ///
    /// `all`, `trim` (every run, the ones the trim drops in grey),
    /// `extremes` (the fastest and slowest), or run numbers, `3,1,7`,
    /// coloured in that order.
    #[arg(
        long,
        value_name = "RUNS",
        default_value = "all",
        value_parser = figures::Show::parse,
        add = ArgValueCompleter::new(complete_show)
    )]
    pub show: figures::Show,

    /// The x axis.
    #[arg(long, value_name = "AXIS", value_enum, default_value_t = figures::XAxis::Time)]
    pub x_axis: figures::XAxis,

    /// The figure to write, a .png or a .svg by its extension.
    #[arg(long, value_name = "FILE", default_value = "block-means.png")]
    pub out: std::path::PathBuf,
}

/// An overhead in ns: a finite time, zero or more.
fn parse_overhead(s: &str) -> Result<f64, String> {
    match s.parse::<f64>() {
        Ok(ns) if ns.is_finite() && ns >= 0.0 => Ok(ns),
        _ => Err(format!("{s} is not a time in ns")),
    }
}

/// The run parameters: every flag a config file has a key for, layered over the files by
/// `run_config::layered` and written as keys by [`line_values`].
#[derive(Args)]
pub struct MeasureArgs {
    /// Benches to run, comma-separated or repeated.
    ///
    /// The flag form of `run`'s bench names, for a line that
    /// reads better with every input named: names, prefixes,
    /// patterns, or 'all', never a command. Overrides the config
    /// `benches`. An error beside bench names given positionally.
    /// On a 'config init' or 'config update' line it sets the
    /// file's `benches`.
    #[arg(long = "benches", value_name = "BENCH", value_delimiter = ',', add = ArgValueCompleter::new(complete_bench_list))]
    pub benches_flag: Vec<String>,

    /// Target wall-clock time per bench: seconds bare, or a
    /// duration with unit (us, ms, s).
    ///
    /// Default 0.25, or the config `duration`. Auto-sizes the sample
    /// and inner loop counts. Mutually exclusive with -D.
    #[arg(short = 'd', long, conflicts_with = "total_duration", value_name = "DUR", value_parser = timespec::parse_seconds)]
    pub duration: Option<f64>,

    /// Target total wall-clock time across all benches: seconds
    /// bare, or a duration with unit.
    ///
    /// The budget is split equally over every run of every bench,
    /// benches times --runs. Mutually exclusive with -d. Overrides
    /// the config `total_duration` and `duration`.
    #[arg(short = 'D', long, value_name = "DUR", value_parser = timespec::parse_seconds)]
    pub total_duration: Option<f64>,

    /// Override the sample count (skips auto-sizing, and inner still
    /// adapts), rounded up to whole blocks so every block runs
    /// the same count, never cut by the blocks' time cap. `-o` / `--outer`, the count's old name,
    /// still work. Overrides the config `samples`. The run table's `samples` column shows the
    /// count each run took, and its summary the range across runs.
    #[arg(short, long, short_alias = 'o', alias = "outer")]
    pub samples: Option<u64>,

    /// Override inner loop count (skips auto-sizing): a count, or a span like `1-16`.
    ///
    /// inner=1 measures single-call latency (each sample = one
    /// step). Higher inner measures back-to-back/burst rate
    /// (each sample = N steps averaged). A span draws each
    /// sample's count from it, so one run holds samples of every
    /// length and its record carries their totals by length,
    /// what `linear-regression` reads. Overrides the config `inner`.
    ///
    /// The run table's `inner` column shows the count each run
    /// sized itself to. Under a span, each sample of a run picks
    /// its own loop count from the span, so one run times every
    /// count in it. The column is then titled `inner LO-HI` and
    /// draws a bar a loop count, LO first: how far the mean time of
    /// that run's samples at that count falls above or below a
    /// straight line fitted through all the counts, the run's own
    /// fit of time on loop count, as `linear-regression` makes it.
    /// A `residual` column beside it is the largest distance from
    /// that line, in ns of sample time, signed and at its loop
    /// count, `+4.288 ns at 14`: `+` where the samples of 14 ran
    /// slower than the line, `-` where they ran faster. The 14 is
    /// the loop count, not the bar's position: under `inner 3-18`
    /// it is the twelfth bar. The bars are scaled to each run's own
    /// smallest and largest, so read the shape with `residual`: one
    /// that changes run to run at a small `residual` is noise, and
    /// one every run repeats is a cost that is not a straight line
    /// in the loop count, as a period of 4 would be. `--no-graph`
    /// keeps the column to the span.
    #[arg(short, long)]
    pub inner: Option<inner::InnerSpan>,

    /// Pin bench threads to CPUs (comma-separated, ranges OK).
    ///
    /// A CPU is the kernel's schedulable unit (sysfs cpuN, one
    /// affinity-mask bit). A physical core hosts two of them when
    /// SMT is on. The list is a CPU *pool*: thread `i` of a bench
    /// is pinned to `pool[i]`, in the order its roles are listed
    /// in, and a thread past the pool's end runs unpinned.
    /// Examples: `--pin-cpus 0,1` (2 threads -> 2 CPUs),
    /// `--pin-cpus 0` (main on 0, a worker unpinned), `--pin-cpus
    /// 0,0` (two threads on the same CPU). On 3900X,
    /// CPUs N and N+12 are SMT siblings of the same physical core:
    /// `--pin-cpus 0,12` pairs siblings (max contention),
    /// `--pin-cpus 0,1` gives independent cores. A value naming a
    /// `[profiles]` entry in the config file expands to that
    /// profile's CPU spec (e.g. `--pin-cpus smt`), and a name the
    /// host's topology contradicts, `smt` on two cores, is an
    /// error. A comma list of names, `smt,ccx,unpinned`, runs every
    /// bench at each placement in turn and ends with a table of
    /// benches by placements, and `all` is every declared profile,
    /// nearest first, then `unpinned`. `nearest`, the default, is
    /// the first of `smt`, `ccx`, and `x-ccx` the host declares, and
    /// `--pin-cpus ""` leaves threads unpinned. `--pin` is a hidden
    /// alias. Overrides the config `pin_cpus`.
    #[arg(long, alias = "pin", value_name = "CPUS", add = ArgValueCompleter::new(complete_placements))]
    pub pin_cpus: Option<String>,

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
    pub ticks: Option<bool>,

    /// Runs of each bench, each a fresh process (default 10).
    ///
    /// A process start re-rolls where a bench's memory lands, and
    /// that sets its level, so the runs' means are the replicates
    /// the bench's `CI95 runs` and `LSC runs` come from. A bench's
    /// runs go back to back. One run prints its report as a single
    /// process does, and several print a line per run and the
    /// bench's summary, `-v` adding every run's report. Overrides
    /// the config `runs`.
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u64).range(1..=1000))]
    pub runs: Option<u64>,

    /// Sleep before each run: a duration or range with unit (us, ms, s).
    ///
    /// Default 100ms, a range re-rolled per run, drawn before the first
    /// run too, so every run starts alike: without it the first
    /// run starts from whatever the host did before and the rest
    /// start hot from the run before. 0 starts each run as the
    /// last one ends. Overrides the config `run_sleep`.
    #[arg(long, value_name = "SPAN")]
    pub run_sleep: Option<String>,

    /// The band of the run means the trimmed rows keep, FROM-TO in whole percents: `10-50`
    /// keeps the 10th to the 50th percentile, cutting mostly high since a run can land slow and
    /// never fast. `20-80` is the symmetric middle 60%, `0-100` no trim. Set it once for a
    /// project, never per comparison: a trim picked after seeing the numbers flatters them.
    /// Overrides the config `trim_runs`.
    #[arg(long, value_name = "FROM-TO")]
    pub trim_runs: Option<String>,

    /// Pin the CPU clock for this run, restoring on exit.
    ///
    /// Engages before the warmup, exactly like 'freq pin': min =
    /// max at MHZ (--pin-freq=3800), or at the config [freq]
    /// value named (--pin-freq=min_mhz or max_mhz), bare meaning pin_mhz,
    /// else the discovered base clock, with boost off. The
    /// declared [freq] steady state is restored on normal exit,
    /// panic, SIGINT, and SIGTERM. After SIGKILL or power loss,
    /// run 'freq restore'. --pin-freq=no cancels a config file's
    /// pin_freq for this run. Overrides the config `pin_freq`, and
    /// both absent pins at pin_mhz. Needs root, or the permissions
    /// 'setup --apply' grants, and a declared [freq] steady state,
    /// and a run on a host without them asks to set it up.
    #[arg(
        long,
        value_name = "MHZ|pin_mhz|min_mhz|max_mhz|no",
        num_args = 0..=1,
        require_equals = true,
        add = ArgValueCompleter::new(complete_pin_freq)
    )]
    pub pin_freq: Option<Option<String>>,

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
    pub no_env_probe: Option<bool>,

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
    /// `settle_time`, and both absent defaults to 0.1.
    #[arg(long, value_name = "DUR", allow_negative_numbers = true, value_parser = timespec::parse_seconds)]
    pub settle_time: Option<f64>,

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
    /// defaults to 0.1.
    #[arg(long, value_name = "DUR", allow_negative_numbers = true, value_parser = timespec::parse_seconds)]
    pub warm_cap: Option<f64>,

    /// Band label style for the report's histogram rows.
    ///
    /// 'zpn': nines/zeros + decile names (z3, p50, n4).
    /// 'frac': literal boundary fractions with '_' grouping
    /// (0.001, 0.50, 0.999_9). 'both': zpn and fraction
    /// side by side: the juxtaposition teaches the zpn
    /// vocabulary. Switch to 'zpn' once fluent. Overrides the
    /// config `band_labels`, and both absent defaults to 'both'.
    #[arg(long, value_enum)]
    pub band_labels: Option<bands::BandLabels>,

    /// Decimal digits on the report's time columns (0-3).
    ///
    /// 1 shows the sub-ns precision picosecond recording
    /// captures, 0 restores integer ns, and 3 is the recording
    /// floor - more digits would be artifacts. Overrides the
    /// config `decimals`, and both absent defaults to 1.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=3))]
    pub decimals: Option<u8>,

    /// Measurement blocks per run (default 10).
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
    /// --block-warmup ask (100-200 ms and 2 ms by default, and neither is
    /// counted in the budget): the sleep makes the blocks genuine
    /// replicates, and '--block-sleep 0' leaves them partitions
    /// of one continuous run, where CI95 blocks / LSC blocks print
    /// '-'. Bench-driven benches only. Probe benches
    /// ignore it. Overrides the config `blocks`.
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u64).range(1..=1000))]
    pub blocks: Option<u64>,

    /// Sleep between blocks: a duration or range with unit (us, ms, s).
    ///
    /// E.g. '--block-sleep 1-10ms' re-rolls a random sleep per
    /// block (re-rolls scheduler and frequency state, and a range
    /// avoids phase-locking with kernel ticks), '--block-sleep 1s'
    /// sleeps exactly 1 s (a long sleep reaches deep C-states, so
    /// wakes start colder). Default 100-200ms, so every run's
    /// blocks are replicates. 0 never sleeps: the blocks are partitions of
    /// one continuous run and the replication rows print '-'.
    /// Overrides the config `block_sleep`.
    #[arg(long, value_name = "SPAN")]
    pub block_sleep: Option<String>,

    /// Unrecorded post-wake warmup per block: a duration with unit.
    ///
    /// Steps the bench unrecorded after each block sleep, keeping
    /// the frequency ramp and cache refill out of the samples.
    /// Default 2ms. 0 records from the first post-wake call, which
    /// is how cold-wake behavior is seen. Overrides the config
    /// `block_warmup`.
    #[arg(long, value_name = "DUR")]
    pub block_warmup: Option<String>,

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
    pub record_dir: Option<std::path::PathBuf>,

    /// Append every run's JSONL record to the one file PATH.
    ///
    /// As --record-dir, but every invocation appends to PATH, a
    /// line a run, and the open never truncates. PATH's directory
    /// is created. Overrides the config `record_dir` or
    /// `record_file`.
    #[arg(long, value_name = "PATH")]
    pub record_file: Option<std::path::PathBuf>,

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
    pub record_label: Option<String>,

    /// Retired: refused with a message naming --record-dir and
    /// --record-file, since a path's trailing '/' picked its mode.
    #[arg(long, hide = true, value_name = "PATH")]
    pub record: Option<std::path::PathBuf>,

    /// Tag every record with KEY=VALUE (repeatable).
    ///
    /// Recorded verbatim, never interpreted: the caller, not the
    /// tool, knows which runs form one experiment, so e.g.
    /// '--tag series=20260816T09' labels a series and '--tag
    /// condition=pinned' a condition. Adds to the config `[tags]`
    /// table, winning on a shared key. Needs a record target, from
    /// --record-dir, --record-file, or the config.
    #[arg(long, value_name = "KEY=VALUE")]
    pub tag: Vec<String>,

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
    pub no_inhibit: Option<bool>,
}

/// `freq`: the CPU clock's commands.
// One is built per process, as `Command` is.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
pub enum FreqCommand {
    /// Print the clock state, one line per policy group.
    ///
    /// The governor, EPP, boost, clamp, current frequency, and the base clock with its source.
    /// No root needed, and shaped for a prompt or a status bar.
    Read {
        /// Print the state as a config `[freq]` section instead.
        ///
        /// Ready to paste into a `toml` fence of the config file (usually
        /// ~/.config/iiac-perf/config.md), which is how the steady state that `freq pin` and
        /// `freq restore` need gets declared.
        #[arg(long)]
        as_config: bool,
    },

    /// Hold the CPU clock still (min = max, boost off) until `freq restore`.
    ///
    /// Pins at MHZ, or at the config [freq] value named, pin_mhz by default, else the base clock.
    /// The target must fit under the ceiling with boost off. Needs root or `setup`'s permissions,
    /// and refuses without a declared [freq] steady state in the config, the way home.
    Pin {
        /// The clock to pin: MHZ, or pin_mhz, min_mhz, or max_mhz from the config's [freq].
        #[arg(value_name = "MHZ|pin_mhz|min_mhz|max_mhz", add = ArgValueCompleter::new(complete_pin_target))]
        target: Option<String>,

        /// The config file that declares [freq], by name, as `run --config` finds it.
        #[arg(long, value_name = "NAME", add = ArgValueCompleter::new(complete_config))]
        config: Option<std::path::PathBuf>,
    },

    /// Converge to the declared [freq] steady state.
    ///
    /// The governor, EPP, boost, and clamps the config declares, from any starting point,
    /// including after an unclean death. Needs root or `setup`'s permissions.
    Restore {
        /// The config file that declares [freq], by name, as `run --config` finds it.
        #[arg(long, value_name = "NAME", add = ArgValueCompleter::new(complete_config))]
        config: Option<std::path::PathBuf>,
    },

    /// Measure the best pin frequency under a bench's load.
    ///
    /// Descends from the maximum with boost off, pins each candidate, drives BENCH, the real
    /// workload, with this line's run parameters, and reports the highest frequency the box
    /// held, ending with the pin_mhz line to paste. The suggestion is per bench, duration, and
    /// placement, since a schedule selects the state it can hold. Needs root or `setup`'s
    /// permissions and a declared [freq] steady state, and restores on exit as `freq pin` does.
    Suggest(SuggestArgs),
}

/// `freq suggest`: the one bench, the run's config file, and the run parameters.
#[derive(Args)]
pub struct SuggestArgs {
    /// The one bench whose load the suggestion serves, named exactly.
    #[arg(value_name = "BENCH", add = ArgValueCompleter::new(complete_bench))]
    pub bench: String,

    /// The run's config file, by name, as `run --config` finds it.
    #[arg(long, value_name = "NAME", add = ArgValueCompleter::new(complete_config))]
    pub config: Option<std::path::PathBuf>,

    #[command(flatten)]
    pub measure: MeasureArgs,
}

/// `setup`: the whole host, or one part of it, planned or applied.
#[derive(Args)]
pub struct SetupArgs {
    #[command(subcommand)]
    pub part: Option<SetupCommand>,

    /// Do what the plain command prints.
    ///
    /// Without it, the command changes nothing and shows the config it would write and the
    /// permissions it would install or remove. With it, the command writes the config and calls
    /// sudo once for the permissions.
    #[arg(long, global = true)]
    pub apply: bool,
}

/// `setup`'s parts.
#[derive(Subcommand)]
pub enum SetupCommand {
    /// Make this host ready for `freq pin` and `freq restore` alone.
    ///
    /// Prints the [freq] steady state it would write to ~/.config/iiac-perf/config.md from the
    /// live state, clamp limits included, and the udev rule that lets you pin and restore
    /// without sudo. Creates a missing config, appends to one without [freq], and leaves one that
    /// declares [freq] alone, checking it. Run as your user, not under sudo.
    Freq,

    /// Give back what setup took: the udev rule and the cpufreq files.
    ///
    /// Removes the udev rule and hands the cpufreq files back to root, with one sudo. The config
    /// is left alone.
    Uninstall,
}

/// `config`: the commands that write a config file.
// One is built per process, as `Command` is.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Print or write a starting config.
    ///
    /// Every key, commented out at its default, with the prose that explains it. With PATH,
    /// written there, as TOML when PATH ends in .toml. The start is the run keys this host's
    /// config files set, or --from, --config, or --from-record's, and the run parameters on the
    /// line go over it, so the file is the run that line makes here. [freq] and [profiles] stay
    /// the host's. --from /dev/null is the bare template.
    Init(InitArgs),

    /// Rewrite a config in place.
    ///
    /// The starting config with FILE's own values set and the line's run parameters over them,
    /// so 'config update queue.md --blocks 20' changes one key, and with none it brings an older
    /// file up to date. FILE is checked, filled, and checked again before it is touched. Prose
    /// and comments its author added are lost: --backup keeps the old file as FILE.bak.
    Update(UpdateArgs),
}

/// `config init`: where to write, what to start from, and the run parameters to set.
#[derive(Args)]
pub struct InitArgs {
    /// The file to write. Without it the config prints.
    #[arg(value_name = "PATH")]
    pub path: Option<std::path::PathBuf>,

    /// Carry this config file's values over.
    ///
    /// The new file is the starting config with every key OLD
    /// sets uncommented at OLD's value. OLD is read, never written,
    /// and a key no longer known fails by name, which is how an
    /// older file comes up to date.
    #[arg(long, value_name = "OLD", group = "start")]
    pub from: Option<std::path::PathBuf>,

    /// Start from a config file found by name, as `run --config` finds it.
    #[arg(long, value_name = "NAME", group = "start", add = ArgValueCompleter::new(complete_config))]
    pub config: Option<std::path::PathBuf>,

    /// Start from a record's run config.
    ///
    /// FILE is a record file, and the new file is the run one
    /// invocation in it made, from the record's config.run. What
    /// was its host's alone is named this host's way or left out
    /// with a comment: a pinned cpu list becomes this host's
    /// profile for the record's placement (smt, ccx, x-ccx), and a
    /// clock in MHz and a record path are left out. Notes say where
    /// this host's cpu, kernel, rustc, and version differ.
    #[arg(long, value_name = "FILE", group = "start")]
    pub from_record: Option<std::path::PathBuf>,

    /// The invocation of --from-record's FILE to start from, by its series id.
    ///
    /// Needed when FILE holds more than one invocation, as a
    /// --record-file experiment does. Without it such a FILE is
    /// refused with each series listed.
    #[arg(long, value_name = "ID", requires = "from_record")]
    pub series: Option<String>,

    /// Replace PATH when it exists, keeping the old file as PATH.bak.
    #[arg(long, conflicts_with = "overwrite")]
    pub backup: bool,

    /// Replace PATH when it exists, keeping nothing.
    ///
    /// The old file's values are not carried over, which is what
    /// 'config update' is for.
    #[arg(long)]
    pub overwrite: bool,

    #[command(flatten)]
    pub measure: MeasureArgs,
}

/// `config update`: the file to rewrite, and the run parameters to set in it.
#[derive(Args)]
pub struct UpdateArgs {
    /// The config file to rewrite in place.
    #[arg(value_name = "FILE")]
    pub file: std::path::PathBuf,

    /// Keep the old file as FILE.bak.
    ///
    /// Without it nothing of the old file is kept, and prose and
    /// comments its author added are gone.
    #[arg(long)]
    pub backup: bool,

    #[command(flatten)]
    pub measure: MeasureArgs,
}

/// Tab candidates for `run`'s positional: every registered bench name, `all`, and the config
/// files, each starting with what is typed so far. The shell calls the binary itself for these
/// (`COMPLETE=bash`, see `CompleteEnv`), so the list is always the running build's.
fn complete_bench(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let typed = current.to_string_lossy();
    // None for an empty word, where every README in the directory would crowd the bench names.
    let configs = if typed.is_empty() {
        Vec::new()
    } else {
        config_files(&typed)
    }
    .into_iter()
    .map(|f| CompletionCandidate::new(f).help(Some("run this config".into())));
    let all = CompletionCandidate::new("all").help(Some("run every registered bench".into()));
    benches::names()
        .into_iter()
        .map(CompletionCandidate::new)
        .chain(std::iter::once(all))
        .filter(|c| c.get_value().to_string_lossy().starts_with(typed.as_ref()))
        .chain(configs)
        .collect()
}

/// Candidates for one element of a comma list: each of `names` that starts as the typed list's
/// last element does, after the elements typed before it, so `smt,c` offers `smt,ccx`.
fn complete_list(
    current: &std::ffi::OsStr,
    names: impl IntoIterator<Item = (String, Option<&'static str>)>,
) -> Vec<CompletionCandidate> {
    let typed = current.to_string_lossy();
    let (done, last) = match typed.rsplit_once(',') {
        Some((done, last)) => (format!("{done},"), last),
        None => (String::new(), typed.as_ref()),
    };
    names
        .into_iter()
        .filter(|(name, _)| name.starts_with(last))
        .map(|(name, help)| {
            CompletionCandidate::new(format!("{done}{name}")).help(help.map(Into::into))
        })
        .collect()
}

/// Tab candidates for a list of bench names, `--benches` and `figures --bench`.
fn complete_bench_list(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    complete_list(
        current,
        benches::names().into_iter().map(|n| (n.to_string(), None)),
    )
}

/// Tab candidates for `--config`: the config files in the directory typed so far.
fn complete_config(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    config_files(&current.to_string_lossy())
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

/// Tab candidates for `--pin-cpus`: the profiles this host's config files declare and the names
/// every host has, each element of a comma list. A config that does not load offers the names
/// alone, since a completion has nowhere to say why.
fn complete_placements(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let declared = match crate::config::load(None) {
        Ok((config, _)) => config.profiles.into_keys().collect(),
        Err(_) => Vec::new(),
    };
    let names = declared
        .into_iter()
        .map(|n| (n, Some("a declared profile")))
        .chain([
            (
                "nearest".to_string(),
                Some("the first of smt, ccx, and x-ccx declared"),
            ),
            (
                "all".to_string(),
                Some("every declared profile, then unpinned"),
            ),
            ("unpinned".to_string(), Some("no pin")),
        ]);
    complete_list(current, names)
}

/// Fixed candidates, each with its help, that start as `current` does.
fn complete_words(current: &std::ffi::OsStr, words: &[(&str, &str)]) -> Vec<CompletionCandidate> {
    let typed = current.to_string_lossy();
    words
        .iter()
        .filter(|(word, _)| word.starts_with(typed.as_ref()))
        .map(|(word, help)| CompletionCandidate::new(*word).help(Some(help.to_string().into())))
        .collect()
}

/// Tab candidates for `figures --show`.
fn complete_show(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    complete_words(
        current,
        &[
            ("all", "every run"),
            ("trim", "every run, the ones the trim drops in grey"),
            ("extremes", "the fastest and the slowest run"),
        ],
    )
}

/// Tab candidates for `freq pin`'s target: the config's [freq] values by name.
fn complete_pin_target(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    complete_words(current, PIN_WORDS)
}

/// Tab candidates for `--pin-freq=`: the config's [freq] values by name, and `no`.
fn complete_pin_freq(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let mut words = PIN_WORDS.to_vec();
    words.push(("no", "pin nothing, over a config file's pin_freq"));
    complete_words(current, &words)
}

/// The [freq] values a pin may name in place of a frequency.
const PIN_WORDS: &[(&str, &str)] = &[
    ("pin_mhz", "the config's pin_mhz, else the base clock"),
    ("min_mhz", "the config's min_mhz"),
    ("max_mhz", "the config's max_mhz"),
];

/// Tab candidates for `analyze --compare`: the keys that are not tags.
fn complete_compare(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    complete_words(
        current,
        &[
            ("bench", "compare benches"),
            ("host", "compare hosts"),
            ("placement", "compare placements"),
            ("binary", "compare builds"),
            ("file", "compare record files"),
        ],
    )
}

/// The config files `typed` could become: the `.md` and `.toml` files in the directory it
/// names so far whose names start as it does.
fn config_files(typed: &str) -> Vec<String> {
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
        // A dotfile is offered only once a dot is typed, as a shell's own completion does.
        .filter(|name| name.starts_with(prefix) && is_config_arg(name))
        .filter(|name| !name.starts_with('.') || prefix.starts_with('.'))
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

impl RunArgs {
    /// Move a config file among the positionals into `--config`, so `iiac-perf run queue.md` is
    /// `iiac-perf run --config queue.md`, the common line without the flag.
    pub fn take_config_arg(&mut self) -> Result<(), String> {
        let (files, names): (Vec<String>, Vec<String>) =
            self.benches.drain(..).partition(|w| is_config_arg(w));
        self.benches = names;
        let mut files = files.into_iter();
        let Some(file) = files.next() else {
            return Ok(());
        };
        if let Some(second) = files.next() {
            return Err(format!(
                "'{file}' and '{second}' both name the run's config: a run has one"
            ));
        }
        if let Some(flag) = &self.config {
            return Err(format!(
                "'{file}' and --config {} both name the run's config: keep one",
                flag.display()
            ));
        }
        self.config = Some(file.into());
        Ok(())
    }
}

/// Refuse a bench list holding a subcommand's name, which in `--benches` or the config `benches`
/// would otherwise reach bench resolution and read as an unknown bench.
pub fn check_bench_words(words: &[String]) -> Result<(), String> {
    use clap::CommandFactory;
    let cli = Cli::command();
    match words
        .iter()
        .find(|w| cli.find_subcommand(w.as_str()).is_some())
    {
        Some(word) => Err(format!(
            "benches: '{word}' is a command, not a bench: run it as '{BIN_NAME} {word}'"
        )),
        None => Ok(()),
    }
}

/// The old command words, each with the line that does its work now, for the answer an old
/// line gets.
const RETIRED: &[(&str, &str)] = &[
    ("qualify-environment", "qualify"),
    ("read-freq", "freq read"),
    ("pin-freq", "freq pin"),
    ("restore-freq", "freq restore"),
    ("suggest-freq", "freq suggest"),
    ("setup-freq", "setup freq"),
    ("setup-uninstall", "setup uninstall"),
    ("init-config", "config init"),
    ("update-config", "config update"),
];

/// The answer to a first word that is no subcommand: the line to type, the old word's new form,
/// or `run` before a bench, and `-h` or `run` for anything else, which list the commands and the
/// benches.
pub fn unknown_word(words: &[String]) -> String {
    let word = words.first().map_or("", String::as_str);
    let rest = words.iter().skip(1).cloned().collect::<Vec<_>>().join(" ");
    let line = |command: &str| {
        format!("{BIN_NAME} {command} {rest}")
            .trim_end()
            .to_string()
    };
    if let Some((_, new)) = RETIRED.iter().find(|(old, _)| *old == word) {
        format!("'{word}' is '{new}' now: {}", line(new))
    } else if word == "all" || benches::resolve(&[word.to_string()]).is_ok() {
        format!("benches run under 'run': {}", line(&format!("run {word}")))
    } else {
        format!("'{word}' is not a command, try '-h' or 'run'")
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

/// The run flags on the line as config keys, for `config init` to set in the file it writes and
/// for the record's run config: a flag's key and its value as the config spells it. `verbose` is
/// the top level's, since every subcommand takes it.
pub fn line_values(m: &MeasureArgs, verbose: Option<bool>) -> Result<toml::Table, String> {
    use toml::Value;
    let count = |n: u64, flag: &str| match i64::try_from(n) {
        Ok(n) => Ok(Value::Integer(n)),
        Err(_) => Err(format!("{flag}: {n} is too large for a config")),
    };
    let text = |s: &str| Value::String(s.to_string());
    let mut t = toml::Table::new();
    if !m.benches_flag.is_empty() {
        let names = m.benches_flag.iter().map(|b| text(b)).collect();
        t.insert("benches".to_string(), Value::Array(names));
    }
    // Seconds reach here parsed, so a `-d 250ms` is written `0.25`.
    for (key, value) in [
        ("duration", m.duration),
        ("total_duration", m.total_duration),
        ("settle_time", m.settle_time),
        ("warm_cap", m.warm_cap),
    ] {
        if let Some(seconds) = value {
            t.insert(key.to_string(), Value::Float(seconds));
        }
    }
    for (key, flag, value) in [
        ("samples", "--samples", m.samples),
        ("runs", "--runs", m.runs),
        ("blocks", "--blocks", m.blocks),
        ("decimals", "--decimals", m.decimals.map(u64::from)),
    ] {
        if let Some(n) = value {
            t.insert(key.to_string(), count(n, flag)?);
        }
    }
    // A count is written as the integer it always was, and a span as its text.
    if let Some(span) = m.inner {
        let value = if span.is_fixed() {
            count(span.lo(), "--inner")?
        } else {
            text(&span.to_string())
        };
        t.insert("inner".to_string(), value);
    }
    for (key, value) in [
        ("pin_cpus", m.pin_cpus.as_deref()),
        ("run_sleep", m.run_sleep.as_deref()),
        ("trim_runs", m.trim_runs.as_deref()),
        ("block_sleep", m.block_sleep.as_deref()),
        ("block_warmup", m.block_warmup.as_deref()),
        ("band_labels", m.band_labels.map(bands::BandLabels::as_str)),
    ] {
        if let Some(s) = value {
            t.insert(key.to_string(), text(s));
        }
    }
    for (key, value) in [
        ("record_dir", &m.record_dir),
        ("record_file", &m.record_file),
    ] {
        if let Some(path) = value {
            t.insert(key.to_string(), text(&path.to_string_lossy()));
        }
    }
    // The two `no-` flags are the key's opposite.
    for (key, value) in [
        ("verbose", verbose),
        ("ticks", m.ticks),
        ("env_probe", m.no_env_probe.map(|no| !no)),
        ("inhibit", m.no_inhibit.map(|no| !no)),
    ] {
        if let Some(on) = value {
            t.insert(key.to_string(), Value::Boolean(on));
        }
    }
    // A bare --pin-freq is the word it stands for, and a frequency is a number.
    let pin_freq = match &m.pin_freq {
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
    if !m.tag.is_empty() || m.record_label.is_some() {
        let mut tags = toml::Table::new();
        if let Some(label) = &m.record_label {
            tags.insert(record::LABEL_TAG.to_string(), text(label));
        }
        for tag in &m.tag {
            match tag.split_once('=') {
                Some((k, v)) if !k.is_empty() => tags.insert(k.to_string(), text(v)),
                _ => return Err(format!("--tag '{tag}' is not key=value")),
            };
        }
        t.insert("tags".to_string(), Value::Table(tags));
    }
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `run`'s arguments from a line, `run` itself included.
    fn run_args(args: &[&str]) -> Result<(RunArgs, Option<bool>), clap::Error> {
        let cli = Cli::try_parse_from(args)?;
        match cli.command {
            Some(Command::Run(run)) => Ok((run, cli.verbose)),
            _ => panic!("not a run line: {args:?}"),
        }
    }

    #[test]
    fn the_graph_is_on_unless_no_graph() {
        let (run, _) =
            run_args(&["iiac-perf", "run", "min-now", "--inner", "5-12"]).expect("parses");
        assert!(!run.no_graph);
        let (run, _) = run_args(&["iiac-perf", "run", "min-now", "--no-graph"]).expect("parses");
        assert!(run.no_graph);
    }

    #[test]
    fn a_bench_list_refuses_command_words_but_all() {
        let words = |w: &[&str]| w.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(check_bench_words(&words(&["all"])).is_ok());
        assert!(check_bench_words(&words(&["min-now", "zcr"])).is_ok());
        let err = check_bench_words(&words(&["min-now", "setup"])).unwrap_err();
        assert!(err.contains("'setup'"), "unexpected error: {err}");
        // An alias is a command too.
        assert!(check_bench_words(&words(&["lr"])).is_err());
    }

    #[test]
    fn benches_flag_splits_on_commas_and_sits_beside_config_inits_path() {
        let (run, _) = run_args(&[
            "iiac-perf",
            "run",
            "--benches",
            "min-now,std-now",
            "--benches",
            "zcr",
        ])
        .expect("parses");
        assert_eq!(run.measure.benches_flag, ["min-now", "std-now", "zcr"]);
        assert!(run.benches.is_empty());
        // The flag beside a positional parses, since `config init PATH --benches a` needs both,
        // and `main` refuses it on a bench line.
        let cli = Cli::try_parse_from([
            "iiac-perf",
            "config",
            "init",
            "q.md",
            "--benches",
            "min-now",
        ])
        .expect("parses");
        let Some(Command::Config(ConfigCommand::Init(init))) = cli.command else {
            panic!("not config init");
        };
        assert_eq!(init.path.as_deref(), Some(std::path::Path::new("q.md")));
        let line = line_values(&init.measure, cli.verbose).expect("values");
        assert_eq!(line["benches"].as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn the_record_flags_become_their_keys_and_the_label_a_tag() {
        let (run, verbose) = run_args(&[
            "iiac-perf",
            "run",
            "--record-dir",
            "runs",
            "--record-label",
            "pins",
            "--tag",
            "host=a",
        ])
        .expect("parses");
        let line = line_values(&run.measure, verbose).expect("values");
        assert_eq!(line["record_dir"].as_str(), Some("runs"));
        assert_eq!(line["tags"]["label"].as_str(), Some("pins"));
        assert_eq!(line["tags"]["host"].as_str(), Some("a"));
        assert!(
            run_args(&[
                "iiac-perf",
                "run",
                "--record-dir",
                "a",
                "--record-file",
                "b"
            ])
            .is_err()
        );
    }

    #[test]
    fn an_on_off_flag_is_bare_or_takes_yes_or_no() {
        let (run, verbose) = run_args(&["iiac-perf", "run", "min-now"]).expect("parses");
        let m = &run.measure;
        assert_eq!(
            (verbose, m.ticks, m.no_env_probe, m.no_inhibit),
            (None, None, None, None)
        );
        let (run, verbose) =
            run_args(&["iiac-perf", "run", "-v", "-t", "--no-inhibit", "min-now"]).expect("parses");
        let m = &run.measure;
        assert_eq!(
            (verbose, m.ticks, m.no_inhibit),
            (Some(true), Some(true), Some(true))
        );
        // The bare flag takes no following word, so the bench name stays a bench name.
        assert_eq!(run.benches, ["min-now"]);
        // `-v` is the top level's, so it reads before the subcommand too.
        let (_, verbose) = run_args(&["iiac-perf", "-v", "run", "min-now"]).expect("parses");
        assert_eq!(verbose, Some(true));
        let (run, verbose) =
            run_args(&["iiac-perf", "run", "--verbose=no", "--no-env-probe=no"]).expect("parses");
        assert_eq!(
            (verbose, run.measure.no_env_probe),
            (Some(false), Some(false))
        );
        assert!(run_args(&["iiac-perf", "run", "--ticks=maybe"]).is_err());
        // A tag needs a record, but the record may come from a file, so the line alone parses.
        assert!(run_args(&["iiac-perf", "run", "--tag", "k=v", "min-now"]).is_ok());
    }

    #[test]
    fn a_positional_ending_in_a_carriers_extension_is_the_config() {
        let parse = |args: &[&str]| {
            let (mut run, _) = run_args(args).expect("parses");
            run.take_config_arg().map(|()| run)
        };
        let run = parse(&["iiac-perf", "run", "min-now", "configs/queue.md", "zcr"]).unwrap();
        assert_eq!(run.benches, ["min-now", "zcr"]);
        assert_eq!(
            run.config.as_deref(),
            Some(std::path::Path::new("configs/queue.md"))
        );
        let run = parse(&["iiac-perf", "run", "queue.toml"]).unwrap();
        assert!(run.benches.is_empty());
        // A bare name stays a bench, and a pattern with a dot in it is no file.
        let run = parse(&["iiac-perf", "run", "queue", "zcr-.psc"]).unwrap();
        assert_eq!((run.benches.len(), run.config.is_none()), (2, true));
        assert!(parse(&["iiac-perf", "run", "a.md", "b.toml"]).is_err());
        assert!(parse(&["iiac-perf", "run", "a.md", "--config", "b"]).is_err());
    }

    #[test]
    fn the_record_commands_own_their_flags() {
        let cli = Cli::try_parse_from(["iiac-perf", "lr", "records", "--overhead", "6.5"])
            .expect("parses");
        match cli.command {
            Some(Command::LinearRegression(a)) => {
                assert_eq!(a.overhead, Some(6.5));
                assert_eq!(a.records.trim_runs, series::Trim::DEFAULT);
            }
            _ => panic!("not linear-regression"),
        }
        let cli = Cli::try_parse_from([
            "iiac-perf",
            "figures",
            "r.jsonl",
            "--show",
            "3,1",
            "--x-axis",
            "block",
            "--trim-runs",
            "20-80",
        ])
        .expect("parses");
        match cli.command {
            Some(Command::Figures(a)) => {
                assert_eq!(a.show, figures::Show::Runs(vec![3, 1]));
                assert_eq!(a.x_axis, figures::XAxis::Block);
                assert_eq!(a.records.trim_runs.to_string(), "20-80");
                assert_eq!(a.out, std::path::Path::new("block-means.png"));
            }
            _ => panic!("not figures"),
        }
        // Each refuses what is another's, and the records are required.
        assert!(Cli::try_parse_from(["iiac-perf", "analyze", "r", "--overhead", "1"]).is_err());
        assert!(Cli::try_parse_from(["iiac-perf", "figures", "r", "--by", "cpus"]).is_err());
        assert!(Cli::try_parse_from(["iiac-perf", "lr", "r", "--overhead", "-1"]).is_err());
        assert!(Cli::try_parse_from(["iiac-perf", "analyze"]).is_err());
    }

    #[test]
    fn qualify_takes_what_it_passes_its_children_and_its_old_name() {
        let qualify = |args: &[&str]| match Cli::try_parse_from(args).map(|c| c.command) {
            Ok(Some(Command::Qualify(q))) => Ok(q),
            Ok(_) => panic!("not qualify: {args:?}"),
            Err(e) => Err(e),
        };
        let q = qualify(&["iiac-perf", "qualify"]).expect("parses");
        assert_eq!((q.runs, q.duration, q.run_sleep), (10, 1.0, (0.0, 0.0)));
        assert!(!q.print_only && q.pin_cpus.is_none() && q.settle_time.is_none());
        let q = qualify(&[
            "iiac-perf",
            "qualify-environment",
            "-d",
            "250ms",
            "--run-sleep",
            "1-2ms",
            "--pin-cpus",
            "smt",
        ])
        .expect("parses");
        assert_eq!((q.duration, q.run_sleep), (0.25, (0.001, 0.002)));
        assert_eq!(q.pin_cpus.as_deref(), Some("smt"));
        // A run parameter it does not pass on is refused, not ignored.
        assert!(qualify(&["iiac-perf", "qualify", "--blocks", "3"]).is_err());
    }

    #[test]
    fn a_word_that_is_no_subcommand_is_answered_with_the_line_to_type() {
        let words = |w: &[&str]| w.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let unknown = |args: &[&str]| match Cli::try_parse_from(args).map(|c| c.command) {
            Ok(Some(Command::Unknown(w))) => unknown_word(&w),
            _ => panic!("not an unknown word: {args:?}"),
        };
        assert_eq!(
            unknown(&["iiac-perf", "pin-freq", "3800"]),
            format!("'pin-freq' is 'freq pin' now: {BIN_NAME} freq pin 3800")
        );
        assert_eq!(
            unknown(&["iiac-perf", "min-now", "-d", "1"]),
            format!("benches run under 'run': {BIN_NAME} run min-now -d 1")
        );
        assert!(unknown_word(&words(&["all"])).contains("run all"));
        let nosuch = unknown_word(&words(&["nosuch"]));
        assert_eq!(nosuch, "'nosuch' is not a command, try '-h' or 'run'");
        assert!(
            Cli::try_parse_from(["iiac-perf"])
                .expect("parses")
                .command
                .is_none()
        );
    }

    #[test]
    fn the_groups_nest_and_own_their_flags() {
        let command = |args: &[&str]| Cli::try_parse_from(args).map(|c| c.command);
        let Ok(Some(Command::Freq(FreqCommand::Pin { target, .. }))) =
            command(&["iiac-perf", "freq", "pin", "min_mhz"])
        else {
            panic!("not freq pin");
        };
        assert_eq!(target.as_deref(), Some("min_mhz"));
        let Ok(Some(Command::Freq(FreqCommand::Suggest(s)))) =
            command(&["iiac-perf", "freq", "suggest", "min-now", "-d", "2"])
        else {
            panic!("not freq suggest");
        };
        assert_eq!(
            (s.bench.as_str(), s.measure.duration),
            ("min-now", Some(2.0))
        );
        // --apply reads on setup and on each of its parts.
        for args in [
            &["iiac-perf", "setup", "--apply"][..],
            &["iiac-perf", "setup", "freq", "--apply"][..],
            &["iiac-perf", "setup", "--apply", "uninstall"][..],
        ] {
            let Ok(Some(Command::Setup(setup))) = command(args) else {
                panic!("not setup: {args:?}");
            };
            assert!(setup.apply, "{args:?}");
        }
        // config init's starts are one at most, --series needs a record, and the file's fate
        // is one word.
        let init = |rest: &[&str]| {
            let args: Vec<&str> = ["iiac-perf", "config", "init"]
                .iter()
                .chain(rest)
                .copied()
                .collect();
            command(&args)
        };
        assert!(init(&["--from", "a.md", "--config", "b"]).is_err());
        assert!(init(&["--series", "x"]).is_err());
        assert!(init(&["--from-record", "r.jsonl", "--series", "x"]).is_ok());
        assert!(init(&["q.md", "--backup", "--overwrite"]).is_err());
        // config update rewrites one named file and takes no start.
        assert!(command(&["iiac-perf", "config", "update"]).is_err());
        assert!(command(&["iiac-perf", "config", "update", "q.md", "--from", "a"]).is_err());
        // Each group refuses the others' flags.
        assert!(command(&["iiac-perf", "freq", "read", "--apply"]).is_err());
        assert!(command(&["iiac-perf", "setup", "--as-config"]).is_err());
    }

    #[test]
    fn the_completers_offer_each_flags_own_values() {
        let values = |f: fn(&std::ffi::OsStr) -> Vec<CompletionCandidate>, typed: &str| {
            f(std::ffi::OsStr::new(typed))
                .iter()
                .map(|c| c.get_value().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        };
        // A list completes its last element after the ones typed.
        assert_eq!(
            values(complete_bench_list, "min-now,std"),
            ["min-now,std-now"]
        );
        let placements = values(complete_placements, "smt,un");
        assert_eq!(placements, ["smt,unpinned"]);
        assert!(values(complete_placements, "").contains(&"nearest".to_string()));
        assert_eq!(values(complete_show, "ex"), ["extremes"]);
        assert_eq!(values(complete_pin_target, "m"), ["min_mhz", "max_mhz"]);
        assert!(values(complete_pin_freq, "").contains(&"no".to_string()));
        assert_eq!(values(complete_compare, "b"), ["bench", "binary"]);
        // --config offers the config files, a dotfile only once a dot is typed.
        let configs = values(complete_config, "");
        assert!(
            configs.contains(&"iiac-perf.example.md".to_string()),
            "{configs:?}"
        );
        assert!(!configs.iter().any(|c| c.starts_with('.')), "{configs:?}");
        assert_eq!(values(complete_config, ".vc"), [".vc-config.md"]);
    }

    #[test]
    fn complete_bench_offers_benches_all_and_configs_by_prefix() {
        let values = |typed: &str| -> Vec<String> {
            complete_bench(std::ffi::OsStr::new(typed))
                .iter()
                .map(|c| c.get_value().to_string_lossy().into_owned())
                .collect()
        };
        assert_eq!(values("cb-chan"), ["cb-chan-1t", "cb-chan-2t"]);
        assert_eq!(values("al"), ["all"]);
        assert_eq!(values("").len(), benches::names().len() + 1);
        // A config file is offered once something is typed, a path's directory kept.
        assert_eq!(values("iiac-perf.ex"), ["iiac-perf.example.md"]);
        assert_eq!(values("docs/conf"), ["docs/config.md"]);
    }
}
