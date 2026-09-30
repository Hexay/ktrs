//! Port of ktlint-ruleset-standard `MultilineExpressionWrappingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    self, ARRAY_ACCESS_EXPRESSION, ARROW, BINARY_EXPRESSION, BINARY_WITH_TYPE, BLOCK, CALL_EXPRESSION, COMMA,
    DOT_QUALIFIED_EXPRESSION, ELVIS, EQ, FUN, FUNCTION_LITERAL, IF, IS_EXPRESSION, LAMBDA_EXPRESSION, MUL, OBJECT_LITERAL,
    OPERATION_REFERENCE, POSTFIX_EXPRESSION, PREFIX_EXPRESSION, REFERENCE_EXPRESSION, REGULAR_STRING_PART, RPAR,
    SAFE_ACCESS_EXPRESSION, TRY, VALUE_ARGUMENT, VALUE_ARGUMENT_LIST, VALUE_PARAMETER_LIST, WHEN,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines};
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::function_signature::{FUNCTION_BODY_EXPRESSION_WRAPPING_PROPERTY, FunctionBodyExpressionWrapping};

// Based on https://kotlinlang.org/spec/expressions.html#expressions
const CHAINABLE_EXPRESSION: [SyntaxKind; 13] = [
    ARRAY_ACCESS_EXPRESSION,
    BINARY_WITH_TYPE,
    CALL_EXPRESSION,
    DOT_QUALIFIED_EXPRESSION,
    IF,
    IS_EXPRESSION,
    OBJECT_LITERAL,
    PREFIX_EXPRESSION,
    POSTFIX_EXPRESSION,
    REFERENCE_EXPRESSION,
    SAFE_ACCESS_EXPRESSION,
    TRY,
    WHEN,
];

/// Wraps each multiline expression to a new line.
pub struct MultilineExpressionWrappingRule {
    indent_config: IndentConfig,
    function_body_expression_wrapping: FunctionBodyExpressionWrapping,
}

impl MultilineExpressionWrappingRule {
    pub fn new() -> MultilineExpressionWrappingRule {
        MultilineExpressionWrappingRule {
            indent_config: IndentConfig::default_indent_config(),
            function_body_expression_wrapping: FUNCTION_BODY_EXPRESSION_WRAPPING_PROPERTY.default_value,
        }
    }
}

impl Default for MultilineExpressionWrappingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for MultilineExpressionWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:multiline-expression-wrapping")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![
            PropertyRef::from(&*INDENT_SIZE_PROPERTY),
            PropertyRef::from(&*INDENT_STYLE_PROPERTY),
            PropertyRef::from(&*FUNCTION_BODY_EXPRESSION_WRAPPING_PROPERTY),
        ]
    }

    fn is_official_code_style(&self) -> bool {
        true
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        self.function_body_expression_wrapping = editor_config.get(&FUNCTION_BODY_EXPRESSION_WRAPPING_PROPERTY);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let parent_type = ast.parent(node).map(|p| ast.element_type(p));
        if CHAINABLE_EXPRESSION.contains(&ast.element_type(node))
            && !is_part_of_spread_operator_expression(ast, node)
            && (parent_type.is_none_or(|t| !CHAINABLE_EXPRESSION.contains(&t)) || is_right_hand_side_of_binary_expression(ast, node))
        {
            self.visit_expression(ast, node, emit);
        }
        if ast.element_type(node) == BINARY_EXPRESSION && ast.parent(node).map(|p| ast.element_type(p)) != Some(BINARY_EXPRESSION) {
            self.visit_expression(ast, node, emit);
        }
    }
}

fn is_part_of_spread_operator_expression(ast: &Ast, node: NodeId) -> bool {
    ast.prev_code_leaf(node).map(|it| ast.element_type(it)) == Some(MUL)
        && ast.parent(node).map(|p| ast.element_type(p)) == Some(VALUE_ARGUMENT)
}

