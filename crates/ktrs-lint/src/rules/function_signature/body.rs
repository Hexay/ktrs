//! `FunctionSignatureRule.kt` from `fixFunctionBodyExpression` to `countParameters`.

use ktrs_ast::{Ast, NodeId};

use super::{FunctionBodyExpressionWrapping, FunctionSignatureRule, function_signature_nodes, utf16_len};
use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeQueries};
use crate::editorconfig::MAX_LINE_LENGTH_PROPERTY_OFF;
use crate::element_type::{ANNOTATED_EXPRESSION, BLOCK, EQ, LBRACE, OPEN_QUOTE, VALUE_PARAMETER, VALUE_PARAMETER_LIST};
use crate::rule::Emit;

impl FunctionSignatureRule {
    pub(super) fn fix_function_body_expression(
        &self,
        ast: &mut Ast,
        node: NodeId,
        emit: &mut Emit<'_>,
        max_length_remaining_for_first_line_of_body_expression: i32,
    ) {
        let Some(last_node_of_function_signature_with_body_expression) = ast.find_child_by_type(node, EQ).and_then(|it| ast.next_leaf(it))
        else {
            return;
        };
        let body_nodes = get_function_body(ast, node, Some(last_node_of_function_signature_with_body_expression));
        let white_space_before_function_body_expression = get_starting_whitespace_or_null(ast, &body_nodes);
        let function_body_expression_nodes: Vec<NodeId> = body_nodes.iter().copied().skip_while(|&it| ast.is_white_space(it)).collect();

        let function_body_expression_text = join_text_to_string(ast, &function_body_expression_nodes);
        let function_body_expression_lines: Vec<&str> = function_body_expression_text.split('\n').collect();
        let first_line_of_body_expression = utf16_len(function_body_expression_lines[0]);
        let wrapping = self.function_body_expression_wrapping;
        if ast.is_white_space_with_newline(white_space_before_function_body_expression) {
            if ast
                .next_code_sibling(last_node_of_function_signature_with_body_expression)
                .is_some_and(|it| ast.element_type(it) == ANNOTATED_EXPRESSION)
            {
                // Never merge an annotated expression body with function signature as this conflicts with the Annotation rule
                return;
            }
            let merge_with_function_signature = if wrapping == FunctionBodyExpressionWrapping::Always {
                false
            } else if first_line_of_body_expression < max_length_remaining_for_first_line_of_body_expression {
                (wrapping == FunctionBodyExpressionWrapping::Default && !is_multiline_string_template(ast, &function_body_expression_nodes))
                    || (wrapping == FunctionBodyExpressionWrapping::Multiline && function_body_expression_lines.len() == 1)
                    || is_multiline_function_signature_without_explicit_return_type(
                        ast,
                        node,
                        Some(last_node_of_function_signature_with_body_expression),
                    )
            } else {
                false
            };
            if merge_with_function_signature {
                let white_space = white_space_before_function_body_expression.expect("NullPointerException");
                emit(ast, ast.start_offset(white_space), "First line of body expression fits on same line as function signature", true)
                    .if_autocorrect_allowed(|| ast.replace_text_with(white_space, " "));
            }
        } else if ast.is_white_space_without_newline_or_null(white_space_before_function_body_expression) {
            let first_expression_node =
                || *function_body_expression_nodes.first().expect("NoSuchElementException: List is empty.");
            if is_multiline_function_signature_without_explicit_return_type(ast, node, Some(last_node_of_function_signature_with_body_expression))
                && first_line_of_body_expression + 1 <= max_length_remaining_for_first_line_of_body_expression
                && wrapping != FunctionBodyExpressionWrapping::Always
            {
                if white_space_before_function_body_expression.is_none_or(|it| !ast.text_matches(it, " ")) {
                    let first = first_expression_node();
                    emit(ast, ast.start_offset(first), "Single whitespace expected before expression body", true)
                        .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(first, " "));
                }
            } else if (ast.has_no_max_line_length_suppression_in(node, self.ktlint_version)
                && first_line_of_body_expression + 1 > max_length_remaining_for_first_line_of_body_expression)
                || (wrapping == FunctionBodyExpressionWrapping::Multiline && function_body_expression_lines.len() > 1)
                || wrapping == FunctionBodyExpressionWrapping::Always
            {
                let first = first_expression_node();
                emit(ast, ast.start_offset(first), "Newline expected before expression body", true).if_autocorrect_allowed(|| {
                    let indent = self.indent_config.child_indent_of(ast, node);
                    ast.upsert_whitespace_before_me(first, &indent)
                });
            }
        }
    }
}

