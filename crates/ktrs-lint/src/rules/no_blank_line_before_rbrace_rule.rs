//! Port of ktlint-ruleset-standard `NoBlankLineBeforeRbraceRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{RBRACE, WHITE_SPACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[WHITE_SPACE]);

pub struct NoBlankLineBeforeRbraceRule;

impl RuleV2 for NoBlankLineBeforeRbraceRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-blank-line-before-rbrace")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.is_white_space_with_newline(node) && ast.next_leaf(node).map(|it| ast.element_type(it)) == Some(RBRACE) {
            let text = ast.leaf_text(node).to_owned();
            let split: Vec<&str> = text.split('\n').collect();
            if split.len() > 2 {
                emit(ast, ast.start_offset(node) + split[0].len() + split[1].len() + 1, "Unexpected blank line(s) before \"}\"", true)
                    .if_autocorrect_allowed(|| ast.replace_text_with(node, &format!("{}\n{}", split[0], split[split.len() - 1])));
            }
        }
    }
}
