//! `detekt-core/.../config` and the configuration half of `tooling/ProcessingSpecSettingsBridge.kt`.

mod wrappers;
mod yaml;
mod yaml_config;

use std::sync::Arc;

pub use wrappers::{AllRulesConfig, CompositeConfig, DeprecatedRule, DisabledAutoCorrectConfig};
pub use yaml_config::YamlConfig;

use crate::api::{self, Config};

/// `default-detekt-config.yml` of the pinned release (tools/sync-detekt.sh checks it is current).
const DEFAULT_CONFIG: &str = include_str!("../default-detekt-config.yml");

/// `ProcessingSpec.getDefaultConfiguration()`.
// TODO: plugin jars' `config/config.yml` are appended to the text upstream (DefaultConfigProvider.kt)
pub fn get_default_configuration() -> Arc<dyn Config> {
    YamlConfig::load(DEFAULT_CONFIG).expect("the bundled default config loads")
}

/// `loadConfiguration()`: `configs` are the `--config` files' texts in command-line order (the last one wins per
/// key); none is `Config.empty`.
pub fn load_configuration(configs: &[&str]) -> Result<Arc<dyn Config>, String> {
    let mut loaded: Vec<Arc<dyn Config>> = Vec::new();
    for text in configs {
        loaded.push(YamlConfig::load(text)?);
    }
    if loaded.is_empty() {
        loaded.push(api::config::empty());
    }
    let mut loaded = loaded.into_iter();
    let first = loaded.next().expect("one config at least");
    Ok(loaded.fold(first, |composite, config| -> Arc<dyn Config> { CompositeConfig::new(config, composite) }))
}

/// `workaroundConfiguration(config)`.
pub fn workaround_configuration(config: Arc<dyn Config>, activate_all_rules: bool, auto_correct: bool, use_default_config: bool) -> Arc<dyn Config> {
    let is_empty_config = config.is_empty_config();
    let mut declared_config = config;

    if activate_all_rules {
        // TODO: `loadDeprecations()`: the deprecated rules of the release's generated `deprecation.properties`
        declared_config = AllRulesConfig::new(declared_config, Vec::new());
    }

    if !auto_correct {
        declared_config = DisabledAutoCorrectConfig::new(declared_config);
    }

    if use_default_config || is_empty_config {
        declared_config = CompositeConfig::new(declared_config, get_default_configuration());
    }

    declared_config
}
