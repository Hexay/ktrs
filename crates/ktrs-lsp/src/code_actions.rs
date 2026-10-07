//! `textDocument/codeAction` for ktlint errors: fix one, suppress (line or file), fix all.

use std::collections::{HashMap, HashSet};

use ktrs_lint::{Code, KtLintRuleEngine, LintError};
use lsp_types::{CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, Range, Uri, WorkspaceEdit};

use crate::ktlint::{fix_all, fix_one, suppress};
use crate::text::text_edits;

pub(crate) const FIX_ALL_KIND: &str = "source.fixAll.ktlint";

/// A linted document: its errors and their diagnostics, pairwise.
pub(crate) struct Linted<'a> {
    pub uri: &'a Uri,
    pub engine: &'a KtLintRuleEngine,
    pub code: &'a Code,
    pub errors: &'a [LintError],
    pub diagnostics: &'a [Diagnostic],
}

/// The actions for the errors whose range touches `range`, plus fix-all, filtered by `only` (LSP's kind prefixes).
/// Each action is computed eagerly: an edit that fails or changes nothing is left out.
pub(crate) fn code_actions(linted: &Linted, range: Range, only: Option<&[CodeActionKind]>) -> Vec<CodeActionOrCommand> {
    let wanted = |kind: &str| only.is_none_or(|only| only.iter().any(|o| kind == o.as_str() || kind.starts_with(&format!("{}.", o.as_str()))));
    let mut actions = Vec::new();
    if wanted(CodeActionKind::QUICKFIX.as_str()) {
        let mut suppressed_in_file = HashSet::new();
        for (error, diagnostic) in linted.errors.iter().zip(linted.diagnostics) {
            if !touches(diagnostic.range, range) {
                continue;
            }
            let rule = error.rule_id.value();
            if error.can_be_auto_corrected {
                let edit = fix_one(linted.engine, linted.code, error);
                actions.extend(action(linted, format!("Fix {rule}"), CodeActionKind::QUICKFIX, Some(diagnostic), edit.ok(), true));
            }
            let line = suppress(linted.engine, linted.code, error, false);
            actions.extend(action(linted, format!("Suppress {rule} on this line"), CodeActionKind::QUICKFIX, Some(diagnostic), line.ok(), false));
            if suppressed_in_file.insert(rule) {
                let file = suppress(linted.engine, linted.code, error, true);
                actions.extend(action(linted, format!("Suppress {rule} in this file"), CodeActionKind::QUICKFIX, Some(diagnostic), file.ok(), false));
            }
        }
    }
    if wanted(FIX_ALL_KIND) && linted.errors.iter().any(|e| e.can_be_auto_corrected) {
        let fixed = fix_all(linted.engine, linted.code);
        let title = "Fix all autocorrectable ktlint violations".to_owned();
        actions.extend(action(linted, title, CodeActionKind::new(FIX_ALL_KIND), None, fixed.ok(), false));
    }
    actions
}

fn touches(a: Range, b: Range) -> bool {
    a.start <= b.end && b.start <= a.end
}

fn action(
    linted: &Linted,
    title: String,
    kind: CodeActionKind,
    diagnostic: Option<&Diagnostic>,
    new_text: Option<String>,
    preferred: bool,
) -> Option<CodeActionOrCommand> {
    let edits = text_edits(&linted.code.content, &new_text?);
    if edits.is_empty() {
        return None;
    }
    Some(CodeActionOrCommand::CodeAction(CodeAction {
        title,
        kind: Some(kind),
        diagnostics: diagnostic.map(|d| vec![d.clone()]),
        edit: Some(WorkspaceEdit { changes: Some(HashMap::from([(linted.uri.clone(), edits)])), ..WorkspaceEdit::default() }),
        is_preferred: preferred.then_some(true),
        ..CodeAction::default()
    }))
}
