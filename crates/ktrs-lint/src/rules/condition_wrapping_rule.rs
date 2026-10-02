//! Port of ktlint 1.8 `ConditionWrappingRule.kt` (id `condition-wrapping`): a deprecated, empty rule kept in
//! 1.8's rule set only (removed in 2.0, #3237), so its id stays known to suppressions and `.editorconfig`.

use crate::rule::{About, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

pub struct ConditionWrappingRule;

impl RuleV2 for ConditionWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:condition-wrapping")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(TokenSet::EMPTY)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }
}
