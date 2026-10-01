//! Port of ktlint-ruleset-standard `FunctionTypeModifierSpacingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{FUNCTION_TYPE, MODIFIER_LIST};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[MODIFIER_LIST]);

/// Lints and formats a single space between the modifier list and the function type
pub struct FunctionTypeModifierSpacingRule;

impl RuleV2 for FunctionTypeModifierSpacingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:function-type-modifier-spacing")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if let Some(function_type_node) = Some(node)
            .filter(|&it| ast.element_type(it) == MODIFIER_LIST)
            .and_then(|it| ast.next_code_sibling(it))
            .filter(|&it| ast.element_type(it) == FUNCTION_TYPE)
            .filter(|&it| !is_preceded_by_single_space(ast, it))
        {
            emit(ast, ast.start_offset(function_type_node), "Expected a single space between the modifier list and the function type", true)
                .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(function_type_node, " "));
        }
    }
}

fn is_preceded_by_single_space(ast: &Ast, node: NodeId) -> bool {
    ast.prev_sibling(node).is_some_and(|it| ast.is_white_space(it) && ast.text_matches(it, " "))
}
