//! Port of `Escaping.kt`. Indices are UTF-16 offsets, as in upstream.

use super::kstring::{KStr, ks, to_string, w};

const SLASH_STAR_ESCAPE: &str = "\u{0004}\u{0005}";

const STAR_SLASH_ESCAPE: &str = "\u{0005}\u{0004}";

/// `s.indexOfAny(listOf(SLASH_STAR_ESCAPE, STAR_SLASH_ESCAPE))` as a UTF-16 index, or -1.
pub fn index_of_comment_escape_sequences(s: &str) -> i32 {
    let s = ks(s);
    (0..s.len())
        .find(|&i| {
            s.starts_with_at(w!("\u{0004}\u{0005}"), i) || s.starts_with_at(w!("\u{0005}\u{0004}"), i)
        })
        .map_or(-1, |i| i as i32)
}

/// kotlin-compiler's KDoc lexer doesn't correctly handle nested slash-star comments, so we escape
/// them into tombstones, format, then unescape.
pub fn escape_kdoc(s: &str) -> String {
    let u = ks(s);
    let start_marker_index = u.index_of(w!("/*"), 0);
    let end_marker_index = u.last_index_of(w!("*/"));

    if start_marker_index == -1 || end_marker_index == -1 {
        panic!("KDoc with no /** and/or */");
    }
    let start = start_marker_index as usize + 3;
    let end = end_marker_index as usize;

    // Throws StringIndexOutOfBoundsException upstream when the markers overlap (e.g. "/*/").
    let middle = u[start..end]
        .replace_seq(w!("/*"), &ks(SLASH_STAR_ESCAPE), false)
        .replace_seq(w!("*/"), &ks(STAR_SLASH_ESCAPE), false);
    to_string(&[&u[..start], &middle[..], &u[end..]].concat())
}

/// See [escape_kdoc].
pub fn unescape_kdoc(s: &str) -> String {
    s.replace(SLASH_STAR_ESCAPE, "/*").replace(STAR_SLASH_ESCAPE, "*/")
}
