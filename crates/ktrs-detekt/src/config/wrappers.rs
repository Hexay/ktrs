//! `detekt-core/.../config/CompositeConfig.kt`, `AllRulesConfig.kt` and `DisabledAutoCorrectConfig.kt`.

use std::sync::Arc;

use crate::api::config::{ACTIVE_KEY, AUTO_CORRECT_KEY};
use crate::api::{Config, Value};

/// Wraps two different configuration which should be considered when retrieving properties.
pub struct CompositeConfig {
    look_first: Arc<dyn Config>,
    look_second: Arc<dyn Config>,
    parent: Option<Arc<dyn Config>>,
}

impl CompositeConfig {
    pub fn new(look_first: Arc<dyn Config>, look_second: Arc<dyn Config>) -> Arc<CompositeConfig> {
        Arc::new(CompositeConfig { look_first, look_second, parent: None })
    }
}

impl Config for CompositeConfig {
    fn parent(&self) -> Option<Arc<dyn Config>> {
        self.parent.clone()
    }

    fn sub_config(self: Arc<Self>, key: &str) -> Arc<dyn Config> {
        let look_first = self.look_first.clone().sub_config(key);
        let look_second = self.look_second.clone().sub_config(key);
        Arc::new(CompositeConfig { look_first, look_second, parent: Some(self) })
    }

    fn sub_config_keys(&self) -> Vec<String> {
        let mut keys = self.look_first.sub_config_keys();
        for key in self.look_second.sub_config_keys() {
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        keys
    }

    fn value_or_default(&self, key: &str, default: Value) -> Value {
        if self.look_first.value_or_null(key).is_some() {
            return self.look_first.value_or_default(key, default);
        }
        self.look_second.value_or_default(key, default)
    }

    fn value_or_null(&self, key: &str) -> Option<Value> {
        self.look_first.value_or_null(key).or_else(|| self.look_second.value_or_null(key))
    }
}

/// `DeprecatedRule(ruleSetId, ruleName, description)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeprecatedRule {
    pub rule_set_id: String,
    pub rule_name: String,
}

/// `--all-rules`: every rule is active unless its config says otherwise or it is deprecated.
pub struct AllRulesConfig {
    wrapped: Arc<dyn Config>,
    deprecated_rules: Arc<Vec<DeprecatedRule>>,
    parent: Option<Arc<AllRulesConfig>>,
    key: Option<String>,
}

impl AllRulesConfig {
    pub fn new(wrapped: Arc<dyn Config>, deprecated_rules: Vec<DeprecatedRule>) -> Arc<AllRulesConfig> {
        Arc::new(AllRulesConfig { wrapped, deprecated_rules: Arc::new(deprecated_rules), parent: None, key: None })
    }

    fn is_deprecated(&self) -> bool {
        let parent_key = self.parent.as_ref().and_then(|p| p.key.as_deref());
        self.deprecated_rules.iter().any(|d| self.key.as_deref() == Some(d.rule_name.as_str()) && parent_key == Some(d.rule_set_id.as_str()))
    }
}

impl Config for AllRulesConfig {
    fn parent(&self) -> Option<Arc<dyn Config>> {
        self.parent.clone().map(|p| p as Arc<dyn Config>)
    }

    fn sub_config(self: Arc<Self>, key: &str) -> Arc<dyn Config> {
        Arc::new(AllRulesConfig {
            wrapped: self.wrapped.clone().sub_config(key),
            deprecated_rules: self.deprecated_rules.clone(),
            key: Some(key.to_owned()),
            parent: Some(self),
        })
    }

    fn sub_config_keys(&self) -> Vec<String> {
        self.wrapped.sub_config_keys()
    }

    fn value_or_default(&self, key: &str, default: Value) -> Value {
        match key {
            ACTIVE_KEY if self.is_deprecated() => Value::Boolean(false),
            ACTIVE_KEY => self.wrapped.value_or_default(key, Value::Boolean(true)),
            _ => self.wrapped.value_or_default(key, default),
        }
    }

    fn value_or_null(&self, key: &str) -> Option<Value> {
        match key {
            ACTIVE_KEY if self.is_deprecated() => Some(Value::Boolean(false)),
            ACTIVE_KEY => Some(self.wrapped.value_or_null(key).unwrap_or(Value::Boolean(true))),
            _ => self.wrapped.value_or_null(key),
        }
    }
}

/// Without `--auto-correct`: every `autoCorrect` reads false.
pub struct DisabledAutoCorrectConfig {
    wrapped: Arc<dyn Config>,
    parent: Option<Arc<dyn Config>>,
}

impl DisabledAutoCorrectConfig {
    pub fn new(wrapped: Arc<dyn Config>) -> Arc<DisabledAutoCorrectConfig> {
        Arc::new(DisabledAutoCorrectConfig { wrapped, parent: None })
    }
}

impl Config for DisabledAutoCorrectConfig {
    fn parent(&self) -> Option<Arc<dyn Config>> {
        self.parent.clone()
    }

    fn sub_config(self: Arc<Self>, key: &str) -> Arc<dyn Config> {
        Arc::new(DisabledAutoCorrectConfig { wrapped: self.wrapped.clone().sub_config(key), parent: Some(self) })
    }

    fn sub_config_keys(&self) -> Vec<String> {
        self.wrapped.sub_config_keys()
    }

    fn value_or_default(&self, key: &str, default: Value) -> Value {
        match key {
            AUTO_CORRECT_KEY => Value::Boolean(false),
            _ => self.wrapped.value_or_default(key, default),
        }
    }

    fn value_or_null(&self, key: &str) -> Option<Value> {
        match key {
            AUTO_CORRECT_KEY => Some(Value::Boolean(false)),
            _ => self.wrapped.value_or_null(key),
        }
    }
}
