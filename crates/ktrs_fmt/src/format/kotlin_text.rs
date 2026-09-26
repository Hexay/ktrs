//! Kotlin stdlib / IntelliJ string semantics the format package relies on (`Char.isWhitespace`,
//! `isBlank`, `trimStart`, `lines()`, UTF-16 `compareTo`, `StringUtilRt.convertLineSeparators`).

use std::cmp::Ordering;

/// Kotlin `Char.isWhitespace()`: Java `isWhitespace` (incl. U+001C..U+001F) or `isSpaceChar` (Zs, Zl, Zp).
pub fn is_kotlin_whitespace(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{0B}' | '\u{0C}' | '\r' | '\u{1C}'..='\u{1F}' | ' ' | '\u{A0}' | '\u{1680}'
            | '\u{2000}'..='\u{200A}' | '\u{2028}' | '\u{2029}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
    )
}

pub fn is_blank(s: &str) -> bool {
    s.chars().all(is_kotlin_whitespace)
}

pub fn trim_start(s: &str) -> &str {
    s.trim_start_matches(is_kotlin_whitespace)
}

/// Kotlin `CharSequence.lines()`: split on `\r\n`, `\n` or `\r`; a trailing terminator yields a final "".
pub fn lines(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            b'\r' => {
                out.push(&s[start..i]);
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    i += 1;
                }
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(&s[start..]);
    out
}

/// `lines(s)` plus each line's start, answering `lines(&s[..offset])` queries without re-splitting.
pub struct LineIndex<'s> {
    text: &'s str,
    pub lines: Vec<&'s str>,
    starts: Vec<usize>,
}

impl<'s> LineIndex<'s> {
    pub fn new(text: &'s str) -> LineIndex<'s> {
        let lines = lines(text);
        let starts = lines.iter().map(|l| l.as_ptr() as usize - text.as_ptr() as usize).collect();
        LineIndex { text, lines, starts }
    }

    /// `lines(&s[..offset]).len() - 1`.
    pub fn line_of(&self, offset: usize) -> usize {
        self.starts.partition_point(|&start| start <= offset) - 1 + usize::from(self.splits_crlf(offset))
    }

    /// `lines(&s[..offset]).last()`.
    pub fn line_prefix(&self, offset: usize) -> &'s str {
        if self.splits_crlf(offset) {
            return "";
        }
        &self.text[self.starts[self.line_of(offset)]..offset]
    }

    /// A prefix ending between `\r` and `\n` ends with a terminator the whole text doesn't have there.
    fn splits_crlf(&self, offset: usize) -> bool {
        let bytes = self.text.as_bytes();
        offset > 0 && bytes[offset - 1] == b'\r' && bytes.get(offset) == Some(&b'\n')
    }
}

/// `String.compareTo`: lexicographic over UTF-16 units.
pub fn compare_utf16(a: &str, b: &str) -> Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

/// `StringUtilRt.convertLineSeparators(text)`: `\r\n` and `\r` become `\n`.
pub fn convert_line_separators(text: &str) -> String {
    convert_line_separators_to(text, "\n")
}

/// `StringUtilRt.convertLineSeparators(text, newSeparator)`: every `\r\n`, `\r` or `\n` becomes `new_separator`.
pub fn convert_line_separators_to(text: &str, new_separator: &str) -> String {
    if new_separator == "\n" && !text.contains('\r') {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push_str(new_separator);
            }
            '\n' => out.push_str(new_separator),
            _ => out.push(c),
        }
    }
    out
}
