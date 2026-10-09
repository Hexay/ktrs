//! `FunctionParameterNaming.kt`.

use std::cell::OnceCell;
use std::sync::Arc;

use ktrs_psi::{KtNamedFunction, KtParameter};

use super::exclude_class::is_containing_excluded_class;
use crate::api::{Config, Entity, Finding, Rule, RuleBase, config_property};
use crate::kotlin::Regex;
use crate::psi::is_override;

/// Reports function parameter names that do not follow the specified naming convention.
pub struct FunctionParameterNaming {
    base: RuleBase,
    parameter_pattern: OnceCell<Regex>,
    exclude_class_pattern: OnceCell<Regex>,
}

impl FunctionParameterNaming {
    pub fn new(config: Arc<dyn Config>) -> Self {
        FunctionParameterNaming {
            base: RuleBase::new(config, "Function parameter names should follow the naming convention set in detekt's configuration."),
            parameter_pattern: OnceCell::new(),
            exclude_class_pattern: OnceCell::new(),
        }
    }

    fn parameter_pattern(&self) -> &Regex {
        self.parameter_pattern
            .get_or_init(|| Regex::new(&config_property::string(self.base.config.as_ref(), "parameterPattern", "[a-z][A-Za-z0-9]*")))
    }

    fn exclude_class_pattern(&self) -> &Regex {
        self.exclude_class_pattern.get_or_init(|| Regex::new(&config_property::string(self.base.config.as_ref(), "excludeClassPattern", "$^")))
    }

    /// True for the parameters this rule leaves alone: unnamed, not of a named function, or in an excluded class.
    fn is_parameter_in_function(&self, parameter: &KtParameter) -> bool {
        parameter.name_as_safe_name().is_special()
            || parameter.name_identifier().is_none()
            || !parameter.owner_function().is_some_and(|owner| owner.is::<KtNamedFunction>())
            || is_containing_excluded_class(parameter, self.exclude_class_pattern())
    }
}

impl Rule for FunctionParameterNaming {
    crate::rule_base!(FunctionParameterNaming);
}

crate::detekt_visitor! {
    impl FunctionParameterNaming {
        fn visit_parameter(&mut self, parameter: &KtParameter) {
            if self.is_parameter_in_function(parameter) {
                return;
            }

            if parameter.owner_function().is_some_and(|owner| is_override(&owner)) {
                return;
            }

            let Some(identifier) = parameter.name() else { return };
            if !self.parameter_pattern().matches(&identifier) {
                let message = format!("Function parameter names should match the pattern: {}", self.parameter_pattern());
                self.report(Finding::new(Entity::from(parameter), message));
            }
        }
    }
}
