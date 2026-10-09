//! The config model on the bundled `default-detekt-config.yml` and on small documents: what `YamlConfig`,
//! `CompositeConfig`, `AllRulesConfig` and `getRules` return (detekt-core's YamlConfigSpec, CompositeConfigSpec and
//! AllRulesConfigSpec are ported with the CLI; these are the cases the spike's rules depend on).

use std::path::PathBuf;
use std::sync::Arc;

use ktrs_detekt::api::{Config, Value, config, config_property};
use ktrs_detekt::config::{AllRulesConfig, CompositeConfig, YamlConfig, get_default_configuration, load_configuration, workaround_configuration};
use ktrs_detekt::engine::create_analyzer;

fn strings(items: &[&str]) -> Value {
    Value::List(items.iter().map(|item| Value::String((*item).to_owned())).collect())
}

fn sub(config: &Arc<dyn Config>, path: &[&str]) -> Arc<dyn Config> {
    path.iter().fold(config.clone(), |config, key| config.sub_config(key))
}

#[test]
fn default_config_values() {
    let default = get_default_configuration();
    let magic_number = sub(&default, &["style", "MagicNumber"]);
    assert_eq!(magic_number.value_or_null("active"), Some(Value::Boolean(true)));
    assert_eq!(magic_number.value_or_null("ignoreNumbers"), Some(strings(&["-1", "0", "1", "2"])));
    let excludes = config_property::list(magic_number.as_ref(), "excludes", &[]);
    assert_eq!(excludes.len(), 9);
    assert_eq!((excludes[0].as_str(), excludes[8].as_str()), ("**/test/**", "**/*.kts"));

    let package_naming = sub(&default, &["naming", "PackageNaming"]);
    assert_eq!(config_property::string(package_naming.as_ref(), "packagePattern", ""), r"[a-z]+(\.[a-z][A-Za-z0-9]*)*");
    assert_eq!(package_naming.value_or_null("aliases"), Some(strings(&["PackageName"])));

    let complexity = sub(&default, &["complexity", "CyclomaticComplexMethod"]);
    assert_eq!(config_property::int(complexity.as_ref(), "allowedComplexity", 0), 14);
    assert_eq!(config_property::list(complexity.as_ref(), "nestingFunctions", &[]).len(), 9);

    let reports = sub(&default, &["console-reports"]);
    assert_eq!(config_property::list(reports.as_ref(), "exclude", &[]).len(), 5);
    // `exclude:` followed only by comments is null, which `Map<String, Any>` reads as absent.
    assert_eq!(sub(&default, &["processors"]).value_or_null("exclude"), None);
    assert_eq!(default.sub_config_keys()[..4], ["config", "processors", "console-reports", "comments"]);
}

#[test]
fn default_run_activates_the_ported_default_rules() {
    let config = workaround_configuration(load_configuration(&[]).unwrap(), false, false, false);
    let analyzer = create_analyzer(PathBuf::from("."), &config);
    let ids: Vec<&str> = analyzer.rules().iter().map(|rule| rule.rule_instance.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "CyclomaticComplexMethod",
            "LongMethod",
            "TooManyFunctions",
            "EmptyCatchBlock",
            "EmptyClassBlock",
            "EmptyDefaultConstructor",
            "EmptyDoWhileBlock",
            "EmptyElseBlock",
            "EmptyFinallyBlock",
            "EmptyForBlock",
            "EmptyFunctionBlock",
            "EmptyIfBlock",
            "EmptyInitBlock",
            "EmptyKotlinFile",
            "EmptySecondaryConstructor",
            "EmptyTryBlock",
            "EmptyWhenBlock",
            "EmptyWhileBlock",
            "SwallowedException",
            "TooGenericExceptionCaught",
            "FunctionParameterNaming",
            "InvalidPackageDeclaration",
            "PackageNaming",
            "MagicNumber",
            "MaxLineLength",
            "ReturnCount",
            "WildcardImport",
        ]
    );
    let url = analyzer.rules()[0].rule_instance.url.as_deref();
    assert_eq!(url, Some("https://detekt.dev/docs/2.0.0-alpha.6/rules/complexity#cyclomaticcomplexmethod"));
}

#[test]
fn yaml_values_are_coerced_by_the_default() {
    let yaml: Arc<dyn Config> = YamlConfig::load("set:\n  Rule:\n    max: '3'\n    flag: 'true'\n    names: [a, 'b c']\n    count: 7\n").unwrap();
    let rule = sub(&yaml, &["set", "Rule"]);
    assert_eq!(config_property::int(rule.as_ref(), "max", 1), 3);
    assert!(config_property::boolean(rule.as_ref(), "flag", false));
    assert_eq!(config_property::list(rule.as_ref(), "names", &[]), ["a", "b c"]);
    assert_eq!(rule.value_or_null("count"), Some(Value::Int(7)));
    assert_eq!(config_property::string(rule.as_ref(), "missing", "default"), "default");

    let wrong_type = std::panic::catch_unwind(|| {
        let yaml: Arc<dyn Config> = YamlConfig::load("set:\n  Rule:\n    max: abc\n").unwrap();
        config_property::int(sub(&yaml, &["set", "Rule"]).as_ref(), "max", 1)
    });
    let message = wrong_type.unwrap_err().downcast_ref::<String>().cloned().unwrap();
    assert_eq!(message, "Value \"abc\" set for config parameter \"set > Rule > max\" is not of required type `kotlin.Int`");

    assert_eq!(YamlConfig::load("a: 1\na: 2\n").err().as_deref(), Some("line 2: found duplicate key a"));
    assert!(YamlConfig::load("- a\n- b\n").is_err());
    assert!(YamlConfig::load("# nothing\n").unwrap().sub_config_keys().is_empty());
}

#[test]
fn composite_and_all_rules_configs() {
    let first: Arc<dyn Config> = YamlConfig::load("style:\n  MagicNumber:\n    active: false\n  Extra:\n    x: 1\n").unwrap();
    let composite: Arc<dyn Config> = CompositeConfig::new(first.clone(), get_default_configuration());
    let magic_number = sub(&composite, &["style", "MagicNumber"]);
    assert_eq!(magic_number.value_or_null("active"), Some(Value::Boolean(false)));
    assert_eq!(config_property::boolean(magic_number.as_ref(), "ignoreEnums", true), false);
    assert_eq!(sub(&composite, &["style"]).sub_config_keys()[..2], ["MagicNumber", "Extra"]);
    assert!(magic_number.parent().is_some());

    let all_rules: Arc<dyn Config> = AllRulesConfig::new(get_default_configuration(), Vec::new());
    let no_tabs = sub(&all_rules, &["style", "NoTabs"]);
    assert_eq!(no_tabs.value_or_null("active"), Some(Value::Boolean(false)));
    assert_eq!(sub(&all_rules, &["style", "Unlisted"]).value_or_null("active"), Some(Value::Boolean(true)));
    assert!(config::empty().is_empty_config());
}
