//! `KotlinInputAstVisitor.kt` lines 170-289: named functions and type elements.

use ktrs_psi::*;
use ktrs_syntax::SyntaxKind;

use crate::doc::Indent;

use super::KotlinInputAstVisitor;
use super::comma_separated::{EachCommaSeparated, psi_list};
use super::function_like::ParameterList;

impl KotlinInputAstVisitor<'_, '_, '_> {
    /// Example: `fun foo(n: Int) { println(n) }`
    pub(super) fn visit_named_function(&mut self, function: &KtNamedFunction) {
        self.sync(function);
        let context_receiver_list = function
            .get_stub_or_psi_child::<PsiElement>(SyntaxKind::CONTEXT_PARAMETER_LIST)
            .and_then(|e| e.cast::<KtContextReceiverList>());
        let name = function.name_identifier().map(|n| n.text());
        let parameter_list = function.value_parameter_list().map(|l| ParameterList::of(&l));
        let body_expression = function.body_block_expression().map(|b| b.upcast::<KtExpression>()).or_else(|| function.body_expression());
        let type_or_delegation_call = function.type_reference().map(PsiElement::from);
        self.block(Indent::ZERO, |v| {
            v.visit_function_like_expression(
                context_receiver_list.as_ref(),
                function.modifier_list().as_ref(),
                Some("fun"),
                function.type_parameter_list().as_ref(),
                function.receiver_type_reference().as_ref(),
                name.as_deref(),
                parameter_list.as_ref(),
                function.type_constraint_list().as_ref(),
                body_expression.as_ref(),
                type_or_delegation_call.as_ref(),
            );
        });
    }

    /// Example `Int`, `(String)` or `() -> Int`. The modifier list can be inside the parens
    /// (`(@Composable (x) -> Unit)`) or outside (`@Composable ((x) -> Unit)`), so walk the children.
    pub(super) fn visit_type_reference(&mut self, type_reference: &KtTypeReference) {
        self.sync(type_reference);
        let modifier_list = type_reference.modifier_list().map(PsiElement::from);
        let type_element = type_reference.type_element().map(PsiElement::from);
        for child in type_reference.node().children() {
            let psi = child.psi();
            if Some(&psi) == modifier_list.as_ref() {
                self.visit(modifier_list.as_ref());
            } else if Some(&psi) == type_element.as_ref() {
                self.visit(type_element.as_ref());
            } else if child.element_type() == SyntaxKind::LPAR {
                self.token("(");
            } else if child.element_type() == SyntaxKind::RPAR {
                self.token(")");
            }
        }
    }

    pub(super) fn visit_dynamic_type(&mut self, _type: &KtDynamicType) {
        self.token("dynamic");
    }

    /// Example: `String?` or `((Int) -> Unit)?`; there can be multiple layers of parens.
    pub(super) fn visit_nullable_type(&mut self, nullable_type: &KtNullableType) {
        self.sync(nullable_type);

        let modifier_list = nullable_type.modifier_list().map(PsiElement::from);
        let inner_type = nullable_type.inner_type().map(PsiElement::from);
        for child in nullable_type.node().children() {
            let psi = child.psi();
            if Some(&psi) == modifier_list.as_ref() {
                self.visit(modifier_list.as_ref());
            } else if Some(&psi) == inner_type.as_ref() {
                self.visit(inner_type.as_ref());
            } else if child.element_type() == SyntaxKind::LPAR {
                self.token("(");
            } else if child.element_type() == SyntaxKind::RPAR {
                self.token(")");
            }
        }
        self.token("?");
    }

    /// Example: `String` or `List<Int>`,
    pub(super) fn visit_user_type(&mut self, type_: &KtUserType) {
        self.sync(type_);

        if let Some(qualifier) = type_.qualifier() {
            self.visit(Some(&qualifier));
            self.token(".");
        }
        self.visit(type_.reference_expression().as_ref());
        if let Some(type_argument_list) = type_.type_argument_list() {
            self.block(self.expression_break_indent(), |v| v.visit(Some(&type_argument_list)));
        }
    }

    /// Example: `A & B`,
    pub(super) fn visit_intersection_type(&mut self, type_: &KtIntersectionType) {
        self.sync(type_);

        // TODO(strulovich): Should this have the same indentation behaviour as `x && y`?
        self.visit(type_.left_type_ref().as_ref());
        self.builder.space();
        self.token("&");
        self.builder.space();
        self.visit(type_.right_type_ref().as_ref());
    }

    /// Example `<Int, String>` in `List<Int, String>`
    pub(super) fn visit_type_argument_list(&mut self, type_argument_list: &KtTypeArgumentList) {
        self.sync(type_argument_list);
        self.visit_each_comma_separated(
            &psi_list(type_argument_list.arguments()),
            EachCommaSeparated {
                has_trailing_comma: type_argument_list.trailing_comma().is_some(),
                wrap_in_block: !self.options.manage_trailing_commas(),
                prefix: Some("<"),
                postfix: Some(">"),
                ..self.comma_separated()
            },
        );
    }

    pub(super) fn visit_type_projection(&mut self, type_projection: &KtTypeProjection) {
        self.sync(type_projection);
        let type_reference = type_projection.type_reference();
        match type_projection.projection_kind() {
            KtProjectionKind::In => {
                self.token("in");
                self.builder.space();
                self.visit(type_reference.as_ref());
            }
            KtProjectionKind::Out => {
                self.token("out");
                self.builder.space();
                self.visit(type_reference.as_ref());
            }
            KtProjectionKind::Star => self.token("*"),
            KtProjectionKind::None => self.visit(type_reference.as_ref()),
        }
    }
}
