//! Port of ktfmt's KDoc formatter (`com.facebook.ktfmt.kdoc`). Strings are handled as UTF-16 (see
//! `kstring`) so widths and indices match the JVM exactly. Upstream exceptions become panics.

mod char_tables;
mod comment_type;
mod comments_helper;
mod escaping;
mod formatter;
mod formatting_task;
mod kdoc_token;
mod kdoc_writer;
pub mod kstring;
mod options;
mod paragraph;
mod paragraph_list;
mod paragraph_breaking;
mod paragraph_list_builder;
mod paragraph_list_builder_adjust;
mod paragraph_list_builder_arrange;
mod paragraph_list_builder_blocks;
mod paragraph_list_builder_scan;
mod paragraph_reflow;
mod table;
pub mod utilities;

#[cfg(test)]
mod tests;

pub use comment_type::{CommentType, comment_type, is_block_comment, is_kdoc_comment, is_line_comment};
pub use comments_helper::{CommentTok, KDocCommentsHelper};
pub use escaping::{escape_kdoc, index_of_comment_escape_sequences, unescape_kdoc};
pub use formatter::KDocFormatter;
pub use formatting_task::FormattingTask;
pub use kdoc_token::{KDocToken, KDocTokenType, NestingCounter};
pub use kdoc_writer::{KDocWriter, RequestedWhitespace};
pub use options::KDocFormattingOptions;
pub use paragraph::Paragraph;
pub use paragraph_list::ParagraphList;
pub use paragraph_list_builder::ParagraphListBuilder;
pub use table::{Align, Row, Table};
