//! Port of `MultilineStringFormatter.kt` (lines 26-246): re-indents `"""..."""` strings followed by
//! `.trimIndent()` / `.trimMargin()`.

use std::cell::OnceCell;

use ktrs_psi::{KtFile, KtQualifiedExpression, KtStringTemplateExpression, KtVisitorVoid, PsiComment, PsiElement, kt_tree_visitor_void, kt_visitor_void};

use super::kotlin_text::{LineIndex, is_blank, trim_start};

const TQ: &str = "\"\"\"";

pub struct MultilineStringFormatter {
    pub continuation_indent_size: i32,
}

impl MultilineStringFormatter {
    pub fn new(continuation_indent_size: i32) -> MultilineStringFormatter {
        MultilineStringFormatter { continuation_indent_size }
    }

    pub fn format(&self, file: &KtFile) -> String {
        let mut result = file.text();
        let mut multiline_string_list = self.get_multiline_trimmed_string_list(file);
        multiline_string_list.sort_by_key(|s| std::cmp::Reverse(s.open_string_offset));
        for multiline_string in &multiline_string_list {
            if multiline_string.string_line_count < 2 {
                // Single line multiline strings are left alone
                continue;
            }
            if multiline_string.has_template_expression() {
                // Template output could affect the result of the trimIndent/trimMargin call.
                continue;
            }
            if multiline_string.is_nested_multiline {
                // We currently do not format code inside of template expressions
                continue;
            }
            let indentation = " ".repeat(multiline_string.indent_count);
            let continuation_indentation = " ".repeat(self.continuation_indent_size.max(0) as usize);

            let mut multiline = String::new();
            if multiline_string.is_dollar_string {
                multiline.push_str("$$");
            }
            multiline.push_str(TQ);
            multiline.push('\n');

            let mut is_last_line_empty = true;
            for line_content in multiline_string.get_string_content() {
                if multiline_string.uses_trim_margin || !line_content.is_empty() {
                    multiline.push_str(&indentation);
                    multiline.push_str(multiline_string.indentation_suffix());
                }
                multiline.push_str(&line_content);
                multiline.push('\n');

                is_last_line_empty = line_content.is_empty();
            }

            // Close string
            if multiline_string.uses_trim_margin && is_last_line_empty {
                // Remove the last new line character
                multiline.pop();
            } else {
                multiline.push_str(&indentation);
            }
            multiline.push_str(TQ);
            multiline.push('\n');

            // Preserve any comments between the string and the trim method call
            for comment in &multiline_string.comments_between_string_and_trim_call {
                multiline.push_str(&indentation);
                multiline.push_str(&continuation_indentation);
                multiline.push_str(comment);
                multiline.push('\n');
            }

            // Trim method call
            multiline.push_str(&indentation);
            multiline.push_str(&continuation_indentation);

            // Now replace the original multiline string with the newly formatted one
            result.replace_range(multiline_string.open_string_offset..multiline_string.trim_method_call_offset, &multiline);
        }

        result
    }

    pub fn get_multiline_trimmed_string_list(&self, file: &KtFile) -> Vec<MultilineTrimmedString> {
        let code = file.text();
        if !may_have_trimmed_strings(&code) {
            return Vec::new();
        }
        let mut collector = Collector { line_index: LineIndex::new(&code), strings: Vec::new() };
        file.accept(&mut collector);
        collector.strings
    }
}

/// False when [MultilineStringFormatter::format] would return `code` unchanged: only a selector text
/// starting with one of these is collected.
pub fn may_have_trimmed_strings(code: &str) -> bool {
    code.contains("trimIndent()") || code.contains("trimMargin()")
}

struct Collector<'c> {
    line_index: LineIndex<'c>,
    strings: Vec<MultilineTrimmedString>,
}

