//! Port of ktlint-rule-engine `RuleExecutionContext.kt` (2.0 traversal: every rule at a node, then the
//! children, then every rule's `after` at that node).

use ktrs_ast::{Ast, NodeId};
use ktrs_parser::{FileKind, parse_file};
use ktrs_syntax::SyntaxKind::{ERROR_ELEMENT, FILE};

use crate::engine::ktlint_rule_engine::{Code, KtLintParseException, KtLintRuleEngine, UTF8_BOM};
use crate::engine::position_in_text_locator::PositionInTextLocator;
use crate::rule::{AutocorrectDecision, EditorConfig, RuleId, RuleV2, RuleV2Provider};

/// `emitAndApprove(offset, ruleId, errorMessage, canBeAutoCorrected)`; `offset` in UTF-16 units.
pub type EmitAndApprove<'a> = dyn FnMut(usize, RuleId, &str, bool) -> AutocorrectDecision + 'a;

pub(crate) struct RuleExecutionContext {
    pub(crate) ast: Ast,
    pub(crate) rule_providers: Vec<RuleV2Provider>,
    pub(crate) editor_config: EditorConfig,
    pub(crate) position_in_text_locator: PositionInTextLocator,
}

// TODO: SuppressionLocator (`@Suppress("ktlint:…")`, formatter tags) and the internal
// ktlint-suppression rule, which always runs first in its own traversal (`executeRules`).
/// One traversal of `rules` over `ast` (`RuleExecutionContext.executeRules`); public for measurements.
pub fn execute_rules(
    ast: &mut Ast,
    rules: &mut [Box<dyn RuleV2>],
    editor_config: &EditorConfig,
    emit_and_approve: &mut EmitAndApprove<'_>,
) {
    execute_rules_on_ast(ast, rules, editor_config, emit_and_approve);
}

fn execute_rules_on_ast(
    ast: &mut Ast,
    rules: &mut [Box<dyn RuleV2>],
    editor_config: &EditorConfig,
    emit_and_approve: &mut EmitAndApprove<'_>,
) {
    for rule in rules.iter_mut() {
        rule.before_first_node(editor_config);
    }
    let root = ast.root();
    execute_rules_on_node_recursively(ast, root, rules, emit_and_approve, &mut Vec::new());
    for rule in rules.iter_mut() {
        rule.after_last_node();
    }
}

/// `children` is one stack shared by the whole walk: each level pushes its `getChildren(null)` snapshot
/// and pops it when done, so the traversal allocates nothing per node.
fn execute_rules_on_node_recursively(
    ast: &mut Ast,
    node: NodeId,
    rules: &mut [Box<dyn RuleV2>],
    emit_and_approve: &mut EmitAndApprove<'_>,
    children: &mut Vec<NodeId>,
) {
    for rule in rules.iter_mut() {
        if is_replaced(ast, node) {
            return;
        }
        let rule_id = rule.rule_id();
        let mut emit = |ast: &Ast, offset: usize, message: &str, can_be_auto_corrected: bool| {
            emit_and_approve(ast.utf16_offset(ast.root(), offset), rule_id, message, can_be_auto_corrected)
        };
        rule.before_visit_child_nodes(ast, node, &mut emit);
    }
    let start = children.len();
    ast.get_children(node, children);
    let end = children.len();
    for i in start..end {
        let child = children[i];
        execute_rules_on_node_recursively(ast, child, rules, emit_and_approve, children);
    }
    children.truncate(start);
    for rule in rules.iter_mut() {
        if is_replaced(ast, node) {
            return;
        }
        let rule_id = rule.rule_id();
        let mut emit = |ast: &Ast, offset: usize, message: &str, can_be_auto_corrected: bool| {
            emit_and_approve(ast.utf16_offset(ast.root(), offset), rule_id, message, can_be_auto_corrected)
        };
        rule.after_visit_child_nodes(ast, node, &mut emit);
    }
}

/// The 2.0 bail-out: a node without parent that is not the file was replaced (e.g. `rawReplaceWithText`).
fn is_replaced(ast: &Ast, node: NodeId) -> bool {
    ast.tree_parent(node).is_none() && ast.element_type(node) != FILE
}

pub(crate) fn create_rule_execution_context(
    engine: &KtLintRuleEngine,
    code: &Code,
) -> Result<RuleExecutionContext, KtLintParseException> {
    let normalized_text = normalize_text(&code.content);
    let position_in_text_locator = PositionInTextLocator::new(&normalized_text);
    let psi_file_name = code.psi_file_name();
    let parse = parse_file(&normalized_text, FileKind::from_file_name(&psi_file_name));
    let ast = Ast::from_parse(&parse);
    if let Some(error_element) = find_error_element(&ast, ast.root()) {
        let (line, col) = position_in_text_locator.locate(ast.utf16_offset(error_element, ast.start_offset(error_element)));
        return Err(KtLintParseException { line, col, message: ast.error_description(error_element).to_owned() });
    }
    Ok(RuleExecutionContext {
        ast,
        rule_providers: engine.rule_providers.clone(),
        editor_config: EditorConfig::default(),
        position_in_text_locator,
    })
}

fn normalize_text(text: &str) -> String {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    match text.strip_prefix(UTF8_BOM) {
        Some(rest) => rest.to_owned(),
        None => text,
    }
}

/// The first `PsiErrorElement` depth-first; error elements are composites, so the PSI walk over
/// `children` and a node walk agree.
fn find_error_element(ast: &Ast, node: NodeId) -> Option<NodeId> {
    if ast.element_type(node) == ERROR_ELEMENT && !ast.is_leaf_element(node) {
        return Some(node);
    }
    let mut child = ast.first_child_node(node);
    while let Some(c) = child {
        if let Some(error_element) = find_error_element(ast, c) {
            return Some(error_element);
        }
        child = ast.tree_next(c);
    }
    None
}
