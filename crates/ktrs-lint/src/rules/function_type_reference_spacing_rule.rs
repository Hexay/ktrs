//! Port of ktlint-ruleset-standard `FunctionTypeReferenceSpacingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{FUN, NULLABLE_TYPE, TYPE_REFERENCE, VALUE_PARAMETER_LIST};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[FUN]);

pub struct FunctionTypeReferenceSpacingRule;

impl RuleV2 for FunctionTypeReferenceSpacingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:function-type-reference-spacing")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == FUN
            && let Some(type_reference) = find_function_receiver_type_reference(ast, node)
        {
            if let Some(nullable_type_element) =
                ast.first_child_node(type_reference).filter(|&it| ast.element_type(it) == NULLABLE_TYPE)
            {
                visit_nodes_until_start_of_value_parameter_list(ast, ast.first_child_node(nullable_type_element), emit);
            }

            if ast.element_type(type_reference) != NULLABLE_TYPE {
                visit_nodes_until_start_of_value_parameter_list(ast, Some(type_reference), emit);
            }
        }
    }
}

fn find_function_receiver_type_reference(ast: &Ast, node: NodeId) -> Option<NodeId> {
    assert!(ast.element_type(node) == FUN, "IllegalArgumentException: Failed requirement.");
    let mut current_node = ast.first_child_node(node);
    while let Some(current) = current_node.filter(|&it| ast.element_type(it) != VALUE_PARAMETER_LIST) {
        if ast.element_type(current) == TYPE_REFERENCE {
            return Some(current);
        }
        current_node = ast.next_sibling(current);
    }
    None
}

fn visit_nodes_until_start_of_value_parameter_list(ast: &mut Ast, node: Option<NodeId>, emit: &mut Emit<'_>) {
    let mut current_node = node;
    while let Some(current) = current_node.filter(|&it| ast.element_type(it) != VALUE_PARAMETER_LIST) {
        let next_node = ast.next_sibling(current);
        remove_if_non_empty_white_space(ast, current, emit);
        current_node = next_node;
    }
}

fn remove_if_non_empty_white_space(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    if ast.is_white_space(node) && ast.text_length(node) > 0 {
        emit(ast, ast.start_offset(node), "Unexpected whitespace", true).if_autocorrect_allowed(|| ast.remove(node));
    }
}
