//! Port of ktlint-ruleset-standard `ModifierListSpacingRule.kt` (id `modifier-list-spacing`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{ANNOTATION, ANNOTATION_ENTRY, CONTEXT_PARAMETER_LIST, MODIFIER_LIST};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TokenSet, VisitorModifier};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[MODIFIER_LIST]);

/// Lint and format the spacing between the modifiers in and after the last modifier in a modifier list.
pub struct ModifierListSpacingRule {
    indent_config: IndentConfig,
}

impl ModifierListSpacingRule {
    pub fn new() -> ModifierListSpacingRule {
        ModifierListSpacingRule { indent_config: IndentConfig::default_indent_config() }
    }
}

impl Default for ModifierListSpacingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for ModifierListSpacingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:modifier-list-spacing")
    }

    fn visitor_modifiers(&self) -> &'static [VisitorModifier] {
        const MODIFIERS: &[VisitorModifier] = &[
            VisitorModifier::run_after("standard:annotation"),
            VisitorModifier::run_after("standard:modifier-order"),
        ];
        MODIFIERS
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

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == MODIFIER_LIST {
            // Kotlin's `children.forEach` walks the live sibling chain.
            let mut child = ast.first_child_node(node);
            while let Some(it) = child {
                self.visit_modifier_child(ast, it, emit);
                child = ast.next_sibling(it);
            }
            // The whitespace of the last entry of the modifier list is actually placed outside the modifier list
            self.visit_modifier_child(ast, node, emit);
        }
    }
}

impl ModifierListSpacingRule {
    fn visit_modifier_child(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.is_white_space(node) {
            return;
        }
        let Some(whitespace) = ast
            .next_sibling_matching(node, |it| {
                ast.is_white_space(it) && !ast.next_leaf(it).is_some_and(|l| ast.is_part_of_comment(l))
            })
            // A single newline after a comment is always ok and does not need further checking.
            .filter(|&it| {
                !(ast.leaf_text(it).trim_matches([' ', '\t']).contains('\n')
                    && ast.prev_leaf(it).is_some_and(|l| ast.is_part_of_comment(l)))
            })
        else {
            return;
        };
        let text = ast.leaf_text(whitespace).to_owned();
        if is_annotation(ast, node) {
            if text.contains("\n\n") {
                emit(ast, ast.start_offset(whitespace), "Single newline expected after annotation", true).if_autocorrect_allowed(|| {
                    let after_last_newline = &text[text.rfind('\n').map_or(0, |i| i + 1)..];
                    ast.replace_text_with(whitespace, &format!("\n{after_last_newline}"));
                });
            } else if !text.contains('\n') && text != " " {
                emit(ast, ast.start_offset(whitespace), "Single whitespace or newline expected after annotation", true)
                    .if_autocorrect_allowed(|| ast.replace_text_with(whitespace, " "));
            }
        } else if is_context_receiver_list(ast, node) {
            if !text.contains('\n') {
                emit(ast, ast.start_offset(whitespace), "Single newline expected after context receiver list", true)
                    .if_autocorrect_allowed(|| {
                        let parent_indent = self.indent_config.parent_indent_of(ast, node);
                        ast.replace_text_with(whitespace, &parent_indent);
                    });
            }
        } else if text != " " {
            emit(ast, ast.start_offset(whitespace), "Single whitespace expected after modifier", true)
                .if_autocorrect_allowed(|| ast.replace_text_with(whitespace, " "));
        }
    }
}

fn is_annotation(ast: &Ast, node: NodeId) -> bool {
    is_annotation_element(ast, Some(node))
        || (ast.element_type(node) == MODIFIER_LIST && is_annotation_element(ast, ast.last_child_node(node)))
}

fn is_annotation_element(ast: &Ast, node: Option<NodeId>) -> bool {
    node.is_some_and(|it| matches!(ast.element_type(it), ANNOTATION | ANNOTATION_ENTRY))
}

fn is_context_receiver_list(ast: &Ast, node: NodeId) -> bool {
    ast.element_type(node) == CONTEXT_PARAMETER_LIST
        || (ast.element_type(node) == MODIFIER_LIST && ast.last_child_node(node).is_some_and(|it| is_context_receiver_list(ast, it)))
}
