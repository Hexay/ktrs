//! Port of ktlint-ruleset-standard `ExpressionOperandWrappingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::{ANDAND, BINARY_EXPRESSION, DIV, EOL_COMMENT, MINUS, MUL, OPERATION_REFERENCE, OROR, PLUS};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines};
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

const WRAPPABLE_OPERAND: TokenSet = TokenSet::create(&[ANDAND, OROR, PLUS, MINUS, MUL, DIV]);

/// Wraps each operand of a multiline expression (sub-expressions ignored) to a new line: `foo || bar ||\n baz` becomes
/// `foo ||\n bar ||\n baz`.
pub struct ExpressionOperandWrappingRule {
    indent_config: IndentConfig,
}

impl ExpressionOperandWrappingRule {
    pub fn new() -> ExpressionOperandWrappingRule {
        ExpressionOperandWrappingRule { indent_config: IndentConfig::default_indent_config() }
    }
}

impl Default for ExpressionOperandWrappingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for ExpressionOperandWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:expression-operand-wrapping")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*INDENT_SIZE_PROPERTY), PropertyRef::from(&*INDENT_STYLE_PROPERTY)]
    }

    fn is_experimental(&self) -> bool {
        true
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if is_binary_expression_with_wrappable_operand(ast, node) && is_multiline(ast, node) {
            self.visit_multiline_binary_expression(ast, node, emit);
        }
    }
}

fn is_binary_expression_with_wrappable_operand(ast: &Ast, node: NodeId) -> bool {
    Some(node)
        .filter(|&it| ast.element_type(it) == BINARY_EXPRESSION)
        .and_then(|it| ast.find_child_by_type(it, OPERATION_REFERENCE))
        .and_then(|it| ast.first_child_node(it))
        .is_some_and(|it| WRAPPABLE_OPERAND.contains(ast.element_type(it)))
}

impl ExpressionOperandWrappingRule {
    fn visit_multiline_binary_expression(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let Some(operation_reference) = ast.find_child_by_type(node, OPERATION_REFERENCE) else { return };
        let next_sibling = ast.next_sibling(operation_reference);
        if ast.is_white_space_with_newline(next_sibling) {
            return;
        }
        if ast.is_white_space_without_newline(next_sibling)
            && next_sibling.and_then(|it| ast.next_sibling(it)).map(|it| ast.element_type(it)) == Some(EOL_COMMENT)
        {
            return;
        }
        let start_offset = ast.start_offset(operation_reference)
            + ast.text_length(operation_reference)
            + next_sibling.filter(|&it| ast.is_white_space(it)).map_or(0, |it| ast.text_length(it));
        let indent_config = &self.indent_config;
        emit(ast, start_offset, "Newline expected before operand in multiline expression", true).if_autocorrect_allowed(|| {
            let indent = indent_config.sibling_indent_of(ast, node);
            ast.upsert_whitespace_after_me(operation_reference, &indent);
        });
    }
}

fn is_multiline(ast: &Ast, node: NodeId) -> bool {
    if is_multiline_operand(ast, left_hand_side(ast, node)) || is_multiline_operand(ast, right_hand_side(ast, node)) {
        return true;
    }

    any_parent_binary_expression(ast, node, |parent| ast.children(parent).any(|it| ast.is_white_space_with_newline(it)))
}

fn left_hand_side(ast: &Ast, node: NodeId) -> Option<NodeId> {
    ast.children(node).find(|&it| ast.is_code(it))
}

fn right_hand_side(ast: &Ast, node: NodeId) -> Option<NodeId> {
    ast.children(node).filter(|&it| ast.is_code(it)).last()
}

fn is_multiline_operand(ast: &Ast, node: Option<NodeId>) -> bool {
    match node {
        None => false,
        Some(node) => ast
            .leaves_in_closed_range(ast.first_child_leaf_or_self(node), ast.last_child_leaf_or_self(node))
            .any(|it| ast.is_white_space_with_newline(it)),
    }
}

fn any_parent_binary_expression(ast: &Ast, node: NodeId, predicate: impl Fn(NodeId) -> bool) -> bool {
    ast.parents(node).take_while(|&it| is_binary_expression_with_wrappable_operand(ast, it)).any(predicate)
}
