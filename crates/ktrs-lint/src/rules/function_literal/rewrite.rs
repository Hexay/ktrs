//! `FunctionLiteralRule.rewriteToMultilineParameterList` .. `wrapAfterLbrace`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ARROW, BLOCK, ELSE, FUNCTION_LITERAL, LAMBDA_EXPRESSION, LBRACE, RBRACE, THEN, VALUE_PARAMETER, VALUE_PARAMETER_LIST, WHEN_ENTRY,
};

use super::FunctionLiteralRule;
use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::Emit;

impl FunctionLiteralRule {
    pub(super) fn rewrite_to_multiline_parameter_list(&self, ast: &mut Ast, parameter_list: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(parameter_list) == VALUE_PARAMETER_LIST, "IllegalArgumentException: Failed requirement.");
        let value_parameters: Vec<NodeId> =
            ast.children(parameter_list).filter(|&it| ast.element_type(it) == VALUE_PARAMETER).collect();
        for value_parameter in value_parameters {
            self.wrap_value_parameter(ast, value_parameter, emit);
        }
        if let Some(arrow) = ast.parent(parameter_list).and_then(|p| ast.find_child_by_type(p, ARROW)) {
            self.wrap_arrow(ast, arrow, emit);
        }
        if let Some(rbrace) = ast.parent(parameter_list).and_then(|p| ast.find_child_by_type(p, RBRACE)) {
            self.wrap_before_rbrace(ast, rbrace, emit);
        }
    }

    fn wrap_value_parameter(&self, ast: &mut Ast, value_parameter: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(value_parameter) == VALUE_PARAMETER, "IllegalArgumentException: Failed requirement.");
        if ast.is_white_space_without_newline(ast.prev_leaf(value_parameter)) {
            let indent_config = &self.indent_config;
            emit(ast, ast.start_offset(value_parameter), "Newline expected before parameter", true).if_autocorrect_allowed(|| {
                let function_literal =
                    ast.find_parent_by_type(value_parameter, FUNCTION_LITERAL).expect("NullPointerException: findParentByType!!");
                let indent = indent_config.child_indent_of(ast, function_literal);
                ast.upsert_whitespace_before_me(value_parameter, &indent);
            });
        }
    }

    fn wrap_arrow(&self, ast: &mut Ast, arrow: NodeId, emit: &mut Emit<'_>) {
        self.wrap_before_arrow(ast, arrow, emit);
        self.wrap_after_arrow(ast, arrow, emit);
    }

    fn wrap_before_arrow(&self, ast: &mut Ast, arrow: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(arrow) == ARROW, "IllegalArgumentException: Failed requirement.");
        if ast.is_white_space_without_newline(ast.prev_leaf(arrow)) {
            let indent_config = &self.indent_config;
            emit(ast, ast.start_offset(arrow), "Newline expected before arrow", true).if_autocorrect_allowed(|| {
                let indent = indent_config.child_indent_of(ast, ast.parent(arrow).expect("NullPointerException: parent!!"));
                ast.upsert_whitespace_before_me(arrow, &indent);
            });
        }
    }

    fn wrap_after_arrow(&self, ast: &mut Ast, arrow: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(arrow) == ARROW, "IllegalArgumentException: Failed requirement.");
        // `takeIf { isWhiteSpaceWithoutNewlineOrNull }?.let`: a missing leaf is filtered out by the `?.`
        if ast.next_leaf(arrow).is_some_and(|it| ast.is_white_space_without_newline_or_null(it)) {
            let indent_config = &self.indent_config;
            let offset = ast.start_offset(arrow) + ast.text_length(arrow) - 1;
            emit(ast, offset, "Newline expected after arrow", true).if_autocorrect_allowed(|| {
                let indent = indent_config.sibling_indent_of(ast, arrow);
                ast.upsert_whitespace_after_me(arrow, &indent);
            });
        }
    }

    fn wrap_before_rbrace(&self, ast: &mut Ast, rbrace: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(rbrace) == RBRACE, "IllegalArgumentException: Failed requirement.");
        if ast.prev_leaf(rbrace).is_some_and(|it| ast.is_white_space_without_newline_or_null(it)) {
            let indent_config = &self.indent_config;
            emit(ast, ast.start_offset(rbrace), "Newline expected before closing brace", true).if_autocorrect_allowed(|| {
                let indent = indent_config.parent_indent_of(ast, rbrace);
                ast.upsert_whitespace_before_me(rbrace, &indent);
            });
        }
    }
}

