//! `when`, `if`, loops, `try`/`catch`/`finally`, `throw`, `is` and `as`.

use ktrs_syntax::SyntaxKind::{self, *};

use crate::element::PsiElement;
use crate::tree_util::get_trailing_comma_by_closing_element;
use crate::types::*;

/// `KtExpressionImpl.findExpressionUnder(type)`: the first expression inside the container child of `type`.
fn find_expression_under(e: &PsiElement, kind: SyntaxKind) -> Option<KtExpression> {
    e.find_child_by_type::<KtContainerNode>(kind)?.find_child_by_class()
}

impl KtWhenExpression {
    pub fn entries(&self) -> Vec<KtWhenEntry> {
        self.find_children_by_type(WHEN_ENTRY)
    }

    pub fn subject_variable(&self) -> Option<KtProperty> {
        self.find_child_by_class()
    }

    pub fn subject_expression(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }
}

impl KtWhenEntry {
    pub fn is_else(&self) -> bool {
        self.else_keyword().is_some() && self.guard().is_none()
    }

    pub fn else_keyword(&self) -> Option<PsiElement> {
        self.find_child_by_type(ELSE_KEYWORD)
    }

    pub fn expression(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }

    pub fn conditions(&self) -> Vec<KtWhenCondition> {
        self.find_children_by_class()
    }

    pub fn guard(&self) -> Option<KtWhenEntryGuard> {
        self.find_child_by_class()
    }

    pub fn trailing_comma(&self) -> Option<PsiElement> {
        get_trailing_comma_by_closing_element(self.arrow().as_ref())
    }

    pub fn arrow(&self) -> Option<PsiElement> {
        self.find_child_by_type(ARROW)
    }
}

impl KtWhenEntryGuard {
    pub fn expression(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }
}

impl KtWhenConditionWithExpression {
    pub fn expression(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }
}

impl KtWhenConditionIsPattern {
    pub fn is_negated(&self) -> bool {
        self.node().find_child_by_type(NOT_IS).is_some()
    }

    pub fn type_reference(&self) -> Option<KtTypeReference> {
        self.find_child_by_type(TYPE_REFERENCE)
    }
}

impl KtWhenConditionInRange {
    pub fn operation_reference(&self) -> Option<KtOperationReferenceExpression> {
        self.find_child_by_type(OPERATION_REFERENCE)
    }

    /// `isNegated()`; upstream throws without an operation reference (false here).
    pub fn is_negated(&self) -> bool {
        self.operation_reference().is_some_and(|r| r.node().find_child_by_type(NOT_IN).is_some())
    }

    pub fn range_expression(&self) -> Option<KtExpression> {
        self.operation_reference()?.siblings(true, false).find_map(|s| s.cast())
    }
}

impl KtIfExpression {
    pub fn condition(&self) -> Option<KtExpression> {
        find_expression_under(self, CONDITION)
    }

    pub fn left_parenthesis(&self) -> Option<PsiElement> {
        self.find_child_by_type(LPAR)
    }

    pub fn right_parenthesis(&self) -> Option<PsiElement> {
        self.find_child_by_type(RPAR)
    }

    pub fn then(&self) -> Option<KtExpression> {
        find_expression_under(self, THEN)
    }

    pub fn r#else(&self) -> Option<KtExpression> {
        find_expression_under(self, ELSE)
    }

    pub fn else_keyword(&self) -> Option<PsiElement> {
        self.find_child_by_type(ELSE_KEYWORD)
    }

    pub fn if_keyword(&self) -> Option<PsiElement> {
        self.find_child_by_type(IF_KEYWORD)
    }
}

macro_rules! loop_expression {
    ($($t:ident),*) => {$(impl $t {
        pub fn body(&self) -> Option<KtExpression> {
            find_expression_under(self, BODY)
        }

        pub fn left_parenthesis(&self) -> Option<PsiElement> {
            self.find_child_by_type(LPAR)
        }

        pub fn right_parenthesis(&self) -> Option<PsiElement> {
            self.find_child_by_type(RPAR)
        }
    })*};
}

loop_expression!(KtLoopExpression, KtForExpression, KtWhileExpressionBase, KtWhileExpression, KtDoWhileExpression);

macro_rules! while_expression_base {
    ($($t:ident),*) => {$(impl $t {
        pub fn condition(&self) -> Option<KtExpression> {
            find_expression_under(self, CONDITION)
        }
    })*};
}

while_expression_base!(KtWhileExpressionBase, KtWhileExpression, KtDoWhileExpression);

impl KtForExpression {
    pub fn loop_parameter(&self) -> Option<KtParameter> {
        self.find_child_by_type(VALUE_PARAMETER)
    }

    pub fn destructuring_declaration(&self) -> Option<KtDestructuringDeclaration> {
        self.loop_parameter()?.destructuring_declaration()
    }

    pub fn loop_range(&self) -> Option<KtExpression> {
        find_expression_under(self, LOOP_RANGE)
    }
}

impl KtTryExpression {
    pub fn try_block(&self) -> Option<KtBlockExpression> {
        self.find_child_by_type(BLOCK)
    }

    pub fn catch_clauses(&self) -> Vec<KtCatchClause> {
        self.find_children_by_type(CATCH)
    }

    pub fn finally_block(&self) -> Option<KtFinallySection> {
        self.find_child_by_type(FINALLY)
    }
}

impl KtCatchClause {
    pub fn parameter_list(&self) -> Option<KtParameterList> {
        self.find_child_by_type(VALUE_PARAMETER_LIST)
    }

    pub fn catch_parameter(&self) -> Option<KtParameter> {
        let parameters = self.parameter_list()?.parameters();
        if parameters.len() == 1 { parameters.into_iter().next() } else { None }
    }

    pub fn catch_body(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }
}

impl KtFinallySection {
    pub fn final_expression(&self) -> Option<KtBlockExpression> {
        self.find_child_by_type(BLOCK)
    }
}

impl KtThrowExpression {
    pub fn thrown_expression(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }
}

impl KtIsExpression {
    pub fn left_hand_side(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }

    pub fn type_reference(&self) -> Option<KtTypeReference> {
        self.find_child_by_type(TYPE_REFERENCE)
    }

    pub fn operation_reference(&self) -> Option<KtSimpleNameExpression> {
        self.find_child_by_type(OPERATION_REFERENCE)
    }
}

impl KtBinaryExpressionWithTypeRHS {
    /// `getLeft()`: upstream asserts non-null (off at runtime).
    pub fn left(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }

    /// `getRight()`: the first type reference from the operation reference on (upstream NPEs without one;
    /// None here).
    pub fn right(&self) -> Option<KtTypeReference> {
        self.operation_reference()?.siblings(true, true).find_map(|s| s.cast())
    }

    pub fn operation_reference(&self) -> Option<KtSimpleNameExpression> {
        self.find_child_by_type(OPERATION_REFERENCE)
    }
}
