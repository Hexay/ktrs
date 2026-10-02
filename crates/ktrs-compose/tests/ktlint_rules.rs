//! The `KtlintRule` adapter end to end through the engine (dispatch, emit offsets, `.editorconfig` gating, fixes,
//! the 1.8/2.0 switch), and `KtlintComposeKtConfig`'s lookups. Parity with the jars: `tests/golden`.

use ktrs_compose::core::compose_kt_config::ComposeKtConfig;
use ktrs_compose::core::util::kotlin_utils::{to_camel_case, to_snake_case};
use ktrs_compose::ktlint::editor_config_properties::{
    CONTENT_EMITTERS_PROPERTY, CUSTOM_MODIFIERS, COMPOSE_PREVIEW_NAMING_ENABLED, ComposeProperty,
};
use ktrs_compose::ktlint::ktlint_compose_kt_config::KtlintComposeKtConfig;
use ktrs_ast::psi::EmbeddedKotlin;
use ktrs_editorconfig::Property;
use ktrs_lint::editorconfig::{KTLINT_VERSION_PROPERTY, KtlintVersion, PropertyRef};
use ktrs_lint::{AutocorrectDecision, Code, EditorConfig, EditorConfigDefaults, EditorConfigOverride, KtLintRuleEngine};

fn engine(version: KtlintVersion, overrides: Vec<(PropertyRef, Option<String>)>) -> KtLintRuleEngine {
    let editor_config_override = if overrides.is_empty() { EditorConfigOverride::empty() } else { EditorConfigOverride::from(overrides) };
    let editor_config_override = editor_config_override.with(&KTLINT_VERSION_PROPERTY, version);
    KtLintRuleEngine::with_editor_config(ktrs_compose::compose_rule_providers(), EditorConfigDefaults::empty(), editor_config_override)
}

fn lint(engine: &KtLintRuleEngine, text: &str) -> Vec<String> {
    let mut rows = Vec::new();
    engine.lint(&Code::from_snippet(text, false), &mut |e| rows.push(format!("{}:{} {}", e.line, e.col, e.rule_id))).unwrap();
    rows
}

fn format(engine: &KtLintRuleEngine, text: &str) -> String {
    engine.format(&Code::from_snippet(text, false), &mut |_| AutocorrectDecision::AllowAutocorrect).unwrap()
}

#[test]
fn reports_at_the_name_identifier_and_fixes_through_the_psi_factory() {
    // ModifierWithoutDefault reports the parameter (a PsiNameIdentifierOwner: its name's offset) and replaces it
    // with `KtPsiFactory.createParameter("<text> = Modifier")`.
    let engine = engine(KtlintVersion::V2_0, Vec::new());
    let code = "@Composable\nfun Foo(@Ann modifier: Modifier) {}\n";
    assert_eq!(lint(&engine, code), ["2:14 compose:modifier-without-default-check"]);
    assert_eq!(format(&engine, code), "@Composable\nfun Foo(@Ann modifier: Modifier = Modifier) {}\n");
}

#[test]
fn opt_in_rules_need_their_editorconfig_property() {
    let code = "@Preview\n@Composable\nprivate fun Foo() {}\n";
    assert!(lint(&engine(KtlintVersion::V2_0, Vec::new()), code).is_empty());
    let enabled = vec![(PropertyRef::from(&*COMPOSE_PREVIEW_NAMING_ENABLED), Some("true".to_owned()))];
    let engine = engine(KtlintVersion::V2_0, enabled);
    assert_eq!(lint(&engine, code), ["3:13 compose:preview-naming"]);
    assert_eq!(format(&engine, code), "@Preview\n@Composable\nprivate fun FooPreview() {}\n");
}

#[test]
fn set_name_quotes_latin1_letters_in_1_8_only() {
    // KtNamedDeclarationStub.setName -> quoteIfNeeded: Kotlin 2.2.21 (ktlint 1.8.0) quotes `é`, 2.4.10 does not.
    let enabled = || vec![(PropertyRef::from(&*COMPOSE_PREVIEW_NAMING_ENABLED), Some("true".to_owned()))];
    let code = "@Preview\n@Composable\nprivate fun Café() {}\n";
    assert_eq!(format(&engine(KtlintVersion::V1_8, enabled()), code), "@Preview\n@Composable\nprivate fun `CaféPreview`() {}\n");
    assert_eq!(format(&engine(KtlintVersion::V2_0, enabled()), code), "@Preview\n@Composable\nprivate fun CaféPreview() {}\n");
}

fn property(p: ComposeProperty, value: &str) -> Property {
    let ComposeProperty::String(lazy) = p else { panic!("a string property") };
    Property::new(p.name(), Some(std::sync::LazyLock::force(lazy).type_), Some(value))
}

#[test]
fn config_keys_values_and_the_memoization_quirk() {
    // ktlintKey = "compose_" + toSnakeCase(key); getList splits on ',' and ';' and trims; the cache is keyed by key
    // alone, so a getList after a getSet of the same key reads a Set `as? List` (null) and yields the default.
    assert_eq!(to_snake_case("contentEmittersDenylist"), "content_emitters_denylist");
    assert_eq!(to_camel_case("content_emitters"), "ContentEmitters");
    let editor_config = EditorConfig::new(vec![property(ComposeProperty::String(&CUSTOM_MODIFIERS), "A; B ,A")]);
    let config = KtlintComposeKtConfig::new(
        editor_config,
        vec![ComposeProperty::String(&CUSTOM_MODIFIERS), ComposeProperty::String(&CONTENT_EMITTERS_PROPERTY)],
        EmbeddedKotlin::V2_4_10,
    );
    assert_eq!(config.get_set("customModifiers", &[]), ["A", "B"]);
    assert_eq!(config.get_list("customModifiers", &["d".to_owned()]), ["d"]);
    assert_eq!(config.get_set("notMine", &["x".to_owned()]), ["x"]);
    assert!(config.get_boolean("customModifiers", true));
}
