//! Port of `ktlint/ModifierWithoutDefaultCheck.kt`.

use crate::ktlint::editor_config_properties::{CUSTOM_MODIFIERS, ComposeProperty};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::modifier_without_default::ModifierWithoutDefault;

pub const CHECK: Option<fn() -> KtlintRule> = Some(modifier_without_default_check);

pub fn modifier_without_default_check() -> KtlintRule {
    KtlintRule::new(
        "compose:modifier-without-default-check",
        vec![ComposeProperty::String(&CUSTOM_MODIFIERS)],
        Box::new(ModifierWithoutDefault),
    )
}
