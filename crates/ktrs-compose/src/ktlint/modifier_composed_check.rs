//! Port of `ktlint/ModifierComposedCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(modifier_composed_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
