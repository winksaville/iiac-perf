//! Layered configuration for defaults: built-in < XDG config file <
//! project-local file < CLI flags.
//!
//! - **XDG file**: `$XDG_CONFIG_HOME/iiac-perf/config.md` (or
//!   `.toml`), falling back to `$HOME/.config/iiac-perf/` when
//!   `XDG_CONFIG_HOME` is unset. The per-user home for defaults
//!   and pin profiles.
//! - **Project-local file**: the nearest `iiac-perf.md` (or
//!   `.toml`), the current directory first and then each parent.
//!   The search stops at the first found, so a file high in the
//!   tree is a fallback, never a layer under every directory below
//!   it. Overrides the XDG file field-by-field, and profiles merge
//!   by key.
//! - **CLI**: always wins, resolved in `main` after [`load`].
//! - **`--config NAME`**: a named file, searched for in the current directory, its parents, and
//!   the XDG directory. The run keys then come from it and the built-ins alone, and the two
//!   files above give only `[freq]` and `[profiles]`.
//!
//! Every run parameter has a key. The command words' flags have none: they say what to do, not
//! how a run is shaped.
//!
//! Two carriers, one per directory. A `.md` config is a markdown
//! document whose `toml` fences, concatenated in document order,
//! are the config ([`crate::md_fence`]), so the prose between them
//! documents the file to its reader. `.md` is the recommended
//! form. Plain `.toml` stays accepted. A directory holding both is
//! a hard error naming both paths, because the one the user edits
//! could otherwise be the one the loader ignores.
//!
//! A malformed or unreadable-but-present file is a hard error, so a
//! typo surfaces rather than silently reverting to built-ins.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::bands::BandLabels;
use crate::md_fence::md_to_toml;

/// Project-local override filenames (markdown carrier, TOML
/// carrier), looked up in the current directory and then its parents.
const LOCAL_MD: &str = "iiac-perf.md";
/// The project-local TOML carrier beside [`LOCAL_MD`].
const LOCAL_TOML: &str = "iiac-perf.toml";

/// Inclusive upper bound on `decimals`, mirroring the
/// `--decimals` CLI `value_parser` range.
const DECIMALS_MAX: u8 = 3;

/// Fewest blocks a config may ask for. One block is a plain run,
/// and every stat that needs more withholds itself, so the range
/// starts at one. `main`'s `--blocks` carries the same bounds
/// inline, the way `--decimals` carries [`DECIMALS_MAX`]'s.
const BLOCKS_MIN: u64 = 1;
/// Most blocks a config may ask for.
const BLOCKS_MAX: u64 = 1000;

/// Fewest runs per bench a config may ask for: one run is one process, with no spread across
/// runs. `main`'s `--runs` carries the same bounds inline.
pub const RUNS_MIN: u64 = 1;
/// Most runs per bench a config may ask for.
pub const RUNS_MAX: u64 = 1000;

/// The config file's shape as deserialized, before validation.
/// Scalars are `Option` so an absent key stays absent, letting a
/// lower layer or built-in default show through. Unknown keys are
/// rejected to catch typos.
/// A seconds key as TOML spells it: a bare number, seconds, or a
/// string with a unit, `"250ms"`, read by
/// [`timespec::parse_seconds`](crate::timespec::parse_seconds).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Seconds {
    Num(f64),
    Spec(String),
}

impl Seconds {
    /// The seconds, `key` naming the field in an error.
    fn seconds(&self, key: &str) -> Result<f64, String> {
        let v = match self {
            Seconds::Num(v) => *v,
            Seconds::Spec(s) => {
                crate::timespec::parse_seconds(s).map_err(|e| format!("{key}: {e}"))?
            }
        };
        if v < 0.0 {
            return Err(format!("{key}: {v} is negative"));
        }
        Ok(v)
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct TomlConfig {
    /// The benches a run with no bench names on the line runs: a list of names, or one name.
    benches: Option<RawBenches>,
    /// Default `--duration`: seconds as a number, or a duration with unit as a string.
    duration: Option<Seconds>,
    /// Default `--band-labels` style, as its lowercase name.
    band_labels: Option<String>,
    /// Default `--decimals` count.
    decimals: Option<u8>,
    /// Default `--settle-time`: seconds as a number, or a duration with unit as a string.
    settle_time: Option<Seconds>,
    /// Default `--warm-cap`: seconds as a number, or a duration with unit as a string.
    warm_cap: Option<Seconds>,
    /// Default `--blocks` count.
    blocks: Option<u64>,
    /// Default `--runs` count, runs per bench.
    runs: Option<u64>,
    /// Default `--run-sleep` span spec (e.g. `"1-3s"`).
    run_sleep: Option<String>,
    /// Default `--trim-runs`, the edges of the band of run means kept (e.g. `"10-50"`).
    trim_runs: Option<String>,
    /// Default `--block-sleep` span spec (e.g. `"1-10ms"`).
    block_sleep: Option<String>,
    /// Default `--block-warmup` duration spec (e.g. `"2ms"`).
    block_warmup: Option<String>,
    /// Default `--pin-freq`: a frequency in MHz, `"pin_mhz"`, `"min_mhz"`, `"max_mhz"`, or `"no"`.
    pin_freq: Option<RawPinFreq>,
    /// Default `--total-duration`: seconds as a number, or a duration with unit as a string. A
    /// file sets this or `duration`, never both.
    total_duration: Option<Seconds>,
    /// Default `--samples` count.
    samples: Option<u64>,
    /// Default `--inner` count.
    inner: Option<u64>,
    /// Default `--pin-cpus`: a CPU spec or a `[profiles]` name.
    pin_cpus: Option<String>,
    /// The retired `record` key, kept only to refuse it by name, since a path's trailing `/`
    /// picked its mode.
    record: Option<toml::Value>,
    /// Default `--record-dir`. A relative one resolves against the current directory. A file
    /// sets this or `record_file`, never both.
    record_dir: Option<PathBuf>,
    /// Default `--record-file`. A relative one resolves against the current directory.
    record_file: Option<PathBuf>,
    /// `false` is `--no-env-probe`.
    env_probe: Option<bool>,
    /// `false` is `--no-inhibit`.
    inhibit: Option<bool>,
    /// `true` is `--ticks`.
    ticks: Option<bool>,
    /// `true` is `--verbose`.
    verbose: Option<bool>,
    /// The record's tags, each a `--tag KEY=VALUE`: key -> value.
    #[serde(default)]
    tags: BTreeMap<String, String>,
    /// Named pin profiles: name -> `--pin-cpus` CPU spec.
    #[serde(default)]
    profiles: BTreeMap<String, String>,
    /// The declared `[freq]` steady state and pin target.
    freq: Option<FreqConfig>,
    /// Which file set each scalar key, the last overlay winning. Filled by [`overlay`], never
    /// read from a file.
    #[serde(skip)]
    sources: BTreeMap<&'static str, PathBuf>,
}

/// `benches` as a config file spells it: a list of bench names, or one name such as `"all"`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawBenches {
    /// One bench name, a prefix, or `"all"`.
    One(String),
    /// Several, each a name, a prefix, or `"all"`.
    List(Vec<String>),
}

/// `pin_freq` as a config file spells it: a frequency or a word.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawPinFreq {
    /// A frequency in MHz.
    Mhz(u64),
    /// `"pin_mhz"`, `"min_mhz"`, `"max_mhz"`, or `"no"`.
    Word(String),
}

/// A run's clock pin, from `pin_freq` in a config or `--pin-freq` on the line. A run setting, not
/// part of the `[freq]` declaration: the host's `[freq]` table declares its values, and a run
/// names one of them or gives a frequency, so a benchmark directory's config pins every run, moves
/// between hosts, and every run still restores to the host's steady state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinFreq {
    /// No pin: `--pin-freq=no`, one run. In a file `"no"` is the same as no key.
    Off,
    /// Pin at the `[freq]` table's `pin_mhz`, else the base clock: `pin_mhz`, and the bare flag.
    PinMhz,
    /// Pin at the `[freq]` table's `min_mhz`: `min_mhz`.
    MinMhz,
    /// Pin at the `[freq]` table's `max_mhz`: `max_mhz`. It holds only when the declared value
    /// fits under the ceiling with boost off, which a pin turns off.
    MaxMhz,
    /// Pin at this frequency (MHz).
    Mhz(u64),
}

