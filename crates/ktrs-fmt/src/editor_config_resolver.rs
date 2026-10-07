//! Port of ktfmt's `cli/EditorConfigResolver.kt` (v0.64). ec4rs finds and cascades the files; the
//! value parsing here follows ec4j's property types (`getValue(type, default, warn)`: a missing or
//! invalid value yields the default, a valid one its parsed value, which may be null).

use std::path::Path;

use ec4rs::Properties;
use crate::{FormattingOptions, TrailingCommaManagementStrategy};

/// A parsed property: `None` when missing or invalid, `Some(None)` for a valid null (`off`, `tab`).
type Value<T> = Option<Option<T>>;

pub fn resolve_formatting_options(file: &Path, base_options: &FormattingOptions) -> FormattingOptions {
    let absolute = std::path::absolute(file).unwrap_or_else(|_| file.to_path_buf());
    match ec4rs::properties_of(&absolute) {
        Ok(properties) => resolve(&properties, base_options),
        // ec4j logs unreadable or malformed files and carries on.
        Err(_) => *base_options,
    }
}

fn resolve(properties: &Properties, base_options: &FormattingOptions) -> FormattingOptions {
    let get = |value: Value<i32>, default: Option<i32>| match value {
        Some(parsed) => parsed,
        None => default,
    };
    // `max_line_length` may return null to indicate 'off', in which case we keep the base maxWidth
    let max_width = get(max_line_length(properties), Some(base_options.max_width)).unwrap_or(base_options.max_width);
    // `indent_size` may return null to indicate 'tab', in which case we defer to `tab_width`
    // `ij_kotlin_indent_size` takes priority over `indent_size` when present
    let block_indent = get(positive_int(properties, "ij_kotlin_indent_size"), None)
        .or_else(|| get(indent_size(properties), Some(base_options.block_indent)))
        .or_else(|| get(positive_int(properties, "tab_width"), Some(base_options.block_indent)))
        .expect("tab_width falls back to a non-null default");
    let continuation_indent = get(positive_int(properties, "ij_kotlin_continuation_indent_size"), None)
        .or_else(|| get(positive_int(properties, "ij_continuation_indent_size"), Some(base_options.continuation_indent)))
        .expect("ij_continuation_indent_size falls back to a non-null default");
    let trailing_comma_management_strategy =
        trailing_comma_strategy(properties).unwrap_or(base_options.trailing_comma_management_strategy);
    FormattingOptions {
        max_width,
        block_indent,
        continuation_indent,
        trailing_comma_management_strategy,
        ..*base_options
    }
}

fn raw<'a>(properties: &'a Properties, key: &str) -> Option<&'a str> {
    properties.get_raw_for_key(key).filter_unset().into_option()
}

/// ec4j's `POSITIVE_INT_VALUE_PARSER` (lowercasing, where ktfmt's types do it, can't change an int).
fn positive_int(properties: &Properties, key: &str) -> Value<i32> {
    parse_positive_int(raw(properties, key)?).map(Some)
}

fn parse_positive_int(value: &str) -> Option<i32> {
    value.parse::<i32>().ok().filter(|&i| i > 0)
}

fn max_line_length(properties: &Properties) -> Value<i32> {
    match raw(properties, "max_line_length")? {
        "off" => Some(None),
        value => parse_positive_int(value).map(Some),
    }
}

/// With `indent_style = tab` and no `indent_size`, ec4j resolves `indent_size` to `tab`.
fn indent_size(properties: &Properties) -> Value<i32> {
    let value = match raw(properties, "indent_size") {
        Some(value) => value,
        None if raw(properties, "indent_style") == Some("tab") => "tab",
        None => return None,
    };
    match value {
        "tab" => Some(None),
        value => parse_positive_int(value).map(Some),
    }
}

fn trailing_comma_strategy(properties: &Properties) -> Option<TrailingCommaManagementStrategy> {
    match raw(properties, "ktfmt_trailing_comma_management_strategy")?.to_lowercase().as_str() {
        "none" => Some(TrailingCommaManagementStrategy::None),
        "only_add" => Some(TrailingCommaManagementStrategy::OnlyAdd),
        "complete" => Some(TrailingCommaManagementStrategy::Complete),
        _ => None,
    }
}
