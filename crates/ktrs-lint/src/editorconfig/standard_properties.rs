//! Ports of ktlint-rule-engine-core `editorconfig/EndOfLineProperty.kt`, `IndentSizeEditorConfigProperty.kt`,
//! `IndentStyleEditorConfigProperty.kt`, `InsertFinalNewLineEditorConfigProperty.kt` and
//! `MaxLineLengthEditorConfigProperty.kt`.

use std::sync::LazyLock;

use ktrs_editorconfig::property_type::{
    END_OF_LINE, INDENT_SIZE, INDENT_STYLE, INSERT_FINAL_NEWLINE, MAX_LINE_LENGTH,
};
use ktrs_editorconfig::{EndOfLineValue, IndentStyleValue, Property};

use crate::editorconfig::code_style::CodeStyleValue;
use crate::editorconfig::editor_config_property::EditorConfigProperty;

pub static END_OF_LINE_PROPERTY: LazyLock<EditorConfigProperty<EndOfLineValue>> =
    LazyLock::new(|| EditorConfigProperty::new(&END_OF_LINE, EndOfLineValue::Lf));

const DEFAULT_INDENT_SIZE: i32 = 4;

pub static INDENT_SIZE_PROPERTY: LazyLock<EditorConfigProperty<i32>> =
    LazyLock::new(|| EditorConfigProperty {
        property_mapper: Some(|property, _| {
            if property.is_some_and(Property::is_unset) {
                Some(-1)
            } else {
                match property.and_then(|p| p.get_value_as(&INDENT_SIZE)) {
                    Some(it) if it > 0 => Some(it),
                    _ => Some(DEFAULT_INDENT_SIZE),
                }
            }
        }),
        ..EditorConfigProperty::new(&INDENT_SIZE, DEFAULT_INDENT_SIZE)
    });

pub static INDENT_STYLE_PROPERTY: LazyLock<EditorConfigProperty<IndentStyleValue>> =
    LazyLock::new(|| EditorConfigProperty::new(&INDENT_STYLE, IndentStyleValue::Space));

pub static INSERT_FINAL_NEWLINE_PROPERTY: LazyLock<EditorConfigProperty<bool>> =
    LazyLock::new(|| EditorConfigProperty::new(&INSERT_FINAL_NEWLINE, true));

// https://developer.android.com/kotlin/style-guide#line_wrapping
const MAX_LINE_LENGTH_PROPERTY_ANDROID_STUDIO_CODE_STYLE: i32 = 100;
const MAX_LINE_LENGTH_PROPERTY_KTLINT_OFFICIAL_CODE_STYLE: i32 = 140;
const MAX_LINE_LENGTH_PROPERTY_OFF_EDITOR_CONFIG: &str = "off";

/// The value that disables the max line length (`Int.MAX_VALUE`, easy to compare against).
pub const MAX_LINE_LENGTH_PROPERTY_OFF: i32 = i32::MAX;

pub static MAX_LINE_LENGTH_PROPERTY: LazyLock<EditorConfigProperty<i32>> =
    LazyLock::new(|| EditorConfigProperty {
        android_studio_code_style_default_value: MAX_LINE_LENGTH_PROPERTY_ANDROID_STUDIO_CODE_STYLE,
        intellij_idea_code_style_default_value: MAX_LINE_LENGTH_PROPERTY_OFF,
        ktlint_official_code_style_default_value:
            MAX_LINE_LENGTH_PROPERTY_KTLINT_OFFICIAL_CODE_STYLE,
        property_mapper: Some(|property, code_style_value| match property {
            None => Some(max_line_length_default_value(code_style_value)),
            Some(p) if p.is_unset() => Some(max_line_length_default_value(code_style_value)),
            Some(p) if p.source_value() == Some(MAX_LINE_LENGTH_PROPERTY_OFF_EDITOR_CONFIG) => {
                Some(MAX_LINE_LENGTH_PROPERTY_OFF)
            }
            Some(p) => {
                let it = MAX_LINE_LENGTH.parse(p.source_value());
                if !it.is_valid() {
                    if it.source() == Some("-1") {
                        Some(MAX_LINE_LENGTH_PROPERTY_OFF)
                    } else {
                        Some(max_line_length_default_value(code_style_value))
                    }
                } else {
                    it.into_parsed()
                }
            }
        }),
        property_writer: |property| {
            if *property <= 0 || *property == MAX_LINE_LENGTH_PROPERTY_OFF {
                MAX_LINE_LENGTH_PROPERTY_OFF_EDITOR_CONFIG.to_owned()
            } else {
                property.to_string()
            }
        },
        ..EditorConfigProperty::new(&MAX_LINE_LENGTH, MAX_LINE_LENGTH_PROPERTY_OFF)
    });

fn max_line_length_default_value(code_style: CodeStyleValue) -> i32 {
    match code_style {
        CodeStyleValue::AndroidStudio => MAX_LINE_LENGTH_PROPERTY_ANDROID_STUDIO_CODE_STYLE,
        CodeStyleValue::IntellijIdea => MAX_LINE_LENGTH_PROPERTY_OFF,
        CodeStyleValue::KtlintOfficial => MAX_LINE_LENGTH_PROPERTY_KTLINT_OFFICIAL_CODE_STYLE,
    }
}
