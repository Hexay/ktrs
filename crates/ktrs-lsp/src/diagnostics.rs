//! ktlint lint errors as LSP diagnostics.

use std::borrow::Cow;
use std::path::Path;

use ktrs_lint::LintError;
use ktrs_parser::FileKind;
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};
use serde_json::json;

use crate::text::LineIndex;

pub(crate) const SOURCE: &str = "ktlint";

/// One diagnostic per error, in order.
pub(crate) fn diagnostics(text: &str, path: Option<&Path>, errors: &[LintError], unfixable_as_error: bool) -> Vec<Diagnostic> {
    if errors.is_empty() {
        return Vec::new();
    }
    // ktlint's lines and columns are those of the LF-normalized text; dropping a line's `\r` moves nothing before it.
    let text: Cow<str> = if text.contains('\r') { Cow::Owned(text.replace("\r\n", "\n")) } else { Cow::Borrowed(text) };
    let index = LineIndex::new(&text);
    let tokens = token_ranges(&text, path);
    errors
        .iter()
        .map(|error| {
            let severity = if unfixable_as_error && !error.can_be_auto_corrected { DiagnosticSeverity::ERROR } else { DiagnosticSeverity::WARNING };
            Diagnostic {
                range: token_range(&index, &tokens, error),
                severity: Some(severity),
                code: Some(NumberOrString::String(error.rule_id.value().to_owned())),
                source: Some(SOURCE.to_owned()),
                message: error.detail.clone(),
                data: Some(json!({ "autocorrectable": error.can_be_auto_corrected })),
                ..Diagnostic::default()
            }
        })
        .collect()
}

/// The byte ranges of the file's tokens, in order.
fn token_ranges(text: &str, path: Option<&Path>) -> Vec<(usize, usize)> {
    let name = path.and_then(|p| p.file_name()).map_or(Cow::Borrowed("File.kt"), |n| n.to_string_lossy());
    let tree = ktrs_parser::parse_file(text, FileKind::from_file_name(&name)).tree;
    (0..tree.len() as u32)
        .filter(|&e| tree.is_token(e))
        .map(|e| {
            let range = tree.text_range(e);
            (usize::from(range.start()), usize::from(range.end()))
        })
        .collect()
}

/// The whole token at the error's line and column (as the IntelliJ ktlint plugin's `findElementAt`), cut to that
/// line; zero-width without one.
fn token_range(index: &LineIndex, tokens: &[(usize, usize)], error: &LintError) -> Range {
    let line = error.line.saturating_sub(1) as u32;
    let offset = index.offset(Position::new(line, error.col.saturating_sub(1) as u32));
    let token = tokens.partition_point(|&(_, end)| end <= offset);
    let (start, end) = match tokens.get(token) {
        Some(&(token_start, token_end)) if token_start <= offset => {
            (token_start.max(index.offset(Position::new(line, 0))), token_end.min(index.line_end(offset)).max(offset))
        }
        _ => (offset, offset),
    };
    Range::new(index.position(start), index.position(end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ktrs_lint::RuleId;

    fn error(line: usize, col: usize) -> LintError {
        LintError { line, col, rule_id: RuleId("standard:x"), detail: "d".to_owned(), can_be_auto_corrected: true }
    }

    #[test]
    fn ranges_cover_the_token_and_stop_at_the_line_end() {
        let text = "val  foo = 1\r\n/* a\nb */\n";
        let errors = [error(1, 5), error(1, 7), error(2, 1), error(3, 2), error(1, 13), error(9, 1)];
        let ranges: Vec<Range> = diagnostics(text, None, &errors, false).into_iter().map(|d| d.range).collect();
        let r = |l, a, b| Range::new(Position::new(l, a), Position::new(l, b));
        assert_eq!(ranges, [r(0, 3, 5), r(0, 5, 8), r(1, 0, 4), r(2, 0, 4), r(0, 12, 12), r(3, 0, 0)]);
    }
}
