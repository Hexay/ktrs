//! `PsiElement` / `ASTNode` over the flat [`Tree`]. IntelliJ semantics from intellij-community
//! idea/251.27812.49 (`ASTDelegatePsiElement`, `CompositePsiElement`, `LazyParseablePsiElement`,
//! `PsiFileImpl`, `LeafPsiElement`, `CompositeElement`).

use std::rc::Rc;

use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::{ElementId, SyntaxKind, TextRange, Tree};

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

    /// `PsiElement.getTextOffset()` for elements that don't override it (ktfmt only reads it for arguments).
    pub fn text_offset(&self) -> usize {
        self.start_offset()
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
        std::iter::successors(self.first_child(), PsiElement::next_sibling)
    }

    /// `PsiElement.getChildren()`. `ASTDelegatePsiElement` (almost every Kt class) and `KtBlockExpression`
    /// return composite children only; `PsiFileImpl`, `LazyParseablePsiElement` and `CompositePsiElement`
    /// return every child including leaves.
    pub fn children(&self) -> Vec<PsiElement> {
        if self.is_leaf() {
            return Vec::new();
        }
        if self.is_file() || classes::children_include_leaves(self.kind()) {
            return self.all_children().collect();
        }
        self.all_children().filter(|c| !c.is_leaf()).collect()
    }

    /// `getChildren().length > 0` without collecting (or materializing skipped leaves).
    pub fn has_children(&self) -> bool {
        if self.is_file() || classes::children_include_leaves(self.kind()) {
            return self.tree.first_child(self.id).is_some();
        }
        self.tree.children(self.id).any(|c| !self.tree.is_token(c))
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
    fn first_child_by_kind(&self, matches: impl Fn(SyntaxKind) -> bool) -> Option<PsiElement> {
        self.tree.children(self.id).find(|&c| matches(self.tree.kind(c))).map(|c| self.at(c))
    }

    /// The children (leaves included) whose kind matches.
    fn children_by_kind(&self, matches: impl Fn(SyntaxKind) -> bool) -> impl Iterator<Item = PsiElement> {
        self.all_children().filter(move |c| matches(c.kind()))
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

/// `ASTNode` view of a [`PsiElement`] (same underlying tree element).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AstNode(PsiElement);

impl AstNode {
    pub fn psi(&self) -> PsiElement {
        self.0.clone()
    }

    pub fn element_type(&self) -> SyntaxKind {
        self.0.kind()
    }

    pub fn text(&self) -> String {
        self.0.text()
    }

    pub fn text_slice(&self) -> &str {
        self.0.text_slice()
    }

    pub fn text_range(&self) -> TextRange {
        self.0.text_range()
    }

    pub fn start_offset(&self) -> usize {
        self.0.start_offset()
    }

    pub fn text_contains(&self, c: char) -> bool {
        self.0.text_contains(c)
    }

    /// Kotlin `node is PsiElement`: true for leaves and for composites whose node is their own PSI
    /// (`LazyParseablePsiElement` / `CompositePsiElement` classes); false for `ASTWrapperPsiElement` classes.
    pub fn is_psi_element(&self) -> bool {
        self.0.is_leaf() || (!self.0.is_file() && classes::node_is_psi(self.0.kind()))
    }

    pub fn first_child_node(&self) -> Option<AstNode> {
        self.0.first_child().map(AstNode)
    }

    pub fn last_child_node(&self) -> Option<AstNode> {
        self.0.last_child().map(AstNode)
    }

    pub fn tree_next(&self) -> Option<AstNode> {
        self.0.next_sibling().map(AstNode)
    }

    pub fn tree_prev(&self) -> Option<AstNode> {
        self.0.prev_sibling().map(AstNode)
    }

    pub fn tree_parent(&self) -> Option<AstNode> {
        self.0.parent().map(AstNode)
    }

    /// psiUtil `ASTNode.children()`: every child node, leaves included.
    pub fn children(&self) -> impl Iterator<Item = AstNode> + use<> {
        self.0.all_children().map(AstNode)
    }

    /// `ASTNode.findChildByType(IElementType)`.
    pub fn find_child_by_type(&self, kind: SyntaxKind) -> Option<AstNode> {
        self.0.first_child_by_kind(|k| k == kind).map(AstNode)
    }

    /// `ASTNode.findChildByType(TokenSet)`.
    pub fn find_child_by_type_set(&self, kinds: TokenSet) -> Option<AstNode> {
        self.0.first_child_by_kind(|k| kinds.contains(k)).map(AstNode)
    }

    /// `ASTNode.getChildren(TokenSet)`.
    pub fn get_children(&self, kinds: TokenSet) -> Vec<AstNode> {
        self.0.children_by_kind(|k| kinds.contains(k)).map(AstNode).collect()
    }
}
