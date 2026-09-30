//! Port of ec4j `parser/EditorConfigParser.java` with the `EditorConfigModelHandler` /
//! `AbstractValidatingHandler` it drives, `PropertyTypeRegistry` and `EditorConfigLoader.load`.
//! The Java reader works on a refilled buffer; here the whole (BOM-stripped) text is the buffer, which
//! gives the same captures. `pos` is the index of `current` (the text length at the end of input).

use std::path::Path;

use crate::editor_config::{EditorConfig, Property, Section};
use crate::glob::Glob;
use crate::property_type::{self, AnyPropertyType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorType {
    ExpectedEndOfInput,
    ExpectedStringCharacter,
    GlobNotClosed,
    InvalidGlob,
    InvalidPropertyValue,
    PropertyAssignmentMissing,
    PropertyValueMissing,
    UnexpectedEndOfInput,
    Other,
}

impl ErrorType {
    pub fn is_syntax_error(self) -> bool {
        !matches!(
            self,
            ErrorType::InvalidGlob | ErrorType::InvalidPropertyValue | ErrorType::Other
        )
    }
}

/// `ParseException` (thrown for syntax errors by `THROW_SYNTAX_ERRORS_IGNORE_OTHERS`) or an `IOException`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseException {
    pub error_type: ErrorType,
    pub message: String,
}

impl std::fmt::Display for ParseException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// `PropertyTypeRegistry`: lowercased type name -> type.
#[derive(Clone, Debug, Default)]
pub struct PropertyTypeRegistry {
    types: Vec<&'static dyn AnyPropertyType>,
}

impl PropertyTypeRegistry {
    /// `builder().defaults().type(t)...build()`.
    pub fn with_defaults(
        types: impl IntoIterator<Item = &'static dyn AnyPropertyType>,
    ) -> PropertyTypeRegistry {
        let mut registry = PropertyTypeRegistry::default();
        property_type::standard_types()
            .into_iter()
            .chain(types)
            .for_each(|t| registry.type_(t));
        registry
    }

    fn type_(&mut self, t: &'static dyn AnyPropertyType) {
        let name = t.name().to_lowercase();
        match self
            .types
            .iter_mut()
            .find(|e| e.name().to_lowercase() == name)
        {
            Some(existing) => *existing = t,
            None => self.types.push(t),
        }
    }

    pub fn get_type(&self, name: &str) -> Option<&'static dyn AnyPropertyType> {
        let name = name.to_lowercase();
        self.types
            .iter()
            .copied()
            .find(|t| t.name().to_lowercase() == name)
    }
}

/// `EditorConfigLoader.load(resource)` with the `THROW_SYNTAX_ERRORS_IGNORE_OTHERS` error handler.
pub fn load(path: &Path, registry: &PropertyTypeRegistry) -> Result<EditorConfig, ParseException> {
    let bytes = std::fs::read(path).map_err(|e| ParseException {
        error_type: ErrorType::Other,
        message: format!("Could not load {}: {e}", path.display()),
    })?;
    let text =
        String::from_utf8_lossy(bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes)).into_owned();
    parse(&text, &path.display().to_string(), registry)
}

/// Parses `.editorconfig` text; `resource` only labels error messages.
pub fn parse(
    text: &str,
    resource: &str,
    registry: &PropertyTypeRegistry,
) -> Result<EditorConfig, ParseException> {
    let mut parser = EditorConfigParser {
        chars: text.chars().collect(),
        pos: 0,
        current: None,
        last: None,
        started: false,
        line: 1,
        capture_start: None,
        in_section: false,
        resource,
        registry,
        editor_config: EditorConfig::default(),
        section: None,
        property_name: None,
    };
    parser.read_lines()?;
    if !parser.is_end_of_text() {
        return Err(parser.error(
            "Found unexpected character; expected end of input",
            ErrorType::ExpectedEndOfInput,
        ));
    }
    Ok(parser.editor_config)
}

enum StopReading {
    Glob,
    PropertyName,
    PropertyValue,
}

struct EditorConfigParser<'a> {
    chars: Vec<char>,
    pos: usize,
    current: Option<char>,
    last: Option<char>,
    started: bool,
    line: usize,
    capture_start: Option<usize>,
    in_section: bool,
    resource: &'a str,
    registry: &'a PropertyTypeRegistry,
    editor_config: EditorConfig,
    section: Option<Section>,
    property_name: Option<(String, Option<&'static dyn AnyPropertyType>)>,
}

