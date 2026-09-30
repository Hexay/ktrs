//! Ports of ktlint-rule-engine-core `editorconfig/` property tests: `MaxLineLengthEditorConfigPropertyTest`,
//! `IndentSizeEditorConfigPropertyTest`, `CodeStyleEditorConfigPropertyTest`, `RuleExecutionEditorConfigPropertyTest`,
//! `SafeEnumValueParserTest` and `CommaSeparatedListValueParserTest`. Parameterized cases loop.

use ktrs_editorconfig::{EnumValue, PropertyType};
use ktrs_lint::editorconfig::{
    CODE_STYLE_PROPERTY, CodeStyleValue, INDENT_SIZE_PROPERTY, MAX_LINE_LENGTH_PROPERTY,
    RULE_EXECUTION_PROPERTY_TYPE, RuleExecution, comma_separated_list_value_parser,
    safe_enum_value_parser, to_property_with_value,
};

const CODE_STYLES: [CodeStyleValue; 3] = [
    CodeStyleValue::AndroidStudio,
    CodeStyleValue::IntellijIdea,
    CodeStyleValue::KtlintOfficial,
];
const MAX_LINE_LENGTH_DEFAULTS: [(CodeStyleValue, i32); 3] = [
    (CodeStyleValue::AndroidStudio, 100),
    (CodeStyleValue::IntellijIdea, i32::MAX),
    (CodeStyleValue::KtlintOfficial, 140),
];

fn max_line_length(value: Option<&str>, code_style: CodeStyleValue) -> Option<i32> {
    let mapper = MAX_LINE_LENGTH_PROPERTY.property_mapper.unwrap();
    let property = value.map(|v| to_property_with_value(&*MAX_LINE_LENGTH_PROPERTY, v));
    mapper(property.as_ref(), code_style)
}

#[test]
fn max_line_length_property_mapper_given_a_null_property_then_the_property_mapper_returns_the_default_value_of_the_code_style()
 {
    for (code_style, expected) in MAX_LINE_LENGTH_DEFAULTS {
        assert_eq!(
            max_line_length(None, code_style),
            Some(expected),
            "{code_style:?}"
        );
    }
}

#[test]
fn max_line_length_property_mapper_given_a_property_which_is_unset_then_the_property_mapper_returns_the_default_value_of_the_code_style()
 {
    for (code_style, expected) in MAX_LINE_LENGTH_DEFAULTS {
        assert_eq!(
            max_line_length(Some("unset"), code_style),
            Some(expected),
            "{code_style:?}"
        );
    }
}

#[test]
fn max_line_length_property_mapper_given_a_valid_string_value_then_the_property_mapper_returns_the_integer_value()
 {
    for code_style in CODE_STYLES {
        assert_eq!(
            max_line_length(Some("123"), code_style),
            Some(123),
            "{code_style:?}"
        );
    }
}

#[test]
fn max_line_length_property_mapper_given_the_value_off_then_the_property_mapper_returns_integer_max_value_which_denotes_that_no_maximum_line_length_is_set()
 {
    for code_style in CODE_STYLES {
        assert_eq!(
            max_line_length(Some("off"), code_style),
            Some(i32::MAX),
            "{code_style:?}"
        );
    }
}

#[test]
fn max_line_length_property_mapper_given_the_value_minus_1_then_the_property_mapper_returns_integer_max_value_which_denotes_that_no_maximum_line_length_is_set()
 {
    for code_style in CODE_STYLES {
        assert_eq!(
            max_line_length(Some("-1"), code_style),
            Some(i32::MAX),
            "{code_style:?}"
        );
    }
}

#[test]
fn max_line_length_property_mapper_given_an_invalid_value_then_the_property_mapper_returns_the_default_value_of_the_code_style()
 {
    for (code_style, expected) in MAX_LINE_LENGTH_DEFAULTS {
        assert_eq!(
            max_line_length(Some("some-invalid-value"), code_style),
            Some(expected),
            "{code_style:?}"
        );
    }
}

#[test]
fn max_line_length_given_a_property_with_an_integer_value_than_write_that_property() {
    for (input_value, expected_output_value) in
        [(-1, "off"), (0, "off"), (1, "1"), (i32::MAX, "off")]
    {
        assert_eq!(
            (MAX_LINE_LENGTH_PROPERTY.property_writer)(&input_value),
            expected_output_value,
            "{input_value}"
        );
    }
}

fn indent_size(value: Option<&str>, code_style: CodeStyleValue) -> Option<i32> {
    let mapper = INDENT_SIZE_PROPERTY.property_mapper.unwrap();
    let property = value.map(|v| to_property_with_value(&*INDENT_SIZE_PROPERTY, v));
    mapper(property.as_ref(), code_style)
}

#[test]
fn indent_size_given_a_null_property_then_the_property_mapper_returns_4_which_is_set_as_the_default_value()
 {
    for code_style in CODE_STYLES {
        assert_eq!(indent_size(None, code_style), Some(4));
    }
}

