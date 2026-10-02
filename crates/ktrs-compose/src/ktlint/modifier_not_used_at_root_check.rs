//! Port of `ktlint/ModifierNotUsedAtRootCheck.kt`.

use crate::ktlint::editor_config_properties::{
    CONTENT_EMITTERS_DENYLIST, CONTENT_EMITTERS_PROPERTY, CUSTOM_MODIFIERS, ComposeProperty,
};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::modifier_not_used_at_root::ModifierNotUsedAtRoot;

pub const CHECK: Option<fn() -> KtlintRule> = Some(modifier_not_used_at_root_check);

pub fn modifier_not_used_at_root_check() -> KtlintRule {
    KtlintRule::new(
        "compose:modifier-not-used-at-root",
        vec![
            ComposeProperty::String(&CONTENT_EMITTERS_PROPERTY),
            ComposeProperty::String(&CUSTOM_MODIFIERS),
            ComposeProperty::String(&CONTENT_EMITTERS_DENYLIST),
        ],
        Box::new(ModifierNotUsedAtRoot),
    )
}
