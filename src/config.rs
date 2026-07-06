//! Layered configuration for defaults: built-in < XDG config file <
//! project-local file < CLI flags.
//!
//! - **XDG file** — `$XDG_CONFIG_HOME/iiac-perf/config.toml`, or
//!   `$HOME/.config/iiac-perf/config.toml` when `XDG_CONFIG_HOME`
//!   is unset. The per-user home for defaults and pin profiles.
//! - **Project-local file** — [`LOCAL_FILE`] in the current
//!   directory (no upward walk). Overrides the XDG file
//!   field-by-field; profiles merge by key.
//! - **CLI** — always wins, resolved in `main` after [`load`].
//!
//! A malformed or unreadable-but-present file is a hard error, so a
//! typo surfaces rather than silently reverting to built-ins.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Deserialize;

use crate::bands::BandLabels;

/// Project-local override filename, looked up in the current
/// directory only.
const LOCAL_FILE: &str = "iiac-perf.toml";

/// Inclusive upper bound on `decimals`, mirroring the
/// `--decimals` CLI `value_parser` range.
const DECIMALS_MAX: u8 = 3;

/// The on-disk TOML shape, before validation. Scalars are
/// `Option` so an absent key stays absent (letting a lower layer
/// or built-in default show through); unknown keys are rejected
/// to catch typos.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    /// Default `--duration` seconds.
    duration: Option<f64>,
    /// Default `--band-labels` style, as its lowercase name.
    band_labels: Option<String>,
    /// Default `--decimals` count.
    decimals: Option<u8>,
    /// Named pin profiles: name -> `--pin` core spec.
    #[serde(default)]
    profiles: BTreeMap<String, String>,
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
    /// Named pin profiles: name -> `--pin` core spec.
    pub profiles: BTreeMap<String, String>,
}

impl Config {
    /// Resolve a `--pin` spec against the configured profiles: a
    /// spec that names a profile expands to that profile's core
    /// spec; anything else is returned unchanged for
    /// [`crate::pin::parse_cores`] to parse as a raw core list.
    pub fn resolve_pin<'a>(&'a self, spec: &'a str) -> &'a str {
        self.profiles.get(spec).map(String::as_str).unwrap_or(spec)
    }
}

/// The XDG config-file path: `$XDG_CONFIG_HOME/iiac-perf/config.toml`,
/// falling back to `$HOME/.config/...`. `None` if neither env var is
/// set (e.g. a stripped-down service environment).
fn xdg_path() -> Option<PathBuf> {
    let base = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(x) if !x.is_empty() => PathBuf::from(x),
        _ => PathBuf::from(std::env::var_os("HOME")?).join(".config"),
    };
    Some(base.join("iiac-perf").join("config.toml"))
}

/// Load and merge the XDG and project-local config files. Returns
/// the merged [`Config`] plus the paths of the files that actually
/// existed (for the startup banner). Built-in-default `Config` when
/// no file exists; errors on a present-but-unreadable or malformed
/// file.
///
/// Layering: start from the XDG file, then overlay the local file
/// (scalars replace, profiles merge by key), so the nearer file
/// wins per field.
pub fn load() -> Result<(Config, Vec<PathBuf>), String> {
    let mut raw = RawConfig::default();
    let mut loaded = Vec::new();
    if let Some(path) = xdg_path()
        && overlay(&mut raw, &path)?
    {
        loaded.push(path);
    }
    let local = PathBuf::from(LOCAL_FILE);
    if overlay(&mut raw, &local)? {
        loaded.push(local);
    }
    Ok((validate(raw)?, loaded))
}

/// Read one config file and overlay it onto `base`: each present
/// scalar replaces `base`'s, and profiles are merged by key (the
/// file's entries win). Returns whether the file existed; a missing
/// file is a no-op returning `false`.
fn overlay(base: &mut RawConfig, path: &PathBuf) -> Result<bool, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(format!("reading {}: {e}", path.display())),
    };
    let over: RawConfig =
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
    base.profiles.extend(over.profiles);
    Ok(true)
}

/// Validate a merged [`RawConfig`] into a [`Config`]: map the
/// `band_labels` name to the enum and range-check `decimals`.
fn validate(raw: RawConfig) -> Result<Config, String> {
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
    Ok(Config {
        duration: raw.duration,
        band_labels,
        decimals: raw.decimals,
        profiles: raw.profiles,
    })
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
        let c = parse("duration = 2.5\nband_labels = \"zpn\"\ndecimals = 0\n").unwrap();
        assert_eq!(c.duration, Some(2.5));
        assert_eq!(c.band_labels, Some(BandLabels::Zpn));
        assert_eq!(c.decimals, Some(0));
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
}
