//! Port of `ktlint/NamingCheck.kt`.

use crate::ktlint::editor_config_properties::{ALLOWED_COMPOSE_NAMING_NAMES, ComposeProperty};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::naming::Naming;

pub const CHECK: Option<fn() -> KtlintRule> = Some(naming_check);

pub fn naming_check() -> KtlintRule {
    KtlintRule::new("compose:naming-check", vec![ComposeProperty::String(&ALLOWED_COMPOSE_NAMING_NAMES)], Box::new(Naming))
}
