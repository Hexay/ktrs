//! `FormatterState.kt` (`markForPartialFormat`), `OpsUtils.kt` (the `OpsBuilder` extensions `token`,
//! `block`, `sync`, `fenceComments`), `helpers.kt` (`fail`, `format`, here `visit`),
//! `ControlFlowExpressionFormatter.kt` (`emitKeywordWithCondition`); plus getters for the indents
//! (`Indentation.kt` arithmetic as precomputed `Indent` values, cloned on use).

use ktrs_psi::{KtExpression, PsiElement, PsiType};

use crate::doc::{FillMode, FormattingError, Indent, Op, RealOrImaginary};

use super::super::FormatError;
use super::super::input::ParseError;
use super::KotlinInputAstVisitor;

impl KotlinInputAstVisitor<'_, '_, '_> {
    pub(super) fn block_indent(&self) -> Indent {
        self.block_indent.clone()
    }

    pub(super) fn expression_break_indent(&self) -> Indent {
        self.expression_break_indent.clone()
    }

    pub(super) fn block_plus_expression_break_indent(&self) -> Indent {
        self.block_plus_expression_break_indent.clone()
    }

    pub(super) fn double_expression_break_indent(&self) -> Indent {
        self.double_expression_break_indent.clone()
    }

    pub(super) fn expression_break_negative_indent(&self) -> Indent {
        self.expression_break_negative_indent.clone()
    }

    /// Delineates the smallest areas of code that must be formatted together.
    pub(super) fn mark_for_partial_format(&mut self) {
        if !*self.in_expression.last().unwrap() {
            self.builder.mark_for_partial_format();
        }
    }

    /// `OpsBuilder.token(token)` extension: a real token, no extra comment indent.
    pub(super) fn token(&mut self, token: &str) {
        self.token_indent(token, Indent::ZERO);
    }

    /// `OpsBuilder.token(token, plusIndentCommentsBefore)` extension.
    pub(super) fn token_indent(&mut self, token: &str, plus_indent_comments_before: Indent) {
        self.builder.token(token, RealOrImaginary::Real, plus_indent_comments_before, None);
    }

    /// Opens a new level, emits into it and closes it.
    pub(super) fn block(&mut self, plus_indent: Indent, block: impl FnOnce(&mut Self)) {
        self.block_if(plus_indent, true, block);
    }

    /// `OpsBuilder.block(plusIndent, isEnabled, block)`.
    pub(super) fn block_if(&mut self, plus_indent: Indent, is_enabled: bool, block: impl FnOnce(&mut Self)) {
        if is_enabled {
            self.builder.open(plus_indent);
        }
        block(self);
        if is_enabled {
            self.builder.close();
        }
    }

    /// Syncs the current offset to match any element in the AST.
    pub(super) fn sync(&mut self, psi_element: &PsiElement) {
        self.builder.sync(psi_element.start_offset() as i32);
    }

    /// Prevent subsequent comments from being moved ahead of this point, into parent levels.
    pub(super) fn fence_comments(&mut self) {
        self.builder.add(Op::FenceComments);
    }

    /// Records the `FormattingError` upstream throws; callers return right after.
    pub(super) fn fail(&mut self) {
        self.fail_with("Unexpected");
    }

    pub(super) fn fail_with(&mut self, message: &str) {
        let diagnostic = self.builder.diagnostic(message.to_owned());
        self.builder.fail(FormattingError::new(diagnostic));
    }

    /// Records an exception other than `FormattingError` (a `ParseError`, `IllegalStateException`,
    /// ...); a placeholder failure makes the visit unwind like any other. Callers return right after.
    pub(super) fn throw(&mut self, exception: FormatError) {
        if self.builder.error().is_none() {
            // What an enclosing `visitElement` rethrows; no op runs between the throw and that catch.
            let wrapped = self.builder.diagnostic(java_stack_trace_header(&exception));
            self.exception = Some((exception, wrapped));
        }
        self.fail();
    }

    /// Upstream's `error(..)`, `check(..)` or `AssertionError`: not caught by ktfmt's CLI either.
    /// `java_to_string` is the exception's `toString()`.
    pub(super) fn throw_runtime(&mut self, java_to_string: &str) {
        self.throw(FormatError::Runtime(java_to_string.to_owned()));
    }

    /// `ParseError(errorDescription, element)`.
    pub(super) fn throw_parse_error(&mut self, error_description: &str, element: &PsiElement) {
        let text = self.builder.get_input().get_text();
        let error = ParseError::at_offset(error_description, text, element.start_offset());
        self.throw(error.into());
    }

    pub(super) fn visit<T: PsiType>(&mut self, element: Option<&T>) {
        if let Some(element) = element {
            element.psi().accept(self);
        }
    }

    /// Emits a keyword followed by a condition, e.g. `if (b)`; guards omit the parens.
    pub(super) fn emit_keyword_with_condition(
        &mut self,
        keyword: &str,
        condition: Option<&KtExpression>,
        surround_condition_with_parens: bool,
    ) {
        let Some(condition) = condition else {
            self.token(keyword);
            return;
        };

        self.block(Indent::ZERO, |v| {
            v.token(keyword);
            v.builder.space();
            if surround_condition_with_parens {
                v.token("(");
            }
            if v.options.manage_trailing_commas() {
                v.block(v.expression_break_indent(), |v| {
                    v.builder.break_op(FillMode::Unified, "", Indent::ZERO);
                    v.visit(Some(condition));
                    v.builder.break_op(FillMode::Unified, "", v.expression_break_negative_indent());
                });
            } else {
                v.block(Indent::ZERO, |v| v.visit(Some(condition)));
            }
        });
        if surround_condition_with_parens {
            self.token(")");
        }
    }
}

/// The first line of `Throwables.getStackTraceAsString(t)`; the frames can't be matched.
fn java_stack_trace_header(exception: &FormatError) -> String {
    match exception {
        FormatError::Parse(e) => format!("org.jetbrains.kotlinx.ktfmt.format.ParseError: {e}"),
        _ => exception.to_string(),
    }
}
