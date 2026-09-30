//! Port of `KDocFormatter.kt`.

use super::comment_type::{is_block_comment, is_line_comment};
use super::formatting_task::FormattingTask;
use super::kstring::{KStr, KString, ks, to_string, w};
use super::options::KDocFormattingOptions;
use super::paragraph_list_builder::ParagraphListBuilder;
use super::utilities::get_indent_size;

/// Formatter which can reformat KDoc comments.
pub struct KDocFormatter {
    options: KDocFormattingOptions,
}

impl KDocFormatter {
    pub fn new(options: KDocFormattingOptions) -> Self {
        KDocFormatter { options }
    }

    /// Reformats the [comment], which follows the given [initial_indent] string.
    pub fn reformat_comment(&self, comment: &str, initial_indent: &str) -> String {
        self.reformat_comment_task(&FormattingTask::new(self.options, comment, initial_indent))
    }

    pub fn reformat_comment_task(&self, task: &FormattingTask) -> String {
        let options = &self.options;
        let indent = ks(&task.secondary_indent);
        let indent_size = get_indent_size(&indent, options);
        let first_indent_size = get_indent_size(&ks(&task.initial_indent), options);
        let comment = &task.comment;
        let line_comment = is_line_comment(comment);
        let block_comment = is_block_comment(comment);
        let paragraphs = ParagraphListBuilder::new(comment, options, task).scan(indent_size);
        let comment_type = task.comment_type;
        let line_separator: KString = [w!("\n"), &indent, &ks(comment_type.line_prefix())].concat();
        let prefix = ks(comment_type.prefix());

        // Collapse single line? If alternate is turned on, use the opposite of the setting
        let collapse_line = options.collapse_single_line != options.alternate;
        if paragraphs.is_single_paragraph() && collapse_line && !line_comment {
            // Does the text fit on a single line?
            let trimmed = paragraphs.iter().next().map_or(KString::new(), |p| p.text().trim().to_vec());
            // Subtract out space for "/** " and " */" and the indent:
            let width = (options.max_line_width - first_indent_size - comment_type.single_line_overhead())
                .min(options.max_comment_width);
            let suffix =
                if comment_type.suffix().is_empty() { String::new() } else { format!(" {}", comment_type.suffix()) };
            let single_line = || format!("{} {}{}", comment_type.prefix(), to_string(&trimmed), suffix);
            if trimmed.len() as i32 <= width {
                return single_line();
            }
            if indent_size < first_indent_size {
                let next_line_width = (options.max_line_width - indent_size - comment_type.single_line_overhead())
                    .min(options.max_comment_width);
                if trimmed.len() as i32 <= next_line_width {
                    return single_line();
                }
            }
        }

        let mut sb = KString::new();

        sb.extend_from_slice(&prefix);
        if line_comment {
            sb.push(' ' as u16);
        } else {
            sb.extend_from_slice(&line_separator);
        }

        for paragraph in paragraphs.iter() {
            if paragraph.separate {
                // Remove trailing spaces which can happen when we have a paragraph separator
                self.strip_trailing_spaces(line_comment, &mut sb);
                sb.extend_from_slice(&line_separator);
            }
            let text = paragraph.text();
            if paragraph.preformatted || paragraph.table {
                sb.extend_from_slice(text);
                // Remove trailing spaces (e.g. from an empty line in a preformatted paragraph).
                self.strip_trailing_spaces(line_comment, &mut sb);
                sb.extend_from_slice(&line_separator);
                continue;
            }

            let line_without_indent = options.max_line_width - comment_type.line_overhead();
            let quote_adjustment = if paragraph.quoted > 0 { 2 * paragraph.quoted } else { 0 };
            let max_line_width =
                options.max_comment_width.min(line_without_indent - indent_size) - quote_adjustment;
            let first_max_line_width = if !sb.contains(&('\n' as u16)) {
                options.max_comment_width.min(line_without_indent - first_indent_size) - quote_adjustment
            } else {
                max_line_width
            };

            let lines = paragraph.reflow(first_max_line_width, max_line_width);
            let mut first = true;
            let hanging_indent = &paragraph.hanging_indent;
            for line in &lines {
                sb.extend_from_slice(&paragraph.indent);
                if first && !paragraph.continuation {
                    first = false;
                } else {
                    sb.extend_from_slice(hanging_indent);
                }
                for _ in 0..paragraph.quoted.max(0) {
                    sb.extend_from_slice(w!("> "));
                }
                if line.is_empty() {
                    // Remove trailing spaces which can happen when we have a paragraph separator
                    self.strip_trailing_spaces(line_comment, &mut sb);
                } else {
                    sb.extend_from_slice(line);
                }
                sb.extend_from_slice(&line_separator);
            }
        }
        if !line_comment {
            if sb.ends_with(w!("* ")) {
                sb.truncate(sb.len() - 2);
            }
            sb.extend_from_slice(w!("*/"));
        }
        // (Upstream's `sb.removeSuffix(lineSeparator)` for line comments is a no-op.)

        let formatted = if line_comment {
            to_string(sb.trim().remove_suffix(w!("//")).trim())
        } else if block_comment {
            to_string(&sb.replace_seq(&[&line_separator[..], w!("\n")].concat(), w!("\n\n"), false))
        } else {
            to_string(&sb)
        };

        let separator_index = comment.find('\n');
        match separator_index {
            // CRLF separator
            Some(i) if i > 0 && comment.as_bytes()[i - 1] == b'\r' => formatted.replace('\n', "\r\n"),
            _ => formatted,
        }
    }

    fn strip_trailing_spaces(&self, line_comment: bool, sb: &mut KString) {
        if !line_comment && sb.ends_with(w!("* ")) {
            sb.truncate(sb.len() - 1);
        } else if line_comment && sb.ends_with(w!("// ")) {
            sb.truncate(sb.len() - 1);
        }
    }
}
