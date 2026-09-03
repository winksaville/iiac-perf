//! Layered configuration for defaults: built-in < XDG config file <
//! project-local file < CLI flags.
//!
//! - **XDG file**: `$XDG_CONFIG_HOME/iiac-perf/config.md` (or
//!   `.toml`), falling back to `$HOME/.config/iiac-perf/` when
//!   `XDG_CONFIG_HOME` is unset. The per-user home for defaults
//!   and pin profiles.
//! - **Project-local file**: `iiac-perf.md` (or `.toml`) in the
//!   current directory (no upward walk). Overrides the XDG file
//!   field-by-field, and profiles merge by key.
//! - **CLI**: always wins, resolved in `main` after [`load`].
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
/// carrier), looked up in the current directory only.
const LOCAL_MD: &str = "iiac-perf.md";
/// The project-local TOML carrier beside [`LOCAL_MD`].
const LOCAL_TOML: &str = "iiac-perf.toml";

/// Inclusive upper bound on `decimals`, mirroring the
/// `--decimals` CLI `value_parser` range.
const DECIMALS_MAX: u8 = 3;

/// Fewest blocks a config may ask for. One block is not a
/// replication, so the range starts at two. `main`'s `--blocks`
/// carries the same bounds inline, the way `--decimals` carries
/// [`DECIMALS_MAX`]'s.
const BLOCKS_MIN: u64 = 2;
/// Most blocks a config may ask for.
const BLOCKS_MAX: u64 = 1000;

/// The config file's shape as deserialized, before validation.
/// Scalars are `Option` so an absent key stays absent, letting a
/// lower layer or built-in default show through. Unknown keys are
/// rejected to catch typos.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct TomlConfig {
    /// Default `--duration` seconds.
    duration: Option<f64>,
    /// Default `--band-labels` style, as its lowercase name.
    band_labels: Option<String>,
    /// Default `--decimals` count.
    decimals: Option<u8>,
    /// Default `--settle-time` seconds.
    settle_time: Option<f64>,
    /// Default `--warm-cap` seconds.
    warm_cap: Option<f64>,
    /// Default `--blocks` count.
    blocks: Option<u64>,
    /// Default `--block-sleep` span spec (e.g. `"1-10ms"`).
    block_sleep: Option<String>,
    /// Default `--block-warmup` duration spec (e.g. `"2ms"`).
    block_warmup: Option<String>,
    /// Named pin profiles: name -> `--pin-cpus` CPU spec.
    #[serde(default)]
    profiles: BTreeMap<String, String>,
    /// The declared `[freq]` steady state and pin target.
    freq: Option<FreqConfig>,
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
    /// Steady-state lower clamp (MHz). Absent means the hardware minimum.
    pub min_mhz: Option<u64>,
    /// Steady-state upper clamp (MHz). Absent means the hardware maximum.
    pub max_mhz: Option<u64>,
    /// Pin target (MHz) for `pin-freq` and `--pin-freq`. Absent means the discovered base
    /// clock.
    pub pin_mhz: Option<u64>,
}

/// The merged, validated configuration handed to `main`.
///
/// Each scalar is `Option`: `None` means "no config opinion, use
/// the built-in default". Profiles are a flat name->spec map.
#[derive(Debug, Default, PartialEq)]
pub struct Config {
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
    /// Named pin profiles: name -> `--pin-cpus` CPU spec.
    pub profiles: BTreeMap<String, String>,
    /// The declared `[freq]` steady state and pin target, if configured.
    pub freq: Option<FreqConfig>,
}

