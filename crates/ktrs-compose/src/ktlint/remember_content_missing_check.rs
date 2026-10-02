//! Port of `ktlint/RememberContentMissingCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(remember_content_missing_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
