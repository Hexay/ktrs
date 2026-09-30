//! Port of ktlint-rule-engine-core `editorconfig/EditorConfigTest.kt`. Kotlin's `String?` code-style
//! defaults are plain `String`s here; test property types are leaked to get the `'static` the API needs.

mod engine_editorconfig_support;

use std::panic::{AssertUnwindSafe, catch_unwind};

use engine_editorconfig_support::TestFileSystem;
use ktrs_editorconfig::property_type::identity_value_parser;
use ktrs_editorconfig::{Property, PropertyType};
use ktrs_lint::EditorConfig;
use ktrs_lint::EditorConfigDefaults;
use ktrs_lint::editorconfig::{
    CODE_STYLE_PROPERTY, EditorConfigProperty, PropertyRef, RuleExecution, safe_enum_value_parser,
    to_property_with_value,
};

const SOME_PROPERTY_NAME: &str = "some-property-name";
const SOME_PROPERTY_VALUE: &str = "some-property-value";
const SOME_PROPERTY_VALUE_ANDROID_STUDIO: &str = "some-property-value-android";
const SOME_PROPERTY_VALUE_DEFAULT: &str = "some-property-value-default";
const SOME_PROPERTY_VALUE_INTELLIJ_IDEA: &str = "some-property-value-intellij-idea";
const SOME_PROPERTY_VALUE_KTLINT_OFFICIAL: &str = "some-property-value-ktlint-official";

fn string_type(name: &str) -> &'static PropertyType<String> {
    Box::leak(Box::new(PropertyType {
        name: Box::leak(name.to_owned().into_boxed_str()),
        description: "",
        parser: identity_value_parser,
        possible_values: &[
            SOME_PROPERTY_VALUE_ANDROID_STUDIO,
            SOME_PROPERTY_VALUE_INTELLIJ_IDEA,
        ],
        lower_casing: false,
    }))
}

/// `sampleEditorConfigProperty(...)` with its defaults.
fn sample(name: &str) -> EditorConfigProperty<String> {
    EditorConfigProperty {
        android_studio_code_style_default_value: SOME_PROPERTY_VALUE_ANDROID_STUDIO.to_owned(),
        ktlint_official_code_style_default_value: SOME_PROPERTY_VALUE_KTLINT_OFFICIAL.to_owned(),
        intellij_idea_code_style_default_value: SOME_PROPERTY_VALUE_INTELLIJ_IDEA.to_owned(),
        ..EditorConfigProperty::new(string_type(name), SOME_PROPERTY_VALUE_DEFAULT.to_owned())
    }
}

fn sample_editor_config_property() -> EditorConfigProperty<String> {
    sample(SOME_PROPERTY_NAME)
}

fn with_default(default_value: &str) -> EditorConfigProperty<String> {
    EditorConfigProperty {
        default_value: default_value.to_owned(),
        ..sample_editor_config_property()
    }
}

fn property_ref(p: EditorConfigProperty<String>) -> PropertyRef {
    PropertyRef::from(p)
}

fn panic_message(f: impl FnOnce()) -> String {
    let payload = catch_unwind(AssertUnwindSafe(f)).expect_err("expected an exception");
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default()
}

#[test]
fn given_an_editor_config_from_which_a_non_existing_property_is_retrieved_then_an_exception_is_thrown()
 {
    let message = panic_message(|| {
        EditorConfig::default().get(&sample_editor_config_property());
    });
    assert!(
        message.starts_with(&format!("IllegalStateException: Property '{SOME_PROPERTY_NAME}' can not be retrieved from this EditorConfig.")),
        "{message}"
    );
}

#[test]
fn given_an_editor_config_from_which_an_existing_property_is_retrieved_then_return_the_value_of_that_property()
 {
    let editor_config = EditorConfig::new(vec![to_property_with_value(
        &sample_editor_config_property(),
        SOME_PROPERTY_VALUE,
    )]);
    assert_eq!(
        editor_config.get(&sample_editor_config_property()),
        SOME_PROPERTY_VALUE
    );
}

#[test]
fn given_an_empty_editor_config_and_add_a_property_with_default_value_then_the_default_value_can_be_retrieved_for_the_default_code_style()
 {
    let editor_config =
        EditorConfig::default().add_properties_with_default_value_if_missing(&[property_ref(
            sample_editor_config_property(),
        )]);
    assert_eq!(
        editor_config.get(&sample_editor_config_property()),
        SOME_PROPERTY_VALUE_KTLINT_OFFICIAL
    );
}

#[test]
fn given_an_editor_config_with_a_defined_code_style_and_add_a_property_with_default_value_then_the_default_value_can_be_retrieved_for_the_default_code_style()
 {
    for (code_style_value, expected_value) in [
        ("android_studio", SOME_PROPERTY_VALUE_ANDROID_STUDIO),
        ("intellij_idea", SOME_PROPERTY_VALUE_INTELLIJ_IDEA),
        ("ktlint_official", SOME_PROPERTY_VALUE_KTLINT_OFFICIAL),
    ] {
        let editor_config = EditorConfig::new(vec![to_property_with_value(
            &*CODE_STYLE_PROPERTY,
            code_style_value,
        )])
        .add_properties_with_default_value_if_missing(&[property_ref(
            sample_editor_config_property(),
        )]);
        assert_eq!(
            editor_config.get(&sample_editor_config_property()),
            expected_value,
            "code style {code_style_value}"
        );
    }
}

