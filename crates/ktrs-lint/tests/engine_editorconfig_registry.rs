//! Port of ktlint-rule-engine `api/EditorConfigPropertyRegistryTest.kt`. Properties compare by identity
//! (what `@Poko` equality of `EditorConfigProperty` covers).

use std::sync::LazyLock;

use ktrs_editorconfig::PropertyType;
use ktrs_editorconfig::property_type::positive_int_value_parser;
use ktrs_lint::editorconfig::{
    EditorConfigProperty, INDENT_SIZE_PROPERTY, PropertyRef, RuleExecution,
    create_rule_execution_editor_config_property, create_rule_set_execution_editor_config_property,
    rule_execution_property_name, rule_set_execution_property_name,
};
use ktrs_lint::engine::EditorConfigPropertyRegistry;
use ktrs_lint::{RuleId, RuleV2, RuleV2Provider};

const SOME_RULE_ID: &str = "some-ruleset:some-rule-name";
const SOME_RULE_SET_ID: &str = "some-ruleset";
const SOME_PROPERTY_NAME: &str = "some_property_name";

static SOME_PROPERTY_TYPE: PropertyType<i32> = PropertyType {
    name: SOME_PROPERTY_NAME,
    description: "some description",
    parser: positive_int_value_parser,
    possible_values: &["1", "2", "3"],
    lower_casing: true,
};

static SOME_PROPERTY: LazyLock<EditorConfigProperty<i32>> =
    LazyLock::new(|| EditorConfigProperty::new(&SOME_PROPERTY_TYPE, 1));

struct SomeTestRule;

impl RuleV2 for SomeTestRule {
    fn rule_id(&self) -> RuleId {
        RuleId("test:some-test-rule")
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*SOME_PROPERTY)]
    }
}

fn assert_same_property(actual: &PropertyRef, expected: &PropertyRef) {
    assert_eq!(actual.name(), expected.name());
    assert_eq!(actual.identity(), expected.identity());
}

#[test]
fn given_an_editor_config_property_without_rule_providers_given_a_property_name_defined_in_the_ktlint_rule_engine_core_module_then_return_that_property()
 {
    let actual = EditorConfigPropertyRegistry::new(&[])
        .find(&INDENT_SIZE_PROPERTY.name)
        .unwrap();
    assert_same_property(&actual, &PropertyRef::from(&*INDENT_SIZE_PROPERTY));
}

#[test]
fn given_an_editor_config_property_without_rule_providers_given_a_property_name_starting_with_ktlint_and_for_which_the_suffix_is_a_valid_rule_id_then_return_the_rule_execution_property_for_that_rule_id()
 {
    let actual = EditorConfigPropertyRegistry::new(&[])
        .find(&rule_execution_property_name(SOME_RULE_ID))
        .unwrap();
    assert_same_property(
        &actual,
        &create_rule_execution_editor_config_property(SOME_RULE_ID, RuleExecution::Enabled).into(),
    );
}

#[test]
fn given_an_editor_config_property_without_rule_providers_given_a_property_name_starting_with_ktlint_and_for_which_the_suffix_is_not_valid_rule_id_but_the_suffix_is_a_valid_rule_set_id_then_return_the_rule_execution_property_for_that_rule_set_id()
 {
    let actual = EditorConfigPropertyRegistry::new(&[])
        .find(&rule_set_execution_property_name(SOME_RULE_SET_ID))
        .unwrap();
    assert_same_property(
        &actual,
        &create_rule_set_execution_editor_config_property(SOME_RULE_SET_ID, RuleExecution::Enabled)
            .into(),
    );
}

#[test]
fn given_an_editor_config_property_without_rule_providers_given_a_property_name_that_can_not_be_found_by_name_in_the_editor_config_property_registry_then_throw_an_exception()
 {
    let actual = EditorConfigPropertyRegistry::new(&[]).find("some-unknown-property-name");
    let message = actual.expect_err("EditorConfigPropertyNotFoundException");
    assert!(
        message.starts_with(
            "Property with name 'some-unknown-property-name' is not found in any of given rules."
        ),
        "{message}"
    );
}

#[test]
fn given_a_property_name_defined_in_a_rule_provided_to_the_editor_config_property_registry_then_return_that_property()
 {
    let registry = EditorConfigPropertyRegistry::new(&[RuleV2Provider::new(|| {
        Box::new(SomeTestRule) as Box<dyn RuleV2>
    })]);
    let actual = registry.find(SOME_PROPERTY_NAME).unwrap();
    assert_same_property(&actual, &PropertyRef::from(&*SOME_PROPERTY));
}
