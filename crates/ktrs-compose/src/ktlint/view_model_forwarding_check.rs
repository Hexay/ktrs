//! Port of `ktlint/ViewModelForwardingCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(view_model_forwarding_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
