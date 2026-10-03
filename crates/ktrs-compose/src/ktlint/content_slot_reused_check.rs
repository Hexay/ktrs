//! Port of `ktlint/ContentSlotReusedCheck.kt`.

use crate::ktlint::editor_config_properties::{ComposeProperty, TREAT_AS_COMPOSABLE_LAMBDA, TREAT_AS_LAMBDA};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::content_slot_reused::ContentSlotReused;

pub const CHECK: Option<fn() -> KtlintRule> = Some(content_slot_reused_check);

pub fn content_slot_reused_check() -> KtlintRule {
    KtlintRule::new(
        "compose:content-slot-reused",
        vec![ComposeProperty::String(&TREAT_AS_LAMBDA), ComposeProperty::String(&TREAT_AS_COMPOSABLE_LAMBDA)],
        Box::new(ContentSlotReused),
    )
}
