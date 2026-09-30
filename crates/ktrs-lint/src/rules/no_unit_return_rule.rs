//! Port of ktlint-ruleset-standard `NoUnitReturnRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{COLON, FUN, LBRACE, TYPE_REFERENCE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

pub struct NoUnitReturnRule;

impl RuleV2 for NoUnitReturnRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-unit-return")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == TYPE_REFERENCE
            && ast.text_matches(node, "Unit")
            && ast.parent(node).map(|it| ast.element_type(it)) == Some(FUN)
            && ast.next_code_sibling(node).and_then(|it| ast.first_child_node(it)).map(|it| ast.element_type(it)) == Some(LBRACE)
        {
            emit(ast, ast.start_offset(node), "Unnecessary \"Unit\" return type", true).if_autocorrect_allowed(|| {
                if let Some(colon_node) = ast.parent(node).and_then(|it| ast.find_child_by_type(it, COLON)) {
                    // Remove space after colon when not followed by Unit node
                    if let Some(it) = ast.next_leaf(node).filter(|&it| ast.is_white_space(it)).filter(|&it| ast.next_leaf(it) != Some(node)) {
                        ast.remove(it);
                    }
                    ast.remove(colon_node);
                }
                if let Some(it) = ast
                    .prev_leaf(node)
                    .filter(|&it| ast.is_white_space(it))
                    .filter(|&it| ast.prev_leaf(it).map(|p| ast.element_type(p)) != Some(COLON))
                {
                    ast.remove(it);
                }
                ast.remove(node);
            });
        }
    }
}
