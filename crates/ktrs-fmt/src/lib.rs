//! Port of ktfmt v0.64 (third_party/ktfmt/core/src/main/java/com/facebook/ktfmt) and the
//! google-java-format v1.23.0 layout engine it builds on. Output must be byte-identical to ktfmt.
//!
//! - `doc`    — google-java-format's Doc/OpsBuilder/Level engine and JavaOutput.
//! - `kdoc`   — ktfmt's KDoc comment formatter (`com.facebook.ktfmt.kdoc`).
//! - `format` — ktfmt's `format` package: tokenizer, AST visitor, import/semicolon/trailing-comma passes.
//! - `editor_config_resolver` — ktfmt's `cli/EditorConfigResolver.kt` (`--editorconfig`).

pub mod doc;
pub mod editor_config_resolver;
pub mod format;
pub mod kdoc;

pub use format::{
    FormatError, FormattingOptions, GOOGLE_FORMAT, KOTLINLANG_FORMAT, META_FORMAT, TrailingCommaManagementStrategy, format,
};