fn is_multiline_string_template(ast: &Ast, nodes: &[NodeId]) -> bool {
    let it = *nodes.iter().find(|&&it| ast.is_code(it)).expect("NoSuchElementException: Collection contains no element matching the predicate.");
    ast.element_type(it) == OPEN_QUOTE && ast.next_leaf(it).is_some_and(|next| ast.text(next).starts_with('\n'))
}

fn is_multiline_function_signature_without_explicit_return_type(
    ast: &Ast,
    node: NodeId,
    last_node_of_function_signature_with_body_expression: Option<NodeId>,
) -> bool {
    let nodes = children_between(function_signature_nodes(ast, node), |_| true, |it| Some(it) == last_node_of_function_signature_with_body_expression);
    let text: String = nodes.iter().map(|&it| ast.text(it)).collect();
    text.rsplit('\n').next().is_some_and(matches_indent_with_closing_parenthesis)
}

/// `INDENT_WITH_CLOSING_PARENTHESIS` = `\s*\) =` (whole text, Java `\s`).
fn matches_indent_with_closing_parenthesis(line: &str) -> bool {
    line.trim_start_matches([' ', '\t', '\n', '\u{b}', '\u{c}', '\r']) == ") ="
}

impl FunctionSignatureRule {
    pub(super) fn fix_whitespace_before_function_body_block(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>, dry_run: bool) -> i32 {
        let mut white_space_correction = 0;

        if let Some(block) = ast.find_child_by_type(node, BLOCK).filter(|&it| ast.find_child_by_type(it, LBRACE).is_some()) {
            let white_space_before_block = ast.prev_leaf(block).filter(|&it| ast.is_white_space(it));
            if white_space_before_block.is_none_or(|it| !ast.text_matches(it, " ")) {
                if !dry_run {
                    emit(ast, ast.start_offset(block), "Expected a single space before body block", true)
                        .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(block, " "));
                } else {
                    white_space_correction += 1 - white_space_before_block.map_or(0, |it| ast.text_length_utf16(it) as i32);
                }
            }
        }

        white_space_correction
    }
}

fn get_function_body(ast: &Ast, node: NodeId, split_node: Option<NodeId>) -> Vec<NodeId> {
    children_between(
        collect_leaves_recursively(ast, node),
        |it| Some(it) == split_node,
        // collect all remaining nodes
        |_| false,
    )
}

fn get_starting_whitespace_or_null(ast: &Ast, nodes: &[NodeId]) -> Option<NodeId> {
    nodes.first().copied().filter(|&first| ast.is_white_space(first))
}

impl FunctionSignatureRule {
    pub(super) fn is_max_line_length_set(&self) -> bool {
        self.max_line_length != MAX_LINE_LENGTH_PROPERTY_OFF
    }
}

/// Both `collectLeavesRecursively` overloads: the leaves (nodes without children) of `node`, in order.
pub(crate) fn collect_leaves_recursively(ast: &Ast, node: NodeId) -> Vec<NodeId> {
    ast.preorder(node).filter(|&it| ast.is_leaf(it)).collect()
}

pub(crate) fn children_between(
    nodes: Vec<NodeId>,
    start_ast_node_predicate: impl Fn(NodeId) -> bool,
    end_ast_node_predicate: impl Fn(NodeId) -> bool,
) -> Vec<NodeId> {
    let mut iterator = nodes.into_iter();
    let mut children_between = Vec::new();

    for current_node in iterator.by_ref() {
        if start_ast_node_predicate(current_node) {
            children_between.push(current_node);
            break;
        }
    }

    for current_node in iterator {
        children_between.push(current_node);
        if end_ast_node_predicate(current_node) {
            break;
        }
    }

    children_between
}

fn join_text_to_string(ast: &Ast, nodes: &[NodeId]) -> String {
    nodes.iter().flat_map(|&it| collect_leaves_recursively(ast, it)).map(|it| ast.text(it)).collect()
}

/// `joinTextToString().length`.
pub(crate) fn join_text_length(ast: &Ast, nodes: &[NodeId]) -> i32 {
    nodes.iter().flat_map(|&it| collect_leaves_recursively(ast, it)).map(|it| ast.text_length_utf16(it) as i32).sum()
}

impl FunctionSignatureRule {
    pub(super) fn has_minimum_number_of_parameters(&self, ast: &Ast, node: NodeId) -> bool {
        count_parameters(ast, node) >= self.function_signature_wrapping_minimum_parameters
    }
}

pub(super) fn count_parameters(ast: &Ast, node: NodeId) -> i32 {
    let value_parameter_list =
        ast.find_child_by_type(node, VALUE_PARAMETER_LIST).expect("IllegalArgumentException: Required value was null.");
    ast.children(value_parameter_list).filter(|&it| ast.element_type(it) == VALUE_PARAMETER).count() as i32
}
