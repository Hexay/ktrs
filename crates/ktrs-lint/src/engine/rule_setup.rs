//! The per-`.editorconfig` part of a file's setup, shared by every file whose config is equal: the enabled
//! rule providers (sorted for execution) and each rule's view of the config. Both are pure functions of the
//! config and the engine's providers, and rebuilding them per file and per pass was most of the fixed cost.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::editorconfig::{EditorConfig, KtlintVersion};
use crate::engine::rule_execution_context::rule_editor_config;
use crate::engine::rule_filter::{InternalRuleProvidersFilter, RuleExecutionRuleFilter, apply_rule_filters};
use crate::engine::visitor_provider::VisitorProvider;
use crate::rule::{RuleId, RuleV2};
use crate::rule_provider::{RuleV2Provider, rule_providers_in};

/// Distinct configs kept; a run usually has one per `.editorconfig` directory.
const CAPACITY: usize = 16;

pub(crate) struct RuleSetup {
    pub(crate) editor_config: EditorConfig,
    pub(crate) rule_providers: Vec<RuleV2Provider>,
    pub(crate) visitor_provider: VisitorProvider,
    rule_editor_configs: Mutex<HashMap<RuleId, Arc<EditorConfig>>>,
}

impl RuleSetup {
    /// [`rule_editor_config`], built on first request so a panic in it still surfaces inside the rule's
    /// `execute`, as upstream.
    pub(crate) fn rule_editor_config(&self, rule: &dyn RuleV2) -> Arc<EditorConfig> {
        if let Some(cached) = self.rule_editor_configs.lock().unwrap().get(&rule.rule_id()) {
            return cached.clone();
        }
        let built = Arc::new(rule_editor_config(&self.editor_config, &self.rule_providers, rule));
        self.rule_editor_configs.lock().unwrap().insert(rule.rule_id(), built.clone());
        built
    }
}

#[derive(Default)]
pub(crate) struct RuleSetupCache {
    entries: Mutex<Vec<Arc<RuleSetup>>>,
}

impl RuleSetupCache {
    pub(crate) fn get(&self, editor_config: EditorConfig, engine_rule_providers: &[RuleV2Provider]) -> Arc<RuleSetup> {
        if let Some(hit) = self.entries.lock().unwrap().iter().find(|s| s.editor_config == editor_config) {
            return hit.clone();
        }
        // The version's rule set stands in for the engine's providers, so suppressions of rules it lacks
        // are "unknown or not loaded" as in that release.
        let engine_rule_providers = &rule_providers_in(engine_rule_providers, KtlintVersion::of(&editor_config));
        let rule_providers = apply_rule_filters(
            engine_rule_providers,
            &[
                &InternalRuleProvidersFilter::new(engine_rule_providers),
                &RuleExecutionRuleFilter::new(&editor_config),
            ],
        );
        let setup = Arc::new(RuleSetup {
            visitor_provider: VisitorProvider::new(&rule_providers),
            editor_config,
            rule_providers,
            rule_editor_configs: Mutex::default(),
        });
        let mut entries = self.entries.lock().unwrap();
        if entries.len() == CAPACITY {
            entries.remove(0);
        }
        entries.push(setup.clone());
        setup
    }
}
