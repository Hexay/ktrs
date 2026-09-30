//! Port of ktlint-rule-engine `internal/EditorConfigLoaderTest.kt`, second half: null path, non-Kotlin
//! files, stdin, defaults and overrides.

mod engine_editorconfig_support;

use std::sync::LazyLock;

use engine_editorconfig_support::{
    TestFileSystem, assert_contains, assert_contains_exactly_in_any_order, assert_not_contains,
    convert_to_property_values,
};
use ktrs_editorconfig::PropertyType;
use ktrs_editorconfig::property_type::identity_value_parser;
use ktrs_lint::editorconfig::{
    EditorConfigProperty, INDENT_SIZE_PROPERTY, INSERT_FINAL_NEWLINE_PROPERTY, PropertyRef,
};
use ktrs_lint::engine::editor_config_loader::{EditorConfigLoader, EditorConfigLoaderEc4j};
use ktrs_lint::{EditorConfigDefaults, EditorConfigOverride};

const SOME_PROPERTY_NAME: &str = "some-property-name";
const SOME_PROPERTY_VALUE_1: &str = "some-property-value-1";
const SOME_PROPERTY_VALUE_2: &str = "some-property-value-2";

static SOME_PROPERTY_TYPE: PropertyType<String> = PropertyType {
    name: SOME_PROPERTY_NAME,
    description: "",
    parser: identity_value_parser,
    possible_values: &[SOME_PROPERTY_VALUE_1, SOME_PROPERTY_VALUE_2],
    lower_casing: false,
};

static SOME_EDITOR_CONFIG_PROPERTY: LazyLock<EditorConfigProperty<String>> = LazyLock::new(|| {
    EditorConfigProperty::new(&SOME_PROPERTY_TYPE, SOME_PROPERTY_VALUE_1.to_owned())
});

/// What KtLint always adds for its own processing.
const INTERNAL_PROPERTIES: [&str; 6] = [
    "end_of_line = lf",
    "ij_formatter_tags_enabled = false",
    "ij_formatter_off_tag = @formatter:off",
    "ij_formatter_on_tag = @formatter:on",
    "ktlint_code_style = ktlint_official",
    "ktlint_experimental = disabled",
];

fn create_editor_config_loader(
    defaults: EditorConfigDefaults,
    editor_config_override: EditorConfigOverride,
) -> EditorConfigLoader {
    EditorConfigLoader::new(
        EditorConfigLoaderEc4j::new(&[]),
        defaults,
        editor_config_override,
    )
}

fn default_loader() -> EditorConfigLoader {
    create_editor_config_loader(EditorConfigDefaults::empty(), EditorConfigOverride::empty())
}

fn override_loader(property: PropertyRef, value: &str) -> EditorConfigLoader {
    create_editor_config_loader(
        EditorConfigDefaults::empty(),
        EditorConfigOverride::from(vec![(property, Some(value.to_owned()))]),
    )
}

fn insert_final_newline() -> PropertyRef {
    PropertyRef::from(&*INSERT_FINAL_NEWLINE_PROPERTY)
}

#[test]
fn given_a_null_file_path_given_no_default_and_no_override_properties_then_return_only_the_properties_required_for_internal_processing_by_ktlint()
 {
    let actual = convert_to_property_values(&default_loader().load(None).unwrap());
    assert_contains_exactly_in_any_order(&actual, &INTERNAL_PROPERTIES);
}

#[test]
fn given_a_null_file_path_given_some_override_properties_then_return_the_override_properties() {
    let actual = convert_to_property_values(
        &override_loader(insert_final_newline(), "true")
            .load(None)
            .unwrap(),
    );
    assert_contains(&actual, &["insert_final_newline = true"]);
}

#[test]
fn given_a_null_file_path_given_some_properties_in_an_editorconfig_file_then_return_the_properties_from_this_file()
 {
    let fs = TestFileSystem::new();
    fs.write_root_editor_config_file("[*.{kt,kts}]\nsome_property = some_value");
    let actual =
        convert_to_property_values(&default_loader().load(Some(&fs.resolve(".kt"))).unwrap());
    assert_contains(&actual, &["some_property = some_value"]);
}

#[test]
fn given_a_null_file_path_given_some_properties_in_an_editorconfig_file_which_is_passed_in_as_default_editorconfig_then_return_the_properties_from_this_file()
 {
    let fs = TestFileSystem::new();
    fs.write_editor_config_file("some/dir", "[*.{kt,kts}]\nsome_property = some_value");
    let editor_config_defaults = EditorConfigDefaults::load(
        Some(&fs.resolve("some/dir/.editorconfig")),
        &[&SOME_PROPERTY_TYPE],
    )
    .unwrap();
    let loader = create_editor_config_loader(editor_config_defaults, EditorConfigOverride::empty());
    let actual = convert_to_property_values(&loader.load(Some(&fs.resolve(".kt"))).unwrap());
    assert_contains(&actual, &["some_property = some_value"]);
}

#[test]
fn given_a_file_path_with_a_non_kotlin_extension_given_no_default_and_no_override_properties_then_return_only_the_properties_required_for_internal_processing_by_ktlint()
 {
    let fs = TestFileSystem::new();
    let actual = convert_to_property_values(
        &default_loader()
            .load(Some(&fs.resolve("test.java")))
            .unwrap(),
    );
    assert_contains_exactly_in_any_order(&actual, &INTERNAL_PROPERTIES);
}

