//! Port of `ktlint/ModifierShouldBeForwardedToEveryBranchCheck.kt`.

use crate::ktlint::editor_config_properties::{CUSTOM_MODIFIERS, ComposeProperty};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::modifier_should_be_forwarded_to_every_branch::ModifierShouldBeForwardedToEveryBranch;

pub const CHECK: Option<fn() -> KtlintRule> = Some(modifier_should_be_forwarded_to_every_branch_check);

pub fn modifier_should_be_forwarded_to_every_branch_check() -> KtlintRule {
    KtlintRule::new(
        "compose:modifier-should-be-forwarded-to-every-branch",
        vec![ComposeProperty::String(&CUSTOM_MODIFIERS)],
        Box::new(ModifierShouldBeForwardedToEveryBranch::default()),
    )
}
