//! `ChainMethodContinuationRule.insertWhiteSpaceBeforeChainOperator` .. `fixWhiteSpaceAfterChainOperators`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{CLOSING_QUOTE, STRING_TEMPLATE};

use super::chained_expression::ChainedExpression;
use super::{ChainMethodContinuationRule, GROUP_CLOSING_ELEMENT_TYPE};
use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::Emit;

impl ChainMethodContinuationRule {
    /// Disallows `fooBar\n .bar { ... }.foo()`, also with comments before each chained method.
    pub(super) fn insert_white_space_before_chain_operator(&self, ast: &mut Ast, chain_operator: NodeId, emit: &mut Emit<'_>) {
        let white_space_or_comment = ast.prev_leaf(chain_operator).filter(|&it| !ast.is_code(it));
        let is_comment = white_space_or_comment.is_some_and(|it| ast.is_part_of_comment(it));
        if is_comment || white_space_or_comment.is_none() || ast.is_white_space_without_newline(white_space_or_comment) {
            let message = format!("Expected newline before '{}'", ast.text(chain_operator));
            let indent_config = &self.indent_config;
            emit(ast, ast.start_offset(chain_operator), &message, true).if_autocorrect_allowed(|| {
                let indent = indent_config.child_indent_of(ast, ast.parent(chain_operator).expect("NullPointerException: parent!!"));
                ast.upsert_whitespace_before_me(chain_operator, &indent);
            });
        }
    }
}

pub(super) fn should_be_on_same_line_as_closing_element_of_previous_expression_in_method_chain(ast: &Ast, node: NodeId) -> bool {
    ast.prev_leaf_matching(node, |it| !ast.is_white_space(it))
        .filter(|&it| GROUP_CLOSING_ELEMENT_TYPE.contains(ast.element_type(it)))
        .is_some_and(|closing_element| {
            is_preceded_by_newline(ast, closing_element)
                || (ast.element_type(closing_element) == CLOSING_QUOTE && is_part_of_multiline_string_template(ast, closing_element))
        })
}

fn is_preceded_by_newline(ast: &Ast, node: NodeId) -> bool {
    ast.is_white_space_with_newline(ast.prev_leaf(node))
}

fn is_part_of_multiline_string_template(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .filter(|&it| ast.element_type(it) == STRING_TEMPLATE)
        .is_some_and(|it| ast.children(it).any(|child| ast.text_matches(child, "\n")))
}

/// Disallows `bar {\n ...\n }.\nfoo()` and `"""\nsome text\n"""\n .trimIndent()`.
pub(super) fn remove_white_space_before_chain_operator(ast: &mut Ast, chain_operator: NodeId, emit: &mut Emit<'_>) {
    let white_space = ast.prev_leaf(chain_operator).filter(|&it| ast.is_white_space(it));
    if let Some(white_space) = white_space.filter(|&it| ast.is_white_space_with_newline(it)) {
        let message = format!("Unexpected newline before '{}'", ast.text(chain_operator));
        emit(ast, ast.start_offset(chain_operator), &message, true).if_autocorrect_allowed(|| ast.remove(white_space));
    }
}

pub(super) fn disallow_comment_between_dot_and_call_expression(ast: &mut Ast, chained_expression: &ChainedExpression, emit: &mut Emit<'_>) {
    for &chain_operator in &chained_expression.chain_operators {
        if let Some(comment) = ast
            .next_sibling_matching(chain_operator, |it| !ast.is_white_space(it))
            .filter(|&it| ast.is_part_of_comment(it))
        {
            emit(ast, ast.start_offset(comment), "No comment expected at this location in method chain", false);
        }
    }
}

pub(super) fn fix_white_space_after_chain_operators(ast: &mut Ast, chained_expression: &ChainedExpression, emit: &mut Emit<'_>) {
    for &chain_operator in &chained_expression.chain_operators {
        if let Some(white_space) = ast.next_leaf(chain_operator).filter(|&it| ast.is_white_space_with_newline(it)) {
            let message = format!("Unexpected newline after '{}'", ast.text(chain_operator));
            emit(ast, ast.start_offset(white_space) - 1, &message, true).if_autocorrect_allowed(|| ast.remove(white_space));
        }
    }
}
