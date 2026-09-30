//! Port of ktlint-ruleset-standard `EnumWrappingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{ANNOTATION_ENTRY, CLASS, CLASS_BODY, ENUM_ENTRY, ENUM_KEYWORD, MODIFIER_LIST, RBRACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines};
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

pub struct EnumWrappingRule {
    indent_config: IndentConfig,
}

impl EnumWrappingRule {
    pub fn new() -> EnumWrappingRule {
        EnumWrappingRule { indent_config: IndentConfig::default_indent_config() }
    }
}

impl Default for EnumWrappingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for EnumWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:enum-wrapping")
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
        if ast.element_type(node) == CLASS
            && ast.has_modifier(node, ENUM_KEYWORD)
            && let Some(class_body) = ast.find_child_by_type(node, CLASS_BODY)
        {
            self.visit_enum_class(ast, class_body, emit);
        }
    }
}

impl EnumWrappingRule {
    fn visit_enum_class(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(node) == CLASS_BODY, "IllegalArgumentException: Failed requirement.");

        let comment_before_first_enum_entry = self.wrap_comment_before_first_enum_entry(ast, node, emit);
        if comment_before_first_enum_entry
            || is_multiline(ast, node)
            || has_annotated_enum_entry(ast, node)
            || has_commented_enum_entry(ast, node)
        {
            self.wrap_enum_entries(ast, node, emit);
            self.wrap_closing_brace(ast, node, emit);
        }
        self.add_blank_line_between_enum_entries_and_other_declarations(ast, node, emit);
    }

    fn wrap_comment_before_first_enum_entry(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) -> bool {
        let first_enum_entry = ast.find_child_by_type(node, ENUM_ENTRY).map(|it| ast.first_child_leaf_or_self(it));
        if let Some(first_enum_entry) = first_enum_entry {
            let comment_before_first_enum_entry = ast
                .leaves_forwards_including_self(ast.first_child_leaf_or_self(node))
                .take_while(|&it| it != first_enum_entry)
                .find(|&it| ast.is_part_of_comment(it));
            if let Some(comment_before_first_enum_entry) = comment_before_first_enum_entry {
                let expected_indent = self.indent_config.child_indent_of(ast, node);
                if ast.prev_leaf(comment_before_first_enum_entry).map(|it| ast.text(it)).as_deref() != Some(expected_indent.as_str()) {
                    emit(ast, ast.start_offset(node), "Expected a (single) newline before comment", true).if_autocorrect_allowed(|| {
                        let indent = self.indent_config.sibling_indent_of(ast, node);
                        ast.upsert_whitespace_before_me(comment_before_first_enum_entry, &indent);
                    });
                    return true;
                }
            }
        }
        false
    }

    fn wrap_enum_entries(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let enum_entries: Vec<NodeId> = ast.children(node).filter(|&it| ast.element_type(it) == ENUM_ENTRY).collect();
        for enum_entry in enum_entries {
            self.wrap_enum_entry(ast, enum_entry, emit);
        }
    }

    fn wrap_enum_entry(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let Some(prev_leaf) = ast
            .prev_leaf_matching(node, |it| !ast.is_part_of_comment(it) && !ast.is_white_space_without_newline(it))
            .filter(|&it| !ast.is_white_space_with_newline(it))
        else {
            return;
        };
        emit(ast, ast.start_offset(node), "Enum entry should start on a separate line", true).if_autocorrect_allowed(|| {
            let indent = self.indent_config.sibling_indent_of(ast, node);
            ast.upsert_whitespace_after_me(prev_leaf, &indent);
        });
    }

    fn wrap_closing_brace(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let Some(rbrace) = ast.find_child_by_type(node, RBRACE) else { return };
        let prev_leaf = ast.prev_leaf(rbrace);
        let expected_indent = self.indent_config.parent_indent_of(ast, node);
        if prev_leaf.map(|it| ast.text(it)).as_deref() != Some(expected_indent.as_str()) {
            emit(ast, ast.start_offset(rbrace), "Expected newline before '}'", true)
                .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(rbrace, &expected_indent));
        }
    }

    fn add_blank_line_between_enum_entries_and_other_declarations(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let Some(next_sibling) = ast
            .children(node)
            .filter(|&it| ast.element_type(it) == ENUM_ENTRY)
            .last()
            .and_then(|it| ast.next_sibling_matching(it, |s| !ast.is_part_of_comment(s)))
            .filter(|&it| ast.next_code_sibling(it).map(|s| ast.element_type(s)) != Some(RBRACE))
        else {
            return;
        };
        let expected_indent = format!("\n{}", self.indent_config.sibling_indent_of(ast, node));
        if ast.text(next_sibling) != expected_indent {
            emit(ast, ast.start_offset(next_sibling) + 1, "Expected blank line between enum entries and other declaration(s)", true)
                .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(next_sibling, &expected_indent));
        }
    }
}

fn is_multiline(ast: &Ast, n: NodeId) -> bool {
    ast.text_contains(n, '\n')
}

fn has_annotated_enum_entry(ast: &Ast, n: NodeId) -> bool {
    ast.children(n).filter(|&it| ast.element_type(it) == ENUM_ENTRY).any(|it| is_annotated(ast, it))
}

fn is_annotated(ast: &Ast, n: NodeId) -> bool {
    ast.find_child_by_type(n, MODIFIER_LIST)
        .is_some_and(|list| ast.children(list).any(|it| ast.element_type(it) == ANNOTATION_ENTRY))
}

fn has_commented_enum_entry(ast: &Ast, n: NodeId) -> bool {
    ast.children(n).any(|it| contains_comment_in_enum_entry(ast, it))
}

fn contains_comment_in_enum_entry(ast: &Ast, n: NodeId) -> bool {
    ast.children(n).any(|it| ast.is_part_of_comment(it))
}
