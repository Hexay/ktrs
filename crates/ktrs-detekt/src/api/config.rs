//! `detekt-api/.../Config.kt`. Values are a [`Value`] instead of `Any`; `valueOrDefault<T>` takes and returns one,
//! and the typed reads are in `config_property.rs`.

use std::sync::{Arc, OnceLock};

/// A configuration value: what snakeyaml builds, or what a test passes to `TestConfig`.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    String(String),
    Boolean(bool),
    Int(i32),
    Long(i64),
    Double(f64),
    List(Vec<Value>),
    /// In key order (`LinkedHashMap`).
    Map(Vec<(String, Value)>),
}

impl Value {
    /// Kotlin `Any.toString()` as detekt's error messages render a value.
    pub fn render(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Long(l) => l.to_string(),
            Value::Double(d) => format!("{d:?}"),
            Value::List(items) => format!("[{}]", items.iter().map(Value::render).collect::<Vec<_>>().join(", ")),
            Value::Map(entries) => {
                let entries: Vec<String> = entries.iter().map(|(k, v)| format!("{k}={}", v.render())).collect();
                format!("{{{}}}", entries.join(", "))
            }
        }
    }

    /// `default::class.qualifiedName` after `getDefaultName`.
    pub fn kotlin_type_name(&self) -> &'static str {
        match self {
            Value::String(_) => "kotlin.String",
            Value::Boolean(_) => "kotlin.Boolean",
            Value::Int(_) => "kotlin.Int",
            Value::Long(_) => "kotlin.Long",
            Value::Double(_) => "kotlin.Double",
            Value::List(_) => "kotlin.List",
            Value::Map(_) => "kotlin.collections.Map",
        }
    }
}

pub const ACTIVE_KEY: &str = "active";
pub const ALIASES_KEY: &str = "aliases";
pub const AUTO_CORRECT_KEY: &str = "autoCorrect";
pub const IGNORE_ANNOTATED_KEY: &str = "ignoreAnnotated";
pub const SEVERITY_KEY: &str = "severity";
pub const EXCLUDES_KEY: &str = "excludes";
pub const INCLUDES_KEY: &str = "includes";

/// A configuration holds information about how to configure specific rules.
pub trait Config: Send + Sync {
    fn parent(&self) -> Option<Arc<dyn Config>>;

    fn sub_config(self: Arc<Self>, key: &str) -> Arc<dyn Config>;

    fn sub_config_keys(&self) -> Vec<String>;

    fn value_or_default(&self, key: &str, default: Value) -> Value {
        self.value_or_null(key).unwrap_or(default)
    }

    fn value_or_null(&self, key: &str) -> Option<Value>;

    /// `this === Config.empty`.
    fn is_empty_config(&self) -> bool {
        false
    }
}

/// `Config.empty`: no properties; its own parent and sub config.
struct EmptyConfig;

impl Config for EmptyConfig {
    fn parent(&self) -> Option<Arc<dyn Config>> {
        Some(empty())
    }

    fn sub_config(self: Arc<Self>, _key: &str) -> Arc<dyn Config> {
        self
    }

    fn sub_config_keys(&self) -> Vec<String> {
        Vec::new()
    }

    fn value_or_null(&self, _key: &str) -> Option<Value> {
        None
    }

    fn is_empty_config(&self) -> bool {
        true
    }
}

/// `Config.empty`.
pub fn empty() -> Arc<dyn Config> {
    static EMPTY: OnceLock<Arc<EmptyConfig>> = OnceLock::new();
    EMPTY.get_or_init(|| Arc::new(EmptyConfig)).clone()
}
