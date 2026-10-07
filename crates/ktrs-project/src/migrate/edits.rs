//! Text edits on one file, and the layout-preserving insertions the rewrites share.

use std::ops::Range;

use super::scan::Script;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Edit {
    pub range: Range<usize>,
    pub text: String,
}

#[derive(Default)]
pub(crate) struct Edits(Vec<Edit>);

impl Edits {
    pub(crate) fn replace(&mut self, range: Range<usize>, text: impl Into<String>) {
        self.0.push(Edit { range, text: text.into() });
    }

    pub(crate) fn insert(&mut self, at: usize, text: impl Into<String>) {
        self.replace(at..at, text);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn extend(&mut self, other: Edits) {
        self.0.extend(other.0);
    }

    /// `text` with the edits applied; overlapping edits keep the first one (insertions at one offset keep
    /// their order).
    pub(crate) fn apply(mut self, text: &str) -> String {
        self.0.sort_by_key(|e| (e.range.start, e.range.end));
        let mut out = String::with_capacity(text.len());
        let mut pos = 0;
        for e in self.0 {
            if e.range.start < pos {
                continue;
            }
            out.push_str(&text[pos..e.range.start]);
            out.push_str(&e.text);
            pos = e.range.end;
        }
        out.push_str(&text[pos..]);
        out
    }
}

pub(crate) fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |n| n + 1)
}

/// The whitespace that starts the line containing `at`.
pub(crate) fn indent_at(text: &str, at: usize) -> &str {
    let start = line_start(text, at);
    let line = &text[start..];
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

/// Whether only whitespace precedes `at` on its line.
pub(crate) fn starts_line(text: &str, at: usize) -> bool {
    text[line_start(text, at)..at].trim().is_empty()
}

/// One indentation step, guessed from the file (4 spaces when nothing is indented).
pub(crate) fn indent_unit(text: &str) -> String {
    let indents = text.lines().filter(|l| !l.trim().is_empty()).map(|l| &l[..l.len() - l.trim_start().len()]);
    match indents.filter(|i| !i.is_empty()).min_by_key(|i| i.len()) {
        Some(i) if i.starts_with('\t') => "\t".into(),
        Some(i) => i.to_string(),
        None => "    ".into(),
    }
}

/// Appends `stmt` as the last statement of the block opened at token `open`, on its own line indented like
/// the block's statements; a one-line block gets `; stmt` (or `{ stmt }` when empty).
pub(crate) fn append_to_block(s: &Script, open: usize, stmt: &str, edits: &mut Edits) {
    let close = s.close_of(open);
    let Some(close_tok) = s.toks.get(close) else { return };
    let at = close_tok.span.start;
    if starts_line(s.text, at) {
        let indent = match (open + 1..close).next() {
            Some(first) => indent_at(s.text, s.toks[first].span.start).to_string(),
            None => format!("{}{}", indent_at(s.text, at), indent_unit(s.text)),
        };
        let stmt = stmt.replace('\n', &format!("\n{indent}"));
        edits.insert(line_start(s.text, at), format!("{indent}{stmt}\n"));
    } else if close == open + 1 {
        edits.replace(s.toks[open].span.end..at, format!(" {stmt} "));
    } else {
        let last_end = s.toks[close - 1].span.end;
        edits.insert(last_end, format!("; {stmt}"));
    }
}

/// Inserts `block` (lines ending in `\n`) before the first statement of a file, after its `import`s and
/// leading comments, followed by a blank line.
pub(crate) fn insert_at_top(s: &Script, block: &str, edits: &mut Edits) {
    let mut i = 0;
    while s.is_ident(i, "import") {
        let line_end = s.text[s.toks[i].span.start..].find('\n').map_or(s.text.len(), |n| s.toks[i].span.start + n);
        while s.toks.get(i).is_some_and(|t| t.span.start < line_end) {
            i += 1;
        }
    }
    match s.toks.get(i) {
        Some(t) => edits.insert(line_start(s.text, t.span.start), format!("{block}\n")),
        None if s.text.is_empty() => edits.insert(0, block),
        None => {
            let sep = if s.text.ends_with('\n') { "\n" } else { "\n\n" };
            edits.insert(s.text.len(), format!("{sep}{block}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn append(text: &str, block: &str) -> String {
        let s = Script::new(text);
        let open = s.child_blocks(None, block)[0];
        let mut e = Edits::default();
        append_to_block(&s, open, "x()", &mut e);
        e.apply(text)
    }

    #[test]
    fn appends_by_layout() {
        assert_eq!(append("r {\n  a()\n}\n", "r"), "r {\n  a()\n  x()\n}\n");
        assert_eq!(append("b {\n    r {\n    }\n}\n", "b"), "b {\n    r {\n    }\n    x()\n}\n");
        assert_eq!(append("r { a() }\n", "r"), "r { a(); x() }\n");
        assert_eq!(append("r {}\n", "r"), "r { x() }\n");
        assert_eq!(append("r {\n}\n", "r"), "r {\n    x()\n}\n");
    }

    #[test]
    fn inserts_after_imports() {
        let text = "// c\nimport a.B\nimport c.D\n\nplugins {}\n";
        let s = Script::new(text);
        let mut e = Edits::default();
        insert_at_top(&s, "x {}\n", &mut e);
        assert_eq!(e.apply(text), "// c\nimport a.B\nimport c.D\n\nx {}\n\nplugins {}\n");
    }
}
