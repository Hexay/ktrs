//! Port of ktlint-ruleset-standard `ValueParameterCommentRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::VALUE_PARAMETER;

use crate::ast_node_extension::AstNodeExtension;
use crate::element_type::KDOC;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

/// Disallows comments inside a value parameter, except a leading KDoc. One of the rules split from
/// `DiscouragedCommentLocationRule`.
pub struct ValueParameterCommentRule;

impl RuleV2 for ValueParameterCommentRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:value-parameter-comment")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let parent = ast.parent(node);
        if parent.map(|it| ast.element_type(it)) == Some(VALUE_PARAMETER) && ast.is_part_of_comment(node) {
            // EOL and block comments before a value parameter are children of the list, so only a KDoc can start one.
            if ast.element_type(node) == KDOC && parent.and_then(|it| ast.first_child_node(it)) == Some(node) {
                return;
            }
            emit(
                ast,
                ast.start_offset(node),
                "A comment inside or on same line after a 'value_parameter' is not allowed. It may be placed on a separate line above.",
                false,
            );
        }
    }
}
