//! Port of `ktlint/ComposableNestingDepthCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(composable_nesting_depth_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
