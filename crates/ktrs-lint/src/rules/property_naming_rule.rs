//! Port of ktlint-ruleset-standard `PropertyNamingRule.kt` (https://kotlinlang.org/docs/coding-conventions.html#property-names).

use std::sync::LazyLock;

use ktrs_ast::{Ast, NodeId};
use ktrs_editorconfig::{EnumValue, PropertyType};
use ktrs_syntax::SyntaxKind::{
    CLASS_BODY, CONST_KEYWORD, FILE, GET_KEYWORD, IDENTIFIER, OBJECT_DECLARATION, OVERRIDE_KEYWORD, PROPERTY, PROPERTY_ACCESSOR,
    VAL_KEYWORD,
};

use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{EditorConfigProperty, PropertyRef, safe_enum_value_parser};
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::internal::kotlin_string::remove_surrounding;
use crate::rules::internal::{KotlinRegex, is_keyword, reg_ex_ignoring_diacritics_and_strokes_on_letters};

pub struct PropertyNamingRule {
    constant_naming_property: ConstantNamingStyle,
}

impl PropertyNamingRule {
    pub fn new() -> PropertyNamingRule {
        PropertyNamingRule { constant_naming_property: CONSTANT_NAMING_PROPERTY.default_value }
    }
}

impl Default for PropertyNamingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for PropertyNamingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:property-naming")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*CONSTANT_NAMING_PROPERTY)]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.constant_naming_property = editor_config.get(&CONSTANT_NAMING_PROPERTY);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == PROPERTY {
            self.visit_property(ast, node, emit);
        }
    }
}

impl PropertyNamingRule {
    fn visit_property(&self, ast: &Ast, property: NodeId, emit: &mut Emit<'_>) {
        let Some(identifier) = ast.find_child_by_type(property, IDENTIFIER).filter(|&it| !is_token_keyword_between_backticks(ast, it))
        else {
            return;
        };
        if has_const_modifier(ast, property) {
            self.visit_const_property(ast, identifier, emit);
        } else if has_custom_getter(ast, property) || is_top_level_value(ast, property) || is_object_value(ast, property) {
            // Can not reliably determine whether the value is immutable or not
        } else {
            visit_non_const_property(ast, identifier, emit);
        }
    }

    fn visit_const_property(&self, ast: &Ast, identifier: NodeId, emit: &mut Emit<'_>) {
        let text = ast.leaf_text(identifier);
        // Allow `private const val serialVersionUID: Long = 123` in an object
        if text == SERIAL_VERSION_UID_PROPERTY_NAME || self.constant_naming_property.reg_ex().matches(text) {
            return;
        }
        let expected_naming = self.constant_naming_property.name().replace('_', " ");
        emit(
            ast,
            ast.start_offset(identifier),
            &format!("Property name should use the {expected_naming} notation when the value can not be changed"),
            false,
        );
    }
}

fn visit_non_const_property(ast: &Ast, identifier: NodeId, emit: &mut Emit<'_>) {
    let text = ast.leaf_text(identifier);
    // Ignore backing properties
    if LOWER_CAMEL_CASE_REGEXP.matches(text) || text.starts_with('_') {
        return;
    }
    emit(ast, ast.start_offset(identifier), "Property name should start with a lowercase letter and use camel case", false);
}

fn has_custom_getter(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, PROPERTY_ACCESSOR).and_then(|it| ast.find_child_by_type(it, GET_KEYWORD)).is_some()
}

fn has_const_modifier(ast: &Ast, node: NodeId) -> bool {
    ast.has_modifier(node, CONST_KEYWORD)
}

fn is_top_level_value(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node).map(|it| ast.element_type(it)) == Some(FILE) && contains_val_keyword(ast, node)
}

fn contains_val_keyword(ast: &Ast, node: NodeId) -> bool {
    ast.children(node).any(|it| ast.element_type(it) == VAL_KEYWORD)
}

fn is_object_value(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node).map(|it| ast.element_type(it)) == Some(CLASS_BODY)
        && ast.parent(node).and_then(|it| ast.parent(it)).map(|it| ast.element_type(it)) == Some(OBJECT_DECLARATION)
        && contains_val_keyword(ast, node)
        && !ast.has_modifier(node, OVERRIDE_KEYWORD)
}

fn is_token_keyword_between_backticks(ast: &Ast, node: NodeId) -> bool {
    let text = if ast.element_type(node) == IDENTIFIER { ast.leaf_text(node) } else { "" };
    is_keyword(remove_surrounding(text, "`", "`"))
}

static LOWER_CAMEL_CASE_REGEXP: LazyLock<KotlinRegex> =
    LazyLock::new(|| reg_ex_ignoring_diacritics_and_strokes_on_letters("[a-z][a-zA-Z0-9]*"));
const SERIAL_VERSION_UID_PROPERTY_NAME: &str = "serialVersionUID";

/// `ConstantNamingStyle`; the latin letters may be combined with strokes and diacritics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstantNamingStyle {
    /// Uppercase, digits and underscores separating the words.
    ScreamingSnakeCase,
    /// Each word starts with an uppercase letter; no underscores.
    PascalCase,
}

impl ConstantNamingStyle {
    fn reg_ex(self) -> &'static KotlinRegex {
        static SCREAMING_SNAKE_CASE: LazyLock<KotlinRegex> =
            LazyLock::new(|| reg_ex_ignoring_diacritics_and_strokes_on_letters("[A-Z][_A-Z0-9]*"));
        static PASCAL_CASE: LazyLock<KotlinRegex> =
            LazyLock::new(|| reg_ex_ignoring_diacritics_and_strokes_on_letters("[A-Z][a-zA-Z0-9]*"));
        match self {
            ConstantNamingStyle::ScreamingSnakeCase => &SCREAMING_SNAKE_CASE,
            ConstantNamingStyle::PascalCase => &PASCAL_CASE,
        }
    }
}

impl EnumValue for ConstantNamingStyle {
    const ENUM_TYPE_NAME: &'static str = "io.github.ktlint.core.ruleset.standard.rules.PropertyNamingRule$Companion$ConstantNamingStyle";
    const ENTRIES: &'static [Self] = &[ConstantNamingStyle::ScreamingSnakeCase, ConstantNamingStyle::PascalCase];

    fn name(self) -> &'static str {
        match self {
            ConstantNamingStyle::ScreamingSnakeCase => "screaming_snake_case",
            ConstantNamingStyle::PascalCase => "pascal_case",
        }
    }
}

crate::enum_property_value_type!(ConstantNamingStyle);

pub static CONSTANT_NAMING_PROPERTY_TYPE: PropertyType<ConstantNamingStyle> = PropertyType {
    name: "ktlint_property_naming_constant_naming",
    description: "The naming style ('screaming_snake_case', or 'pascal_case') to be applied on constant properties. All code styles use \
                  'screaming_snake_case' code as default.",
    parser: safe_enum_value_parser::<ConstantNamingStyle>,
    possible_values: &["screaming_snake_case", "pascal_case"],
    lower_casing: true,
};

pub static CONSTANT_NAMING_PROPERTY: LazyLock<EditorConfigProperty<ConstantNamingStyle>> =
    LazyLock::new(|| EditorConfigProperty::new(&CONSTANT_NAMING_PROPERTY_TYPE, ConstantNamingStyle::ScreamingSnakeCase));
