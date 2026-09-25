//! `PsiTreeUtil` (intellij-community idea/251.27812.49) and Kotlin `psiUtil` helpers
//! (`psiUtils.kt`, `ktPsiUtil.kt`), as methods on [`PsiElement`].

use ktrs_syntax::SyntaxKind;
use rowan::WalkEvent;

use crate::cast::PsiType;
use crate::element::PsiElement;
use std::collections::VecDeque;

use crate::types::{
    KtBinaryExpression, KtOperationReferenceExpression, KtStringTemplateExpression, PsiComment, PsiErrorElement,
    PsiWhiteSpace,
};

impl PsiElement {
    /// psiUtil `siblings(forward, withItself)`.
    pub fn siblings(&self, forward: bool, with_itself: bool) -> impl Iterator<Item = PsiElement> + use<> {
        let step = if forward { PsiElement::next_sibling } else { PsiElement::prev_sibling };
        let first = if with_itself { Some(self.clone()) } else { step(self) };
        std::iter::successors(first, step)
    }

    /// psiUtil `getNextSiblingIgnoringWhitespaceAndComments(withItself)`.
    pub fn get_next_sibling_ignoring_whitespace_and_comments(&self, with_itself: bool) -> Option<PsiElement> {
        self.siblings(true, with_itself).find(|e| !e.is::<PsiWhiteSpace>() && !e.is::<PsiComment>())
    }

    /// psiUtil `getNextSiblingIgnoringWhitespace(withItself)`.
    pub fn get_next_sibling_ignoring_whitespace(&self, with_itself: bool) -> Option<PsiElement> {
        self.siblings(true, with_itself).find(|e| !e.is::<PsiWhiteSpace>())
    }

    /// psiUtil `getPrevSiblingIgnoringWhitespaceAndComments(withItself)`.
    pub fn get_prev_sibling_ignoring_whitespace_and_comments(&self, with_itself: bool) -> Option<PsiElement> {
        self.siblings(false, with_itself).find(|e| !e.is::<PsiWhiteSpace>() && !e.is::<PsiComment>())
    }

    /// psiUtil `getPrevSiblingIgnoringWhitespace(withItself)`.
    pub fn get_prev_sibling_ignoring_whitespace(&self, with_itself: bool) -> Option<PsiElement> {
        self.siblings(false, with_itself).find(|e| !e.is::<PsiWhiteSpace>())
    }

    /// psiUtil `startsWithComment()`.
    pub fn starts_with_comment(&self) -> bool {
        self.first_child().is_some_and(|c| c.is::<PsiComment>())
    }

    /// psiUtil `prevLeaf(skipEmptyElements)` = `PsiTreeUtil.prevLeaf(element, skipEmptyElements)`.
    pub fn prev_leaf(&self, skip_empty_elements: bool) -> Option<PsiElement> {
        let mut prev_leaf = tree_prev_leaf(self);
        while skip_empty_elements && prev_leaf.as_ref().is_some_and(|l| l.text_length() == 0) {
            prev_leaf = tree_prev_leaf(prev_leaf.as_ref().unwrap());
        }
        prev_leaf
    }

    /// psiUtil `nextLeaf(skipEmptyElements)` = `PsiTreeUtil.nextLeaf(element, skipEmptyElements)`.
    pub fn next_leaf(&self, skip_empty_elements: bool) -> Option<PsiElement> {
        let mut next_leaf = tree_next_leaf(self);
        while skip_empty_elements && next_leaf.as_ref().is_some_and(|l| l.text_length() == 0) {
            next_leaf = tree_next_leaf(next_leaf.as_ref().unwrap());
        }
        next_leaf
    }

    /// psiUtil `getParentOfType<T>(strict)` = `PsiTreeUtil.getParentOfType(element, T, strict)`.
    pub fn get_parent_of_type<T: PsiType>(&self, strict: bool) -> Option<T> {
        let mut element = Some(self.clone());
        if strict {
            if self.is_file() {
                return None;
            }
            element = self.parent();
        }
        while let Some(e) = element {
            if let Some(t) = e.cast::<T>() {
                return Some(t);
            }
            if e.is_file() {
                return None;
            }
            element = e.parent();
        }
        None
    }

    /// psiUtil `getChildOfType<T>()` = `PsiTreeUtil.getChildOfType`.
    pub fn get_child_of_type<T: PsiType>(&self) -> Option<T> {
        self.all_children().find_map(|c| c.cast::<T>())
    }

    /// psiUtil `getChildrenOfType<T>()` / `PsiTreeUtil.getChildrenOfTypeAsList` (empty instead of null).
    pub fn get_children_of_type<T: PsiType>(&self) -> Vec<T> {
        self.all_children().filter_map(|c| c.cast::<T>()).collect()
    }

