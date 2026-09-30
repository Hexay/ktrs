//! Port of `FormattingTask.kt`.

use super::comment_type::{CommentType, comment_type};
use super::options::KDocFormattingOptions;

#[derive(Clone, Debug)]
pub struct FormattingTask {
    /// Options to format with.
    pub options: KDocFormattingOptions,
    /// The original comment to be formatted.
    pub comment: String,
    /// The initial indentation on the first line of the KDoc; subsequent lines get [Self::secondary_indent].
    pub initial_indent: String,
    /// Indent to use after the first line (differs when the comment trails code on its first line).
    pub secondary_indent: String,
    /// Parameter names in signature order; `@param` tags are sorted to match when `order_doc_tags`.
    pub ordered_parameter_names: Vec<String>,
    /// The type of comment being formatted.
    pub comment_type: CommentType,
}

impl FormattingTask {
    /// `FormattingTask(options, comment, initialIndent)` with upstream's defaults for the rest.
    pub fn new(options: KDocFormattingOptions, comment: &str, initial_indent: &str) -> Self {
        FormattingTask {
            options,
            comment: comment.to_string(),
            initial_indent: initial_indent.to_string(),
            secondary_indent: initial_indent.to_string(),
            ordered_parameter_names: Vec::new(),
            comment_type: comment_type(comment),
        }
    }
}
