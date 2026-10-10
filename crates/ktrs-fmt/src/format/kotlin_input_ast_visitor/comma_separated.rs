//! `ListFormatter.kt` (`formatParameterList`, `formatCommaSeparatedList`, here still
//! `visit_each_comma_separated`), `CallFormatter.kt` (`formatArgument`), and reference and return expressions
//! (`ExpressionFormatter.kt`, `ControlFlowExpressionFormatter.kt`).

use ktrs_psi::*;

use crate::doc::{BreakTag, FillMode, Indent};

use super::KotlinInputAstVisitor;

/// The named arguments of `visitEachCommaSeparated`; `KotlinInputAstVisitor::comma_separated` has
/// upstream's defaults.
#[derive(Clone, Copy)]
pub(super) struct EachCommaSeparated<'p> {
    /// Each element on its own line, even if they'd fit on one, and a trailing comma is emitted.
    pub has_trailing_comma: bool,
    /// Place all elements (not the prefix/postfix) in a block, negatively indented without a leading break.
    pub wrap_in_block: bool,
    /// Break before the first element.
    pub leading_break: bool,
    pub prefix: Option<&'p str>,
    pub postfix: Option<&'p str>,
    /// Break after the prefix, before the block.
    pub break_after_prefix: bool,
    /// Break after the last element; redundant with a trailing comma.
    pub break_before_postfix: bool,
}

pub(super) fn psi_list<T: Into<PsiElement>>(list: Vec<T>) -> Vec<PsiElement> {
    list.into_iter().map(Into::into).collect()
}

impl KotlinInputAstVisitor<'_, '_, '_> {
    pub(super) fn comma_separated(&self) -> EachCommaSeparated<'static> {
        EachCommaSeparated {
            has_trailing_comma: false,
            wrap_in_block: true,
            leading_break: true,
            prefix: None,
            postfix: None,
            break_after_prefix: true,
            break_before_postfix: self.options.manage_trailing_commas(),
        }
    }

    /// e.g., `(a: Int, b: Int, c: Int)` in `(a: Int, b: Int, c: Int) -> Unit`.
    pub(super) fn visit_parameter_list(&mut self, list: &KtParameterList) {
        self.visit_each_comma_separated(
            &psi_list(list.parameters()),
            EachCommaSeparated {
                has_trailing_comma: list.trailing_comma().is_some(),
                prefix: Some("("),
                postfix: Some(")"),
                ..self.comma_separated()
            },
        );
    }

    /// Visit each element in `list` with commas in between: either all on one line, or one per line,
    /// optionally wrapped in `prefix`/`postfix`. Returns a [BreakTag] telling whether a break was
    /// taken, but only when the list doesn't end in a negative closing indent (no trailing comma and
    /// no break before the postfix); otherwise `None`.
    pub(super) fn visit_each_comma_separated(&mut self, list: &[PsiElement], args: EachCommaSeparated<'_>) -> Option<BreakTag> {
        let EachCommaSeparated {
            has_trailing_comma,
            wrap_in_block,
            leading_break,
            prefix,
            postfix,
            break_after_prefix,
            break_before_postfix,
        } = args;
        let break_after_last_element = has_trailing_comma || (postfix.is_some() && break_before_postfix);
        let name_tag = if break_after_last_element { None } else { Some(self.gen_sym()) };

        if let Some(prefix) = prefix {
            self.token(prefix);
            if break_after_prefix {
                self.builder.break_op_tagged(FillMode::Unified, "", Indent::ZERO, name_tag.clone());
            }
        }

        let break_type = if has_trailing_comma { FillMode::Forced } else { FillMode::Unified };
        let emit_comma = |v: &mut Self| {
            v.token(",");
            v.builder.break_op(break_type, " ", Indent::ZERO);
        };

        let indent = if leading_break { Indent::ZERO } else { self.expression_break_negative_indent() };
        self.block_if(indent, wrap_in_block, |v| {
            if leading_break {
                v.builder.break_op(break_type, "", Indent::ZERO);
            }

            let mut first = true;
            for value in list {
                if !first {
                    emit_comma(v);
                }
                first = false;
                v.visit(Some(value));
            }

            if has_trailing_comma {
                emit_comma(v);
            }
        });

        if break_after_last_element {
            // a negative closing indent places the postfix to the left of the elements
            self.builder.break_op(break_type, "", self.expression_break_negative_indent());
        }

        if let Some(postfix) = postfix {
            if break_after_last_element {
                self.block(self.expression_break_negative_indent(), |v| {
                    v.fence_comments();
                    v.token_indent(postfix, v.expression_break_indent());
                });
            } else {
                self.token(postfix);
            }
        }

        name_tag
    }

    /// Example `a` in `foo(a)`, or `*a`, or `limit = 50`
    pub(super) fn visit_argument(&mut self, argument: &KtValueArgument) {
        self.visit_argument_internal(argument, true, None);
    }

    /// The internal version of [Self::visit_argument]; `wrap_in_block` places the expression in a block.
    pub(super) fn visit_argument_internal(
        &mut self,
        argument: &KtValueArgument,
        wrap_in_block: bool,
        broke_before_brace: Option<&BreakTag>,
    ) {
        self.sync(argument);
        let argument_name = argument.argument_name();
        let argument_expression = argument.argument_expression();
        let has_arg_name = argument_name.is_some();
        let is_lambda = argument_expression.as_ref().is_some_and(|e| e.is::<KtLambdaExpression>());
        if has_arg_name {
            self.visit(argument_name.as_ref());
            self.builder.space();
            self.token("=");
            if is_lambda {
                self.builder.space();
            }
        }
        let indent = if has_arg_name && !is_lambda { self.expression_break_indent() } else { Indent::ZERO };
        self.block_if(indent, wrap_in_block, |v| {
            if has_arg_name && !is_lambda {
                v.builder.break_op(FillMode::Independent, " ", Indent::ZERO);
            }
            if argument.is_spread() {
                v.token("*");
            }
            match argument_expression.as_ref().and_then(|e| e.cast::<KtLambdaExpression>()) {
                Some(lambda) => v.visit_lambda_expression_internal(&lambda, broke_before_brace.cloned()),
                None => v.visit(argument_expression.as_ref()),
            }
        });
    }

    pub(super) fn visit_reference_expression(&mut self, expression: &KtReferenceExpression) {
        self.sync(expression);
        self.token(expression.text_slice());
    }

    pub(super) fn visit_return_expression(&mut self, expression: &KtReturnExpression) {
        self.sync(expression);
        self.token("return");
        self.visit(expression.target_label().as_ref());
        if let Some(returned_expression) = expression.returned_expression() {
            self.builder.space();
            self.visit(Some(&returned_expression));
        }
        self.builder.guess_token(";");
    }
}
