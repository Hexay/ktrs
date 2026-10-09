//! `EmptyFunctionBlock.kt`.

use std::cell::OnceCell;
use std::sync::Arc;

use ktrs_psi::{KtClass, KtNamedFunction, kt_visitor_void};

use super::empty_rule::{DESCRIPTION, EmptyRule};
use crate::api::{Config, Rule, RuleBase, config_property};
use crate::psi::{is_open, is_override};

/// Reports empty functions. This rule will not report functions with the override modifier that have a comment
/// as their only body contents (e.g., a `// no-op` comment in an unused listener function).
///
/// Set the `ignoreOverridden` parameter to `true` to exclude all functions which are overriding other functions
/// from the superclass or from an interface (i.e., functions declared with the override modifier).
pub struct EmptyFunctionBlock {
    base: RuleBase,
    ignore_overridden: OnceCell<bool>,
}

impl EmptyFunctionBlock {
    pub fn new(config: Arc<dyn Config>) -> Self {
        EmptyFunctionBlock { base: RuleBase::new(config, DESCRIPTION), ignore_overridden: OnceCell::new() }
    }

    fn ignore_overridden(&self) -> bool {
        *self.ignore_overridden.get_or_init(|| config_property::boolean(self.base.config.as_ref(), "ignoreOverridden", false))
    }
}

impl Rule for EmptyFunctionBlock {
    crate::rule_base!(EmptyFunctionBlock);
}

impl EmptyRule for EmptyFunctionBlock {}

crate::detekt_visitor! {
    impl EmptyFunctionBlock {
        fn visit_named_function(&mut self, function: &KtNamedFunction) {
            kt_visitor_void::visit_named_function(self, function);
            if is_open(function) || is_default_function(function) {
                return;
            }
            let body_expression = function.body_expression();
            if !self.ignore_overridden() {
                if is_override(function) {
                    if let Some(body_expression) = &body_expression {
                        self.add_finding_if_block_expr_is_empty_and_not_commented(body_expression);
                    }
                } else if let Some(body_expression) = &body_expression {
                    self.add_finding_if_block_expr_is_empty(body_expression);
                }
            } else if !is_override(function)
                && let Some(body_expression) = &body_expression
            {
                self.add_finding_if_block_expr_is_empty(body_expression);
            }
        }
    }
}

fn is_default_function(function: &KtNamedFunction) -> bool {
    function.get_parent_of_type::<KtClass>(true).is_some_and(|class| class.is_interface()) && function.has_body()
}
