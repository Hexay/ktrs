//! `PsiElement` / `ASTNode` over the flat [`Tree`]. IntelliJ semantics from intellij-community
//! idea/251.27812.49 (`ASTDelegatePsiElement`, `CompositePsiElement`, `LazyParseablePsiElement`,
//! `PsiFileImpl`, `LeafPsiElement`, `CompositeElement`).

use std::rc::Rc;

use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::{ElementId, SyntaxKind, TextRange, Tree};

pub use crate::ast_node::AstNode;
use crate::cast::PsiType;
use crate::classes;

/// One PSI element: a composite node or a leaf token. Equality is tree identity.
#[derive(Clone)]
pub struct PsiElement {
    tree: Rc<Tree>,
    id: ElementId,
}

impl PartialEq for PsiElement {
    fn eq(&self, other: &PsiElement) -> bool {
        self.id == other.id && Rc::ptr_eq(&self.tree, &other.tree)
    }
}

impl Eq for PsiElement {}

impl std::hash::Hash for PsiElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.tree).hash(state);
        self.id.hash(state);
    }
}

impl std::fmt::Debug for PsiElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{:?}", self.kind().debug_name(), self.text_range())
    }
}

impl PsiElement {
    /// The file element of `tree`.
    pub fn root(tree: Rc<Tree>) -> PsiElement {
        PsiElement { tree, id: Tree::ROOT }
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub fn id(&self) -> ElementId {
        self.id
    }

    /// Another element of the same tree.
    pub fn at(&self, id: ElementId) -> PsiElement {
        PsiElement { tree: self.tree.clone(), id }
    }

    /// Repoints this handle at another element of the same tree.
    pub(crate) fn move_to(&mut self, id: ElementId) {
        self.id = id;
    }

    pub fn is_leaf(&self) -> bool {
        self.tree.is_token(self.id)
    }

    /// `getNode().getElementType()`.
    pub fn element_type(&self) -> SyntaxKind {
        self.kind()
    }

    pub fn kind(&self) -> SyntaxKind {
        self.tree.kind(self.id)
    }

    /// The `PsiFile` (root) has no PSI parent.
    pub fn is_file(&self) -> bool {
        self.id == Tree::ROOT
    }

    pub fn cast<T: PsiType>(&self) -> Option<T> {
        T::can_cast(self).then(|| T::cast_unchecked(self.clone()))
    }

    pub fn is<T: PsiType>(&self) -> bool {
        T::can_cast(self)
    }

    /// Offsets are UTF-8 byte offsets into the file text (IntelliJ uses UTF-16 units).
    pub fn text_range(&self) -> TextRange {
        self.tree.text_range(self.id)
    }

    /// psiUtil `startOffset`.
    pub fn start_offset(&self) -> usize {
        self.text_range().start().into()
    }

    /// psiUtil `endOffset`.
    pub fn end_offset(&self) -> usize {
        self.text_range().end().into()
    }

    pub fn text_length(&self) -> usize {
        self.text_range().len().into()
    }

    pub fn parent(&self) -> Option<PsiElement> {
        self.tree.parent(self.id).map(|p| self.at(p))
    }

    pub fn first_child(&self) -> Option<PsiElement> {
        self.tree.first_child(self.id).map(|c| self.at(c))
    }

    pub fn last_child(&self) -> Option<PsiElement> {
        self.tree.last_child(self.id).map(|c| self.at(c))
    }

    pub fn next_sibling(&self) -> Option<PsiElement> {
        self.tree.next_sibling(self.id).map(|s| self.at(s))
    }

    pub fn prev_sibling(&self) -> Option<PsiElement> {
        self.tree.prev_sibling(self.id).map(|s| self.at(s))
    }

    /// Every child including leaves, in order (the `getFirstChild`/`getNextSibling` walk).
    pub fn all_children(&self) -> impl Iterator<Item = PsiElement> + use<> {
        let tree = self.tree.clone();
        let (mut next, end) = (self.id + 1, tree.subtree_end(self.id));
        std::iter::from_fn(move || {
            let id = (next < end).then_some(next)?;
            next = tree.subtree_end(id);
            Some(PsiElement { tree: tree.clone(), id })
        })
    }

    /// The child ids, leaves included, in order: each next sibling starts where the child's subtree ends.
    pub(crate) fn child_id_iter(&self) -> impl Iterator<Item = ElementId> + '_ {
        let (mut next, end) = (self.id + 1, self.tree.subtree_end(self.id));
        std::iter::from_fn(move || {
            let id = (next < end).then_some(next)?;
            next = self.tree.subtree_end(id);
            Some(id)
        })
    }

