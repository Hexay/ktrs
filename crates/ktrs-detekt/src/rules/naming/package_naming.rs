//! `PackageNaming.kt`.

use std::cell::OnceCell;
use std::sync::Arc;

use ktrs_psi::KtPackageDirective;

use crate::api::{Config, Entity, Finding, Rule, RuleBase, config_property};
use crate::kotlin::Regex;

/// Reports package names that do not follow the specified naming convention.
pub struct PackageNaming {
    base: RuleBase,
    package_pattern: OnceCell<Regex>,
}

impl PackageNaming {
    pub fn new(config: Arc<dyn Config>) -> Self {
        PackageNaming {
            base: RuleBase::new(config, "Package names should follow the naming convention set in detekt's configuration."),
            package_pattern: OnceCell::new(),
        }
    }

    fn package_pattern(&self) -> &Regex {
        self.package_pattern.get_or_init(|| {
            Regex::new(&config_property::string(self.base.config.as_ref(), "packagePattern", r"[a-z]+(\.[a-z][A-Za-z0-9]*)*"))
        })
    }
}

impl Rule for PackageNaming {
    crate::rule_base!(PackageNaming);
}

crate::detekt_visitor! {
    impl PackageNaming {
        fn visit_package_directive(&mut self, directive: &KtPackageDirective) {
            let name = directive.qualified_name();
            if !name.is_empty() && !self.package_pattern().matches(&name) {
                let message = format!("Package name should match the pattern: {}", self.package_pattern());
                self.report(Finding::new(Entity::from(directive), message));
            }
        }
    }
}
