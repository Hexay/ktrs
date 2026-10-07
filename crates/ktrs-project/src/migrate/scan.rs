//! Span-keeping tokens of Kotlin DSL and Groovy build scripts: identifiers, string literals, numbers and
//! single-char punctuation; comments and whitespace dropped. Enough to find calls and literals to rewrite in
//! place; the detection parsers (`kotlin_dsl`, `groovy`) don't keep offsets.

use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Ident,
    /// A quoted literal; `templated` when it interpolates (`$x`, `${x}`).
    Str {
        templated: bool,
    },
    Num,
    Punct(char),
}

#[derive(Debug, Clone)]
pub(crate) struct Tok {
    pub kind: Kind,
    /// The whole token, quotes included.
    pub span: Range<usize>,
    /// Strings: the text between the quotes; otherwise `span`.
    pub inner: Range<usize>,
}

pub(crate) struct Script<'a> {
    pub text: &'a str,
    pub toks: Vec<Tok>,
    /// For each `{` token, the index of its `}` (or `toks.len()` when unclosed).
    close: Vec<Option<usize>>,
}

impl<'a> Script<'a> {
    pub(crate) fn new(text: &'a str) -> Script<'a> {
        let toks = lex(text);
        let mut close = vec![None; toks.len()];
        let mut stack = Vec::new();
        for (i, t) in toks.iter().enumerate() {
            match t.kind {
                Kind::Punct('{') => stack.push(i),
                Kind::Punct('}') => {
                    if let Some(open) = stack.pop() {
                        close[open] = Some(i);
                    }
                }
                _ => {}
            }
        }
        for open in stack {
            close[open] = Some(toks.len());
        }
        Script { text, toks, close }
    }

    pub(crate) fn src(&self, i: usize) -> &'a str {
        &self.text[self.toks[i].span.clone()]
    }

    /// A string literal's content, `None` for other tokens and templates.
    pub(crate) fn plain_str(&self, i: usize) -> Option<&'a str> {
        let t = self.toks.get(i)?;
        matches!(t.kind, Kind::Str { templated: false }).then(|| &self.text[t.inner.clone()])
    }

    pub(crate) fn is_ident(&self, i: usize, name: &str) -> bool {
        self.toks.get(i).is_some_and(|t| t.kind == Kind::Ident && self.src(i) == name)
    }

    pub(crate) fn is_punct(&self, i: usize, c: char) -> bool {
        self.toks.get(i).is_some_and(|t| t.kind == Kind::Punct(c))
    }

    pub(crate) fn close_of(&self, open: usize) -> usize {
        self.close.get(open).copied().flatten().unwrap_or(self.toks.len())
    }

    /// The index after the group opened at `open` (`(`, `[`, `<` or `{`).
    pub(crate) fn skip_group(&self, open: usize) -> usize {
        let (o, c) = match self.toks[open].kind {
            Kind::Punct('(') => ('(', ')'),
            Kind::Punct('[') => ('[', ']'),
            Kind::Punct('<') => ('<', '>'),
            _ => return self.close_of(open) + 1,
        };
        let mut depth = 0;
        for i in open..self.toks.len() {
            if self.is_punct(i, o) {
                depth += 1;
            } else if self.is_punct(i, c) {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
        }
        self.toks.len()
    }

    /// The name of the block opened at `open`: the identifier before it, past a call's `(..)` and a
    /// `<Type>` argument (`configure<SpotlessExtension> {`, `register("x") {`).
    pub(crate) fn block_name(&self, open: usize) -> Option<&'a str> {
        let mut i = open.checked_sub(1)?;
        if self.is_punct(i, ')') {
            i = self.group_start(i, '(', ')')?.checked_sub(1)?;
        }
        if self.is_punct(i, '>') {
            i = self.group_start(i, '<', '>')?.checked_sub(1)?;
        }
        (self.toks[i].kind == Kind::Ident).then(|| self.src(i))
    }

    /// The type argument of a `name<Type> {` block.
    pub(crate) fn block_type_arg(&self, open: usize) -> Option<&'a str> {
        let gt = open.checked_sub(1).filter(|&i| self.is_punct(i, '>'))?;
        (self.toks[gt - 1].kind == Kind::Ident).then(|| self.src(gt - 1))
    }

    fn group_start(&self, close: usize, o: char, c: char) -> Option<usize> {
        let mut depth = 0;
        for i in (0..=close).rev() {
            if self.is_punct(i, c) {
                depth += 1;
            } else if self.is_punct(i, o) {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        }
        None
    }

    /// The `{` tokens enclosing token `i`, innermost first.
    pub(crate) fn enclosing(&self, i: usize) -> Vec<usize> {
        (0..i).rev().filter(|&o| self.is_punct(o, '{') && self.close_of(o) > i).collect()
    }

    /// Top-level (depth 0) or direct children of `parent`: the `{` of each block named `name`.
    pub(crate) fn child_blocks(&self, parent: Option<usize>, name: &str) -> Vec<usize> {
        let (start, end) = match parent {
            Some(p) => (p + 1, self.close_of(p)),
            None => (0, self.toks.len()),
        };
        let mut out = Vec::new();
        let mut i = start;
        while i < end {
            if self.is_punct(i, '{') {
                if self.block_name(i) == Some(name) {
                    out.push(i);
                }
                i = self.close_of(i) + 1;
            } else {
                i += 1;
            }
        }
        out
    }
}

