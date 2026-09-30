//! Port of ktlint-ruleset-standard `NoLineBreakAfterElseRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{ELSE_KEYWORD, IF_KEYWORD, LBRACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

pub struct NoLineBreakAfterElseRule;

impl RuleV2 for NoLineBreakAfterElseRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-line-break-after-else")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.is_white_space_with_newline(node)
            && ast.prev_leaf(node).map(|it| ast.element_type(it)) == Some(ELSE_KEYWORD)
            && ast.next_leaf(node).is_some_and(|it| matches!(ast.element_type(it), IF_KEYWORD | LBRACE))
        {
            emit(ast, ast.start_offset(node) + 1, "Unexpected line break after \"else\"", true)
                .if_autocorrect_allowed(|| ast.replace_text_with(node, " "));
        }
    }
}
