//! `ASTNode` over the flat tree: the node view of a [`PsiElement`].

use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::{SyntaxKind, TextRange};

use crate::classes;
use crate::element::PsiElement;

/// `ASTNode` view of a [`PsiElement`] (same underlying tree element).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AstNode(pub(crate) PsiElement);

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
