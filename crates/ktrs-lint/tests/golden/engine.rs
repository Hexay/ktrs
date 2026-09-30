//! The only glue between a golden case's `.options` and the ktrs-lint API: rule registry and editorconfig
//! overrides. Keep it here so engine API changes touch one file.

use ktrs_lint::rules::standard_rule_provider;
use ktrs_lint::{Code, KtLintRuleEngine};

use crate::case::Options;

pub enum Skip {
    /// A rule of the case is not registered in ktrs-lint.
    Rule(String),
    /// The engine can't apply this editorconfig override yet.
    EditorConfig(String),
}

// TODO: pass overrides through once ktrs-lint has EditorConfigOverride; until then only no-op values are accepted.
fn is_default(name: &str, value: &str) -> bool {
    match name {
        "ktlint_code_style" => value == "ktlint_official",
        "indent_size" => value == "4",
        "indent_style" => value == "space",
        "ktlint_experimental" => value == "enabled",
        _ => name.strip_prefix("ktlint_").is_some_and(|set| !set.contains('_')) && value == "enabled",
    }
}

pub fn setup(options: &Options, input: &str) -> Result<(KtLintRuleEngine, Code), Skip> {
    let rule_providers = options
        .rules
        .iter()
        .map(|id| standard_rule_provider(id).ok_or_else(|| Skip::Rule(id.clone())))
        .collect::<Result<Vec<_>, _>>()?;
    if let Some((name, value)) = options.editor_config.iter().find(|(n, v)| !is_default(n, v)) {
        return Err(Skip::EditorConfig(format!("{name}={value}")));
    }
    let code = match &options.path {
        Some(path) => Code::from_file(path, input.to_owned()),
        None => Code::from_snippet(input, options.script),
    };
    Ok((KtLintRuleEngine { rule_providers }, code))
}
