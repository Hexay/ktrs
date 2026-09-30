//! Port of ktlint-rule-engine-core `editorconfig/SafeEnumValueParser.kt` and `CommaSeparatedListValueParser.kt`.

use ktrs_editorconfig::{EnumValue, PropertyValue};

/// `SafeEnumValueParser`: ec4j's enum parser, but the value is trimmed first (an EOL comment after a
/// value leaves its separating space behind).
pub fn safe_enum_value_parser<E: EnumValue>(_name: &str, value: Option<&str>) -> PropertyValue<E> {
    match value {
        None => PropertyValue::invalid(
            None,
            format!("Cannot make enum {} out of null", E::ENUM_TYPE_NAME),
        ),
        Some(v) => match E::value_of(&v.trim().to_lowercase()) {
            Some(e) => PropertyValue::valid(value, Some(e)),
            None => PropertyValue::invalid(
                value,
                format!("Unexpected parsed \"{v}\" for enum {}", E::ENUM_TYPE_NAME),
            ),
        },
    }
}

/// `CommaSeparatedListValueParser`: a set of trimmed items (`unset` is the empty set).
pub fn comma_separated_list_value_parser(
    _name: &str,
    value: Option<&str>,
) -> PropertyValue<Vec<String>> {
    if value == Some("unset") {
        return PropertyValue::valid(value, Some(Vec::new()));
    }
    let mut set: Vec<String> = Vec::new();
    for item in value.unwrap_or("").split(',').map(str::trim) {
        if !set.iter().any(|s| s == item) {
            set.push(item.to_owned());
        }
    }
    PropertyValue::valid(value, Some(set))
}
