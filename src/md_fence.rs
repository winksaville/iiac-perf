//! The md -> toml filter for markdown-carried config files.
//!
//! A config markdown document's `toml` fences, concatenated in
//! document order, form the TOML the loader parses, and the prose
//! between them documents the config without ever reaching a
//! parser. One rule falls out for authors: a `[table]` header
//! captures every key after it until the next header, so a table's
//! keys must stay in its stretch of the document.
//!
//! A copy of vc-x1's `src/md_fence.rs` (the format's origin), kept
//! std-only and dependency-free like the original, and a candidate
//! for extraction into a small shared crate once the family
//! converges on one implementation.

/// What the filter is inside of, line by line.
enum Fence {
    /// Outside any fence: prose, blanked.
    None,
    /// Inside a ```toml fence: lines pass through.
    Toml,
    /// Inside any other fence (illustration idiom): blanked.
    Other,
}

/// Extract the TOML a config markdown document carries.
///
/// - `toml`-tagged fence interiors pass through verbatim.
/// - Every other line (prose, fence markers, other fences'
///   interiors) is blanked rather than removed, so the result has
///   the source's line count and any parse diagnostic points at
///   the real line.
/// - The tag must be exactly `toml` (` ```toml `); a fence tagged
///   otherwise or untagged is illustration and is ignored.
/// - An unclosed fence is an error naming its opening line.
pub fn md_to_toml(content: &str) -> Result<String, String> {
    let mut state = Fence::None;
    let mut opened_at = 0usize;
    let mut out = String::with_capacity(content.len());
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        match state {
            Fence::None => {
                if let Some(info) = trimmed.strip_prefix("```") {
                    state = if info.trim() == "toml" {
                        Fence::Toml
                    } else {
                        Fence::Other
                    };
                    opened_at = idx + 1;
                }
            }
            Fence::Toml => {
                if is_fence_close(trimmed) {
                    state = Fence::None;
                } else {
                    out.push_str(line);
                }
            }
            Fence::Other => {
                if is_fence_close(trimmed) {
                    state = Fence::None;
                }
            }
        }
        out.push('\n');
    }
    if matches!(state, Fence::None) {
        Ok(out)
    } else {
        Err(format!("unclosed fence opened at line {opened_at}"))
    }
}

/// True when a (trim_start'ed) line closes a fence: backticks with
/// nothing but whitespace after them.
fn is_fence_close(trimmed: &str) -> bool {
    trimmed
        .strip_prefix("```")
        .is_some_and(|rest| rest.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fences_concatenate_in_document_order() {
        let doc = "# Title\n\nprose\n```toml\na = 1\n```\nmore prose\n```toml\nb = 2\n```\n";
        let toml = md_to_toml(doc).unwrap();
        let kept: Vec<&str> = toml.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(kept, ["a = 1", "b = 2"]);
    }

    #[test]
    fn line_count_is_preserved() {
        let doc = "prose\n```toml\nk = \"v\"\n```\ntrailing prose\n";
        let toml = md_to_toml(doc).unwrap();
        assert_eq!(toml.lines().count(), doc.lines().count());
        // The kept line sits at its source line number.
        assert_eq!(toml.lines().nth(2), Some("k = \"v\""));
    }

    #[test]
    fn prose_never_reaches_the_parser() {
        // A prose line containing `=` or `[..]` outside a fence
        // must not survive the filter.
        let doc = "prose with spurious = sign\n[not-a-section] in prose\n```toml\nk = \"v\"\n```\n";
        let toml = md_to_toml(doc).unwrap();
        assert!(!toml.contains("spurious"));
        assert!(!toml.contains("not-a-section"));
        assert!(toml.contains("k = \"v\""));
    }

    #[test]
    fn non_toml_fences_are_illustration() {
        let doc = "```\nignored = \"yes\"\n```\n```sh\nexport X=1\n```\n```toml\nk = \"v\"\n```\n";
        let toml = md_to_toml(doc).unwrap();
        assert!(!toml.contains("ignored"));
        assert!(!toml.contains("export"));
        assert!(toml.contains("k = \"v\""));
    }

    #[test]
    fn unclosed_fence_errors() {
        let doc = "prose\n```toml\nk = \"v\"\n";
        let err = md_to_toml(doc).unwrap_err();
        assert!(err.contains("line 2"), "unexpected error: {err}");
    }
}
