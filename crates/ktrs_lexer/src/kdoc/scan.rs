//! Longest-match scanners for the macros of `KDoc.flex`, returning the end offset of the match.
//! Rules with trailing context (`r1 / r2`) return `(r1r2 end, r1 end)`: JFlex picks the rule by the
//! whole `r1r2` length but consumes only `r1`.

use crate::chars::{is_java_whitespace, is_kdoc_letter};

fn byte(text: &str, pos: usize) -> u8 {
    text.as_bytes().get(pos).copied().unwrap_or(0)
}

fn run(text: &str, mut pos: usize, pred: impl Fn(u8) -> bool) -> usize {
    let bytes = text.as_bytes();
    while pos < bytes.len() && pred(bytes[pos]) {
        pos += 1;
    }
    pos
}

fn non_empty(start: usize, end: usize) -> Option<usize> {
    (end > start).then_some(end)
}

pub(crate) fn any_char(text: &str, pos: usize) -> Option<usize> {
    text.get(pos..)?.chars().next().map(|c| pos + c.len_utf8())
}

/// `{LINE_BREAK_CHAR} = [\r\n]`.
pub(crate) fn line_break(text: &str, pos: usize) -> Option<usize> {
    matches!(byte(text, pos), b'\r' | b'\n').then_some(pos + 1)
}

fn is_white_space_char(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\x0c')
}

/// `{WHITE_SPACE_CHAR}+ = [\ \t\f]+`.
pub(crate) fn white_space(text: &str, pos: usize) -> Option<usize> {
    non_empty(pos, run(text, pos, is_white_space_char))
}

/// `"*"+`.
pub(crate) fn asterisks(text: &str, pos: usize) -> Option<usize> {
    non_empty(pos, run(text, pos, |b| b == b'*'))
}

/// `"*"+ "/"`.
pub(crate) fn asterisks_slash(text: &str, pos: usize) -> Option<usize> {
    asterisks(text, pos)
        .filter(|&end| byte(text, end) == b'/')
        .map(|end| end + 1)
}

/// `{BACKTICK_STRING} = "`"+`.
pub(crate) fn backtick_string(text: &str, pos: usize) -> Option<usize> {
    non_empty(pos, run(text, pos, |b| b == b'`'))
}

fn plain_identifier(text: &str, pos: usize) -> Option<usize> {
    let mut chars = text.get(pos..)?.char_indices();
    let (_, first) = chars.next()?;
    if !is_kdoc_letter(first) {
        return None;
    }
    let end = chars
        .find(|&(_, c)| !(is_kdoc_letter(c) || c.is_ascii_digit()))
        .map_or(text.len(), |(i, _)| pos + i);
    Some(end)
}

/// `"@"{PLAIN_IDENTIFIER}`.
pub(crate) fn tag_name(text: &str, pos: usize) -> Option<usize> {
    if byte(text, pos) != b'@' {
        return None;
    }
    plain_identifier(text, pos + 1)
}

/// `` IDENTIFIER = {PLAIN_IDENTIFIER} | `[^`\n]+` ``.
fn identifier(text: &str, pos: usize) -> Option<usize> {
    if byte(text, pos) != b'`' {
        return plain_identifier(text, pos);
    }
    let close = run(text, pos + 1, |b| b != b'`' && b != b'\n');
    (close > pos + 1 && byte(text, close) == b'`').then_some(close + 1)
}

/// `QUALIFIED_NAME = {IDENTIFIER} ([\.] {IDENTIFIER}?)*`.
pub(crate) fn qualified_name(text: &str, pos: usize) -> Option<usize> {
    let mut end = identifier(text, pos)?;
    while byte(text, end) == b'.' {
        end = identifier(text, end + 1).unwrap_or(end + 1);
    }
    Some(end)
}

/// `CODE_LINK=\[{QUALIFIED_NAME}\]`.
pub(crate) fn code_link(text: &str, pos: usize) -> Option<usize> {
    if byte(text, pos) != b'[' {
        return None;
    }
    let end = qualified_name(text, pos + 1)?;
    (byte(text, end) == b']').then_some(end + 1)
}

/// `{CODE_LINK} / [^\(\[]`.
pub(crate) fn code_link_not_followed_by_link(text: &str, pos: usize) -> Option<(usize, usize)> {
    let end = code_link(text, pos)?;
    let after = any_char(text, end).filter(|_| !matches!(byte(text, end), b'(' | b'['))?;
    Some((after, end))
}

/// `{ESCAPED_CHARS} = "\\"[!#$%&'()*+,-./:;<=>?@_`{|}~\"\^\[\\\]]`.
pub(crate) fn escaped_chars(text: &str, pos: usize) -> Option<usize> {
    let escapable = br##"!#$%&'()*+,-./:;<=>?@_`{|}~"^[\]"##;
    (byte(text, pos) == b'\\' && escapable.contains(&byte(text, pos + 1))).then_some(pos + 2)
}

/// `{CODE_FENCE_START} / {LINE_BREAK_CHAR}`.
pub(crate) fn code_fence_start(text: &str, pos: usize) -> Option<(usize, usize)> {
    let fence = byte(text, pos);
    if fence != b'`' && fence != b'~' {
        return None;
    }
    let fence_end = run(text, pos, |b| b == fence);
    if fence_end - pos < 3 {
        return None;
    }
    let end = run(text, fence_end, |b| {
        b != b'\r' && b != b'\n' && (fence == b'~' || b != b'`')
    });
    line_break(text, end).map(|after| (after, end))
}

/// `{CODE_FENCE_END} / [ \t\f]* [\n]`.
pub(crate) fn code_fence_end(text: &str, pos: usize) -> Option<(usize, usize)> {
    let fence = byte(text, pos);
    if fence != b'`' && fence != b'~' {
        return None;
    }
    let end = run(text, pos, |b| b == fence);
    let spaces_end = run(text, end, is_white_space_char);
    (byte(text, spaces_end) == b'\n').then_some((spaces_end + 1, end))
}

#[derive(PartialEq, Eq)]
enum LinePos {
    AfterNewline,
    AfterLeadingAsterisk,
    InContent,
}

/// `hasMatchingCloseFence`: whether a run of exactly `length` `c`s closes the span opened at `pos`.
pub(crate) fn has_matching_close_fence(text: &str, mut pos: usize, c: u8, length: usize) -> bool {
    let bytes = text.as_bytes();
    let mut line_pos = LinePos::InContent;
    while pos < bytes.len() {
        let ch = text[pos..].chars().next().unwrap_or('\0');
        if ch == '\n' {
            if line_pos != LinePos::InContent {
                return false;
            }
            line_pos = LinePos::AfterNewline;
            pos += 1;
        } else if is_java_whitespace(ch) {
            pos += ch.len_utf8();
        } else if line_pos == LinePos::AfterNewline && ch == '*' {
            pos = run(text, pos, |b| b == b'*');
            if byte(text, pos) == b'/' {
                return false;
            }
            line_pos = LinePos::AfterLeadingAsterisk;
        } else if ch == c as char {
            let fence_start = pos;
            pos = run(text, pos, |b| b == c);
            if pos - fence_start == length {
                return true;
            }
            line_pos = LinePos::InContent;
        } else {
            line_pos = LinePos::InContent;
            pos += ch.len_utf8();
        }
    }
    false
}
