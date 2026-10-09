//! The ported rule sets: one module per `detekt-rules-*`, one file per rule, in upstream's layout.

pub mod complexity;
pub mod emptyblocks;
pub mod exceptions;
pub mod naming;
pub mod style;

use crate::api::RuleSetProvider;

/// The `RuleSetProvider` services of detekt-cli, in rule set id order (`getRules` follows the providers; the
/// JVM's `ServiceLoader` order is the order of the jar's service file).
// TODO: comments, coroutines, performance, potential-bugs; the rule sets of the three plugin jars
pub fn default_rule_set_providers() -> Vec<RuleSetProvider> {
    vec![complexity::provider(), emptyblocks::provider(), exceptions::provider(), naming::provider(), style::provider()]
}
