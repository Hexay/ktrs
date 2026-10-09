//! More of Kotlin `psiUtils.kt` / `ktPsiUtil.kt` / `ktElementUtils.kt` and `PsiTreeUtil`: descendant searches,
//! range queries on the file, and the expression helpers detekt's rules call.

use ktrs_syntax::ElementId;
use ktrs_syntax::SyntaxKind::*;

use crate::cast::PsiType;
use crate::element::PsiElement;
use crate::kt::KtFile;
use crate::types::*;

impl PsiElement {
    /// psiUtil `findDescendantOfType<T>(predicate)`: preorder (`PsiRecursiveElementWalkingVisitor`), `self` included.
    pub fn find_descendant_of_type<T: PsiType>(&self, mut predicate: impl FnMut(&T) -> bool) -> Option<T> {
        self.preorder(self.id()).find(|t| predicate(t))
    }

    /// psiUtil `anyDescendantOfType<T>(predicate)`.
    pub fn any_descendant_of_type<T: PsiType>(&self, predicate: impl FnMut(&T) -> bool) -> bool {
        self.find_descendant_of_type(predicate).is_some()
    }

    /// psiUtil `forEachDescendantOfTypeInPreorder<T>(action)`, `self` included.
    pub fn for_each_descendant_of_type_in_preorder<T: PsiType>(&self, action: impl FnMut(T)) {
        self.preorder(self.id()).for_each(action);
    }

    /// `PsiTreeUtil.findChildrenOfType(element, T)`: the strict descendants that are a `T`, in preorder.
    pub fn find_children_of_type<T: PsiType>(&self) -> Vec<T> {
        self.preorder(self.id() + 1).collect()
    }

    /// The `T`s of this subtree from element `first` on, in preorder (one handle walks the flat tree).
    fn preorder<T: PsiType>(&self, first: ElementId) -> impl Iterator<Item = T> + use<T> {
        let end = self.tree().subtree_end(self.id());
        let mut handle = self.clone();
        (first..end).filter_map(move |id| {
            handle.move_to(id);
            T::can_cast(&handle).then(|| T::cast_unchecked(handle.clone()))
        })
    }

    /// psiUtil `startOffsetSkippingComments`.
    pub fn start_offset_skipping_comments(&self) -> usize {
        if !self.starts_with_comment() {
            return self.start_offset();
        }
        let first_non_comment_child = self.all_children().find(|c| !c.is::<PsiWhiteSpace>() && !c.is::<PsiComment>());
        first_non_comment_child.map_or_else(|| self.start_offset(), |c| c.start_offset())
    }
}

impl KtFile {
    /// `PsiFile.findElementAt(offset)`: the leaf covering `offset`; None at or past the end of the text.
    pub fn find_element_at(&self, offset: usize) -> Option<PsiElement> {
        if offset >= self.text_length() {
            return None;
        }
        let mut element = PsiElement::clone(self);
        while !element.is_leaf() {
            element = element.all_children().find(|c| c.start_offset() <= offset && offset < c.end_offset())?;
        }
        Some(element)
    }

    /// psiUtil `PsiFile.elementsInRange(range)`: the outermost elements lying wholly inside `start..end`, in order.
    pub fn elements_in_range(&self, start: usize, end: usize) -> Vec<PsiElement> {
        let mut offset = start;
        let mut result = Vec::new();
        while offset < end {
            let Some(leaf) = self.find_first_leaf_wholly_in_range(offset, end) else { break };
            let mut element = leaf;
            while !element.is_file() {
                let Some(parent) = element.parent() else { break };
                if !(offset <= parent.start_offset() && parent.end_offset() <= end) {
                    break;
                }
                element = parent;
            }
            offset = element.end_offset();
            result.push(element);
        }
        result
    }

