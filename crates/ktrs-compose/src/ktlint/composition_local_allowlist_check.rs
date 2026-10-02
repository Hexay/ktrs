//! Port of `ktlint/CompositionLocalAllowlistCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(composition_local_allowlist_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
