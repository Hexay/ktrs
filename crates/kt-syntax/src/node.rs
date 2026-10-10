use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Range;

use ktrs_syntax::{ElementId, SyntaxKind as Raw};

use crate::iter::{Ancestors, Children, Descendants, Preorder};
use crate::{SourceFile, SyntaxKind};

/// A node or a token of a [`SourceFile`]'s tree.
///
/// The compiler's PSI has one element type for both, and so has this crate: a **token** is a leaf that owns
/// text ([`is_token`](Node::is_token)), anything else is a composite node, which may have no children (an empty
/// `VALUE_PARAMETER_LIST` has its parentheses, an empty `ERROR_ELEMENT` has nothing).
///
/// A `Node` is a `Copy` handle of two words that borrows its file; navigation is index arithmetic and never
/// allocates. Two nodes are equal when they are the same element of the same `SourceFile`.
#[derive(Clone, Copy)]
pub struct Node<'a> {
    file: &'a SourceFile,
    id: ElementId,
}

impl<'a> Node<'a> {
    pub(crate) fn new(file: &'a SourceFile, id: ElementId) -> Node<'a> {
        Node { file, id }
    }

    pub(crate) fn id(self) -> ElementId {
        self.id
    }

    fn at(self, id: ElementId) -> Node<'a> {
        Node { file: self.file, id }
    }

    /// The file this node belongs to.
    pub fn file(self) -> &'a SourceFile {
        self.file
    }

    /// The node's kind.
    pub fn kind(self) -> SyntaxKind {
        SyntaxKind(self.raw_kind())
    }

    pub(crate) fn raw_kind(self) -> Raw {
        self.file.tree().kind(self.id)
    }

    /// Whether this is a token (a leaf with text) rather than a composite node.
    pub fn is_token(self) -> bool {
        self.file.tree().is_token(self.id)
    }

    /// White space or a comment; a KDoc comment is trivia although it is a node with children.
    pub fn is_trivia(self) -> bool {
        self.raw_kind().is_trivia()
    }

    /// A `WHITE_SPACE` token.
    pub fn is_whitespace(self) -> bool {
        self.raw_kind() == Raw::WHITE_SPACE
    }

    /// An end-of-line, block or shebang comment token, or a KDoc node.
    pub fn is_comment(self) -> bool {
        self.kind().is_comment()
    }

    /// An `ERROR_ELEMENT`: a syntax error.
    pub fn is_error(self) -> bool {
        self.raw_kind() == Raw::ERROR_ELEMENT
    }

    /// The compiler's message if this is an `ERROR_ELEMENT`.
    pub fn error_message(self) -> Option<&'a str> {
        self.file.error_message(self.id)
    }

    /// The source text of the node: for a composite node, everything its tokens spell.
    pub fn text(self) -> &'a str {
        self.file.tree().text_of(self.id)
    }

    /// The byte range of the node in [`SourceFile::text`].
    pub fn range(self) -> Range<usize> {
        let range = self.file.tree().text_range(self.id);
        range.start().into()..range.end().into()
    }

    /// The enclosing node; `None` for the root.
    pub fn parent(self) -> Option<Node<'a>> {
        self.file.tree().parent(self.id).map(|id| self.at(id))
    }

    /// The first child, token or node, trivia included.
    pub fn first_child(self) -> Option<Node<'a>> {
        self.file.tree().first_child(self.id).map(|id| self.at(id))
    }

    /// The last child, token or node, trivia included.
    pub fn last_child(self) -> Option<Node<'a>> {
        self.file.tree().last_child(self.id).map(|id| self.at(id))
    }

    /// The next child of the same parent.
    pub fn next_sibling(self) -> Option<Node<'a>> {
        self.file.tree().next_sibling(self.id).map(|id| self.at(id))
    }

    /// The previous child of the same parent.
    pub fn prev_sibling(self) -> Option<Node<'a>> {
        self.file.tree().prev_sibling(self.id).map(|id| self.at(id))
    }

    /// The siblings after this node, nearest first.
    pub fn next_siblings(self) -> impl Iterator<Item = Node<'a>> + use<'a> {
        std::iter::successors(self.next_sibling(), |node| node.next_sibling())
    }

    /// The siblings before this node, nearest first.
    pub fn prev_siblings(self) -> impl Iterator<Item = Node<'a>> + use<'a> {
        std::iter::successors(self.prev_sibling(), |node| node.prev_sibling())
    }

    /// All children in source order: nodes and tokens, trivia included.
    pub fn children(self) -> Children<'a> {
        Children::new(self)
    }

    /// The children that are composite nodes (no tokens, so no white space and no comments except KDoc).
    pub fn child_nodes(self) -> impl Iterator<Item = Node<'a>> + use<'a> {
        self.children().filter(|child| !child.is_token())
    }

    /// The first child of `kind`.
    pub fn child(self, kind: SyntaxKind) -> Option<Node<'a>> {
        self.children().find(|child| child.raw_kind() == kind.0)
    }

    /// The enclosing nodes, innermost first, up to the root; the node itself is not included.
    pub fn ancestors(self) -> Ancestors<'a> {
        Ancestors::new(self.parent())
    }

    /// The node and everything below it, tokens included, in source (pre)order.
    pub fn descendants(self) -> Descendants<'a> {
        Descendants::new(self.file, self.id..self.file.tree().subtree_end(self.id))
    }

    /// An enter/leave walk of the node's subtree that can skip subtrees; see [`Preorder`].
    pub fn preorder(self) -> Preorder<'a> {
        Preorder::new(self)
    }

    /// The tokens of the node in source order, trivia included: their texts concatenate to [`text`](Node::text).
    /// A token yields itself.
    pub fn tokens(self) -> impl DoubleEndedIterator<Item = Node<'a>> + use<'a> {
        self.descendants().filter(|node| node.is_token())
    }

    /// The first token of the node (itself for a token); `None` for a node without tokens.
    pub fn first_token(self) -> Option<Node<'a>> {
        self.tokens().next()
    }

    /// The last token of the node (itself for a token); `None` for a node without tokens.
    pub fn last_token(self) -> Option<Node<'a>> {
        self.tokens().next_back()
    }

    /// The token that follows the node in the file, whatever its parent.
    pub fn next_token(self) -> Option<Node<'a>> {
        let tree = self.file.tree();
        (tree.subtree_end(self.id)..tree.len() as ElementId).map(|id| self.at(id)).find(|node| node.is_token())
    }

    /// The token that precedes the node in the file, whatever its parent.
    pub fn prev_token(self) -> Option<Node<'a>> {
        (0..self.id).rev().map(|id| self.at(id)).find(|node| node.is_token())
    }
}

impl PartialEq for Node<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && std::ptr::eq(self.file, other.file)
    }
}

impl Eq for Node<'_> {}

impl Hash for Node<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::from_ref(self.file).hash(state);
        self.id.hash(state);
    }
}

impl fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let range = self.range();
        write!(f, "{}@{}..{}", self.kind(), range.start, range.end)
    }
}