pub(super) fn rewrite_to_single_line_function_literal(ast: &mut Ast, parameter_list: NodeId, emit: &mut Emit<'_>) {
    assert!(ast.element_type(parameter_list) == VALUE_PARAMETER_LIST, "IllegalArgumentException: Failed requirement.");
    if !is_preceded_by_comment(ast, parameter_list)
        && let Some(whitespace_before_parameter_list) =
            ast.prev_sibling_matching(parameter_list, |it| ast.is_white_space_with_newline(it))
    {
        emit(ast, ast.start_offset(parameter_list), "No newline expected before parameter", true)
            .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(whitespace_before_parameter_list, " "));
    }
    if let Some(whitespace_after_parameter_list) = ast
        .next_sibling_matching(parameter_list, |it| ast.is_white_space(it))
        .filter(|&it| ast.is_white_space_with_newline(it))
    {
        let offset = ast.start_offset(parameter_list) + ast.text_length(parameter_list);
        emit(ast, offset, "No newline expected after parameter", true)
            .if_autocorrect_allowed(|| ast.upsert_whitespace_after_me(whitespace_after_parameter_list, " "));
    }
}

fn is_preceded_by_comment(ast: &Ast, node: NodeId) -> bool {
    ast.siblings(node, false).any(|it| ast.is_part_of_comment(it))
}

pub(super) fn visit_arrow(ast: &mut Ast, arrow: NodeId, emit: &mut Emit<'_>) {
    assert!(ast.element_type(arrow) == ARROW, "IllegalArgumentException: Failed requirement.");
    let redundant = ast
        .prev_sibling_matching(arrow, |it| ast.element_type(it) == VALUE_PARAMETER_LIST)
        .is_some_and(|it| has_empty_parameter_list(ast, it))
        && !is_lambda_expression_not_wrapped_in_block(ast, arrow)
        && is_followed_by_non_empty_block(ast, arrow);
    if redundant {
        emit(ast, ast.start_offset(arrow), "Arrow is redundant when parameter list is empty", true).if_autocorrect_allowed(|| {
            if let Some(white_space) = ast.next_sibling(arrow).filter(|&it| ast.is_white_space(it)) {
                ast.remove(white_space);
            }
            ast.remove(arrow);
        });
    }
}

fn has_empty_parameter_list(ast: &Ast, node: NodeId) -> bool {
    assert!(ast.element_type(node) == VALUE_PARAMETER_LIST, "IllegalArgumentException: Failed requirement.");
    ast.find_child_by_type(node, VALUE_PARAMETER).is_none()
}

/// Allows `1 == 2 -> { -> "hi" }` in a when and `if (cond) { -> "hi" } else { -> "ho" }`.
fn is_lambda_expression_not_wrapped_in_block(ast: &Ast, node: NodeId) -> bool {
    assert!(ast.element_type(node) == ARROW, "IllegalArgumentException: Failed requirement.");
    ast.find_parent_by_type(node, LAMBDA_EXPRESSION)
        .and_then(|it| ast.parent(it))
        .is_some_and(|it| matches!(ast.element_type(it), WHEN_ENTRY | THEN | ELSE))
}

fn is_followed_by_non_empty_block(ast: &Ast, node: NodeId) -> bool {
    assert!(ast.element_type(node) == ARROW, "IllegalArgumentException: Failed requirement.");
    ast.next_sibling_matching(node, |it| ast.element_type(it) == BLOCK).and_then(|it| ast.first_child_node(it)).is_some()
}

impl FunctionLiteralRule {
    pub(super) fn visit_block(&self, ast: &mut Ast, block: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(block) == BLOCK, "IllegalArgumentException: Failed requirement.");
        if ast.text_contains(block, '\n') || self.exceeds_max_line_length(ast, block) {
            if let Some(prev_code_sibling) = ast.prev_code_sibling(block) {
                match ast.element_type(prev_code_sibling) {
                    ARROW => self.wrap_after_arrow(ast, prev_code_sibling, emit),
                    LBRACE => self.wrap_after_lbrace(ast, prev_code_sibling, emit),
                    _ => {}
                }
            }

            if let Some(rbrace) = ast.next_code_sibling(block).filter(|&it| ast.element_type(it) == RBRACE) {
                self.wrap_before_rbrace(ast, rbrace, emit);
            }
        }
    }

    fn wrap_after_lbrace(&self, ast: &mut Ast, lbrace: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(lbrace) == LBRACE, "IllegalArgumentException: Failed requirement.");
        let whitespace_after_lbrace = ast.next_leaf(lbrace).filter(|&it| ast.is_white_space(it));
        if ast.is_white_space_without_newline_or_null(whitespace_after_lbrace) {
            let indent_config = &self.indent_config;
            emit(ast, ast.start_offset(lbrace), "Newline expected after opening brace", true).if_autocorrect_allowed(|| {
                let indent = indent_config.child_indent_of(ast, lbrace);
                ast.upsert_whitespace_after_me(lbrace, &indent);
            });
        }
    }
}
