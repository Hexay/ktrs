//! Java `String.length()` for Rust strings.
//!
//! Positions in this port are UTF-8 byte offsets (the syntax tree's), but every width and column that gjf
//! compares against `maxWidth` is a count of UTF-16 code units, so width sites convert with
//! [`utf16_len`]. Position arithmetic (`getPosition() + length()`) uses byte lengths instead.

pub fn utf16_len(s: &str) -> i32 {
    if s.is_ascii() {
        return s.len() as i32;
    }
    s.chars().map(|c| c.len_utf16() as i32).sum()
}
