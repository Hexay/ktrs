//! Port of ktlint-ruleset-standard `SpacingAroundCommaRule.kt` (id `comma-spacing` in 2.0).

use ktrs_ast::{Ast, NodeId};
use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::{GT, KDOC_CODE_SPAN_TEXT, RBRACKET, RPAR};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{EditorConfig, KtlintVersion};
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

const R_TOKEN_SET: TokenSet = TokenSet::create(&[RPAR, RBRACKET, GT]);

#[derive(Default)]
pub struct SpacingAroundCommaRule {
    ktlint_version: KtlintVersion,
}

impl RuleV2 for SpacingAroundCommaRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:comma-spacing")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.ktlint_version = KtlintVersion::of(editor_config);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        // KDoc inline code spans (`` `,` ``) tokenize to a lone `,` leaf only with the newer KDoc lexer
        // (Kotlin 2.4, ktlint 2.0); ktlint 1.8 (Kotlin 2.2) has no code-span state, so the comma stays
        // inside KDOC_TEXT and is never flagged. ktrs shares the newer lexer, so skip it in 1.8 mode.
        if self.ktlint_version.is_1_8() && ast.element_type(node) == KDOC_CODE_SPAN_TEXT {
            return;
        }
        if ast.is_leaf_element(node) && ast.text_matches(node, ",") && !ast.is_part_of_string(node) {
            if let Some(prev_leaf) = ast.prev_leaf(node).filter(|&it| ast.is_white_space(it)) {
                let message = format!("Unexpected spacing before \"{}\"", ast.text(node));
                emit(ast, ast.start_offset(prev_leaf), &message, true).if_autocorrect_allowed(|| {
                    let is_preceded_by_comment = ast
                        .prev_leaf_matching(prev_leaf, |it| !ast.is_white_space(it))
                        .is_some_and(|it| ast.is_part_of_comment(it));
                    if is_preceded_by_comment && ast.is_white_space_with_newline(prev_leaf) {
                        // If comma is on new line and preceded by a comment, it should be moved before this comment
                        // https://github.com/ktlint/ktlint/issues/367
                        let previous_statement = ast.prev_code_leaf(node).expect("NullPointerException: prevCodeLeaf!!");
                        if let Some(parent) = ast.parent(previous_statement) {
                            let clone = ast.clone(node);
                            let anchor = ast.next_sibling(previous_statement);
                            ast.add_child(parent, clone, anchor);
                        }
                        if let Some(next_leaf) = ast.next_leaf(node).filter(|&it| ast.is_white_space(it)) {
                            ast.remove(next_leaf);
                        }
                        ast.remove(node);
                    } else {
                        ast.remove(prev_leaf);
                    }
                });
            }
            if ast
                .next_leaf(node)
                .filter(|&it| !ast.is_white_space(it))
                .is_some_and(|it| !R_TOKEN_SET.contains(ast.element_type(it)))
            {
                let message = format!("Missing spacing after \"{}\"", ast.text(node));
                emit(ast, ast.start_offset(node) + 1, &message, true).if_autocorrect_allowed(|| ast.upsert_whitespace_after_me(node, " "));
            }
        }
    }
}
