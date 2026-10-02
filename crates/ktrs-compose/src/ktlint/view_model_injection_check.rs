//! Port of `ktlint/ViewModelInjectionCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(view_model_injection_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
