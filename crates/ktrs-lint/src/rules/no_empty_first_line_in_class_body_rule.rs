//! Port of ktlint-ruleset-standard `NoEmptyFirstLineInClassBodyRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::CLASS_BODY;

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{EditorConfig, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[CLASS_BODY]);

pub struct NoEmptyFirstLineInClassBodyRule {
    indent_config: IndentConfig,
}

impl NoEmptyFirstLineInClassBodyRule {
    pub fn new() -> NoEmptyFirstLineInClassBodyRule {
        NoEmptyFirstLineInClassBodyRule { indent_config: IndentConfig::default_indent_config() }
    }
}

impl Default for NoEmptyFirstLineInClassBodyRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for NoEmptyFirstLineInClassBodyRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-empty-first-line-in-class-body")
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

    fn is_official_code_style(&self) -> bool {
        true
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == CLASS_BODY {
            let lbrace = ast.first_child_node(node).expect("NullPointerException: firstChildNode");
            if let Some(whitespace) = ast.next_leaf(lbrace).filter(|&it| ast.is_white_space_with_newline(it)) {
                let count_newlines = ast.leaf_text(whitespace).matches('\n').count();
                if count_newlines > 1 {
                    emit(ast, ast.start_offset(whitespace) + 1, "Class body should not start with blank line", true).if_autocorrect_allowed(|| {
                        let child_indent = self.indent_config.child_indent_of(ast, node);
                        ast.replace_text_with(whitespace, &child_indent);
                    });
                }
            }
        }
    }
}
