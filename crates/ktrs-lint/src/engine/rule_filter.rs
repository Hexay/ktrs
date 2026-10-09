//! Ports of ktlint-rule-engine `internal/rulefilter/RuleFilter.kt`, `InternalRuleProvidersFilter.kt` and
//! `RuleExecutionRuleFilter.kt`.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::editorconfig::{
    ALL_RULES_EXECUTION_PROPERTY, CODE_STYLE_PROPERTY, CodeStyleValue,
    EXPERIMENTAL_RULES_EXECUTION_PROPERTY, EditorConfig, RULE_EXECUTION_PROPERTY_TYPE,
    RuleExecution, rule_execution_property_name, rule_set_execution_property_name,
};
use crate::engine::internal_rules::{KTLINT_SUPPRESSION_RULE_ID, ktlint_suppression_rule_provider};
use crate::rule::RuleV2;
use crate::rule_provider::RuleV2Provider;

pub(crate) const RULE_EXECUTION_RULE_FILTER_LOGGER: &str =
    "io.github.ktlint.core.rule.engine.internal.rulefilter.RuleExecutionRuleFilter";

pub trait RuleFilter {
    fn filter(&self, rule_providers: Vec<RuleV2Provider>) -> Vec<RuleV2Provider>;
}

/// `KtLintRuleEngine.applyRuleFilters(filters)`: the distinct providers (first per rule id) through each
/// filter in turn.
pub fn apply_rule_filters(
    rule_providers: &[RuleV2Provider],
    rule_filters: &[&dyn RuleFilter],
) -> Vec<RuleV2Provider> {
    let mut providers = initial_rule_providers(rule_providers);
    for rule_filter in rule_filters {
        providers = rule_filter.filter(providers);
    }
    providers
}

fn initial_rule_providers(rule_providers: &[RuleV2Provider]) -> Vec<RuleV2Provider> {
    let mut distinct: Vec<RuleV2Provider> = Vec::new();
    for p in rule_providers {
        if !distinct.iter().any(|d| d.rule_id() == p.rule_id()) {
            distinct.push(p.clone());
        }
    }
    distinct
}

/// Adds the internal rules, which always run; a consumer's rule with an internal rule's id is dropped.
pub struct InternalRuleProvidersFilter {
    internal_rule_providers: Vec<RuleV2Provider>,
}

impl InternalRuleProvidersFilter {
    pub fn new(engine_rule_providers: &[RuleV2Provider]) -> InternalRuleProvidersFilter {
        let allowed_rule_ids = engine_rule_providers
            .iter()
            .map(RuleV2Provider::rule_id)
            .collect();
        InternalRuleProvidersFilter {
            internal_rule_providers: vec![ktlint_suppression_rule_provider(allowed_rule_ids)],
        }
    }
}

impl RuleFilter for InternalRuleProvidersFilter {
    fn filter(&self, rule_providers: Vec<RuleV2Provider>) -> Vec<RuleV2Provider> {
        let internal_rule_ids: Vec<_> = self
            .internal_rule_providers
            .iter()
            .map(RuleV2Provider::rule_id)
            .collect();
        let mut providers: Vec<RuleV2Provider> = rule_providers
            .into_iter()
            .filter(|p| !internal_rule_ids.contains(&p.rule_id()))
            .collect();
        providers.extend(self.internal_rule_providers.iter().cloned());
        providers
    }
}

/// The providers of the rules that are enabled in the `.editorconfig`.
pub struct RuleExecutionRuleFilter<'a> {
    editor_config: &'a EditorConfig,
    warnings: RefCell<Vec<String>>,
}

impl RuleExecutionRuleFilter<'_> {
    pub fn new(editor_config: &EditorConfig) -> RuleExecutionRuleFilter<'_> {
        RuleExecutionRuleFilter { editor_config, warnings: RefCell::default() }
    }

    /// What `filter` logged at WARN (upstream: on every file; here replayed from the cached `RuleSetup`).
    pub fn into_warnings(self) -> Vec<String> {
        self.warnings.into_inner()
    }

    fn disable_ktlint_entirely(&self) -> bool {
        self.editor_config.get_editor_config_value_or_null(
            &RULE_EXECUTION_PROPERTY_TYPE,
            &ALL_RULES_EXECUTION_PROPERTY.name,
        ) == Some(RuleExecution::Disabled)
    }

    fn rule_execution_properties(
        &self,
        rule_providers: &[RuleV2Provider],
    ) -> HashMap<String, Option<RuleExecution>> {
        let mut names: Vec<String> = rule_providers
            .iter()
            .map(|p| rule_execution_property_name(p.rule_id().value()))
            .collect();
        names.extend(
            rule_providers
                .iter()
                .map(|p| rule_set_execution_property_name(p.rule_id().rule_set_id().value())),
        );
        names.push(EXPERIMENTAL_RULES_EXECUTION_PROPERTY.name.to_string());
        self.editor_config
            .map(|p| (p.name().to_owned(), p.source_value().map(str::to_owned)))
            .into_iter()
            .filter(|(name, _)| names.contains(name))
            .map(|(name, value)| {
                (
                    name,
                    RULE_EXECUTION_PROPERTY_TYPE
                        .parse(value.as_deref())
                        .into_parsed(),
                )
            })
            .collect()
    }
}

