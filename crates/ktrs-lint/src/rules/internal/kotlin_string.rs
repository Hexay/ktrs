//! Kotlin stdlib `Char`/`String` functions with the JVM's semantics (UTF-16 units, Java's character
//! classes), where they differ from Rust's.

/// `Char.isWhitespace()`: Java `isWhitespace` or `isSpaceChar` (so U+00A0 yes, U+0085 no).
pub fn is_whitespace(c: char) -> bool {
    matches!(c, '\t'..='\r' | '\u{1c}'..='\u{1f}' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}')
        || matches!(c, '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}')
}

/// `CharSequence.isBlank()`.
pub fn is_blank(s: &str) -> bool {
    s.chars().all(is_whitespace)
}

/// `String.trim()`.
pub fn trim(s: &str) -> &str {
    s.trim_matches(is_whitespace)
}

/// `Char.isLetterOrDigit()` of one UTF-16 unit: general category `L*` or `Nd`; surrogates never are.
pub fn is_letter_or_digit(unit: u16) -> bool {
    char::from_u32(u32::from(unit)).is_some_and(|c| {
        static LETTER_OR_DIGIT: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new(r"^[\p{L}\p{Nd}]$").unwrap());
        c.is_ascii_alphanumeric() || (!c.is_ascii() && LETTER_OR_DIGIT.is_match(c.encode_utf8(&mut [0; 4])))
    })
}

/// `String.substringAfterLast(delimiter)` (the whole string when absent).
pub fn substring_after_last<'a>(s: &'a str, delimiter: &str) -> &'a str {
    s.rfind(delimiter).map_or(s, |i| &s[i + delimiter.len()..])
}

/// `String.substringBeforeLast(delimiter)` (the whole string when absent).
pub fn substring_before_last<'a>(s: &'a str, delimiter: &str) -> &'a str {
    s.rfind(delimiter).map_or(s, |i| &s[..i])
}

/// `String.substringBefore(delimiter)` (the whole string when absent).
pub fn substring_before<'a>(s: &'a str, delimiter: &str) -> &'a str {
    s.find(delimiter).map_or(s, |i| &s[..i])
}

/// `String.substringAfter(delimiter)` (the whole string when absent).
pub fn substring_after<'a>(s: &'a str, delimiter: &str) -> &'a str {
    s.find(delimiter).map_or(s, |i| &s[i + delimiter.len()..])
}

/// `String.removeSurrounding(prefix, suffix)`: only when both are there without overlapping.
pub fn remove_surrounding<'a>(s: &'a str, prefix: &str, suffix: &str) -> &'a str {
    if s.len() >= prefix.len() + suffix.len() && s.starts_with(prefix) && s.ends_with(suffix) {
        &s[prefix.len()..s.len() - suffix.len()]
    } else {
        s
    }
}

/// `String.replaceFirstChar { it.uppercaseChar() }`: Java's simple (one-to-one) upper case mapping of the
/// first UTF-16 unit.
pub fn replace_first_char_uppercase_char(s: &str) -> String {
    let mut chars = s.chars();
    let Some(first) = chars.next() else { return String::new() };
    let mut upper = first.to_uppercase();
    match (first.len_utf16(), upper.next(), upper.next()) {
        (1, Some(u), None) => format!("{u}{}", chars.as_str()),
        _ => s.to_owned(),
    }
}

/// `String.replaceFirstChar { it.uppercase() }`: the full upper case mapping of the first UTF-16 unit.
pub fn replace_first_char_uppercase(s: &str) -> String {
    let mut chars = s.chars();
    let Some(first) = chars.next() else { return String::new() };
    if first.len_utf16() != 1 {
        return s.to_owned();
    }
    format!("{}{}", first.to_uppercase(), chars.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn like_kotlin() {
        assert!(is_blank(" \t\u{a0}\u{1f}"));
        assert!(!is_blank("\u{85}"));
        assert_eq!(trim("\u{3000} a b\n"), "a b");
        assert!(is_letter_or_digit(u16::from(b'x')) && is_letter_or_digit(0x00e9) && !is_letter_or_digit(0xd83d));
        assert!(!is_letter_or_digit(0x2167)); // ROMAN NUMERAL EIGHT is Nl
        assert_eq!(remove_surrounding("`", "`", "`"), "`");
        assert_eq!(remove_surrounding("`a`", "`", "`"), "a");
        assert_eq!(replace_first_char_uppercase_char("ßa"), "ßa");
        assert_eq!(replace_first_char_uppercase("ßa"), "SSa");
        assert_eq!(substring_before("a.b.c", "."), "a");
        assert_eq!(substring_after_last("abc", "/"), "abc");
    }
}