impl KtVisitorVoid for Collector<'_> {
    fn ignores_leaves(&self) -> bool {
        true
    }

    fn visit_element(&mut self, element: &PsiElement) {
        kt_tree_visitor_void::visit_element(self, element);
    }

    fn visit_qualified_expression(&mut self, expression: &KtQualifiedExpression) {
        kt_visitor_void::visit_qualified_expression(self, expression);
        let Some(receiver) = expression.receiver_expression() else { return };
        if !receiver.is::<KtStringTemplateExpression>() {
            return;
        }
        let is_dollar_string = receiver.text_slice().starts_with("$$");
        let selector_text = expression.selector_expression().map(|s| s.text()).unwrap_or_default();
        let selector_expression = selector_text.trim();
        let is_trim_margin = selector_expression.starts_with("trimMargin()");
        let is_trim_indent = selector_expression.starts_with("trimIndent()");
        if is_trim_indent || is_trim_margin {
            // -1 here to account for the space after the dot
            let trim_offset = expression.selector_expression().expect("selector").start_offset() - 1;
            let string_offset = receiver.start_offset();
            let line_start = self.line_index.line_of(string_offset);
            let line_end = self.line_index.line_of(trim_offset);
            let before_string = self.line_index.line_prefix(string_offset);
            let before_tq = before_string.split_once(TQ).map_or(before_string, |(before, _)| before);
            let indent_count = before_tq.chars().count() - trim_start(before_tq).chars().count();
            // Collect comments between the closing """ and the .trimX() call
            let mut comments = Vec::new();
            let selector = expression.selector_expression().map(PsiElement::from);
            let mut child = receiver.next_sibling();
            while let Some(c) = child {
                if Some(&c) == selector.as_ref() {
                    break;
                }
                if c.is::<PsiComment>() {
                    comments.push(c.text());
                }
                child = c.next_sibling();
            }
            self.strings.push(MultilineTrimmedString::new(
                is_trim_margin,
                is_dollar_string,
                indent_count,
                self.line_index.lines[line_start..=line_end].iter().map(|s| s.to_string()).collect(),
                line_start,
                line_end,
                string_offset,
                trim_offset,
                expression.get_parent_of_type::<KtStringTemplateExpression>(false).is_some(),
                comments,
            ));
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultilineTrimmedString {
    /// Whether this is a trimMargin or a trimIndent call.
    pub uses_trim_margin: bool,
    /// Whether this is a dollar string or a simple string.
    pub is_dollar_string: bool,
    /// The number of spaces to indent the string template.
    pub indent_count: usize,
    /// The lines of the string template, including the trimX call.
    pub lines: Vec<String>,
    /// The line number of the first line of the string template, 0-indexed.
    pub line_start: usize,
    /// The line number of the last line of the string template, including the trimX call, 0-indexed.
    pub line_end: usize,
    /// Offset of the opening `"""` or `$$"""`.
    pub open_string_offset: usize,
    /// Offset right before `.trimX`.
    pub trim_method_call_offset: usize,
    /// Whether this multiline string is nested in another multiline string.
    pub is_nested_multiline: bool,
    /// Comments between the closing `"""` and the `.trimX()` call.
    pub comments_between_string_and_trim_call: Vec<String>,
    pub last_string_line_index: usize,
    /// The number of lines in the string template, excluding the trimX call.
    pub string_line_count: usize,
}

impl MultilineTrimmedString {
    #[allow(clippy::too_many_arguments)]
    fn new(
        uses_trim_margin: bool,
        is_dollar_string: bool,
        indent_count: usize,
        lines: Vec<String>,
        line_start: usize,
        line_end: usize,
        open_string_offset: usize,
        trim_method_call_offset: usize,
        is_nested_multiline: bool,
        comments_between_string_and_trim_call: Vec<String>,
    ) -> MultilineTrimmedString {
        // `indexOfLast` is -1 when no line holds `"""`; wrapping makes the count 0 then.
        let last_string_line_index = lines.iter().rposition(|l| l.contains(TQ)).unwrap_or(usize::MAX);
        MultilineTrimmedString {
            uses_trim_margin,
            is_dollar_string,
            indent_count,
            lines,
            line_start,
            line_end,
            open_string_offset,
            trim_method_call_offset,
            is_nested_multiline,
            comments_between_string_and_trim_call,
            last_string_line_index,
            string_line_count: last_string_line_index.wrapping_add(1),
        }
    }

    pub fn uses_trim_indent(&self) -> bool {
        !self.uses_trim_margin
    }

    /// The minimal indent level of the string template, useful to adjust trimIndent.
    pub fn minimal_indent(&self) -> usize {
        let last = &self.lines[self.last_string_line_index];
        let first_after_open = self.lines[0].rsplit_once(TQ).map_or(self.lines[0].as_str(), |(_, after)| after);
        let last_before_close = last.split_once(TQ).map_or(last.as_str(), |(before, _)| before);
        self.lines[1..self.last_string_line_index]
            .iter()
            .map(String::as_str)
            .chain([first_after_open, last_before_close])
            .map(|it| if is_blank(it) { usize::MAX } else { indent_level(it) })
            .min()
            .unwrap()
    }

    pub fn indentation_suffix(&self) -> &'static str {
        if self.uses_trim_margin { "|" } else { "" }
    }

    pub fn has_template_expression(&self) -> bool {
        let dollars = if self.is_dollar_string { 2 } else { 1 };
        self.lines.iter().any(|line| find_template_expression(line, dollars))
    }

    pub fn get_string_content(&self) -> Vec<String> {
        let minimal_indent = OnceCell::new();
        let mut out = Vec::new();
        for (i, line) in self.lines.iter().enumerate() {
            if i == 0 || i == self.last_string_line_index {
                let string_content = if i == 0 {
                    line.split_once(TQ).map_or(line.as_str(), |(_, after)| after)
                } else {
                    line.rsplit_once(TQ).map_or(line.as_str(), |(before, _)| before)
                };
                // Ignores first and last line content if they are blank
                if !is_blank(string_content) {
                    out.push(self.trimmed(string_content, &minimal_indent));
                }
            } else if i > self.last_string_line_index {
                // No longer part of the string template, so we can ignore it
            } else {
                out.push(self.trimmed(line, &minimal_indent));
            }
        }
        out
    }

    /// `minimal_indent` caches [Self::minimal_indent] (upstream recomputes it per line).
    fn trimmed(&self, s: &str, minimal_indent: &OnceCell<usize>) -> String {
        if self.uses_trim_indent() {
            return s.chars().skip(*minimal_indent.get_or_init(|| self.minimal_indent())).collect();
        }

        if trim_start(s).starts_with('|') {
            return s.split_once('|').map_or(s, |(_, after)| after).to_owned();
        }
        s.to_owned()
    }
}

fn indent_level(s: &str) -> usize {
    s.chars().count() - trim_start(s).chars().count()
}

/// `Regex("""\${n}((\{?[A-Za-z_\s])|\{$)""").find(line) != null` for `n` dollars.
fn find_template_expression(line: &str, dollars: usize) -> bool {
    let is_class = |c: Option<char>| c.is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || matches!(c, ' ' | '\t' | '\n' | '\u{0B}' | '\u{0C}' | '\r'));
    let marker = "$".repeat(dollars);
    line.match_indices('$').any(|(i, _)| {
        let Some(rest) = line[i..].strip_prefix(marker.as_str()) else { return false };
        let mut chars = rest.chars();
        let first = chars.next();
        is_class(first) || (first == Some('{') && (is_class(chars.next()) || rest.len() == 1))
    })
}
