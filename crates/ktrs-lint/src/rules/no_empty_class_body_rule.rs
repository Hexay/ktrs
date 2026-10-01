//! Port of ktlint-ruleset-standard `NoEmptyClassBodyRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{CLASS_BODY, LBRACE, OBJECT_LITERAL, RBRACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[CLASS_BODY]);

pub struct NoEmptyClassBodyRule;

impl RuleV2 for NoEmptyClassBodyRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-empty-class-body")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == CLASS_BODY && is_empty_block_body(ast, node) && !ast.is_part_of(node, OBJECT_LITERAL) && is_not_companion(ast, node) {
            emit(ast, ast.start_offset(node), "Unnecessary block (\"{}\")", true).if_autocorrect_allowed(|| {
                if let Some(prev_sibling) = ast.prev_sibling(node).filter(|&it| ast.is_white_space(it)) {
                    ast.remove(prev_sibling);
                }
                ast.remove(node);
            });
        }
    }
}

fn is_empty_block_body(ast: &Ast, node: NodeId) -> bool {
    ast.first_child_node(node).is_some_and(|first_child_node| {
        ast.element_type(first_child_node) == LBRACE
            && ast.next_leaf_matching(first_child_node, |it| !ast.is_white_space(it)).map(|it| ast.element_type(it)) == Some(RBRACE)
    })
}

fn is_not_companion(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .and_then(|it| ast.first_child_node(it))
        .is_none_or(|first_child_node| !ast.children(first_child_node).any(|it| ast.text_matches(it, "companion")))
}
