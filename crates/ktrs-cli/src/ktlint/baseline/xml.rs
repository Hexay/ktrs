//! The slice of a DOM parser (`DocumentBuilder.parse`) that baseline files need: elements and attributes.
//! Well-formedness errors carry Xerces' message and `line:col` for the common cases only.

pub struct Element {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Element>,
}

impl Element {
    /// `getAttribute(name)`: `""` when absent.
    pub fn attribute(&self, name: &str) -> &str {
        self.attributes.iter().find(|(n, _)| n == name).map_or("", |(_, v)| v.as_str())
    }

    /// `getElementsByTagName(name)` on this element: its descendants in document order.
    pub fn elements_by_tag_name<'a>(&'a self, name: &str, found: &mut Vec<&'a Element>) {
        for child in &self.children {
            if child.name == name {
                found.push(child);
            }
            child.elements_by_tag_name(name, found);
        }
    }
}

/// A `SAXParseException`: what Xerces' default error handler prints as `[Fatal Error] :line:col: message`.
#[derive(Debug, PartialEq, Eq)]
pub struct XmlError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

/// Parses `text` into its root element, wrapped in a document node (so the root itself is found by name).
pub fn parse_document(text: &str) -> Result<Element, XmlError> {
    let mut p = Parser { chars: text.trim_start_matches('\u{feff}').chars().collect(), pos: 0, line: 1, col: 1 };
    p.skip_misc(true)?;
    if p.peek().is_none() {
        return Err(p.error("Premature end of file."));
    }
    let root = p.element()?;
    p.skip_misc(false)?;
    Ok(Element { name: "#document".to_owned(), attributes: Vec::new(), children: vec![root] })
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn starts_with(&self, s: &str) -> bool {
        s.chars().enumerate().all(|(i, c)| self.chars.get(self.pos + i) == Some(&c))
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn error(&self, message: &str) -> XmlError {
        XmlError { line: self.line, col: self.col, message: message.to_owned() }
    }

    fn eof(&self) -> XmlError {
        self.error("XML document structures must start and end within the same entity.")
    }

    fn skip_until(&mut self, end: &str) -> Result<(), XmlError> {
        while !self.starts_with(end) {
            self.bump().ok_or_else(|| self.eof())?;
        }
        end.chars().for_each(|_| {
            self.bump();
        });
        Ok(())
    }

    /// Whitespace, comments and processing instructions around the root element.
    fn skip_misc(&mut self, prolog: bool) -> Result<(), XmlError> {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('<') if self.starts_with("<?") => self.skip_until("?>")?,
                Some('<') if self.starts_with("<!--") => self.skip_until("-->")?,
                Some('<') if prolog && self.starts_with("<!DOCTYPE") => self.skip_until(">")?,
                Some('<') if prolog => return Ok(()),
                Some('<') => {
                    self.bump();
                    return Err(self.error("The markup in the document following the root element must be well-formed."));
                }
                Some(_) if prolog => return Err(self.error("Content is not allowed in prolog.")),
                Some(_) => return Err(self.error("Content is not allowed in trailing section.")),
                None => return Ok(()),
            }
        }
    }

    fn name(&mut self) -> String {
        let mut name = String::new();
        while let Some(c) = self.peek().filter(|&c| c.is_alphanumeric() || matches!(c, '_' | ':' | '-' | '.')) {
            name.push(c);
            self.bump();
        }
        name
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.bump();
        }
    }

    fn element(&mut self) -> Result<Element, XmlError> {
        self.bump();
        let name = self.name();
        let mut element = Element { name, attributes: Vec::new(), children: Vec::new() };
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => return Err(self.eof()),
                Some('/') if self.starts_with("/>") => {
                    self.bump();
                    self.bump();
                    return Ok(element);
                }
                Some('>') => {
                    self.bump();
                    break;
                }
                Some(_) => {
                    let attribute = self.attribute(&element.name)?;
                    element.attributes.push(attribute);
                }
            }
        }
        loop {
            match self.peek() {
                None => return Err(self.eof()),
                Some('<') if self.starts_with("</") => {
                    self.bump();
                    self.bump();
                    let at_name = self.error("");
                    let end = self.name();
                    if end != element.name {
                        return Err(XmlError { message: format!(
                            "The element type \"{0}\" must be terminated by the matching end-tag \"</{0}>\".",
                            element.name
                        ), ..at_name });
                    }
                    self.skip_whitespace();
                    if self.bump() != Some('>') {
                        return Err(self.error(&format!("The end-tag for element type \"{end}\" must end with a '>' delimiter.")));
                    }
                    return Ok(element);
                }
                Some('<') if self.starts_with("<!--") => self.skip_until("-->")?,
                Some('<') if self.starts_with("<![CDATA[") => self.skip_until("]]>")?,
                Some('<') if self.starts_with("<?") => self.skip_until("?>")?,
                Some('<') => element.children.push(self.element()?),
                Some('&') => {
                    self.reference()?;
                }
                Some(_) => {
                    self.bump();
                }
            }
        }
    }

    fn attribute(&mut self, element_name: &str) -> Result<(String, String), XmlError> {
        let expected = || {
            format!("Element type \"{element_name}\" must be followed by either attribute specifications, \">\" or \"/>\".")
        };
        let name = self.name();
        if name.is_empty() {
            return Err(self.error(&expected()));
        }
        self.skip_whitespace();
        if self.bump() != Some('=') {
            return Err(self.error(&format!(
                "Attribute name \"{name}\" associated with an element type \"{element_name}\" must be followed by the ' = ' character."
            )));
        }
        self.skip_whitespace();
        let quote = match self.peek() {
            Some(q @ ('"' | '\'')) => {
                self.bump();
                q
            }
            _ => {
                return Err(self.error(&format!(
                    "Open quote is expected for attribute \"{name}\" associated with an  element type  \"{element_name}\"."
                )));
            }
        };
        let mut value = String::new();
        loop {
            match self.peek() {
                None => return Err(self.eof()),
                Some(c) if c == quote => {
                    self.bump();
                    return Ok((name, value));
                }
                Some('<') => {
                    return Err(self.error(&format!(
                        "The value of attribute \"{name}\" associated with an element type \"{element_name}\" must not contain the '<' character."
                    )));
                }
                Some('&') => value.push_str(&self.reference()?),
                Some(c) => {
                    self.bump();
                    value.push(if matches!(c, '\n' | '\r' | '\t') { ' ' } else { c });
                }
            }
        }
    }

    fn reference(&mut self) -> Result<String, XmlError> {
        self.bump();
        let mut name = String::new();
        loop {
            match self.bump() {
                None => return Err(self.eof()),
                Some(';') => break,
                Some(c) => name.push(c),
            }
        }
        let numeric = |digits: &str, radix| u32::from_str_radix(digits, radix).ok().and_then(char::from_u32);
        let c = match name.as_str() {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            n if n.starts_with("#x") => numeric(&n[2..], 16),
            n if n.starts_with('#') => numeric(&n[1..], 10),
            _ => None,
        };
        c.map(String::from)
            .ok_or_else(|| self.error(&format!("The entity \"{name}\" was referenced, but not declared.")))
    }
}
