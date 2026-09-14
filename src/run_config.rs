//! The run's parameters as resolved: each one's value and where the value came from.
//!
//! A value reaches a run from a built-in default, a config file, or a flag, and the report used
//! to print some values and no sources, so a reader could not tell a box's declared `blocks`
//! from the built-in, or a flag typed from one forgotten. The `Config:` list prints every
//! parameter with its source, and marks a source that restates the default, since that value
//! would survive the source's removal unchanged.

use std::path::Path;

use crate::config::Config;

/// Where a resolved value came from.
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    /// No file and no flag set it.
    Default,
    /// A config file set it, the path as loaded.
    File(std::path::PathBuf),
    /// A flag set it, as typed on the line (`--blocks`), with any qualifier the value needs.
    Flag(String),
}

/// One run parameter: its config-key-style name, its value as the report prints it, and its
/// source.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// The parameter's name, the config key where one exists (`block_sleep`).
    pub key: &'static str,
    /// The resolved value, rendered.
    pub value: String,
    /// Where the value came from.
    pub source: Source,
    /// A file or flag set the value to what the default would have given.
    pub same_as_default: bool,
}

impl Param {
    /// A parameter whose value is `value` and whose default renders as `default`.
    pub fn new(key: &'static str, value: String, default: &str, source: Source) -> Self {
        let same_as_default = source != Source::Default && value == default;
        Param {
            key,
            value,
            source,
            same_as_default,
        }
    }
}

/// Resolve one layered parameter, the flag winning, then the config file, then `default`,
/// returning the value and its source.
pub fn layered<T>(
    flag_value: Option<T>,
    flag: &str,
    file_value: Option<T>,
    key: &str,
    config: &Config,
    default: T,
) -> (T, Source) {
    if let Some(v) = flag_value {
        return (v, Source::Flag(flag.to_string()));
    }
    match (file_value, config.source(key)) {
        (Some(v), Some(path)) => (v, Source::File(path.to_path_buf())),
        (Some(v), None) => (v, Source::Default),
        (None, _) => (default, Source::Default),
    }
}

/// A path as a reader types it: the home directory as `~`.
pub fn display_path(path: &Path) -> String {
    if let Some(home) = std::env::var_os("HOME")
        && let Ok(rest) = path.strip_prefix(&home)
    {
        return format!("~/{}", rest.display());
    }
    path.display().to_string()
}

/// A source as the list prints it: `(default)`, the flag, or the file, with the restatement
/// marked.
pub fn source_cell(param: &Param) -> String {
    let source = match &param.source {
        Source::Default => return "(default)".to_string(),
        Source::File(path) => display_path(path),
        Source::Flag(flag) => flag.clone(),
    };
    if param.same_as_default {
        format!("({source}, same as default)")
    } else {
        format!("({source})")
    }
}

/// Values longer than this do not widen the value column, so one long value (the `freq`
/// summary) does not push every other line's source to the right edge.
const VALUE_COLUMN_CAP: usize = 24;

/// The `Config:` list's lines, one per parameter, name and value columns aligned, a value past
/// [`VALUE_COLUMN_CAP`] running over its column.
pub fn lines(params: &[Param]) -> Vec<String> {
    let mut max_value_width = 0;
    for p in params {
        if p.value.len() <= VALUE_COLUMN_CAP {
            max_value_width = max_value_width.max(p.value.len());
        }
    }
    params
        .iter()
        .map(|p| {
            format!(
                "  {:<17} {:<max_value_width$}  {}",
                p.key,
                p.value,
                source_cell(p)
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn config_with_source(key: &'static str, path: &str) -> Config {
        let mut config = Config::default();
        config.sources.insert(key, PathBuf::from(path));
        config
    }

    #[test]
    fn the_flag_wins_then_the_file_then_the_default() {
        let config = config_with_source("blocks", "iiac-perf.md");
        assert_eq!(
            layered(Some(10), "--blocks", Some(50), "blocks", &config, 100),
            (10, Source::Flag("--blocks".to_string()))
        );
        assert_eq!(
            layered(None, "--blocks", Some(50), "blocks", &config, 100),
            (50, Source::File(PathBuf::from("iiac-perf.md")))
        );
        assert_eq!(
            layered(
                None,
                "--blocks",
                None::<u64>,
                "blocks",
                &Config::default(),
                100
            ),
            (100, Source::Default)
        );
    }

    #[test]
    fn a_source_restating_the_default_is_marked() {
        let file = Source::File(PathBuf::from("iiac-perf.md"));
        let same = Param::new("blocks", "100".to_string(), "100", file.clone());
        assert_eq!(source_cell(&same), "(iiac-perf.md, same as default)");
        let differs = Param::new("blocks", "50".to_string(), "100", file);
        assert_eq!(source_cell(&differs), "(iiac-perf.md)");
        let default = Param::new("blocks", "100".to_string(), "100", Source::Default);
        assert!(!default.same_as_default);
        assert_eq!(source_cell(&default), "(default)");
    }

    #[test]
    fn lines_align_the_value_column() {
        let params = [
            Param::new("blocks", "100".to_string(), "100", Source::Default),
            Param::new(
                "pin_cpus",
                "0,1".to_string(),
                "none",
                Source::Flag("--pin-cpus".to_string()),
            ),
            Param::new("record", "none".to_string(), "none", Source::Default),
        ];
        assert_eq!(
            lines(&params),
            [
                "  blocks            100   (default)",
                "  pin_cpus          0,1   (--pin-cpus)",
                "  record            none  (default)",
            ]
        );
    }

    #[test]
    fn a_long_value_does_not_widen_the_column() {
        let long = "powersave, EPP balance_performance, boost on, clamp 1745-4673 MHz";
        let params = [
            Param::new("blocks", "100".to_string(), "100", Source::Default),
            Param::new(
                "freq",
                long.to_string(),
                "none declared",
                Source::File(PathBuf::from("iiac-perf.md")),
            ),
        ];
        assert_eq!(
            lines(&params),
            [
                "  blocks            100  (default)".to_string(),
                format!("  freq              {long}  (iiac-perf.md)"),
            ]
        );
    }
}
