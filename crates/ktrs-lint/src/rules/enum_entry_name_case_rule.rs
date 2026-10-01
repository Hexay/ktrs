//! Port of ktlint-ruleset-standard `EnumEntryNameCaseRule.kt` (https://kotlinlang.org/docs/coding-conventions.html#property-names).

use std::sync::LazyLock;

use ktrs_ast::psi::unquote_identifier;
use ktrs_ast::{Ast, NodeId};
use ktrs_editorconfig::{EnumValue, PropertyType};
use ktrs_syntax::SyntaxKind::{ENUM_ENTRY, IDENTIFIER};

use crate::editorconfig::{EditorConfigProperty, PropertyRef, safe_enum_value_parser};
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::internal::{KotlinRegex, reg_ex_ignoring_diacritics_and_strokes_on_letters};

#[derive(Default)]
pub struct EnumEntryNameCaseRule {
    /// `lateinit`: set in `before_first_node`.
    enum_entry_casing: Option<(&'static KotlinRegex, &'static str)>,
}

const VISITED_TYPES: TokenSet = TokenSet::create(&[ENUM_ENTRY]);

impl RuleV2 for EnumEntryNameCaseRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:enum-entry-name-case")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*ENUM_ENTRY_NAME_CASING_PROPERTY)]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        static UPPER_CASES: LazyLock<KotlinRegex> = LazyLock::new(|| reg_ex_ignoring_diacritics_and_strokes_on_letters("[A-Z][A-Z_\\d]*"));
        static CAMEL_CASES: LazyLock<KotlinRegex> =
            LazyLock::new(|| reg_ex_ignoring_diacritics_and_strokes_on_letters("[A-Z]([A-Za-z\\d]*)"));
        static UPPER_OR_CAMEL_CASES: LazyLock<KotlinRegex> =
            LazyLock::new(|| reg_ex_ignoring_diacritics_and_strokes_on_letters("[A-Z]([A-Za-z\\d]*|[A-Z_\\d]*)"));
        self.enum_entry_casing = Some(match editor_config.get(&ENUM_ENTRY_NAME_CASING_PROPERTY) {
            EnumEntryNameCasing::UpperCases => {
                (&*UPPER_CASES, "Enum entry name should be uppercase underscore-separated names like \"ENUM_ENTRY\"")
            }
            EnumEntryNameCasing::CamelCases => (&*CAMEL_CASES, "Enum entry name should be upper camel-case like \"EnumEntry\""),
            EnumEntryNameCasing::UpperOrCamelCases => (
                &*UPPER_OR_CAMEL_CASES,
                "Enum entry name should be uppercase underscore-separated names like \"ENUM_ENTRY\" or upper camel-case like \"EnumEntry\"",
            ),
        });
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.is_leaf_element(node) {
            return;
        }
        if ast.element_type(node) != ENUM_ENTRY {
            return;
        }
        let Some(name_node) = ast.find_child_by_type(node, IDENTIFIER) else { return };
        let name = unquote_identifier(&ast.text(name_node));

        let (enum_entry_casing_regex, enum_entry_casing_violation) =
            self.enum_entry_casing.expect("UninitializedPropertyAccessException: lateinit property enumEntryCasingRegex");
        if !enum_entry_casing_regex.matches(&name) {
            emit(ast, ast.start_offset(node), enum_entry_casing_violation, false);
        }
    }
}

/// `EnumEntryNameCasing`: digits, diacritics and strokes are always allowed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnumEntryNameCasing {
    /// Uppercase underscore-separated names like "ENUM_ENTRY".
    UpperCases,
    /// Upper camel-case like "EnumEntry".
    CamelCases,
    /// Either of the above.
    UpperOrCamelCases,
}

impl EnumValue for EnumEntryNameCasing {
    const ENUM_TYPE_NAME: &'static str =
        "io.github.ktlint.core.ruleset.standard.rules.EnumEntryNameCaseRule$Companion$EnumEntryNameCasing";
    const ENTRIES: &'static [Self] = &[EnumEntryNameCasing::UpperCases, EnumEntryNameCasing::CamelCases, EnumEntryNameCasing::UpperOrCamelCases];

    fn name(self) -> &'static str {
        match self {
            EnumEntryNameCasing::UpperCases => "upper_cases",
            EnumEntryNameCasing::CamelCases => "camel_cases",
            EnumEntryNameCasing::UpperOrCamelCases => "upper_or_camel_cases",
        }
    }
}

crate::enum_property_value_type!(EnumEntryNameCasing);

pub static ENUM_ENTRY_NAME_CASING_PROPERTY_TYPE: PropertyType<EnumEntryNameCasing> = PropertyType {
    name: "ktlint_enum_entry_name_casing",
    description: "Enforce all enum entry names to be uppercase underscore-separated names like \"ENUM_ENTRY\" and/or upper \
                  camel-case like \"EnumEntry\". Digits, diacritics and strokes are always allowed.",
    parser: safe_enum_value_parser::<EnumEntryNameCasing>,
    possible_values: &["upper_cases", "camel_cases", "upper_or_camel_cases"],
    lower_casing: true,
};

pub static ENUM_ENTRY_NAME_CASING_PROPERTY: LazyLock<EditorConfigProperty<EnumEntryNameCasing>> =
    LazyLock::new(|| EditorConfigProperty::new(&ENUM_ENTRY_NAME_CASING_PROPERTY_TYPE, EnumEntryNameCasing::UpperOrCamelCases));
