//! `detekt-rules-complexity`. `ComplexityProvider.kt` is [`provider`]; ported so far: the rules listed there.

mod cyclomatic_complex_method;
mod long_method;
mod too_many_functions;

pub use cyclomatic_complex_method::CyclomaticComplexMethod;
pub use long_method::LongMethod;
pub use too_many_functions::TooManyFunctions;

use crate::api::{RuleSet, RuleSetId, RuleSetProvider};

/// `ComplexityProvider`.
pub fn provider() -> RuleSetProvider {
    RuleSetProvider { rule_set_id: RuleSetId::new("complexity"), instance, is_default: true }
}

fn instance() -> RuleSet {
    RuleSet::new(RuleSetId::new("complexity"), crate::rule_providers![CyclomaticComplexMethod, LongMethod, TooManyFunctions])
}
