//! Lossless Kotlin syntax tree. Node and token kinds mirror the Kotlin compiler's PSI element
//! types one-to-one (see `kinds.tsv`), so trees can be diffed against `DebugUtil.psiToString`.

mod dump;
mod generated {
    pub(crate) mod kinds;
}
pub mod tree;

use std::rc::Rc;

pub use dump::psi_dump;
pub use generated::kinds::SyntaxKind;
pub use text_size::{TextRange, TextSize};
pub use tree::{ElementId, Tree, TreeBuilder};

impl SyntaxKind {
    pub fn from_raw(raw: u16) -> SyntaxKind {
        generated::kinds::ALL[raw as usize]
    }

    pub fn is_trivia(self) -> bool {
        matches!(
            self,
            SyntaxKind::WHITE_SPACE
                | SyntaxKind::EOL_COMMENT
                | SyntaxKind::BLOCK_COMMENT
                | SyntaxKind::SHEBANG_COMMENT
                | SyntaxKind::DOC_COMMENT
        )
    }
}

/// A parsed file: the tree plus one message per `ERROR_ELEMENT`, in tree preorder.
#[derive(Debug, Clone)]
pub struct Parse {
    pub tree: Rc<Tree>,
    pub error_messages: Vec<String>,
}

impl Parse {
    pub fn has_errors(&self) -> bool {
        !self.error_messages.is_empty()
    }
}
