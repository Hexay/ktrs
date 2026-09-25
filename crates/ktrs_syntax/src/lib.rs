//! Lossless Kotlin syntax tree. Node and token kinds mirror the Kotlin compiler's PSI element
//! types one-to-one (see `kinds.tsv`), so trees can be diffed against `DebugUtil.psiToString`.

mod dump;
mod generated {
    pub(crate) mod kinds;
}

pub use dump::psi_dump;
pub use generated::kinds::SyntaxKind;
pub use rowan::{GreenNode, TextRange, TextSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KotlinLanguage {}

impl rowan::Language for KotlinLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind {
        SyntaxKind::from_raw(raw.0)
    }

    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

pub type SyntaxNode = rowan::SyntaxNode<KotlinLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<KotlinLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<KotlinLanguage>;

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

/// A parsed file: the green tree plus one message per `ERROR_ELEMENT`, in tree preorder.
#[derive(Debug, Clone)]
pub struct Parse {
    pub green: GreenNode,
    pub error_messages: Vec<String>,
}

impl Parse {
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }

    pub fn has_errors(&self) -> bool {
        !self.error_messages.is_empty()
    }
}
