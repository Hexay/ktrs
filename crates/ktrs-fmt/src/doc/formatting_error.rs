//! Ports of `FormatterDiagnostic`, `FormattingError` and `java/FormatterException`.

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatterDiagnostic {
    line_number: i32,
    message: String,
    column: i32,
}

impl FormatterDiagnostic {
    pub fn create(message: impl Into<String>) -> FormatterDiagnostic {
        FormatterDiagnostic {
            line_number: -1,
            column: -1,
            message: message.into(),
        }
    }

    pub fn create_at(
        line_number: i32,
        column: i32,
        message: impl Into<String>,
    ) -> FormatterDiagnostic {
        assert!(line_number >= 0 && column >= 0);
        FormatterDiagnostic {
            line_number,
            column,
            message: message.into(),
        }
    }

    pub fn line(&self) -> i32 {
        self.line_number
    }

    /// 0-based, in UTF-16 units.
    pub fn column(&self) -> i32 {
        self.column
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for FormatterDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line_number >= 0 {
            write!(f, "{}:", self.line_number)?;
        }
        if self.column >= 0 {
            write!(f, "{}:", self.column + 1)?;
        }
        if self.line_number >= 0 || self.column >= 0 {
            f.write_str(" ")?;
        }
        write!(f, "error: {}", self.message)
    }
}

/// Thrown (in Java) by `OpsBuilder` when the ops don't match the input; see `OpsBuilder::error`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormattingError {
    diagnostics: Vec<FormatterDiagnostic>,
}

impl FormattingError {
    pub fn new(diagnostic: FormatterDiagnostic) -> FormattingError {
        FormattingError {
            diagnostics: vec![diagnostic],
        }
    }

    pub fn from_diagnostics(diagnostics: Vec<FormatterDiagnostic>) -> FormattingError {
        FormattingError { diagnostics }
    }

    pub fn diagnostics(&self) -> &[FormatterDiagnostic] {
        &self.diagnostics
    }
}

/// `getMessage()`: diagnostics joined by `\n`, plus a trailing `\n`.
impl fmt::Display for FormattingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, d) in self.diagnostics.iter().enumerate() {
            if i > 0 {
                f.write_str("\n")?;
            }
            write!(f, "{d}")?;
        }
        f.write_str("\n")
    }
}

impl std::error::Error for FormattingError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatterException {
    diagnostics: Vec<FormatterDiagnostic>,
}

impl FormatterException {
    pub fn new(message: impl Into<String>) -> FormatterException {
        FormatterException {
            diagnostics: vec![FormatterDiagnostic::create(message)],
        }
    }

    pub fn diagnostics(&self) -> &[FormatterDiagnostic] {
        &self.diagnostics
    }
}

/// `getMessage()`: the first diagnostic.
impl fmt::Display for FormatterException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.diagnostics[0])
    }
}

impl std::error::Error for FormatterException {}