#[test]
fn given_an_editor_config_containing_a_certain_property_with_a_non_default_value_and_add_the_same_property_again_with_default_value_then_the_non_default_value_is_not_overwritten()
 {
    let editor_config = EditorConfig::new(vec![to_property_with_value(
        &sample_editor_config_property(),
        SOME_PROPERTY_VALUE,
    )])
    .add_properties_with_default_value_if_missing(&[property_ref(sample_editor_config_property())]);
    assert_eq!(
        editor_config.get(&sample_editor_config_property()),
        SOME_PROPERTY_VALUE
    );
}

#[test]
fn given_an_editor_config_from_which_a_deprecated_error_level_property_is_retrieved_then_thrown_an_exception()
 {
    let deprecated = EditorConfigProperty {
        deprecation_error: Some("some-deprecation-message"),
        ..sample_editor_config_property()
    };
    let editor_config = EditorConfig::default()
        .add_properties_with_default_value_if_missing(&[property_ref(deprecated.clone())]);
    let message = panic_message(|| {
        editor_config.get(&deprecated);
    });
    assert!(
        message.ends_with(&format!(
            "Property '{SOME_PROPERTY_NAME}' is disallowed: some-deprecation-message"
        )),
        "{message}"
    );
}

#[test]
fn given_an_editor_config_from_which_a_deprecated_warning_level_property_is_retrieved_then_do_not_throw_an_exception()
 {
    let deprecated = EditorConfigProperty {
        deprecation_warning: Some("some-deprecation-message"),
        ..sample_editor_config_property()
    };
    let editor_config = EditorConfig::default()
        .add_properties_with_default_value_if_missing(&[property_ref(deprecated.clone())]);
    editor_config.get(&sample_editor_config_property());
    editor_config.get(&deprecated);
}

#[test]
fn given_an_editor_config_containing_a_property_then_contains_returns_true_when_that_property_is_retrieved()
 {
    let editor_config = EditorConfig::new(vec![to_property_with_value(
        &sample_editor_config_property(),
        SOME_PROPERTY_VALUE,
    )]);
    assert!(editor_config.contains(SOME_PROPERTY_NAME));
}

#[test]
fn given_an_editor_config_then_contains_returns_false_when_a_non_existent_property_is_retrieved() {
    assert!(!EditorConfig::default().contains(SOME_PROPERTY_NAME));
}

#[test]
fn given_an_editor_config_containing_some_properties_then_mapping_of_all_properties_is_possible() {
    let property = |name: &str, value: &str| EditorConfigProperty {
        ktlint_official_code_style_default_value: value.to_owned(),
        ..sample(name)
    };
    let editor_config = EditorConfig::default().add_properties_with_default_value_if_missing(&[
        property_ref(property("property-1", "value-1")),
        property_ref(property("property-2", "value-1")),
    ]);
    let actual = editor_config.map(|p| {
        (
            p.name().to_uppercase(),
            p.source_value().unwrap().to_uppercase(),
        )
    });
    assert_eq!(
        actual,
        [
            ("PROPERTY-1".to_owned(), "VALUE-1".to_owned()),
            ("PROPERTY-2".to_owned(), "VALUE-1".to_owned())
        ]
    );
}

#[test]
fn given_an_editor_config_to_which_properties_are_added_with_the_same_name_but_different_identities_than_those_properties_can_not_be_loaded_in_the_same_editor_config()
 {
    let message = panic_message(|| {
        EditorConfig::default().add_properties_with_default_value_if_missing(&[
            property_ref(with_default(SOME_PROPERTY_VALUE_ANDROID_STUDIO)),
            property_ref(with_default(SOME_PROPERTY_VALUE_INTELLIJ_IDEA)),
        ]);
    });
    let expected = format!(
        "IllegalArgumentException: Found multiple editorconfig properties with name '{SOME_PROPERTY_NAME}' but having distinct identities:"
    );
    assert!(message.starts_with(&expected), "{message}");
}

#[test]
fn given_two_editorconfig_properties_with_the_same_name_but_different_identities_than_those_properties_can_not_be_loaded_in_the_same_editor_config()
 {
    let message = panic_message(|| {
        EditorConfig::default().filter_by(&[
            property_ref(with_default(SOME_PROPERTY_VALUE_ANDROID_STUDIO)),
            property_ref(with_default(SOME_PROPERTY_VALUE_INTELLIJ_IDEA)),
        ]);
    });
    assert!(
        message.contains(&format!(
            "Found multiple editorconfig properties with name '{SOME_PROPERTY_NAME}'"
        )),
        "{message}"
    );
}

