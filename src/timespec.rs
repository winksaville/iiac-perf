//! Duration-span parsing for the block knobs (`--block-sleep`,
//! `--block-warmup`, and their config keys).
//!
//! A spec is a duration (`2ms`, `0.5s`, `250us`) or a range
//! (`1-10ms`, `500ms-2s`), returned in seconds. The unit is
//! required on any nonzero value: a bare `5` silently meaning
//! milliseconds to one reader and seconds to another is exactly
//! the ambiguity these knobs exist to remove. A bare `0` is exact
//! in any unit.

/// Seconds per unit token, `Ok(None)` for a unitless number.
fn unit_scale(unit: &str) -> Result<Option<f64>, String> {
    match unit {
        "" => Ok(None),
        "us" => Ok(Some(1e-6)),
        "ms" => Ok(Some(1e-3)),
        "s" => Ok(Some(1.0)),
        other => Err(format!("unknown unit {other:?} (use us, ms, or s)")),
    }
}

/// Split one part into its number and unit texts at the first
/// alphabetic character (`10ms` -> `10`, `ms`).
fn split_unit(part: &str) -> (&str, &str) {
    match part.find(|c: char| c.is_ascii_alphabetic()) {
        Some(i) => part.split_at(i),
        None => (part, ""),
    }
}

/// Parse one part into seconds. `fallback` is the other end's unit
/// scale, so a range's trailing unit distributes (`1-10ms`).
fn parse_part(part: &str, fallback: Option<f64>) -> Result<f64, String> {
    let part = part.trim();
    let (num, unit) = split_unit(part);
    let num = num.trim();
    if num.is_empty() {
        return Err(format!("{part:?} has no number"));
    }
    let v: f64 = num
        .parse()
        .map_err(|_| format!("{num:?} is not a number"))?;
    if !v.is_finite() || v < 0.0 {
        return Err(format!("{part:?} is not a non-negative duration"));
    }
    let scale = match unit_scale(unit.trim())? {
        Some(s) => Some(s),
        None if v == 0.0 => Some(1.0),
        None => fallback,
    };
    match scale {
        Some(s) => Ok(v * s),
        None => Err(format!("{part:?} needs a unit: us, ms, or s")),
    }
}

/// Parse a span spec into `(min_s, max_s)` seconds. A scalar is a
/// span with min = max; a range's ends must be ordered.
pub fn parse_span(spec: &str) -> Result<(f64, f64), String> {
    match spec.split_once('-') {
        None => {
            let v = parse_part(spec, None)?;
            Ok((v, v))
        }
        Some((lo, hi)) => {
            let (_, hi_unit) = split_unit(hi.trim());
            let hi_scale = unit_scale(hi_unit.trim())?;
            let hi_v = parse_part(hi, None)?;
            let lo_v = parse_part(lo, hi_scale)?;
            if lo_v > hi_v {
                return Err(format!("{spec:?}: min exceeds max"));
            }
            Ok((lo_v, hi_v))
        }
    }
}

/// Parse a scalar duration spec into seconds, rejecting ranges:
/// the warmup knob is one value, never a re-rolled span.
pub fn parse_scalar(spec: &str) -> Result<f64, String> {
    if spec.contains('-') {
        return Err(format!("{spec:?}: one duration, not a range"));
    }
    Ok(parse_span(spec)?.0)
}

/// Render seconds back into the largest unit that reads whole-ish
/// (`0.002` -> `2 ms`), for the Setup block.
pub fn display(seconds: f64) -> String {
    let (v, unit) = if seconds >= 1.0 {
        (seconds, "s")
    } else if seconds >= 1e-3 {
        (seconds * 1e3, "ms")
    } else {
        (seconds * 1e6, "us")
    };
    // Trim a trailing ".0" so whole values read as integers.
    let s = format!("{v:.1}");
    let s = s.strip_suffix(".0").unwrap_or(&s); // OK: obvious
    format!("{s} {unit}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalars_parse_with_units() {
        assert_eq!(parse_span("2ms").unwrap(), (0.002, 0.002));
        assert_eq!(parse_span("0.5s").unwrap(), (0.5, 0.5));
        assert_eq!(parse_span("250us").unwrap(), (0.000_25, 0.000_25));
    }

    #[test]
    fn zero_needs_no_unit() {
        assert_eq!(parse_span("0").unwrap(), (0.0, 0.0));
        assert_eq!(parse_scalar("0").unwrap(), 0.0);
    }

    #[test]
    fn nonzero_without_unit_errs() {
        assert!(parse_span("5").is_err());
        assert!(parse_span("1-10").is_err());
    }

    #[test]
    fn ranges_distribute_the_trailing_unit() {
        assert_eq!(parse_span("1-10ms").unwrap(), (0.001, 0.010));
        assert_eq!(parse_span("500ms-2s").unwrap(), (0.5, 2.0));
        assert_eq!(parse_span("0-1s").unwrap(), (0.0, 1.0));
    }

    #[test]
    fn inverted_range_errs() {
        assert!(parse_span("10-1ms").is_err());
    }

    #[test]
    fn junk_errs() {
        assert!(parse_span("").is_err());
        assert!(parse_span("fast").is_err());
        assert!(parse_span("5parsecs").is_err());
        assert!(parse_span("-5ms").is_err());
        assert!(parse_span("NaNs").is_err());
    }

    #[test]
    fn scalar_rejects_ranges() {
        assert!(parse_scalar("1-10ms").is_err());
        assert_eq!(parse_scalar("2ms").unwrap(), 0.002);
    }

    #[test]
    fn display_picks_a_readable_unit() {
        assert_eq!(display(2.0), "2 s");
        assert_eq!(display(0.002), "2 ms");
        assert_eq!(display(0.0005), "500 us");
        assert_eq!(display(0.0015), "1.5 ms");
    }
}
