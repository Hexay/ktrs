//! Port of ktlint-ruleset-standard `SpacingAroundUnaryOperatorRule.kt` (id `unary-op-spacing`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{POSTFIX_EXPRESSION, PREFIX_EXPRESSION};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[PREFIX_EXPRESSION, POSTFIX_EXPRESSION]);

/// Ensures there are no spaces around unary operators.
///
/// See [Kotlin Style Guide](https://kotlinlang.org/docs/reference/coding-conventions.html#horizontal-whitespace)
pub struct SpacingAroundUnaryOperatorRule;

impl RuleV2 for SpacingAroundUnaryOperatorRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:unary-op-spacing")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) != PREFIX_EXPRESSION && ast.element_type(node) != POSTFIX_EXPRESSION {
            return;
        }
        let children: Vec<NodeId> = ast.children(node).collect();

        // ignore: var a = + /* comment */ 1
        if children.iter().any(|&it| ast.is_part_of_comment(it)) {
            return;
        }

        if let Some(white_space) = children.iter().copied().find(|&it| ast.is_white_space(it)) {
            let message = format!("Unexpected spacing in {}", ast.text(node).replace('\n', "\\n"));
            emit(ast, ast.start_offset(white_space), &message, true).if_autocorrect_allowed(|| ast.remove(white_space));
        }
    }
}
