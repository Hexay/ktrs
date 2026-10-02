//! Port of `ktlint/CompositionLocalAllowlistCheck.kt`.

use crate::ktlint::editor_config_properties::{COMPOSITION_LOCAL_ALLOWLIST_PROPERTY, ComposeProperty};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::composition_local_allowlist::CompositionLocalAllowlist;

pub const CHECK: Option<fn() -> KtlintRule> = Some(composition_local_allowlist_check);

pub fn composition_local_allowlist_check() -> KtlintRule {
    KtlintRule::new(
        "compose:compositionlocal-allowlist",
        vec![ComposeProperty::String(&COMPOSITION_LOCAL_ALLOWLIST_PROPERTY)],
        Box::new(CompositionLocalAllowlist),
    )
}
