//! Ports of ktlint-rule-engine `internal/rulefilter/RuleExecutionRuleFilterTest.kt`,
//! `InternalRuleProvidersFilterTest.kt` and `RuleFilterKtTest.kt`.

use ktrs_editorconfig::Property;
use ktrs_lint::editorconfig::{
    ALL_RULES_EXECUTION_PROPERTY, CODE_STYLE_PROPERTY, PropertyRef, RULE_EXECUTION_PROPERTY_TYPE,
    rule_execution_property_name,
};
use ktrs_lint::engine::internal_rules::{KTLINT_SUPPRESSION_RULE_ID, KtlintSuppressionRule};
use ktrs_lint::engine::rule_filter::{
    InternalRuleProvidersFilter, RuleExecutionRuleFilter, RuleFilter, apply_rule_filters,
};
use ktrs_lint::{EditorConfig, KtLintRuleEngine, RuleId, RuleV2, RuleV2Provider};

const STANDARD_RULE_A: RuleId = RuleId("standard:rule-a");
const STANDARD_RULE_B: RuleId = RuleId("standard:rule-b");
const STANDARD_RULE_C: RuleId = RuleId("standard:rule-c");
const STANDARD_RULE_D: RuleId = RuleId("standard:rule-d");
const CUSTOM_RULE_B: RuleId = RuleId("custom:rule-b");
const CUSTOM_RULE_C: RuleId = RuleId("custom:rule-c");

#[derive(Clone, Copy)]
enum Kind {
    Normal,
    Experimental,
    OnlyWhenEnabledInEditorconfig,
}

struct TestRule {
    rule_id: RuleId,
    kind: Kind,
}

impl RuleV2 for TestRule {
    fn rule_id(&self) -> RuleId {
        self.rule_id
    }

    fn is_experimental(&self) -> bool {
        matches!(self.kind, Kind::Experimental)
    }

    fn is_only_when_enabled_in_editorconfig(&self) -> bool {
        matches!(self.kind, Kind::OnlyWhenEnabledInEditorconfig)
    }
}

fn provider(rule_id: RuleId, kind: Kind) -> RuleV2Provider {
    RuleV2Provider::new(move || Box::new(TestRule { rule_id, kind }) as Box<dyn RuleV2>)
}

fn normal(rule_id: RuleId) -> RuleV2Provider {
    provider(rule_id, Kind::Normal)
}

fn experimental(rule_id: RuleId) -> RuleV2Provider {
    provider(rule_id, Kind::Experimental)
}

fn ktlint_suppression(allowed: RuleId) -> RuleV2Provider {
    RuleV2Provider::new(move || {
        Box::new(KtlintSuppressionRule::new(vec![allowed])) as Box<dyn RuleV2>
    })
}

/// `ktLintRuleExecutionEditorConfigProperty(name, ruleExecution)`.
fn execution(name: &str, rule_execution: &str) -> Property {
    Property::new(
        name,
        Some(&RULE_EXECUTION_PROPERTY_TYPE),
        Some(rule_execution),
    )
}

fn rule(rule_id: RuleId, rule_execution: &str) -> Property {
    execution(
        &rule_execution_property_name(rule_id.value()),
        rule_execution,
    )
}

fn run_with_rule_execution_rule_filter(
    rule_providers: Vec<RuleV2Provider>,
    properties: Vec<Property>,
) -> Vec<RuleId> {
    let editor_config = EditorConfig::new(properties)
        .add_properties_with_default_value_if_missing(&[PropertyRef::from(&*CODE_STYLE_PROPERTY)]);
    RuleExecutionRuleFilter::new(&editor_config)
        .filter(rule_providers)
        .iter()
        .map(RuleV2Provider::rule_id)
        .collect()
}

#[test]
fn given_that_standard_rule_set_is_enabled_explicitly_then_run_all_standard_rules_except_those_that_are_disabled_explicitly()
 {
    let actual = run_with_rule_execution_rule_filter(
        vec![
            normal(STANDARD_RULE_A),
            normal(STANDARD_RULE_B),
            normal(STANDARD_RULE_C),
        ],
        vec![
            execution("ktlint_standard", "enabled"),
            rule(STANDARD_RULE_C, "disabled"),
        ],
    );
    assert_eq!(actual, [STANDARD_RULE_A, STANDARD_RULE_B]);
}

