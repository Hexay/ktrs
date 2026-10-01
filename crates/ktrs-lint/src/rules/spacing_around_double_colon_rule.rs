//! Port of ktlint-ruleset-standard `SpacingAroundDoubleColonRule.kt` (id `double-colon-spacing`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{CALLABLE_REFERENCE_EXPRESSION, CLASS_LITERAL_EXPRESSION, COLONCOLON};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[COLONCOLON]);

pub struct SpacingAroundDoubleColonRule;

impl RuleV2 for SpacingAroundDoubleColonRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:double-colon-spacing")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) != COLONCOLON {
            return;
        }
        let prev_leaf = ast.prev_leaf(node);
        let next_leaf = ast.next_leaf(node);

        let mut remove_single_white_space = false;
        let spacing_before = if ast.is_part_of(node, CLASS_LITERAL_EXPRESSION) && ast.is_white_space(prev_leaf) {
            // Clazz::class
            true
        } else if ast.is_part_of(node, CALLABLE_REFERENCE_EXPRESSION) && ast.is_white_space(prev_leaf) {
            // String::length, ::isOdd
            if ast.prev_sibling(node).is_none() {
                // compose(length, ::isOdd), val predicate = ::isOdd
                remove_single_white_space = true;
                prev_leaf.filter(|&it| ast.is_white_space_without_newline(it)).is_some_and(|it| ast.text_length(it) > 1)
            } else {
                // String::length, List<String>::isEmpty
                ast.is_white_space_without_newline(prev_leaf)
            }
        } else {
            false
        };
        let spacing_after = ast.is_white_space(next_leaf);
        if spacing_before && spacing_after {
            let message = format!("Unexpected spacing around \"{}\"", ast.text(node));
            emit(ast, ast.start_offset(node), &message, true).if_autocorrect_allowed(|| {
                remove_self(ast, prev_leaf.expect("NullPointerException: prevLeaf!!"), remove_single_white_space);
                ast.remove(next_leaf.expect("NullPointerException: nextLeaf!!"));
            });
        } else if spacing_before {
            let prev_leaf = prev_leaf.expect("NullPointerException: prevLeaf!!");
            let message = format!("Unexpected spacing before \"{}\"", ast.text(node));
            emit(ast, ast.start_offset(prev_leaf), &message, true)
                .if_autocorrect_allowed(|| remove_self(ast, prev_leaf, remove_single_white_space));
        } else if spacing_after {
            let next_leaf = next_leaf.expect("NullPointerException: nextLeaf!!");
            let message = format!("Unexpected spacing after \"{}\"", ast.text(node));
            emit(ast, ast.start_offset(next_leaf), &message, true).if_autocorrect_allowed(|| ast.remove(next_leaf));
        }
    }
}

fn remove_self(ast: &mut Ast, node: NodeId, remove_single_white_space: bool) {
    if remove_single_white_space {
        let text = ast.leaf_text(node);
        let text = text[..text.len() - 1].to_owned();
        ast.replace_text_with(node, &text);
    } else {
        ast.remove(node);
    }
}