impl EditorConfigParser<'_> {
    fn read_lines(&mut self) -> Result<(), ParseException> {
        let mut current_line = 0;
        loop {
            self.read();
            if current_line != self.line {
                current_line = self.line;
                self.read_line()?;
            }
            if self.is_end_of_text() {
                break;
            }
        }
        if self.in_section {
            self.end_section();
            self.in_section = false;
        }
        Ok(())
    }

    fn read_line(&mut self) -> Result<(), ParseException> {
        self.skip_white_space();
        if self.is_new_line() {
            return Ok(());
        }
        match self.current {
            Some('\u{FEFF}') | None => Ok(()),
            Some('#' | ';') => {
                self.read_comment();
                Ok(())
            }
            Some('[') => self.read_section(),
            Some(_) => self.read_property(),
        }
    }

    fn read_comment(&mut self) {
        self.start_capture();
        loop {
            self.read();
            if self.is_end_of_text() || self.is_new_line() {
                break;
            }
        }
        self.end_capture(false);
    }

    fn read_section(&mut self) -> Result<(), ParseException> {
        if self.in_section {
            self.end_section();
            self.in_section = false;
        }
        self.section = Some(Section::open());
        self.in_section = true;
        self.read();
        if self.is_end_of_text() {
            return Err(self.glob_not_closed());
        }
        if !self.read_char(']') {
            self.read_glob()?;
        }
        Ok(())
    }

    fn glob_not_closed(&self) -> ParseException {
        self.error(
            "Glob pattern not closed. Expected ']'",
            ErrorType::GlobNotClosed,
        )
    }

    fn read_glob(&mut self) -> Result<(), ParseException> {
        let glob_and_l_bracket: Vec<char> = self
            .read_string(StopReading::Glob, false)?
            .chars()
            .collect();
        let mut close = None;
        for (i, &c) in glob_and_l_bracket.iter().enumerate().rev() {
            if c == ']' {
                close = Some(i);
                break;
            }
            if !is_white_space(Some(c)) {
                return Err(self.glob_not_closed());
            }
        }
        let Some(i) = close else {
            return Err(self.glob_not_closed());
        };
        let glob = Glob::new(&glob_and_l_bracket[..i].iter().collect::<String>());
        // An invalid glob is not a syntax error: the section is kept and never matches.
        self.section.as_mut().expect("in section").set_glob(glob);
        Ok(())
    }

    fn read_string(
        &mut self,
        stop: StopReading,
        trim_trailing: bool,
    ) -> Result<String, ParseException> {
        self.start_capture();
        while !self.is_stop_reading(&stop) {
            if self.is_end_of_text() {
                return Err(self.error("Unexpected end of input", ErrorType::UnexpectedEndOfInput));
            } else if self.current.is_some_and(|c| (c as u32) < 32) {
                return Err(self.error(
                    "Expected a valid string character",
                    ErrorType::ExpectedStringCharacter,
                ));
            }
            self.read();
        }
        Ok(self.end_capture(trim_trailing))
    }

    fn is_stop_reading(&self, stop: &StopReading) -> bool {
        if self.is_end_of_text() || self.is_new_line() {
            return true;
        }
        match stop {
            StopReading::Glob | StopReading::PropertyValue => {
                matches!(self.current, Some(';' | '#')) && is_white_space(self.last)
            }
            StopReading::PropertyName => self.is_colon_separator(),
        }
    }

    fn read_property(&mut self) -> Result<(), ParseException> {
        if !self.in_section {
            self.section = Some(Section::open());
            self.in_section = true;
        }
        self.skip_white_space();
        let name = self
            .read_string(StopReading::PropertyName, true)?
            .to_lowercase();
        self.end_property_name(name.clone());
        self.skip_white_space();
        if !self.read_char('=') && !self.read_char(':') {
            return Err(self.error(
                &format!("Equals sign '=' missing after property name '{name}'"),
                ErrorType::PropertyAssignmentMissing,
            ));
        }
        self.skip_white_space();
        let value = self.read_string(StopReading::PropertyValue, true)?;
        self.end_property_value(&value);
        Ok(())
    }

    fn read_char(&mut self, ch: char) -> bool {
        if self.current != Some(ch) {
            return false;
        }
        self.read();
        true
    }

    fn skip_white_space(&mut self) {
        while is_white_space(self.current) {
            self.read();
        }
    }

    /// Like the Java reader, the end of input is detected before the line count moves past a `\n`.
    fn read(&mut self) {
        let next = if self.started { self.pos + 1 } else { 0 };
        self.started = true;
        if next >= self.chars.len() {
            self.pos = self.chars.len();
            self.current = None;
            self.last = None;
            return;
        }
        if self.current == Some('\n') {
            self.line += 1;
        }
        self.last = self.current;
        self.current = Some(self.chars[next]);
        self.pos = next;
    }

    fn start_capture(&mut self) {
        self.capture_start = Some(self.pos);
    }

    fn end_capture(&mut self, trim_trailing: bool) -> String {
        let start = self.capture_start.take().unwrap_or(self.pos);
        let mut end = self.pos.max(start);
        if trim_trailing {
            while end > start && is_white_space(Some(self.chars[end - 1])) {
                end -= 1;
            }
        }
        self.chars[start..end].iter().collect()
    }

    fn error(&self, message: &str, error_type: ErrorType) -> ParseException {
        ParseException {
            error_type,
            message: format!("{}:{}: {message}", self.resource, self.line),
        }
    }

    fn is_new_line(&self) -> bool {
        matches!(self.current, Some('\n' | '\r'))
    }

    fn is_end_of_text(&self) -> bool {
        self.current.is_none()
    }

    fn is_colon_separator(&self) -> bool {
        matches!(self.current, Some('=' | ':'))
    }

    /// `AbstractValidatingHandler.endPropertyName`: a registered type names the property.
    fn end_property_name(&mut self, name: String) {
        let t = self.registry.get_type(&name);
        let name = t.map_or(name, |t| t.name().to_owned());
        self.property_name = Some((name, t));
    }

    /// `AbstractValidatingHandler.endPropertyValue` + `endProperty` (invalid values are no syntax error).
    fn end_property_value(&mut self, value: &str) {
        let (name, t) = self.property_name.take().expect("property name");
        let property = Property::new(&name, t, Some(value));
        self.section
            .as_mut()
            .expect("in section")
            .property(property);
    }

    /// `EditorConfigModelHandler.endSection`: `applyDefaults().closeSection()`.
    fn end_section(&mut self) {
        let mut section = self.section.take().expect("in section");
        section.apply_defaults();
        self.editor_config.close_section(section);
    }
}

fn is_white_space(c: Option<char>) -> bool {
    matches!(c, Some(' ' | '\t'))
}
