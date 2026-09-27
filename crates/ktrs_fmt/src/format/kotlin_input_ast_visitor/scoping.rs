//! `KotlinInputAstVisitor.kt` lines 1587-1797: lambdas and scoping functions (`= scope { ... }`,
//! `runnnnn { ... }.baz()`).

use ktrs_psi::*;

use crate::doc::{FillMode, Indent};

use super::KotlinInputAstVisitor;

impl KotlinInputAstVisitor<'_, '_> {
    /// Whether an expression is a lambda or scoping function whose block we don't want to indent:
    /// `{ ... }`, `Runnable { ... }`, `scope { ... }`, `scope.launch { ... }`, but not
    /// `foo() { ... }` (parens) or `Runnable @Annotation { ... }`.
    pub(super) fn is_lambda_or_scoping_function(&self, expression: Option<&KtExpression>) -> bool {
        let Some(expression) = expression else { return false };
        let prev = expression.get_prev_sibling_ignoring_whitespace(false);
        if prev.is_some_and(|p| p.is::<PsiComment>() && p.text_slice().starts_with("//")) {
            return false; // Leading line comments cause weird indentation; block comments are ok.
        }

        let mut carry = Some(expression.clone());
        if let Some(qualified) = carry.as_ref().and_then(|c| c.cast::<KtQualifiedExpression>()) {
            if qualified.receiver_expression().is_some_and(|r| r.is::<KtSimpleNameExpression>()) {
                carry = qualified.selector_expression();
            }
        }
        if let Some(call) = carry.as_ref().and_then(|c| c.cast::<KtCallExpression>()) {
            let lambda_arguments = call.lambda_arguments();
            if call.value_argument_list().and_then(|l| l.left_parenthesis()).is_none()
                && !lambda_arguments.is_empty()
                && call.type_argument_list().is_none_or(|l| l.arguments().is_empty())
            {
                carry = lambda_arguments[0].argument_expression();
            } else {
                return false;
            }
        }
        if let Some(labeled) = carry.as_ref().and_then(|c| c.cast::<KtLabeledExpression>()) {
            carry = labeled.base_expression();
        }
        carry.is_some_and(|c| c.is::<KtLambdaExpression>())
    }

    /// Whether `expression` is a chain whose innermost receiver is a scoping function call, like
    /// `runnnnn { ... }.baz()`.
    pub(super) fn is_chained_scoping_function(&self, expression: &PsiElement) -> bool {
        if !expression.is::<KtQualifiedExpression>() {
            return false;
        }
        self.is_lambda_or_scoping_function(Some(&self.chain_root(expression)))
    }

    /// Returns the innermost receiver of a (possibly nested) qualified [expression].
    pub(super) fn chain_root(&self, expression: &PsiElement) -> KtExpression {
        let mut root: KtExpression = expression.upcast();
        while let Some(receiver) = root.cast::<KtQualifiedExpression>().and_then(|q| q.receiver_expression()) {
            root = receiver;
        }
        root
    }

    /// Whether any chained selector after the innermost receiver has value arguments (`.foo(a)`).
    fn chained_selectors_have_value_arguments(&self, expression: &PsiElement) -> bool {
        let mut current: Option<KtExpression> = Some(expression.upcast());
        while let Some(qualified) = current.as_ref().and_then(|c| c.cast::<KtQualifiedExpression>()) {
            let selector = qualified.selector_expression();
            if selector
                .and_then(|s| s.cast::<KtCallExpression>())
                .is_some_and(|s| s.value_argument_list().is_some_and(|l| !l.arguments().is_empty()))
            {
                return true;
            }
            current = qualified.receiver_expression();
        }
        false
    }

    /// Whether no chained selector passes value arguments; those chains keep the general layout.
    pub(super) fn chained_selectors_have_no_value_arguments(&self, expression: &PsiElement) -> bool {
        !self.chained_selectors_have_value_arguments(expression)
    }

