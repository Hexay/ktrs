use std::fmt;
use std::ops::Range;

use crate::Node;

/// Why [`parse`](crate::parse) returned no tree. Syntax errors are not among the reasons: they are part of the
/// tree ([`SourceFile::errors`](crate::SourceFile::errors)).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    /// Brackets are nested deeper than [`MAX_NESTING_DEPTH`](crate::MAX_NESTING_DEPTH). The parser is recursive, so
    /// deeper input is refused rather than risking the stack.
    TooDeeplyNested {
        /// Offset, in the normalized text, of the opening bracket that exceeds the limit.
        offset: usize,
    },
    /// The text is 4 GiB or longer; offsets are 32-bit.
    TooLarge {
        /// Length of the normalized text in bytes.
        len: usize,
    },
    /// Kotlin's parser stopped before the last token of the file or of a block, lambda or KDoc it re-parses,
    /// and left the remaining tokens out of the tree. IntelliJ logs this as an error ("Tokens [..] were not
    /// inserted into the tree") and ktfmt and ktlint fail on it; the tree would not spell the input, so none is
    /// returned. Only malformed input does this.
    TokensNotInserted {
        /// The dump names of the tokens left out.
        tokens: Vec<String>,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::TooDeeplyNested { offset } => {
                write!(f, "brackets nested deeper than {} at offset {offset}", crate::MAX_NESTING_DEPTH)
            }
            ParseError::TooLarge { len } => write!(f, "text of {len} bytes is too large (the limit is 4 GiB)"),
            ParseError::TokensNotInserted { tokens } => {
                write!(f, "Tokens [{}] were not inserted into the tree. Language: kotlin", tokens.join(", "))
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// One syntax error: an `ERROR_ELEMENT` node and the compiler's message for it.
///
/// The node is where Kotlin's parser put the error. It is often empty (something is missing at that offset)
/// and otherwise wraps the tokens the parser could not use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxError<'a> {
    pub(crate) node: Node<'a>,
    pub(crate) message: &'a str,
}

impl<'a> SyntaxError<'a> {
    /// The `ERROR_ELEMENT` node.
    pub fn node(&self) -> Node<'a> {
        self.node
    }

    /// The compiler's message, e.g. `Expecting an element`.
    pub fn message(&self) -> &'a str {
        self.message
    }

    /// The node's byte range; empty when the error marks a position.
    pub fn range(&self) -> Range<usize> {
        self.node.range()
    }
}

impl fmt::Display for SyntaxError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let at = self.node.file().line_col(self.range().start).unwrap_or_default();
        write!(f, "{}:{}: {}", at.line + 1, at.col + 1, self.message)
    }
}
