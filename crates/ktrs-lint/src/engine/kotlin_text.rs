//! Kotlin stdlib string functions whose exact behavior shows in the snippets the engine parses.

/// `CharSequence.lines()`: split on `\r\n`, `\n` and `\r`.
fn lines(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' => {
                out.push(&s[start..i]);
                i += if bytes.get(i + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                };
                start = i;
            }
            b'\n' => {
                out.push(&s[start..i]);
                i += 1;
                start = i;
            }
            _ => i += 1,
        }
    }
    out.push(&s[start..]);
    out
}

fn is_blank(s: &str) -> bool {
    s.chars().all(char::is_whitespace)
}

/// `reindent`: drops a blank first and last line, cuts the others (a `None` cut keeps the line).
fn reindent(lines: &[&str], cut: impl Fn(&str) -> Option<String>) -> String {
    let last_index = lines.len().saturating_sub(1);
    lines
        .iter()
        .enumerate()
        .filter(|&(index, value)| !((index == 0 || index == last_index) && is_blank(value)))
        .map(|(_, value)| cut(value).unwrap_or_else(|| (*value).to_owned()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `trimMargin()` with the `|` margin; lines without the margin are kept as they are.
pub(crate) fn trim_margin(s: &str) -> String {
    reindent(&lines(s), |line| {
        let first_non_whitespace_index = line
            .char_indices()
            .find(|(_, c)| !c.is_whitespace())
            .map(|(i, _)| i)?;
        line[first_non_whitespace_index..]
            .strip_prefix('|')
            .map(str::to_owned)
    })
}

/// `trimIndent()`: removes the smallest indent of the non-blank lines.
pub(crate) fn trim_indent(s: &str) -> String {
    let lines = lines(s);
    let min_common_indent = lines
        .iter()
        .filter(|l| !is_blank(l))
        .map(|l| {
            l.char_indices()
                .find(|(_, c)| !c.is_whitespace())
                .map_or(l.len(), |(i, _)| l[..i].chars().count())
        })
        .min()
        .unwrap_or(0);
    reindent(&lines, |line| {
        Some(line.chars().skip(min_common_indent).collect())
    })
}

#[cfg(test)]
mod tests {
    use super::{trim_indent, trim_margin};

    #[test]
    fn like_kotlin() {
        assert_eq!(
            trim_margin("\n    |  a\n    |b\n  || c\n    "),
            "  a\nb\n| c"
        );
        assert_eq!(trim_margin("\n    |a\nno margin\n    "), "a\nno margin");
        assert_eq!(trim_indent("\n    x\n      y\n    "), "x\n  y");
    }
}