impl PinFreq {
    /// A `pin_freq` word or number as text, shared by the flag and the file: `pin_mhz`,
    /// `min_mhz`, `max_mhz`, `no`, or a frequency in MHz. `None` is `no`, which the caller turns into a
    /// one-run cancel on the line and into no opinion in a file.
    fn parse_word(value: &str) -> Result<Option<PinFreq>, String> {
        match value {
            "pin_mhz" => Ok(Some(PinFreq::PinMhz)),
            "min_mhz" => Ok(Some(PinFreq::MinMhz)),
            "max_mhz" => Ok(Some(PinFreq::MaxMhz)),
            "no" => Ok(None),
            v => match v.parse::<u64>() {
                Ok(0) => Err("0 is not a frequency".to_string()),
                Ok(mhz) => Ok(Some(PinFreq::Mhz(mhz))),
                Err(_) => Err(format!(
                    "{v:?} is not a frequency in MHz, \"pin_mhz\", \"min_mhz\", \"max_mhz\", or \"no\""
                )),
            },
        }
    }

    /// The `--pin-freq` flag's value: bare for [`PinFreq::PinMhz`], `no` for no pin this run, or
    /// any other `pin_freq` value.
    pub fn from_flag(value: Option<&str>) -> Result<PinFreq, String> {
        match value {
            None => Ok(PinFreq::PinMhz),
            Some(v) => match PinFreq::parse_word(v)? {
                Some(p) => Ok(p),
                None => Ok(PinFreq::Off),
            },
        }
    }
}

/// A file's `pin_freq`: `Ok(None)` for `"no"`, which is the same as no key, so a lower file's pin
/// still applies.
fn pin_freq_from_raw(raw: &RawPinFreq) -> Result<Option<PinFreq>, String> {
    match raw {
        RawPinFreq::Mhz(0) => Err("pin_freq: 0 is not a frequency".to_string()),
        RawPinFreq::Mhz(mhz) => Ok(Some(PinFreq::Mhz(*mhz))),
        RawPinFreq::Word(w) => PinFreq::parse_word(w).map_err(|e| format!("pin_freq: {e}")),
    }
}

/// The `[freq]` table: the box's declared steady state, and optionally a pin target.
///
/// This is what `restore-freq` converges to, written once by the user rather than remembered
/// from before a pin: a remembered state ratchets when run 2 starts while run 1's pin is live,
/// and a transient state file is a demonstrated failure mode in this repo. It normally lives in
/// the XDG config (`~/.config/iiac-perf/config.md`), the steady state being the box's rather
/// than the project's.
///
/// - `governor` is the one required key. `epp` and `boost` are required exactly when the box
///   exposes those knobs, which only [`crate::freqctl`] can check, so it is checked at use time
///   rather than here.
/// - Frequencies are MHz, the unit humans quote clocks in. The kHz conversion happens at the
///   sysfs boundary.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreqConfig {
    /// Steady-state `scaling_governor` token, e.g. `powersave`.
    pub governor: String,
    /// Steady-state `energy_performance_preference` token. Required when the box has EPP,
    /// meaningless when it does not (rpi5-20cd exposes none).
    pub epp: Option<String>,
    /// Steady-state boost switch. Required when the box has a boost knob.
    pub boost: Option<bool>,
    /// Steady-state lower clamp (MHz). Optional to parse, required by every command that pins or
    /// restores, since a restore without it would fall to the hardware floor.
    pub min_mhz: Option<u64>,
    /// Steady-state upper clamp (MHz), required with `min_mhz`. Equal to it is allowed: a steady
    /// state holding the clock at one frequency.
    pub max_mhz: Option<u64>,
    /// Pin target (MHz) for `pin-freq` and `--pin-freq`. Absent means the discovered base
    /// clock.
    pub pin_mhz: Option<u64>,
}

impl FreqConfig {
    /// The declared table in one line, as the `Config:` list and the record print it.
    pub fn summary(&self) -> String {
        let mut parts = vec![self.governor.clone()];
        if let Some(epp) = &self.epp {
            parts.push(format!("EPP {epp}"));
        }
        if let Some(boost) = self.boost {
            parts.push(format!("boost {}", if boost { "on" } else { "off" }));
        }
        match (self.min_mhz, self.max_mhz) {
            (Some(min), Some(max)) => parts.push(format!("clamp {min}-{max} MHz")),
            (min, max) => {
                let limit = |v: Option<u64>| match v {
                    Some(mhz) => mhz.to_string(),
                    None => "undeclared".to_string(),
                };
                parts.push(format!("clamp {}-{} (incomplete)", limit(min), limit(max)));
            }
        }
        if let Some(pin) = self.pin_mhz {
            parts.push(format!("pin {pin} MHz"));
        }
        parts.join(", ")
    }
}

/// The merged, validated configuration handed to `main`.
///
/// Each scalar is `Option`: `None` means "no config opinion, use
/// the built-in default". Profiles are a flat name->spec map.
#[derive(Debug, Default, PartialEq)]
pub struct Config {
    /// The benches to run when neither bench names nor `--benches` are on the line, if
    /// configured. Never empty.
    pub benches: Option<Vec<String>>,
    /// Default `--duration` seconds, if configured.
    pub duration: Option<f64>,
    /// Default `--band-labels` style, if configured.
    pub band_labels: Option<BandLabels>,
    /// Default `--decimals` count, if configured.
    pub decimals: Option<u8>,
    /// Default `--settle-time` seconds, if configured.
    pub settle_time: Option<f64>,
    /// Default `--warm-cap` seconds, if configured.
    pub warm_cap: Option<f64>,
    /// Default `--blocks` count, if configured. A box declares it
    /// so every run replicates without the flag being typed.
    pub blocks: Option<u64>,
    /// Default `--block-sleep` span, `(min_s, max_s)` seconds, if
    /// configured.
    pub block_sleep: Option<(f64, f64)>,
    /// Default `--block-warmup` seconds, if configured.
    pub block_warmup: Option<f64>,
    /// Default `--runs`, runs per bench, if configured.
    pub runs: Option<u64>,
    /// Default `--run-sleep` span, `(min_s, max_s)` seconds, if configured.
    pub run_sleep: Option<(f64, f64)>,
    /// Default `--trim-runs`, if configured.
    pub trim_runs: Option<crate::series::Trim>,
    /// Default `--pin-freq`, if configured.
    pub pin_freq: Option<PinFreq>,
    /// Default `--total-duration` seconds, if configured. Never set with `duration`: the nearer
    /// file's choice of the two clears the other.
    pub total_duration: Option<f64>,
    /// Default `--samples` count, if configured.
    pub samples: Option<u64>,
    /// Default `--inner` count, if configured.
    pub inner: Option<u64>,
    /// Default `--pin-cpus` spec, if configured.
    pub pin_cpus: Option<String>,
    /// Default record target, `record_dir` or `record_file`, if configured. Its source is under
    /// `record`, whichever key set it.
    pub record: Option<crate::record::Target>,
    /// Seam probes on or off, if configured. `false` is `--no-env-probe`.
    pub env_probe: Option<bool>,
    /// The sleep inhibit on or off, if configured. `false` is `--no-inhibit`.
    pub inhibit: Option<bool>,
    /// Tprobe results in ticks, if configured. `true` is `--ticks`.
    pub ticks: Option<bool>,
    /// Verbose internals, if configured. `true` is `--verbose`.
    pub verbose: Option<bool>,
    /// The record's tags from the files, merged by key, the nearer file winning.
    pub tags: BTreeMap<String, String>,
    /// Named pin profiles: name -> `--pin-cpus` CPU spec.
    pub profiles: BTreeMap<String, String>,
    /// The declared `[freq]` steady state and pin target, if configured.
    pub freq: Option<FreqConfig>,
    /// The file each configured scalar came from, keyed by its config key, and the file the
    /// `[freq]` table came from under `freq`, so the report can name a value's source.
    pub sources: BTreeMap<&'static str, PathBuf>,
}

