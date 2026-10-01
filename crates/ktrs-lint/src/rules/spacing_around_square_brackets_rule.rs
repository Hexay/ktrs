//! Port of ktlint-ruleset-standard `SpacingAroundSquareBracketsRule.kt` (id `square-brackets-spacing`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{COLLECTION_LITERAL_EXPRESSION, DESTRUCTURING_DECLARATION, KDOC_MARKDOWN_LINK, LBRACKET, RBRACKET};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[LBRACKET, RBRACKET]);

/// Ensures there are no extra spaces around square brackets.
///
/// See https://kotlinlang.org/docs/reference/coding-conventions.html#horizontal-whitespace
pub struct SpacingAroundSquareBracketsRule;

impl RuleV2 for SpacingAroundSquareBracketsRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:square-brackets-spacing")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let element_type = ast.element_type(node);
        if element_type != LBRACKET && element_type != RBRACKET {
            return;
        }
        let prev_leaf = ast.prev_leaf(node);
        let next_leaf = ast.next_leaf(node);
        let spacing_before = match ast.parent(node).map(|p| ast.element_type(p)) {
            // Allow `@see [Foo] for more information` in KDoc
            Some(KDOC_MARKDOWN_LINK) => false,
            // Allow a newline after `fooBaz = [`; disallow `["foo", "bar" ]`
            Some(COLLECTION_LITERAL_EXPRESSION | DESTRUCTURING_DECLARATION) => {
                element_type == RBRACKET && ast.is_white_space_without_newline(prev_leaf)
            }
            _ => ast.is_white_space_without_newline(prev_leaf),
        };
        // Allow a newline after `bar[` and `fooBaz = [`; disallow `[ "foo", "bar"]`
        let spacing_after = element_type == LBRACKET && ast.is_white_space_without_newline(next_leaf);
        if spacing_before && spacing_after {
            let message = format!("Unexpected spacing around '{}'", ast.text(node));
            emit(ast, ast.start_offset(node), &message, true).if_autocorrect_allowed(|| {
                if let Some(prev_leaf) = prev_leaf {
                    ast.remove(prev_leaf);
                }
                if let Some(next_leaf) = next_leaf {
                    ast.remove(next_leaf);
                }
            });
        } else if spacing_before {
            let prev_leaf = prev_leaf.expect("NullPointerException: prevLeaf!!");
            let message = format!("Unexpected spacing before '{}'", ast.text(node));
            emit(ast, ast.start_offset(prev_leaf), &message, true).if_autocorrect_allowed(|| ast.remove(prev_leaf));
        } else if spacing_after {
            let message = format!("Unexpected spacing after '{}'", ast.text(node));
            emit(ast, ast.start_offset(node) + 1, &message, true)
                .if_autocorrect_allowed(|| ast.remove(next_leaf.expect("NullPointerException: nextLeaf!!")));
        }
    }
}
