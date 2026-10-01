//! Port of ktlint-ruleset-standard `NoEmptyFirstLineInMethodBlockRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{CLASS_BODY, FUN, LBRACE, WHITE_SPACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[WHITE_SPACE]);

pub struct NoEmptyFirstLineInMethodBlockRule;

impl RuleV2 for NoEmptyFirstLineInMethodBlockRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-empty-first-line-in-method-block")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        // The CLASS_BODY check allows `fun fn() = object : Builder {\n\n fun stuff() = Unit }`.
        if ast.is_white_space_with_newline(node)
            && ast.prev_leaf(node).map(|it| ast.element_type(it)) == Some(LBRACE)
            && ast.is_part_of(node, FUN)
            && ast.parent(node).map(|it| ast.element_type(it)) != Some(CLASS_BODY)
        {
            let text = ast.leaf_text(node).to_owned();
            let split: Vec<&str> = text.split('\n').collect();
            if split.len() > 2 {
                emit(ast, ast.start_offset(node) + 1, "First line in a method block should not be empty", true)
                    .if_autocorrect_allowed(|| ast.replace_text_with(node, &format!("{}\n{}", split[0], split[split.len() - 1])));
            }
        }
    }
}