#[test]
fn given_that_standard_rule_set_is_not_enabled_explicitly_then_run_all_standard_rules_except_experimental_and_explicitly_disabled_rules()
 {
    let actual = run_with_rule_execution_rule_filter(
        vec![
            normal(STANDARD_RULE_A),
            experimental(STANDARD_RULE_B),
            normal(STANDARD_RULE_C),
            normal(STANDARD_RULE_D),
        ],
        vec![rule(STANDARD_RULE_C, "disabled")],
    );
    assert_eq!(actual, [STANDARD_RULE_A, STANDARD_RULE_D]);
}

#[test]
fn given_that_standard_rule_set_is_disabled_explicitly_then_only_run_standard_rules_that_are_enabled_explicitly()
 {
    let actual = run_with_rule_execution_rule_filter(
        vec![
            normal(STANDARD_RULE_A),
            normal(STANDARD_RULE_B),
            normal(STANDARD_RULE_C),
        ],
        vec![
            execution("ktlint_standard", "disabled"),
            rule(STANDARD_RULE_A, "enabled"),
            rule(STANDARD_RULE_B, "enabled"),
        ],
    );
    assert_eq!(actual, [STANDARD_RULE_A, STANDARD_RULE_B]);
}

fn four_experimental_rules() -> Vec<RuleV2Provider> {
    vec![
        experimental(STANDARD_RULE_B),
        experimental(STANDARD_RULE_C),
        experimental(CUSTOM_RULE_B),
        experimental(CUSTOM_RULE_C),
    ]
}

#[test]
fn given_that_the_experimental_rules_are_not_disabled_explicitly_then_only_run_rules_that_are_enabled_explicitly()
 {
    let actual = run_with_rule_execution_rule_filter(
        four_experimental_rules(),
        vec![
            rule(STANDARD_RULE_B, "enabled"),
            rule(CUSTOM_RULE_B, "enabled"),
        ],
    );
    assert_eq!(actual, [STANDARD_RULE_B, CUSTOM_RULE_B]);
}

#[test]
fn given_that_a_experimental_rules_are_disabled_explicitly_then_only_run_rules_that_are_enabled_explicitly()
 {
    let actual = run_with_rule_execution_rule_filter(
        four_experimental_rules(),
        vec![
            execution("ktlint_experimental", "disabled"),
            rule(STANDARD_RULE_B, "enabled"),
            execution("ktlint_custom", "disabled"),
            rule(CUSTOM_RULE_B, "enabled"),
        ],
    );
    assert_eq!(actual, [STANDARD_RULE_B, CUSTOM_RULE_B]);
}

#[test]
fn given_that_the_experimental_rules_are_enabled_then_only_run_rules_that_are_not_disabled_explicitly()
 {
    let actual = run_with_rule_execution_rule_filter(
        four_experimental_rules(),
        vec![
            execution("ktlint_experimental", "enabled"),
            rule(STANDARD_RULE_C, "disabled"),
            execution("ktlint_custom", "enabled"),
            rule(CUSTOM_RULE_C, "disabled"),
        ],
    );
    assert_eq!(actual, [STANDARD_RULE_B, CUSTOM_RULE_B]);
}

#[test]
fn when_some_standard_rules_which_are_all_disabled_explicitly_then_return_empty() {
    let actual = run_with_rule_execution_rule_filter(
        vec![normal(STANDARD_RULE_A), normal(STANDARD_RULE_B)],
        vec![
            rule(STANDARD_RULE_A, "disabled"),
            rule(STANDARD_RULE_B, "disabled"),
        ],
    );
    assert!(actual.is_empty());
}

#[test]
fn given_that_the_ktlint_suppression_is_disabled_in_the_editorconfig_properties_then_ignore_that_property()
 {
    let actual = run_with_rule_execution_rule_filter(
        vec![normal(STANDARD_RULE_A), ktlint_suppression(STANDARD_RULE_A)],
        vec![
            rule(STANDARD_RULE_A, "disabled"),
            rule(KTLINT_SUPPRESSION_RULE_ID, "disabled"),
        ],
    );
    assert_eq!(actual, [KTLINT_SUPPRESSION_RULE_ID]);
}

#[test]
fn given_that_ktlint_is_disabled_entirely_then_the_internal_rule_for_migrating_the_ktlint_disable_directives_is_disabled()
 {
    let actual = run_with_rule_execution_rule_filter(
        vec![normal(STANDARD_RULE_A), ktlint_suppression(STANDARD_RULE_A)],
        vec![execution(&ALL_RULES_EXECUTION_PROPERTY.name, "disabled")],
    );
    assert!(actual.is_empty());
}

