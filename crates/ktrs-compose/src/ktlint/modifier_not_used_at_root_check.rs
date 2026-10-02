//! Port of `ktlint/ModifierNotUsedAtRootCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(modifier_not_used_at_root_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
