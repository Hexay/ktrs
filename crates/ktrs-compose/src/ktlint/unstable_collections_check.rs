//! Port of `ktlint/UnstableCollectionsCheck.kt`.

use crate::ktlint::ktlint_rule::KtlintRule;

// TODO: port it (conventions: crates/ktrs-compose/src/lib.rs), then `Some(unstable_collections_check)`.
pub const CHECK: Option<fn() -> KtlintRule> = None;