fn lex(src: &str) -> Vec<Tok> {
    let b = src.as_bytes();
    let mut toks = Vec::new();
    let mut i = 0;
    if src.starts_with("#!") {
        i = src.find('\n').unwrap_or(src.len());
    }
    while i < b.len() {
        let c = b[i];
        let start = i;
        match c {
            b'/' if b.get(i + 1) == Some(&b'/') => i = src[i..].find('\n').map_or(b.len(), |n| i + n),
            b'/' if b.get(i + 1) == Some(&b'*') => i = src[i + 2..].find("*/").map_or(b.len(), |n| i + n + 4),
            b'"' | b'\'' => {
                let (inner, end, templated) = string(src, i);
                toks.push(Tok { kind: Kind::Str { templated }, span: start..end, inner });
                i = end;
            }
            b'`' => {
                i = src[i + 1..].find('`').map_or(b.len(), |n| i + n + 2);
                toks.push(Tok { kind: Kind::Ident, span: start..i, inner: start + 1..i - 1 });
            }
            _ if c.is_ascii_alphabetic() || c == b'_' || c >= 0x80 => {
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] >= 0x80) {
                    i += 1;
                }
                toks.push(Tok { kind: Kind::Ident, span: start..i, inner: start..i });
            }
            _ if c.is_ascii_digit() => {
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'.') {
                    i += 1;
                }
                toks.push(Tok { kind: Kind::Num, span: start..i, inner: start..i });
            }
            _ if c.is_ascii_whitespace() => i += 1,
            _ => {
                let ch = src[i..].chars().next().unwrap_or(' ');
                i += ch.len_utf8();
                toks.push(Tok { kind: Kind::Punct(ch), span: start..i, inner: start..i });
            }
        }
    }
    toks
}

/// A literal at `start`: (content range, end, templated). Triple quotes, escapes and `${..}` (with nested
/// braces and strings) are skipped as a unit.
fn string(src: &str, start: usize) -> (Range<usize>, usize, bool) {
    let b = src.as_bytes();
    let q = b[start];
    let triple = b.get(start + 1) == Some(&q) && b.get(start + 2) == Some(&q);
    let open = if triple { 3 } else { 1 };
    let mut i = start + open;
    let mut templated = false;
    while i < b.len() {
        match b[i] {
            b'\\' if !triple => i += 2,
            c if c == q && (!triple || src[i..].starts_with(if q == b'"' { "\"\"\"" } else { "'''" })) => {
                let close = if triple { 3 } else { 1 };
                return (start + open..i, i + close, templated);
            }
            b'\n' if !triple => return (start + open..i, i, templated),
            b'$' if q == b'"' && b.get(i + 1).is_some_and(|n| *n == b'{' || n.is_ascii_alphabetic() || *n == b'_') => {
                templated = true;
                if b[i + 1] == b'{' {
                    i = template_end(src, i + 2);
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    (start + open..b.len(), b.len(), templated)
}

fn template_end(src: &str, mut i: usize) -> usize {
    let b = src.as_bytes();
    let mut depth = 1;
    while i < b.len() {
        match b[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            b'"' | b'\'' => {
                i = string(src, i).1;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    b.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_and_blocks() {
        let s = Script::new("plugins { id(\"a\") version \"${v}\" } // x {\n/* { */ spotless { kotlin {} }");
        let strs: Vec<_> = (0..s.toks.len()).filter_map(|i| s.plain_str(i)).collect();
        assert_eq!(strs, ["a"]);
        let spotless = s.child_blocks(None, "spotless");
        assert_eq!(spotless.len(), 1);
        assert_eq!(s.child_blocks(Some(spotless[0]), "kotlin").len(), 1);
        assert_eq!(s.child_blocks(None, "plugins").len(), 1);
    }
}
