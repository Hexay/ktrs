//! Port of ktfmt's `format` package. Entry point mirrors `Formatter.format(options, code)`.
//! JVM exceptions become [`FormatError`]s: the visitor records them in the `OpsBuilder` (like the
//! engine's `FormattingError`s) and unwinds by returning early; either way no output is produced.

mod enum_entry_list;
mod formatter;
mod formatter_context;
mod formatting_options;
pub mod input;
mod kotlin_code;
mod kotlin_input_ast_visitor;
pub mod kotlin_text;
mod multiline_string_formatter;
mod parser;
mod psi_utils;
mod redundant_element_manager;
mod redundant_import_detector;
mod redundant_semicolon_detector;
mod trailing_commas;

use std::fmt;

use crate::doc::{FormatterException, FormattingError, RangeSet};

pub use formatter::{GOOGLE_FORMAT, KOTLINLANG_FORMAT, META_FORMAT, format_meta};
pub use formatting_options::{Builder as FormattingOptionsBuilder, FormattingOptions, TrailingCommaManagementStrategy};
pub use input::ParseError;
pub use kotlin_code::{FileType, KotlinCode};

/// Whatever `Formatter.format` throws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormatError {
    Parse(ParseError),
    Formatting(FormattingError),
    Formatter(FormatterException),
    /// Any other JVM exception, as its `toString()` (e.g. `IllegalArgumentException` for an unsupported file type).
    Runtime(String),
    /// The parser's `AssertionError`, a JVM `Error` ktfmt's CLI doesn't catch (research/24, finding 4).
    MissedTokens(ktrs_syntax::MissedTokens),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Parse(e) => write!(f, "{e}"),
            FormatError::Formatting(e) => write!(f, "{}", e.to_string().trim_end()),
            FormatError::Formatter(e) => write!(f, "{e}"),
            FormatError::Runtime(message) => f.write_str(message),
            FormatError::MissedTokens(e) => f.write_str(&e.assertion_error()),
        }
    }
}

impl std::error::Error for FormatError {}

impl From<ParseError> for FormatError {
    fn from(e: ParseError) -> Self {
        FormatError::Parse(e)
    }
}

impl From<FormattingError> for FormatError {
    fn from(e: FormattingError) -> Self {
        FormatError::Formatting(e)
    }
}

impl From<FormatterException> for FormatError {
    fn from(e: FormatterException) -> Self {
        FormatError::Formatter(e)
    }
}

/// `Formatter.format(options, KotlinCode(code, fileType))`: formats a whole file; fails like ktfmt on unparseable input.
pub fn format(code: &str, file_type: FileType, options: &FormattingOptions) -> Result<String, FormatError> {
    formatter::format(options, &KotlinCode::from(code, file_type)?, None)
}

/// `Formatter.format(options, code, characterRanges)`: with ranges (byte offsets into `code.code`),
/// only those are pretty-printed; the whole-file cleanup passes still run.
pub fn format_code(
    options: &FormattingOptions,
    code: &KotlinCode,
    character_ranges: Option<&RangeSet>,
) -> Result<String, FormatError> {
    formatter::format(options, code, character_ranges)
}
