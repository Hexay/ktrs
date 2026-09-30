//! Port of ktlint-rule-engine-core `RuleV2Provider.kt`.

use std::fmt;
use std::sync::Arc;

use ktrs_editorconfig::AnyPropertyType;

use crate::editorconfig::PropertyRef;
use crate::rule::{RuleId, RuleV2};

/// Creates a fresh [`RuleV2`] per traversal, so rules can keep state and files can be processed in parallel.
#[derive(Clone)]
pub struct RuleV2Provider {
    provider: Arc<dyn Fn() -> Box<dyn RuleV2> + Send + Sync>,
    rule_id: RuleId,
    uses_editor_config_properties: Vec<PropertyRef>,
}

impl RuleV2Provider {
    /// `RuleV2Provider { ... }`: instantiates the rule once to learn its id and properties.
    pub fn new(provider: impl Fn() -> Box<dyn RuleV2> + Send + Sync + 'static) -> RuleV2Provider {
        let rule = provider();
        RuleV2Provider {
            rule_id: rule.rule_id(),
            uses_editor_config_properties: rule.uses_editor_config_properties(),
            provider: Arc::new(provider),
        }
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
}

impl fmt::Debug for RuleV2Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuleV2Provider({})", self.rule_id)
    }
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
