//! Port of ktlint-ruleset-standard `CommentSpacingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::EOL_COMMENT;

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

pub struct CommentSpacingRule;

impl RuleV2 for CommentSpacingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:comment-spacing")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == EOL_COMMENT {
            let prev_leaf = ast.prev_leaf(node);
            if !ast.is_white_space(prev_leaf) && prev_leaf.is_some_and(|it| ast.is_leaf_element(it)) {
                emit(ast, ast.start_offset(node), "Missing space before //", true).if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(node, " "));
            }
            let text = ast.leaf_text(node);
            if text.len() != 2
                && !text.starts_with("// ")
                && !text.starts_with("//noinspection")
                && !text.starts_with("//region")
                && !text.starts_with("//endregion")
                && !text.starts_with("//language=")
            {
                let new_text = format!("// {}", text.strip_prefix("//").unwrap_or(text));
                emit(ast, ast.start_offset(node), "Missing space after //", true).if_autocorrect_allowed(|| {
                    // Replacing the text in place would detach the node, which rules running after this one still hold.
                    let new_eol_comment = ast.new_leaf(EOL_COMMENT, &new_text);
                    ast.raw_insert_before_me(node, new_eol_comment);
                    ast.remove(node);
                });
            }
        }
    }
}
