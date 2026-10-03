//! Port of `ktlint/ModifierMissingCheck.kt`.

use crate::ktlint::editor_config_properties::{
    CHECK_MODIFIERS_FOR_VISIBILITY, CONTENT_EMITTERS_DENYLIST, CONTENT_EMITTERS_PROPERTY, CUSTOM_MODIFIERS,
    ComposeProperty, MODIFIER_MISSING_IGNORE_ANNOTATED,
};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::modifier_missing::ModifierMissing;

pub const CHECK: Option<fn() -> KtlintRule> = Some(modifier_missing_check);

pub fn modifier_missing_check() -> KtlintRule {
    KtlintRule::new(
        "compose:modifier-missing-check",
        vec![
            ComposeProperty::String(&CHECK_MODIFIERS_FOR_VISIBILITY),
            ComposeProperty::String(&CONTENT_EMITTERS_PROPERTY),
            ComposeProperty::String(&CUSTOM_MODIFIERS),
            ComposeProperty::String(&CONTENT_EMITTERS_DENYLIST),
            ComposeProperty::String(&MODIFIER_MISSING_IGNORE_ANNOTATED),
        ],
        Box::new(ModifierMissing),
    )
}
