//! `KotlinInputAstVisitor.kt` lines 2005-2184: modifier lists, annotations, file annotation lists,
//! super type lists.

use ktrs_psi::*;

use crate::doc::{FillMode, Indent};

use super::KotlinInputAstVisitor;
use super::comma_separated::psi_list;

impl KotlinInputAstVisitor<'_, '_> {
    /// For example `@Magic private final`
    pub(super) fn visit_modifier_list(&mut self, list: &KtModifierList) {
        self.sync(list);
        let mut only_annotations_so_far = true;

        for child in list.node().children() {
            let psi = child.psi();
            if psi.is::<PsiWhiteSpace>() {
                continue;
            }

            // In Kotlin 2.3+, context receiver lists are children of the modifier list.
            if let Some(context_receiver_list) = psi.cast::<KtContextReceiverList>() {
                self.visit_context_receiver_list(&context_receiver_list);
                continue;
            }

            if is_modifier_keyword_token(child.element_type()) {
                only_annotations_so_far = false;
                self.token(child.text_slice());
            } else {
                self.visit(Some(&psi));
            }

            if only_annotations_so_far {
                self.builder.break_op(FillMode::Unified, " ", Indent::ZERO);
            } else {
                self.builder.space();
            }
        }
    }

    /// Example: `@SuppressLint("MagicNumber") print(10)` inside a function body.
    pub(super) fn visit_annotated_expression(&mut self, expression: &KtAnnotatedExpression) {
        self.sync(expression);
        self.block(Indent::ZERO, |v| {
            let base_expression = expression.base_expression();

            v.block(Indent::ZERO, |v| {
                let annotation_entries = expression.annotation_entries();
                for (i, annotation_entry) in annotation_entries.iter().enumerate() {
                    if i != 0 {
                        v.builder.break_op(FillMode::Unified, " ", Indent::ZERO);
                    }
                    v.visit(Some(annotation_entry));
                }
            });

            // An annotation on the line above a binary expression in a block applies to the whole
            // expression, on the same line only to the first operand: force a break to keep meaning.
            let base_is = |test: fn(&PsiElement) -> bool| base_expression.as_ref().is_some_and(|b| test(b));
            if (base_is(|b| b.is::<KtBinaryExpression>()) || base_is(|b| b.is::<KtBinaryExpressionWithTypeRHS>()))
                && expression.parent().is_some_and(|p| p.is::<KtBlockExpression>())
            {
                v.builder.forced_break();
            } else if base_is(|b| b.is::<KtLambdaExpression>()) {
                v.builder.space();
            } else if base_is(|b| b.is::<KtReturnExpression>()) {
                v.builder.forced_break();
            } else {
                v.builder.break_op(FillMode::Unified, " ", Indent::ZERO);
            }

            v.visit(expression.base_expression().as_ref());
        });
    }

    /// For example, `@field:[Inject Named("WEB_VIEW")]`: groups annotations with one use-site target.
    pub(super) fn visit_annotation(&mut self, annotation: &KtAnnotation) {
        self.sync(annotation);
        self.block(Indent::ZERO, |v| {
            v.token("@");
            if let Some(use_site_target) = annotation.use_site_target() {
                v.visit(Some(&use_site_target));
                v.token(":");
            }
            v.block(v.expression_break_indent(), |v| {
                v.token("[");

                v.block(Indent::ZERO, |v| {
                    let mut first = true;
                    v.builder.break_op(FillMode::Unified, "", Indent::ZERO);
                    for value in annotation.entries() {
                        if !first {
                            v.builder.break_op(FillMode::Unified, " ", Indent::ZERO);
                        }
                        first = false;

                        v.visit(Some(&value));
                    }
                });
            });
            v.token("]");
        });
        self.builder.forced_break();
    }

    /// For example, 'field' in `@field:[Inject Named("WEB_VIEW")]`
    pub(super) fn visit_annotation_use_site_target(&mut self, annotation_target: &KtAnnotationUseSiteTarget) {
        let Some(target) = annotation_target.annotation_use_site_target() else { return self.fail() };
        self.token(target.render_name());
    }

    /// For example `@Magic` or `@Fred(1, 5)`
    pub(super) fn visit_annotation_entry(&mut self, annotation_entry: &KtAnnotationEntry) {
        self.sync(annotation_entry);
        if annotation_entry.at_symbol().is_some() {
            self.token("@");
        }
        if let Some(use_site_target) = annotation_entry.use_site_target() {
            if use_site_target.parent().as_ref() == Some(annotation_entry.psi()) {
                self.visit(Some(&use_site_target));
                self.token(":");
            }
        }
        let callee = annotation_entry.callee_expression().map(|c| c.upcast::<KtExpression>());
        self.visit_call_element(
            callee.as_ref(),
            None, // Type-arguments are included in the annotation's callee expression.
            annotation_entry.value_argument_list().as_ref(),
            &[],
            self.expression_break_indent(),
            Indent::ZERO,
            Indent::ZERO,
        );
    }

    pub(super) fn visit_file_annotation_list(&mut self, file_annotation_list: &KtFileAnnotationList) {
        for child in file_annotation_list.node().children() {
            if child.is_psi_element() {
                continue;
            }
            self.visit(Some(&child.psi()));
            self.builder.forced_break();
        }
    }

    pub(super) fn visit_super_type_list(&mut self, list: &KtSuperTypeList) {
        self.sync(list);
        self.block(self.expression_break_indent(), |v| {
            v.visit_each_comma_separated(&psi_list(list.entries()), v.comma_separated());
        });
    }

    pub(super) fn visit_super_type_call_entry(&mut self, call: &KtSuperTypeCallEntry) {
        self.sync(call);
        let callee = call.callee_expression().map(|c| c.upcast::<KtExpression>());
        self.visit_call_element(
            callee.as_ref(),
            None,
            call.value_argument_list().as_ref(),
            &call.lambda_arguments(),
            self.expression_break_indent(),
            Indent::ZERO,
            Indent::ZERO,
        );
    }

    /// Example `Collection<Int> by list` in `class MyList(list: List<Int>) : Collection<Int> by list`
    pub(super) fn visit_delegated_super_type_entry(&mut self, specifier: &KtDelegatedSuperTypeEntry) {
        self.sync(specifier);
        self.visit(specifier.type_reference().as_ref());
        self.builder.space();
        self.token("by");
        self.builder.space();
        self.visit(specifier.delegate_expression().as_ref());
    }
}
