//! Port of `TrailingCommas.kt` (lines 32-162): detects trailing commas or elements that should have them.

use ktrs_psi::{
    KtClassBody, KtCollectionLiteralExpression, KtElement, KtEnumEntry, KtFunctionLiteral, KtLambdaExpression,
    KtParameterList, KtTypeArgumentList, KtTypeParameterList, KtValueArgumentList, KtWhenEntry, LeafPsiElement,
    PsiComment, PsiElement, PsiWhiteSpace,
};
use ktrs_syntax::SyntaxKind;

use super::enum_entry_list::EnumEntryList;

#[derive(Default)]
pub struct Detector {
    trailing_commas: Vec<PsiElement>,
}

impl Detector {
    pub fn get_trailing_comma_elements(&self) -> &[PsiElement] {
        &self.trailing_commas
    }

    pub fn take_element(&mut self, element: &PsiElement) {
        if Self::is_trailing_comma(element) {
            self.trailing_commas.push(element.clone());
        }
    }

    fn is_trailing_comma(element: &PsiElement) -> bool {
        if !element.is::<LeafPsiElement>() || element.kind() != SyntaxKind::COMMA {
            return false;
        }
        element
            .parent()
            .and_then(|p| extract_managed_list(&p))
            .is_some_and(|l| l.trailing_comma.as_ref() == Some(element))
    }
}

#[derive(Default)]
pub struct Suggestor {
    suggestion_elements: Vec<PsiElement>,
}

impl Suggestor {
    pub fn get_trailing_comma_suggestions(&self) -> &[PsiElement] {
        &self.suggestion_elements
    }

    /// The only elements [`Self::take_element`] can record a suggestion for.
    pub fn may_be_list(element: &PsiElement) -> bool {
        !element.is_leaf() && !element.is_file() && Self::LIST_KINDS.contains(&element.kind())
    }

    /// The node kinds of [`Self::may_be_list`]'s classes (`KtValueArgumentList`, `KtParameterList`,
    /// `KtTypeArgumentList`, `KtTypeParameterList`, `KtCollectionLiteralExpression`, `KtClassBody`).
    pub const LIST_KINDS: [SyntaxKind; 6] = [
        SyntaxKind::VALUE_ARGUMENT_LIST,
        SyntaxKind::VALUE_PARAMETER_LIST,
        SyntaxKind::TYPE_ARGUMENT_LIST,
        SyntaxKind::TYPE_PARAMETER_LIST,
        SyntaxKind::COLLECTION_LITERAL_EXPRESSION,
        SyntaxKind::CLASS_BODY,
    ];

    /// Records the item after which a trailing comma should be inserted: only in multi-line lists of
    /// more than one element that have none yet.
    pub fn take_element(&mut self, element: &KtElement) {
        if element.is::<KtEnumEntry>() || element.is::<KtWhenEntry>() {
            return;
        }
        if element.is::<KtParameterList>() {
            let parent = element.parent();
            if parent.as_ref().is_some_and(|p| p.is::<KtFunctionLiteral>())
                && parent.and_then(|p| p.parent()).is_some_and(|pp| pp.is::<KtLambdaExpression>())
            {
                return; // Never add trailing commas to lambda param lists
            }
        }
        if let Some(class_body) = element.cast::<KtClassBody>() {
            if EnumEntryList::extract_child_list(&class_body).is_some_and(|it| it.terminating_semicolon.is_some()) {
                return; // Never add a trailing comma after there is already a terminating semicolon
            }
        }

        // Cheap necessary conditions go before building the list (all checks here are pure): every
        // item is a composite child, and most lists have fewer than two items or are one line.
        if !Self::may_be_list(element) || !has_two_composite_children(element) || !element.text_contains('\n') {
            return; // Only suggest trailing commas where there is already a line break
        }
        let Some(list) = extract_managed_list(element) else { return };
        if list.items.len() <= 1 {
            return; // Never insert commas to single-element lists
        }
        if list.trailing_comma.is_some() {
            return; // Never insert a comma if there already is one somehow
        }

        self.suggestion_elements.push(left_leaf_ignoring_comments_and_whitespace(list.items.last().unwrap()));
    }
}

struct ManagedList {
    items: Vec<PsiElement>,
    trailing_comma: Option<PsiElement>,
}

fn managed<T: Into<PsiElement>>(items: Vec<T>, trailing_comma: Option<PsiElement>) -> ManagedList {
    ManagedList { items: items.into_iter().map(Into::into).collect(), trailing_comma }
}

fn extract_managed_list(element: &PsiElement) -> Option<ManagedList> {
    if let Some(e) = element.cast::<KtValueArgumentList>() {
        Some(managed(e.arguments(), e.trailing_comma()))
    } else if let Some(e) = element.cast::<KtParameterList>() {
        Some(managed(e.parameters(), e.trailing_comma()))
    } else if let Some(e) = element.cast::<KtTypeArgumentList>() {
        Some(managed(e.arguments(), e.trailing_comma()))
    } else if let Some(e) = element.cast::<KtTypeParameterList>() {
        Some(managed(e.parameters(), e.trailing_comma()))
    } else if let Some(e) = element.cast::<KtCollectionLiteralExpression>() {
        Some(managed(e.inner_expressions(), e.trailing_comma()))
    } else if let Some(e) = element.cast::<KtWhenEntry>() {
        Some(managed(e.conditions(), e.trailing_comma()))
    } else if let Some(e) = element.cast::<KtEnumEntry>() {
        let it = EnumEntryList::extract_parent_list(&e);
        Some(managed(it.enum_entries, it.trailing_comma))
    } else if let Some(e) = element.cast::<KtClassBody>() {
        EnumEntryList::extract_child_list(&e).map(|it| managed(it.enum_entries, it.trailing_comma))
    } else {
        None
    }
}

fn has_two_composite_children(element: &PsiElement) -> bool {
    let tree = element.tree();
    let (mut child, end) = (element.id() + 1, tree.subtree_end(element.id()));
    let mut count = 0;
    while child < end {
        count += usize::from(!tree.is_token(child));
        if count == 2 {
            return true;
        }
        child = tree.subtree_end(child);
    }
    false
}

/// The element after which a comma belongs for a list item: its last leaf that isn't a comment or whitespace.
fn left_leaf_ignoring_comments_and_whitespace(element: &PsiElement) -> PsiElement {
    let mut child = element.last_child();
    while let Some(c) = child {
        if c.is::<PsiWhiteSpace>() || c.is::<PsiComment>() {
            child = c.prev_sibling();
        } else {
            return left_leaf_ignoring_comments_and_whitespace(&c);
        }
    }
    element.clone()
}