#[test]
fn given_an_editorconfig_containing_a_property_and_a_filter_by_for_a_property_with_the_same_name_is_given_but_with_different_identity_then_the_existing_property_is_not_overwritten()
 {
    let editor_config = EditorConfig::new(vec![to_property_with_value(
        &sample_editor_config_property(),
        SOME_PROPERTY_VALUE_ANDROID_STUDIO,
    )])
    .filter_by(&[property_ref(with_default(
        SOME_PROPERTY_VALUE_INTELLIJ_IDEA,
    ))]);
    assert_eq!(
        editor_config.get(&sample_editor_config_property()),
        SOME_PROPERTY_VALUE_ANDROID_STUDIO
    );
}

#[test]
fn given_an_editorconfig_containing_a_property_for_which_the_name_of_the_property_is_not_identical_to_the_name_of_the_property_type_then_the_property_can_be_retrieved_by_name_and_type_combination()
 {
    let editor_config = EditorConfig::new(vec![
        to_property_with_value(&sample("property-1"), SOME_PROPERTY_VALUE_ANDROID_STUDIO),
        to_property_with_value(&sample("property-2"), SOME_PROPERTY_VALUE_INTELLIJ_IDEA),
    ]);
    let actual = editor_config
        .get_editor_config_value_or_null(sample_editor_config_property().type_, "property-2");
    assert_eq!(actual.as_deref(), Some(SOME_PROPERTY_VALUE_INTELLIJ_IDEA));
}

#[test]
fn given_an_editorconfig_containing_a_property_for_which_a_property_mapper_is_defined_then_the_property_mapper_is_called()
 {
    let with_mapper = EditorConfigProperty {
        property_mapper: Some(|_, _| Some(SOME_PROPERTY_VALUE_KTLINT_OFFICIAL.to_owned())),
        ..with_default(SOME_PROPERTY_VALUE)
    };
    let editor_config = EditorConfig::default()
        .add_properties_with_default_value_if_missing(&[property_ref(with_mapper.clone())]);
    assert_eq!(
        editor_config.get(&with_mapper),
        SOME_PROPERTY_VALUE_KTLINT_OFFICIAL
    );
}

#[test]
fn given_an_editorconfig_containing_a_property_for_which_the_value_is_unset_then_return_its_default_value()
 {
    let editor_config = EditorConfig::new(vec![to_property_with_value(
        &sample_editor_config_property(),
        "unset",
    )]);
    assert_eq!(
        editor_config.get(&sample_editor_config_property()),
        SOME_PROPERTY_VALUE_KTLINT_OFFICIAL
    );
}

fn rule_execution_property(name: &str) -> EditorConfigProperty<RuleExecution> {
    let type_: &'static PropertyType<RuleExecution> = Box::leak(Box::new(PropertyType {
        name: Box::leak(name.to_owned().into_boxed_str()),
        description: "",
        parser: safe_enum_value_parser::<RuleExecution>,
        possible_values: &["enabled", "disabled"],
        lower_casing: true,
    }));
    EditorConfigProperty::new(type_, RuleExecution::Enabled)
}

#[test]
fn given_an_editorconfig_containing_a_property_with_an_invalid_source_value_then_return_its_default_value()
 {
    let some_editor_config_property = rule_execution_property("some-editor-config-property");
    let editor_config = EditorConfig::new(vec![to_property_with_value(
        &some_editor_config_property,
        "some-invalid-value",
    )]);
    assert_eq!(
        editor_config.get(&some_editor_config_property),
        some_editor_config_property.default_value
    );
}

#[test]
fn given_an_editorconfig_containing_a_property_with_undefined_type_then_retrieving_that_property_via_the_edit_config_may_not_result_in_an_exception()
 {
    let (name_1, name_2) = ("ktlint_test_rule-1", "ktlint_test_rule-2");
    let (property_1, property_2) = (
        rule_execution_property(name_1),
        rule_execution_property(name_2),
    );
    let fs = TestFileSystem::new();
    fs.write_root_editor_config_file(&format!(
        "[*.{{kt,kts}}]\n{name_1} = disabled\n{name_2} = disabled"
    ));
    // Only the type of the second property is registered.
    let defaults = EditorConfigDefaults::load(Some(&fs.resolve("")), &[property_2.type_]).unwrap();
    let properties: Vec<Property> = defaults
        .value
        .sections()
        .iter()
        .flat_map(|s| s.properties().iter().cloned())
        .collect();
    let find = |name: &str| {
        properties
            .iter()
            .find(|p| p.name() == name)
            .unwrap_or_else(|| panic!("{name} not loaded"))
    };
    assert!(
        find(name_1).type_().is_none(),
        "{name_1} should have an undefined type"
    );
    assert!(
        find(name_2).type_().is_some(),
        "{name_2} should have a defined type"
    );

    let editor_config = EditorConfig::new(properties);
    assert_eq!(editor_config.get(&property_1), RuleExecution::Disabled);
    assert_eq!(editor_config.get(&property_2), RuleExecution::Disabled);
}
