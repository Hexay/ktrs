//! Port of ktfmt's `KotlinTok.kt`. The `kind` field is omitted: ktfmt always sets it to
//! `KtTokens.EOF`.

use crate::doc::{Tok, newlines, utf16_len};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KotlinTok {
    index: i32,
    original_text: String,
    text: String,
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
        KotlinTok {
            index,
            original_text,
            text,
            position,
            column,
            is_token,
        }
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
        &self.text
    }

    fn get_original_text(&self) -> &str {
        &self.original_text
    }

    fn length(&self) -> i32 {
        utf16_len(&self.original_text)
    }

    fn is_newline(&self) -> bool {
        newlines::is_newline(&self.text)
    }

    fn is_slash_slash_comment(&self) -> bool {
        self.text.starts_with("//")
    }

    fn is_slash_star_comment(&self) -> bool {
        self.text.starts_with("/*")
    }

    fn is_javadoc_comment(&self) -> bool {
        self.text.starts_with("/**") && utf16_len(&self.text) > 4
    }

    fn is_comment(&self) -> bool {
        self.is_slash_slash_comment() || self.is_slash_star_comment()
    }
}
