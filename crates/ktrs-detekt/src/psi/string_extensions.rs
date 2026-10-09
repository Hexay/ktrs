//! `StringExtensions.kt`: what a too long line may end with.

use std::sync::OnceLock;

use regex::Regex;

use crate::kotlin::is_url_with_uri;

/// Java `\s`.
fn is_java_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{b}' | '\u{c}' | '\r')
}

/// `String.lastArgumentMatchesUrl()`.
pub fn last_argument_matches_url(line: &str) -> bool {
    let last_argument = line.trim_end().rsplit(is_java_space).next().unwrap_or_default();
    is_url_with_uri(last_argument)
}

// Gotcha: the two patterns below are the upstream ones rewritten without lookahead and backreference (the
// `regex` crate has neither): `(?:(?!\s).)` is a class, and the `(["'])...\1` title is spelled out per quote.
// `D` = Java's `.`, `S` = Java's `\s`.
const D: &str = r"[^\n\r\x{85}\x{2028}\x{2029}]";
const S: &str = r"[ \t\n\x0B\f\r]";

/// `String.lastArgumentMatchesMarkdownUrlSyntax()`.
pub fn last_argument_matches_markdown_url_syntax(line: &str) -> bool {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = REGEX.get_or_init(|| {
        let url = r"[^ \t\n\x0B\f\r\x{85}\x{2028}\x{2029}]+";
        let quoted = |q: &str| format!(r"{q}[^{q}\\\n\r\x{{85}}\x{{2028}}\x{{2029}}]*(?:\\{D}[^{q}\\\n\r\x{{85}}\x{{2028}}\x{{2029}}]*)*{q}");
        let braces = format!(r"\([^(\\\n]*(?:\\{D}[^)\\\n]*)*\)");
        let title = format!("(?:{}|{}|{braces})", quoted("\""), quoted("'"));
        Regex::new(&format!(r"\[{D}+\]\({url}(?:{S}+{title})?{S}*\)[.,]?$")).expect("valid pattern")
    });
    regex.is_match(line.trim_end())
}

/// `String.lastArgumentMatchesKotlinReferenceUrlSyntax()`.
pub fn last_argument_matches_kotlin_reference_url_syntax(line: &str) -> bool {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = REGEX.get_or_init(|| Regex::new(r"\[[A-Za-z0-9_|.]*\][.,]?$").expect("valid pattern"));
    regex.is_match(line.trim_end())
}
