//! Port of ktfmt's `format` package. Entry point mirrors `Formatter.format(options, code)`.

pub mod input;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattingOptions {
    pub max_width: usize,
    pub block_indent: usize,
    pub continuation_indent: usize,
    pub manage_trailing_commas: bool,
    pub remove_unused_imports: bool,
}

impl FormattingOptions {
    // TODO: port the exact values and remaining fields from FormattingOptions.kt (meta/google/kotlinlang).
    pub fn meta() -> Self {
        todo!("port Formatter.META_FORMAT")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

/// `Formatter.format(options, code)`: formats a whole file; fails like ktfmt on unparseable input.
pub fn format(_code: &str, _options: &FormattingOptions) -> Result<String, FormatError> {
    todo!("port Formatter.format")
}