    /// psiUtil `collectDescendantsOfType<T>()`: `PsiRecursiveElementVisitor` order, i.e. children before
    /// the element itself, `self` included.
    pub fn collect_descendants_of_type<T: PsiType>(&self) -> Vec<T> {
        let Some(node) = self.as_node() else {
            return self.cast::<T>().into_iter().collect();
        };
        node.preorder_with_tokens()
            .filter_map(|event| match event {
                WalkEvent::Leave(e) => PsiElement::new(e).cast::<T>(),
                WalkEvent::Enter(_) => None,
            })
            .collect()
    }

    /// `PsiTreeUtil.hasErrorElements(element)` (`element` itself included).
    pub fn has_error_elements(&self) -> bool {
        self.as_node().is_some_and(|n| n.descendants().any(|d| PsiElement::new(d.into()).is::<PsiErrorElement>()))
    }

    /// `PsiTreeUtil.getDeepestLast` / `lastChild`.
    pub fn deepest_last(&self) -> PsiElement {
        std::iter::successors(Some(self.clone()), PsiElement::last_child).last().unwrap()
    }

    /// `PsiTreeUtil.getDeepestFirst` / `firstChild`.
    pub fn deepest_first(&self) -> PsiElement {
        std::iter::successors(Some(self.clone()), PsiElement::first_child).last().unwrap()
    }
}

/// psiUtils `tryFlattenStringConcatenationDescendants()`: for a `+` chain whose operands are all string
/// templates, every operand, operator and whitespace/comment in source order; else None.
pub fn try_flatten_string_concatenation_descendants(e: &KtBinaryExpression) -> Option<Vec<PsiElement>> {
    try_flatten_string_concatenation(e, true)
}

#[allow(clippy::question_mark)] // keeps upstream's `when` shape
fn try_flatten_string_concatenation(e: &KtBinaryExpression, full_fidelity: bool) -> Option<Vec<PsiElement>> {
    if e.operation_token() != Some(SyntaxKind::PLUS) {
        return None;
    }
    let mut input = vec![e.psi().clone()];
    let mut output = VecDeque::new();
    while let Some(node) = input.pop() {
        if node.is::<KtBinaryExpression>() {
            input.extend(node.all_children());
        } else if node.is::<KtStringTemplateExpression>() {
            output.push_front(node);
        } else if node.is::<PsiWhiteSpace>() || node.is::<PsiComment>() {
            if full_fidelity {
                output.push_front(node);
            }
        } else if let Some(operation) = node.cast::<KtOperationReferenceExpression>() {
            if operation.operation_sign_token_type()?.0 != SyntaxKind::PLUS {
                return None;
            }
            if full_fidelity {
                output.push_front(node);
            }
        } else {
            return None;
        }
    }
    Some(output.into())
}

/// `PsiTreeUtil.getNextSiblingOfType(sibling, T)`.
pub fn get_next_sibling_of_type<T: PsiType>(sibling: Option<&PsiElement>) -> Option<T> {
    sibling?.siblings(true, false).find_map(|c| c.cast::<T>())
}

/// `PsiTreeUtil.getPrevSiblingOfType(sibling, T)`.
pub fn get_prev_sibling_of_type<T: PsiType>(sibling: Option<&PsiElement>) -> Option<T> {
    sibling?.siblings(false, false).find_map(|c| c.cast::<T>())
}

/// ktPsiUtil `getTrailingCommaByClosingElement(closingElement)`.
pub fn get_trailing_comma_by_closing_element(closing_element: Option<&PsiElement>) -> Option<PsiElement> {
    let before = closing_element?.get_prev_sibling_ignoring_whitespace_and_comments(false)?;
    (before.kind() == SyntaxKind::COMMA).then_some(before)
}

/// ktPsiUtil `getTrailingCommaByElementsList(elementList)`.
pub fn get_trailing_comma_by_elements_list(element_list: Option<&PsiElement>) -> Option<PsiElement> {
    let last_child = element_list?.last_child()?;
    let last_child = if !last_child.is::<PsiComment>() {
        Some(last_child)
    } else {
        last_child.get_prev_sibling_ignoring_whitespace_and_comments(false)
    };
    last_child.filter(|c| c.kind() == SyntaxKind::COMMA)
}

fn tree_prev_leaf(current: &PsiElement) -> Option<PsiElement> {
    let mut current = current.clone();
    loop {
        if current.is_file() {
            return None;
        }
        if let Some(prev) = current.prev_sibling() {
            return Some(prev.deepest_last());
        }
        let parent = current.parent()?;
        if parent.is_file() {
            return None;
        }
        current = parent;
    }
}

fn tree_next_leaf(current: &PsiElement) -> Option<PsiElement> {
    let mut current = current.clone();
    loop {
        if current.is_file() {
            return None;
        }
        if let Some(next) = current.next_sibling() {
            return Some(next.deepest_first());
        }
        let parent = current.parent()?;
        if parent.is_file() {
            return None;
        }
        current = parent;
    }
}
