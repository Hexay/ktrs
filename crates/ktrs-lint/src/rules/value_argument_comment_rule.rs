//! Port of ktlint-ruleset-standard `ValueArgumentCommentRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::VALUE_ARGUMENT;

use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

/// Disallows comments inside a value argument. One of the rules split from `DiscouragedCommentLocationRule`.
pub struct ValueArgumentCommentRule;

impl RuleV2 for ValueArgumentCommentRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:value-argument-comment")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.parent(node).map(|it| ast.element_type(it)) == Some(VALUE_ARGUMENT) && ast.is_part_of_comment(node) {
            emit(
                ast,
                ast.start_offset(node),
                "A (block or EOL) comment inside or on same line after a 'value_argument' is not allowed. It may be placed on a \
                 separate line above.",
                false,
            );
        }
    }
}
