//! Port of `ktlint/ParameterNamingCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(parameter_naming_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