impl Config {
    /// Resolve a `--pin-cpus` spec against the configured profiles: a spec that names a profile
    /// expands to that profile's CPU spec, and a CPU list passes through for
    /// [`crate::pin::parse_cpus`] to parse. A spec that is neither, a bare word no `[profiles]`
    /// entry names, is refused here rather than left to fail later as a number, since a host
    /// that has declared no profiles meets this first and the number's error names no fix.
    pub fn resolve_pin<'a>(&'a self, spec: &'a str) -> Result<&'a str, String> {
        if let Some(cpus) = self.profiles.get(spec) {
            return Ok(cpus);
        }
        // A CPU list always opens with a digit, so anything else was meant as a profile name.
        // An empty spec passes through, since `--pin-cpus ""` clears a config file's pin.
        if spec.starts_with(|c: char| !c.is_ascii_digit()) {
            return Err(self.no_such_profile(spec));
        }
        Ok(spec)
    }

    /// The refusal printed when a `--pin-cpus` spec names no declared profile. Pinning by name is
    /// what lets one config serve every host, so the fix is always the host's own file: the
    /// message names that file, shows an entry's shape, and says what this host declares, since
    /// "none" and "a different set" are different mistakes.
    fn no_such_profile(&self, spec: &str) -> String {
        if self.profiles.is_empty() {
            return crate::wrap::wrap(
                &format!(
                    "{spec:?} is no declared profile and is not a CPU list, and this host declares \
                 none.\n\
                 A pin named rather than numbered is what lets one config serve every host, so \
                 the names belong to the host: add a [profiles] table beside [freq] in \
                 ~/.config/iiac-perf/config.toml, each entry a name and a CPU spec, as in \
                 `smt = \"3,9\"` for the two threads of one core or `ccx = \"3,2\"` for two \
                 cores sharing a last-level cache. `lscpu -e` shows which CPUs pair.\n\
                 `{bin} setup-freq` creates that file when it is missing, and docs/config.md \
                     explains the table.",
                    bin = crate::BIN_NAME
                ),
                crate::wrap::WIDTH,
            );
        }
        let declared = self
            .profiles
            .iter()
            .map(|(name, cpus)| format!("{name} = {cpus:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        crate::wrap::wrap(
            &format!(
                "{spec:?} is no declared profile and is not a CPU list.\n\
             This host declares: {declared}.\n\
             Name one of those, give a CPU spec instead, or add {spec:?} to [profiles] in \
             ~/.config/iiac-perf/config.toml. A host that cannot form the hop a name means, an \
             SMT pair where nothing pairs, leaves the name out rather than aliasing it to a \
                 different hop, so a config asking for it belongs to another host."
            ),
            crate::wrap::WIDTH,
        )
    }

    /// The file that set config key `key`, or `None` when no file did.
    pub fn source(&self, key: &str) -> Option<&Path> {
        self.sources.get(key).map(PathBuf::as_path)
    }
}

/// The XDG config directory: `$XDG_CONFIG_HOME/iiac-perf`, falling
/// back to `$HOME/.config/iiac-perf`. `None` if neither env var is
/// set (e.g. a stripped-down service environment).
fn xdg_dir() -> Option<PathBuf> {
    let base = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(x) if !x.is_empty() => PathBuf::from(x),
        _ => PathBuf::from(std::env::var_os("HOME")?).join(".config"),
    };
    Some(base.join("iiac-perf"))
}

/// Resolve which carrier a layer holds: the `.md` file, the
/// `.toml` file, or neither (`Ok(None)`). Both present is a hard
/// error naming both paths: a silent precedence would leave the
/// user editing the ignored file with no effect and no clue.
fn resolve_carrier(md: PathBuf, toml: PathBuf) -> Result<Option<PathBuf>, String> {
    match (md.exists(), toml.exists()) {
        (true, true) => Err(format!(
            "both {} and {} exist: keep one (.md is the recommended carrier)",
            md.display(),
            toml.display()
        )),
        (true, false) => Ok(Some(md)),
        (false, true) => Ok(Some(toml)),
        (false, false) => Ok(None),
    }
}

/// Load and merge the config files. Returns the merged [`Config`] plus the paths of the files
/// read, in load order, the later winning. Built-in-default `Config` when no file exists. It
/// errors on a present-but-unreadable or malformed file, on a directory holding both carriers,
/// and on a `named` file that is not found.
///
/// Without `named`, the XDG file then the project-local one, the nearer winning per field
/// (scalars replace, profiles and tags merge by key, `[freq]` replaces whole). The carrier rule
/// is per directory, so the layers may use different carriers.
///
/// With `named`, `--config NAME`, the run keys come from that file and the built-ins alone, so
/// one file means one run on every host. The XDG and local files still give `[freq]` and
/// `[profiles]`, the host's own facts, under whatever the named file sets of them.
pub fn load(named: Option<&Path>) -> Result<(Config, Vec<PathBuf>), String> {
    let cwd = std::env::current_dir().map_err(|e| format!("current directory: {e}"))?;
    load_from(xdg_dir().as_deref(), &cwd, true, named)
}

/// [`load`] with its places given: the XDG directory, the absolute directory the local and
/// named files' searches start in, and whether that is the process's own, so a local file in
/// it is named as it always was, `iiac-perf.md`, and only one found higher by its full path.
fn load_from(
    xdg: Option<&Path>,
    cwd: &Path,
    own: bool,
    named: Option<&Path>,
) -> Result<(Config, Vec<PathBuf>), String> {
    let mut raw = TomlConfig::default();
    let mut loaded = Vec::new();
    for path in layer_files(xdg, cwd, own)? {
        overlay(&mut raw, &path)?;
        loaded.push(path);
    }
    if let Some(name) = named {
        let path = find_named(name, cwd, xdg)?;
        raw = host_facts(raw);
        overlay(&mut raw, &path)?;
        loaded.push(path);
    }
    Ok((validate(raw)?, loaded))
}

/// The host's files a plain run layers, those that exist, in load order: the XDG file, then the
/// project-local one, the nearest up from `cwd`. `own` is [`load_from`]'s.
fn layer_files(xdg: Option<&Path>, cwd: &Path, own: bool) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    if let Some(dir) = xdg
        && let Some(path) = resolve_carrier(dir.join("config.md"), dir.join("config.toml"))?
    {
        files.push(path);
    }
    for dir in cwd.ancestors() {
        // The first found ends the search: no level above it is merged in.
        if let Some(path) = resolve_carrier(dir.join(LOCAL_MD), dir.join(LOCAL_TOML))? {
            let here = own && dir == cwd;
            files.push(match (here, path.file_name()) {
                (true, Some(name)) => PathBuf::from(name),
                _ => path,
            });
            break;
        }
    }
    Ok(files)
}

/// [`layer_files`] from where the process stands: what `init-config` starts from, so the file
/// it writes is the run a plain line would make here.
pub fn host_files() -> Result<Vec<PathBuf>, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("current directory: {e}"))?;
    layer_files(xdg_dir().as_deref(), &cwd, true)
}

/// What the host's files keep giving a run whose keys come from a named file: `[freq]` and
/// `[profiles]`, with their sources, and nothing else.
fn host_facts(raw: TomlConfig) -> TomlConfig {
    let mut sources = raw.sources;
    sources.retain(|key, _| *key == "freq");
    TomlConfig {
        profiles: raw.profiles,
        freq: raw.freq,
        sources,
        ..TomlConfig::default()
    }
}

/// [`find_named`] from where the process stands: what `init-config --config NAME` starts from.
pub fn find(name: &Path) -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("current directory: {e}"))?;
    find_named(name, &cwd, xdg_dir().as_deref())
}

