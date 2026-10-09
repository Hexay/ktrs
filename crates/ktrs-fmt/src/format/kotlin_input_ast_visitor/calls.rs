//! `KotlinInputAstVisitor.kt` lines 766-1016: call elements, value argument lists, lambdas, `this`
//! and simple names.

use ktrs_psi::*;

use crate::doc::{BlankLineWanted, BreakTag, FillMode, Indent};
use crate::format::psi_utils::value_argument_list_has_empty_parens;

use super::KotlinInputAstVisitor;
use super::comma_separated::{EachCommaSeparated, psi_list};

impl KotlinInputAstVisitor<'_, '_, '_> {
    /// Examples `foo<T>(a, b)`, `foo(a)`, `boo()`, `super(a)`. `lambda_indent` indents the trailing
    /// lambda; `negative_lambda_indent` undoes it for the callee and arguments, so all share one block
    /// and breaks in the argument list cause a break in the lambda.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn visit_call_element(
        &mut self,
        callee: Option<&KtExpression>,
        type_argument_list: Option<&KtTypeArgumentList>,
        argument_list: Option<&KtValueArgumentList>,
        lambda_arguments: &[KtLambdaArgument],
        arguments_indent: Indent,
        lambda_indent: Indent,
        negative_lambda_indent: Indent,
    ) {
        // `KtCallElement.trailingLambda`, which upstream's callers evaluate before any op of this call.
        if lambda_arguments.len() > 1 {
            return self.throw_parse_error("Maximum one trailing lambda is allowed", &lambda_arguments[1].upcast());
        }
        self.block(lambda_indent, |v| {
            // Tracks whether a break in the argument list requires indenting the lambda.
            let mut broke_before_brace: Option<BreakTag> = None;

            v.block(negative_lambda_indent, |v| {
                v.visit(callee);
                v.block(arguments_indent, |v| {
                    v.block(Indent::ZERO, |v| v.visit(type_argument_list));
                    if let Some(argument_list) = argument_list {
                        broke_before_brace = v.visit_value_argument_list_internal(argument_list);
                    }
                });
            });
            if let Some(trailing_lambda) = lambda_arguments.first() {
                v.builder.space();
                v.visit_argument_internal(&trailing_lambda.upcast(), false, broke_before_brace.as_ref());
            }
        });
    }

    /// Example (`1, "hi"`) in a function call
    pub(super) fn visit_value_argument_list(&mut self, list: &KtValueArgumentList) {
        self.visit_value_argument_list_internal(list);
    }

    /// Example (`1, "hi"`) in a function call. Returns a [BreakTag] telling whether a break was
    /// taken, unless the list ends in a negative closing indent (see `visit_each_comma_separated`).
    fn visit_value_argument_list_internal(&mut self, list: &KtValueArgumentList) -> Option<BreakTag> {
        self.sync(list);

        let arguments = list.arguments();
        let is_single_unnamed_lambda = arguments.len() == 1
            && arguments[0].argument_expression().is_some_and(|e| e.is::<KtLambdaExpression>())
            && arguments[0].argument_name().is_none();
        let has_trailing_comma = list.trailing_comma().is_some();
        let has_empty_parens = value_argument_list_has_empty_parens(list);

        let wrap_in_block;
        let break_before_postfix;
        let leading_break;
        let break_after_prefix;
        if is_single_unnamed_lambda {
            wrap_in_block = true;
            break_before_postfix = false;
            leading_break = !has_empty_parens && has_trailing_comma;
            break_after_prefix = false;
        } else {
            wrap_in_block = !self.options.manage_trailing_commas();
            break_before_postfix = self.options.manage_trailing_commas() && !has_empty_parens;
            leading_break = !has_empty_parens;
            break_after_prefix = !has_empty_parens;
        }

        self.visit_each_comma_separated(
            &psi_list(arguments),
            EachCommaSeparated {
                has_trailing_comma,
                wrap_in_block,
                break_before_postfix,
                leading_break,
                prefix: Some("("),
                postfix: Some(")"),
                break_after_prefix,
            },
        )
    }

    /// Example `{ 1 + 1 }` (as lambda) or `{ (x, y) -> x + y }`
    pub(super) fn visit_lambda_expression(&mut self, lambda_expression: &KtLambdaExpression) {
        self.visit_lambda_expression_internal(lambda_expression, None);
    }

    /// `broke_before_brace` tracks whether a break was taken right before the lambda (e.g. after
    /// `fun foo() =`), so a scoping function's body can be indented as if it were a block. The
    /// conditional indents below must not be used inside interior blocks (they'd apply twice).
    pub(super) fn visit_lambda_expression_internal(
        &mut self,
        lambda_expression: &KtLambdaExpression,
        broke_before_brace: Option<BreakTag>,
    ) {
        self.sync(lambda_expression);

        let value_params = lambda_expression.value_parameters();
        let has_params = !value_params.is_empty();
        let Some(body_expression) = lambda_expression.body_expression() else { return self.fail() };
        let expression_statements = body_expression.children();
        let has_statements = !expression_statements.is_empty();
        let has_comments = body_expression.node().children().any(|c| c.psi().is::<PsiComment>());
        let Some(function_literal) = lambda_expression.function_literal() else { return self.fail() };
        let has_arrow = function_literal.arrow().is_some();

        let if_broke_before_brace = |on_true: Indent, on_false: Indent| match &broke_before_brace {
            None => on_false,
            Some(tag) => Indent::make_if(tag, on_true, on_false),
        };

        let brace_plus_block_indent = if_broke_before_brace(self.block_plus_expression_break_indent(), self.block_indent());
        let brace_plus_expression_indent =
            if_broke_before_brace(self.double_expression_break_indent(), self.expression_break_indent());
        let brace_plus_zero_indent = if_broke_before_brace(self.expression_break_indent(), Indent::ZERO);

        self.token("{");

        if has_params || has_arrow {
            self.builder.space();
            self.block(brace_plus_expression_indent, |v| {
                v.visit_each_comma_separated(&psi_list(value_params), v.comma_separated());
            });
            self.block(brace_plus_block_indent.clone(), |v| {
                if function_literal.value_parameter_list().and_then(|l| l.trailing_comma()).is_some() {
                    v.token(",");
                    v.builder.forced_break();
                } else if has_params {
                    v.builder.break_op(FillMode::Independent, " ", Indent::ZERO);
                }
                v.token("->");
            });
        }

        if has_params || has_arrow || has_statements || has_comments {
            self.builder.break_op(FillMode::Unified, " ", brace_plus_zero_indent.clone());
        }

        let block_comments: Vec<PsiElement> = body_expression
            .node()
            .children()
            .map(|c| c.psi())
            .filter(|c| c.is::<PsiComment>() && c.text_slice().starts_with("/*"))
            .collect();
        if has_statements {
            self.builder.break_op(FillMode::Unified, "", brace_plus_block_indent.clone());
            self.block(brace_plus_block_indent, |v| {
                v.builder.blank_line_wanted(BlankLineWanted::NO);

                let should_force_multiline =
                    v.options.preserve_lambda_breaks && v.has_source_newline_in_lambda_body(lambda_expression);

                if !should_force_multiline
                    && expression_statements.len() == 1
                    && !expression_statements[0].is::<KtReturnExpression>()
                    && !body_expression.starts_with_comment()
                {
                    v.visit_statement(&expression_statements[0]);
                } else {
                    v.visit_statements(&expression_statements);
                }
                v.builder.break_op(FillMode::Unified, " ", brace_plus_zero_indent.clone());
            });
        } else if !block_comments.is_empty() {
            self.builder.break_op(FillMode::Unified, "", brace_plus_block_indent.clone());
            self.block(brace_plus_block_indent, |v| {
                v.fence_comments();
                v.builder.blank_line_wanted(BlankLineWanted::NO);
                for (i, comment) in block_comments.iter().enumerate() {
                    if i > 0 {
                        v.builder.forced_break();
                    }
                    v.token(comment.text_slice());
                }
                v.builder.break_op(FillMode::Unified, " ", brace_plus_zero_indent.clone());
            });
        }

        if has_params || has_arrow || has_statements || has_comments {
            // If we had to break in the body, ensure there is a break before the closing brace
            self.builder.break_op(FillMode::Unified, "", brace_plus_zero_indent.clone());
        }
        self.block(brace_plus_zero_indent, |v| {
            v.fence_comments();
            v.token_indent("}", v.block_indent());
        });
    }

    /// Example `this` or `this@Foo`
    pub(super) fn visit_this_expression(&mut self, expression: &KtThisExpression) {
        self.sync(expression);
        self.token("this");
        self.visit(expression.target_label().as_ref());
    }

    /// Example `Foo` or `@Foo`
    pub(super) fn visit_simple_name_expression(&mut self, expression: &KtSimpleNameExpression) {
        self.sync(expression);
        if let Some(label) = expression.cast::<KtLabelReferenceExpression>() {
            let Some(identifier) = label.identifier() else { return self.fail() };
            if expression.text_slice().starts_with('@') {
                self.token("@");
                self.token(identifier.text_slice());
            } else {
                self.token(identifier.text_slice());
                self.token("@");
            }
        } else {
            let text = expression.text_slice();
            if !text.is_empty() {
                self.token(text);
            }
        }
    }
}