    fn find_first_leaf_wholly_in_range(&self, start: usize, end: usize) -> Option<PsiElement> {
        let mut element = self.find_element_at(start)?;
        if element.start_offset() < start {
            element = element.next_leaf(true)?;
        }
        debug_assert!(element.start_offset() >= start);
        (element.end_offset() <= end).then_some(element)
    }
}

impl KtBlockExpression {
    /// `getStatements()`.
    pub fn statements(&self) -> Vec<KtExpression> {
        self.find_children_by_class()
    }
}

impl KtReturnExpression {
    /// `getLabeledExpression()`: the `LABEL_QUALIFIER` child.
    pub fn labeled_expression(&self) -> Option<PsiElement> {
        self.find_child_by_type(LABEL_QUALIFIER)
    }
}

impl KtExpression {
    /// ktElementUtils `unpackFunctionLiteral(allowParentheses)`.
    pub fn unpack_function_literal(&self, allow_parentheses: bool) -> Option<KtLambdaExpression> {
        match self.kind() {
            LAMBDA_EXPRESSION => self.cast(),
            LABELED_EXPRESSION => self.upcast::<KtLabeledExpression>().base_expression()?.unpack_function_literal(allow_parentheses),
            ANNOTATED_EXPRESSION => self.upcast::<KtAnnotatedExpression>().base_expression()?.unpack_function_literal(allow_parentheses),
            PARENTHESIZED if allow_parentheses => {
                self.upcast::<KtParenthesizedExpression>().expression()?.unpack_function_literal(allow_parentheses)
            }
            _ => None,
        }
    }

    /// psiUtil `getQualifiedExpressionForReceiver()`.
    pub fn get_qualified_expression_for_receiver(&self) -> Option<KtQualifiedExpression> {
        let parent = self.parent()?.cast::<KtQualifiedExpression>()?;
        (parent.receiver_expression().as_ref() == Some(self)).then_some(parent)
    }

    /// psiUtil `getQualifiedExpressionForReceiverOrThis()`.
    pub fn get_qualified_expression_for_receiver_or_this(&self) -> KtExpression {
        self.get_qualified_expression_for_receiver().map_or_else(|| self.clone(), |q| q.upcast())
    }

    /// psiUtil `lastBlockStatementOrThis()`.
    pub fn last_block_statement_or_this(&self) -> KtExpression {
        self.cast::<KtBlockExpression>().and_then(|b| b.statements().pop()).unwrap_or_else(|| self.clone())
    }
}

impl KtLambdaArgument {
    /// `getLambdaExpression()`.
    pub fn get_lambda_expression(&self) -> Option<KtLambdaExpression> {
        self.argument_expression()?.unpack_function_literal(false)
    }
}

macro_rules! call_element {
    ($($t:ident),*) => {$(impl $t {
        /// psiUtil `KtCallElement.getCallNameExpression()`.
        pub fn get_call_name_expression(&self) -> Option<KtSimpleNameExpression> {
            let callee_expression = self.upcast::<KtCallElement>().callee_expression()?;
            if let Some(simple_name) = callee_expression.cast::<KtSimpleNameExpression>() {
                return Some(simple_name);
            }
            callee_expression.cast::<KtConstructorCalleeExpression>()?.constructor_reference_expression()
        }

        /// `KtCallElement.getValueArguments()`.
        pub fn value_arguments(&self) -> Vec<KtValueArgument> {
            let call = self.upcast::<KtCallElement>();
            let mut arguments = call.value_argument_list().map(|l| l.arguments()).unwrap_or_default();
            arguments.extend(call.lambda_arguments().into_iter().map(|a| a.upcast()));
            arguments
        }
    })*};
}

call_element!(KtCallElement, KtAnnotationEntry, KtConstructorDelegationCall, KtSuperTypeCallEntry);

impl KtCallExpression {
    /// psiUtil `KtCallElement.getCallNameExpression()`.
    pub fn get_call_name_expression(&self) -> Option<KtSimpleNameExpression> {
        self.upcast::<KtCallElement>().get_call_name_expression()
    }
}
