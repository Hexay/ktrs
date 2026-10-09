//! `detekt-rules-exceptions`. `ExceptionsProvider.kt` is [`provider`]; ported so far: the rules listed there.

mod swallowed_exception;
mod too_generic_exception_caught;

pub use swallowed_exception::SwallowedException;
pub use too_generic_exception_caught::TooGenericExceptionCaught;

use crate::api::{RuleSet, RuleSetId, RuleSetProvider};

/// `ExceptionsProvider`.
pub fn provider() -> RuleSetProvider {
    RuleSetProvider { rule_set_id: RuleSetId::new("exceptions"), instance, is_default: true }
}

fn instance() -> RuleSet {
    RuleSet::new(RuleSetId::new("exceptions"), crate::rule_providers![SwallowedException, TooGenericExceptionCaught])
}
