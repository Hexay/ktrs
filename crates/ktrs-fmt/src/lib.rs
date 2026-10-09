//! Port of ktfmt v0.65 (third_party/ktfmt/core/src/main/kotlin/org/jetbrains/kotlinx/ktfmt) and the
//! google-java-format v1.23.0 layout engine it builds on. Output must be byte-identical to ktfmt.
//!
//! - `doc`    — google-java-format's Doc/OpsBuilder/Level engine and JavaOutput.
//! - `kdoc`   — ktfmt's KDoc comment formatter (`org.jetbrains.kotlinx.ktfmt.kdoc`).
//! - `format` — ktfmt's `format` package: tokenizer, AST visitor, import/semicolon/trailing-comma passes.
//! - `editor_config_resolver` — ktfmt's `cli/EditorConfigResolver.kt` (`--editorconfig`).

pub mod doc;
pub mod editor_config_resolver;
pub mod format;
pub mod kdoc;

pub use doc::{Range, RangeSet};
pub use format::{
    FileType, FormatError, FormattingOptions, GOOGLE_FORMAT, KOTLINLANG_FORMAT, KotlinCode, META_FORMAT,
    TrailingCommaManagementStrategy, format, format_code,
};
