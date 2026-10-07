//! Groovy tokens, enough for build scripts: identifiers, strings (GString `$x` / `${x}` interpolation kept),
//! numbers, punctuation and newlines (Groovy statements end at a newline). Slashy strings lex as `/`.

use crate::ir::{Expr, Part};

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Tok {
    Ident(String),
    Str(Expr),
    Num(String),
    Punct(&'static str),
    Newline,
}

const PUNCTS: [&str; 34] = [
    "?.", "*.", "->", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "::", "..", "(", ")", "{", "}", "[", "]",
    ",", ":", "=", ".", ";", "+", "-", "*", "/", "<", ">", "!", "?", "&", "|",
];

pub(super) fn lex(src: &str) -> Vec<Tok> {
    let chars: Vec<char> = src.chars().collect();
    let mut toks = Vec::new();
    let mut i = 0;
    if src.starts_with("#!") {
        i = chars.iter().position(|&c| c == '\n').unwrap_or(chars.len());
    }
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match c {
            '\n' => {
                if toks.last() != Some(&Tok::Newline) {
                    toks.push(Tok::Newline);
                }
                i += 1;
            }
            _ if c.is_whitespace() => i += 1,
            '\\' if next == Some('\n') => i += 2,
            '/' if next == Some('/') => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if next == Some('*') => {
                i += 2;
                while i < chars.len() && !(chars[i] == '*' && chars.get(i + 1) == Some(&'/')) {
                    i += 1;
                }
                i += 2;
            }
            '\'' | '"' => {
                let (tok, end) = string(&chars, i);
                toks.push(Tok::Str(tok));
                i = end;
            }
            _ if c.is_ascii_digit() => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '.')
                {
                    if chars[i] == '.' && !chars.get(i + 1).is_some_and(char::is_ascii_digit) {
                        break;
                    }
                    i += 1;
                }
                toks.push(Tok::Num(chars[start..i].iter().collect()));
            }
            _ if is_ident_start(c) => {
                let start = i;
                while i < chars.len() && is_ident_part(chars[i]) {
                    i += 1;
                }
                toks.push(Tok::Ident(chars[start..i].iter().collect()));
            }
            _ => {
                let rest: String = chars[i..chars.len().min(i + 2)].iter().collect();
                match PUNCTS.iter().find(|p| rest.starts_with(**p)) {
                    Some(p) => {
                        toks.push(Tok::Punct(p));
                        i += p.chars().count();
                    }
                    None => i += 1,
                }
            }
        }
    }
    toks
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

fn is_ident_part(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// The string literal starting at `start`, and the index after it.
fn string(chars: &[char], start: usize) -> (Expr, usize) {
    let quote = chars[start];
    let triple = chars.get(start + 1) == Some(&quote) && chars.get(start + 2) == Some(&quote);
    let mut i = start + if triple { 3 } else { 1 };
    let interpolate = quote == '"';
    let mut parts = Vec::new();
    let mut lit = String::new();
    while i < chars.len() {
        let c = chars[i];
        if triple && c == quote && chars.get(i + 1) == Some(&quote) && chars.get(i + 2) == Some(&quote) {
            i += 3;
            break;
        }
        if !triple && c == quote {
            i += 1;
            break;
        }
        if !triple && c == '\n' {
            break;
        }
        match c {
            '\\' if i + 1 < chars.len() => {
                lit.push(match chars[i + 1] {
                    'n' => '\n',
                    't' => '\t',
                    other => other,
                });
                i += 2;
            }
            '$' if interpolate && chars.get(i + 1) == Some(&'{') => {
                let close = chars[i..].iter().position(|&c| c == '}').map_or(chars.len(), |p| i + p);
                let inner: String = chars[i + 2..close].iter().collect();
                parts.push(Part::Lit(std::mem::take(&mut lit)));
                parts.push(Part::Expr(super::parser::parse_expr(&inner)));
                i = close + 1;
            }
            '$' if interpolate && chars.get(i + 1).is_some_and(|&c| is_ident_start(c)) => {
                let begin = i + 1;
                let mut end = begin;
                while end < chars.len()
                    && (is_ident_part(chars[end])
                        || chars[end] == '.' && chars.get(end + 1).is_some_and(|&c| is_ident_start(c)))
                {
                    end += 1;
                }
                let inner: String = chars[begin..end].iter().collect();
                parts.push(Part::Lit(std::mem::take(&mut lit)));
                parts.push(Part::Expr(super::parser::parse_expr(&inner)));
                i = end;
            }
            _ => {
                lit.push(c);
                i += 1;
            }
        }
    }
    if parts.is_empty() {
        return (Expr::Str(lit), i);
    }
    parts.push(Part::Lit(lit));
    (Expr::Template(parts), i)
}
