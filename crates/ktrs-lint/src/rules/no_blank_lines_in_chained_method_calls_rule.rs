//! Port of ktlint-ruleset-standard `NoBlankLinesInChainedMethodCallsRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{DOT_QUALIFIED_EXPRESSION, WHITE_SPACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[WHITE_SPACE]);

pub struct NoBlankLinesInChainedMethodCallsRule;

impl RuleV2 for NoBlankLinesInChainedMethodCallsRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-blank-lines-in-chained-method-calls")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let is_blank_line = ast.is_white_space(node) && ast.leaf_text(node).contains("\n\n");
        if is_blank_line && ast.parent(node).map(|it| ast.element_type(it)) == Some(DOT_QUALIFIED_EXPRESSION) {
            emit(ast, ast.start_offset(node) + 1, "Needless blank line(s)", true).if_autocorrect_allowed(|| {
                let new_text = "\n".to_owned() + ast.leaf_text(node).split("\n\n").nth(1).expect("IndexOutOfBoundsException");
                ast.replace_text_with(node, &new_text);
            });
        }
    }
}
