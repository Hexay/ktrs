//! Port of `ktlint/RememberStateMissingCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(remember_state_missing_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
