//! `detekt-core`: rule discovery, the analyzer and suppression (plus `detekt-utils`' path filters).

mod analyzer;
mod path_filters;
mod rule_descriptor;
mod suppressions;

use std::path::PathBuf;
use std::sync::Arc;

pub use analyzer::Analyzer;
pub use path_filters::PathFilters;
pub use rule_descriptor::{AnalysisMode, DETEKT_VERSION, RuleDescriptor, get_rules};
pub use suppressions::is_suppressed_by;

use crate::api::Config;
use crate::rules::default_rule_set_providers;

/// The analyzer `Lifecycle.analyze` builds: the rules of the default providers that `config` activates.
// TODO: the rest of Lifecycle (config validation, processors, reports) comes with the CLI.
pub fn create_analyzer(base_path: PathBuf, config: &Arc<dyn Config>) -> Analyzer {
    let analysis_mode = AnalysisMode::Light;
    let rules = get_rules(analysis_mode, &default_rule_set_providers(), config, &mut |_| {});
    Analyzer::new(base_path, rules.into_iter().filter(|rule| rule.rule_instance.active).collect(), analysis_mode)
}
