//! Port of ktlint-rule-engine-core `editorconfig/CodeStyleEditorConfigProperty.kt`.

use std::sync::LazyLock;

use ktrs_editorconfig::{EnumValue, PropertyType};

use crate::editorconfig::editor_config_property::EditorConfigProperty;
use crate::editorconfig::value_parsers::safe_enum_value_parser;

/// `CodeStyleValue` (`android_studio`, `intellij_idea`, `ktlint_official`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CodeStyleValue {
    AndroidStudio,
    IntellijIdea,
    KtlintOfficial,
}

impl EnumValue for CodeStyleValue {
    const ENUM_TYPE_NAME: &'static str =
        "io.github.ktlint.core.rule.engine.core.api.editorconfig.CodeStyleValue";
    const ENTRIES: &'static [Self] = &[
        CodeStyleValue::AndroidStudio,
        CodeStyleValue::IntellijIdea,
        CodeStyleValue::KtlintOfficial,
    ];

    fn name(self) -> &'static str {
        match self {
            CodeStyleValue::AndroidStudio => "android_studio",
            CodeStyleValue::IntellijIdea => "intellij_idea",
            CodeStyleValue::KtlintOfficial => "ktlint_official",
        }
    }
}

crate::enum_property_value_type!(CodeStyleValue);

pub static CODE_STYLE_PROPERTY_TYPE: PropertyType<CodeStyleValue> = PropertyType {
    name: "ktlint_code_style",
    description: "The code style ('ktlint_official', 'intellij_idea' or 'android_studio') to be applied. By default the 'ktlint_official' code style is used",
    parser: safe_enum_value_parser::<CodeStyleValue>,
    possible_values: &["android_studio", "intellij_idea", "ktlint_official"],
    lower_casing: true,
};

pub static CODE_STYLE_PROPERTY: LazyLock<EditorConfigProperty<CodeStyleValue>> =
    LazyLock::new(|| EditorConfigProperty {
        android_studio_code_style_default_value: CodeStyleValue::AndroidStudio,
        intellij_idea_code_style_default_value: CodeStyleValue::IntellijIdea,
        ktlint_official_code_style_default_value: CodeStyleValue::KtlintOfficial,
        ..EditorConfigProperty::new(&CODE_STYLE_PROPERTY_TYPE, CodeStyleValue::KtlintOfficial)
    });