#[test]
fn given_a_rule_that_only_should_be_run_when_enabled_explicitly_and_the_rule_execution_property_is_not_set_then_do_not_execute_the_rule()
 {
    let actual = run_with_rule_execution_rule_filter(
        vec![provider(
            STANDARD_RULE_A,
            Kind::OnlyWhenEnabledInEditorconfig,
        )],
        vec![],
    );
    assert!(actual.is_empty());
}

#[test]
fn given_a_rule_that_only_should_be_run_when_enabled_explicitly_and_the_rule_execution_property_is_enabled_then_do_execute_the_rule()
 {
    let actual = run_with_rule_execution_rule_filter(
        vec![provider(
            STANDARD_RULE_A,
            Kind::OnlyWhenEnabledInEditorconfig,
        )],
        vec![rule(STANDARD_RULE_A, "enabled")],
    );
    assert_eq!(actual, [STANDARD_RULE_A]);
}

#[test]
fn given_a_ktlint_rule_engine_then_add_the_ktlint_suppression_rule_provider() {
    let engine = KtLintRuleEngine::new(vec![normal(STANDARD_RULE_A)]);
    let actual: Vec<RuleId> = InternalRuleProvidersFilter::new(engine.rule_providers())
        .filter(engine.rule_providers().to_vec())
        .iter()
        .map(RuleV2Provider::rule_id)
        .collect();
    assert_eq!(actual, [STANDARD_RULE_A, KTLINT_SUPPRESSION_RULE_ID]);
}

const RULE_SET_A_RULE_A: RuleId = RuleId("ruleset-a:rule-a");
const RULE_SET_A_RULE_B: RuleId = RuleId("ruleset-a:rule-b");
const RULE_SET_B_RULE_A: RuleId = RuleId("ruleset-b:rule-a");
const RULE_SET_B_RULE_B: RuleId = RuleId("ruleset-b:rule-b");

struct RuleIdRuleFilter(&'static str);

impl RuleFilter for RuleIdRuleFilter {
    fn filter(&self, rule_providers: Vec<RuleV2Provider>) -> Vec<RuleV2Provider> {
        rule_providers
            .into_iter()
            .filter(|p| p.rule_id().value().contains(self.0))
            .collect()
    }
}

fn apply(rule_ids: &[RuleId], rule_filters: &[&dyn RuleFilter]) -> Vec<RuleId> {
    let engine = KtLintRuleEngine::new(rule_ids.iter().map(|&id| normal(id)).collect());
    let mut actual: Vec<RuleId> = apply_rule_filters(engine.rule_providers(), rule_filters)
        .iter()
        .map(RuleV2Provider::rule_id)
        .collect();
    actual.sort();
    actual
}

#[test]
fn given_an_empty_list_of_rule_filters_then_the_list_of_rule_providers_contains_provider_for_all_rule_ids_initially_provided_by_the_ktlint_engine()
 {
    let all = [
        RULE_SET_A_RULE_A,
        RULE_SET_A_RULE_B,
        RULE_SET_B_RULE_A,
        RULE_SET_B_RULE_B,
    ];
    assert_eq!(apply(&all, &[]), all);
}

#[test]
fn given_a_single_rule_filter_then_the_list_of_rule_providers_contains_only_rule_ids_that_match_that_filter()
 {
    let all = [
        RULE_SET_A_RULE_A,
        RULE_SET_A_RULE_B,
        RULE_SET_B_RULE_A,
        RULE_SET_B_RULE_B,
    ];
    assert_eq!(
        apply(&all, &[&RuleIdRuleFilter("ruleset-a")]),
        [RULE_SET_A_RULE_A, RULE_SET_A_RULE_B]
    );
}

#[test]
fn given_multiple_rule_filters_then_the_list_of_rule_providers_contains_only_rule_ids_that_match_all_filters()
 {
    let actual = apply(
        &[RULE_SET_A_RULE_A, RULE_SET_A_RULE_B, RULE_SET_B_RULE_B],
        &[&RuleIdRuleFilter("ruleset-a"), &RuleIdRuleFilter("rule-b")],
    );
    assert_eq!(actual, [RULE_SET_A_RULE_B]);
}

#[test]
fn given_multiple_rule_filters_that_exclude_each_other_then_the_list_of_rule_providers_is_empty() {
    assert!(
        apply(
            &[RULE_SET_A_RULE_A],
            &[&RuleIdRuleFilter("rule-a"), &RuleIdRuleFilter("ruleset-b")]
        )
        .is_empty()
    );
}
