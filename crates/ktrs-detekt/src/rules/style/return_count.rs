//! `ReturnCount.kt`.

use std::cell::OnceCell;
use std::sync::Arc;

use ktrs_psi::{KtLambdaExpression, KtNamedFunction, KtReturnExpression, PsiElement, kt_visitor_void};

use super::junk::yield_statements_skipping_guard_clauses;
use crate::api::{Config, Entity, Finding, Rule, RuleBase, config_property};
use crate::kotlin::Regex;
use crate::psi::path_glob_to_regex;

/// Restrict the number of return methods allowed in methods.
pub struct ReturnCount {
    base: RuleBase,
    max: OnceCell<i32>,
    excluded_functions: OnceCell<Vec<Regex>>,
    exclude_labeled: OnceCell<bool>,
    exclude_return_from_lambda: OnceCell<bool>,
    exclude_guard_clauses: OnceCell<bool>,
}

impl ReturnCount {
    pub fn new(config: Arc<dyn Config>) -> Self {
        ReturnCount {
            base: RuleBase::new(config, "Restrict the number of return statements in methods."),
            max: OnceCell::new(),
            excluded_functions: OnceCell::new(),
            exclude_labeled: OnceCell::new(),
            exclude_return_from_lambda: OnceCell::new(),
            exclude_guard_clauses: OnceCell::new(),
        }
    }

    fn max(&self) -> i32 {
        *self.max.get_or_init(|| config_property::int(self.base.config.as_ref(), "max", 2))
    }

    fn excluded_functions(&self) -> &[Regex] {
        self.excluded_functions.get_or_init(|| {
            config_property::list(self.base.config.as_ref(), "excludedFunctions", &["equals"]).iter().map(|it| path_glob_to_regex(it)).collect()
        })
    }

    fn exclude_labeled(&self) -> bool {
        *self.exclude_labeled.get_or_init(|| config_property::boolean(self.base.config.as_ref(), "excludeLabeled", false))
    }

    fn exclude_return_from_lambda(&self) -> bool {
        *self.exclude_return_from_lambda.get_or_init(|| config_property::boolean(self.base.config.as_ref(), "excludeReturnFromLambda", true))
    }

    fn exclude_guard_clauses(&self) -> bool {
        *self.exclude_guard_clauses.get_or_init(|| config_property::boolean(self.base.config.as_ref(), "excludeGuardClauses", false))
    }

    fn should_be_ignored(&self, function: &KtNamedFunction) -> bool {
        function.name().is_some_and(|name| self.excluded_functions().iter().any(|it| it.matches(&name)))
    }

    fn count_return_statements(&self, function: &KtNamedFunction) -> usize {
        let is_excluded = |it: &KtReturnExpression| {
            (self.exclude_return_from_lambda() && is_named_return_from_lambda(it)) || (self.exclude_labeled() && it.labeled_expression().is_some())
        };

        let statements = if self.exclude_guard_clauses() {
            yield_statements_skipping_guard_clauses::<KtReturnExpression>(function)
        } else {
            function.body_block_expression().map(|block| block.statements()).unwrap_or_default()
        };

        let mut count = 0;
        for statement in &statements {
            // Upstream collects the returns (`collectDescendantsOfType`, postorder) and then counts them.
            statement.for_each_descendant_of_type_in_preorder::<KtReturnExpression>(|it| {
                if !is_excluded(&it) && it.get_parent_of_type::<KtNamedFunction>(true).as_ref() == Some(function) {
                    count += 1;
                }
            });
        }
        count
    }
}

impl Rule for ReturnCount {
    crate::rule_base!(ReturnCount);
}

crate::detekt_visitor! {
    impl ReturnCount {
        fn visit_named_function(&mut self, function: &KtNamedFunction) {
            kt_visitor_void::visit_named_function(self, function);

            if !self.should_be_ignored(function) {
                let number_of_returns = self.count_return_statements(function);

                if number_of_returns as i64 > i64::from(self.max()) {
                    let name = function.name().unwrap_or_else(|| "null".to_owned());
                    let max = self.max();
                    self.report(Finding::new(
                        Entity::at_name(function),
                        format!("Function {name} has {number_of_returns} return statements which exceeds the limit of {max}."),
                    ));
                }
            }
        }
    }
}

fn is_named_return_from_lambda(expression: &KtReturnExpression) -> bool {
    let label = expression.labeled_expression();
    if label.is_some() {
        return get_parent_lambda_before_named_function(expression).is_some();
    }
    false
}

/// `getParentOfType<KtLambdaExpression>(strict = true, KtNamedFunction::class.java)`: the nearest lambda above,
/// unless a named function comes first.
fn get_parent_lambda_before_named_function(element: &PsiElement) -> Option<KtLambdaExpression> {
    let mut element = element.parent();
    while let Some(current) = element {
        if let Some(lambda) = current.cast::<KtLambdaExpression>() {
            return Some(lambda);
        }
        if current.is::<KtNamedFunction>() || current.is_file() {
            return None;
        }
        element = current.parent();
    }
    None
}