impl RuleFilter for RuleExecutionRuleFilter<'_> {
    fn filter(&self, rule_providers: Vec<RuleV2Provider>) -> Vec<RuleV2Provider> {
        if self.disable_ktlint_entirely() {
            return Vec::new();
        }
        let rule_execution_filter = RuleExecutionFilter {
            rule_execution_properties: self.rule_execution_properties(&rule_providers),
            code_style_value: self.editor_config.get(&CODE_STYLE_PROPERTY),
            warnings: &self.warnings,
        };
        rule_providers
            .into_iter()
            .filter(|p| rule_execution_filter.is_enabled(p))
            .collect()
    }
}

/// The rule execution properties of the `.editorconfig` (no `EditorConfigProperty` exists for them).
struct RuleExecutionFilter<'a> {
    rule_execution_properties: HashMap<String, Option<RuleExecution>>,
    code_style_value: CodeStyleValue,
    warnings: &'a RefCell<Vec<String>>,
}

impl RuleExecutionFilter<'_> {
    fn is_enabled(&self, rule_provider: &RuleV2Provider) -> bool {
        self.is_rule_enabled(&*rule_provider.create_new_rule_instance())
    }

    /// The rule's own execution property beats the conditions, so one experimental or official rule can be
    /// enabled alone, or one rule disabled when its group is enabled.
    fn is_rule_enabled(&self, rule: &dyn RuleV2) -> bool {
        match self.rule_execution(&rule_execution_property_name(rule.rule_id().value())) {
            Some(RuleExecution::Disabled) if rule.rule_id() == KTLINT_SUPPRESSION_RULE_ID => {
                let warning = format!("Rule '{}' can not be disabled via the '.editorconfig'", rule.rule_id().value());
                self.warnings.borrow_mut().push(warning);
                true
            }
            Some(it) => it == RuleExecution::Enabled,
            None => self.is_rule_conditionally_enabled(rule),
        }
    }

    fn is_rule_conditionally_enabled(&self, rule: &dyn RuleV2) -> bool {
        if rule.is_experimental() && rule.is_official_code_style() {
            self.is_experimental_enabled(rule) && self.is_official_code_style_enabled(rule)
        } else if rule.is_experimental() {
            self.is_experimental_enabled(rule)
        } else if rule.is_official_code_style() {
            self.is_official_code_style_enabled(rule)
        } else if rule.is_only_when_enabled_in_editorconfig() {
            self.rule_execution(&rule_execution_property_name(rule.rule_id().value()))
                == Some(RuleExecution::Disabled)
        } else {
            self.is_rule_set_enabled(rule)
        }
    }

    fn is_experimental_enabled(&self, rule: &dyn RuleV2) -> bool {
        self.rule_execution(&EXPERIMENTAL_RULES_EXECUTION_PROPERTY.name)
            == Some(RuleExecution::Enabled)
            && self.rule_execution(&rule_set_execution_property_name(
                rule.rule_id().rule_set_id().value(),
            )) != Some(RuleExecution::Disabled)
            && self.rule_execution(&rule_execution_property_name(rule.rule_id().value()))
                != Some(RuleExecution::Disabled)
    }

    fn is_official_code_style_enabled(&self, rule: &dyn RuleV2) -> bool {
        self.code_style_value == CodeStyleValue::KtlintOfficial
            && self.rule_execution(&rule_set_execution_property_name(
                rule.rule_id().rule_set_id().value(),
            )) != Some(RuleExecution::Disabled)
            && self.rule_execution(&rule_execution_property_name(rule.rule_id().value()))
                != Some(RuleExecution::Disabled)
    }

    /// Upstream compares the enum's name with "ktlint_experimental" here, which never matches, so every
    /// rule set is enabled unless disabled.
    fn is_rule_set_enabled(&self, rule: &dyn RuleV2) -> bool {
        self.rule_execution(&rule_set_execution_property_name(
            rule.rule_id().rule_set_id().value(),
        )) != Some(RuleExecution::Disabled)
    }

    fn rule_execution(&self, rule_execution_property_name: &str) -> Option<RuleExecution> {
        self.rule_execution_properties
            .get(rule_execution_property_name)
            .copied()
            .flatten()
    }
}