impl Config {
    /// Resolve a `--pin-cpus` spec against the configured
    /// profiles: a spec that names a profile expands to that
    /// profile's CPU spec, and anything else is returned unchanged for
    /// [`crate::pin::parse_cpus`] to parse as a raw CPU list.
    pub fn resolve_pin<'a>(&'a self, spec: &'a str) -> &'a str {
        self.profiles.get(spec).map(String::as_str).unwrap_or(spec)
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

/// Load and merge the XDG and project-local config files. Returns
/// the merged [`Config`] plus the paths of the files that actually
/// existed (for the startup banner). Built-in-default `Config` when
/// no file exists. It errors on a present-but-unreadable or
/// malformed file, and on a directory holding both carriers.
///
/// Layering: start from the XDG file, then overlay the local file
/// (scalars replace, profiles merge by key), so the nearer file
/// wins per field. The carrier rule is per directory, so the two
/// layers may use different carriers.
pub fn load() -> Result<(Config, Vec<PathBuf>), String> {
    let mut raw = TomlConfig::default();
    let mut loaded = Vec::new();
    if let Some(dir) = xdg_dir()
        && let Some(path) = resolve_carrier(dir.join("config.md"), dir.join("config.toml"))?
    {
        overlay(&mut raw, &path)?;
        loaded.push(path);
    }
    if let Some(path) = resolve_carrier(PathBuf::from(LOCAL_MD), PathBuf::from(LOCAL_TOML))? {
        overlay(&mut raw, &path)?;
        loaded.push(path);
    }
    Ok((validate(raw)?, loaded))
}

/// Read one config file (either carrier) and overlay it onto
/// `base`: each present scalar replaces `base`'s, and profiles are
/// merged by key (the file's entries win). A `.md` path runs
/// through the fence filter first, and anything else parses as
/// plain TOML.
fn overlay(base: &mut TomlConfig, path: &Path) -> Result<(), String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let text = if path.extension().is_some_and(|e| e == "md") {
        md_to_toml(&text).map_err(|e| format!("{}: {e}", path.display()))?
    } else {
        text
    };
    let over: TomlConfig =
        toml::from_str(&text).map_err(|e| format!("parsing {}: {e}", path.display()))?;
    if over.duration.is_some() {
        base.duration = over.duration;
    }
    if over.band_labels.is_some() {
        base.band_labels = over.band_labels;
    }
    if over.decimals.is_some() {
        base.decimals = over.decimals;
    }
    if over.settle_time.is_some() {
        base.settle_time = over.settle_time;
    }
    if over.warm_cap.is_some() {
        base.warm_cap = over.warm_cap;
    }
    if over.blocks.is_some() {
        base.blocks = over.blocks;
    }
    if over.block_sleep.is_some() {
        base.block_sleep = over.block_sleep;
    }
    if over.block_warmup.is_some() {
        base.block_warmup = over.block_warmup;
    }
    // The whole [freq] table replaces, never field-merges: the steady state is one declaration
    // of one box's state, and half of one file's declaration on top of half of another's would
    // be a state nobody declared.
    if over.freq.is_some() {
        base.freq = over.freq;
    }
    base.profiles.extend(over.profiles);
    Ok(())
}

/// Validate a merged [`TomlConfig`] into a [`Config`]: map the
/// `band_labels` name to the enum, range-check `decimals`, and
/// reject a negative `settle_time`.
fn validate(raw: TomlConfig) -> Result<Config, String> {
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
    if let Some(t) = raw.settle_time
        && t < 0.0
    {
        return Err(format!("settle_time: {t} is negative"));
    }
    if let Some(t) = raw.warm_cap
        && t < 0.0
    {
        return Err(format!("warm_cap: {t} is negative"));
    }
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
    if let Some(f) = &raw.freq {
        validate_freq(f)?;
    }
    Ok(Config {
        duration: raw.duration,
        band_labels,
        decimals: raw.decimals,
        settle_time: raw.settle_time,
        warm_cap: raw.warm_cap,
        blocks: raw.blocks,
        block_sleep,
        block_warmup,
        profiles: raw.profiles,
        freq: raw.freq,
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
    fn negative_settle_time_errs() {
        assert!(parse("settle_time = -1.0\n").is_err());
        // Zero is legal: it means "skip the warm".
        assert_eq!(parse("settle_time = 0.0\n").unwrap().settle_time, Some(0.0));
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
        // One block is not a replication, and the ceiling matches --blocks.
        assert!(parse("blocks = 1\n").is_err());
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
        assert_eq!(c.resolve_pin("smt"), "0,12");
        assert_eq!(c.resolve_pin("ccx"), "0,1");
        // A non-profile spec passes through untouched.
        assert_eq!(c.resolve_pin("0,3-5"), "0,3-5");
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
        assert_eq!(c.resolve_pin("smt"), "0,12");
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
}
