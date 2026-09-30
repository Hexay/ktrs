//! `KotlinInputAstVisitor.kt` lines 291-473: `visitFunctionLikeExpression`, `genSym`, braced blocks,
//! statements, `visitProperty`, `visitBackingField`.

use ktrs_psi::*;

use crate::doc::{BlankLineWanted, BreakTag, FillMode, Indent, RealOrImaginary};
use crate::format::psi_utils::parens_have_only_whitespace_between;

use super::KotlinInputAstVisitor;
use super::comma_separated::{EachCommaSeparated, psi_list};
use super::declarations::DeclarationKind;

/// The parts of a `KtParameterList` the visitor reads; also stands for the fake list of
/// `getParameterListWithBugFixes`.
pub(super) struct ParameterList {
    pub parameters: Vec<KtParameter>,
    pub trailing_comma: Option<PsiElement>,
    pub left_parenthesis: Option<PsiElement>,
    pub right_parenthesis: Option<PsiElement>,
}

impl ParameterList {
    pub fn of(list: &KtParameterList) -> ParameterList {
        ParameterList {
            parameters: list.parameters(),
            trailing_comma: list.trailing_comma(),
            left_parenthesis: list.left_parenthesis(),
            right_parenthesis: list.right_parenthesis(),
        }
    }

    /// psi_utils `hasEmptyParens`.
    pub fn has_empty_parens(&self) -> bool {
        parens_have_only_whitespace_between(self.left_parenthesis.clone(), self.right_parenthesis.clone())
    }
}

