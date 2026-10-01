//! Port of ktlint-ruleset-standard `SpacingBetweenFunctionNameAndOpeningParenthesisRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{FUN, IDENTIFIER};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[FUN]);

pub struct SpacingBetweenFunctionNameAndOpeningParenthesisRule;

impl RuleV2 for SpacingBetweenFunctionNameAndOpeningParenthesisRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:spacing-between-function-name-and-opening-parenthesis")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if let Some(white_space) = Some(node)
            .filter(|&it| ast.element_type(it) == FUN)
            .and_then(|it| ast.find_child_by_type(it, IDENTIFIER))
            .and_then(|it| ast.next_sibling(it))
            .filter(|&it| ast.is_white_space(it))
        {
            emit(ast, ast.start_offset(white_space), "Unexpected whitespace", true).if_autocorrect_allowed(|| ast.remove(white_space));
        }
    }
}
