//! Port of ktlint-ruleset-standard `MixedConditionOperatorsRule.kt`: a condition should not mix `&&` and `||` at the
//! same level; parentheses clarify it.

use ktrs_ast::{Ast, NodeId};
use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::{self, ANDAND, BINARY_EXPRESSION, OPERATION_REFERENCE, OROR};

use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[BINARY_EXPRESSION]);

pub struct MixedConditionOperatorsRule;

impl RuleV2 for MixedConditionOperatorsRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:mixed-condition-operators")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if is_logical_binary_expression(ast, node) && is_part_of_expression_using_different_logical_operators(ast, node) {
            visit_logical_expression(ast, node, emit);
        }
    }
}

fn is_logical_binary_expression(ast: &Ast, node: NodeId) -> bool {
    Some(node)
        .filter(|&it| ast.element_type(it) == BINARY_EXPRESSION)
        .and_then(|it| ast.find_child_by_type(it, OPERATION_REFERENCE))
        .is_some_and(|it| LOGICAL_OPERATORS.contains(ast.first_child_node(it).map(|c| ast.element_type(c))))
}

fn visit_logical_expression(ast: &Ast, node: NodeId, emit: &mut Emit<'_>) {
    let root_binary_expression = ast.parent_matching(node, |it| {
        ast.element_type(it) == BINARY_EXPRESSION && ast.parent(it).map(|p| ast.element_type(p)) != Some(BINARY_EXPRESSION)
    });
    if let Some(root_binary_expression) = root_binary_expression {
        emit(
            ast,
            ast.start_offset(root_binary_expression),
            "A condition with mixed usage of '&&' and '||' is hard to read. Use parenthesis to clarify the (sub)condition.",
            false,
        );
    }
}

fn any_parent_binary_expression(ast: &Ast, node: NodeId, predicate: impl Fn(NodeId) -> bool) -> bool {
    let mut current = Some(node);
    while let Some(c) = current.filter(|&c| ast.element_type(c) == BINARY_EXPRESSION) {
        if predicate(c) {
            return true;
        }
        current = ast.parent(c);
    }
    false
}

fn is_part_of_expression_using_different_logical_operators(ast: &Ast, node: NodeId) -> bool {
    let Some(first_logical_operator) = find_operation_element_type_or_null(ast, node) else { return false };
    any_parent_binary_expression(ast, node, |parent| {
        find_operation_element_type_or_null(ast, parent).is_some_and(|it| it != first_logical_operator)
    })
}

fn find_operation_element_type_or_null(ast: &Ast, node: NodeId) -> Option<SyntaxKind> {
    ast.find_child_by_type(node, OPERATION_REFERENCE)
        .and_then(|it| ast.first_child_node(it))
        .map(|it| ast.element_type(it))
        .filter(|&it| LOGICAL_OPERATORS.contains(it))
}

const LOGICAL_OPERATORS: TokenSet = TokenSet::create(&[OROR, ANDAND]);
