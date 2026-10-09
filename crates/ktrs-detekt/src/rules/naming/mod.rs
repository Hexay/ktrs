//! `detekt-rules-naming`. `NamingProvider.kt` is [`provider`]; ported so far: the rules listed there.

mod exclude_class;
mod function_parameter_naming;
mod invalid_package_declaration;
mod package_naming;

pub use function_parameter_naming::FunctionParameterNaming;
pub use invalid_package_declaration::InvalidPackageDeclaration;
pub use package_naming::PackageNaming;

use crate::api::{RuleSet, RuleSetId, RuleSetProvider};

/// `NamingProvider`.
pub fn provider() -> RuleSetProvider {
    RuleSetProvider { rule_set_id: RuleSetId::new("naming"), instance, is_default: true }
}

fn instance() -> RuleSet {
    RuleSet::new(RuleSetId::new("naming"), crate::rule_providers![FunctionParameterNaming, InvalidPackageDeclaration, PackageNaming])
}
