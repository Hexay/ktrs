//! Operator expressions, labels, array access, `::` expressions, parentheses, annotated and collection literals.

use ktrs_syntax::SyntaxKind::{self, *};

use crate::element::PsiElement;
use crate::tree_util::{get_next_sibling_of_type, get_prev_sibling_of_type, get_trailing_comma_by_closing_element};
use crate::types::*;

macro_rules! unary_expression {
    ($($t:ident),*) => {$(impl $t {
        /// `getOperationReference()`: upstream throws when absent (None here).
        pub fn operation_reference(&self) -> Option<KtSimpleNameExpression> {
            self.find_child_by_type(OPERATION_REFERENCE)
        }

        pub fn operation_token(&self) -> Option<SyntaxKind> {
            self.operation_reference()?.referenced_name_element_type()
        }

        /// Prefix: the expression after the operator; postfix: the one before it.
        pub fn base_expression(&self) -> Option<KtExpression> {
            let operation_reference = self.operation_reference().map(PsiElement::from);
            if self.kind() == PREFIX_EXPRESSION {
                get_next_sibling_of_type(operation_reference.as_ref())
            } else {
                get_prev_sibling_of_type(operation_reference.as_ref())
            }
        }
    })*};
}

unary_expression!(KtUnaryExpression, KtPrefixExpression, KtPostfixExpression);

impl KtBinaryExpression {
    /// `getOperationReference()`: upstream throws when absent (None here).
    pub fn operation_reference(&self) -> Option<KtOperationReferenceExpression> {
        self.find_child_by_type(OPERATION_REFERENCE)
    }

    pub fn left(&self) -> Option<KtExpression> {
        self.operation_reference()?.siblings(false, false).find_map(|s| s.cast())
    }

    pub fn right(&self) -> Option<KtExpression> {
        self.operation_reference()?.siblings(true, false).find_map(|s| s.cast())
    }

    pub fn operation_token(&self) -> Option<SyntaxKind> {
        self.operation_reference()?.referenced_name_element_type()
    }
}

macro_rules! expression_with_label {
    ($($t:ident),*) => {$(impl $t {
        pub fn target_label(&self) -> Option<KtSimpleNameExpression> {
            self.label_qualifier()?.find_child_by_type(LABEL)
        }

        pub fn label_qualifier(&self) -> Option<KtContainerNode> {
            self.find_child_by_type(LABEL_QUALIFIER)
        }

        pub fn label_name(&self) -> Option<String> {
            Some(self.target_label()?.referenced_name())
        }
    })*};
}

expression_with_label!(
    KtExpressionWithLabel, KtReturnExpression, KtContinueExpression, KtBreakExpression, KtThisExpression,
    KtSuperExpression, KtLabeledExpression
);

impl KtReturnExpression {
    pub fn returned_expression(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }
}

impl KtLabeledExpression {
    pub fn base_expression(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }
}

impl KtSuperExpression {
    pub fn super_type_qualifier(&self) -> Option<KtTypeReference> {
        self.find_child_by_type(TYPE_REFERENCE)
    }
}

impl KtArrayAccessExpression {
    pub fn array_expression(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }

    pub fn index_expressions(&self) -> Vec<KtExpression> {
        self.indices_node().map(|n| n.get_children_of_type()).unwrap_or_default()
    }

    /// `getIndicesNode()`: always present in parsed code (upstream asserts).
    pub fn indices_node(&self) -> Option<KtContainerNode> {
        self.find_child_by_type(INDICES)
    }

    pub fn left_bracket(&self) -> Option<PsiElement> {
        self.indices_node()?.find_child_by_type(LBRACKET)
    }

    pub fn right_bracket(&self) -> Option<PsiElement> {
        self.indices_node()?.find_child_by_type(RBRACKET)
    }

    pub fn trailing_comma(&self) -> Option<PsiElement> {
        get_trailing_comma_by_closing_element(self.right_bracket().as_ref())
    }
}

impl KtParenthesizedExpression {
    pub fn expression(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }
}

impl KtAnnotatedExpression {
    pub fn base_expression(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }

    pub fn annotations(&self) -> Vec<KtAnnotation> {
        self.find_children_by_type(ANNOTATION)
    }

    /// ktPsiUtil `collectAnnotationEntriesFromPsi` over `getChildren()`.
    pub fn annotation_entries(&self) -> Vec<KtAnnotationEntry> {
        collect_annotation_entries_from_psi(self)
    }
}

pub(super) fn collect_annotation_entries_from_psi(container: &PsiElement) -> Vec<KtAnnotationEntry> {
    let mut entries = Vec::new();
    for child in container.children() {
        if let Some(entry) = child.cast::<KtAnnotationEntry>() {
            entries.push(entry);
        } else if let Some(annotation) = child.cast::<KtAnnotation>() {
            entries.extend(annotation.entries());
        }
    }
    entries
}

macro_rules! double_colon_expression {
    ($($t:ident),*) => {$(impl $t {
        pub fn receiver_expression(&self) -> Option<KtExpression> {
            self.node().first_child_node()?.psi().cast()
        }

        /// `getHasQuestionMarks()`: a `?` before `::` (upstream errors if `::` is missing; false here).
        pub fn has_question_marks(&self) -> bool {
            for element in self.node().children() {
                match element.element_type() {
                    QUEST => return true,
                    COLONCOLON => return false,
                    _ => {}
                }
            }
            false
        }

        pub fn find_colon_colon(&self) -> Option<PsiElement> {
            self.find_child_by_type(COLONCOLON)
        }
    })*};
}

double_colon_expression!(KtDoubleColonExpression, KtCallableReferenceExpression, KtClassLiteralExpression);

impl KtCallableReferenceExpression {
    /// `getCallableReference()`: upstream throws when absent (None here).
    pub fn callable_reference(&self) -> Option<KtSimpleNameExpression> {
        self.find_colon_colon()?.siblings(true, true).find_map(|s| s.cast())
    }
}

impl KtCollectionLiteralExpression {
    pub fn left_bracket(&self) -> Option<PsiElement> {
        Some(self.node().find_child_by_type(LBRACKET)?.psi())
    }

    pub fn right_bracket(&self) -> Option<PsiElement> {
        Some(self.node().find_child_by_type(RBRACKET)?.psi())
    }

    pub fn trailing_comma(&self) -> Option<PsiElement> {
        get_trailing_comma_by_closing_element(self.right_bracket().as_ref())
    }

    pub fn inner_expressions(&self) -> Vec<KtExpression> {
        self.get_children_of_type()
    }
}

impl KtContainerNodeForControlStructureBody {
    pub fn expression(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }
}
