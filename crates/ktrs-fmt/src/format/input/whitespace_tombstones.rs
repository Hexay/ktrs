//! Port of ktfmt's `WhitespaceTombstones.kt`.
//!
//! gjf removes trailing spaces when it emits formatted code, which breaks multiline string
//! literals. The last trailing space of each such line is replaced by a tombstone before
//! formatting and restored afterwards.

pub const SPACE_TOMBSTONE: char = '\u{3}';

/// `String.indexOfWhitespaceTombstone()`: byte index, or -1.
pub fn index_of_whitespace_tombstone(s: &str) -> i32 {
    s.find(SPACE_TOMBSTONE).map_or(-1, |i| i as i32)
}

/// `Pattern.compile(" ($)", MULTILINE).matcher(s).replaceAll(tombstone)`: a space directly before
/// a Java line terminator or the end of input.
pub fn replace_trailing_whitespace_with_tombstone(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        let at_line_end = is_line_end(chars.peek().copied());
        out.push(if c == ' ' && at_line_end {
            SPACE_TOMBSTONE
        } else {
            c
        });
    }
    out
}

/// Whether [`replace_trailing_whitespace_with_tombstone`] would change `s`.
pub fn has_trailing_whitespace(s: &str) -> bool {
    s.match_indices(' ').any(|(i, _)| is_line_end(s[i + 1..].chars().next()))
}

fn is_line_end(next: Option<char>) -> bool {
    matches!(next, None | Some('\n' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}'))
}

pub fn replace_tombstone_with_trailing_whitespace(s: &str) -> String {
    s.replace(SPACE_TOMBSTONE, " ")
}
