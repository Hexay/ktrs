//! Port of ktlint-ruleset-standard `NoConsecutiveBlankLinesRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{CLASS, IDENTIFIER, PRIMARY_CONSTRUCTOR, WHITE_SPACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[WHITE_SPACE]);

pub struct NoConsecutiveBlankLinesRule;

impl RuleV2 for NoConsecutiveBlankLinesRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-consecutive-blank-lines")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.is_white_space(node) && ast.prev_sibling(node).is_some() {
            let new_line_count = ast.leaf_text(node).matches('\n').count();
            if new_line_count < 2 {
                return;
            }
            let text = ast.leaf_text(node).to_owned();
            let eof = ast.next_leaf(node).is_none();
            let between_class_and_primary_constructor = is_between_class_and_primary_constructor(ast, node);
            if new_line_count > 2 || eof || between_class_and_primary_constructor {
                let split: Vec<&str> = text.split('\n').collect();
                let offset = ast.start_offset(node) + split[0].len() + split[1].len() + if between_class_and_primary_constructor { 1 } else { 2 };
                emit(ast, offset, "Needless blank line(s)", true).if_autocorrect_allowed(|| {
                    let mut new_text = String::new();
                    new_text.push_str(split[0]);
                    new_text.push('\n');
                    if !eof && !between_class_and_primary_constructor {
                        new_text.push('\n');
                    }
                    new_text.push_str(split[split.len() - 1]);
                    ast.replace_text_with(node, &new_text);
                });
            }
        }
    }
}

fn is_between_class_and_primary_constructor(ast: &Ast, node: NodeId) -> bool {
    ast.prev_code_leaf(node).is_some_and(|prev_node| {
        ast.element_type(prev_node) == IDENTIFIER
            && ast.parent(prev_node).map(|it| ast.element_type(it)) == Some(CLASS)
            && ast.next_sibling(node).map(|it| ast.element_type(it)) == Some(PRIMARY_CONSTRUCTOR)
    })
}
