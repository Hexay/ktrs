//! Port of `ktlint/ViewModelForwardingCheck.kt`.

use crate::ktlint::editor_config_properties::{
    ALLOWED_FORWARDING, ALLOWED_FORWARDING_OF_TYPES, ALLOWED_STATE_HOLDER_NAMES, ComposeProperty,
};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::view_model_forwarding::ViewModelForwarding;

pub const CHECK: Option<fn() -> KtlintRule> = Some(view_model_forwarding_check);

pub fn view_model_forwarding_check() -> KtlintRule {
    KtlintRule::new(
        "compose:vm-forwarding-check",
        vec![
            ComposeProperty::String(&ALLOWED_STATE_HOLDER_NAMES),
            ComposeProperty::String(&ALLOWED_FORWARDING),
            ComposeProperty::String(&ALLOWED_FORWARDING_OF_TYPES),
        ],
        Box::new(ViewModelForwarding),
    )
}