/// Find the file `--config NAME` names. An absolute NAME is looked for where it says. A
/// relative one is tried in `cwd`, each of its parents up to the root, then the XDG directory,
/// the first found winning, so a tree of bench directories shares a parent's file by name and a
/// nearer file of that name overrides it. Not found is an error listing the places tried.
fn find_named(name: &Path, cwd: &Path, xdg: Option<&Path>) -> Result<PathBuf, String> {
    if name.as_os_str().is_empty() {
        return Err("the run's config: empty name".to_string());
    }
    if name.is_absolute() {
        return match named_in(name)? {
            Some(path) => Ok(path),
            None => Err(format!("the run's config {} not found", name.display())),
        };
    }
    let mut tried = Vec::new();
    for dir in cwd.ancestors().chain(xdg) {
        if let Some(path) = named_in(&dir.join(name))? {
            return Ok(path);
        }
        tried.push(dir.display().to_string());
    }
    // A name that carries a carrier's extension is looked for as given alone.
    let forms = if name.extension().is_some_and(|e| e == "md" || e == "toml") {
        String::new()
    } else {
        ", as given or with .md or .toml".to_string()
    };
    Err(format!(
        "the run's config {} not found{forms}, in:\n  {}",
        name.display(),
        tried.join("\n  ")
    ))
}

/// The file at `candidate`: itself when it is a file, else, when it ends in neither carrier's
/// extension, the one carrier present beside that name, both present an error.
fn named_in(candidate: &Path) -> Result<Option<PathBuf>, String> {
    if candidate.is_file() {
        return Ok(Some(candidate.to_path_buf()));
    }
    if candidate
        .extension()
        .is_some_and(|e| e == "md" || e == "toml")
    {
        return Ok(None);
    }
    let with = |ext: &str| {
        let mut name = candidate.as_os_str().to_os_string();
        name.push(ext);
        PathBuf::from(name)
    };
    resolve_carrier(with(".md"), with(".toml"))
}

/// Read one config file (either carrier) and overlay it onto
/// `base`: each present scalar replaces `base`'s, and profiles are
/// merged by key (the file's entries win). A `.md` path runs
/// through the fence filter first, and anything else parses as
/// plain TOML.
fn overlay(base: &mut TomlConfig, path: &Path) -> Result<(), String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let mut over = parse_raw(path, &text)?;
    // Each file's pin_freq is checked here, where its path is known, and a "no" becomes no key,
    // so it neither overrides a lower file's pin nor claims to be its source.
    if let Some(raw) = &over.pin_freq
        && pin_freq_from_raw(raw)
            .map_err(|e| format!("{}: {e}", path.display()))?
            .is_none()
    {
        over.pin_freq = None;
    }
    // The two durations are one choice, so a file makes it once, and the nearer file's choice
    // clears the other, which would otherwise still read as set.
    match (over.duration.is_some(), over.total_duration.is_some()) {
        (true, true) => {
            return Err(format!(
                "{}: duration and total_duration are both set: keep one",
                path.display()
            ));
        }
        (true, false) => {
            base.total_duration = None;
            base.sources.remove("total_duration");
        }
        (false, true) => {
            base.duration = None;
            base.sources.remove("duration");
        }
        (false, false) => {}
    }
    // The two record modes are one choice too, made and cleared the same way.
    match (over.record_dir.is_some(), over.record_file.is_some()) {
        (true, true) => {
            return Err(format!(
                "{}: record_dir and record_file are both set: keep one",
                path.display()
            ));
        }
        (true, false) => {
            base.record_file = None;
            base.sources.remove("record_file");
        }
        (false, true) => {
            base.record_dir = None;
            base.sources.remove("record_dir");
        }
        (false, false) => {}
    }
    // Each present scalar replaces base's and records this file as its source.
    macro_rules! take {
        ($($key:ident),*) => {$(
            if over.$key.is_some() {
                base.$key = over.$key;
                base.sources.insert(stringify!($key), path.to_path_buf());
            }
        )*};
    }
    take!(
        benches,
        duration,
        band_labels,
        decimals,
        settle_time,
        warm_cap,
        blocks,
        block_sleep,
        block_warmup,
        runs,
        run_sleep,
        trim_runs,
        pin_freq,
        total_duration,
        samples,
        inner,
        pin_cpus,
        record,
        record_dir,
        record_file,
        env_probe,
        inhibit,
        ticks,
        verbose
    );
    // Tags merge by key like profiles, and the last file to set any is the list's source.
    if !over.tags.is_empty() {
        base.sources.insert("tags", path.to_path_buf());
    }
    base.tags.extend(over.tags);
    // The whole [freq] table replaces, never field-merges: the steady state is one declaration
    // of one box's state, and half of one file's declaration on top of half of another's would
    // be a state nobody declared.
    if over.freq.is_some() {
        base.freq = over.freq;
        base.sources.insert("freq", path.to_path_buf());
    }
    base.profiles.extend(over.profiles);
    Ok(())
}

/// Parse one file's text as its carrier, named by `path`'s extension: a `.md` path runs through
/// the fence filter, anything else is plain TOML.
fn parse_raw(path: &Path, text: &str) -> Result<TomlConfig, String> {
    toml::from_str(&carried(path, text)?).map_err(|e| parse_error(path, &e))
}

/// A file's text as TOML, by its carrier, refusing a record by name: a JSON object opens with
/// `{`, which no config does, and a TOML error on one quotes a record line too long to read.
fn carried(path: &Path, text: &str) -> Result<String, String> {
    if text.trim_start().starts_with('{') {
        return Err(format!(
            "{} holds JSON, a record rather than a config: `{} init-config --from-record {}` \
             writes a config from a record",
            path.display(),
            crate::BIN_NAME,
            path.display()
        ));
    }
    if path.extension().is_some_and(|e| e == "md") {
        md_to_toml(text).map_err(|e| format!("{}: {e}", path.display()))
    } else {
        Ok(text.to_string())
    }
}

/// Longest quoted line a parse error keeps, so the key it names stays readable.
const ERROR_LINE_MAX: usize = 100;

/// A TOML parse error with each source line it quotes, `N | ...`, cut to [`ERROR_LINE_MAX`]
/// characters. The caret line under it is left whole, so a column past the cut still has its
/// caret.
fn parse_error(path: &Path, e: &toml::de::Error) -> String {
    let text = e.to_string();
    let quoted = |line: &str| {
        let head = line.trim_start();
        let digits = head.len() - head.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        digits > 0 && head[digits..].starts_with(" |")
    };
    let lines: Vec<String> = text
        .lines()
        .map(|line| match line.char_indices().nth(ERROR_LINE_MAX) {
            Some((at, _)) if quoted(line) => format!("{}...", &line[..at]),
            _ => line.to_string(),
        })
        .collect();
    format!("parsing {}: {}", path.display(), lines.join("\n"))
}

/// Parse and validate one config file's text on its own, no layering: what `setup-freq` checks an
/// existing file and its own additions with before writing.
pub fn parse_text(path: &Path, text: &str) -> Result<Config, String> {
    validate(parse_raw(path, text)?)
}

/// One config file's text as a bare TOML table, unchecked: what `init-config --from` reads a
/// file's own values out of, after [`parse_text`] has checked them.
pub fn parse_table(path: &Path, text: &str) -> Result<toml::Table, String> {
    toml::from_str(&carried(path, text)?).map_err(|e| parse_error(path, &e))
}

/// The XDG config file `setup-freq` writes: the carrier already present, else `config.md` in the XDG
/// directory. `None` when neither `XDG_CONFIG_HOME` nor `HOME` is set.
pub fn xdg_target() -> Result<Option<PathBuf>, String> {
    let Some(dir) = xdg_dir() else {
        return Ok(None);
    };
    let md = dir.join("config.md");
    match resolve_carrier(md.clone(), dir.join("config.toml"))? {
        Some(path) => Ok(Some(path)),
        None => Ok(Some(md)),
    }
}

