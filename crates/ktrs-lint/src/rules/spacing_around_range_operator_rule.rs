//! Port of ktlint-ruleset-standard `SpacingAroundRangeOperatorRule.kt` (id `range-spacing`).

use ktrs_ast::psi::single_value;
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{RANGE, RANGE_UNTIL};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[RANGE, RANGE_UNTIL]);

pub struct SpacingAroundRangeOperatorRule;

impl RuleV2 for SpacingAroundRangeOperatorRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:range-spacing")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) != RANGE && ast.element_type(node) != RANGE_UNTIL {
            return;
        }
        let prev_leaf = ast.prev_leaf(node);
        let next_leaf = ast.next_leaf(node);
        let description = element_type_description(ast, node);
        if ast.is_white_space(prev_leaf) && ast.is_white_space(next_leaf) {
            let message = format!("Unexpected spacing around \"{description}\"");
            emit(ast, ast.start_offset(node), &message, true).if_autocorrect_allowed(|| {
                if let Some(prev_leaf) = prev_leaf {
                    ast.remove(prev_leaf);
                }
                if let Some(next_leaf) = next_leaf {
                    ast.remove(next_leaf);
                }
            });
        } else if let Some(prev_leaf) = prev_leaf.filter(|&it| ast.is_white_space(it)) {
            let message = format!("Unexpected spacing before \"{description}\"");
            emit(ast, ast.start_offset(prev_leaf), &message, true).if_autocorrect_allowed(|| ast.remove(prev_leaf));
        } else if let Some(next_leaf) = next_leaf.filter(|&it| ast.is_white_space(it)) {
            let message = format!("Unexpected spacing after \"{description}\"");
            emit(ast, ast.start_offset(next_leaf), &message, true).if_autocorrect_allowed(|| ast.remove(next_leaf));
        }
    }
}

fn element_type_description(ast: &Ast, node: NodeId) -> String {
    let element_type = ast.element_type(node);
    single_value(element_type).map_or_else(|| format!("{element_type:?}"), str::to_owned)
}