    /// Emits `runnnnn { ... }.baz().qux()`: the scoping-function receiver block-like, then each
    /// `.selector` as a continuation; a multi-line receiver lambda forces them onto their own lines.
    pub(super) fn visit_chained_scoping_function(&mut self, expression: &KtQualifiedExpression, emit_leading_break: bool) {
        let parts = self.break_into_parts(&expression.upcast());
        let root = parts[0].clone();
        let force_break_before_chain = self.is_multiline_scoping_function(&root);

        self.visit_lambda_or_scoping_function(Some(&root), emit_leading_break);

        self.block(self.expression_break_indent(), |v| {
            for part in &parts[1..] {
                let Some(part) = part.cast::<KtQualifiedExpression>() else { return v.fail() };
                if force_break_before_chain {
                    v.builder.forced_break();
                } else {
                    v.builder.break_op(FillMode::Unified, "", Indent::ZERO);
                }
                let Some(operation_sign) = part.operation_sign() else { return v.fail() };
                v.token(operation_sign.value());
                let selector_expression = part.selector_expression();
                if let Some(call) = selector_expression.as_ref().and_then(|s| s.cast::<KtCallExpression>()) {
                    v.visit(call.callee_expression().as_ref());
                    v.visit_call_element(
                        None,
                        call.type_argument_list().as_ref(),
                        call.value_argument_list().as_ref(),
                        &call.lambda_arguments(),
                        v.expression_break_indent(),
                        Indent::ZERO,
                        Indent::ZERO,
                    );
                } else {
                    v.visit(selector_expression.as_ref());
                }
            }
        });
    }

    /// Whether `expression` is a scoping-function call whose lambda body spans multiple source lines.
    pub(super) fn is_multiline_scoping_function(&self, expression: &KtExpression) -> bool {
        let mut carry = Some(expression.clone());
        if let Some(qualified) = carry.as_ref().and_then(|c| c.cast::<KtQualifiedExpression>()) {
            if qualified.receiver_expression().is_some_and(|r| r.is::<KtSimpleNameExpression>()) {
                carry = qualified.selector_expression();
            }
        }
        if let Some(call) = carry.as_ref().and_then(|c| c.cast::<KtCallExpression>()) {
            carry = call.lambda_arguments().first().and_then(|a| a.argument_expression());
        }
        if let Some(labeled) = carry.as_ref().and_then(|c| c.cast::<KtLabeledExpression>()) {
            carry = labeled.base_expression();
        }
        match carry.and_then(|c| c.cast::<KtLambdaExpression>()) {
            Some(lambda) => self.has_source_newline_in_lambda_body(&lambda),
            None => false,
        }
    }

    /// Whether the source has a newline between the `{` and `}` of the lambda's function literal
    /// (`FormattingOptions.preserveLambdaBreaks`).
    pub(super) fn has_source_newline_in_lambda_body(&self, lambda_expression: &KtLambdaExpression) -> bool {
        let Some(function_literal) = lambda_expression.function_literal() else { return false };
        function_literal.node().children().any(|child| child.psi().is::<PsiWhiteSpace>() && child.text_contains('\n'))
    }

    /// See [Self::is_lambda_or_scoping_function] for examples.
    pub(super) fn visit_lambda_or_scoping_function<T: PsiType>(&mut self, expr: Option<&T>, emit_leading_break: bool) {
        let break_to_expr = self.gen_sym();
        let break_space = if emit_leading_break { " " } else { "" };
        self.builder.break_op_tagged(
            FillMode::Independent,
            break_space,
            self.expression_break_indent(),
            Some(break_to_expr.clone()),
        );

        let mut carry = expr.map(|e| e.psi().clone());
        if let Some(qualified) = carry.as_ref().and_then(|c| c.cast::<KtQualifiedExpression>()) {
            if let Some(receiver) = qualified.receiver_expression().filter(|r| r.is::<KtSimpleNameExpression>()) {
                let Some(operation_sign) = qualified.operation_sign() else { return self.fail() };
                self.visit(Some(&receiver));
                self.token(operation_sign.value());
                carry = qualified.selector_expression().map(PsiElement::from);
            }
        }
        if let Some(call) = carry.as_ref().and_then(|c| c.cast::<KtCallExpression>()) {
            self.visit(call.callee_expression().as_ref());
            self.builder.space();
            let Some(lambda_argument) = call.lambda_arguments().into_iter().next() else { return self.fail() };
            carry = lambda_argument.argument_expression().map(PsiElement::from);
        }
        if let Some(labeled) = carry.as_ref().and_then(|c| c.cast::<KtLabeledExpression>()) {
            self.visit(labeled.label_qualifier().as_ref());
            let Some(base_expression) = labeled.base_expression() else { return self.fail() };
            carry = Some(base_expression.into());
        }
        if let Some(lambda) = carry.as_ref().and_then(|c| c.cast::<KtLambdaExpression>()) {
            self.visit_lambda_expression_internal(&lambda, Some(break_to_expr));
            return;
        }

        self.throw_runtime("AssertionError");
    }
}
