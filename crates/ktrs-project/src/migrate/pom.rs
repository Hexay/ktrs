//! `pom.xml` elements with their source spans (the detection reader in `xml.rs` decodes and drops them).

use std::ops::Range;

#[derive(Debug, Default)]
pub(crate) struct El {
    pub name: String,
    /// The start tag, `<name ...>` or `<name/>`.
    pub open: Range<usize>,
    /// Between the tags (empty, at `open.end`, when self-closing).
    pub inner: Range<usize>,
    /// The end tag (empty when self-closing).
    pub close: Range<usize>,
    pub children: Vec<El>,
}

impl El {
    pub(crate) fn child(&self, name: &str) -> Option<&El> {
        self.children.iter().find(|c| c.name == name)
    }

    pub(crate) fn path(&self, names: &[&str]) -> Option<&El> {
        names.iter().try_fold(self, |e, n| e.child(n))
    }

    /// Trimmed text of a leaf.
    pub(crate) fn text<'a>(&self, src: &'a str) -> &'a str {
        src[self.inner.clone()].trim()
    }

    /// The trimmed text's span.
    pub(crate) fn text_span(&self, src: &str) -> Range<usize> {
        let raw = &src[self.inner.clone()];
        let start = self.inner.start + (raw.len() - raw.trim_start().len());
        start..start + raw.trim().len()
    }

    pub(crate) fn has_attr(&self, src: &str, name: &str) -> bool {
        let tag = &src[self.open.clone()];
        tag.split(|c: char| c.is_whitespace()).any(|part| part.starts_with(&format!("{name}=")))
    }

    /// Every element named `name` at any depth below.
    pub(crate) fn descendants<'s>(&'s self, name: &str, out: &mut Vec<&'s El>) {
        for c in &self.children {
            if c.name == name {
                out.push(c);
            }
            c.descendants(name, out);
        }
    }
}

/// A synthetic root whose children are the document's top-level elements.
pub(crate) fn parse(src: &str) -> El {
    let mut stack = vec![El::default()];
    let mut i = 0;
    while let Some(lt) = src[i..].find('<').map(|p| p + i) {
        let rest = &src[lt..];
        let skip_to = |end: &str| rest.find(end).map_or(src.len(), |p| lt + p + end.len());
        if rest.starts_with("<!--") {
            i = skip_to("-->");
        } else if rest.starts_with("<![CDATA[") {
            i = skip_to("]]>");
        } else if rest.starts_with("<?") {
            i = skip_to("?>");
        } else if rest.starts_with("<!") {
            i = skip_to(">");
        } else if let Some(name) = rest.strip_prefix("</") {
            let name: String = name.chars().take_while(|c| !c.is_whitespace() && *c != '>').collect();
            let end = skip_to(">");
            if stack.iter().skip(1).any(|e| e.name == name) {
                loop {
                    let mut done = stack.pop().unwrap_or_default();
                    done.inner.end = lt;
                    done.close = lt..end;
                    let matched = done.name == name;
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(done);
                    }
                    if matched {
                        break;
                    }
                }
            }
            i = end;
        } else {
            let end = tag_end(src, lt);
            let name: String =
                src[lt + 1..end].chars().take_while(|c| !c.is_whitespace() && *c != '>' && *c != '/').collect();
            let self_closing = src[..end].ends_with("/>");
            let el = El { name, open: lt..end, inner: end..end, close: end..end, children: Vec::new() };
            if self_closing {
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(el);
                }
            } else {
                stack.push(el);
            }
            i = end;
        }
    }
    while stack.len() > 1 {
        let mut done = stack.pop().unwrap_or_default();
        done.inner.end = src.len();
        if let Some(parent) = stack.last_mut() {
            parent.children.push(done);
        }
    }
    stack.pop().unwrap_or_default()
}

/// The index after the `>` closing the tag at `lt`, skipping quoted attribute values.
fn tag_end(src: &str, lt: usize) -> usize {
    let mut quote = None;
    for (off, c) in src[lt..].char_indices() {
        match (quote, c) {
            (None, '"' | '\'') => quote = Some(c),
            (Some(q), _) if c == q => quote = None,
            (None, '>') => return lt + off + 1,
            _ => {}
        }
    }
    src.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans() {
        let src = "<?xml?><project><!-- <x> --><a k=\"v>\"> t </a><b/></project>";
        let root = parse(src);
        let project = root.child("project").unwrap();
        let a = project.child("a").unwrap();
        assert_eq!(a.text(src), "t");
        assert_eq!(&src[a.text_span(src)], "t");
        assert!(a.has_attr(src, "k"));
        assert_eq!(&src[a.close.clone()], "</a>");
        assert!(project.child("b").is_some());
    }
}
