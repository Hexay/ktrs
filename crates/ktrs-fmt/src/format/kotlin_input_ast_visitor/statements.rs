//! `KotlinInputAstVisitor.kt` lines 2515-2662: loops, `break`/`continue`, parameters, callable
//! references, class literals, function types.

use ktrs_psi::*;

use crate::doc::{FillMode, Indent};

use super::KotlinInputAstVisitor;

impl KotlinInputAstVisitor<'_, '_, '_> {
    /// Example `for (i in items) { ... }`
    pub(super) fn visit_for_expression(&mut self, expression: &KtForExpression) {
        self.sync(expression);
        let ebi = self.expression_break_indent();
        self.block(Indent::ZERO, |v| {
            v.token("for");
            v.builder.space();
            v.token("(");
            v.visit(expression.loop_parameter().as_ref());
            v.builder.space();
            v.token("in");
            v.block(Indent::ZERO, |v| {
                v.builder.break_op(FillMode::Unified, " ", ebi.clone());
                v.block(ebi.clone(), |v| v.visit(expression.loop_range().as_ref()));
            });
            v.token(")");
            v.builder.space();
            v.visit(expression.body().as_ref());
        });
    }

    /// Example `while (a < b) { ... }`
    pub(super) fn visit_while_expression(&mut self, expression: &KtWhileExpression) {
        self.sync(expression);
        self.emit_keyword_with_condition("while", expression.condition().as_ref(), true);
        self.builder.space();
        self.visit(expression.body().as_ref());
    }

    /// Example `do { ... } while (a < b)`
    pub(super) fn visit_do_while_expression(&mut self, expression: &KtDoWhileExpression) {
        self.sync(expression);
        self.token("do");
        self.builder.space();
        if let Some(body) = expression.body() {
            self.visit(Some(&body));
            self.builder.space();
        }
        self.emit_keyword_with_condition("while", expression.condition().as_ref(), true);
    }

    /// Example `break` or `break@foo` in a loop
    pub(super) fn visit_break_expression(&mut self, expression: &KtBreakExpression) {
        self.sync(expression);
        self.token("break");
        self.visit(expression.label_qualifier().as_ref());
    }

    /// Example `continue` or `continue@foo` in a loop
    pub(super) fn visit_continue_expression(&mut self, expression: &KtContinueExpression) {
        self.sync(expression);
        self.token("continue");
        self.visit(expression.label_qualifier().as_ref());
    }

    /// Example `f: String`, or `private val n: Int` or `(a: Int, b: String)` (in for-loops)
    pub(super) fn visit_parameter(&mut self, parameter: &KtParameter) {
        self.sync(parameter);
        self.block(Indent::ZERO, |v| {
            let destructuring_declaration = parameter.destructuring_declaration();
            let type_reference = parameter.type_reference();
            if let Some(destructuring_declaration) = destructuring_declaration {
                v.block(Indent::ZERO, |v| {
                    v.visit(Some(&destructuring_declaration));
                    if let Some(type_reference) = &type_reference {
                        v.token(":");
                        v.builder.space();
                        v.visit(Some(type_reference));
                    }
                });
            } else {
                let val_or_var_keyword = parameter.val_or_var_keyword().map(|k| k.text());
                let name = parameter.name_identifier().map(|n| n.text());
                v.declare_one(
                    parameter.modifier_list().as_ref(),
                    val_or_var_keyword.as_deref(),
                    None,
                    None,
                    name.as_deref(),
                    type_reference.as_ref(),
                    None,
                    parameter.default_value().as_ref(),
                    None,
                    None,
                    None,
                );
            }
        });
    }

    /// Example `String::isNullOrEmpty`
    pub(super) fn visit_callable_reference_expression(&mut self, expression: &KtCallableReferenceExpression) {
        self.sync(expression);
        self.visit(expression.receiver_expression().as_ref());

        // The receiver doesn't contain the `?` of a nullable type (`String?::isNullOrEmpty`).
        if expression.has_question_marks() {
            self.token("?");
        }

        self.block(self.expression_break_indent(), |v| {
            v.token("::");
            v.builder.break_op(FillMode::Independent, "", Indent::ZERO);
            v.visit(expression.callable_reference().as_ref());
        });
    }

    pub(super) fn visit_class_literal_expression(&mut self, expression: &KtClassLiteralExpression) {
        self.sync(expression);
        let receiver_expression = expression.receiver_expression();
        if let Some(call) = receiver_expression.as_ref().and_then(|r| r.cast::<KtCallExpression>()) {
            self.visit_call_element(
                call.callee_expression().as_ref(),
                call.type_argument_list().as_ref(),
                call.value_argument_list().as_ref(),
                &call.lambda_arguments(),
                self.expression_break_indent(),
                Indent::ZERO,
                Indent::ZERO,
            );
        } else {
            self.visit(receiver_expression.as_ref());
        }
        self.token("::");
        self.token("class");
    }

    pub(super) fn visit_function_type(&mut self, type_: &KtFunctionType) {
        self.sync(type_);

        if let Some(function_type_context_receiver_list) = type_.context_receiver_list() {
            self.visit_context_receiver_list(&function_type_context_receiver_list);
            self.builder.space();
        }

        if let Some(receiver) = type_.receiver() {
            self.visit(Some(&receiver));
            self.token(".");
        }
        self.block(self.expression_break_indent(), |v| v.visit(type_.parameter_list().as_ref()));
        self.builder.space();
        self.token("->");
        self.builder.space();
        self.block(self.expression_break_indent(), |v| v.visit(type_.return_type_reference().as_ref()));
    }
}
