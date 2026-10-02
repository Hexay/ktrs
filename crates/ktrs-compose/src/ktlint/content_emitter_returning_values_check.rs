//! Port of `ktlint/ContentEmitterReturningValuesCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(content_emitter_returning_values_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
