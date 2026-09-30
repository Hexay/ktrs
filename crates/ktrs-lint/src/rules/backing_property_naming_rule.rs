//! Port of ktlint-ruleset-standard `BackingPropertyNamingRule.kt`
//! (https://kotlinlang.org/docs/coding-conventions.html#property-names, https://developer.android.com/kotlin/style-guide#backing_properties).

use std::sync::LazyLock;

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    CLASS_BODY, COMPANION_KEYWORD, FUN, IDENTIFIER, INTERNAL_KEYWORD, MODIFIER_LIST, OBJECT_DECLARATION, OVERRIDE_KEYWORD,
    PRIVATE_KEYWORD, PROPERTY, PROTECTED_KEYWORD, VALUE_PARAMETER, VALUE_PARAMETER_LIST,
};

use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{CODE_STYLE_PROPERTY, CodeStyleValue, PropertyRef};
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::internal::kotlin_string::replace_first_char_uppercase;
use crate::rules::internal::{KotlinRegex, reg_ex_ignoring_diacritics_and_strokes_on_letters};

pub struct BackingPropertyNamingRule {
    code_style: CodeStyleValue,
}

impl BackingPropertyNamingRule {
    pub fn new() -> BackingPropertyNamingRule {
        BackingPropertyNamingRule { code_style: CODE_STYLE_PROPERTY.default_value }
    }
}

impl Default for BackingPropertyNamingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for BackingPropertyNamingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:backing-property-naming")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*CODE_STYLE_PROPERTY)]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.code_style = editor_config.get(&CODE_STYLE_PROPERTY);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == PROPERTY {
            self.visit_property(ast, node, emit);
        }
    }
}

impl BackingPropertyNamingRule {
    fn visit_property(&self, ast: &Ast, property: NodeId, emit: &mut Emit<'_>) {
        let identifier = ast
            .find_child_by_type(property, IDENTIFIER)
            .filter(|&it| ast.leaf_text(it).starts_with('_'))
            // A local property named "_" expresses that the value is not used (KEEP-0412).
            .filter(|&it| ast.text_length_utf16(it) != 1)
            // Overridden properties can only be changed by changing the base property.
            .filter(|&it| !ast.has_modifier(parent(ast, it), OVERRIDE_KEYWORD));
        if let Some(identifier) = identifier {
            self.visit_backing_property(ast, identifier, emit);
        }
    }

    fn visit_backing_property(&self, ast: &Ast, identifier: NodeId, emit: &mut Emit<'_>) {
        if !BACKING_PROPERTY_LOWER_CAMEL_CASE_REGEXP.matches(&ast.text(identifier)) {
            emit(ast, ast.start_offset(identifier), "Backing property should start with underscore followed by lower camel case", false);
        }

        if !has_private_modifier_in_property_declaration(ast, identifier) && !is_declared_in_private_companion_object(ast, identifier) {
            emit(ast, ast.start_offset(identifier), "Backing property not allowed when 'private' modifier is missing", false);
        }

        // A backing property can only exist when a correlated public property or function exists
        match find_correlated_property_or_function(ast, identifier) {
            None => {
                emit(ast, ast.start_offset(identifier), "Backing property is only allowed when a matching property or function exists", false);
            }
            Some(correlated_property_or_function) => {
                if self.code_style == CodeStyleValue::AndroidStudio || is_public(ast, correlated_property_or_function) {
                    return;
                }
                emit(
                    ast,
                    ast.start_offset(identifier),
                    "Backing property is only allowed when the matching property or function is public",
                    false,
                );
            }
        }
    }
}

/// `parent!!`.
fn parent(ast: &Ast, node: NodeId) -> NodeId {
    ast.parent(node).expect("NullPointerException: parent")
}

fn has_private_modifier_in_property_declaration(ast: &Ast, node: NodeId) -> bool {
    ast.has_modifier(parent(ast, node), PRIVATE_KEYWORD)
}

