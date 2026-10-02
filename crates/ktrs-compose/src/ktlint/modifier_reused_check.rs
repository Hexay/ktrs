//! Port of `ktlint/ModifierReusedCheck.kt`.

use crate::ktlint::editor_config_properties::{
    CONTENT_EMITTERS_DENYLIST, CONTENT_EMITTERS_PROPERTY, CUSTOM_MODIFIERS, ComposeProperty,
};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::modifier_reused::ModifierReused;

pub const CHECK: Option<fn() -> KtlintRule> = Some(modifier_reused_check);

pub fn modifier_reused_check() -> KtlintRule {
    KtlintRule::new(
        "compose:modifier-reused-check",
        vec![
            ComposeProperty::String(&CONTENT_EMITTERS_PROPERTY),
            ComposeProperty::String(&CUSTOM_MODIFIERS),
            ComposeProperty::String(&CONTENT_EMITTERS_DENYLIST),
        ],
        Box::new(ModifierReused),
    )
}
