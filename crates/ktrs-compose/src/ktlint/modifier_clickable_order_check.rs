//! Port of `ktlint/ModifierClickableOrderCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(modifier_clickable_order_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
