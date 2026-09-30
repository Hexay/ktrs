//! Port of ktlint-ruleset-standard `FunctionStartOfBodySpacingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{BLOCK, EQ, FUN, WHITE_SPACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

/// Lints and formats the spacing after the fun keyword
pub struct FunctionStartOfBodySpacingRule;

impl RuleV2 for FunctionStartOfBodySpacingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:function-start-of-body-spacing")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == FUN {
            if ast.find_child_by_type(node, EQ).is_some() {
                visit_function_followed_by_body_expression(ast, node, emit);
            }

            if ast.find_child_by_type(node, BLOCK).is_some() {
                visit_function_followed_by_body_block(ast, node, emit);
            }
        }
    }
}

fn visit_function_followed_by_body_expression(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    fix_white_space_before_assignment_of_body_expression(ast, node, emit);
    fix_white_space_between_assignment_and_body_expression(ast, node, emit);
}

fn fix_white_space_before_assignment_of_body_expression(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    let Some(assignment_expression) = ast.find_child_by_type(node, EQ) else { return };
    let white_space_before_assignment = ast.prev_leaf(assignment_expression).filter(|&it| ast.element_type(it) == WHITE_SPACE);
    match white_space_before_assignment {
        None => {
            emit(ast, ast.start_offset(assignment_expression), "Expected a single white space before assignment of expression body", true)
                .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(assignment_expression, " "));
        }
        Some(white_space_before_assignment) if !ast.text_matches(white_space_before_assignment, " ") => {
            emit(ast, ast.start_offset(white_space_before_assignment), "Unexpected whitespace", true)
                .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(assignment_expression, " "));
        }
        Some(_) => {}
    }
}

fn fix_white_space_between_assignment_and_body_expression(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    let Some(assignment_expression) = ast.find_child_by_type(node, EQ) else { return };
    let white_space_after_assignment = ast.next_leaf(assignment_expression).filter(|&it| ast.element_type(it) == WHITE_SPACE);
    if white_space_after_assignment.is_none_or(|it| !ast.text_matches(it, " "))
        && !ast.is_white_space_with_newline(white_space_after_assignment)
    {
        emit(ast, ast.start_offset(assignment_expression), "Expected a single white space between assignment and expression body on same line", true)
            .if_autocorrect_allowed(|| ast.upsert_whitespace_after_me(assignment_expression, " "));
    }
}

fn visit_function_followed_by_body_block(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    let Some(block) = ast.find_child_by_type(node, BLOCK) else { return };
    let white_space_before_expression_block = ast.prev_leaf(block).filter(|&it| ast.element_type(it) == WHITE_SPACE);
    if white_space_before_expression_block.is_none_or(|it| !ast.text_matches(it, " ")) {
        emit(ast, ast.start_offset(block), "Expected a single white space before start of function body", true).if_autocorrect_allowed(|| {
            if let Some(prev_leaf) = ast.first_child_node(block).and_then(|it| ast.prev_leaf(it)) {
                ast.upsert_whitespace_after_me(prev_leaf, " ");
            }
        });
    }
}
