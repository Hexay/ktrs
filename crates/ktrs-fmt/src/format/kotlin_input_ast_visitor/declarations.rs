//! `KotlinInputAstVisitor.kt` lines 1363-1585: `declareOne`, `emitBackingField`,
//! `getParameterListWithBugFixes`.

use ktrs_psi::*;

use crate::doc::{BlankLineWanted, FillMode, Indent};

use super::KotlinInputAstVisitor;
use super::function_like::ParameterList;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DeclarationKind {
    Field,
    Parameter,
}

impl KotlinInputAstVisitor<'_, '_, '_> {
    /// Declare one variable or variable-like thing, e.g. `var a: Int = 5` or `a: Int`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn declare_one(
        &mut self,
        kind: DeclarationKind,
        modifiers: Option<&KtModifierList>,
        val_or_var_keyword: Option<&str>,
        type_parameters: Option<&KtTypeParameterList>,
        receiver: Option<&KtTypeReference>,
        name: Option<&str>,
        type_: Option<&KtTypeReference>,
        type_constraint_list: Option<&KtTypeConstraintList>,
        initializer: Option<&KtExpression>,
        delegate: Option<&KtPropertyDelegate>,
        accessors: Option<&[KtPropertyAccessor]>,
        backing_field: Option<&KtBackingField>,
    ) {
        let vertical_annotation_break = self.gen_sym();

        let is_field = kind == DeclarationKind::Field;

        if is_field {
            self.builder.blank_line_wanted(BlankLineWanted::conditional(&vertical_annotation_break));
        }

        let ebi = self.expression_break_indent();
        self.visit(modifiers);
        self.block(Indent::ZERO, |v| {
            v.block(Indent::ZERO, |v| {
                if let Some(val_or_var_keyword) = val_or_var_keyword {
                    v.token(val_or_var_keyword);
                    v.builder.space();
                }

                if let Some(type_parameters) = type_parameters {
                    v.visit(Some(type_parameters));
                    v.builder.space();
                }

                // conditionally indent the name and initializer +4 if the type spans multiple lines
                if let Some(name) = name {
                    if let Some(receiver) = receiver {
                        v.visit(Some(receiver));
                        v.token(".");
                    }
                    v.token(name);
                }
            });

            v.block_if(ebi.clone(), name.is_some(), |v| {
                // For example `: String` in `val thisIsALongName: String` or `fun f(): String`
                if let Some(type_) = type_ {
                    if name.is_some() {
                        v.token(":");
                        v.builder.break_op(FillMode::Unified, " ", Indent::ZERO);
                    }
                    v.visit(Some(type_));
                }
            });

            // For example `where T : Int` in a generic method
            if let Some(type_constraint_list) = type_constraint_list {
                v.visit(Some(type_constraint_list));
                v.builder.space();
            }

            // for example `by lazy { compute() }`
            if let Some(delegate) = delegate {
                v.builder.space();
                v.token("by");
                let delegate_expr = delegate.expression();
                if v.is_lambda_or_scoping_function(delegate_expr.as_ref()) {
                    v.builder.space();
                    v.visit(Some(delegate));
                } else if let Some(chain) =
                    delegate_expr.as_ref().filter(|d| v.is_chained_scoping_function(d)).and_then(|d| d.cast::<KtQualifiedExpression>())
                {
                    v.visit_chained_scoping_function(&chain, true);
                } else {
                    v.builder.break_op(FillMode::Unified, " ", ebi.clone());
                    v.block(ebi.clone(), |v| {
                        v.fence_comments();
                        v.visit(Some(delegate));
                    });
                }
            } else if let Some(initializer) = initializer {
                v.builder.space();
                v.token("=");
                v.emit_initializer(initializer);
            }
        });
        // for example `field = value`, `private set`, or `get = 2 * field`
        let mut property_components: Vec<PsiElement> = Vec::new();
        if let Some(backing_field) = backing_field {
            property_components.push(backing_field.clone().into());
        }
        if let Some(accessors) = accessors {
            property_components.extend(accessors.iter().cloned().map(PsiElement::from));
        }
        property_components.sort_by_key(PsiElement::start_offset);
        if !property_components.is_empty() {
            self.block(self.block_indent(), |v| {
                for component in &property_components {
                    v.builder.forced_break();
                    // The semicolon must come after the newline, or the output code will not parse.
                    v.builder.guess_token(";");

                    if let Some(component) = component.cast::<KtPropertyAccessor>() {
                        let Some(name_placeholder) = component.name_placeholder() else { return v.fail() };
                        let parameter_list = v.get_parameter_list_with_bug_fixes(&component);
                        let body_expression = component
                            .body_block_expression()
                            .map(|b| b.upcast::<KtExpression>())
                            .or_else(|| component.body_expression());
                        let return_type_reference = component.return_type_reference().map(PsiElement::from);
                        v.block(Indent::ZERO, |v| {
                            v.visit_function_like_expression(
                                None,
                                component.modifier_list().as_ref(),
                                Some(&name_placeholder.text()),
                                None,
                                None,
                                None,
                                parameter_list.as_ref(),
                                None,
                                body_expression.as_ref(),
                                return_type_reference.as_ref(),
                            );
                        });
                    } else if let Some(component) = component.cast::<KtBackingField>() {
                        v.emit_backing_field(&component);
                    } else {
                        // Upstream appends `component::class`, a JVM class name we don't model.
                        return v.throw_runtime("java.lang.IllegalStateException: Unexpected property component");
                    }
                }
            });
        }

        self.builder.guess_token(";");

        if is_field {
            self.builder.blank_line_wanted(BlankLineWanted::conditional(&vertical_annotation_break));
        }
    }

    /// The `= initializer` tail shared by `declareOne` and `emitBackingField` (after the `=` token).
    fn emit_initializer(&mut self, initializer: &KtExpression) {
        if self.is_lambda_or_scoping_function(Some(initializer)) {
            self.visit_lambda_or_scoping_function(Some(initializer), true);
        } else if self.is_chained_scoping_function(initializer) {
            self.visit_chained_scoping_function(&initializer.upcast(), true);
        } else {
            let ebi = self.expression_break_indent();
            self.builder.break_op(FillMode::Unified, " ", ebi.clone());
            self.block(ebi, |v| {
                v.fence_comments();
                v.visit(Some(initializer));
            });
        }
    }

    pub(super) fn emit_backing_field(&mut self, backing_field: &KtBackingField) {
        self.sync(backing_field);
        self.block(Indent::ZERO, |v| {
            v.block(Indent::ZERO, |v| v.token(backing_field.name_placeholder().text_slice()));

            if let Some(type_) = backing_field.return_type_reference() {
                v.block(v.expression_break_indent(), |v| {
                    v.token(":");
                    v.builder.break_op(FillMode::Unified, " ", Indent::ZERO);
                    v.visit(Some(&type_));
                });
            }

            if let Some(initializer) = backing_field.initializer() {
                v.builder.space();
                v.token("=");
                v.emit_initializer(&initializer);
            }
        });
    }

    /// Kotlin 1.9.10 bug workaround (KT-70922): getters have no parameter list and the parens are
    /// children of the accessor, so upstream builds a fake list.
    fn get_parameter_list_with_bug_fixes(&self, accessor: &KtPropertyAccessor) -> Option<ParameterList> {
        if accessor.body_expression().is_none() && accessor.body_block_expression().is_none() {
            return None;
        }

        let parameter_list = accessor.parameter_list();
        Some(ParameterList {
            parameters: accessor.value_parameters(),
            trailing_comma: parameter_list.as_ref().and_then(|l| l.trailing_comma()),
            left_parenthesis: parameter_list.as_ref().and_then(|l| l.left_parenthesis()),
            right_parenthesis: parameter_list.as_ref().and_then(|l| l.right_parenthesis()),
        })
    }
}
