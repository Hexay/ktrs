//! Port of ktfmt's `KotlinTok.kt`. The `kind` field is omitted: ktfmt always sets it to
//! `KtTokens.EOF`.
//!
//! The original text is a range of the shared file text, and `text` is stored only where it
//! differs (string templates with tombstones): no per-tok string allocations.

use std::fmt;
use std::ops::Range;
use std::rc::Rc;

use crate::doc::{Tok, newlines, utf16_len};

#[derive(Clone)]
pub struct KotlinTok {
    index: i32,
    source: Rc<str>,
    original_range: Range<usize>,
    /// `text` when it differs from the original text.
    text: Option<Box<str>>,
    position: i32,
    column: i32,
    pub is_token: bool,
}

impl KotlinTok {
    pub fn new(
        index: i32,
        original_text: String,
        text: String,
        position: i32,
        column: i32,
        is_token: bool,
    ) -> Self {
        let text = (text != original_text).then(|| text.into_boxed_str());
        let original_range = 0..original_text.len();
        KotlinTok { index, source: original_text.into(), original_range, text, position, column, is_token }
    }

    /// A tok whose original text is `source[original_range]`.
    pub fn from_source(
        index: i32,
        source: &Rc<str>,
        original_range: Range<usize>,
        text: Option<String>,
        column: i32,
        is_token: bool,
    ) -> Self {
        let position = original_range.start as i32;
        let text = text.filter(|t| **t != source[original_range.clone()]).map(String::into_boxed_str);
        KotlinTok { index, source: source.clone(), original_range, text, position, column, is_token }
    }

    fn text(&self) -> &str {
        self.text.as_deref().unwrap_or_else(|| self.get_original_text())
    }
}

impl PartialEq for KotlinTok {
    fn eq(&self, other: &Self) -> bool {
        (self.index, self.get_original_text(), self.text(), self.position, self.column, self.is_token)
            == (other.index, other.get_original_text(), other.text(), other.position, other.column, other.is_token)
    }
}

impl Eq for KotlinTok {}

impl fmt::Debug for KotlinTok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KotlinTok")
            .field("index", &self.index)
            .field("original_text", &self.get_original_text())
            .field("text", &self.text())
            .field("position", &self.position)
            .field("column", &self.column)
            .field("is_token", &self.is_token)
            .finish()
    }
}

impl Tok for KotlinTok {
    fn get_index(&self) -> i32 {
        self.index
    }

    fn get_position(&self) -> i32 {
        self.position
    }

    fn get_column(&self) -> i32 {
        self.column
    }

    fn get_text(&self) -> &str {
        self.text()
    }

    fn get_original_text(&self) -> &str {
        &self.source[self.original_range.clone()]
    }

    fn length(&self) -> i32 {
        utf16_len(self.get_original_text())
    }

    fn is_newline(&self) -> bool {
        newlines::is_newline(self.text())
    }

    fn is_slash_slash_comment(&self) -> bool {
        self.text().starts_with("//")
    }

    fn is_slash_star_comment(&self) -> bool {
        self.text().starts_with("/*")
    }

    fn is_javadoc_comment(&self) -> bool {
        self.text().starts_with("/**") && utf16_len(self.text()) > 4
    }

    fn is_comment(&self) -> bool {
        self.is_slash_slash_comment() || self.is_slash_star_comment()
    }
}
