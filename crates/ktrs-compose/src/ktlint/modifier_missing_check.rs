//! Port of `ktlint/ModifierMissingCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(modifier_missing_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
