//! `TooGenericExceptionCaught.kt`.

use std::cell::OnceCell;
use std::sync::Arc;

use ktrs_psi::{KtCatchClause, KtTypeReference, kt_visitor_void};

use crate::api::{Config, Entity, Finding, Rule, RuleBase, config_property};
use crate::kotlin::Regex;
use crate::psi::is_allowed_exception_name;

/// `caughtExceptionDefaults`.
pub const CAUGHT_EXCEPTION_DEFAULTS: [&str; 8] = [
    "ArrayIndexOutOfBoundsException",
    "Error",
    "Exception",
    "IllegalMonitorStateException",
    "IndexOutOfBoundsException",
    "NullPointerException",
    "RuntimeException",
    "Throwable",
];

/// This rule reports `catch` blocks for exceptions that have a type that is too generic.
pub struct TooGenericExceptionCaught {
    base: RuleBase,
    exception_names: OnceCell<Vec<String>>,
    allowed_exception_name_regex: OnceCell<Regex>,
}

impl TooGenericExceptionCaught {
    pub fn new(config: Arc<dyn Config>) -> Self {
        TooGenericExceptionCaught {
            base: RuleBase::new(
                config,
                "The caught exception is too generic. Prefer catching specific exceptions to the case that is currently handled.",
            ),
            exception_names: OnceCell::new(),
            allowed_exception_name_regex: OnceCell::new(),
        }
    }

    fn exception_names(&self) -> &[String] {
        self.exception_names.get_or_init(|| config_property::list(self.base.config.as_ref(), "exceptionNames", &CAUGHT_EXCEPTION_DEFAULTS))
    }

    fn allowed_exception_name_regex(&self) -> &Regex {
        self.allowed_exception_name_regex.get_or_init(|| {
            Regex::new(&config_property::string(self.base.config.as_ref(), "allowedExceptionNameRegex", "_|(ignore|expected).*"))
        })
    }

    fn is_too_generic_exception(&self, type_reference: Option<KtTypeReference>) -> bool {
        type_reference.is_some_and(|type_reference| self.exception_names().iter().any(|name| name == type_reference.text_slice()))
    }
}

impl Rule for TooGenericExceptionCaught {
    crate::rule_base!(TooGenericExceptionCaught);
}

crate::detekt_visitor! {
    impl TooGenericExceptionCaught {
        fn visit_catch_section(&mut self, catch_clause: &KtCatchClause) {
            if let Some(catch_parameter) = catch_clause.catch_parameter()
                && self.is_too_generic_exception(catch_parameter.type_reference())
                && !is_allowed_exception_name(catch_clause, self.allowed_exception_name_regex())
            {
                let description = self.base.description;
                self.report(Finding::new(Entity::from(&catch_parameter), description));
            }
            kt_visitor_void::visit_catch_section(self, catch_clause);
        }
    }
}
