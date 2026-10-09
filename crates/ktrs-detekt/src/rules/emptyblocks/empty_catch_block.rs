//! `EmptyCatchBlock.kt`.

use std::cell::OnceCell;
use std::sync::Arc;

use ktrs_psi::{KtCatchClause, kt_visitor_void};

use super::empty_rule::EmptyRule;
use crate::api::{Config, Rule, RuleBase, config_property};
use crate::kotlin::Regex;
use crate::psi::is_allowed_exception_name;

/// Reports empty `catch` blocks. In case exceptions are ignored intentionally, this should be made explicit by
/// using the specified names in the `allowedExceptionNameRegex`.
pub struct EmptyCatchBlock {
    base: RuleBase,
    allowed_exception_name_regex: OnceCell<Regex>,
}

impl EmptyCatchBlock {
    pub fn new(config: Arc<dyn Config>) -> Self {
        EmptyCatchBlock {
            base: RuleBase::new(
                config,
                "Empty catch block detected. Empty catch blocks indicate that an exception is ignored and not handled.",
            ),
            allowed_exception_name_regex: OnceCell::new(),
        }
    }

    fn allowed_exception_name_regex(&self) -> &Regex {
        self.allowed_exception_name_regex.get_or_init(|| {
            Regex::new(&config_property::string(self.base.config.as_ref(), "allowedExceptionNameRegex", "_|(ignore|expected).*"))
        })
    }
}

impl Rule for EmptyCatchBlock {
    crate::rule_base!(EmptyCatchBlock);
}

impl EmptyRule for EmptyCatchBlock {
    fn finding_message(&self) -> &'static str {
        "Empty catch block detected. If the exception can be safely ignored, \
         name the exception according to one of the exemptions as per the configuration of this rule."
    }
}

crate::detekt_visitor! {
    impl EmptyCatchBlock {
        fn visit_catch_section(&mut self, catch_clause: &KtCatchClause) {
            kt_visitor_void::visit_catch_section(self, catch_clause);
            if is_allowed_exception_name(catch_clause, self.allowed_exception_name_regex()) {
                return;
            }
            if let Some(catch_body) = catch_clause.catch_body() {
                self.add_finding_if_block_expr_is_empty(&catch_body);
            }
        }
    }
}
