//! Longest-match scanners for the macros of `Kotlin.flex`. Each takes a byte offset at a char
//! boundary and returns the end offset of the longest match (or `None` when nothing matches).

use crate::chars::{is_kotlin_identifier_part, is_kotlin_letter};

pub(crate) fn char_at(text: &str, pos: usize) -> Option<char> {
    text.get(pos..)?.chars().next()
}

fn byte(text: &str, pos: usize) -> u8 {
    text.as_bytes().get(pos).copied().unwrap_or(0)
}

fn skip_bytes(text: &str, mut pos: usize, pred: impl Fn(u8) -> bool) -> usize {
    let bytes = text.as_bytes();
    while pos < bytes.len() && pred(bytes[pos]) {
        pos += 1;
    }
    pos
}

/// `[A-Za-z0-9_]`; false for every non-ASCII byte.
const ASCII_IDENTIFIER_PART: [bool; 256] = {
    let mut set = [false; 256];
    let mut b = 0;
    while b < 128 {
        set[b] = (b as u8).is_ascii_alphanumeric() || b == b'_' as usize;
        b += 1;
    }
    set
};

/// `{IDENTIFIER} = {PLAIN_IDENTIFIER}|{ESCAPED_IDENTIFIER}`.
pub(crate) fn identifier(text: &str, pos: usize) -> Option<usize> {
    let b0 = byte(text, pos);
    if b0 == b'`' {
        return escaped_identifier(text, pos);
    }
    let mut end = if b0.is_ascii() {
        if !(b0.is_ascii_alphabetic() || b0 == b'_') {
            return None;
        }
        pos + 1
    } else {
        let first = char_at(text, pos)?;
        if !is_kotlin_letter(first) {
            return None;
        }
        pos + first.len_utf8()
    };
    let bytes = text.as_bytes();
    while end < bytes.len() {
        let b = bytes[end];
        if ASCII_IDENTIFIER_PART[b as usize] {
            end += 1;
        } else if b.is_ascii() {
            break;
        } else {
            let c = char_at(text, end).unwrap();
            if !is_kotlin_identifier_part(c) {
                break;
            }
            end += c.len_utf8();
        }
    }
    Some(end)
}

/// `` ESCAPED_IDENTIFIER = `[^`\n]+` ``.
fn escaped_identifier(text: &str, pos: usize) -> Option<usize> {
    let close = skip_bytes(text, pos + 1, |b| b != b'`' && b != b'\n');
    (close > pos + 1 && byte(text, close) == b'`').then_some(close + 1)
}

/// `({WHITE_SPACE_CHAR})+` with `WHITE_SPACE_CHAR=[\ \n\t\f]`.
pub(crate) fn white_space(text: &str, pos: usize) -> usize {
    skip_bytes(text, pos, |b| matches!(b, b' ' | b'\n' | b'\t' | b'\x0c'))
}

/// `[^\n]*`.
pub(crate) fn rest_of_line(text: &str, pos: usize) -> usize {
    skip_bytes(text, pos, |b| b != b'\n')
}

fn digits_or_underscores(text: &str, pos: usize) -> usize {
    skip_bytes(text, pos, |b| b.is_ascii_digit() || b == b'_')
}

/// `TYPED_INTEGER_SUFFIX = [Uu]? [Ll]?`.
fn integer_suffix(text: &str, mut pos: usize) -> usize {
    if matches!(byte(text, pos), b'u' | b'U') {
        pos += 1;
    }
    if matches!(byte(text, pos), b'l' | b'L') {
        pos += 1;
    }
    pos
}

/// `{INTEGER_LITERAL}` at an ASCII digit.
pub(crate) fn integer_literal(text: &str, pos: usize) -> usize {
    let mut best = integer_suffix(text, digits_or_underscores(text, pos + 1));
    if byte(text, pos) == b'0' {
        let body_end = match byte(text, pos + 1) {
            b'x' | b'X' => Some(skip_bytes(text, pos + 2, |b| {
                b.is_ascii_hexdigit() || b == b'_'
            })),
            b'b' | b'B' => Some(digits_or_underscores(text, pos + 2)),
            _ => None,
        };
        if let Some(body_end) = body_end {
            best = best.max(integer_suffix(text, body_end));
        }
    }
    best
}

/// `EXPONENT_PART = [Ee][+-]? {DIGIT_OR_UNDERSCORE}*`.
fn exponent_part(text: &str, pos: usize) -> Option<usize> {
    if !matches!(byte(text, pos), b'e' | b'E') {
        return None;
    }
    let sign = pos + 1 + usize::from(matches!(byte(text, pos + 1), b'+' | b'-'));
    Some(digits_or_underscores(text, sign))
}

fn float_suffix(text: &str, pos: usize) -> usize {
    pos + usize::from(matches!(byte(text, pos), b'f' | b'F'))
}

/// `{DOUBLE_LITERAL}` at an ASCII digit or `.`.
pub(crate) fn double_literal(text: &str, pos: usize) -> Option<usize> {
    let starts_with_digits = byte(text, pos).is_ascii_digit();
    let int_end = if starts_with_digits {
        digits_or_underscores(text, pos + 1)
    } else {
        pos
    };

    // {DIGITS}? "." {DIGITS} {EXPONENT_PART}? [Ff]?
    let fraction =
        (byte(text, int_end) == b'.' && byte(text, int_end + 1).is_ascii_digit()).then(|| {
            let frac_end = digits_or_underscores(text, int_end + 2);
            float_suffix(text, exponent_part(text, frac_end).unwrap_or(frac_end))
        });
    // {DIGITS} ({EXPONENT_PART} [Ff]? | [Ff])
    let exponent = starts_with_digits
        .then(|| match exponent_part(text, int_end) {
            Some(exp_end) => Some(float_suffix(text, exp_end)),
            None => matches!(byte(text, int_end), b'f' | b'F').then_some(int_end + 1),
        })
        .flatten();
    fraction.max(exponent)
}

/// `CHARACTER_LITERAL="'"([^\\\'\n]|{ESCAPE_SEQUENCE})*("'"|\\)?`.
pub(crate) fn character_literal(text: &str, pos: usize) -> usize {
    let bytes = text.as_bytes();
    let mut end = pos + 1;
    while end < bytes.len() {
        match bytes[end] {
            b'\'' => return end + 1,
            b'\n' => return end,
            b'\\' => match char_at(text, end + 1) {
                Some(c) if c != '\n' => end += 1 + c.len_utf8(),
                _ => return end + 1,
            },
            _ => end += 1,
        }
    }
    end
}

/// `ESCAPE_SEQUENCE=\\(u{HEX_DIGIT}{4}|[^\n])` at a backslash.
pub(crate) fn escape_sequence(text: &str, pos: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.get(pos + 1) == Some(&b'u')
        && bytes
            .get(pos + 2..pos + 6)
            .is_some_and(|h| h.iter().all(u8::is_ascii_hexdigit))
    {
        return Some(pos + 6);
    }
    char_at(text, pos + 1)
        .filter(|&c| c != '\n')
        .map(|c| pos + 1 + c.len_utf8())
}

/// `REGULAR_STRING_PART=[^\\\"\n\$]+`.
pub(crate) fn regular_string_part(text: &str, pos: usize) -> usize {
    skip_bytes(text, pos, |b| !matches!(b, b'\\' | b'"' | b'\n' | b'$'))
}

pub(crate) fn run_of(text: &str, pos: usize, b: u8) -> usize {
    skip_bytes(text, pos, |x| x == b)
}
