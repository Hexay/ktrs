//! Port of ktlint-ruleset-standard `UnnecessaryParenthesesBeforeTrailingLambdaRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{CALL_EXPRESSION, FUNCTION_LITERAL, LAMBDA_ARGUMENT, LAMBDA_EXPRESSION, LPAR, RPAR, VALUE_ARGUMENT_LIST};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

/// Ensures there are no unnecessary parentheses before a trailing lambda.
pub struct UnnecessaryParenthesesBeforeTrailingLambdaRule;

impl RuleV2 for UnnecessaryParenthesesBeforeTrailingLambdaRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:unnecessary-parentheses-before-trailing-lambda")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let type_of = |n: Option<NodeId>| n.map(|n| ast.element_type(n));
        if is_empty_argument_list(ast, node)
            && ast.is_part_of(node, CALL_EXPRESSION)
            && is_not_preceded_by_call_expression_ending_with_lambda_argument(ast, node)
            && type_of(ast.next_code_sibling(node)) == Some(LAMBDA_ARGUMENT)
            && type_of(ast.prev_code_sibling(node)) != Some(CALL_EXPRESSION)
        {
            emit(ast, ast.start_offset(node), "Empty parentheses in function call followed by lambda are unnecessary", true)
                .if_autocorrect_allowed(|| ast.remove(node));
        }
    }
}

fn is_empty_argument_list(ast: &Ast, node: NodeId) -> bool {
    ast.element_type(node) == VALUE_ARGUMENT_LIST && ast.children(node).all(|it| matches!(ast.element_type(it), LPAR | RPAR))
}

fn is_not_preceded_by_call_expression_ending_with_lambda_argument(ast: &Ast, node: NodeId) -> bool {
    ast.prev_code_sibling(node)
        .filter(|&it| ast.element_type(it) == CALL_EXPRESSION)
        .and_then(|it| ast.last_child_node(it))
        .filter(|&it| ast.element_type(it) == LAMBDA_ARGUMENT)
        .and_then(|it| ast.last_child_node(it))
        .filter(|&it| ast.element_type(it) == LAMBDA_EXPRESSION)
        .and_then(|it| ast.last_child_node(it))
        .map(|it| ast.element_type(it) != FUNCTION_LITERAL)
        .unwrap_or(true)
}
