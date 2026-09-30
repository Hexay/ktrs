//! Port of ktlint-ruleset-standard `NullableTypeSpacingRule.kt` (id `nullable-type-spacing`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::QUEST;

use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

pub struct NullableTypeSpacingRule;

impl RuleV2 for NullableTypeSpacingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:nullable-type-spacing")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) != QUEST {
            return;
        }
        if let Some(white_space_before_quest) = ast.prev_leaf(node).filter(|&it| ast.is_white_space(it)) {
            emit(ast, ast.start_offset(white_space_before_quest), "Unexpected whitespace", true)
                .if_autocorrect_allowed(|| ast.raw_remove(white_space_before_quest));
        }
    }
}
