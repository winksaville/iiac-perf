//! Wrapping a refusal for the terminal.
//!
//! A message the reader has to act on is wrapped here rather than left to the terminal. A terminal
//! breaks wherever the column runs out, mid-word and mid-path, and on a wide window it leaves a
//! paragraph as one line nobody scans. Wrapping at the message's own width makes it read the same
//! in every window, and a refusal is read once, under pressure, so that matters more than for
//! prose a reader is browsing.

/// The column a refusal wraps at. An 80-column terminal is the narrow case worth reading in. The
/// caller's `error: <key>: ` prefix rides on the first line, so that line alone runs a little past
/// this, which is why a message's first line is a short summary and the detail follows it.
pub const WIDTH: usize = 80;

/// Wrap `text` to `width` columns on whitespace, keeping the line breaks it already has so a
/// message can still give a sentence a line of its own. A word longer than `width`, a path or a
/// command, takes a line rather than being broken, since a broken path cannot be copied.
pub fn wrap(text: &str, width: usize) -> String {
    let mut out = String::new();
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let mut col = 0;
        for word in line.split_whitespace() {
            let len = word.chars().count();
            if col == 0 {
                out.push_str(word);
                col = len;
            } else if col + 1 + len <= width {
                out.push(' ');
                out.push_str(word);
                col += 1 + len;
            } else {
                out.push('\n');
                out.push_str(word);
                col = len;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapping_breaks_on_whitespace_and_keeps_the_breaks_it_was_given() {
        assert_eq!(wrap("a b c", 5), "a b c");
        assert_eq!(wrap("a b c", 3), "a b\nc");
        // An explicit break survives, so a message keeps the paragraphs it wrote.
        assert_eq!(wrap("a\nb c", 80), "a\nb c");
    }

    #[test]
    fn a_word_past_the_width_takes_a_line_rather_than_breaking() {
        assert_eq!(
            wrap("see /a/very/long/path now", 10),
            "see\n/a/very/long/path\nnow"
        );
    }

    #[test]
    fn every_wrapped_line_fits_but_for_a_word_that_cannot() {
        let text = "no [freq] steady state is declared, so a pin would have no way home. \
                    `iiac-perf-dev setup-freq` writes one to ~/.config/iiac-perf/config.toml \
                    (or config.md, whichever that directory holds) from the live state.";
        for line in wrap(text, WIDTH).lines() {
            let long_word = line.split_whitespace().count() == 1;
            assert!(
                line.chars().count() <= WIDTH || long_word,
                "{} cols: {line}",
                line.chars().count()
            );
        }
    }
}