#[test]
fn indent_size_given_a_property_which_is_unset_then_the_property_mapper_returns_minus_1_as_default_value()
 {
    for code_style in CODE_STYLES {
        assert_eq!(indent_size(Some("unset"), code_style), Some(-1));
    }
}

#[test]
fn indent_size_given_a_valid_string_value_then_the_property_mapper_returns_the_integer_value() {
    for code_style in CODE_STYLES {
        assert_eq!(indent_size(Some("123"), code_style), Some(123));
    }
}

#[test]
fn indent_size_given_value_tab_then_the_property_mapper_returns_4_which_is_set_as_the_default_value()
 {
    for code_style in CODE_STYLES {
        assert_eq!(indent_size(Some("tab"), code_style), Some(4));
    }
}

#[test]
fn given_a_code_style_property() {
    for (value, expected) in [
        ("ktlint_official", CodeStyleValue::KtlintOfficial),
        (" ktlint_official", CodeStyleValue::KtlintOfficial),
        ("ktlint_official ", CodeStyleValue::KtlintOfficial),
        (" ktlint_official ", CodeStyleValue::KtlintOfficial),
        ("intellij_idea", CodeStyleValue::IntellijIdea),
        (" intellij_idea", CodeStyleValue::IntellijIdea),
        ("intellij_idea ", CodeStyleValue::IntellijIdea),
        (" intellij_idea ", CodeStyleValue::IntellijIdea),
        ("android_studio", CodeStyleValue::AndroidStudio),
        (" android_studio", CodeStyleValue::AndroidStudio),
        ("android_studio ", CodeStyleValue::AndroidStudio),
        (" android_studio ", CodeStyleValue::AndroidStudio),
    ] {
        assert_eq!(
            CODE_STYLE_PROPERTY.type_.parse(Some(value)).into_parsed(),
            Some(expected),
            "[{value}]"
        );
    }
}

#[test]
fn given_a_rule_execution_property_for_which_the_value() {
    for (value, expected) in [
        ("enabled", RuleExecution::Enabled),
        (" enabled", RuleExecution::Enabled),
        ("enabled ", RuleExecution::Enabled),
        (" enabled ", RuleExecution::Enabled),
        ("disabled", RuleExecution::Disabled),
        (" disabled", RuleExecution::Disabled),
        ("disabled ", RuleExecution::Disabled),
        (" disabled ", RuleExecution::Disabled),
    ] {
        assert_eq!(
            RULE_EXECUTION_PROPERTY_TYPE
                .parse(Some(value))
                .into_parsed(),
            Some(expected),
            "[{value}]"
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SomePropertyType {
    Value1,
    Value2,
}

impl EnumValue for SomePropertyType {
    const ENUM_TYPE_NAME: &'static str = "SomePropertyType";
    const ENTRIES: &'static [Self] = &[SomePropertyType::Value1, SomePropertyType::Value2];

    fn name(self) -> &'static str {
        match self {
            SomePropertyType::Value1 => "value1",
            SomePropertyType::Value2 => "value2",
        }
    }
}

#[test]
fn safe_enum_value_parser_given_a_rule_execution_property_for_which_the_value() {
    let property_type = PropertyType {
        name: "some-property-type",
        description: "",
        parser: safe_enum_value_parser::<SomePropertyType>,
        possible_values: &["value1", "value2"],
        lower_casing: true,
    };
    assert_eq!(
        property_type.parse(Some(" value2 ")).into_parsed(),
        Some(SomePropertyType::Value2)
    );
}

static COMMA_SEPARATED_TYPE: PropertyType<Vec<String>> = PropertyType {
    name: "some-property-type",
    description: "",
    parser: comma_separated_list_value_parser,
    possible_values: &[],
    lower_casing: true,
};

fn sorted(value: &str) -> Vec<String> {
    let mut parsed = COMMA_SEPARATED_TYPE
        .parse(Some(value))
        .into_parsed()
        .unwrap();
    parsed.sort();
    parsed
}

#[test]
fn given_a_comma_separated_list_property_with_value_unset() {
    assert!(COMMA_SEPARATED_TYPE.parse(Some("unset")).is_unset());
}

#[test]
fn given_a_comma_separated_list_property_with_a_single_value() {
    assert_eq!(sorted("some-value-1"), ["some-value-1"]);
}

#[test]
fn given_a_comma_separated_list_property_with_a_multiple_values() {
    assert_eq!(
        sorted("some-value-1,some-value-2"),
        ["some-value-1", "some-value-2"]
    );
}

#[test]
fn given_a_comma_separated_list_property_with_a_multiple_values_and_redundant_space_before_or_after_value()
 {
    assert_eq!(
        sorted(" some-value-1 , some-value-2 "),
        ["some-value-1", "some-value-2"]
    );
}
