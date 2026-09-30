//! The top-level helpers of ktlint-ruleset-standard `MaxLineLengthRule.kt`, used by other rules.
// TODO: port `MaxLineLengthRule` itself here.

use crate::editorconfig::{MAX_LINE_LENGTH_PROPERTY, RULE_EXECUTION_PROPERTY_TYPE, RuleExecution, rule_execution_property_name};
use crate::rule::EditorConfig;

const MAX_LINE_LENGTH_RULE_ID: &str = "standard:max-line-length";

/// `EditorConfig.maxLineLength()`: `max_line_length`, or `Int.MAX_VALUE` unless `max-line-length` runs.
pub fn max_line_length(editor_config: &EditorConfig) -> i32 {
    if max_line_length_rule_enabled(editor_config) { editor_config.get(&MAX_LINE_LENGTH_PROPERTY) } else { i32::MAX }
}

fn max_line_length_rule_enabled(editor_config: &EditorConfig) -> bool {
    Some(RuleExecution::Enabled)
        == editor_config.get_editor_config_value_or_null(
            &RULE_EXECUTION_PROPERTY_TYPE,
            &rule_execution_property_name(MAX_LINE_LENGTH_RULE_ID),
        )
}
