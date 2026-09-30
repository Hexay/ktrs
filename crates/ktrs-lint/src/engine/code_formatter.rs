//! Port of ktlint-rule-engine `internal/CodeFormatter.kt` (the format loop) and `AutocorrectHandler.kt`.

use std::collections::HashSet;

use ktrs_ast::Ast;

use crate::editorconfig::{END_OF_LINE_PROPERTY, EndOfLineValue};
use crate::engine::code::{Code, KtLintException, LintError};
use crate::engine::ktlint_rule_engine::{KtLintRuleEngine, UTF8_BOM};
use crate::engine::rule_execution_context::{RuleExecutionContext, create_rule_execution_context};
use crate::engine::visitor_provider::VisitorProvider;
use crate::rule::AutocorrectDecision;

pub(crate) enum AutocorrectHandler<'a> {
    None,
    LintErrorAutocorrectHandler(&'a mut dyn FnMut(&LintError) -> AutocorrectDecision),
}

impl AutocorrectHandler<'_> {
    fn autocorrect_decision(&mut self, lint_error: &LintError) -> AutocorrectDecision {
        match self {
            AutocorrectHandler::None => AutocorrectDecision::NoAutocorrect,
            AutocorrectHandler::LintErrorAutocorrectHandler(callback) => callback(lint_error),
        }
    }

    fn is_none(&self) -> bool {
        matches!(self, AutocorrectHandler::None)
    }
}

/// `mutableSetOf<Pair<LintError, Boolean>>()`: insertion-ordered, duplicates dropped.
#[derive(Default)]
struct ErrorSet {
    order: Vec<(LintError, bool)>,
    seen: HashSet<(LintError, bool)>,
}

impl ErrorSet {
    fn add(&mut self, error: (LintError, bool)) {
        if self.seen.insert(error.clone()) {
            self.order.push(error);
        }
    }
}

/// `format(code, autocorrectHandler, callback, maxFormatRunsPerFile)`.
pub(crate) fn format(
    engine: &KtLintRuleEngine,
    code: &Code,
    autocorrect_handler: AutocorrectHandler<'_>,
    callback: &mut dyn FnMut(&LintError, bool),
    max_format_runs_per_file: usize,
    after_pass: &mut dyn FnMut(&Ast),
) -> Result<String, KtLintException> {
    let (formatted_code, mut errors) = format_code(
        engine,
        code,
        autocorrect_handler,
        max_format_runs_per_file,
        after_pass,
    )?;
    errors.sort_by_key(|(e, _)| (e.line, e.col));
    for (e, corrected) in &errors {
        callback(e, *corrected);
    }
    let bom = if code.content.starts_with(UTF8_BOM) {
        UTF8_BOM.to_string()
    } else {
        String::new()
    };
    Ok(if formatted_code.starts_with(&bom) {
        formatted_code
    } else {
        bom + &formatted_code
    })
}

fn format_code(
    engine: &KtLintRuleEngine,
    code: &Code,
    mut autocorrect_handler: AutocorrectHandler<'_>,
    max_format_runs_per_file: usize,
    after_pass: &mut dyn FnMut(&Ast),
) -> Result<(String, Vec<(LintError, bool)>), KtLintException> {
    let mut context = create_rule_execution_context(engine, code)?;
    let line_separator =
        determine_line_separator(code, context.editor_config.get(&END_OF_LINE_PROPERTY));
    let mut code_content = formatted_code(&context, line_separator);
    let mut errors = ErrorSet::default();
    let mut format_run_count = 0;
    let mut mutated = false;
    loop {
        let new_errors = format_pass(&mut context, &mut autocorrect_handler, after_pass)?;
        let any_autocorrected = new_errors
            .order
            .iter()
            .any(|(e, corrected)| e.can_be_auto_corrected && *corrected);
        new_errors.order.into_iter().for_each(|e| errors.add(e));
        if !any_autocorrected {
            break;
        }
        // Rule errors can cancel out, so the text decides whether the code changed.
        let updated_code_content = formatted_code(&context, line_separator);
        if updated_code_content == code_content {
            break;
        }
        code_content = updated_code_content;
        mutated = true;
        format_run_count += 1;
        if format_run_count >= max_format_runs_per_file {
            break;
        }
    }
    if mutated && format_run_count == max_format_runs_per_file && !autocorrect_handler.is_none() {
        lint_after_format(&mut context, after_pass)?;
    }
    let formatted = if mutated {
        formatted_code(&context, line_separator)
    } else {
        code.content.clone()
    };
    Ok((formatted, errors.order))
}

fn formatted_code(context: &RuleExecutionContext, line_separator: &str) -> String {
    context
        .ast
        .text(context.ast.root())
        .replace('\n', line_separator)
}

fn format_pass(
    context: &mut RuleExecutionContext,
    autocorrect_handler: &mut AutocorrectHandler<'_>,
    after_pass: &mut dyn FnMut(&Ast),
) -> Result<ErrorSet, KtLintException> {
    let rules = VisitorProvider::new(&context.rule_providers).rules();
    execute_rules_collecting(context, rules, autocorrect_handler, after_pass)
}

/// One more lint pass after the last allowed format pass: it only informs (upstream logs a warning).
fn lint_after_format(
    context: &mut RuleExecutionContext,
    after_pass: &mut dyn FnMut(&Ast),
) -> Result<bool, KtLintException> {
    let rules = VisitorProvider::new(&context.rule_providers).rules();
    let mut has_errors_which_can_be_autocorrected = false;
    context.execute_rules(rules, true, &mut |_, _, _, can_be_auto_corrected| {
        has_errors_which_can_be_autocorrected |= can_be_auto_corrected;
        AutocorrectDecision::NoAutocorrect
    })?;
    after_pass(&context.ast);
    Ok(has_errors_which_can_be_autocorrected)
}

fn execute_rules_collecting(
    context: &mut RuleExecutionContext,
    rules: Vec<Box<dyn crate::rule::RuleV2>>,
    autocorrect_handler: &mut AutocorrectHandler<'_>,
    after_pass: &mut dyn FnMut(&Ast),
) -> Result<ErrorSet, KtLintException> {
    let mut errors = ErrorSet::default();
    let lint_mode = autocorrect_handler.is_none();
    let locator = context.position_in_text_locator.clone();
    context.execute_rules(
        rules,
        lint_mode,
        &mut |offset, rule_id, error_message, can_be_auto_corrected| {
            let (line, col) = locator.locate(offset);
            let lint_error = LintError {
                line,
                col,
                rule_id,
                detail: error_message.to_owned(),
                can_be_auto_corrected,
            };
            // Always asked, even when the error can't be autocorrected, so the consumer sees every error.
            let autocorrect_decision = autocorrect_handler.autocorrect_decision(&lint_error);
            // A rule that got the approval is assumed to have fixed the error.
            let autocorrect = autocorrect_decision == AutocorrectDecision::AllowAutocorrect
                && can_be_auto_corrected;
            errors.add((lint_error, autocorrect));
            autocorrect_decision
        },
    )?;
    after_pass(&context.ast);
    Ok(errors)
}

/// `determineLineSeparator`. Upstream's helper `doesNotContain` actually tests *contains*; ported as is.
fn determine_line_separator(
    code: &Code,
    eol_editor_config_property: EndOfLineValue,
) -> &'static str {
    if eol_editor_config_property == EndOfLineValue::Crlf
        || (eol_editor_config_property != EndOfLineValue::Lf && code.content.contains('\r'))
    {
        "\r\n"
    } else {
        "\n"
    }
}