impl KotlinInputAstVisitor<'_, '_, '_> {
    /// `keyword` is e.g. "fun" or "class"; `type_or_delegation_call` is a function's return type or
    /// a constructor's delegation call.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn visit_function_like_expression(
        &mut self,
        context_receiver_list: Option<&KtContextReceiverList>,
        modifier_list: Option<&KtModifierList>,
        keyword: Option<&str>,
        type_parameters: Option<&KtTypeParameterList>,
        receiver_type_reference: Option<&KtTypeReference>,
        name: Option<&str>,
        parameter_list: Option<&ParameterList>,
        type_constraint_list: Option<&KtTypeConstraintList>,
        body_expression: Option<&KtExpression>,
        type_or_delegation_call: Option<&PsiElement>,
    ) {
        fn emit_type_or_delegation_call<'v, 'b, 'a, 'o>(
            v: &'v mut KotlinInputAstVisitor<'b, 'a, 'o>,
            type_or_delegation_call: Option<&PsiElement>,
            block: impl FnOnce(&mut KotlinInputAstVisitor<'b, 'a, 'o>),
        ) {
            if let Some(type_or_delegation_call) = type_or_delegation_call {
                v.block(Indent::ZERO, |v| {
                    if type_or_delegation_call.is::<KtConstructorDelegationCall>() {
                        v.builder.space();
                    }
                    v.token(":");
                    block(v);
                });
            }
        }

        let ebi = self.expression_break_indent();
        let force_trailing_break = name.is_some();
        self.block_if(Indent::ZERO, force_trailing_break, |v| {
            if let Some(context_receiver_list) = context_receiver_list {
                v.visit_context_receiver_list(context_receiver_list);
            }
            if let Some(modifier_list) = modifier_list {
                v.visit_modifier_list(modifier_list);
            }
            if let Some(keyword) = keyword {
                v.token(keyword);
            }
            if let Some(type_parameters) = type_parameters {
                v.builder.space();
                v.block(Indent::ZERO, |v| v.visit(Some(type_parameters)));
            }

            if name.is_some() || receiver_type_reference.is_some() {
                v.builder.space();
            }
            v.block(Indent::ZERO, |v| {
                if let Some(receiver_type_reference) = receiver_type_reference {
                    v.visit(Some(receiver_type_reference));
                    v.builder.break_op(FillMode::Independent, "", ebi.clone());
                    v.token(".");
                }
                if let Some(name) = name {
                    v.token(name);
                }
            });

            if parameter_list.is_some_and(ParameterList::has_empty_parens) {
                v.block(Indent::ZERO, |v| {
                    v.token("(");
                    v.token(")");
                    emit_type_or_delegation_call(v, type_or_delegation_call, |v| {
                        v.builder.break_op(FillMode::Independent, " ", ebi.clone());
                        v.block(ebi.clone(), |v| v.visit(type_or_delegation_call));
                    });
                });
            } else {
                v.block(ebi.clone(), |v| {
                    if let Some(parameter_list) = parameter_list {
                        v.visit_each_comma_separated(
                            &psi_list(parameter_list.parameters.clone()),
                            EachCommaSeparated {
                                has_trailing_comma: parameter_list.trailing_comma.is_some(),
                                prefix: Some("("),
                                postfix: Some(")"),
                                wrap_in_block: false,
                                break_before_postfix: true,
                                ..v.comma_separated()
                            },
                        );
                    }
                    emit_type_or_delegation_call(v, type_or_delegation_call, |v| {
                        v.builder.space();
                        v.block(v.expression_break_negative_indent(), |v| v.visit(type_or_delegation_call));
                    });
                });
            }

            if let Some(type_constraint_list) = type_constraint_list {
                v.visit(Some(type_constraint_list));
            }
            if let Some(body_expression) = body_expression {
                if body_expression.is::<KtBlockExpression>() {
                    v.builder.space();
                    v.visit(Some(body_expression));
                } else {
                    v.builder.space();
                    v.block(Indent::ZERO, |v| {
                        v.token("=");
                        if v.is_lambda_or_scoping_function(Some(body_expression)) {
                            v.visit_lambda_or_scoping_function(Some(body_expression), true);
                        } else if v.is_chained_scoping_function(body_expression) {
                            v.visit_chained_scoping_function(&body_expression.upcast(), true);
                        } else {
                            v.block(ebi.clone(), |v| {
                                v.builder.break_op(FillMode::Independent, " ", Indent::ZERO);
                                v.block(Indent::ZERO, |v| v.visit(Some(body_expression)));
                            });
                        }
                    });
                }
            }
            v.builder.guess_token(";");
        });
        if force_trailing_break {
            self.builder.forced_break();
        }
    }

    pub(super) fn gen_sym(&self) -> BreakTag {
        BreakTag::new()
    }

    pub(super) fn emit_braced_block(
        &mut self,
        body_block_expression: &PsiElement,
        emit_children: impl FnOnce(&mut Self, Vec<PsiElement>),
    ) {
        let block_indent = self.block_indent();
        self.builder.token("{", RealOrImaginary::Real, block_indent.clone(), Some(block_indent.clone()));
        let statements = body_block_expression.children();
        if !statements.is_empty() {
            self.block(block_indent.clone(), |v| {
                v.builder.forced_break();
                v.builder.blank_line_wanted(BlankLineWanted::PRESERVE);
                emit_children(v, statements);
            });
            self.builder.forced_break();
            self.builder.blank_line_wanted(BlankLineWanted::NO);
        }
        self.token_indent("}", block_indent);
    }

    pub(super) fn visit_statement(&mut self, statement: &PsiElement) {
        self.block(Indent::ZERO, |v| v.visit(Some(statement)));
        self.builder.guess_token(";");
    }

    pub(super) fn visit_statements(&mut self, statements: &[PsiElement]) {
        let mut first = true;
        self.builder.guess_token(";");
        for statement in statements {
            self.builder.forced_break();
            if !first {
                self.builder.blank_line_wanted(BlankLineWanted::PRESERVE);
            }
            first = false;
            self.visit_statement(statement);
        }
    }

    pub(super) fn visit_property(&mut self, property: &KtProperty) {
        self.sync(property);
        let Some(val_or_var_keyword) = property.val_or_var_keyword() else { return self.fail() };
        let name = property.name_identifier().map(|n| n.text());
        self.block(Indent::ZERO, |v| {
            v.declare_one(
                DeclarationKind::Field,
                property.modifier_list().as_ref(),
                Some(&val_or_var_keyword.text()),
                property.type_parameter_list().as_ref(),
                property.receiver_type_reference().as_ref(),
                name.as_deref(),
                property.type_reference().as_ref(),
                property.type_constraint_list().as_ref(),
                property.initializer().as_ref(),
                property.delegate().as_ref(),
                Some(&property.accessors()),
                property.field_declaration().as_ref(),
            );
        });
        self.builder.guess_token(";");
        if !property.parent().is_some_and(|p| p.is::<KtWhenExpression>()) {
            self.builder.forced_break();
        }
    }

    /// Not an override upstream (the bundled compiler's visitor has no `visitBackingField`).
    #[allow(dead_code)]
    pub(super) fn visit_backing_field(&mut self, backing_field: &KtBackingField) {
        self.emit_backing_field(backing_field);
    }
}
