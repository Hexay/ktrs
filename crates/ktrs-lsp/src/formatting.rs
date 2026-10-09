//! `textDocument/formatting` and `rangeFormatting`'s formatters: ktfmt with the effective options.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use ktrs_fmt::editor_config_resolver::resolve_formatting_options;
use ktrs_fmt::{
    FileType, FormattingOptions, GOOGLE_FORMAT, KOTLINLANG_FORMAT, KotlinCode, META_FORMAT, RangeSet,
    TrailingCommaManagementStrategy,
};
use ktrs_project::{KtfmtSettings, KtfmtStyle};

use crate::text::LineIndex;

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

/// The ktfmt-formatted text; `Err` is a message for the log. With `range`, ktfmt's partial formatting
/// (`--offset`/`--length`): only the statements the range touches are pretty-printed, then the
/// whole-file cleanups (imports, redundant elements, multiline strings) run.
pub(crate) fn ktfmt(
    text: &str,
    options: &FormattingOptions,
    path: Option<&Path>,
    range: Option<lsp_types::Range>,
    name: &str,
) -> Result<String, String> {
    let formatted = catch_unwind(AssertUnwindSafe(|| {
        let code = KotlinCode::from(text, FileType::of_file_or_script(path))?;
        let character_ranges = range.map(|range| character_ranges(&code, range));
        ktrs_fmt::format_code(options, &code, character_ranges.as_ref())
    }));
    match formatted {
        Ok(Ok(formatted)) => Ok(formatted),
        Ok(Err(e)) => Err(format!("{name}: ktfmt: {e}")),
        Err(_) => Err(format!("{name}: internal error in ktrs (please report it with this file)")),
    }
}

/// `range` as a ktfmt character range over the normalized code; an empty one selects the character
/// at the cursor (ktfmt's `--length=0`: its line's statement).
fn character_ranges(code: &KotlinCode, range: lsp_types::Range) -> RangeSet {
    let text = code.code.as_str();
    let index = LineIndex::new(text);
    let (mut start, mut end) = (index.offset(range.start), index.offset(range.end));
    if end <= start {
        end = start;
        match text[start..].chars().next() {
            Some(c) => end += c.len_utf8(),
            None => start -= text[..start].chars().next_back().map_or(0, char::len_utf8),
        }
    }
    let mut character_ranges = RangeSet::create();
    character_ranges.add(ktrs_fmt::Range::closed_open(start as i32, end as i32));
    character_ranges
}
