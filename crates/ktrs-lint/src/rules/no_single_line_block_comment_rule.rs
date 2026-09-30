//! Port of ktlint-ruleset-standard `NoSingleLineBlockCommentRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{BLOCK_COMMENT, EOL_COMMENT};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::internal::kotlin_string::{remove_surrounding, trim};

/// A block comment following another element on the same line is replaced with an EOL comment, if possible.
pub struct NoSingleLineBlockCommentRule;

impl RuleV2 for NoSingleLineBlockCommentRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-single-line-block-comment")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*INDENT_SIZE_PROPERTY), PropertyRef::from(&*INDENT_STYLE_PROPERTY)]
    }

    fn is_official_code_style(&self) -> bool {
        true
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == BLOCK_COMMENT {
            let after_block_comment = ast
                .leaves(node, true)
                .take_while(|&it| ast.is_white_space_without_newline(it))
                .next()
                .unwrap_or_else(|| ast.last_child_leaf_or_self(node));
            if !ast.text_contains(node, '\n') && is_whitespace_with_newline_or_null(ast, ast.next_leaf(after_block_comment)) {
                emit(ast, ast.start_offset(node), "Replace the block comment with an EOL comment", true).if_autocorrect_allowed(|| {
                    let before_block_comment = ast
                        .leaves(node, false)
                        .take_while(|&it| ast.is_white_space_without_newline(it))
                        .next()
                        .unwrap_or_else(|| ast.first_child_leaf_or_self(node));
                    if ast.prev_leaf(before_block_comment).is_some_and(|it| !is_whitespace_with_newline_or_null(ast, Some(it))) {
                        ast.upsert_whitespace_before_me(node, " ");
                    }
                    replace_with_end_of_line_comment(ast, node);
                });
            }
        }
    }
}

fn replace_with_end_of_line_comment(ast: &mut Ast, node: NodeId) {
    let content = trim(remove_surrounding(ast.leaf_text(node), "/*", "*/")).to_owned();
    let eol_comment = ast.new_leaf(EOL_COMMENT, &format!("// {content}"));
    ast.raw_insert_before_me(node, eol_comment);
    ast.raw_remove(node);
}

fn is_whitespace_with_newline_or_null(ast: &Ast, node: Option<NodeId>) -> bool {
    node.is_none_or(|it| ast.is_white_space_with_newline(it))
}
