//! Port of ktlint-ruleset-standard `BlankLineBeforeImports.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{IMPORT_LIST, PACKAGE_DIRECTIVE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{CODE_STYLE_PROPERTY, CodeStyleValue, EditorConfig};
use crate::rule::{About, Emit, RuleId, RuleV2, TraversalState};
use crate::rules::STANDARD_RULE_ABOUT;

/// Insert a blank line before the imports list: <https://developer.android.com/kotlin/style-guide#structure>
#[derive(Default)]
pub struct BlankLineBeforeImports {
    traversal_state: TraversalState,
}

impl RuleV2 for BlankLineBeforeImports {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:blank-line-before-imports")
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
        if ast.element_type(node) != PACKAGE_DIRECTIVE {
            return;
        }
        // Too many blank lines are left to no-consecutive-blank-lines.
        let whitespace = ast
            .siblings(node, true)
            .take_while(|&it| ast.element_type(it) != IMPORT_LIST)
            .find(|&it| ast.is_white_space_with_newline(it))
            .filter(|&it| ast.leaf_text(it).matches('\n').count() == 1);
        if let Some(whitespace) = whitespace {
            emit(ast, ast.start_offset(whitespace) + 1, "Expected a blank line before the import(s)", true)
                .if_autocorrect_allowed(|| ast.replace_text_with(whitespace, "\n\n"));
        }
    }
}
