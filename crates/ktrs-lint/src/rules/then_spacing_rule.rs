//! Port of ktlint-ruleset-standard `ThenSpacingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::THEN;

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeQueries};
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

/// Checks spacing around then block in an if-statement
pub struct ThenSpacingRule;

impl RuleV2 for ThenSpacingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:then-spacing")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == THEN {
            visit_then(ast, node, emit);
        }
    }
}

fn visit_then(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    if !ast.is_white_space(ast.prev_sibling(node)) {
        emit(ast, ast.start_offset(node), "Expected a whitespace before 'then' block", true).if_autocorrect_allowed(|| {
            if let Some(prev_leaf) = ast.prev_leaf(node) {
                ast.upsert_whitespace_after_me(prev_leaf, " ");
            }
        });
    }
    if !(ast.next_sibling(node).is_none() || ast.is_white_space(ast.next_sibling(node))) {
        let last_leaf_in_then = ast.last_child_leaf_or_self(node);
        emit(ast, ast.end_offset(last_leaf_in_then), "Expected a whitespace after 'then' block", true)
            .if_autocorrect_allowed(|| ast.upsert_whitespace_after_me(last_leaf_in_then, " "));
    }
}
