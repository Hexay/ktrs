//! Port of ktlint-ruleset-standard `NoBlankLineAtStartOfFileRule.kt`.

use ktrs_ast::{Ast, NodeId, tree_util};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TraversalState};
use crate::rules::STANDARD_RULE_ABOUT;

#[derive(Default)]
pub struct NoBlankLineAtStartOfFileRule {
    traversal: TraversalState,
}

impl RuleV2 for NoBlankLineAtStartOfFileRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-blank-line-at-start-of-file")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn is_experimental(&self) -> bool {
        true
    }

    fn traversal_state(&self) -> Option<TraversalState> {
        Some(self.traversal)
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.is_root(node) {
            // `leavesForwardsIncludingSelf` is lazy: the seed of psiUtil `leaves` is taken up front, every later
            // leaf after the previous one was handled (and maybe removed).
            let first = ast.first_child_leaf_or_self(node);
            let mut seed = Some(tree_util::next_leaf(ast, first));
            let mut current = if ast.is_leaf(first) { Some(first) } else { seed.take().flatten() };
            while let Some(it) = current {
                if !(ast.is_white_space(it) || ast.text_length(it) == 0) {
                    break;
                }
                if ast.is_white_space(it) {
                    emit(ast, ast.start_offset(it), "Unexpected whitespace at start of file", true).if_autocorrect_allowed(|| ast.remove(it));
                }
                current = match seed.take() {
                    Some(next) => next,
                    None => tree_util::next_leaf(ast, it),
                };
            }
            self.traversal.stop_traversal_of_ast();
        }
    }
}
