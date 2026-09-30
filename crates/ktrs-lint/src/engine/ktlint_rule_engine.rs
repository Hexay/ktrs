//! Port of ktlint-rule-engine's API types: `KtLintRuleEngine` (lint/format), `Code`, `LintError`,
//! `KtLintParseException`.

use ktrs_ast::Ast;

use crate::engine::code_formatter::{AutocorrectHandler, format};
use crate::rule::{AutocorrectDecision, RuleId, RuleV2Provider};

pub const MAX_FORMAT_RUNS_PER_FILE: usize = 3;
pub const UTF8_BOM: char = '\u{FEFF}';

/// `Code`: the content plus what the engine derives the PSI file name and script-ness from.
#[derive(Clone, Debug)]
pub struct Code {
    pub content: String,
    /// `filePath.pathString`; `None` for a snippet.
    pub file_path: Option<String>,
    pub script: bool,
}

impl Code {
    pub fn from_snippet(content: &str, script: bool) -> Code {
        Code { content: content.to_owned(), file_path: None, script }
    }

    /// `Code.fromFile(file)` with the file already read.
    pub fn from_file(path: &str, content: String) -> Code {
        let script = path.to_ascii_lowercase().ends_with(".kts");
        Code { content, file_path: Some(path.to_owned()), script }
    }

    pub(crate) fn psi_file_name(&self) -> String {
        match &self.file_path {
            Some(path) => path.clone(),
            None if self.script => "File.kts".to_owned(),
            None => "File.kt".to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LintError {
    pub line: usize,
    pub col: usize,
    pub rule_id: RuleId,
    pub detail: String,
    pub can_be_auto_corrected: bool,
}

/// `KtLintParseException`: the file has a `PsiErrorElement`; no rule ran.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KtLintParseException {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

pub struct KtLintRuleEngine {
    pub rule_providers: Vec<RuleV2Provider>,
}

impl KtLintRuleEngine {
    /// `lint(code, callback)`: one pass, no autocorrect; errors sorted by (line, col), duplicates dropped.
    pub fn lint(&self, code: &Code, callback: &mut dyn FnMut(&LintError)) -> Result<(), KtLintParseException> {
        format(self, code, AutocorrectHandler::None, &mut |e, _| callback(e), 1, &mut |_| {}).map(drop)
    }

    /// `format(code, callback)`: up to [`MAX_FORMAT_RUNS_PER_FILE`] passes; `callback` is asked for every
    /// error in emit order (duplicates across passes included) and decides whether it is autocorrected.
    pub fn format(
        &self,
        code: &Code,
        callback: &mut dyn FnMut(&LintError) -> AutocorrectDecision,
    ) -> Result<String, KtLintParseException> {
        self.format_observed(code, callback, &mut |_| {})
    }

    /// [`Self::format`], calling `after_pass` with the tree at the end of every rule traversal (what a
    /// last-sorted probe rule sees in `afterLastNode`, lint-after-format included).
    pub fn format_observed(
        &self,
        code: &Code,
        callback: &mut dyn FnMut(&LintError) -> AutocorrectDecision,
        after_pass: &mut dyn FnMut(&Ast),
    ) -> Result<String, KtLintParseException> {
        let handler = AutocorrectHandler::LintErrorAutocorrectHandler(callback);
        format(self, code, handler, &mut |_, _| {}, MAX_FORMAT_RUNS_PER_FILE, after_pass)
    }
}
