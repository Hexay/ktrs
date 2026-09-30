//! `WrappingRule.kt` from `rearrangeSuperTypeList` to `rearrangeTypeArgumentList`.

use ktrs_ast::psi::KtSuperTypeList;
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ANNOTATION, COMMA, DESTRUCTURING_DECLARATION, FUN, GT, LAMBDA_EXPRESSION, LT, RPAR, SUPER_TYPE_CALL_ENTRY, SUPER_TYPE_ENTRY,
    TYPE_PARAMETER, TYPE_PROJECTION, VALUE_ARGUMENT, VALUE_PARAMETER,
};

use super::WrappingRule;
use super::helpers::has_line_break;
use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::Emit;
use crate::token_sets::COMMENTS;

impl WrappingRule {
    pub(super) fn rearrange_super_type_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let entries: Vec<NodeId> = KtSuperTypeList::of(ast, node).entries(ast).into_iter().map(|it| it.node()).collect();
        if ast.text_contains(node, '\n')
            && entries.len() > 1
            // e.g. `class A : B, C,\n D` or `class A : B, C({\n}), D`, but not `class A : B, C, D({\n})`
            && !(entries[..entries.len() - 1].iter().all(|&it| ast.element_type(it) == SUPER_TYPE_ENTRY)
                && ast.element_type(entries[entries.len() - 1]) == SUPER_TYPE_CALL_ENTRY)
        {
            // put space after :
            if !ast.is_white_space_with_newline(ast.prev_leaf(node)) {
                let colon = ast.prev_code_leaf(node).expect("NullPointerException: prevCodeLeaf!!");
                if !ast.is_white_space_with_newline(ast.prev_leaf(colon))
                    && ast
                        .prev_code_leaf(colon)
                        .is_none_or(|it| ast.element_type(it) != RPAR || !ast.is_white_space_with_newline(ast.prev_leaf(it)))
                {
                    self.require_newline_after_leaf(ast, colon, emit, None);
                }
            }
            // put entries on separate lines
            let mut child = ast.first_child_node(node);
            while let Some(c) = child {
                if ast.element_type(c) == COMMA
                    && !ast.is_white_space_with_newline(ast.next_sibling(c))
                    && !is_followed_by_comment_on_same_line(ast, c)
                {
                    let indent = ast.indent(node);
                    self.require_newline_after_leaf(ast, c, emit, Some(indent));
                }
                child = ast.next_sibling(c);
            }
        }
    }

    pub(super) fn rearrange_value_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let mut child = ast.first_child_node(node);
        while let Some(c) = child {
            let has_line_break = match ast.element_type(c) {
                VALUE_ARGUMENT => has_line_break(ast, c, &[LAMBDA_EXPRESSION, FUN]),
                VALUE_PARAMETER | ANNOTATION => has_line_break(ast, c, &[]),
                _ => false,
            };
            if has_line_break {
                // rearrange `a, b, value(\n), c, d` to `a, b,\nvalue(\n),\nc, d`

                // insert \n in front of multi-line value
                if let Some(prev_sibling) = ast.prev_sibling_matching(c, |it| !ast.is_white_space(it))
                    && ast.element_type(prev_sibling) == COMMA
                    && !ast.is_white_space_with_newline(ast.next_sibling(prev_sibling))
                {
                    self.require_newline_after_leaf(ast, prev_sibling, emit, None);
                }
                // insert \n after multi-line value
                let next_sibling = ast.next_sibling_matching(c, |it| !ast.is_white_space(it));
                let has_destructuring_declaration_as_last_value_parameter = is_last_value_parameter(ast, c)
                    && ast.element_type(ast.first_child_node(c).expect("NullPointerException: firstChildNode")) == DESTRUCTURING_DECLARATION;
                if let Some(next_sibling) = next_sibling
                    && ast.element_type(next_sibling) == COMMA
                    && !has_destructuring_declaration_as_last_value_parameter
                    && !ast.is_white_space_with_newline(ast.next_sibling(next_sibling))
                    // value(
                    // ), // a comment
                    // c, d
                    && ast
                        .next_sibling(next_sibling)
                        .and_then(|it| ast.next_sibling(it))
                        .is_none_or(|it| !COMMENTS.contains(ast.element_type(it)))
                {
                    self.require_newline_after_leaf(ast, next_sibling, emit, None);
                }
            }
            child = ast.next_sibling(c);
        }
    }

    pub(super) fn rearrange_type_argument_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if !ast.text_contains(node, '\n') {
            return;
        }
        // Each type projection must be preceded with a whitespace containing a newline
        let mut child = ast.first_child_node(node);
        while let Some(type_projection) = child {
            if matches!(ast.element_type(type_projection), TYPE_PROJECTION | TYPE_PARAMETER) {
                let prev_sibling = ast.prev_sibling_matching(type_projection, |it| !ast.is_part_of_comment(it));
                if prev_sibling.map(|it| ast.element_type(it)) == Some(LT) || ast.is_white_space_without_newline(prev_sibling) {
                    let message = format!("A newline was expected before '{}'", ast.text(type_projection));
                    emit(ast, ast.start_offset(type_projection), &message, true).if_autocorrect_allowed(|| {
                        let indent = self.indent_config.sibling_indent_of(ast, node);
                        ast.upsert_whitespace_before_me(type_projection, &indent);
                    });
                }
            }
            child = ast.next_sibling(type_projection);
        }

        // After the last type projection a whitespace containing a newline must exist
        if let Some(closing_angle) = ast.find_child_by_type(node, GT) {
            let prev_sibling = ast.prev_sibling_matching(closing_angle, |it| !ast.is_part_of_comment(it));
            if !ast.is_white_space_with_newline(prev_sibling) {
                let message = format!("A newline was expected before '{}'", ast.text(closing_angle));
                emit(ast, ast.start_offset(closing_angle), &message, true).if_autocorrect_allowed(|| {
                    let indent = self.indent_config.sibling_indent_of(ast, node);
                    ast.upsert_whitespace_before_me(closing_angle, &indent);
                });
            }
        }
    }
}

fn is_followed_by_comment_on_same_line(ast: &Ast, node: NodeId) -> bool {
    ast.next_leaf_matching(node, |it| !ast.is_white_space_without_newline(it)).is_some_and(|it| ast.is_part_of_comment(it))
}

fn is_last_value_parameter(ast: &Ast, node: NodeId) -> bool {
    ast.element_type(node) == VALUE_PARAMETER && !ast.siblings(node, true).any(|it| ast.element_type(it) == VALUE_PARAMETER)
}
