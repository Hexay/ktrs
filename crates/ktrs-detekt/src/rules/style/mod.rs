//! `detekt-rules-style`. `StyleGuideProvider.kt` is [`provider`]; ported so far: the rules listed there.

mod junk;
mod magic_number;
mod max_line_length;
mod return_count;
mod wildcard_import;

pub use magic_number::MagicNumber;
pub use max_line_length::MaxLineLength;
pub use return_count::ReturnCount;
pub use wildcard_import::WildcardImport;

use crate::api::{RuleSet, RuleSetId, RuleSetProvider};

/// `StyleGuideProvider`.
pub fn provider() -> RuleSetProvider {
    RuleSetProvider { rule_set_id: RuleSetId::new("style"), instance, is_default: true }
}

fn instance() -> RuleSet {
    RuleSet::new(RuleSetId::new("style"), crate::rule_providers![ReturnCount, WildcardImport, MaxLineLength, MagicNumber])
}
