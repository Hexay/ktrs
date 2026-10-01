//! Port of ktlint-ruleset-standard `KdocWrappingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{KDOC_END, KDOC_START};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::element_type::KDOC;
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[KDOC]);

/// Checks external wrapping of KDoc comment. Wrapping inside the KDoc comment is not altered.
pub struct KdocWrappingRule;

impl RuleV2 for KdocWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:kdoc-wrapping")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*INDENT_SIZE_PROPERTY), PropertyRef::from(&*INDENT_STYLE_PROPERTY)]
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == KDOC {
            if ast
                .find_child_by_type(node, KDOC_START)
                .and_then(|it| ast.prev_leaf(it))
                .is_some_and(|it| !ast.is_white_space_with_newline(it))
            {
                emit(ast, ast.start_offset(node), "A KDoc comment after any other element on the same line must be separated by a new line", false);
            }
            if let Some(next_leaf) =
                ast.find_child_by_type(node, KDOC_END).and_then(|it| ast.next_leaf(it)).filter(|&it| !ast.is_white_space_with_newline(it))
            {
                emit(ast, ast.start_offset(next_leaf), "A KDoc comment may not be followed by any other element on that same line", true)
                    .if_autocorrect_allowed(|| {
                        let indent = ast.indent(node);
                        ast.upsert_whitespace_after_me(node, &indent);
                    });
            }
        }
    }
}
