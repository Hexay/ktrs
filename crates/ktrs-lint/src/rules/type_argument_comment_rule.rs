//! Port of ktlint-ruleset-standard `TypeArgumentCommentRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::{BLOCK_COMMENT, EOL_COMMENT, TYPE_ARGUMENT_LIST, TYPE_PROJECTION};

use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

const TYPE_ARGUMENT_TOKEN_SET: TokenSet = TokenSet::create(&[TYPE_PROJECTION, TYPE_ARGUMENT_LIST]);

/// Disallows comments at places in a type argument (list) that make the code hard to read or the rules complex. One of
/// the rules split from `DiscouragedCommentLocationRule`.
pub struct TypeArgumentCommentRule;

impl RuleV2 for TypeArgumentCommentRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:type-argument-comment")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let parent_type = ast.parent(node).map(|it| ast.element_type(it));
        if parent_type.is_some_and(|it| TYPE_ARGUMENT_TOKEN_SET.contains(it)) && ast.is_part_of_comment(node) {
            if let EOL_COMMENT | BLOCK_COMMENT = ast.element_type(node) {
                if parent_type == Some(TYPE_PROJECTION) {
                    emit(
                        ast,
                        ast.start_offset(node),
                        "A (block or EOL) comment inside or on same line after a 'type_projection' is not allowed. It may be placed on a \
                         separate line above.",
                        false,
                    );
                } else if parent_type == Some(TYPE_ARGUMENT_LIST) {
                    // Allowed on a line of its own; after an element it is unclear which element it belongs to.
                    if !ast.is_white_space_with_newline(ast.prev_leaf(node)) {
                        emit(ast, ast.start_offset(node), "A comment in a 'type_argument_list' is only allowed when placed on a separate line", false);
                    }
                }
            }
        }
    }
}
