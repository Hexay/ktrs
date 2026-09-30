//! Port of `CommentsHelper.java`.

use super::input::Tok;
use super::java_identifier_tables::{JAVA_IDENTIFIER_PART, JAVA_IDENTIFIER_START};

/// Rewrites comments; ktfmt's implementation is `KDocCommentsHelper`.
pub trait CommentsHelper {
    /// Returns the comment text to emit when starting at `column0` (UTF-16 units).
    fn rewrite(&self, tok: &Tok<'_>, max_width: i32, column0: i32) -> String;
}

/// `/* name = */` becomes `/* name= */`.
pub fn reformat_parameter_comment(tok: &Tok<'_>) -> Option<String> {
    if !tok.is_slash_star_comment() {
        return None;
    }
    let name = match_parameter_comment(tok.get_original_text())?;
    Some(format!("/* {name}= */"))
}

/// Full match of `PARAMETER_COMMENT`, returning group 1:
/// `/\*\s*(\p{javaJavaIdentifierStart}\p{javaJavaIdentifierPart}*(\Q...\E)?)\s*=\s*\*/`.
/// No backtracking is needed: identifier chars never overlap `.`, `\s` or `=`.
fn match_parameter_comment(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("/*")?.trim_start_matches(is_regex_space);
    let first = rest.chars().next()?;
    if !is_java_identifier_start(first) {
        return None;
    }
    let ident_end = rest
        .char_indices()
        .skip(1)
        .find(|&(_, c)| !is_java_identifier_part(c))
        .map_or(rest.len(), |(i, _)| i);
    let name_end = if rest[ident_end..].starts_with("...") {
        ident_end + 3
    } else {
        ident_end
    };
    let tail = rest[name_end..]
        .trim_start_matches(is_regex_space)
        .strip_prefix('=')?;
    if tail.trim_start_matches(is_regex_space) != "*/" {
        return None;
    }
    Some(&rest[..name_end])
}

/// Java regex `\s` without `UNICODE_CHARACTER_CLASS`.
fn is_regex_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{0b}' | '\u{0c}' | '\r')
}

fn in_table(table: &[(u32, u32)], c: char) -> bool {
    let c = c as u32;
    let at = table.partition_point(|&(_, hi)| hi < c);
    table.get(at).is_some_and(|&(lo, _)| lo <= c)
}

fn is_java_identifier_start(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_alphabetic() || c == '_' || c == '$'
    } else {
        in_table(JAVA_IDENTIFIER_START, c)
    }
}

fn is_java_identifier_part(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_alphanumeric()
            || matches!(c, '_' | '$' | '\0'..='\u{08}' | '\u{0e}'..='\u{1b}' | '\u{7f}')
    } else {
        in_table(JAVA_IDENTIFIER_PART, c)
    }
}
