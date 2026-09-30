//! Port of ktlint-rule-engine `internal/FormatterTags.kt`: IntelliJ's `@formatter:off`/`on` comments.

use std::sync::LazyLock;

use ktrs_editorconfig::PropertyType;
use ktrs_editorconfig::property_type::{boolean_value_parser, identity_value_parser};

use crate::editorconfig::{EditorConfig, EditorConfigProperty};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatterTags {
    pub formatter_tag_off: Option<String>,
    pub formatter_tag_on: Option<String>,
}

impl FormatterTags {
    pub fn from(editor_config: &EditorConfig) -> FormatterTags {
        if editor_config.get(&FORMATTER_TAGS_ENABLED_PROPERTY) {
            FormatterTags {
                formatter_tag_off: Some(editor_config.get(&FORMATTER_TAG_OFF_ENABLED_PROPERTY)),
                formatter_tag_on: Some(editor_config.get(&FORMATTER_TAG_ON_ENABLED_PROPERTY)),
            }
        } else {
            FormatterTags {
                formatter_tag_off: None,
                formatter_tag_on: None,
            }
        }
    }
}

static FORMATTER_TAGS_ENABLED_TYPE: PropertyType<bool> = PropertyType {
    name: "ij_formatter_tags_enabled",
    description: "When enabled, IntelliJ IDEA Formatter tags will be respected (e.g. disable and enable all ktlint rules for the code enclosed between the formatter tags.",
    parser: boolean_value_parser,
    possible_values: &["true", "false"],
    lower_casing: true,
};

static FORMATTER_TAG_OFF_TYPE: PropertyType<String> = PropertyType {
    name: "ij_formatter_off_tag",
    description: "The IntelliJ IDEA formatter tag to disable formatting. This also disables the ktlint rules.",
    parser: identity_value_parser,
    possible_values: &[],
    lower_casing: true,
};

static FORMATTER_TAG_ON_TYPE: PropertyType<String> = PropertyType {
    name: "ij_formatter_on_tag",
    description: "The IntelliJ IDEA formatter tag to enable formatting. This also enables the ktlint rules.",
    parser: identity_value_parser,
    possible_values: &[],
    lower_casing: true,
};

pub static FORMATTER_TAGS_ENABLED_PROPERTY: LazyLock<EditorConfigProperty<bool>> =
    LazyLock::new(|| EditorConfigProperty::new(&FORMATTER_TAGS_ENABLED_TYPE, false));

pub static FORMATTER_TAG_OFF_ENABLED_PROPERTY: LazyLock<EditorConfigProperty<String>> =
    LazyLock::new(|| {
        EditorConfigProperty::new(&FORMATTER_TAG_OFF_TYPE, "@formatter:off".to_owned())
    });

pub static FORMATTER_TAG_ON_ENABLED_PROPERTY: LazyLock<EditorConfigProperty<String>> =
    LazyLock::new(|| EditorConfigProperty::new(&FORMATTER_TAG_ON_TYPE, "@formatter:on".to_owned()));
