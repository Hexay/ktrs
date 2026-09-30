//! Port of ktlint-rule-engine-core `editorconfig/RuleExecutionEditorConfigProperty.kt`.

use std::borrow::Cow;
use std::sync::LazyLock;

use ktrs_editorconfig::{EnumValue, PropertyType};

use crate::editorconfig::editor_config_property::EditorConfigProperty;
use crate::editorconfig::value_parsers::safe_enum_value_parser;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RuleExecution {
    Enabled,
    Disabled,
}

impl EnumValue for RuleExecution {
    const ENUM_TYPE_NAME: &'static str =
        "io.github.ktlint.core.rule.engine.core.api.editorconfig.RuleExecution";
    const ENTRIES: &'static [Self] = &[RuleExecution::Enabled, RuleExecution::Disabled];

    fn name(self) -> &'static str {
        match self {
            RuleExecution::Enabled => "enabled",
            RuleExecution::Disabled => "disabled",
        }
    }
}

crate::enum_property_value_type!(RuleExecution);

pub static RULE_EXECUTION_PROPERTY_TYPE: PropertyType<RuleExecution> = PropertyType {
    name: "ktlint_rule_execution",
    description: "When enabled, rule execution is allowed. This property can de defined at different levels like an entire ruleset, a specific rule or a specific property of the rule.",
    parser: safe_enum_value_parser::<RuleExecution>,
    possible_values: &["enabled", "disabled"],
    lower_casing: true,
};

/// When disabled, no ktlint rules are executed, internal rules included.
pub static ALL_RULES_EXECUTION_PROPERTY: LazyLock<EditorConfigProperty<RuleExecution>> =
    LazyLock::new(|| named_rule_execution_property("ktlint".into(), RuleExecution::Enabled));

/// When enabled, a rule marked `Experimental` runs unless it is disabled itself.
pub static EXPERIMENTAL_RULES_EXECUTION_PROPERTY: LazyLock<EditorConfigProperty<RuleExecution>> =
    LazyLock::new(|| {
        named_rule_execution_property("ktlint_experimental".into(), RuleExecution::Disabled)
    });

fn named_rule_execution_property(
    name: Cow<'static, str>,
    default_value: RuleExecution,
) -> EditorConfigProperty<RuleExecution> {
    EditorConfigProperty {
        name,
        ..EditorConfigProperty::new(&RULE_EXECUTION_PROPERTY_TYPE, default_value)
    }
}

/// `RuleSetId.createRuleSetExecutionEditorConfigProperty(ruleExecution)`.
pub fn create_rule_set_execution_editor_config_property(
    rule_set_id: &str,
    rule_execution: RuleExecution,
) -> EditorConfigProperty<RuleExecution> {
    named_rule_execution_property(
        rule_set_execution_property_name(rule_set_id).into(),
        rule_execution,
    )
}

/// `RuleId.createRuleExecutionEditorConfigProperty(ruleExecution)`.
pub fn create_rule_execution_editor_config_property(
    rule_id: &str,
    rule_execution: RuleExecution,
) -> EditorConfigProperty<RuleExecution> {
    named_rule_execution_property(rule_execution_property_name(rule_id).into(), rule_execution)
}

/// `RuleId.ktLintRuleExecutionPropertyName()`: `ktlint_<set>_<rule>`.
pub fn rule_execution_property_name(rule_id: &str) -> String {
    format!("ktlint_{}", rule_id.replacen(':', "_", 1))
}

/// `RuleSetId.ktLintRuleSetExecutionPropertyName()`: `ktlint_<set>`.
pub fn rule_set_execution_property_name(rule_set_id: &str) -> String {
    format!("ktlint_{rule_set_id}")
}
