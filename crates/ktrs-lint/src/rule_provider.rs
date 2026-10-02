//! Port of ktlint-rule-engine-core `RuleV2Provider.kt`.

use std::fmt;
use std::sync::Arc;

use ktrs_editorconfig::AnyPropertyType;

use crate::editorconfig::{KtlintVersion, PropertyRef};
use crate::rule::{RuleId, RuleV2, RunAfterRuleMode, VisitorModifier};

/// Creates a fresh [`RuleV2`] per traversal, so rules can keep state and files can be processed in parallel.
#[derive(Clone)]
pub struct RuleV2Provider {
    provider: Arc<dyn Fn() -> Box<dyn RuleV2> + Send + Sync>,
    rule_id: RuleId,
    uses_editor_config_properties: Vec<PropertyRef>,
    visitor_modifiers: &'static [VisitorModifier],
    /// ktrs: registered only for files in this ktlint version (`None`: in every version).
    only_in: Option<KtlintVersion>,
}

impl RuleV2Provider {
    /// `RuleV2Provider { ... }`: instantiates the rule once to learn its id and properties.
    pub fn new(provider: impl Fn() -> Box<dyn RuleV2> + Send + Sync + 'static) -> RuleV2Provider {
        let rule = provider();
        RuleV2Provider {
            rule_id: rule.rule_id(),
            uses_editor_config_properties: rule.uses_editor_config_properties(),
            visitor_modifiers: rule.visitor_modifiers(),
            provider: Arc::new(provider),
            only_in: None,
        }
    }

    /// The rule exists only in `version`'s rule set (added, removed or redefined between releases).
    pub fn only_in(self, version: KtlintVersion) -> RuleV2Provider {
        RuleV2Provider { only_in: Some(version), ..self }
    }

    pub fn runs_in(&self, version: KtlintVersion) -> bool {
        self.only_in.is_none_or(|v| v == version)
    }

    pub fn create_new_rule_instance(&self) -> Box<dyn RuleV2> {
        (self.provider)()
    }

    pub fn rule_id(&self) -> RuleId {
        self.rule_id
    }

    pub fn uses_editor_config_properties(&self) -> &[PropertyRef] {
        &self.uses_editor_config_properties
    }

    /// `RuleProvider.runAfterRules` (1.8).
    pub fn run_after_rules(&self) -> impl Iterator<Item = (RuleId, RunAfterRuleMode)> + '_ {
        self.visitor_modifiers.iter().filter_map(|m| match *m {
            VisitorModifier::RunAfterRule(rule_id, mode) => Some((rule_id, mode)),
            VisitorModifier::RunAsLateAsPossible => None,
        })
    }

    /// `RuleProvider.runAsLateAsPossible` (1.8).
    pub fn run_as_late_as_possible(&self) -> bool {
        self.visitor_modifiers.contains(&VisitorModifier::RunAsLateAsPossible)
    }
}

impl fmt::Debug for RuleV2Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuleV2Provider({})", self.rule_id)
    }
}

/// The providers registered in `version`'s rule set.
pub fn rule_providers_in(rule_providers: &[RuleV2Provider], version: KtlintVersion) -> Vec<RuleV2Provider> {
    rule_providers.iter().filter(|p| p.runs_in(version)).cloned().collect()
}

/// `Collection<RuleV2Provider>.propertyTypes()`: the types of all properties the rules use.
pub fn property_types(rule_providers: &[RuleV2Provider]) -> Vec<&'static dyn AnyPropertyType> {
    let mut types: Vec<&'static dyn AnyPropertyType> = Vec::new();
    for t in rule_providers.iter().flat_map(|p| {
        p.uses_editor_config_properties
            .iter()
            .map(|u| u.property_type())
    }) {
        if !types.iter().any(|e| e.name() == t.name()) {
            types.push(t);
        }
    }
    types
}
