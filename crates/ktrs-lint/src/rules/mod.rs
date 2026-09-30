mod multi_line_if_else_rule;
mod no_semicolons_rule;
mod spacing_around_comma_rule;

pub use multi_line_if_else_rule::MultiLineIfElseRule;
pub use no_semicolons_rule::NoSemicolonsRule;
pub use spacing_around_comma_rule::SpacingAroundCommaRule;

use crate::rule::{RuleV2, RuleV2Provider};

/// The ported slice of `StandardRuleSetProvider().getRuleProviders()`, keyed by rule id.
pub const STANDARD_RULE_PROVIDERS: &[(&str, RuleV2Provider)] = &[
    ("standard:comma-spacing", || Box::new(SpacingAroundCommaRule) as Box<dyn RuleV2>),
    ("standard:multiline-if-else", || Box::new(MultiLineIfElseRule::new())),
    ("standard:no-semi", || Box::new(NoSemicolonsRule)),
];

/// The provider of a ported standard rule by id (`standard:` prefix optional).
pub fn standard_rule_provider(id: &str) -> Option<RuleV2Provider> {
    let id = if id.contains(':') { id.to_owned() } else { format!("standard:{id}") };
    STANDARD_RULE_PROVIDERS.iter().find(|(rule_id, _)| *rule_id == id).map(|(_, provider)| *provider)
}
