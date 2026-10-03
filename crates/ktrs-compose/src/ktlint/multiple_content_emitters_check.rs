//! Port of `ktlint/MultipleContentEmittersCheck.kt`.

use crate::ktlint::editor_config_properties::{CONTENT_EMITTERS_DENYLIST, CONTENT_EMITTERS_PROPERTY, ComposeProperty};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::multiple_content_emitters::MultipleContentEmitters;

pub const CHECK: Option<fn() -> KtlintRule> = Some(multiple_content_emitters_check);

pub fn multiple_content_emitters_check() -> KtlintRule {
    KtlintRule::new(
        "compose:multiple-emitters-check",
        vec![ComposeProperty::String(&CONTENT_EMITTERS_PROPERTY), ComposeProperty::String(&CONTENT_EMITTERS_DENYLIST)],
        Box::new(MultipleContentEmitters),
    )
}
