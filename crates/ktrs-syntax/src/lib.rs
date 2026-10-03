//! Lossless Kotlin syntax tree. Node and token kinds mirror the Kotlin compiler's PSI element
//! types one-to-one (see `kinds.tsv`), so trees can be diffed against `DebugUtil.psiToString`.

pub mod caught_panic;
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
    /// Builders (the file's, or a chameleon's) that stopped before their last token, in build order.
    pub missed_tokens: Vec<MissedTokens>,
}

impl Parse {
    pub fn has_errors(&self) -> bool {
        !self.error_messages.is_empty()
    }

    /// The missed tokens a preorder walk meets first (the first to throw).
    pub fn first_missed_tokens(&self) -> Option<&MissedTokens> {
        self.missed_tokens.iter().min_by_key(|m| m.element)
    }
}

/// `PsiBuilderImpl.prepareLightTree`'s `LOG.error("Tokens [..] were not inserted into the tree. ..")`: the tree
/// leaves those tokens out. IntelliJ's `DefaultLogger` (what ktfmt and ktlint run with) prints [`Self::log`] to
/// stderr and throws `AssertionError`, which their CLIs don't catch. Both expand every chameleon before they
/// look for error elements, so it wins over a parse error anywhere in the file (research/24, finding 4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissedTokens {
    /// The builder's root: the file, or the expanded chameleon.
    pub element: ElementId,
    /// The `debug_name`s of the tokens from the builder's position to its end.
    pub tokens: Vec<&'static str>,
    /// The builder's text (the error's `missedTokensFragment.txt` attachment).
    pub text: String,
}

impl MissedTokens {
    pub fn message(&self) -> String {
        format!("Tokens [{}] were not inserted into the tree. Language: kotlin", self.tokens.join(", "))
    }

    /// `DefaultLogger.error`'s stderr output, without the final line separator (it `println`s).
    pub fn log(&self) -> String {
        format!("ERROR: {}\nDetails:\nmissedTokensFragment.txt\n{}", self.message(), self.text)
    }

    /// The thrown error's `toString()`.
    pub fn assertion_error(&self) -> String {
        format!("java.lang.AssertionError: {}", self.message())
    }
}
