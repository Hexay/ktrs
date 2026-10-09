//! `CyclomaticComplexMethod.kt`.

use std::cell::OnceCell;
use std::sync::Arc;

use ktrs_psi::{KtBlockExpression, KtExpression, KtNamedFunction, KtReturnExpression, KtWhenExpression};

use crate::api::{Config, Entity, Finding, Rule, RuleBase, config_property};
use crate::metrics::CyclomaticComplexity;

/// `DEFAULT_NESTING_FUNCTIONS`.
pub const DEFAULT_NESTING_FUNCTIONS: [&str; 9] = ["also", "apply", "forEach", "isNotNull", "ifNull", "let", "run", "use", "with"];

/// Complex methods are hard to understand and read. This rule uses McCabe's Cyclomatic Complexity (MCC) metric
/// to measure the number of linearly independent paths through a function's source code.
pub struct CyclomaticComplexMethod {
    base: RuleBase,
    allowed_complexity: OnceCell<i32>,
    ignore_single_when_expression: OnceCell<bool>,
    ignore_simple_when_entries: OnceCell<bool>,
    ignore_nesting_functions: OnceCell<bool>,
    ignore_local_functions: OnceCell<bool>,
    nesting_functions: OnceCell<Vec<String>>,
}

macro_rules! flag {
    ($name:ident, $key:literal) => {
        fn $name(&self) -> bool {
            *self.$name.get_or_init(|| config_property::boolean(self.base.config.as_ref(), $key, false))
        }
    };
}

impl CyclomaticComplexMethod {
    pub fn new(config: Arc<dyn Config>) -> Self {
        CyclomaticComplexMethod {
            base: RuleBase::new(config, "Prefer splitting up complex methods into smaller, easier to test methods."),
            allowed_complexity: OnceCell::new(),
            ignore_single_when_expression: OnceCell::new(),
            ignore_simple_when_entries: OnceCell::new(),
            ignore_nesting_functions: OnceCell::new(),
            ignore_local_functions: OnceCell::new(),
            nesting_functions: OnceCell::new(),
        }
    }

    fn allowed_complexity(&self) -> i32 {
        *self.allowed_complexity.get_or_init(|| config_property::int(self.base.config.as_ref(), "allowedComplexity", 14))
    }

    flag!(ignore_single_when_expression, "ignoreSingleWhenExpression");
    flag!(ignore_simple_when_entries, "ignoreSimpleWhenEntries");
    flag!(ignore_nesting_functions, "ignoreNestingFunctions");
    flag!(ignore_local_functions, "ignoreLocalFunctions");

    fn nesting_functions(&self) -> &[String] {
        self.nesting_functions.get_or_init(|| config_property::list(self.base.config.as_ref(), "nestingFunctions", &DEFAULT_NESTING_FUNCTIONS))
    }
}

impl Rule for CyclomaticComplexMethod {
    crate::rule_base!(CyclomaticComplexMethod);
}

crate::detekt_visitor! {
    impl CyclomaticComplexMethod {
        fn visit_named_function(&mut self, function: &KtNamedFunction) {
            if self.ignore_single_when_expression() && has_single_when_expression(function.body_expression()) {
                return;
            }

            let complexity = CyclomaticComplexity::calculate(function, |config| {
                config.ignore_simple_when_entries = self.ignore_simple_when_entries();
                config.ignore_nesting_functions = self.ignore_nesting_functions();
                config.ignore_local_functions = self.ignore_local_functions();
                config.nesting_functions = self.nesting_functions().to_vec();
            });

            if complexity > self.allowed_complexity() {
                let message = format!(
                    "The function {} appears to be too complex based on Cyclomatic Complexity (complexity: {complexity}). \
                     The maximum allowed complexity for methods is set to '{}'",
                    function.name_as_safe_name(),
                    self.allowed_complexity()
                );
                self.report(Finding::new(Entity::at_name(function), message));
            }
        }
    }
}

fn has_single_when_expression(body_expression: Option<KtExpression>) -> bool {
    let Some(body_expression) = body_expression else { return false };
    if let Some(block) = body_expression.cast::<KtBlockExpression>() {
        let statements = block.statements();
        if statements.len() == 1 {
            let statement = &statements[0];
            return statement.is::<KtWhenExpression>() || returns_when_expression(statement);
        }
    }
    // the case where function-expression syntax is used: `fun test() = when { ... }`
    body_expression.is::<KtWhenExpression>()
}

fn returns_when_expression(expression: &KtExpression) -> bool {
    expression.cast::<KtReturnExpression>().and_then(|it| it.returned_expression()).is_some_and(|returned| returned.is::<KtWhenExpression>())
}