    /// `PsiElement.getChildren()`. `ASTDelegatePsiElement` (almost every Kt class) and `KtBlockExpression`
    /// return composite children only; `PsiFileImpl`, `LazyParseablePsiElement` and `CompositePsiElement`
    /// return every child including leaves.
    pub fn children(&self) -> Vec<PsiElement> {
        if self.is_leaf() {
            return Vec::new();
        }
        if self.is_file() || classes::children_include_leaves(self.kind()) {
            return self.child_id_iter().map(|c| self.at(c)).collect();
        }
        self.child_id_iter().filter(|&c| !self.tree.is_token(c)).map(|c| self.at(c)).collect()
    }

    /// `getChildren().length > 0` without collecting (or materializing skipped leaves).
    pub fn has_children(&self) -> bool {
        if self.is_file() || classes::children_include_leaves(self.kind()) {
            return self.tree.first_child(self.id).is_some();
        }
        self.child_id_iter().any(|c| !self.tree.is_token(c))
    }

    pub fn node(&self) -> AstNode {
        AstNode(self.clone())
    }

    // ---- ASTDelegatePsiElement / StubBasedPsiElementBase helpers (AST code path) ----

    /// `findChildByType(IElementType)`: first direct child of `kind`.
    pub fn find_child_by_type<T: PsiType>(&self, kind: SyntaxKind) -> Option<T> {
        self.first_child_by_kind(|k| k == kind).map(T::cast_unchecked)
    }

    /// `findChildByType(TokenSet)`.
    pub fn find_child_by_type_set<T: PsiType>(&self, kinds: TokenSet) -> Option<T> {
        self.first_child_by_kind(|k| kinds.contains(k)).map(T::cast_unchecked)
    }

    /// `findChildrenByType(IElementType)`.
    pub fn find_children_by_type<T: PsiType>(&self, kind: SyntaxKind) -> Vec<T> {
        self.children_by_kind(|k| k == kind).map(T::cast_unchecked).collect()
    }

    /// The first child (leaves included) whose kind matches.
    pub(crate) fn first_child_by_kind(&self, matches: impl Fn(SyntaxKind) -> bool) -> Option<PsiElement> {
        self.child_id_iter().find(|&c| matches(self.tree.kind(c))).map(|c| self.at(c))
    }

    /// The children (leaves included) whose kind matches.
    pub(crate) fn children_by_kind(&self, matches: impl Fn(SyntaxKind) -> bool) -> impl Iterator<Item = PsiElement> {
        self.child_id_iter().filter(move |&c| matches(self.tree.kind(c))).map(|c| self.at(c))
    }

    /// `findLastChildByType(IElementType)`.
    pub fn find_last_child_by_type(&self, kind: SyntaxKind) -> Option<PsiElement> {
        std::iter::successors(self.last_child(), PsiElement::prev_sibling).find(|c| c.kind() == kind)
    }

    /// `findChildByClass(Class)`: first child (leaves included) that is a `T`.
    pub fn find_child_by_class<T: PsiType>(&self) -> Option<T> {
        self.all_children().find_map(|c| c.cast::<T>())
    }

    /// `findChildrenByClass(Class)`.
    pub fn find_children_by_class<T: PsiType>(&self) -> Vec<T> {
        self.all_children().filter_map(|c| c.cast::<T>()).collect()
    }

    /// `getStubOrPsiChild(IElementType)` on the AST path.
    pub fn get_stub_or_psi_child<T: PsiType>(&self, kind: SyntaxKind) -> Option<T> {
        self.find_child_by_type(kind)
    }

    /// `getStubOrPsiChildren(IElementType)` / `getStubOrPsiChildrenAsList` on the AST path.
    pub fn get_stub_or_psi_children<T: PsiType>(&self, kind: SyntaxKind) -> Vec<T> {
        self.find_children_by_type(kind)
    }

    /// `getStubOrPsiChildren(TokenSet)` on the AST path.
    pub fn get_stub_or_psi_children_set<T: PsiType>(&self, kinds: TokenSet) -> Vec<T> {
        self.children_by_kind(|k| kinds.contains(k)).map(T::cast_unchecked).collect()
    }
}
