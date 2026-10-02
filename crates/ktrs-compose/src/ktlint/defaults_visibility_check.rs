//! Port of `ktlint/DefaultsVisibilityCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(defaults_visibility_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
