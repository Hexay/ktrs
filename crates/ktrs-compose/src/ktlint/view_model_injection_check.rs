//! Port of `ktlint/ViewModelInjectionCheck.kt`.

use crate::ktlint::editor_config_properties::{ComposeProperty, VIEW_MODEL_FACTORIES};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::view_model_injection::ViewModelInjection;

pub const CHECK: Option<fn() -> KtlintRule> = Some(view_model_injection_check);

pub fn view_model_injection_check() -> KtlintRule {
    KtlintRule::new("compose:vm-injection-check", vec![ComposeProperty::String(&VIEW_MODEL_FACTORIES)], Box::new(ViewModelInjection))
}
