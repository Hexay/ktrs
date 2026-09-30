//! Port of ktlint-ruleset-standard `SpacingAroundDotRule.kt` (id `dot-spacing`).

use ktrs_ast::{Ast, NodeId};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

pub struct SpacingAroundDotRule;

impl RuleV2 for SpacingAroundDotRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:dot-spacing")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if !(ast.is_leaf_element(node) && ast.is_code(node) && ast.text_matches(node, ".")) {
            return;
        }
        if let Some(prev_leaf) = ast.prev_leaf(node).filter(|&it| ast.is_white_space_without_newline(it)) {
            let message = format!("Unexpected spacing before \"{}\"", ast.text(node));
            emit(ast, ast.start_offset(prev_leaf), &message, true).if_autocorrect_allowed(|| ast.remove(prev_leaf));
        }
        if let Some(next_leaf) = ast.next_leaf(node).filter(|&it| ast.is_white_space(it)) {
            let message = format!("Unexpected spacing after \"{}\"", ast.text(node));
            emit(ast, ast.start_offset(next_leaf), &message, true).if_autocorrect_allowed(|| ast.remove(next_leaf));
        }
    }
}