/// Validate a merged [`TomlConfig`] into a [`Config`]: map the
/// `band_labels` name to the enum, range-check `decimals`, and
/// reject a negative `settle_time`.
fn validate(raw: TomlConfig) -> Result<Config, String> {
    let benches = match raw.benches {
        None => None,
        Some(RawBenches::One(name)) => Some(vec![name]),
        Some(RawBenches::List(names)) => Some(names),
    };
    if let Some(names) = &benches {
        if names.is_empty() {
            return Err("benches: an empty list names no bench".to_string());
        }
        if names.iter().any(|n| n.trim().is_empty()) {
            return Err("benches: a name is empty".to_string());
        }
    }
    let band_labels = match raw.band_labels {
        None => None,
        Some(s) => Some(match s.as_str() {
            "zpn" => BandLabels::Zpn,
            "frac" => BandLabels::Frac,
            "both" => BandLabels::Both,
            other => {
                return Err(format!(
                    "band_labels: {other:?} is not one of zpn, frac, both"
                ));
            }
        }),
    };
    if let Some(d) = raw.decimals
        && d > DECIMALS_MAX
    {
        return Err(format!(
            "decimals: {d} exceeds the maximum of {DECIMALS_MAX}"
        ));
    }
    let duration = raw
        .duration
        .as_ref()
        .map(|t| t.seconds("duration"))
        .transpose()?;
    let settle_time = raw
        .settle_time
        .as_ref()
        .map(|t| t.seconds("settle_time"))
        .transpose()?;
    let warm_cap = raw
        .warm_cap
        .as_ref()
        .map(|t| t.seconds("warm_cap"))
        .transpose()?;
    if let Some(n) = raw.blocks
        && !(BLOCKS_MIN..=BLOCKS_MAX).contains(&n)
    {
        return Err(format!(
            "blocks: {n} is outside {BLOCKS_MIN}..={BLOCKS_MAX}"
        ));
    }
    let block_sleep = match &raw.block_sleep {
        None => None,
        Some(s) => Some(crate::timespec::parse_span(s).map_err(|e| format!("block_sleep: {e}"))?),
    };
    let block_warmup = match &raw.block_warmup {
        None => None,
        Some(s) => {
            Some(crate::timespec::parse_scalar(s).map_err(|e| format!("block_warmup: {e}"))?)
        }
    };
    if let Some(n) = raw.runs
        && !(RUNS_MIN..=RUNS_MAX).contains(&n)
    {
        return Err(format!("runs: {n} is outside {RUNS_MIN}..={RUNS_MAX}"));
    }
    let run_sleep = match &raw.run_sleep {
        None => None,
        Some(s) => Some(crate::timespec::parse_span(s).map_err(|e| format!("run_sleep: {e}"))?),
    };
    let trim_runs = match &raw.trim_runs {
        None => None,
        Some(s) => Some(crate::series::Trim::parse(s).map_err(|e| format!("trim_runs: {e}"))?),
    };
    let pin_freq = match &raw.pin_freq {
        None => None,
        Some(r) => pin_freq_from_raw(r)?,
    };
    if let Some(f) = &raw.freq {
        validate_freq(f)?;
    }
    let total_duration = raw
        .total_duration
        .as_ref()
        .map(|t| t.seconds("total_duration"))
        .transpose()?;
    if duration.is_some() && total_duration.is_some() {
        return Err("duration and total_duration are both set: keep one".to_string());
    }
    if raw.pin_cpus.as_deref().is_some_and(|s| s.trim().is_empty()) {
        return Err("pin_cpus: empty".to_string());
    }
    if raw.record.is_some() {
        return Err(
            "record: a path's shape no longer picks the mode, so the key is gone: set \
             record_dir = \"DIR\" for a file per invocation, or record_file = \"PATH\" to \
             append every record to one file"
                .to_string(),
        );
    }
    let record = match (raw.record_dir, raw.record_file) {
        (Some(_), Some(_)) => {
            return Err("record_dir and record_file are both set: keep one".to_string());
        }
        (Some(dir), None) if dir.as_os_str().is_empty() => {
            return Err("record_dir: empty".to_string());
        }
        (None, Some(file)) if file.as_os_str().is_empty() => {
            return Err("record_file: empty".to_string());
        }
        (Some(dir), None) => Some(crate::record::Target::Dir(dir)),
        (None, Some(file)) => Some(crate::record::Target::File(file)),
        (None, None) => None,
    };
    // Whichever key set the target is the record's source.
    let mut sources = raw.sources;
    if let Some(path) = sources
        .get("record_dir")
        .or_else(|| sources.get("record_file"))
        .cloned()
    {
        sources.insert("record", path);
    }
    // A tag reaches the record as `KEY=VALUE`, split at the first `=`, so a key holds none.
    if let Some(key) = raw.tags.keys().find(|k| k.is_empty() || k.contains('=')) {
        return Err(format!("tags: {key:?} is empty or holds '='"));
    }
    Ok(Config {
        benches,
        duration,
        band_labels,
        decimals: raw.decimals,
        settle_time,
        warm_cap,
        blocks: raw.blocks,
        block_sleep,
        block_warmup,
        runs: raw.runs,
        run_sleep,
        trim_runs,
        pin_freq,
        total_duration,
        samples: raw.samples,
        inner: raw.inner,
        pin_cpus: raw.pin_cpus,
        record,
        env_probe: raw.env_probe,
        inhibit: raw.inhibit,
        ticks: raw.ticks,
        verbose: raw.verbose,
        tags: raw.tags,
        profiles: raw.profiles,
        freq: raw.freq,
        sources,
    })
}

