//! Port of ktfmt's `ParseError.kt` (the `LineColumn` constructor); the tokenizer raises it for
//! unclosed comments. The `Formatter` port should reuse this type rather than redefine it.

use std::fmt;

use super::string_util::offset_to_line_column;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub error_description: String,
    /// 0-based.
    pub line: i32,
    /// 0-based, in UTF-16 units.
    pub column: i32,
}

impl ParseError {
    pub fn new(error_description: impl Into<String>, line: i32, column: i32) -> ParseError {
        ParseError {
            error_description: error_description.into(),
            line,
            column,
        }
    }

    /// `ParseError(description, StringUtil.offsetToLineColumn(text, offset))`.
    pub fn at_offset(
        error_description: impl Into<String>,
        text: &str,
        offset: usize,
    ) -> ParseError {
        let (line, column) = offset_to_line_column(text, offset).expect("offset within text");
        ParseError::new(error_description, line, column)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: error: {}",
            self.line + 1,
            self.column + 1,
            self.error_description
        )
    }
}

impl std::error::Error for ParseError {}
