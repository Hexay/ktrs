//! Port of ktlint-ruleset-standard `FunKeywordSpacingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{FUN_KEYWORD, IDENTIFIER, WHITE_SPACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[FUN_KEYWORD]);

/// Lints and formats the spacing after the fun keyword
pub struct FunKeywordSpacingRule;

impl RuleV2 for FunKeywordSpacingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:fun-keyword-spacing")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let Some(leaf_after_fun_keyword) = Some(node).filter(|&it| ast.element_type(it) == FUN_KEYWORD).and_then(|it| ast.next_leaf(it))
        else {
            return;
        };
        if ast.element_type(leaf_after_fun_keyword) == WHITE_SPACE && !ast.text_matches(leaf_after_fun_keyword, " ") {
            emit(ast, ast.start_offset(leaf_after_fun_keyword), "Single space expected after the fun keyword", true)
                .if_autocorrect_allowed(|| ast.replace_text_with(leaf_after_fun_keyword, " "));
        } else if ast.element_type(leaf_after_fun_keyword) == IDENTIFIER {
            // Identifier can only be adjacent to fun keyword in case the identifier is wrapped between backticks:
            //     fun`foo`() {}
            emit(ast, ast.start_offset(leaf_after_fun_keyword), "Space expected between the fun keyword and backtick", true)
                .if_autocorrect_allowed(|| {
                    if let Some(parent) = ast.parent(leaf_after_fun_keyword) {
                        let white_space = ast.new_leaf(WHITE_SPACE, " ");
                        ast.add_child(parent, white_space, Some(leaf_after_fun_keyword));
                    }
                });
        }
    }
}
