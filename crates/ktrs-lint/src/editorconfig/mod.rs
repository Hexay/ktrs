//! Port of ktlint-rule-engine-core `api/editorconfig/`: typed `.editorconfig` properties over the ec4j
//! model ([`ktrs_editorconfig`]). A rule declares the properties it reads (`uses_editor_config_properties`)
//! and gets an [`EditorConfig`] holding exactly those (plus the code style) in `before_first_node`.

mod code_style;
mod editor_config;
mod editor_config_property;
mod ktlint_version;
mod rule_execution;
mod standard_properties;
mod value_parsers;

pub use code_style::{CODE_STYLE_PROPERTY, CODE_STYLE_PROPERTY_TYPE, CodeStyleValue};
pub use editor_config::EditorConfig;
pub use editor_config_property::{
    AnyEditorConfigProperty, EditorConfigProperty, PropertyRef, PropertyValueType,
    to_property_with_parsed_value, to_property_with_value,
};
pub use ktlint_version::{KTLINT_VERSION_PROPERTY, KTLINT_VERSION_PROPERTY_TYPE, KtlintVersion, with_ktlint_version};
pub use ktrs_editorconfig::{EndOfLineValue, IndentStyleValue};
pub use rule_execution::{
    ALL_RULES_EXECUTION_PROPERTY, EXPERIMENTAL_RULES_EXECUTION_PROPERTY,
    RULE_EXECUTION_PROPERTY_TYPE, RuleExecution, create_rule_execution_editor_config_property,
    create_rule_set_execution_editor_config_property, rule_execution_property_name,
    rule_set_execution_property_name,
};
pub use standard_properties::{
    END_OF_LINE_PROPERTY, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY,
    INSERT_FINAL_NEWLINE_PROPERTY, MAX_LINE_LENGTH_PROPERTY, MAX_LINE_LENGTH_PROPERTY_OFF,
};
pub use value_parsers::{comma_separated_list_value_parser, safe_enum_value_parser};
