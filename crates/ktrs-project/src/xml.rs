//! A small tolerant XML reader for `pom.xml`: elements, attributes, text (entities and CDATA decoded);
//! comments, processing instructions and doctypes skipped. Unbalanced end tags close up to their match.

#[derive(Debug, Default, Clone)]
pub(crate) struct Element {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Element>,
    pub text: String,
}

impl Element {
    pub(crate) fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|c| c.name == name)
    }

    pub(crate) fn children_named<'s>(&'s self, name: &'s str) -> impl Iterator<Item = &'s Element> {
        self.children.iter().filter(move |c| c.name == name)
    }

    pub(crate) fn path(&self, names: &[&str]) -> Option<&Element> {
        names.iter().try_fold(self, |e, n| e.child(n))
    }

    pub(crate) fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }

    /// All text and attribute values below this element, space-separated.
    pub(crate) fn all_text(&self) -> String {
        let mut out = String::new();
        self.collect_text(&mut out);
        out
    }

    fn collect_text(&self, out: &mut String) {
        out.push_str(&self.text);
        for (_, v) in &self.attrs {
            out.push(' ');
            out.push_str(v);
        }
        for c in &self.children {
            out.push(' ');
            c.collect_text(out);
        }
    }
}

/// A synthetic element whose children are the document's top-level elements.
pub(crate) fn parse(text: &str) -> Element {
    let mut stack = vec![Element::default()];
    let mut rest = text;
    while !rest.is_empty() {
        let Some(lt) = rest.find('<') else {
            push_text(&mut stack, rest);
            break;
        };
        push_text(&mut stack, &rest[..lt]);
        rest = &rest[lt..];
        if let Some(r) = rest.strip_prefix("<!--") {
            rest = r.find("-->").map_or("", |i| &r[i + 3..]);
        } else if let Some(r) = rest.strip_prefix("<![CDATA[") {
            let end = r.find("]]>").unwrap_or(r.len());
            if let Some(top) = stack.last_mut() {
                top.text.push_str(&r[..end]);
            }
            rest = r.get(end + 3..).unwrap_or("");
        } else if rest.starts_with("<?") || rest.starts_with("<!") {
            rest = rest.find('>').map_or("", |i| &rest[i + 1..]);
        } else if let Some(r) = rest.strip_prefix("</") {
            let end = r.find('>').unwrap_or(r.len());
            let name = r[..end].trim();
            if stack.iter().skip(1).any(|e| e.name == name) {
                while stack.len() > 1 {
                    let done = stack.pop().expect("len > 1");
                    let matched = done.name == name;
                    stack.last_mut().expect("root").children.push(done);
                    if matched {
                        break;
                    }
                }
            }
            rest = r.get(end + 1..).unwrap_or("");
        } else {
            let end = tag_end(rest);
            let tag = &rest[1..end];
            let self_closing = tag.ends_with('/');
            let element = open_tag(tag.trim_end_matches('/'));
            if self_closing {
                stack.last_mut().expect("root").children.push(element);
            } else {
                stack.push(element);
            }
            rest = rest.get(end + 1..).unwrap_or("");
        }
    }
    while stack.len() > 1 {
        let done = stack.pop().expect("len > 1");
        stack.last_mut().expect("root").children.push(done);
    }
    stack.pop().unwrap_or_default()
}

/// Index of the `>` closing the tag at the start of `s`, skipping quoted attribute values.
fn tag_end(s: &str) -> usize {
    let mut quote = None;
    for (i, c) in s.char_indices() {
        match (quote, c) {
            (None, '"' | '\'') => quote = Some(c),
            (Some(q), _) if q == c => quote = None,
            (None, '>') => return i,
            _ => {}
        }
    }
    s.len()
}

fn open_tag(tag: &str) -> Element {
    let tag = tag.trim();
    let name_end = tag.find(char::is_whitespace).unwrap_or(tag.len());
    let mut element = Element { name: tag[..name_end].to_string(), ..Element::default() };
    let mut rest = &tag[name_end..];
    while let Some(eq) = rest.find('=') {
        let key = rest[..eq].trim().to_string();
        let after = rest[eq + 1..].trim_start();
        let Some(q) = after.chars().next().filter(|c| *c == '"' || *c == '\'') else { break };
        let value_end = after[1..].find(q).map_or(after.len(), |i| i + 1);
        element.attrs.push((key, decode(&after[1..value_end])));
        rest = after.get(value_end + 1..).unwrap_or("");
    }
    element
}

fn push_text(stack: &mut [Element], text: &str) {
    if let Some(top) = stack.last_mut()
        && !text.trim().is_empty()
    {
        top.text.push_str(&decode(text));
    }
}

fn decode(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pom_shapes() {
        let doc = parse(
            "<?xml version=\"1.0\"?>\n<!-- c --><project><build><plugins><plugin a='x &amp; y'>\
             <artifactId>p</artifactId><empty/><c><![CDATA[a<b]]></c></plugin></plugins></build></project>",
        );
        let plugin = doc.path(&["project", "build", "plugins", "plugin"]).unwrap();
        assert_eq!(plugin.attr("a"), Some("x & y"));
        assert_eq!(plugin.child("artifactId").unwrap().text, "p");
        assert!(plugin.child("empty").is_some());
        assert_eq!(plugin.child("c").unwrap().text, "a<b");
    }
}
