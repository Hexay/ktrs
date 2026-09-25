//! Port of `ParagraphListBuilder.scan`. Upstream's 400-line method is split: the larger inline
//! branches live in `paragraph_list_builder_blocks.rs` as `scan_*` helpers, in branch order.

use super::comment_type::CommentType;
use super::kstring::{KStr, KString, w};
use super::paragraph_list::ParagraphList;
use super::paragraph_list_builder::ParagraphListBuilder;
use super::table::Table;
use super::utilities::{
    collapse_spaces, is_directive_marker, is_kdoc_tag, is_line, is_list_item, is_quoted, is_todo,
};
use super::paragraph_list_builder_adjust::contains_only;

/// The line being scanned: `l`, `lineWithIndentation`, `lineWithoutIndentation` upstream.
pub(super) struct ScanLine {
    pub l: KString,
    pub with_indentation: KString,
    pub without_indentation: KString,
}

impl<'a> ParagraphListBuilder<'a> {
    pub fn scan(mut self, indent_size: i32) -> ParagraphList {
        let mut i = 0usize;
        while i < self.lines.len() {
            let l = self.lines[i].clone();
            i += 1;
            let with_indentation = self.line_content(&l);
            let without_indentation = with_indentation.trim().to_vec();
            let line = ScanLine { l, with_indentation, without_indentation };
            let lwi = &line.with_indentation[..];
            let lwoi = &line.without_indentation[..];

            let prev_indent = self.p(self.paragraph).prev.map(|p| self.p(p).original_indent);
            if lwi.starts_with(w!("    ")) // markdown preformatted text
                && (i == 1 || self.line_content(&self.lines[i - 2]).is_blank()) // i was already ++'ed
                // Make sure it's not just deeply indented inside a different block
                && prev_indent.is_none_or(|prev| (lwi.len() - lwoi.len()) as i32 >= prev + 4)
            {
                i = self.add_preformatted(i - 1, false, false, false, &|_, _, _| {}, &|it| {
                    !it.starts_with(w!(" "))
                });
            } else if lwoi.starts_with(w!("-")) && contains_only(lwoi, w!("-| ")) {
                let paragraph = self.new_paragraph_at(i as i32 - 1, &line);
                self.append_text(lwoi);
                let next = self.new_paragraph_at(i as i32, &line);
                self.pm(next).block = true;
                // Dividers must be surrounded by blank lines
                if is_line(lwi, 3)
                    && (i < 2 || self.line_content(&self.lines[i - 2]).is_blank())
                    && (i > self.lines.len() - 1 || self.line_content(&self.lines[i]).is_blank())
                {
                    self.pm(paragraph).separator = true;
                }
            } else if lwoi.starts_with(w!("=")) && contains_only(lwoi, w!("= ")) {
                // Header
                // ======
                self.block_line(i, &line);
            } else if lwoi.starts_with(w!("#"))
                // "## X" is a header, "##X" is not
                && lwoi.iter().find(|&&c| c != '#' as u16) == Some(&(' ' as u16))
            {
                // not isHeader() because <h> is handled separately
                self.block_line(i, &line);
            } else if lwoi.starts_with(w!("*")) && contains_only(lwoi, w!("* ")) {
                // Horizontal rule (*** or * * *); unlike ---, no surrounding blank lines needed.
                self.block_line(i, &line);
            } else if lwoi.starts_with(w!("```")) {
                i = self.add_preformatted(i - 1, false, true, true, &|_, _, _| {}, &|it| {
                    it.trim_start().starts_with(w!("```"))
                });
            } else if lwoi.starts_with_ic(w!("<pre>")) {
                i = self.scan_pre(i);
            } else if is_quoted(lwoi) {
                i = self.scan_quoted(i, &line);
            } else if lwoi.equals_ic(w!("<ul>")) || lwoi.equals_ic(w!("<ol>")) {
                i = self.scan_html_list(i, &line);
            } else if is_list_item(lwoi)
                || (is_kdoc_tag(lwoi) && self.task.comment_type == CommentType::Kdoc)
                || is_todo(lwoi)
            {
                i = self.scan_list_item(i, &line);
            } else if lwoi.is_empty() {
                let p = self.new_paragraph_at(i as i32, &line);
                self.pm(p).separate = true;
            } else if is_directive_marker(lwoi) {
                self.new_paragraph_at(i as i32 - 1, &line);
                self.append_text(lwoi);
                let p = self.new_paragraph_at(i as i32, &line);
                self.pm(p).block = true;
            } else {
                if lwoi.index_of_char('|' as u16, 0) != -1
                    && self.p(self.paragraph).is_empty()
                    && (i < 2 || !self.lines[i - 2].contains_seq(w!("---")))
                {
                    if let Some(next_row) = self.scan_table(i, indent_size, &line) {
                        i = next_row;
                        continue;
                    }
                }

                // Some common HTML block tags
                if self.scan_html_block(i, &line) {
                    continue;
                }

                i = self.add_plain_text(i, lwoi, 0);
            }
        }

        self.close_paragraph();
        self.arrange();
        if !self.line_comment {
            self.punctuate();
        }

        ParagraphList::new(self.arena, self.paragraphs)
    }

