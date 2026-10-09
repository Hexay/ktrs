//! `detekt-core/.../config/YamlConfig.kt` and `BaseConfig.kt`.

use std::sync::Arc;

use super::yaml;
use crate::api::{Config, Value};

/// Config implementation using the yaml format. SubConfigurations can return sub maps according to the yaml
/// specification.
pub struct YamlConfig {
    properties: Vec<(String, Value)>,
    parent_path: Option<String>,
    parent: Option<Arc<dyn Config>>,
}

impl YamlConfig {
    /// `YamlConfig.load(reader)`: `Err` is the message of the exception upstream throws (a YAML error, or
    /// `InvalidConfigurationError` for a document that is not a map).
    pub fn load(text: &str) -> Result<Arc<YamlConfig>, String> {
        let properties = match yaml::load(text)? {
            None => Vec::new(),
            Some(Value::Map(properties)) => properties,
            Some(_) => {
                return Err("Provided configuration file is invalid: Structure must be from type Map<String, Any>!".to_owned());
            }
        };
        Ok(Arc::new(YamlConfig { properties, parent_path: None, parent: None }))
    }

    fn property(&self, key: &str) -> Option<&Value> {
        self.properties.iter().find(|(k, _)| k == key).map(|(_, value)| value)
    }

    /// `keySequence(key)`.
    fn key_sequence(&self, key: &str) -> String {
        match &self.parent_path {
            None => key.to_owned(),
            Some(parent_path) => format!("{parent_path} > {key}"),
        }
    }
}

impl Config for YamlConfig {
    fn parent(&self) -> Option<Arc<dyn Config>> {
        self.parent.clone()
    }

    fn sub_config(self: Arc<Self>, key: &str) -> Arc<dyn Config> {
        let sub_properties = match self.property(key) {
            None => Vec::new(),
            Some(Value::Map(sub_properties)) => sub_properties.clone(),
            Some(other) => panic!("java.lang.ClassCastException: {} cannot be cast to class java.util.Map", other.kotlin_type_name()),
        };
        let parent_path = Some(self.key_sequence(key));
        Arc::new(YamlConfig { properties: sub_properties, parent_path, parent: Some(self) })
    }

    fn sub_config_keys(&self) -> Vec<String> {
        self.properties.iter().map(|(key, _)| key.clone()).collect()
    }

    fn value_or_default(&self, key: &str, default: Value) -> Value {
        value_or_default_internal(self, key, self.property(key), default)
    }

    fn value_or_null(&self, key: &str) -> Option<Value> {
        self.property(key).cloned()
    }
}

/// `YamlConfig.valueOrDefaultInternal(key, result, default)`.
fn value_or_default_internal(config: &YamlConfig, key: &str, result: Option<&Value>, default: Value) -> Value {
    let Some(result) = result else { return default };
    // `None`: the ClassCastException / NumberFormatException upstream turns into the error below.
    let value = match result {
        Value::String(result) => try_parse_based_on_default(result, &default),
        Value::List(items) => {
            if !matches!(default, Value::List(_)) {
                None
            } else {
                assert!(
                    items.iter().all(|item| matches!(item, Value::String(_))),
                    "Only lists of strings are supported. Value \"{}\" set for config parameter \"{}\" contains non-string values",
                    result.render(),
                    config.key_sequence(key)
                );
                Some(result.clone())
            }
        }
        _ if is_primitive(&default) && std::mem::discriminant(result) != std::mem::discriminant(&default) => None,
        _ => Some(result.clone()),
    };
    value.unwrap_or_else(|| {
        // TODO: for a list default upstream prints the default's runtime class (`java.util.Arrays.ArrayList`, ...)
        panic!(
            "Value \"{}\" set for config parameter \"{}\" is not of required type `{}`",
            result.render(),
            config.key_sequence(key),
            default.kotlin_type_name()
        )
    })
}

/// `default::class in PRIMITIVES`.
fn is_primitive(default: &Value) -> bool {
    matches!(default, Value::Int(_) | Value::Boolean(_) | Value::Double(_) | Value::String(_) | Value::Long(_))
}

/// `tryParseBasedOnDefault(result, defaultResult)`.
fn try_parse_based_on_default(result: &str, default_result: &Value) -> Option<Value> {
    match default_result {
        Value::Int(_) => result.parse().ok().map(Value::Int),
        Value::Boolean(_) => Some(Value::Boolean(to_boolean_strict(result))),
        Value::Double(_) => result.trim().parse().ok().map(Value::Double),
        Value::String(_) => Some(Value::String(result.to_owned())),
        _ => None,
    }
}

/// Kotlin `String.toBooleanStrict()`.
fn to_boolean_strict(text: &str) -> bool {
    match text {
        "true" => true,
        "false" => false,
        _ => panic!("java.lang.IllegalArgumentException: The string doesn't represent a boolean value: {text}"),
    }
}
