//! `WrappingRule.kt` from `beforeVisitBlock` to `rearrangeBlock`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    BLOCK, CONDITION, EOL_COMMENT, FUN, FUNCTION_LITERAL, LAMBDA_EXPRESSION, LBRACE, LONG_STRING_TEMPLATE_ENTRY, OBJECT_LITERAL, RBRACE,
    VALUE_ARGUMENT, VALUE_PARAMETER_LIST,
};

use super::helpers::{
    followed_by_newline, get_end_of_block, get_start_of_block, has_line_break, is_part_of_for_loop_condition_with_multiline_expression,
};
use super::{WrappingRule, matching_rtoken};
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines, AstNodeQueries};
use crate::editorconfig::MAX_LINE_LENGTH_PROPERTY_OFF;
use crate::rule::Emit;

impl WrappingRule {
    pub(super) fn before_visit_block(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(node) == BLOCK, "IllegalArgumentException: Failed requirement.");

        let Some(lbrace) = get_start_of_block(ast, node).filter(|&it| ast.element_type(it) == LBRACE) else { return };
        if followed_by_newline(ast, lbrace)
            || followed_by_eol_comment(ast, lbrace)
            || followed_by_function_literal_parameter_list(ast, lbrace)
            || ast.is_part_of(lbrace, LONG_STRING_TEMPLATE_ENTRY)
        {
            // String template inside raw string literal may exceed the maximum line length
            return;
        }

        if let Some(rbrace) = Some(node)
            .filter(|&it| ast.element_type(ast.first_child_leaf_or_self(it)) != EOL_COMMENT)
            .and_then(|it| get_end_of_block(ast, it))
            .filter(|&it| ast.element_type(it) == RBRACE)
            && ast.has_new_line_in_closed_range(lbrace, rbrace)
        {
            self.require_newline_after_leaf(ast, lbrace, emit, None);
        }

        if self.max_line_length != MAX_LINE_LENGTH_PROPERTY_OFF && ast.has_no_max_line_length_suppression_in(node, self.ktlint_version) {
            let length_until_begin_of_line: usize = ast
                .leaves(node, false)
                .take_while(|&it| !ast.is_white_space_with_newline(it))
                .map(|it| ast.text_length_utf16(it))
                .sum();
            let length_until_end_of_line: usize = ast
                .leaves_forwards_including_self(ast.first_child_leaf_or_self(node))
                .take_while(|&it| !ast.is_white_space_with_newline(it))
                .map(|it| ast.text_length_utf16(it))
                .sum();
            if (length_until_begin_of_line + length_until_end_of_line) as i64 > self.max_line_length as i64 {
                self.require_newline_after_leaf(ast, lbrace, emit, None);
            }
        }
    }

    pub(super) fn rearrange_block(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let closing_element_type = matching_rtoken(ast.element_type(node));
        let mut newline_in_between = false;
        let mut parameter_list_in_between = false;
        let mut number_of_args = 0;
        let mut first_arg: Option<NodeId> = None;
        // matching ), ] or }; `nextSibling { }` with a predicate that tallies what it passes
        let mut sibling = ast.next_sibling(node);
        let closing_element = loop {
            let it = sibling.expect("NullPointerException: nextSibling { }!!");
            let is_value_argument = ast.element_type(it) == VALUE_ARGUMENT;
            let has_line_break =
                if is_value_argument { has_line_break(ast, it, &[LAMBDA_EXPRESSION, FUN]) } else { has_line_break(ast, it, &[]) };
            newline_in_between = newline_in_between || has_line_break;
            parameter_list_in_between = parameter_list_in_between || ast.element_type(it) == VALUE_PARAMETER_LIST;
            if is_value_argument {
                number_of_args += 1;
                first_arg = Some(it);
            }
            if Some(ast.element_type(it)) == closing_element_type {
                break it;
            }
            sibling = ast.next_sibling(it);
        };
        if !newline_in_between
            // keep { p ->
            // }
            || (ast.element_type(node) == LBRACE && parameter_list_in_between)
            // keep ({
            // }) and (object : C {
            // })
            || (number_of_args == 1
                && first_arg
                    .and_then(|it| ast.first_child_node(it))
                    .is_some_and(|it| matches!(ast.element_type(it), OBJECT_LITERAL | LAMBDA_EXPRESSION)))
        {
            return;
        }
        if is_part_of_for_loop_condition_with_multiline_expression(ast, node) {
            // keep `for (foo in listOf(\n "foo-1"\n)) { ... }`, reject `for (\n foo in listOf(...)\n) { ... }`
            return;
        }
        if !ast.is_white_space_with_newline(ast.next_code_leaf(node).and_then(|it| {
            ast.prev_leaf_matching(it, |it| {
                // Skip comments, whitespace, and empty nodes
                !ast.is_part_of_comment(it) && !ast.is_white_space_without_newline(it) && ast.text_length(it) > 0
            })
        }))
            // IDEA quirk: `if (true &&\n true\n) {` is kept, instead of `if (\n true &&\n true\n) {`
            && ast.next_sibling(node).map(|it| ast.element_type(it)) != Some(CONDITION)
        {
            self.require_newline_after_leaf(ast, node, emit, None);
        }
        if !ast.is_white_space_with_newline(ast.prev_leaf(closing_element)) {
            let indent = self.indent_config.parent_indent_of(ast, node);
            self.require_newline_before_leaf(ast, closing_element, emit, &indent);
        }
    }
}

fn followed_by_eol_comment(ast: &Ast, node: NodeId) -> bool {
    ast.leaves(node, true)
        .take_while(|&it| ast.is_white_space_without_newline(it) || ast.element_type(it) == EOL_COMMENT)
        .any(|it| ast.element_type(it) == EOL_COMMENT)
}

fn followed_by_function_literal_parameter_list(ast: &Ast, node: NodeId) -> bool {
    Some(node)
        .filter(|&it| ast.parent(it).map(|p| ast.element_type(p)) == Some(FUNCTION_LITERAL))
        .and_then(|it| ast.next_code_sibling(it))
        .is_some_and(|it| ast.element_type(it) == VALUE_PARAMETER_LIST)
}
