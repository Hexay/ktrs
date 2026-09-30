//! Shared helpers for the ported ktlint engine tests.
#![allow(dead_code)]
// The DSL keeps KtLintAssertThat's names (`asKotlinScript`, `isFormattedAs`).
#![allow(clippy::wrong_self_convention)]

pub mod assert_that;
pub mod insert;
pub mod suppression_rule;
