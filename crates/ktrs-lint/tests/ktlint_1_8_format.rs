//! ktlint 1.8 mode, format side (research/26-ktlint-18-mode.md, phase 2): 1.8's rule order from the rules'
//! `VisitorModifier`s, its `RunAfterRuleFilter`, and the pre-#3261 when-entry-bracing rewrite. Expected outputs
//! come from the jars (`ktlint-<version> -F` with `ktlint_code_style = ktlint_official`).

use std::path::{Path, PathBuf};

use ktrs_lint::editorconfig::{KTLINT_VERSION_PROPERTY, KtlintVersion};
use ktrs_lint::engine::rule_provider_sorter_1_8::{get_sorted_rule_providers_1_8, run_after_rule_filter};
use ktrs_lint::rule_provider::RuleV2Provider;
use ktrs_lint::rules::standard_rule_providers;
use ktrs_lint::{AutocorrectDecision, Code, EditorConfigDefaults, EditorConfigOverride, KtLintRuleEngine};

/// `ktlint-1.8.0 --log-level=debug` with `ktlint_code_style = ktlint_official` ("Rules will be executed in
/// order below"), minus the internal suppression rule.
const KTLINT_1_8_OFFICIAL_ORDER: &str = "annotation-spacing backing-property-naming binary-expression-wrapping \
    blank-line-before-declaration blank-line-between-when-conditions chain-wrapping class-naming colon-spacing comma-spacing \
    comment-spacing comment-wrapping condition-wrapping context-receiver-list-wrapping context-receiver-wrapping curly-spacing \
    discouraged-comment-location dot-spacing double-colon-spacing enum-entry-name-case enum-wrapping filename final-newline \
    fun-keyword-spacing function-expression-body function-naming function-return-type-spacing function-start-of-body-spacing \
    function-type-modifier-spacing function-type-reference-spacing if-else-bracing if-else-wrapping import-ordering kdoc \
    kdoc-wrapping keyword-spacing mixed-condition-operators modifier-order multiline-expression-wrapping multiline-if-else \
    multiline-loop no-blank-line-before-rbrace no-blank-line-in-list no-blank-lines-in-chained-method-calls \
    no-consecutive-blank-lines no-consecutive-comments no-empty-class-body no-empty-file no-empty-first-line-in-class-body \
    no-empty-first-line-in-method-block no-line-break-after-else no-line-break-before-assignment no-multi-spaces \
    no-trailing-spaces no-unit-return no-wildcard-imports nullable-type-spacing op-spacing package-name \
    parameter-list-spacing parameter-list-wrapping parameter-wrapping paren-spacing property-naming property-wrapping \
    range-spacing spacing-around-angle-brackets spacing-between-declarations-with-annotations \
    spacing-between-declarations-with-comments spacing-between-function-name-and-opening-parenthesis square-brackets-spacing \
    statement-wrapping string-template then-spacing try-catch-finally-spacing type-argument-comment type-argument-list-spacing \
    type-parameter-comment type-parameter-list-spacing unary-op-spacing unnecessary-parentheses-before-trailing-lambda \
    value-argument-comment value-parameter-comment when-entry-bracing annotation modifier-list-spacing \
    no-single-line-block-comment wrapping no-semi class-signature function-signature argument-list-wrapping \
    chain-method-continuation function-literal trailing-comma-on-call-site trailing-comma-on-declaration-site indent \
    block-comment-initial-star-alignment string-template-indent max-line-length";

fn providers_1_8(rule_ids: &[String]) -> Vec<RuleV2Provider> {
    standard_rule_providers()
        .into_iter()
        .filter(|p| p.runs_in(KtlintVersion::V1_8) && rule_ids.iter().any(|id| id == p.rule_id().value()))
        .collect()
}

fn official_order() -> Vec<String> {
    KTLINT_1_8_OFFICIAL_ORDER.split_whitespace().map(|id| format!("standard:{id}")).collect()
}

fn ids(providers: &[RuleV2Provider]) -> Vec<String> {
    providers.iter().map(|p| p.rule_id().value().to_owned()).collect()
}

#[test]
fn rule_order_1_8_is_the_jars() {
    let expected = official_order();
    let mut providers = providers_1_8(&expected);
    // The sorter must not depend on the input order.
    providers.reverse();
    let filtered = run_after_rule_filter(providers).unwrap();
    assert_eq!(ids(&get_sorted_rule_providers_1_8(&filtered)), expected);
}

#[test]
fn run_after_rule_filter_rejects_a_rule_whose_required_rule_is_disabled() {
    let mut rule_ids = official_order();
    rule_ids.retain(|id| id != "standard:indent" && id != "standard:wrapping");
    let error = run_after_rule_filter(providers_1_8(&rule_ids)).unwrap_err();
    // `ktlint-1.8.0` with `ktlint_standard_wrapping = disabled` (and indent likewise).
    assert_eq!(
        error,
        "Skipping rule(s) which are depending on a rule which is not loaded. Please check if you need to add additional \
         rule sets before creating an issue.\n  \
         - Rule with id 'RuleId(value=standard:string-template-indent)' requires rule with id 'RuleId(value=standard:indent)' to be loaded\n  \
         - Rule with id 'RuleId(value=standard:trailing-comma-on-call-site)' requires rule with id 'RuleId(value=standard:wrapping)' to be loaded\n  \
         - Rule with id 'RuleId(value=standard:trailing-comma-on-declaration-site)' requires rule with id 'RuleId(value=standard:wrapping)' to be loaded"
    );
}

#[test]
fn run_after_rule_filter_keeps_rules_whose_optional_predecessor_is_missing() {
    let mut rule_ids = official_order();
    rule_ids.retain(|id| id != "standard:annotation" && id != "standard:enum-wrapping");
    let filtered = run_after_rule_filter(providers_1_8(&rule_ids)).unwrap();
    assert_eq!(filtered.len(), rule_ids.len());
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/ktlint_1_8_format")
}

fn format_in(ktlint_version: KtlintVersion, file: &str) -> String {
    let editor_config_override = EditorConfigOverride::empty()
        .with(&KTLINT_VERSION_PROPERTY, ktlint_version)
        .with(&ktrs_lint::editorconfig::CODE_STYLE_PROPERTY, ktrs_lint::editorconfig::CodeStyleValue::KtlintOfficial);
    let engine = KtLintRuleEngine::with_editor_config(
        standard_rule_providers(),
        EditorConfigDefaults::empty(),
        editor_config_override,
    );
    let code = Code::from_file(&fixtures().join(file)).unwrap();
    engine.format(&code, &mut |_| AutocorrectDecision::AllowAutocorrect).unwrap()
}

fn expected(file: &str) -> String {
    std::fs::read_to_string(fixtures().join(file)).unwrap()
}

/// 1.8 braces the body in place; 2.0's #3261 rewrite keeps only the last condition.
#[test]
fn when_entry_bracing_1_8_keeps_every_condition() {
    assert_eq!(format_in(KtlintVersion::V1_8, "WhenEntryBracing.kt"), expected("WhenEntryBracing.1.8.kt.txt"));
}

#[test]
fn when_entry_bracing_2_0_is_unchanged() {
    assert_eq!(format_in(KtlintVersion::V2_0, "WhenEntryBracing.kt"), expected("WhenEntryBracing.2.0.kt.txt"));
}
