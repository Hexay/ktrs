//! Port of `KtlintComposeKtConfig.kt`: config keys are camel case upstream; ktlint's are `compose_<snake_case>`,
//! looked up among the rule's own properties only, and memoized per key.

use std::cell::RefCell;
use std::collections::HashMap;

use ktrs_ast::psi::EmbeddedKotlin;
use ktrs_lint::EditorConfig;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::util::kotlin_utils::to_snake_case;
use crate::ktlint::editor_config_properties::ComposeProperty;

/// What `find` returns: the property's value (`Any?`).
#[derive(Clone, Debug, PartialEq)]
enum Raw {
    String(String),
    Boolean(bool),
    Int(i32),
}

/// A cache entry. Upstream caches `Any?` per key and reads it back with `as? T`, so an entry of another type
/// (`getSet` after `getList` on the same key) reads as null and yields the default.
#[derive(Clone, Debug, PartialEq)]
enum Cached {
    Int(i32),
    String(String),
    List(Vec<String>),
    Set(Vec<String>),
    Boolean(bool),
}

pub struct KtlintComposeKtConfig {
    properties: EditorConfig,
    editor_config_properties: Vec<ComposeProperty>,
    cache: RefCell<HashMap<String, Option<Cached>>>,
    embedded_kotlin: EmbeddedKotlin,
}

impl KtlintComposeKtConfig {
    pub fn new(properties: EditorConfig, editor_config_properties: Vec<ComposeProperty>, embedded_kotlin: EmbeddedKotlin) -> Self {
        KtlintComposeKtConfig { properties, editor_config_properties, cache: RefCell::new(HashMap::new()), embedded_kotlin }
    }

    /// `getValueAsOrPut(key, value)`: `cache.getOrPut(key) { value() } as? T` (a null entry is recomputed).
    fn get_value_as_or_put<T>(&self, key: &str, value: impl FnOnce() -> Option<Cached>, as_t: impl Fn(&Cached) -> Option<T>) -> Option<T> {
        let cached = self.cache.borrow().get(key).cloned().flatten();
        let entry = match cached {
            Some(entry) => entry,
            None => {
                let answer = value();
                self.cache.borrow_mut().insert(key.to_owned(), answer.clone());
                answer?
            }
        };
        as_t(&entry)
    }

    /// `find(key)`: the value of the rule's property named `compose_<snake_case key>`.
    fn find(&self, key: &str) -> Option<Raw> {
        let name = ktlint_key(key);
        let property = self.editor_config_properties.iter().find(|p| p.name() == name)?;
        Some(match *property {
            ComposeProperty::String(p) => Raw::String(self.properties.get(p)),
            ComposeProperty::Boolean(p) => Raw::Boolean(self.properties.get(p)),
            ComposeProperty::Int(p) => Raw::Int(self.properties.get(p)),
        })
    }
}

impl ComposeKtConfig for KtlintComposeKtConfig {
    fn get_int(&self, key: &str, default: i32) -> i32 {
        let value = || match self.find(key) {
            Some(Raw::Int(raw)) => Some(Cached::Int(raw)),
            Some(Raw::String(raw)) => raw.parse::<i32>().ok().map(Cached::Int),
            _ => None,
        };
        self.get_value_as_or_put(key, value, |c| if let Cached::Int(v) = c { Some(*v) } else { None }).unwrap_or(default)
    }

    fn get_string(&self, key: &str, default: Option<&str>) -> Option<String> {
        let value = || match self.find(key) {
            Some(Raw::String(raw)) => Some(Cached::String(raw)),
            _ => None,
        };
        self.get_value_as_or_put(key, value, |c| if let Cached::String(v) = c { Some(v.clone()) } else { None })
            .or_else(|| default.map(str::to_owned))
    }

    fn get_list(&self, key: &str, default: &[String]) -> Vec<String> {
        let value = || match self.find(key) {
            Some(Raw::String(raw)) => Some(Cached::List(raw.split([',', ';']).map(|it| it.trim().to_owned()).collect())),
            _ => None,
        };
        self.get_value_as_or_put(key, value, |c| if let Cached::List(v) = c { Some(v.clone()) } else { None })
            .unwrap_or_else(|| default.to_vec())
    }

    fn get_set(&self, key: &str, default: &[String]) -> Vec<String> {
        let value = || {
            let mut set: Vec<String> = Vec::new();
            for item in self.get_list(key, default) {
                if !set.contains(&item) {
                    set.push(item);
                }
            }
            Some(Cached::Set(set))
        };
        self.get_value_as_or_put(key, value, |c| if let Cached::Set(v) = c { Some(v.clone()) } else { None })
            .unwrap_or_else(|| default.to_vec())
    }

    fn get_boolean(&self, key: &str, default: bool) -> bool {
        let value = || match self.find(key) {
            Some(Raw::Boolean(raw)) => Some(Cached::Boolean(raw)),
            _ => None,
        };
        self.get_value_as_or_put(key, value, |c| if let Cached::Boolean(v) = c { Some(*v) } else { None }).unwrap_or(default)
    }

    fn embedded_kotlin(&self) -> EmbeddedKotlin {
        self.embedded_kotlin
    }
}

/// `ktlintKey(key)`: `compose_` + the snake case key.
fn ktlint_key(key: &str) -> String {
    format!("compose_{}", to_snake_case(key))
}
