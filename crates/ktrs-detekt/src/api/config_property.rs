//! `detekt-api/.../ConfigProperty.kt`: `getValueOrDefault` per supported default type. The `by config(default)`
//! delegates are `OnceCell` fields of the rule, initialized with one of these on first read (the upstream
//! memoization), so a bad value fails where upstream throws: when the rule reads the property.

use super::config::{Config, Value};

/// `by config(default: String)`.
pub fn string(config: &dyn Config, property_name: &str, default_value: &str) -> String {
    match config.value_or_default(property_name, Value::String(default_value.to_owned())) {
        Value::String(value) => value,
        other => class_cast_exception(&other, "java.lang.String"),
    }
}

/// `by config(default: Boolean)`.
pub fn boolean(config: &dyn Config, property_name: &str, default_value: bool) -> bool {
    match config.value_or_default(property_name, Value::Boolean(default_value)) {
        Value::Boolean(value) => value,
        other => class_cast_exception(&other, "java.lang.Boolean"),
    }
}

/// `by config(default: Int)`.
pub fn int(config: &dyn Config, property_name: &str, default_value: i32) -> i32 {
    match config.value_or_default(property_name, Value::Int(default_value)) {
        Value::Int(value) => value,
        other => class_cast_exception(&other, "java.lang.Integer"),
    }
}

/// `by config(default: List<String>)`: `getListOrDefault`.
pub fn list(config: &dyn Config, property_name: &str, default_value: &[&str]) -> Vec<String> {
    let default_value = Value::List(default_value.iter().map(|s| Value::String((*s).to_owned())).collect());
    let value = match config.value_or_default(property_name, default_value) {
        Value::List(value) => value,
        other => class_cast_exception(&other, "java.util.List"),
    };
    value
        .into_iter()
        .map(|item| match item {
            Value::String(item) => item,
            _ => panic!("Only lists of strings are supported. '{property_name}' is invalid. "),
        })
        .collect()
}

fn class_cast_exception(value: &Value, target: &str) -> ! {
    panic!("java.lang.ClassCastException: {} cannot be cast to class {target}", value.kotlin_type_name())
}
