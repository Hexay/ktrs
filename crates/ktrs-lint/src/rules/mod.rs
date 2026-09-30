//! The ported standard rules (`ktlint-ruleset-standard`), registered like `StandardRuleSetProvider`.

mod multi_line_if_else_rule;
mod no_semicolons_rule;
mod spacing_around_comma_rule;

pub use multi_line_if_else_rule::MultiLineIfElseRule;
pub use no_semicolons_rule::NoSemicolonsRule;
pub use spacing_around_comma_rule::SpacingAroundCommaRule;

use crate::rule::{About, RuleV2};
use crate::rule_provider::RuleV2Provider;

/// `STANDARD_RULE_ABOUT` (`StandardRule.kt`).
pub const STANDARD_RULE_ABOUT: About = About {
    maintainer: "KtLint",
    repository_url: "https://github.com/ktlint/ktlint",
    issue_tracker_url: "https://github.com/ktlint/ktlint/issues",
};

/// The ported slice of `StandardRuleSetProvider().getRuleProviders()`.
pub fn standard_rule_providers() -> Vec<RuleV2Provider> {
    vec![
        RuleV2Provider::new(|| Box::new(SpacingAroundCommaRule) as Box<dyn RuleV2>),
        RuleV2Provider::new(|| Box::new(MultiLineIfElseRule::new())),
        RuleV2Provider::new(|| Box::new(NoSemicolonsRule)),
    ]
}

/// The provider of a ported standard rule by id (`standard:` prefix optional).
pub fn standard_rule_provider(id: &str) -> Option<RuleV2Provider> {
    let id = if id.contains(':') {
        id.to_owned()
    } else {
        format!("standard:{id}")
    };
    standard_rule_providers()
        .into_iter()
        .find(|p| p.rule_id().value() == id)
}