fn is_declared_in_private_companion_object(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .and_then(|it| ast.parent(it))
        .and_then(|it| find_companion_object(ast, it))
        .and_then(|it| ast.parent(it))
        .and_then(|it| ast.parent(it))
        .is_some_and(|it| ast.has_modifier(it, PRIVATE_KEYWORD))
}

fn find_correlated_property_or_function(ast: &Ast, node: NodeId) -> Option<NodeId> {
    find_correlated_property(ast, node).or_else(|| find_correlated_function(ast, node))
}

/// In the same class body or, for the body of a companion object, in the class body the companion is defined in.
fn find_correlated_property(ast: &Ast, node: NodeId) -> Option<NodeId> {
    let text = ast.text(node);
    let property_name = text.strip_prefix('_').unwrap_or(&text);
    ast.find_parent_by_type(node, CLASS_BODY).and_then(|class_body| {
        find_property_with_name(ast, class_body, property_name).or_else(|| {
            find_companion_object(ast, class_body)
                .and_then(|it| ast.find_parent_by_type(it, CLASS_BODY))
                .and_then(|it| find_property_with_name(ast, it, property_name))
        })
    })
}

fn find_property_with_name(ast: &Ast, node: NodeId, name: &str) -> Option<NodeId> {
    ast.children(node)
        .filter(|&it| ast.element_type(it) == PROPERTY)
        .filter_map(|it| ast.find_child_by_type(it, IDENTIFIER))
        .find(|&it| ast.leaf_text(it) == name)
        .and_then(|it| ast.parent(it))
}

fn find_companion_object(ast: &Ast, node: NodeId) -> Option<NodeId> {
    ast.parent(node)
        .filter(|&it| ast.element_type(it) == OBJECT_DECLARATION)
        .and_then(|it| ast.find_child_by_type(it, MODIFIER_LIST))
        .and_then(|it| ast.children(it).find(|&it| ast.element_type(it) == COMPANION_KEYWORD))
}

/// Like [`find_correlated_property`], for a getter function.
fn find_correlated_function(ast: &Ast, node: NodeId) -> Option<NodeId> {
    let correlated_function_name = format!("get{}", capitalize_first_char(ast, node));
    ast.find_parent_by_type(node, CLASS_BODY).and_then(|class_body| {
        find_function_with_name(ast, class_body, &correlated_function_name).or_else(|| {
            find_companion_object(ast, class_body)
                .and_then(|it| ast.find_parent_by_type(it, CLASS_BODY))
                .and_then(|it| find_function_with_name(ast, it, &correlated_function_name))
        })
    })
}

fn find_function_with_name(ast: &Ast, node: NodeId, name: &str) -> Option<NodeId> {
    ast.children(node)
        .filter(|&it| ast.element_type(it) == FUN)
        .filter(|&it| has_non_empty_parameter_list(ast, it))
        .filter_map(|it| ast.find_child_by_type(it, IDENTIFIER))
        .find(|&it| ast.leaf_text(it) == name)
        .and_then(|it| ast.parent(it))
}

// Gotcha: despite its name, true when the list has no parameters.
fn has_non_empty_parameter_list(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, VALUE_PARAMETER_LIST)
        .is_some_and(|it| !ast.children(it).any(|it| ast.element_type(it) == VALUE_PARAMETER))
}

fn capitalize_first_char(ast: &Ast, node: NodeId) -> String {
    let text = ast.text(node);
    replace_first_char_uppercase(text.strip_prefix('_').unwrap_or(&text))
}

fn is_public(ast: &Ast, node: NodeId) -> bool {
    !ast.has_modifier(node, PRIVATE_KEYWORD) && !ast.has_modifier(node, PROTECTED_KEYWORD) && !ast.has_modifier(node, INTERNAL_KEYWORD)
}

static BACKING_PROPERTY_LOWER_CAMEL_CASE_REGEXP: LazyLock<KotlinRegex> =
    LazyLock::new(|| reg_ex_ignoring_diacritics_and_strokes_on_letters("_[a-z][a-zA-Z0-9]*"));
