//! `FunctionSignatureRule`'s companion object (its `.editorconfig` properties) and `FunctionBodyExpressionWrapping`.

use std::sync::LazyLock;

use ktrs_editorconfig::property_type::{enum_value_parser, positive_int_value_parser};
use ktrs_editorconfig::{EnumValue, PropertyType};

use crate::editorconfig::EditorConfigProperty;

const FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_UNSET: i32 = i32::MAX;

pub static FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_TYPE: PropertyType<i32> = PropertyType {
    name: "ktlint_function_signature_rule_force_multiline_when_parameter_count_greater_or_equal_than",
    description: "Force wrapping the parameters of the function signature in case it contains at least the specified number of \
                  parameters even in case the entire function signature would fit on a single line. By default this parameter is \
                  not enabled.",
    parser: positive_int_value_parser,
    possible_values: &["1", "2", "3", "4", "5", "6", "7", "8", "9", "unset"],
    lower_casing: true,
};

pub static FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY: LazyLock<EditorConfigProperty<i32>> =
    LazyLock::new(|| EditorConfigProperty {
        ktlint_official_code_style_default_value: 2,
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

pub static FUNCTION_BODY_EXPRESSION_WRAPPING_PROPERTY_TYPE: PropertyType<FunctionBodyExpressionWrapping> = PropertyType {
    name: "ktlint_function_signature_body_expression_wrapping",
    description: "Determines how to wrap the body of function in case it is an expression. Use 'default' to wrap the body \
                  expression only when the first line of the expression does not fit on the same line as the function signature. \
                  Use 'multiline' to force wrapping of body expressions that consists of multiple line. Use 'always' to force \
                  wrapping of body expression always.",
    parser: enum_value_parser::<FunctionBodyExpressionWrapping>,
    possible_values: &["default", "multiline", "always"],
    lower_casing: true,
};

pub static FUNCTION_BODY_EXPRESSION_WRAPPING_PROPERTY: LazyLock<EditorConfigProperty<FunctionBodyExpressionWrapping>> =
    LazyLock::new(|| EditorConfigProperty {
        ktlint_official_code_style_default_value: FunctionBodyExpressionWrapping::Multiline,
        ..EditorConfigProperty::new(&FUNCTION_BODY_EXPRESSION_WRAPPING_PROPERTY_TYPE, FunctionBodyExpressionWrapping::Default)
    });

/// Code style to be used while linting and formatting. Note that the `EnumValueParser` requires values to be lowercase.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FunctionBodyExpressionWrapping {
    /// Keep the first line of the body expression on the same line as the function signature if max line length is not exceeded.
    Default,
    /// Force the body expression to start on a separate line in case it is a multiline expression. A single line body expression
    /// is wrapped only when it does not fit on the same line as the function signature.
    Multiline,
    /// Always force the body expression to start on a separate line.
    Always,
}

impl EnumValue for FunctionBodyExpressionWrapping {
    const ENUM_TYPE_NAME: &'static str =
        "io.github.ktlint.core.ruleset.standard.rules.FunctionSignatureRule$FunctionBodyExpressionWrapping";
    const ENTRIES: &'static [Self] =
        &[FunctionBodyExpressionWrapping::Default, FunctionBodyExpressionWrapping::Multiline, FunctionBodyExpressionWrapping::Always];

    fn name(self) -> &'static str {
        match self {
            FunctionBodyExpressionWrapping::Default => "default",
            FunctionBodyExpressionWrapping::Multiline => "multiline",
            FunctionBodyExpressionWrapping::Always => "always",
        }
    }
}

crate::enum_property_value_type!(FunctionBodyExpressionWrapping);
