//! Searching by kind, comments and KDoc, and the edits a node can produce.

use ktrs_syntax::SyntaxKind as Raw;
use ktrs_syntax::tree::KindScan;

use crate::iter::Descendants;
use crate::{Node, SourceFile, SyntaxKind, TextEdit};

/// `Tree::find_kinds` takes up to twelve raw kinds, and each kind is looked for as a node and as a token.
const SCANNED_KINDS: usize = 6;

/// The nodes and tokens of the given kinds in a subtree; see [`Node::find_all`].
pub struct FindAll<'a> {
    file: &'a SourceFile,
    search: Search<'a>,
}

enum Search<'a> {
    Scan(KindScan<'a>),
    Filter(Descendants<'a>, Vec<Raw>),
}

impl<'a> Iterator for FindAll<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Node<'a>> {
        match &mut self.search {
            Search::Scan(scan) => scan.next().map(|id| Node::new(self.file, id)),
            Search::Filter(nodes, kinds) => nodes.find(|node| kinds.contains(&node.raw_kind())),
        }
    }
}

impl<'a> Node<'a> {
    /// The node itself and its descendants that have one of `kinds`, in source order. This scans the tree's
    /// kind array, so it is the fastest way to collect, say, every `FUN` or every `CALL_EXPRESSION` of a file.
    pub fn find_all(self, kinds: &[SyntaxKind]) -> FindAll<'a> {
        let file = self.file();
        let search = match kinds.first() {
            Some(first) if kinds.len() <= SCANNED_KINDS => {
                let mut wanted = [first.0; SCANNED_KINDS];
                for (slot, kind) in wanted.iter_mut().zip(kinds) {
                    *slot = kind.0;
                }
                Search::Scan(file.tree().find_kinds(self.id(), wanted, wanted))
            }
            _ => Search::Filter(self.descendants(), kinds.iter().map(|kind| kind.0).collect()),
        };
        FindAll { file, search }
    }

    /// The comments in this node's subtree, in source order: `EOL_COMMENT`, `BLOCK_COMMENT` and
    /// `SHEBANG_COMMENT` tokens and `DOC_COMMENT` (KDoc) nodes. Use it on the root for every comment of a file.
    pub fn comments(self) -> FindAll<'a> {
        self.find_all(&[SyntaxKind::EOL_COMMENT, SyntaxKind::BLOCK_COMMENT, SyntaxKind::SHEBANG_COMMENT, SyntaxKind::DOC_COMMENT])
    }

    /// The KDoc comment of a declaration: its `DOC_COMMENT` child. The compiler makes a `/** .. */` comment
    /// directly before a declaration the declaration's first child, which is what its `getDocComment()` reads.
    ///
    /// The result is a node: `KDOC_SECTION` children hold the text and the `KDOC_TAG`s (`@param`, `@return`),
    /// and [`text`](Node::text) is the whole comment.
    pub fn doc_comment(self) -> Option<Node<'a>> {
        self.child(SyntaxKind::DOC_COMMENT)
    }

    /// The comments the parser attached to the start of this node: its children that are comments, up to the
    /// first child that is neither comment nor white space. Kotlin binds the comments directly before a
    /// declaration to that declaration, KDoc included.
    pub fn leading_comments(self) -> impl Iterator<Item = Node<'a>> + use<'a> {
        self.children().take_while(|child| child.is_trivia()).filter(|child| child.is_comment())
    }

    /// An edit that replaces this node's text.
    pub fn replace(self, text: impl Into<String>) -> TextEdit {
        TextEdit::replace(self.range(), text)
    }

    /// An edit that deletes this node's text. Surrounding white space stays.
    pub fn remove(self) -> TextEdit {
        TextEdit::delete(self.range())
    }

    /// An edit that inserts `text` directly before this node.
    pub fn insert_before(self, text: impl Into<String>) -> TextEdit {
        TextEdit::insert(self.range().start, text)
    }

    /// An edit that inserts `text` directly after this node.
    pub fn insert_after(self, text: impl Into<String>) -> TextEdit {
        TextEdit::insert(self.range().end, text)
    }
}