/// Validate a `[freq]` table's box-independent facts: frequencies are nonzero and the clamps
/// are ordered. Whether the tokens and values fit the actual box is `freqctl`'s job at use
/// time.
fn validate_freq(f: &FreqConfig) -> Result<(), String> {
    if f.governor.is_empty() {
        return Err("freq.governor: empty".to_string());
    }
    for (name, mhz) in [
        ("min_mhz", f.min_mhz),
        ("max_mhz", f.max_mhz),
        ("pin_mhz", f.pin_mhz),
    ] {
        if mhz == Some(0) {
            return Err(format!("freq.{name}: 0 is not a frequency"));
        }
    }
    if let (Some(min), Some(max)) = (f.min_mhz, f.max_mhz)
        && min > max
    {
        return Err(format!("freq: min_mhz {min} exceeds max_mhz {max}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<Config, String> {
        validate(toml::from_str(text).map_err(|e| e.to_string())?)
    }

    #[test]
    fn pin_freq_takes_a_frequency_or_a_freq_key() {
        let pin = |text: &str| parse(text).map(|c| c.pin_freq);
        assert_eq!(pin("pin_freq = 3801\n"), Ok(Some(PinFreq::Mhz(3801))));
        assert_eq!(pin("pin_freq = \"pin_mhz\"\n"), Ok(Some(PinFreq::PinMhz)));
        assert_eq!(pin("pin_freq = \"min_mhz\"\n"), Ok(Some(PinFreq::MinMhz)));
        assert_eq!(pin("pin_freq = \"max_mhz\"\n"), Ok(Some(PinFreq::MaxMhz)));
        // "no" in a file is no opinion, the same as leaving the key out.
        assert_eq!(pin("pin_freq = \"no\"\n"), Ok(None));
        // A quoted frequency reads as the flag's text does.
        assert_eq!(pin("pin_freq = \"3801\"\n"), Ok(Some(PinFreq::Mhz(3801))));
        for bad in ["0", "\"\"", "\"pin\"", "true", "\"off\""] {
            assert!(
                parse(&format!("pin_freq = {bad}\n")).is_err(),
                "{bad} passed"
            );
        }
        assert_eq!(PinFreq::from_flag(None), Ok(PinFreq::PinMhz));
        assert_eq!(PinFreq::from_flag(Some("pin_mhz")), Ok(PinFreq::PinMhz));
        assert_eq!(PinFreq::from_flag(Some("min_mhz")), Ok(PinFreq::MinMhz));
        assert_eq!(PinFreq::from_flag(Some("max_mhz")), Ok(PinFreq::MaxMhz));
        assert_eq!(PinFreq::from_flag(Some("4701")), Ok(PinFreq::Mhz(4701)));
        // On the line "no" cancels a file's pin for this run.
        assert_eq!(PinFreq::from_flag(Some("no")), Ok(PinFreq::Off));
        for bad in ["", "0", "off", "false", "fast"] {
            assert!(PinFreq::from_flag(Some(bad)).is_err(), "{bad:?} passed");
        }
    }

    #[test]
    fn a_files_no_leaves_a_lower_files_pin_in_place() {
        let dir = scratch("pin-no");
        let xdg = dir.join("xdg.md");
        let local = dir.join("local.md");
        std::fs::write(&xdg, "```toml\npin_freq = 3801\n```\n").unwrap();
        std::fs::write(&local, "```toml\npin_freq = \"no\"\n```\n").unwrap();
        let mut raw = TomlConfig::default();
        overlay(&mut raw, &xdg).unwrap();
        overlay(&mut raw, &local).unwrap();
        let c = validate(raw).unwrap();
        assert_eq!(c.pin_freq, Some(PinFreq::Mhz(3801)));
        assert_eq!(c.source("pin_freq"), Some(xdg.as_path()));
    }

    #[test]
    fn the_freq_summary_reads_as_one_line() {
        let f = parse(
            "[freq]\ngovernor = \"powersave\"\nepp = \"balance_performance\"\nboost = true\n\
             min_mhz = 1745\nmax_mhz = 4673\npin_mhz = 3801\n",
        )
        .unwrap()
        .freq
        .unwrap();
        assert_eq!(
            f.summary(),
            "powersave, EPP balance_performance, boost on, clamp 1745-4673 MHz, pin 3801 MHz"
        );
        let bare = parse("[freq]\ngovernor = \"ondemand\"\nmax_mhz = 2400\n")
            .unwrap()
            .freq
            .unwrap();
        assert_eq!(
            bare.summary(),
            "ondemand, clamp undeclared-2400 (incomplete)"
        );
    }

    #[test]
    fn empty_is_all_none() {
        assert_eq!(parse("").unwrap(), Config::default());
    }

    #[test]
    fn scalars_parse() {
        let c = parse("duration = 2.5\nband_labels = \"zpn\"\ndecimals = 0\nsettle_time = 3.0\n")
            .unwrap();
        assert_eq!(c.duration, Some(2.5));
        assert_eq!(c.band_labels, Some(BandLabels::Zpn));
        assert_eq!(c.decimals, Some(0));
        assert_eq!(c.settle_time, Some(3.0));
    }

    #[test]
    fn benches_take_a_list_or_one_name() {
        let c = parse("benches = [\"zcr-mpsc-v0-2t\", \"zcr-mpsc-v1-2t\"]\n").unwrap();
        assert_eq!(
            c.benches,
            Some(vec![
                "zcr-mpsc-v0-2t".to_string(),
                "zcr-mpsc-v1-2t".to_string()
            ])
        );
        let c = parse("benches = \"all\"\n").unwrap();
        assert_eq!(c.benches, Some(vec!["all".to_string()]));
        assert_eq!(parse("").unwrap().benches, None);
        let err = parse("benches = []\n").unwrap_err();
        assert!(err.contains("benches"), "unexpected error: {err}");
        assert!(parse("benches = [\"\"]\n").is_err());
        assert!(parse("benches = 3\n").is_err());
    }

    #[test]
    fn runs_and_run_sleep_parse_and_range_check() {
        let c = parse("runs = 3\nrun_sleep = \"1-3s\"\n").unwrap();
        assert_eq!(c.runs, Some(3));
        assert_eq!(c.run_sleep, Some((1.0, 3.0)));
        assert!(parse("runs = 0\n").unwrap_err().contains("runs"));
        assert!(parse("runs = 1001\n").is_err());
        assert!(
            parse("run_sleep = \"soon\"\n")
                .unwrap_err()
                .contains("run_sleep")
        );
    }

    #[test]
    fn every_run_parameter_has_a_key() {
        let c = parse(
            "total_duration = \"30s\"\nsamples = 1000\ninner = 1\npin_cpus = \"0,1\"\n\
             record_dir = \"records\"\nenv_probe = false\ninhibit = false\nticks = true\n\
             verbose = true\n[tags]\nexperiment = \"clock-shift\"\ncondition = \"a=b\"\n",
        )
        .unwrap();
        assert_eq!(c.total_duration, Some(30.0));
        assert_eq!(c.samples, Some(1000));
        assert_eq!(c.inner, Some(1));
        assert_eq!(c.pin_cpus.as_deref(), Some("0,1"));
        assert_eq!(
            c.record,
            Some(crate::record::Target::Dir(PathBuf::from("records")))
        );
        assert_eq!(
            parse("record_file = \"r.jsonl\"\n").unwrap().record,
            Some(crate::record::Target::File(PathBuf::from("r.jsonl")))
        );
        let gone = parse("record = \"records/\"\n").unwrap_err();
        assert!(
            gone.contains("record_dir") && gone.contains("record_file"),
            "{gone}"
        );
        assert_eq!(c.env_probe, Some(false));
        assert_eq!(c.inhibit, Some(false));
        assert_eq!(c.ticks, Some(true));
        assert_eq!(c.verbose, Some(true));
        assert_eq!(c.tags["experiment"], "clock-shift");
        // A value may hold '=', as the flag's may. A key may not.
        assert_eq!(c.tags["condition"], "a=b");
        for bad in [
            "pin_cpus = \"\"\n",
            "record_dir = \"\"\n",
            "record_file = \"\"\n",
            "record_dir = \"a\"\nrecord_file = \"b\"\n",
            "verbose = \"yes\"\n",
            "samples = -1\n",
            "[tags]\n\"a=b\" = \"c\"\n",
            "[tags]\n\"\" = \"c\"\n",
            "[tags]\nn = 5\n",
            "duration = 1\ntotal_duration = 10\n",
        ] {
            assert!(parse(bad).is_err(), "{bad:?} passed");
        }
    }

    #[test]
    fn the_nearer_files_duration_choice_clears_the_other() {
        let dir = scratch("duration-choice");
        let xdg = dir.join("xdg.toml");
        let local = dir.join("local.toml");
        let both = dir.join("both.toml");
        std::fs::write(&xdg, "duration = 5\n").unwrap();
        std::fs::write(&local, "total_duration = 60\n").unwrap();
        std::fs::write(&both, "duration = 5\ntotal_duration = 60\n").unwrap();
        let mut raw = TomlConfig::default();
        overlay(&mut raw, &xdg).unwrap();
        overlay(&mut raw, &local).unwrap();
        let err = overlay(&mut TomlConfig::default(), &both).unwrap_err();
        assert!(err.contains("both.toml"), "unexpected error: {err}");
        let c = validate(raw).unwrap();
        assert_eq!(c.duration, None);
        assert_eq!(c.source("duration"), None);
        assert_eq!(c.total_duration, Some(60.0));
        assert_eq!(c.source("total_duration"), Some(local.as_path()));
    }

    #[test]
    fn the_nearer_files_record_mode_clears_the_other() {
        let dir = scratch("record-choice");
        let xdg = dir.join("xdg.toml");
        let local = dir.join("local.toml");
        let both = dir.join("both.toml");
        std::fs::write(&xdg, "record_dir = \"records\"\n").unwrap();
        std::fs::write(&local, "record_file = \"runs.jsonl\"\n").unwrap();
        std::fs::write(&both, "record_dir = \"a\"\nrecord_file = \"b\"\n").unwrap();
        let mut raw = TomlConfig::default();
        overlay(&mut raw, &xdg).unwrap();
        overlay(&mut raw, &local).unwrap();
        let err = overlay(&mut TomlConfig::default(), &both).unwrap_err();
        assert!(err.contains("both.toml"), "unexpected error: {err}");
        let c = validate(raw).unwrap();
        assert_eq!(
            c.record,
            Some(crate::record::Target::File(PathBuf::from("runs.jsonl")))
        );
        assert_eq!(c.source("record"), Some(local.as_path()));
    }

    #[test]
    fn tags_merge_by_key_across_files() {
        let dir = scratch("tags-merge");
        let xdg = dir.join("xdg.toml");
        let local = dir.join("local.toml");
        std::fs::write(&xdg, "[tags]\nhost = \"a\"\ncondition = \"x\"\n").unwrap();
        std::fs::write(&local, "[tags]\ncondition = \"y\"\n").unwrap();
        let mut raw = TomlConfig::default();
        overlay(&mut raw, &xdg).unwrap();
        overlay(&mut raw, &local).unwrap();
        let c = validate(raw).unwrap();
        assert_eq!(c.tags["host"], "a");
        assert_eq!(c.tags["condition"], "y");
        assert_eq!(c.source("tags"), Some(local.as_path()));
    }

    #[test]
    fn a_later_files_benches_replace_the_earlier_list_whole() {
        let dir = std::env::temp_dir().join(format!("iiac-perf-benches-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let xdg = dir.join("xdg.toml");
        let local = dir.join("local.toml");
        std::fs::write(&xdg, "benches = [\"min-now\", \"std-now\"]\n").unwrap();
        std::fs::write(&local, "benches = [\"mpsc-2t\"]\n").unwrap();
        let mut raw = TomlConfig::default();
        overlay(&mut raw, &xdg).unwrap();
        overlay(&mut raw, &local).unwrap();
        let c = validate(raw).unwrap();
        assert_eq!(c.benches, Some(vec!["mpsc-2t".to_string()]));
        assert_eq!(c.source("benches"), Some(local.as_path()));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn negative_settle_time_errs() {
        assert!(parse("settle_time = -1.0\n").is_err());
        // Zero is legal: it means "skip the warm".
        assert_eq!(parse("settle_time = 0.0\n").unwrap().settle_time, Some(0.0));
    }

    #[test]
    fn seconds_keys_take_a_unit_string() {
        let c =
            parse("duration = \"250ms\"\nsettle_time = \"0.1s\"\nwarm_cap = \"100ms\"\n").unwrap();
        assert_eq!(c.duration, Some(0.25));
        assert_eq!(c.settle_time, Some(0.1));
        assert_eq!(c.warm_cap, Some(0.1));
        assert!(parse("duration = \"250\"\n").is_ok());
        assert!(parse("duration = \"1-2s\"\n").is_err());
        assert!(parse("warm_cap = \"-1s\"\n").is_err());
    }

    #[test]
    fn negative_warm_cap_errs() {
        assert!(parse("warm_cap = -1.0\n").is_err());
        // Zero is legal: cap immediately, run uncertified.
        assert_eq!(parse("warm_cap = 0.0\n").unwrap().warm_cap, Some(0.0));
    }

    #[test]
    fn blocks_parses_and_range_checks() {
        assert_eq!(parse("blocks = 10\n").unwrap().blocks, Some(10));
        // One block is a plain run. Zero is nothing, and the ceiling matches --blocks.
        assert_eq!(parse("blocks = 1\n").unwrap().blocks, Some(1));
        assert!(parse("blocks = 0\n").is_err());
        assert!(parse("blocks = 1001\n").is_err());
    }

    #[test]
    fn block_knobs_parse_through_timespec() {
        let c = parse("block_sleep = \"1-10ms\"\nblock_warmup = \"2ms\"\n").unwrap();
        assert_eq!(c.block_sleep, Some((0.001, 0.010)));
        assert_eq!(c.block_warmup, Some(0.002));
    }

    #[test]
    fn block_knob_errors_name_the_key() {
        let err = parse("block_sleep = \"5\"\n").unwrap_err();
        assert!(err.contains("block_sleep"), "unexpected error: {err}");
        let err = parse("block_warmup = \"1-10ms\"\n").unwrap_err();
        assert!(err.contains("block_warmup"), "unexpected error: {err}");
    }

    #[test]
    fn profiles_parse_and_resolve() {
        let c = parse("[profiles]\nsmt = \"0,12\"\nccx = \"0,1\"\n").unwrap();
        assert_eq!(c.resolve_pin("smt").unwrap(), "0,12");
        assert_eq!(c.resolve_pin("ccx").unwrap(), "0,1");
        // A CPU list passes through untouched, and an empty spec clears a file's pin.
        assert_eq!(c.resolve_pin("0,3-5").unwrap(), "0,3-5");
        assert_eq!(c.resolve_pin("").unwrap(), "");
    }

    #[test]
    fn an_undeclared_profile_name_names_the_hosts_file_and_what_it_declares() {
        let c = parse("[profiles]\nsmt = \"0,12\"\n").unwrap();
        let err = c.resolve_pin("x-ccx").unwrap_err();
        assert!(err.contains("[profiles]"), "got: {err}");
        // A host with profiles leads with the ones it has, since naming one is the nearer fix
        // than declaring another, and a host that cannot form the hop never will.
        assert!(err.contains("declares: smt = \"0,12\""), "got: {err}");
        assert!(err.contains("Name one of those"), "got: {err}");
        // A host with none gets the how-to instead, and the command that writes the file.
        let bare = parse("blocks = 10\n").unwrap();
        assert!(bare.resolve_pin("smt").unwrap_err().contains("setup-freq"));
        assert!(
            bare.resolve_pin("smt")
                .unwrap_err()
                .contains("declares none"),
            "a host with no profiles says so"
        );
    }

    #[test]
    fn bad_band_labels_errs() {
        assert!(parse("band_labels = \"nope\"\n").is_err());
    }

    #[test]
    fn out_of_range_decimals_errs() {
        assert!(parse("decimals = 5\n").is_err());
    }

    #[test]
    fn unknown_key_errs() {
        assert!(parse("bogus = 1\n").is_err());
    }

    #[test]
    fn freq_table_parses() {
        let c = parse(
            "[freq]\ngovernor = \"powersave\"\nepp = \"balance_performance\"\n\
             boost = true\npin_mhz = 3800\n",
        )
        .unwrap();
        let f = c.freq.unwrap();
        assert_eq!(f.governor, "powersave");
        assert_eq!(f.epp.as_deref(), Some("balance_performance"));
        assert_eq!(f.boost, Some(true));
        assert_eq!(f.min_mhz, None);
        assert_eq!(f.pin_mhz, Some(3800));
    }

    #[test]
    fn freq_table_requires_governor() {
        assert!(parse("[freq]\nboost = true\n").is_err());
    }

    #[test]
    fn freq_rejects_zero_and_inverted_clamps() {
        assert!(parse("[freq]\ngovernor = \"g\"\npin_mhz = 0\n").is_err());
        assert!(parse("[freq]\ngovernor = \"g\"\nmin_mhz = 4000\nmax_mhz = 3000\n").is_err());
        assert!(parse("[freq]\ngovernor = \"g\"\nmin_mhz = 550\nmax_mhz = 5573\n").is_ok());
    }

    #[test]
    fn freq_table_replaces_whole_never_field_merges() {
        let dir = scratch("freq-replace");
        let xdg = dir.join("config.toml");
        let local = dir.join("local.toml");
        std::fs::write(
            &xdg,
            "[freq]\ngovernor = \"powersave\"\nepp = \"balance_performance\"\nboost = true\n",
        )
        .unwrap();
        std::fs::write(&local, "[freq]\ngovernor = \"schedutil\"\n").unwrap();
        let mut raw = TomlConfig::default();
        overlay(&mut raw, &xdg).unwrap();
        overlay(&mut raw, &local).unwrap();
        let f = validate(raw).unwrap().freq.unwrap();
        // The nearer file's whole table wins: no epp/boost leak through from the XDG layer.
        assert_eq!(f.governor, "schedutil");
        assert_eq!(f.epp, None);
        assert_eq!(f.boost, None);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A fresh scratch directory under the test temp root, named
    /// per test so parallel tests never share one.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("iiac-perf-config-tests")
            .join(name);
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_named_file_is_found_up_the_parents_then_in_the_xdg_directory() {
        let root = scratch("find-named");
        let deep = root.join("benches/spsc");
        let xdg = root.join("xdg");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::create_dir_all(root.join("benches/configs")).unwrap();
        std::fs::create_dir_all(&xdg).unwrap();
        let find = |name: &str| find_named(Path::new(name), &deep, Some(&xdg));
        let err = find("common").unwrap_err();
        assert!(err.contains("benches/spsc") && err.contains("xdg"), "{err}");
        assert!(err.contains("or with .md"), "{err}");
        assert!(!find("common.md").unwrap_err().contains("or with .md"));

        std::fs::write(xdg.join("common.toml"), "").unwrap();
        assert_eq!(find("common").unwrap(), xdg.join("common.toml"));
        // A parent's file is nearer than the XDG one, and the directory's own nearer still.
        std::fs::write(root.join("benches/common.md"), "").unwrap();
        assert_eq!(find("common").unwrap(), root.join("benches/common.md"));
        assert_eq!(find("common.md").unwrap(), root.join("benches/common.md"));
        assert_eq!(find("common.toml").unwrap(), xdg.join("common.toml"));
        std::fs::write(deep.join("common.md"), "").unwrap();
        assert_eq!(find("common").unwrap(), deep.join("common.md"));
        // A name with a directory in it searches the same way.
        std::fs::write(root.join("benches/configs/shift.md"), "").unwrap();
        assert_eq!(
            find("configs/shift").unwrap(),
            root.join("benches/configs/shift.md")
        );
        // Both carriers in one place is the error it is for the other layers.
        std::fs::write(deep.join("common.toml"), "").unwrap();
        assert!(find("common").unwrap_err().contains("both"));
        // An absolute name is looked for where it says and nowhere else.
        let abs = root.join("nowhere/common");
        assert!(find(abs.to_str().unwrap()).is_err());
        assert!(find("").is_err());
    }

    #[test]
    fn the_local_file_is_the_nearest_up_the_parents_and_no_higher_one() {
        let root = scratch("local-up");
        let deep = root.join("benches/spsc");
        std::fs::create_dir_all(&deep).unwrap();
        let load = |own: bool| load_from(None, &deep, own, None).unwrap();
        assert!(load(false).1.is_empty());

        std::fs::write(root.join("iiac-perf.toml"), "blocks = 10\nruns = 2\n").unwrap();
        let (c, files) = load(false);
        assert_eq!((c.blocks, c.runs), (Some(10), Some(2)));
        assert_eq!(files, [root.join("iiac-perf.toml")]);
        // A nearer file ends the search: the higher one's `runs` does not come through.
        std::fs::write(
            root.join("benches/iiac-perf.md"),
            "```toml\nblocks = 20\n```\n",
        )
        .unwrap();
        let (c, files) = load(false);
        assert_eq!((c.blocks, c.runs), (Some(20), None));
        assert_eq!(files, [root.join("benches/iiac-perf.md")]);
        assert_eq!(
            c.source("blocks"),
            Some(root.join("benches/iiac-perf.md").as_path())
        );
        // Both carriers in the directory that ends the search is the error it always was.
        std::fs::write(root.join("benches/iiac-perf.toml"), "").unwrap();
        assert!(
            load_from(None, &deep, false, None)
                .unwrap_err()
                .contains("both")
        );
        // In the process's own directory the file keeps its bare name.
        let files = layer_files(None, &root, true).unwrap();
        assert_eq!(files, [PathBuf::from("iiac-perf.toml")]);
    }

    #[test]
    fn a_named_files_run_keys_stand_alone_over_the_hosts_facts() {
        let root = scratch("named-layers");
        let xdg = root.join("xdg");
        let cwd = root.join("work");
        std::fs::create_dir_all(&xdg).unwrap();
        std::fs::create_dir_all(&cwd).unwrap();
        std::fs::write(
            xdg.join("config.toml"),
            "blocks = 10\n[tags]\nhost = \"a\"\n[profiles]\nsmt = \"0,12\"\n\
             [freq]\ngovernor = \"powersave\"\n",
        )
        .unwrap();
        std::fs::write(cwd.join("iiac-perf.toml"), "runs = 2\n").unwrap();
        std::fs::write(
            cwd.join("run.toml"),
            "benches = \"min-now\"\ndecimals = 2\n",
        )
        .unwrap();

        let (plain, files) = load_from(Some(&xdg), &cwd, false, None).unwrap();
        assert_eq!((plain.blocks, plain.runs), (Some(10), Some(2)));
        assert_eq!(files.len(), 2);

        let (c, files) = load_from(Some(&xdg), &cwd, false, Some(Path::new("run"))).unwrap();
        // The host's run keys are gone, sources and all, and its facts stay.
        assert_eq!((c.blocks, c.runs, c.decimals), (None, None, Some(2)));
        assert!(c.tags.is_empty());
        assert_eq!(c.source("blocks"), None);
        assert_eq!(c.resolve_pin("smt").unwrap(), "0,12");
        assert_eq!(c.source("freq"), Some(xdg.join("config.toml").as_path()));
        assert_eq!(c.source("decimals"), Some(cwd.join("run.toml").as_path()));
        assert_eq!(files.last(), Some(&cwd.join("run.toml")));

        // A named file's own [freq] is the nearest, so it wins, whole.
        std::fs::write(
            cwd.join("pinned.toml"),
            "[freq]\ngovernor = \"schedutil\"\n",
        )
        .unwrap();
        let (c, _) = load_from(Some(&xdg), &cwd, false, Some(Path::new("pinned"))).unwrap();
        assert_eq!(c.freq.unwrap().governor, "schedutil");
        assert!(load_from(Some(&xdg), &cwd, false, Some(Path::new("absent"))).is_err());
    }

    #[test]
    fn md_carrier_overlays_through_the_fence_filter() {
        let dir = scratch("md-carrier");
        let path = dir.join("config.md");
        std::fs::write(
            &path,
            "# iiac-perf config\n\nprose documenting duration\n```toml\nduration = 2.5\n```\n\
             \nprofiles, one per line\n```toml\n[profiles]\nsmt = \"0,12\"\n```\n",
        )
        .unwrap();
        let mut raw = TomlConfig::default();
        overlay(&mut raw, &path).unwrap();
        let c = validate(raw).unwrap();
        assert_eq!(c.duration, Some(2.5));
        assert_eq!(c.resolve_pin("smt").unwrap(), "0,12");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn md_parse_error_names_the_md_path() {
        let dir = scratch("md-bad");
        let path = dir.join("config.md");
        std::fs::write(&path, "prose\n```toml\nk = \"v\"\n").unwrap();
        let mut raw = TomlConfig::default();
        let err = overlay(&mut raw, &path).unwrap_err();
        assert!(err.contains("config.md"), "unexpected error: {err}");
        assert!(err.contains("unclosed fence"), "unexpected error: {err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolve_prefers_lone_carrier_and_rejects_both() {
        let dir = scratch("resolve");
        let md = dir.join("config.md");
        let toml = dir.join("config.toml");
        let resolve = |md: &PathBuf, toml: &PathBuf| resolve_carrier(md.clone(), toml.clone());

        assert_eq!(resolve(&md, &toml).unwrap(), None);

        std::fs::write(&toml, "duration = 1.0\n").unwrap();
        assert_eq!(resolve(&md, &toml).unwrap(), Some(toml.clone()));

        std::fs::write(&md, "```toml\nduration = 2.0\n```\n").unwrap();
        let err = resolve(&md, &toml).unwrap_err();
        assert!(err.contains("config.md"), "unexpected error: {err}");
        assert!(err.contains("config.toml"), "unexpected error: {err}");

        std::fs::remove_file(&toml).unwrap();
        assert_eq!(resolve(&md, &toml).unwrap(), Some(md.clone()));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_record_given_as_a_config_is_named_and_a_long_line_is_cut() {
        let record = "{\"schema_version\":8,\"bench\":\"min-now\"}\n";
        for err in [
            parse_raw(Path::new("r.jsonl"), record).unwrap_err(),
            parse_table(Path::new("r.jsonl"), record).unwrap_err(),
        ] {
            assert!(err.contains("a record rather than a config"), "{err}");
            assert!(err.contains("--from-record r.jsonl"), "{err}");
        }
        // A bad line past the cap is quoted to the cap and no further.
        let long = format!("blocks = 1 {}\n", "x".repeat(300));
        let err = parse_table(Path::new("c.toml"), &long).unwrap_err();
        let source = err
            .lines()
            .find(|l| l.starts_with("1 |"))
            .expect("the line is quoted");
        assert!(source.ends_with("...") && source.chars().count() == ERROR_LINE_MAX + 3);
        // A column past the cut keeps its caret.
        let far = format!("blocks = {} x\n", "1".repeat(200));
        let err = parse_table(Path::new("c.toml"), &far).unwrap_err();
        assert!(err.lines().any(|l| l.trim_end().ends_with('^')), "{err}");
    }
}
