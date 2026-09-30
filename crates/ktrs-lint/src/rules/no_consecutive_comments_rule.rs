//! Port of ktlint-ruleset-standard `NoConsecutiveCommentsRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{BLOCK_COMMENT, EOL_COMMENT, KDOC_END, KDOC_START};

use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

/// Consecutive comments should be disallowed in following cases:
/// - Any mix of a consecutive kdoc, a block comment or an EOL comment unless separated by a blank line in between
/// - Consecutive KDocs (even when separated by a blank line)
/// - Consecutive block comments (even when separated by a blank line)
///
/// Consecutive EOL comments are always allowed as they are often used instead of a block comment.
pub struct NoConsecutiveCommentsRule;

impl RuleV2 for NoConsecutiveCommentsRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-consecutive-comments")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn is_official_code_style(&self) -> bool {
        true
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let Some(previous_comment) = Some(node)
            .filter(|&it| is_start_of_comment(ast, Some(it)))
            .and_then(|it| ast.prev_leaf_matching(it, |it| !ast.is_white_space(it)))
            .filter(|&previous_non_white_space| is_end_of_comment(ast, Some(previous_non_white_space)))
        else {
            return;
        };
        let (previous_type, node_type) = (ast.element_type(previous_comment), ast.element_type(node));
        let offset = ast.start_offset(node);
        if previous_type == KDOC_END && node_type == KDOC_START {
            emit(ast, offset, &format!("{} may not be preceded by {}", comment_type(ast, node), comment_type(ast, previous_comment)), false);
        } else if previous_type == KDOC_END && node_type != KDOC_START {
            emit(
                ast,
                offset,
                &format!(
                    "{} may not be preceded by {}. Reversed order is allowed though when separated by a newline.",
                    comment_type(ast, node),
                    comment_type(ast, previous_comment)
                ),
                false,
            );
        } else if previous_type == BLOCK_COMMENT && node_type == BLOCK_COMMENT {
            emit(ast, offset, &format!("{} may not be preceded by {}", comment_type(ast, node), comment_type(ast, previous_comment)), false);
        } else if previous_type == EOL_COMMENT && node_type == EOL_COMMENT {
            // Consecutive EOL comments are allowed.
        } else if previous_type != node_type
            && ast.prev_leaf(node).filter(|&it| ast.is_white_space(it)).map_or(0, |it| ast.leaf_text(it).matches('\n').count()) > 1
        {
            // Different comment types are allowed when separated by a blank line.
        } else {
            emit(
                ast,
                offset,
                &format!("{} may not be preceded by {} unless separated by a blank line", comment_type(ast, node), comment_type(ast, previous_comment)),
                false,
            );
        }
    }
}

fn is_start_of_comment(ast: &Ast, node: Option<NodeId>) -> bool {
    matches!(node.map(|it| ast.element_type(it)), Some(EOL_COMMENT | BLOCK_COMMENT | KDOC_START))
}

fn is_end_of_comment(ast: &Ast, node: Option<NodeId>) -> bool {
    matches!(node.map(|it| ast.element_type(it)), Some(EOL_COMMENT | BLOCK_COMMENT | KDOC_END))
}

fn comment_type(ast: &Ast, node: NodeId) -> String {
    match ast.element_type(node) {
        EOL_COMMENT => "an EOL comment".to_owned(),
        BLOCK_COMMENT => "a block comment".to_owned(),
        KDOC_START | KDOC_END => "a KDoc".to_owned(),
        other => other.debug_name().to_lowercase(),
    }
}
