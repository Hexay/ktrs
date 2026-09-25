//! Inline branches of `ParagraphListBuilder.scan` (see `paragraph_list_builder_scan.rs`), extracted
//! only to keep files small; bodies follow upstream line by line.

use super::kstring::{KChar, KStr, KString, w};
use super::paragraph_list_builder::ParagraphListBuilder;
use super::paragraph_list_builder_scan::ScanLine;
use super::utilities::{
    collapse_spaces, is_directive_marker, is_header, is_kdoc_tag, is_line, is_list_item, is_quoted,
    is_todo,
};

impl<'a> ParagraphListBuilder<'a> {
    /// `<pre>` branch.
    pub(super) fn scan_pre(&mut self, i: usize) -> usize {
        self.add_preformatted(
            i - 1,
            true,
            true,
            true,
            &|b, _, _| {
                if b.options.convert_markup {
                    b.handle_tag(w!("<pre>"));
                    b.handle_tag(w!("</pre>"));
                }
            },
            &|it| it.index_of_ic(w!("</pre>")) != -1,
        )
    }

    /// The `handleTag` local function of the `<pre>` customizer.
    fn handle_tag(&mut self, tag: &[u16]) {
        let text = self.cur().content.clone();
        let trimmed = text.trim();

        let index = text.index_of_ic(tag);
        if index == -1 {
            return;
        }
        let index = index as usize;
        self.cur().content.clear();
        if trimmed.equals_ic(tag) {
            self.cur().content.extend_from_slice(w!("```"));
            return;
        }

        // Split paragraphs; these things have to be on their own line in the ``` form
        let before = text[..index].replace_seq(w!("</code>"), w!(""), true);
        let before = before.trim();
        if !before.is_blank() {
            self.cur().content.extend_from_slice(before);
            self.new_paragraph();
            self.cur().preformatted = true;
            self.cur().allow_empty = true;
        }
        self.append_text(w!("```"));
        let after = text[index + tag.len()..].replace_seq(w!("<code>"), w!(""), true);
        let after = after.trim();
        if !after.is_blank() {
            self.new_paragraph();
            self.append_text(after);
            self.cur().preformatted = true;
            self.cur().allow_empty = true;
        }
    }

    /// Quoted (`> `) branch: processed line by line to handle list items and nested quotes.
    pub(super) fn scan_quoted(&mut self, i: usize, line: &ScanLine) -> usize {
        let had_blank_line = self.cur().is_empty() && self.cur().separate;
        let mut i = i - 1;

        let mut prev_depth = 0;
        let mut is_first = true;

        while i < self.lines.len() {
            let q_line_content = self.line_content(&self.lines[i]);
            let q_trimmed = q_line_content.trim();

            if q_trimmed.is_blank()
                || is_kdoc_tag(q_trimmed)
                || is_todo(q_trimmed)
                || is_directive_marker(q_trimmed)
                || is_header(q_trimmed)
            {
                break;
            }

            // Non-quoted lines that are list items should end the quoted block
            if !is_quoted(q_trimmed) && is_list_item(q_trimmed) {
                break;
            }

            let (depth, inner): (i32, &[u16]) = if is_quoted(q_trimmed) {
                let mut d = 0;
                let mut s = q_trimmed;
                while s.starts_with(w!("> ")) {
                    d += 1;
                    s = s[2..].trim_start();
                }
                (d, s)
            } else {
                // Continuation line without `> ` prefix - inherits previous depth
                (prev_depth, q_trimmed)
            };

            let needs_break = !is_first && (depth != prev_depth || is_list_item(inner));

            if is_first || needs_break {
                let p = self.new_paragraph_at(i as i32, line);
                let p = self.pm(p);
                if is_first && had_blank_line {
                    p.separate = true;
                }
                p.quoted = depth;
                p.block = false;
            }

            self.append_text(&collapse_spaces(inner));
            self.append_text(w!(" "));

            prev_depth = depth;
            is_first = false;
            i += 1;
        }

        self.new_paragraph_at(i as i32, line);
        i
    }

