//! `ClassSignatureRule`'s companion object.

use std::sync::LazyLock;

use ktrs_editorconfig::PropertyType;
use ktrs_editorconfig::property_type::positive_int_value_parser;

use crate::editorconfig::EditorConfigProperty;

const FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_UNSET: i32 = i32::MAX;

pub static FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_TYPE: PropertyType<i32> = PropertyType {
    name: "ktlint_class_signature_rule_force_multiline_when_parameter_count_greater_or_equal_than",
    description: "Force wrapping of the parameters of the class signature in case it contains at least the specified number of \
                  parameters, even in case the entire class signature would fit on a single line. Use value 'unset' to disable this \
                  setting.",
    parser: positive_int_value_parser,
    possible_values: &["1", "2", "3", "4", "5", "6", "7", "8", "9", "unset"],
    lower_casing: true,
};

pub static FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY: LazyLock<EditorConfigProperty<i32>> =
    LazyLock::new(|| EditorConfigProperty {
        ktlint_official_code_style_default_value: 1,
        property_mapper: Some(|property, _| {
            if property.is_some_and(|p| p.is_unset()) {
                Some(FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_UNSET)
            } else {
                property.and_then(|p| p.get_value_as(&FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_TYPE))
            }
        }),
        property_writer: |property| {
            if *property == FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_UNSET {
                "unset".to_owned()
            } else {
                property.to_string()
            }
        },
        ..EditorConfigProperty::new(
            &FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_TYPE,
            FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_UNSET,
        )
    });