impl MultilineExpressionWrappingRule {
    fn visit_expression(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if !(contains_whitespace_with_newline(ast, node) && self.need_to_wrap_multiline_expression(ast, node)) {
            return;
        }
        let prev_leaf = ast.prev_leaf_matching(node, |it| !ast.is_part_of_comment(it));
        if prev_leaf.is_none() || ast.is_white_space_with_newline(prev_leaf) {
            return;
        }
        let indent_config = &self.indent_config;
        emit(ast, ast.start_offset(node), "A multiline expression should start on a new line", true).if_autocorrect_allowed(|| {
            let indent = indent_config.sibling_indent_of(ast, node);
            ast.upsert_whitespace_before_me(node, &indent);
            let last_child_leaf = ast.last_child_leaf_or_self(node);
            let leaf_on_same_line_after_multiline_expression = ast
                .next_leaf_matching(last_child_leaf, |it| !ast.is_white_space_without_newline(it) && !ast.is_part_of_comment(it))
                .filter(|&it| !ast.is_white_space_with_newline(it));
            let Some(leaf) = leaf_on_same_line_after_multiline_expression else { return };
            let leaf_parent_type = ast.parent(leaf).map(|p| ast.element_type(p));
            if leaf_parent_type == Some(OPERATION_REFERENCE) {
                // Each wrapped binary expression is checked for being multiline on its own
            } else if ast.element_type(leaf) == COMMA
                && (leaf_parent_type == Some(VALUE_ARGUMENT_LIST) || leaf_parent_type == Some(VALUE_PARAMETER_LIST))
            {
                // Keep the comma on the same line as the multiline expression
                if let Some(next_leaf) = ast.next_leaf(leaf) {
                    let indent = indent_config.sibling_indent_of(ast, node);
                    ast.upsert_whitespace_before_me(next_leaf, &indent);
                }
            } else {
                let indent = indent_config.sibling_indent_of(ast, node);
                ast.upsert_whitespace_before_me(leaf, &indent);
            }
        });
    }

    fn need_to_wrap_multiline_expression(&self, ast: &Ast, node: NodeId) -> bool {
        self.is_value_in_an_assignment(ast, node)
            || is_lambda_expression(ast, node)
            || is_value_argument(ast, node)
            || is_after_arrow(ast, node)
    }

    fn is_value_in_an_assignment(&self, ast: &Ast, node: NodeId) -> bool {
        ast.prev_code_sibling(node)
            .filter(|&it| matches!(ast.element_type(it), EQ | OPERATION_REFERENCE))
            .filter(|&it| {
                !(self.function_body_expression_wrapping == FunctionBodyExpressionWrapping::Default
                    && ast.parent(it).map(|p| ast.element_type(p)) == Some(FUN))
            })
            .filter(|&it| !is_elvis_operator(ast, Some(it)))
            .filter(|&it| !ast.is_white_space_with_newline(closing_parenthesis_of_function_or_null(ast, it).and_then(|rpar| ast.prev_leaf(rpar))))
            .is_some()
    }
}

fn contains_whitespace_with_newline(ast: &Ast, node: NodeId) -> bool {
    let last_leaf = ast.last_child_leaf_or_self(node);
    ast.leaves_forwards_including_self(ast.first_child_leaf_or_self(node))
        .take_while(|&it| it != last_leaf)
        .any(|it| ast.is_white_space_with_newline(it) || is_regular_string_part_with_newline(ast, it))
}

fn is_regular_string_part_with_newline(ast: &Ast, node: NodeId) -> bool {
    ast.element_type(node) == REGULAR_STRING_PART && ast.text(node).starts_with('\n')
}

fn is_elvis_operator(ast: &Ast, node: Option<NodeId>) -> bool {
    node.is_some_and(|n| {
        ast.element_type(n) == OPERATION_REFERENCE
            && ast.element_type(ast.first_child_node(n).expect("NullPointerException: firstChildNode")) == ELVIS
    })
}

fn closing_parenthesis_of_function_or_null(ast: &Ast, node: NodeId) -> Option<NodeId> {
    Some(node)
        .filter(|&it| ast.parent(it).map(|p| ast.element_type(p)) == Some(FUN))
        .and_then(|it| ast.prev_code_leaf(it))
        .filter(|&it| ast.element_type(it) == RPAR)
}

/// Function literals in a lambda have an implicit block (no braces), so only its first node is wrapped.
fn is_lambda_expression(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .filter(|&it| ast.element_type(it) == BLOCK && ast.first_child_node(it) == Some(node))
        .and_then(|it| ast.parent(it))
        .filter(|&it| ast.element_type(it) == FUNCTION_LITERAL)
        .and_then(|it| ast.parent(it))
        .is_some_and(|it| ast.element_type(it) == LAMBDA_EXPRESSION)
}

fn is_value_argument(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node).map(|p| ast.element_type(p)) == Some(VALUE_ARGUMENT)
}

fn is_after_arrow(ast: &Ast, node: NodeId) -> bool {
    ast.prev_code_leaf(node).map(|it| ast.element_type(it)) == Some(ARROW)
}

fn is_right_hand_side_of_binary_expression(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node).map(|p| ast.element_type(p)) == Some(BINARY_EXPRESSION)
        && ast.prev_code_sibling(node).map(|it| ast.element_type(it)) == Some(OPERATION_REFERENCE)
}
