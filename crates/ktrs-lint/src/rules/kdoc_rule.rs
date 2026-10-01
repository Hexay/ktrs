//! Port of ktlint-ruleset-standard `KdocRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::{CLASS, ENUM_ENTRY, FILE, FUN, OBJECT_DECLARATION, PROPERTY, SECONDARY_CONSTRUCTOR, TYPEALIAS, VALUE_PARAMETER};

use crate::ast_node_extension::AstNodeExtension;
use crate::element_type::KDOC;
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

const ALLOWED_PARENT_ELEMENT_TYPES: TokenSet =
    TokenSet::create(&[CLASS, ENUM_ENTRY, FUN, OBJECT_DECLARATION, PROPERTY, SECONDARY_CONSTRUCTOR, TYPEALIAS, VALUE_PARAMETER]);
const VISITED_TYPES: TokenSet = TokenSet::create(&[KDOC]);

/// Disallow KDoc except of classes, functions and xxx
pub struct KdocRule;

impl RuleV2 for KdocRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:kdoc")
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
        let Some(parent) = Some(node).filter(|&it| ast.element_type(it) == KDOC).and_then(|it| ast.parent(it)) else { return };
        let parent_type = ast.element_type(parent);
        if ALLOWED_PARENT_ELEMENT_TYPES.contains(parent_type) {
            if ast.first_child_node(parent) != Some(node) {
                emit(ast, ast.start_offset(node), &format!("A KDoc is allowed only at start of '{}'", parent_type.debug_name().to_lowercase()), false);
            }
        } else if parent_type == FILE {
            emit(ast, ast.start_offset(node), "A dangling toplevel KDoc is not allowed", false);
        } else {
            emit(ast, ast.start_offset(node), &format!("A KDoc is not allowed inside '{}'", parent_type.debug_name().to_lowercase()), false);
        }
    }
}
