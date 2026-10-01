//! Port of ktlint-ruleset-standard `CommentWrappingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{BLOCK_COMMENT, LBRACE, RBRACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines};
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[BLOCK_COMMENT]);

/// Checks external wrapping of block comments. Wrapping inside the comment is not altered.
pub struct CommentWrappingRule;

impl RuleV2 for CommentWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:comment-wrapping")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*INDENT_SIZE_PROPERTY), PropertyRef::from(&*INDENT_STYLE_PROPERTY)]
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == BLOCK_COMMENT {
            let before_block_comment = ast
                .leaves(node, false)
                .take_while(|&it| ast.is_white_space_without_newline(it))
                .next()
                .unwrap_or_else(|| ast.first_child_leaf_or_self(node));
            let after_block_comment = ast
                .leaves(node, true)
                .take_while(|&it| ast.is_white_space_without_newline(it))
                .next()
                .unwrap_or_else(|| ast.last_child_leaf_or_self(node));
            if !is_whitespace_with_newline_or_null(ast, ast.prev_leaf(before_block_comment))
                && !is_whitespace_with_newline_or_null(ast, ast.next_leaf(after_block_comment))
            {
                if ast.has_new_line_in_closed_range(before_block_comment, after_block_comment) {
                    // Not fixable: `val foo = "foo" /*\nsome comment\n*/ val bar = "bar"`.
                    emit(
                        ast,
                        ast.start_offset(node),
                        "A block comment starting on same line as another element and ending on another line before another element is \
                         disallowed",
                        false,
                    );
                } else if ast.prev_leaf(before_block_comment).map(|it| ast.element_type(it)) == Some(LBRACE)
                    && ast.next_leaf(after_block_comment).map(|it| ast.element_type(it)) == Some(RBRACE)
                {
                    // Allows a single line block containing a block comment: `val foo = { /* no-op */ }`.
                    return;
                } else {
                    emit(ast, ast.start_offset(node), "A block comment in between other elements on the same line is disallowed", false);
                }
                return;
            }
            if ast.prev_leaf(before_block_comment).is_some_and(|it| !is_whitespace_with_newline_or_null(ast, Some(it)))
                && ast.text_contains(node, '\n')
            {
                emit(ast, ast.start_offset(node), "A block comment after any other element on the same line must be separated by a new line", false);
            }
            if let Some(next_leaf) = ast.next_leaf(after_block_comment).filter(|&it| !is_whitespace_with_newline_or_null(ast, Some(it))) {
                emit(ast, ast.start_offset(next_leaf), "A block comment may not be followed by any other element on that same line", true)
                    .if_autocorrect_allowed(|| {
                        let indent = ast.indent(node);
                        ast.upsert_whitespace_before_me(next_leaf, &indent);
                    });
            }
        }
    }
}

fn is_whitespace_with_newline_or_null(ast: &Ast, node: Option<NodeId>) -> bool {
    node.is_none_or(|it| ast.is_white_space_with_newline(it))
}
