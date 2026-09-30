//! Port of ktlint-ruleset-standard `CallExpressionWrappingRule.kt` (id `call-expression-wrapping`, experimental).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ARROW, CALL_EXPRESSION, FUNCTION_LITERAL, LAMBDA_ARGUMENT, LAMBDA_EXPRESSION, LBRACE, LPAR, RBRACE, REFERENCE_EXPRESSION, RPAR,
    VALUE_ARGUMENT_LIST,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines, AstNodeQueries};
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, MAX_LINE_LENGTH_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

pub struct CallExpressionWrappingRule {
    indent_config: IndentConfig,
    max_line_length: i32,
}

impl CallExpressionWrappingRule {
    pub fn new() -> CallExpressionWrappingRule {
        CallExpressionWrappingRule {
            indent_config: IndentConfig::default_indent_config(),
            max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value,
        }
    }
}

impl Default for CallExpressionWrappingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for CallExpressionWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:call-expression-wrapping")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![
            PropertyRef::from(&*INDENT_SIZE_PROPERTY),
            PropertyRef::from(&*INDENT_STYLE_PROPERTY),
            PropertyRef::from(&*MAX_LINE_LENGTH_PROPERTY),
        ]
    }

    fn is_experimental(&self) -> bool {
        true
    }

    // Upstream reads the raw property here, not `maxLineLength()`.
    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        self.max_line_length = editor_config.get(&MAX_LINE_LENGTH_PROPERTY);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == CALL_EXPRESSION {
            self.visit_call_expression(ast, node, emit);
        }
    }
}

impl CallExpressionWrappingRule {
    fn visit_call_expression(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if let Some(it) = ast
            .find_child_by_type(node, REFERENCE_EXPRESSION)
            .and_then(|it| ast.next_sibling_matching(it, |it| ast.element_type(it) == VALUE_ARGUMENT_LIST))
        {
            self.visit_reference_expression_value_argument_list(ast, it, emit);
        }
        if let Some(it) = ast.find_child_by_type(node, LAMBDA_ARGUMENT) {
            self.visit_lambda_argument(ast, it, emit);
        }
    }

    fn visit_reference_expression_value_argument_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        require(ast.element_type(node) == VALUE_ARGUMENT_LIST);
        if ast.text_contains(node, '\n') || self.exceeds_max_line_length(ast, node, ast.last_child_leaf_or_self(node)) {
            if let Some(lpar) =
                ast.find_child_by_type(node, LPAR).filter(|&it| !ast.is_white_space_with_newline(ast.next_sibling(it)))
            {
                emit(ast, ast.start_offset(lpar), "Expected new line after '('", true).if_autocorrect_allowed(|| {
                    let indent = self.indent_config.sibling_indent_of(ast, lpar);
                    ast.upsert_whitespace_after_me(lpar, &indent);
                });
            }
            if let Some(rbrace) =
                ast.find_child_by_type(node, RPAR).filter(|&it| !ast.is_white_space_with_newline(ast.prev_sibling(it)))
            {
                emit(ast, ast.start_offset(rbrace), "Expected new line before ')'", true).if_autocorrect_allowed(|| {
                    let indent = self.indent_config.parent_indent_of(ast, rbrace);
                    ast.upsert_whitespace_before_me(rbrace, &indent);
                });
            }
        }
    }

    fn visit_lambda_argument(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        require(ast.element_type(node) == LAMBDA_ARGUMENT);
        if ast.text_contains(node, '\n') || self.exceeds_max_line_length(ast, node, ast.last_child_leaf_or_self(node)) {
            let function_literal =
                ast.find_child_by_type(node, LAMBDA_EXPRESSION).and_then(|it| ast.find_child_by_type(it, FUNCTION_LITERAL));
            let arrow = function_literal.and_then(|it| ast.find_child_by_type(it, ARROW));
            match arrow {
                Some(arrow) if !self.exceeds_max_line_length(ast, node, ast.last_child_leaf_or_self(arrow)) => {
                    if !ast.is_white_space_with_newline(ast.next_sibling(arrow)) {
                        emit(ast, ast.start_offset(arrow) + 1, "Expected new line after '->'", true).if_autocorrect_allowed(|| {
                            let indent = self.indent_config.sibling_indent_of(ast, arrow);
                            ast.upsert_whitespace_after_me(arrow, &indent);
                        });
                    }
                }
                _ => {
                    // Arrow not found, or does not fit on the line. Wrap after brace
                    if let Some(lbrace) = function_literal
                        .and_then(|it| ast.find_child_by_type(it, LBRACE))
                        .filter(|&it| !ast.is_white_space_with_newline(ast.next_sibling(it)))
                    {
                        emit(ast, ast.start_offset(lbrace), "Expected new line after '{'", true).if_autocorrect_allowed(|| {
                            let indent = self.indent_config.sibling_indent_of(ast, lbrace);
                            ast.upsert_whitespace_after_me(lbrace, &indent);
                        });
                    }
                }
            }
            if let Some(rbrace) = function_literal
                .and_then(|it| ast.find_child_by_type(it, RBRACE))
                .filter(|&it| !ast.is_white_space_with_newline(ast.prev_leaf(it)))
            {
                emit(ast, ast.start_offset(rbrace), "Expected new line before '}'", true).if_autocorrect_allowed(|| {
                    let indent = self.indent_config.parent_indent_of(ast, rbrace);
                    ast.upsert_whitespace_before_me(rbrace, &indent);
                });
            }
        }
    }

    fn exceeds_max_line_length(&self, ast: &Ast, node: NodeId, stop_at_leaf: NodeId) -> bool {
        ast.has_no_max_line_length_suppression(node)
            && (self.max_line_length as i64)
                < ast.line_length(
                    ast.drop_trailing_eol_comment(ast.leaves_on_line(node)).take_while(|&it| ast.prev_leaf(it) != Some(stop_at_leaf)),
                ) as i64
    }
}

fn require(value: bool) {
    assert!(value, "IllegalArgumentException: Failed requirement.");
}
