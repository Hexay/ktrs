//! `detekt-core/.../RuleDescriptor.kt`.

use std::sync::{Arc, OnceLock};

use super::path_filters::{PathFilters, create_path_filters};
use super::suppressions::{assert_no_suppressors, extract_rule_name};
use crate::api::config::{ACTIVE_KEY, ALIASES_KEY, SEVERITY_KEY};
use crate::api::{Config, RuleInstance, RuleProvider, RuleSet, RuleSetId, RuleSetProvider, Severity, Value, config_property};

/// `whichDetekt()`: the release this crate ports (tools/sync-detekt.sh).
pub const DETEKT_VERSION: &str = "2.0.0-alpha.6";

/// `AnalysisMode`. Only `Light` runs natively; a rule that needs the Analysis API is never in a ported rule set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisMode {
    Full,
    Light,
}

pub struct RuleDescriptor {
    pub rule_provider: RuleProvider,
    pub config: Arc<dyn Config>,
    pub rule_instance: RuleInstance,
    /// What `Analyzer.analyze` reads from `config` for every file; the config never changes, so read once.
    per_file: OnceLock<PerFile>,
}

pub(crate) struct PerFile {
    pub rule_set_filters: Option<PathFilters>,
    pub rule_filters: Option<PathFilters>,
    pub aliases: Vec<String>,
}

impl RuleDescriptor {
    pub(crate) fn per_file(&self) -> &PerFile {
        self.per_file.get_or_init(|| {
            assert_no_suppressors(self.config.as_ref(), &self.rule_instance.id);
            PerFile {
                rule_set_filters: self.config.parent().and_then(|parent| create_path_filters(parent.as_ref())),
                rule_filters: create_path_filters(self.config.as_ref()),
                aliases: config_property::list(self.config.as_ref(), ALIASES_KEY, &[]),
            }
        })
    }
}

/// `getRules(analysisMode, ruleSetProviders, config, log)`.
pub fn get_rules(
    analysis_mode: AnalysisMode,
    rule_set_providers: &[RuleSetProvider],
    config: &Arc<dyn Config>,
    log: &mut dyn FnMut(String),
) -> Vec<RuleDescriptor> {
    let mut descriptors = Vec::new();
    for rule_set_provider in rule_set_providers {
        let rule_set_config = config.clone().sub_config(rule_set_provider.rule_set_id.value());
        let rule_set = (rule_set_provider.instance)();
        descriptors.extend(get_rule_set_rules(&rule_set, &rule_set_config, analysis_mode, log));
    }
    descriptors
}

fn get_rule_set_rules(
    rule_set: &RuleSet,
    config: &Arc<dyn Config>,
    _analysis_mode: AnalysisMode,
    _log: &mut dyn FnMut(String),
) -> Vec<RuleDescriptor> {
    let mut descriptors = Vec::new();
    for rule_id in config.sub_config_keys() {
        let Some(rule_name) = extract_rule_name(&rule_id) else { continue };
        let Some(rule_provider) = rule_set.rule(&rule_name) else { continue };
        let rule = rule_provider(crate::api::config::empty());
        let rule_config = config.clone().sub_config(&rule_id);
        let active = is_active_or_default(config.as_ref(), true) && is_active_or_default(rule_config.as_ref(), false);
        // TODO: rules that need the Analysis API are not in the ported rule sets, so the `executable` check and
        // its "requires type resolution" debug line have nothing to act on yet.
        let executable = true;
        descriptors.push(RuleDescriptor {
            rule_provider,
            rule_instance: RuleInstance {
                id: rule_id,
                rule_set_id: rule_set.id.clone(),
                url: Some(generate_default_url(&rule_set.id, rule_name.value())),
                description: rule.description().to_owned(),
                severity: compute_severity(rule_config.as_ref()),
                active: active && executable,
            },
            config: rule_config,
            per_file: OnceLock::new(),
        });
    }
    descriptors
}

/// `Config.isActiveOrDefault(default)` (ConfigExtensions.kt).
fn is_active_or_default(config: &dyn Config, default: bool) -> bool {
    config_property::boolean(config, ACTIVE_KEY, default)
}

fn generate_default_url(rule_set_id: &RuleSetId, rule_name: &str) -> String {
    format!("https://detekt.dev/docs/{DETEKT_VERSION}/rules/{}#{}", rule_set_id.value().to_lowercase(), rule_name.to_lowercase())
}

/// Compute severity in the priority order: the rule's, the rule set's, the default.
fn compute_severity(config: &dyn Config) -> Severity {
    let config_value = config.value_or_null(SEVERITY_KEY).or_else(|| config.parent()?.value_or_null(SEVERITY_KEY));
    match config_value {
        None => Severity::Error,
        Some(Value::String(severity)) => parse_to_severity(&severity),
        Some(other) => panic!("java.lang.ClassCastException: {} cannot be cast to class java.lang.String", other.kotlin_type_name()),
    }
}

fn parse_to_severity(severity: &str) -> Severity {
    let lowercase = severity.to_lowercase();
    Severity::ENTRIES
        .into_iter()
        .find(|entry| entry.name().to_lowercase() == lowercase)
        .unwrap_or_else(|| panic!("'{severity}' is not a valid Severity. Allowed values are [Error, Warning, Info]"))
}