#[test]
fn given_a_file_path_with_a_non_kotlin_extension_given_some_override_properties_then_return_the_override_properties()
 {
    let fs = TestFileSystem::new();
    let actual = convert_to_property_values(
        &override_loader(insert_final_newline(), "true")
            .load(Some(&fs.resolve("test.java")))
            .unwrap(),
    );
    assert_contains(&actual, &["insert_final_newline = true"]);
}

#[test]
fn given_a_file_path_with_a_non_kotlin_extension_given_some_kotlin_properties_in_an_editorconfig_file_then_do_not_return_the_properties_from_this_file()
 {
    let fs = TestFileSystem::new();
    fs.write_root_editor_config_file("[*.{kt,kts}]\nsome_property = some_value");
    let actual = convert_to_property_values(
        &default_loader()
            .load(Some(&fs.resolve("test.java")))
            .unwrap(),
    );
    assert_not_contains(&actual, &["some_property = some_value"]);
}

#[test]
fn given_input_from_stdin_given_no_default_and_no_override_properties_then_return_only_the_properties_required_for_internal_processing_by_ktlint()
 {
    let fs = TestFileSystem::new();
    let actual =
        convert_to_property_values(&default_loader().load(Some(&fs.resolve(".kt"))).unwrap());
    assert_contains_exactly_in_any_order(&actual, &INTERNAL_PROPERTIES);
}

#[test]
fn given_input_from_stdin_given_some_override_properties_then_return_the_override_properties() {
    let fs = TestFileSystem::new();
    fs.write_root_editor_config_file("[*.{kt,kts}]\nsome_property_1 = some_value_1");
    let actual = convert_to_property_values(
        &override_loader(insert_final_newline(), "true")
            .load(Some(&fs.resolve(".kt")))
            .unwrap(),
    );
    assert_contains(&actual, &["insert_final_newline = true"]);
}

#[test]
fn given_input_from_stdin_given_some_properties_in_an_editorconfig_file_then_return_the_properties_from_this_file()
 {
    let fs = TestFileSystem::new();
    fs.write_root_editor_config_file("[*.{kt,kts}]\nsome_property = some_value");
    let actual =
        convert_to_property_values(&default_loader().load(Some(&fs.resolve(".kt"))).unwrap());
    assert_contains(&actual, &["some_property = some_value"]);
}

#[test]
fn given_a_project_with_editorconfig_properties_root_true_and_override_properties_then_ignore_properties_from_root_dir_but_apply_the_override_properties()
 {
    let fs = TestFileSystem::new();
    fs.write_root_editor_config_file("root = true\n[*]\nsome_property_1 = some_value_1");
    fs.write_editor_config_file(
        "some-project-directory",
        "root = true\n[*.{kt,kts}]\nsome_property_1 = some_value_2\nsome_property_2 = some_value_2\nindent_size = 4",
    );
    let loader = override_loader(PropertyRef::from(&*INDENT_SIZE_PROPERTY), "2");
    let actual = convert_to_property_values(
        &loader
            .load(Some(&fs.resolve("some-project-directory/test.kt")))
            .unwrap(),
    );
    assert_contains(
        &actual,
        &[
            "some_property_1 = some_value_2",
            "some_property_2 = some_value_2",
            "indent_size = 2",
        ],
    );
}

#[test]
fn given_that_code_is_loaded_via_stdin_then_load_properties_from_override_and_properties_required_for_internal_processing_of_ktlint()
 {
    let fs = TestFileSystem::new();
    fs.write_root_editor_config_file(&format!(
        "root = true\n[*]\n{SOME_PROPERTY_NAME} = {SOME_PROPERTY_VALUE_1}"
    ));
    let loader = override_loader(
        PropertyRef::from(&*SOME_EDITOR_CONFIG_PROPERTY),
        SOME_PROPERTY_VALUE_2,
    );
    let actual = convert_to_property_values(&loader.load(None).unwrap());
    assert_contains(
        &actual,
        &[&format!("{SOME_PROPERTY_NAME} = {SOME_PROPERTY_VALUE_2}")],
    );
}

#[test]
fn given_that_the_indent_size_and_tab_width_property_have_same_value_and_the_indent_size_is_changed_in_the_override_properties_then_also_keep_the_tab_width_property_in_sync_with_the_indent_size()
 {
    let fs = TestFileSystem::new();
    fs.write_root_editor_config_file("root = true\n[*]\nindent_size = 5\ntab_width = 5");
    let loader = override_loader(PropertyRef::from(&*INDENT_SIZE_PROPERTY), "3");
    let actual = convert_to_property_values(&loader.load(Some(&fs.resolve("test.kt"))).unwrap());
    assert_contains(&actual, &["indent_size = 3", "tab_width = 3"]);
}

#[test]
fn should_add_property_from_override() {
    let fs = TestFileSystem::new();
    fs.write_root_editor_config_file("[*.{kt,kts}]\nsome_property_1 = some_value_1");
    let actual = convert_to_property_values(
        &override_loader(insert_final_newline(), "true")
            .load(Some(&fs.resolve("test.kt")))
            .unwrap(),
    );
    assert_contains(
        &actual,
        &[
            "some_property_1 = some_value_1",
            "insert_final_newline = true",
        ],
    );
}

#[test]
fn should_replace_property_from_override() {
    let fs = TestFileSystem::new();
    fs.write_root_editor_config_file("[*.{kt,kts}]\ninsert_final_newline = true");
    let actual = convert_to_property_values(
        &override_loader(insert_final_newline(), "false")
            .load(Some(&fs.resolve("test.kt")))
            .unwrap(),
    );
    assert_contains(&actual, &["insert_final_newline = false"]);
}
