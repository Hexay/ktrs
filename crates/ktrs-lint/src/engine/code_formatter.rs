//! Port of ktlint-rule-engine `CodeFormatter.kt` (the format loop) and `AutocorrectHandler.kt`.

use std::collections::HashSet;

use ktrs_ast::Ast;

use crate::engine::ktlint_rule_engine::{Code, KtLintParseException, KtLintRuleEngine, LintError, UTF8_BOM};
use crate::engine::rule_execution_context::{RuleExecutionContext, create_rule_execution_context, execute_rules};
use crate::rule::{AutocorrectDecision, RuleV2, RuleV2Provider};

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
) -> Result<String, KtLintParseException> {
    let (formatted_code, mut errors) = format_code(engine, code, autocorrect_handler, max_format_runs_per_file, after_pass)?;
    errors.sort_by_key(|(e, _)| (e.line, e.col));
    for (e, corrected) in &errors {
        callback(e, *corrected);
    }
    let bom = if code.content.starts_with(UTF8_BOM) { UTF8_BOM.to_string() } else { String::new() };
    Ok(if formatted_code.starts_with(&bom) { formatted_code } else { bom + &formatted_code })
}

fn format_code(
    engine: &KtLintRuleEngine,
    code: &Code,
    mut autocorrect_handler: AutocorrectHandler<'_>,
    max_format_runs_per_file: usize,
    after_pass: &mut dyn FnMut(&Ast),
) -> Result<(String, Vec<(LintError, bool)>), KtLintParseException> {
    let mut context = create_rule_execution_context(engine, code)?;
    let line_separator = determine_line_separator(code, EndOfLine::Lf);
    let mut code_content = formatted_code(&context, line_separator);
    let mut errors = ErrorSet::default();
    let mut format_run_count = 0;
    let mut mutated = false;
    loop {
        let new_errors = format_pass(&mut context, &mut autocorrect_handler, after_pass);
        let any_autocorrected = new_errors.order.iter().any(|(e, corrected)| e.can_be_auto_corrected && *corrected);
        new_errors.order.into_iter().for_each(|e| errors.add(e));
        if !any_autocorrected {
            break;
        }
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
    if mutated && format_run_count == max_format_runs_per_file && !matches!(autocorrect_handler, AutocorrectHandler::None) {
        lint_after_format(&mut context, after_pass);
    }
    let formatted = if mutated { formatted_code(&context, line_separator) } else { code.content.clone() };
    Ok((formatted, errors.order))
}

fn formatted_code(context: &RuleExecutionContext, line_separator: &str) -> String {
    context.ast.text(context.ast.root()).replace('\n', line_separator)
}

fn format_pass(
    context: &mut RuleExecutionContext,
    autocorrect_handler: &mut AutocorrectHandler<'_>,
    after_pass: &mut dyn FnMut(&Ast),
) -> ErrorSet {
    let mut rules = visitor_provider_rules(&context.rule_providers);
    execute_rules_collecting(context, &mut rules, autocorrect_handler, after_pass)
}

fn lint_after_format(context: &mut RuleExecutionContext, after_pass: &mut dyn FnMut(&Ast)) -> bool {
    let mut rules = visitor_provider_rules(&context.rule_providers);
    let mut has_errors_which_can_be_autocorrected = false;
    execute_rules(&mut context.ast, &mut rules, &context.editor_config, &mut |_, _, _, can_be_auto_corrected| {
        has_errors_which_can_be_autocorrected |= can_be_auto_corrected;
        AutocorrectDecision::NoAutocorrect
    });
    after_pass(&context.ast);
    has_errors_which_can_be_autocorrected
}

fn execute_rules_collecting(
    context: &mut RuleExecutionContext,
    rules: &mut [Box<dyn RuleV2>],
    autocorrect_handler: &mut AutocorrectHandler<'_>,
    after_pass: &mut dyn FnMut(&Ast),
) -> ErrorSet {
    let mut errors = ErrorSet::default();
    let locator = &context.position_in_text_locator;
    execute_rules(&mut context.ast, rules, &context.editor_config, &mut |offset, rule_id, error_message, can_be_auto_corrected| {
        let (line, col) = locator.locate(offset);
        let lint_error = LintError { line, col, rule_id, detail: error_message.to_owned(), can_be_auto_corrected };
        let autocorrect_decision = autocorrect_handler.autocorrect_decision(&lint_error);
        let autocorrect = autocorrect_decision == AutocorrectDecision::AllowAutocorrect && can_be_auto_corrected;
        errors.add((lint_error, autocorrect));
        autocorrect_decision
    });
    after_pass(&context.ast);
    errors
}

/// `VisitorProvider.rules`: fresh instances, standard rule set first, then by id (`RuleProviderSorter`).
fn visitor_provider_rules(rule_providers: &[RuleV2Provider]) -> Vec<Box<dyn RuleV2>> {
    let mut rules: Vec<Box<dyn RuleV2>> = rule_providers.iter().map(|provider| provider()).collect();
    rules.sort_by_key(|rule| (rule.rule_id().rule_set_id() != "standard", rule.rule_id().value()));
    rules
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EndOfLine {
    Lf,
    Crlf,
}

/// `determineLineSeparator`. Upstream's helper `doesNotContain` actually tests *contains*; ported as is.
fn determine_line_separator(code: &Code, eol_editor_config_property: EndOfLine) -> &'static str {
    if eol_editor_config_property == EndOfLine::Crlf
        || (eol_editor_config_property != EndOfLine::Lf && code.content.contains('\r'))
    {
        "\r\n"
    } else {
        "\n"
    }
}
