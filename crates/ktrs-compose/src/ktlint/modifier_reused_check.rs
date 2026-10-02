//! Port of `ktlint/ModifierReusedCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(modifier_reused_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
