//! Port of ktlint-ruleset-standard `BlankLineBeforePackage.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{IMPORT_LIST, PACKAGE_DIRECTIVE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{CODE_STYLE_PROPERTY, CodeStyleValue, EditorConfig};
use crate::rule::{About, Emit, RuleId, RuleV2, TraversalState};
use crate::rules::STANDARD_RULE_ABOUT;

/// Insert a blank line before the package statement.
#[derive(Default)]
pub struct BlankLineBeforePackage {
    traversal_state: TraversalState,
}

impl RuleV2 for BlankLineBeforePackage {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:blank-line-before-package")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn is_experimental(&self) -> bool {
        true
    }

    fn traversal_state(&self) -> Option<TraversalState> {
        Some(self.traversal_state)
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        if editor_config.get(&CODE_STYLE_PROPERTY) == CodeStyleValue::IntellijIdea {
            self.traversal_state.stop_traversal_of_ast();
        }
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        match ast.element_type(node) {
            PACKAGE_DIRECTIVE => {
                // Skips an empty package directive (no package statement).
                if let Some(insert_before_node) =
                    Some(node).filter(|&it| ast.first_child_node(it).is_some()).filter(|&it| !is_blank_line(ast, ast.prev_leaf(it)))
                {
                    emit(ast, ast.start_offset(insert_before_node), "Expected a blank line before the package statement", true).if_autocorrect_allowed(|| {
                        let indent = "\n".to_owned() + &ast.indent(node);
                        ast.upsert_whitespace_before_me(insert_before_node, &indent);
                    });
                }
                self.traversal_state.stop_traversal_of_ast();
            }
            IMPORT_LIST => self.traversal_state.stop_traversal_of_ast(),
            _ => {}
        }
    }
}

fn is_blank_line(ast: &Ast, node: Option<NodeId>) -> bool {
    node.is_none_or(|it| ast.is_white_space(it) && ast.leaf_text(it).matches('\n').count() > 1)
}
