//! `detekt-metrics/.../CyclomaticComplexity.kt`.

use ktrs_psi::{
    KtBinaryExpression, KtBlockExpression, KtBreakExpression, KtCallExpression, KtContinueExpression, KtIfExpression, KtLoopExpression,
    KtNamedFunction, KtObjectLiteralExpression, KtTryExpression, KtWhenEntry, KtWhenExpression, PsiElement, kt_visitor_void,
};
use ktrs_syntax::SyntaxKind::{ANDAND, ELVIS, OROR};

/// `CyclomaticComplexity.DEFAULT_NESTING_FUNCTIONS`.
pub const DEFAULT_NESTING_FUNCTIONS: [&str; 9] = ["run", "let", "apply", "with", "also", "use", "forEach", "isNotNull", "ifNull"];

/// `CyclomaticComplexity.Config`.
pub struct Config {
    pub ignore_simple_when_entries: bool,
    pub ignore_nesting_functions: bool,
    pub ignore_local_functions: bool,
    pub nesting_functions: Vec<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            ignore_simple_when_entries: false,
            ignore_nesting_functions: false,
            ignore_local_functions: false,
            nesting_functions: DEFAULT_NESTING_FUNCTIONS.iter().map(|name| (*name).to_owned()).collect(),
        }
    }
}

pub struct CyclomaticComplexity {
    config: Config,
    complexity: i32,
}

impl CyclomaticComplexity {
    pub fn new(config: Config) -> CyclomaticComplexity {
        CyclomaticComplexity { config, complexity: 0 }
    }

    pub fn complexity(&self) -> i32 {
        self.complexity
    }

    fn extract_entries(&self, expression: &KtWhenExpression) -> Vec<KtWhenEntry> {
        let entries = expression.entries();
        if self.config.ignore_simple_when_entries {
            entries.into_iter().filter(|it| it.expression().is_some_and(|e| e.is::<KtBlockExpression>())).collect()
        } else {
            entries
        }
    }

    fn is_used_for_nesting(&self, expression: &KtCallExpression) -> bool {
        expression.get_call_name_expression().is_some_and(|name| self.config.nesting_functions.iter().any(|it| it == name.text_slice()))
    }

    /// `CyclomaticComplexity.calculate(node, configure)`.
    pub fn calculate(node: &PsiElement, configure: impl FnOnce(&mut Config)) -> i32 {
        let mut config = Config::default();
        configure(&mut config);
        let mut visitor = CyclomaticComplexity::new(config);
        node.accept(&mut visitor);
        visitor.complexity
    }
}

fn is_inside_object_literal(function: &KtNamedFunction) -> bool {
    function.get_parent_of_type::<KtObjectLiteralExpression>(true).is_some()
}

fn is_inside_named_function(function: &KtNamedFunction) -> bool {
    function.get_parent_of_type::<KtNamedFunction>(true).is_some()
}

crate::detekt_visitor! {
    impl CyclomaticComplexity {
        fn visit_named_function(&mut self, function: &KtNamedFunction) {
            if !is_inside_object_literal(function) {
                self.complexity += 1;
                if !is_inside_named_function(function) || !self.config.ignore_local_functions {
                    kt_visitor_void::visit_named_function(self, function);
                }
            }
        }

        fn visit_binary_expression(&mut self, expression: &KtBinaryExpression) {
            if matches!(expression.operation_token(), Some(ELVIS | ANDAND | OROR)) {
                self.complexity += 1;
            }
            kt_visitor_void::visit_binary_expression(self, expression);
        }

        fn visit_continue_expression(&mut self, expression: &KtContinueExpression) {
            self.complexity += 1;
            kt_visitor_void::visit_continue_expression(self, expression);
        }

        fn visit_break_expression(&mut self, expression: &KtBreakExpression) {
            self.complexity += 1;
            kt_visitor_void::visit_break_expression(self, expression);
        }

        fn visit_if_expression(&mut self, expression: &KtIfExpression) {
            self.complexity += 1;
            kt_visitor_void::visit_if_expression(self, expression);
        }

        fn visit_loop_expression(&mut self, loop_expression: &KtLoopExpression) {
            self.complexity += 1;
            kt_visitor_void::visit_loop_expression(self, loop_expression);
        }

        fn visit_when_expression(&mut self, expression: &KtWhenExpression) {
            let entries = self.extract_entries(expression);
            self.complexity += if self.config.ignore_simple_when_entries && entries.is_empty() { 1 } else { entries.len() as i32 };
            kt_visitor_void::visit_when_expression(self, expression);
        }

        fn visit_try_expression(&mut self, expression: &KtTryExpression) {
            self.complexity += expression.catch_clauses().len() as i32;
            kt_visitor_void::visit_try_expression(self, expression);
        }

        fn visit_call_expression(&mut self, expression: &KtCallExpression) {
            if !self.config.ignore_nesting_functions && self.is_used_for_nesting(expression) {
                let lambda_expression = expression.lambda_arguments().first().and_then(|argument| argument.get_lambda_expression());
                if lambda_expression.is_some_and(|lambda| lambda.body_expression().is_some()) {
                    self.complexity += 1;
                }
            }
            kt_visitor_void::visit_call_expression(self, expression);
        }
    }
}
