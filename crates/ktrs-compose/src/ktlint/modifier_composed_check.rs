//! Port of `ktlint/ModifierComposedCheck.kt`.

use crate::ktlint::editor_config_properties::{CUSTOM_MODIFIERS, ComposeProperty};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::modifier_composed::ModifierComposed;

pub const CHECK: Option<fn() -> KtlintRule> = Some(modifier_composed_check);

pub fn modifier_composed_check() -> KtlintRule {
    KtlintRule::new("compose:modifier-composed-check", vec![ComposeProperty::String(&CUSTOM_MODIFIERS)], Box::new(ModifierComposed))
}
