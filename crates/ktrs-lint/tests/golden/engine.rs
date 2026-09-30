//! The only glue between a golden case's `.options` and the ktrs-lint API (rule registry, editorconfig
//! override), so engine API changes touch one file.

use std::path::Path;

use ktrs_lint::editorconfig::{
    CODE_STYLE_PROPERTY, END_OF_LINE_PROPERTY, EXPERIMENTAL_RULES_EXECUTION_PROPERTY, INDENT_SIZE_PROPERTY,
    INDENT_STYLE_PROPERTY, INSERT_FINAL_NEWLINE_PROPERTY, MAX_LINE_LENGTH_PROPERTY, PropertyRef, RuleExecution,
    create_rule_execution_editor_config_property, create_rule_set_execution_editor_config_property,
};
use ktrs_lint::rules::standard_rule_provider;
use ktrs_lint::{Code, EditorConfigDefaults, EditorConfigOverride, KtLintRuleEngine, RuleV2Provider};

use crate::case::Options;

pub enum Skip {
    /// A rule of the case is not registered in ktrs-lint.
    Rule(String),
    /// No ported rule or engine property has this `.editorconfig` name.
    EditorConfig(String),
}

/// The property named `name`, as the JVM harness passed it (`EditorConfigProperty.name`).
fn property(name: &str, value: &str, providers: &[RuleV2Provider]) -> Option<PropertyRef> {
    let statics: [PropertyRef; 7] = [
        (&*CODE_STYLE_PROPERTY).into(),
        (&*END_OF_LINE_PROPERTY).into(),
        (&*EXPERIMENTAL_RULES_EXECUTION_PROPERTY).into(),
        (&*INDENT_SIZE_PROPERTY).into(),
        (&*INDENT_STYLE_PROPERTY).into(),
        (&*INSERT_FINAL_NEWLINE_PROPERTY).into(),
        (&*MAX_LINE_LENGTH_PROPERTY).into(),
    ];
    let used = providers.iter().flat_map(|p| p.uses_editor_config_properties().iter().cloned());
    if let Some(p) = statics.into_iter().chain(used).find(|p| p.name() == name) {
        return Some(p);
    }
    // `ktlint_<set>` / `ktlint_<set>_<rule>`: rule set ids have no '_', rule ids no ':'. Rule properties also start
    // with `ktlint_`, but never take a RuleExecution value.
    let rest = name.strip_prefix("ktlint_").filter(|_| matches!(value, "enabled" | "disabled"))?;
    Some(match rest.split_once('_') {
        None => create_rule_set_execution_editor_config_property(rest, RuleExecution::Enabled).into(),
        Some((set, rule)) => create_rule_execution_editor_config_property(&format!("{set}:{rule}"), RuleExecution::Enabled).into(),
    })
}

pub fn setup(options: &Options, input: &str) -> Result<(KtLintRuleEngine, Code), Skip> {
    let providers = options
        .rules
        .iter()
        .map(|id| standard_rule_provider(id).ok_or_else(|| Skip::Rule(id.clone())))
        .collect::<Result<Vec<_>, _>>()?;
    let overrides = options
        .editor_config
        .iter()
        .map(|(name, value)| property(name, value, &providers).map(|p| (p, Some(value.clone()))).ok_or_else(|| Skip::EditorConfig(name.clone())))
        .collect::<Result<Vec<_>, _>>()?;
    let editor_config_override = if overrides.is_empty() { EditorConfigOverride::empty() } else { EditorConfigOverride::from(overrides) };
    let code = match &options.path {
        Some(path) => Code::from_file_content(Path::new(path), input.to_owned()),
        None => Code::from_snippet(input, options.script),
    };
    Ok((KtLintRuleEngine::with_editor_config(providers, EditorConfigDefaults::empty(), editor_config_override), code))
}
