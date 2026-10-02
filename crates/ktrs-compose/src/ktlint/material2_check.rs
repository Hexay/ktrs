//! Port of `ktlint/Material2Check.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(material2_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
