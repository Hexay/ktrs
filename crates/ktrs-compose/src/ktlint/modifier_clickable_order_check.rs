//! Port of `ktlint/ModifierClickableOrderCheck.kt`.

use crate::ktlint::editor_config_properties::{CUSTOM_MODIFIERS, ComposeProperty};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::modifier_clickable_order::ModifierClickableOrder;

pub const CHECK: Option<fn() -> KtlintRule> = Some(modifier_clickable_order_check);

pub fn modifier_clickable_order_check() -> KtlintRule {
    KtlintRule::new(
        "compose:modifier-clickable-order",
        vec![ComposeProperty::String(&CUSTOM_MODIFIERS)],
        Box::new(ModifierClickableOrder),
    )
}