    /// The `newParagraph(i)` local function of `scan`, which records the original indent.
    pub(super) fn new_paragraph_at(&mut self, i: i32, line: &ScanLine) -> usize {
        let paragraph = self.new_paragraph();

        if i >= 0 && (i as usize) < self.lines.len() {
            let i = i as usize;
            let original_indent = if self.lines[i] == line.l {
                (line.with_indentation.len() - line.without_indentation.len()) as i32
            } else {
                // We've looked ahead, e.g. when adding lists etc
                let l = self.line_content(&self.lines[i]);
                (l.len() - l.trim().len()) as i32
            };
            self.pm(paragraph).original_indent = original_indent;
        }
        paragraph
    }

    /// Shared body of the `=`, `#` and `*` branches: the line becomes its own block paragraph.
    fn block_line(&mut self, i: usize, line: &ScanLine) {
        let p = self.new_paragraph_at(i as i32 - 1, line);
        self.pm(p).block = true;
        self.append_text(&line.without_indentation);
        let p = self.new_paragraph_at(i as i32, line);
        self.pm(p).block = true;
    }

    /// Markdown table branch; returns the next row when a table was consumed.
    fn scan_table(&mut self, i: usize, indent_size: i32, line: &ScanLine) -> Option<usize> {
        let (table, next_row) = Table::get_table(&self.lines, i - 1, &|l| self.line_content(l))?;
        let content = if self.options.align_table_columns {
            // Only maxLineWidth, not maxCommentWidth: table lines can't be broken, only padded.
            table.format(self.options.max_line_width - indent_size - 3)
        } else {
            table.original().to_vec()
        };
        for (index, table_line) in content.iter().enumerate() {
            self.append_text(table_line);
            let paragraph = self.cur();
            paragraph.separate = index == 0;
            paragraph.block = true;
            paragraph.table = true;
            self.new_paragraph_at(-1, line);
        }
        self.new_paragraph_at(next_row as i32, line);
        Some(next_row)
    }

    /// HTML block tag branch; returns true when upstream `continue`s the scan loop.
    fn scan_html_block(&mut self, i: usize, line: &ScanLine) -> bool {
        let lwoi = &line.without_indentation[..];
        let block_tags = [
            w!("<p>"), w!("<p/>"), w!("<h1"), w!("<h2"), w!("<h3"), w!("<h4"), w!("<table"),
            w!("<tr"), w!("<caption"), w!("<td"), w!("<div"),
        ];
        if !(lwoi.starts_with(w!("<")) && block_tags.iter().any(|t| lwoi.starts_with_ic(t))) {
            return false;
        }
        let p = self.new_paragraph_at(i as i32 - 1, line);
        self.pm(p).block = true;
        if lwoi.equals_ic(w!("<p>"))
            || lwoi.equals_ic(w!("<p/>"))
            || (self.options.convert_markup && lwoi.equals_ic(w!("</p>")))
        {
            if self.options.convert_markup {
                // Replace <p> with a blank line
                self.cur().separate = true;
            } else {
                self.append_text(lwoi);
                let p = self.new_paragraph_at(i as i32, line);
                self.pm(p).block = true;
            }
            return true;
        } else if [w!("</h1>"), w!("</h2>"), w!("</h3>"), w!("</h4>")].iter().any(|t| lwoi.ends_with_ic(t)) {
            if lwoi.starts_with_ic(w!("<h")) && self.options.convert_markup && self.cur().is_empty() {
                self.cur().separate = true;
                let count = lwoi[lwoi.len() - 2] as i32 - '0' as i32;
                for _ in 0..count.clamp(0, 8) {
                    self.append_text(w!("#"));
                }
                self.append_text(w!(" "));
                self.append_text(&lwoi[4..lwoi.len() - 5]);
            } else if self.options.collapse_spaces {
                self.append_text(&collapse_spaces(lwoi));
            } else {
                self.append_text(lwoi);
            }
            let p = self.new_paragraph_at(i as i32, line);
            self.pm(p).block = true;
            return true;
        }
        false
    }
}
