#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod dump;
mod edit;
mod error;
mod generated {
    pub(crate) mod kinds;
}
mod guard;
mod iter;
mod kind;
mod line_index;
mod node;
mod query;
mod source_file;

pub use edit::{EditError, TextEdit, apply_edits};
pub use error::{ParseError, SyntaxError};
pub use guard::MAX_NESTING_DEPTH;
pub use iter::{Ancestors, Children, Descendants, Preorder, WalkEvent};
pub use kind::SyntaxKind;
pub use line_index::LineCol;
pub use node::Node;
pub use query::FindAll;
pub use source_file::{SourceFile, parse, parse_script};

/// The version of the Kotlin compiler whose parser this is a port of. The kinds and the shape of the tree
/// for a given text are the ones this compiler version produces.
pub const KOTLIN_VERSION: &str = "2.4.20";
