//! The ported standard rules (`ktlint-ruleset-standard`), registered like `StandardRuleSetProvider`.

pub mod max_line_length_rule;
mod modifier_list_spacing_rule;
mod modifier_order_rule;
mod multi_line_if_else_rule;
mod no_semicolons_rule;
mod nullable_type_spacing_rule;
mod parameter_list_spacing_rule;
mod spacing_around_angle_brackets_rule;
mod spacing_around_colon_rule;
mod spacing_around_comma_rule;
mod spacing_around_curly_rule;
mod spacing_around_dot_rule;
mod spacing_around_double_colon_rule;
mod spacing_around_keyword_rule;
mod spacing_around_operators_rule;
mod spacing_around_parens_rule;
mod spacing_around_range_operator_rule;
mod spacing_around_square_brackets_rule;
mod spacing_around_unary_operator_rule;
mod type_argument_list_spacing_rule;
mod type_parameter_list_spacing_rule;

pub use modifier_list_spacing_rule::ModifierListSpacingRule;
pub use modifier_order_rule::ModifierOrderRule;
pub use multi_line_if_else_rule::MultiLineIfElseRule;
pub use no_semicolons_rule::NoSemicolonsRule;
pub use nullable_type_spacing_rule::NullableTypeSpacingRule;
pub use parameter_list_spacing_rule::ParameterListSpacingRule;
pub use spacing_around_angle_brackets_rule::SpacingAroundAngleBracketsRule;
pub use spacing_around_colon_rule::SpacingAroundColonRule;
pub use spacing_around_comma_rule::SpacingAroundCommaRule;
pub use spacing_around_curly_rule::SpacingAroundCurlyRule;
pub use spacing_around_dot_rule::SpacingAroundDotRule;
pub use spacing_around_double_colon_rule::SpacingAroundDoubleColonRule;
pub use spacing_around_keyword_rule::SpacingAroundKeywordRule;
pub use spacing_around_operators_rule::SpacingAroundOperatorsRule;
pub use spacing_around_parens_rule::SpacingAroundParensRule;
pub use spacing_around_range_operator_rule::SpacingAroundRangeOperatorRule;
pub use spacing_around_square_brackets_rule::SpacingAroundSquareBracketsRule;
pub use spacing_around_unary_operator_rule::SpacingAroundUnaryOperatorRule;
pub use type_argument_list_spacing_rule::TypeArgumentListSpacingRule;
pub use type_parameter_list_spacing_rule::TypeParameterListSpacingRule;

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
        RuleV2Provider::new(|| Box::new(ModifierListSpacingRule::new())),
        RuleV2Provider::new(|| Box::new(ModifierOrderRule)),
        RuleV2Provider::new(|| Box::new(MultiLineIfElseRule::new())),
        RuleV2Provider::new(|| Box::new(NoSemicolonsRule)),
        RuleV2Provider::new(|| Box::new(NullableTypeSpacingRule)),
        RuleV2Provider::new(|| Box::new(ParameterListSpacingRule::new())),
        RuleV2Provider::new(|| Box::new(SpacingAroundAngleBracketsRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundColonRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundCurlyRule::new())),
        RuleV2Provider::new(|| Box::new(SpacingAroundDotRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundDoubleColonRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundKeywordRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundOperatorsRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundParensRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundRangeOperatorRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundSquareBracketsRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundUnaryOperatorRule)),
        RuleV2Provider::new(|| Box::new(TypeArgumentListSpacingRule::new())),
        RuleV2Provider::new(|| Box::new(TypeParameterListSpacingRule::new())),
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