    /// `<ul>` / `<ol>` branch.
    pub(super) fn scan_html_list(&mut self, i: usize, line: &ScanLine) -> usize {
        let p = self.new_paragraph_at(i as i32 - 1, line);
        self.pm(p).block = true;
        self.append_text(&line.without_indentation);
        let p = self.new_paragraph_at(i as i32, line);
        self.pm(p).set_hanging(true);
        let i = self.add_lines(
            i,
            true,
            &|_, _, w, _| w.equals_ic(w!("</ul>")) || w.equals_ic(w!("</ol>")),
            &|b, _, p| b.pm(p).block = true,
            &|w, _| {
                w.starts_with_ic(w!("<li>")) || w.starts_with_ic(w!("</ul>")) || w.starts_with_ic(w!("</ol>"))
            },
            w!(" "),
        );
        self.new_paragraph_at(i as i32, line);
        i
    }

    /// List item, KDoc tag and TODO branch.
    pub(super) fn scan_list_item(&mut self, i: usize, line: &ScanLine) -> usize {
        let had_blank_line = self.cur().is_empty()
            && self.cur().separate
            && is_list_item(&line.without_indentation);
        let mut i = i - 1;
        let p = self.new_paragraph_at(i as i32, line);
        self.pm(p).set_hanging(true);
        if had_blank_line {
            self.cur().separate = true;
        }
        let start = i;
        let list_item_until = |b: &ParagraphListBuilder<'a>, j: usize, w: &[u16], s: &[u16]| {
            let lines = &b.lines;
            // See if it's a line continuation
            if s.is_blank() && j + 1 < lines.len() && b.line_content(&lines[j + 1]).starts_with(w!(" ")) {
                false
            } else {
                s.is_blank()
                    || is_list_item(w)
                    || is_quoted(w)
                    || is_kdoc_tag(w)
                    || is_todo(w)
                    || w.starts_with(w!("```"))
                    || w.starts_with(w!("<pre>"))
                    || is_directive_marker(w)
                    || is_line(w, 3)
                    || is_header(w)
                    // Not indented by at least two spaces following a blank line?
                    || (s.len() > 2
                        && (!s[0].is_whitespace() || !s[1].is_whitespace())
                        && j + 1 < lines.len()
                        && b.line_content(&lines[j - 1]).is_blank())
            }
        };
        let list_item_should_break = |w: &[u16], _: &[u16]| w.is_blank();
        let list_item_customize = |b: &mut ParagraphListBuilder<'a>, j: usize, p: usize| {
            if b.line_content(&b.lines[j]).is_blank() && j >= start {
                let p = b.pm(p);
                p.set_hanging(true);
                p.continuation = true;
            }
        };
        i = self.add_lines(i, false, &list_item_until, &list_item_customize, &list_item_should_break, w!(" "));
        // Handle fenced code blocks within list items
        while i < self.lines.len() && self.line_content(&self.lines[i]).trim().starts_with(w!("```")) {
            i = self.add_preformatted(i, false, true, true, &|_, _, _| {}, &|it| {
                it.trim_start().starts_with(w!("```"))
            });
            // Check if list item continues after the code block
            if i >= self.lines.len() {
                break;
            }
            let next_line: KString = self.line_content(&self.lines[i]);
            let next_trimmed = next_line.trim();
            // Continue on indented text, or a blank line followed by indented text.
            let is_continuation = if next_trimmed.is_blank() {
                i + 1 < self.lines.len() && self.line_content(&self.lines[i + 1]).starts_with(w!(" "))
            } else {
                next_line.starts_with(w!(" "))
                    && !is_list_item(next_trimmed)
                    && !is_quoted(next_trimmed)
                    && !is_kdoc_tag(next_trimmed)
                    && !is_todo(next_trimmed)
                    && !is_directive_marker(next_trimmed)
                    && !is_line(next_trimmed, 3)
                    && !is_header(next_trimmed)
            };
            if !is_continuation {
                break;
            }
            // Set up continuation paragraph and continue processing
            let p = self.new_paragraph_at(i as i32, line);
            let p = self.pm(p);
            p.set_hanging(true);
            p.continuation = true;
            i = self.add_lines(i, false, &list_item_until, &list_item_customize, &list_item_should_break, w!(" "));
        }
        self.new_paragraph_at(i as i32, line);
        i
    }
}
