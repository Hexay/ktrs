//! Character classes of the two JFlex specs. Non-ASCII tables come from the generated lexers'
//! char maps (see `tools/gen_unicode_tables.py`), so they match the pinned JFlex Unicode version.

use crate::unicode_tables::{KDOC_JLETTER, KOTLIN_DIGIT, KOTLIN_LETTER};

fn in_table(table: &[(u32, u32)], c: char) -> bool {
    let c = c as u32;
    table
        .binary_search_by(|&(lo, hi)| {
            if hi < c {
                std::cmp::Ordering::Less
            } else if lo > c {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// `LETTER = [:letter:]|_` in `Kotlin.flex`.
pub(crate) fn is_kotlin_letter(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_alphabetic() || c == '_'
    } else {
        in_table(KOTLIN_LETTER, c)
    }
}

/// `IDENTIFIER_PART = [:digit:]|{LETTER}` in `Kotlin.flex`.
pub(crate) fn is_kotlin_identifier_part(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_alphanumeric() || c == '_'
    } else {
        in_table(KOTLIN_LETTER, c) || in_table(KOTLIN_DIGIT, c)
    }
}

/// `LETTER = [:jletter:]` in `KDoc.flex`.
pub(crate) fn is_kdoc_letter(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_alphabetic() || c == '_' || c == '$'
    } else {
        in_table(KDOC_JLETTER, c)
    }
}

/// `java.lang.Character.isWhitespace(char)`; supplementary chars are never whitespace.
pub(crate) fn is_java_whitespace(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t'..='\r'
            | '\u{1c}'..='\u{1f}'
            | '\u{1680}'
            | '\u{2000}'..='\u{2006}'
            | '\u{2008}'..='\u{200a}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{205f}'
            | '\u{3000}'
    )
}
