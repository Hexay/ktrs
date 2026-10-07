//! `textDocument/formatting`'s formatters: ktfmt with the effective options.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use ktrs_fmt::editor_config_resolver::resolve_formatting_options;
use ktrs_fmt::{FormattingOptions, GOOGLE_FORMAT, KOTLINLANG_FORMAT, META_FORMAT, TrailingCommaManagementStrategy};
use ktrs_project::{KtfmtSettings, KtfmtStyle};

/// ktfmt's options: the style's, then `.editorconfig` (with `editorconfig` and a path), then the explicit settings.
pub(crate) fn ktfmt_options(settings: &KtfmtSettings, editorconfig: bool, path: Option<&Path>) -> FormattingOptions {
    let style = match settings.style {
        KtfmtStyle::Meta => META_FORMAT,
        KtfmtStyle::Google => GOOGLE_FORMAT,
        KtfmtStyle::Kotlinlang => KOTLINLANG_FORMAT,
    };
    let mut options = match path {
        Some(path) if editorconfig => resolve_formatting_options(path, &style),
        _ => style,
    };
    let positive = |n: Option<u32>| n.and_then(|n| i32::try_from(n).ok()).filter(|&n| n > 0);
    if let Some(max_width) = positive(settings.max_width) {
        options.max_width = max_width;
    }
    if let Some(block_indent) = positive(settings.block_indent) {
        options.block_indent = block_indent;
    }
    if let Some(continuation_indent) = positive(settings.continuation_indent) {
        options.continuation_indent = continuation_indent;
    }
    if let Some(remove) = settings.remove_unused_imports {
        options.remove_unused_imports = remove;
    }
    // ktfmt-gradle's deprecated `manageTrailingCommas`: true is COMPLETE, false NONE.
    if let Some(manage) = settings.manage_trailing_commas {
        options.trailing_comma_management_strategy =
            if manage { TrailingCommaManagementStrategy::Complete } else { TrailingCommaManagementStrategy::None };
    }
    options
}

/// The ktfmt-formatted text; `Err` is a message for the log.
pub(crate) fn ktfmt(text: &str, options: &FormattingOptions, name: &str) -> Result<String, String> {
    match catch_unwind(AssertUnwindSafe(|| ktrs_fmt::format(text, options))) {
        Ok(Ok(formatted)) => Ok(formatted),
        Ok(Err(e)) => Err(format!("{name}: ktfmt: {e}")),
        Err(_) => Err(format!("{name}: internal error in ktrs (please report it with this file)")),
    }
}
