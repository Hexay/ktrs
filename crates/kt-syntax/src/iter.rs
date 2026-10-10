use std::iter::FusedIterator;
use std::ops::Range;

use ktrs_syntax::ElementId;

use crate::{Node, SourceFile};

/// The children of a node; see [`Node::children`].
#[derive(Clone)]
pub struct Children<'a> {
    next: Option<Node<'a>>,
}

impl<'a> Children<'a> {
    pub(crate) fn new(parent: Node<'a>) -> Children<'a> {
        Children { next: parent.first_child() }
    }
}

impl<'a> Iterator for Children<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Node<'a>> {
        let current = self.next?;
        self.next = current.next_sibling();
        Some(current)
    }
}

impl FusedIterator for Children<'_> {}

/// The enclosing nodes of a node, innermost first; see [`Node::ancestors`].
#[derive(Clone)]
pub struct Ancestors<'a> {
    next: Option<Node<'a>>,
}

impl<'a> Ancestors<'a> {
    pub(crate) fn new(first: Option<Node<'a>>) -> Ancestors<'a> {
        Ancestors { next: first }
    }
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Node<'a>> {
        let current = self.next?;
        self.next = current.parent();
        Some(current)
    }
}

impl FusedIterator for Ancestors<'_> {}

/// A subtree in source order; see [`Node::descendants`]. The tree is stored in this order, so this is a
/// counter, and it also runs backwards.
#[derive(Clone)]
pub struct Descendants<'a> {
    file: &'a SourceFile,
    ids: Range<ElementId>,
}

impl<'a> Descendants<'a> {
    pub(crate) fn new(file: &'a SourceFile, ids: Range<ElementId>) -> Descendants<'a> {
        Descendants { file, ids }
    }
}

impl<'a> Iterator for Descendants<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Node<'a>> {
        self.ids.next().map(|id| Node::new(self.file, id))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ids.size_hint()
    }
}

impl DoubleEndedIterator for Descendants<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.ids.next_back().map(|id| Node::new(self.file, id))
    }
}

impl ExactSizeIterator for Descendants<'_> {}

impl FusedIterator for Descendants<'_> {}

/// A step of a [`Preorder`] walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalkEvent<'a> {
    /// The walk reached the node; its children follow.
    Enter(Node<'a>),
    /// The walk is done with the node and everything below it.
    Leave(Node<'a>),
}

/// A depth-first walk of a subtree that reports each node twice, on the way in and on the way out; see
/// [`Node::preorder`]. Tokens are entered and left like nodes.
///
/// ```
/// use kt_syntax::{SyntaxKind, WalkEvent, parse};
///
/// let file = parse("class A { fun f() { fun local() {} } }\nfun g() {}").unwrap();
/// let mut functions = Vec::new();
/// let mut walk = file.root().preorder();
/// while let Some(event) = walk.next() {
///     if let WalkEvent::Enter(node) = event {
///         if node.kind() == SyntaxKind::FUN {
///             functions.push(node.child(SyntaxKind::IDENTIFIER).unwrap().text());
///             walk.skip_subtree(); // don't look for local functions
///         }
///     }
/// }
/// assert_eq!(functions, ["f", "g"]);
/// ```
#[derive(Clone)]
pub struct Preorder<'a> {
    root: Node<'a>,
    next: Option<WalkEvent<'a>>,
    entered: Option<Node<'a>>,
}

impl<'a> Preorder<'a> {
    pub(crate) fn new(root: Node<'a>) -> Preorder<'a> {
        Preorder { root, next: Some(WalkEvent::Enter(root)), entered: None }
    }

    /// After an [`WalkEvent::Enter`], skips that node's children: the next event is its `Leave`. Does nothing
    /// after a `Leave`.
    pub fn skip_subtree(&mut self) {
        if let Some(node) = self.entered.take() {
            self.next = Some(WalkEvent::Leave(node));
        }
    }
}

impl<'a> Iterator for Preorder<'a> {
    type Item = WalkEvent<'a>;

    fn next(&mut self) -> Option<WalkEvent<'a>> {
        let event = self.next?;
        self.entered = None;
        self.next = match event {
            WalkEvent::Enter(node) => {
                self.entered = Some(node);
                Some(node.first_child().map_or(WalkEvent::Leave(node), WalkEvent::Enter))
            }
            WalkEvent::Leave(node) if node == self.root => None,
            WalkEvent::Leave(node) => {
                Some(node.next_sibling().map_or_else(|| WalkEvent::Leave(node.parent().unwrap_or(self.root)), WalkEvent::Enter))
            }
        };
        Some(event)
    }
}

impl FusedIterator for Preorder<'_> {}
