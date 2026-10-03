//! Port of `ktlint/ModifierNamingCheck.kt`.

use crate::ktlint::editor_config_properties::{CUSTOM_MODIFIERS, ComposeProperty};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::modifier_naming::ModifierNaming;

pub const CHECK: Option<fn() -> KtlintRule> = Some(modifier_naming_check);

pub fn modifier_naming_check() -> KtlintRule {
    KtlintRule::new("compose:modifier-naming", vec![ComposeProperty::String(&CUSTOM_MODIFIERS)], Box::new(ModifierNaming))
}
