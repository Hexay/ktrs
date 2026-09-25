//! Port of `ParagraphListBuilder.kt`, part 4: `adjustParagraphSeparators` through the end of the
//! file (the top-level `containsOnly`, `startsWithUpperCaseLetter`, `isCloseSquareBracket`).

use std::collections::BTreeSet;

use super::kstring::{KChar, KStr, w};
use super::paragraph_list_builder::ParagraphListBuilder;
use super::utilities::{get_indent, is_directive_marker, is_expecting_more, is_header, is_kdoc_tag, is_line, is_todo};

impl<'a> ParagraphListBuilder<'a> {
    pub(super) fn adjust_paragraph_separators(&mut self) {
        let mut prev: Option<usize> = None;

        for index in 0..self.paragraphs.len() {
            let id = self.paragraphs[index];
            self.pm(id).cleanup();
            let paragraph = self.p(id);
            let text = paragraph.text().to_vec();
            let separate = match prev.map(|p| self.p(p)) {
                None => false,
                Some(prev) => {
                    if paragraph.preformatted && prev.preformatted {
                        false
                    } else if paragraph.table {
                        paragraph.separate && (!prev.block || is_kdoc_tag(prev.text()) || prev.table)
                    } else if paragraph.separator || prev.separator {
                        true
                    } else if is_line(&text, 1) || is_line(prev.text(), 1) {
                        false
                    } else if paragraph.separate {
                        true
                    } else if paragraph.doc {
                        // Don't separate kdoc tags, except for the first one
                        !prev.doc
                    } else if is_directive_marker(&text) {
                        false
                    } else if is_todo(&text) && !is_todo(prev.text()) {
                        true
                    } else if is_header(&text) {
                        true
                    } else if paragraph.preformatted {
                        // Set preformatted paragraphs off (but not <pre> tags where it's implicit)
                        !prev.preformatted
                            && !text.starts_with_ic(w!("<pre"))
                            && (!text.trim_start().starts_with(w!("```")) || !is_expecting_more(prev.text()))
                    } else if prev.preformatted && prev.text().starts_with_ic(w!("</pre>")) {
                        false
                    } else if paragraph.continuation {
                        true
                    } else if paragraph.hanging() {
                        false
                    } else if paragraph.quoted > 0 {
                        prev.quoted > 0 && paragraph.quoted == prev.quoted
                    } else if is_header(&text) {
                        true
                    } else if text.starts_with_ic(w!("<p>")) || text.starts_with_ic(w!("<p/>")) {
                        true
                    } else {
                        !paragraph.block && !paragraph.is_empty()
                    }
                }
            };
            self.pm(id).separate = separate;

            let paragraph = self.p(id);
            if paragraph.hanging() {
                if paragraph.doc || text.starts_with_ic(w!("<li>")) || is_todo(&text) {
                    self.pm(id).hanging_indent = get_indent(self.options.hanging_indent);
                } else if paragraph.continuation && paragraph.prev.is_some() {
                    // Walk back through preformatted and empty predecessors to the list item
                    // (e.g. when a code block separates the continuation from its item).
                    let mut source = paragraph.prev.unwrap();
                    while (self.p(source).preformatted || self.p(source).is_empty()) && self.p(source).prev.is_some() {
                        source = self.p(source).prev.unwrap();
                    }
                    let hanging_indent = self.p(source).hanging_indent.clone();
                    let paragraph = self.pm(id);
                    paragraph.hanging_indent = hanging_indent;
                    // Dedent to match hanging indent
                    paragraph.content = paragraph.content.trim_start().to_vec();
                } else {
                    self.pm(id).hanging_indent = get_indent(text.index_of_char(' ' as u16, 0) + 1);
                }
            }
            prev = Some(id);
        }
    }

    pub(super) fn adjust_indentation(&mut self) {
        let first_indent = self.p(self.paragraphs[0]).original_indent;
        if first_indent > 0 {
            for &p in &self.paragraphs {
                let paragraph = &mut self.arena[p];
                if paragraph.original_indent <= first_indent {
                    paragraph.original_indent = 0;
                }
            }
        }

        // Handle nested lists
        let mut in_list = self.paragraphs.first().is_some_and(|&p| self.p(p).hanging());
        let mut start_indent = 0;
        let mut levels: Option<BTreeSet<i32>> = None;
        for i in 1..self.paragraphs.len() {
            let paragraph = &mut self.arena[self.paragraphs[i]];
            if !in_list {
                if paragraph.hanging() {
                    in_list = true;
                    start_indent = paragraph.original_indent;
                }
            } else if !paragraph.hanging() {
                in_list = false;
            } else if paragraph.original_indent == start_indent {
                paragraph.original_indent = 0;
            } else if paragraph.original_indent > 0 {
                levels.get_or_insert_with(BTreeSet::new).insert(paragraph.original_indent);
            }
        }

        if let Some(sorted) = levels {
            let nested = self.options.nested_list_indent();
            let assignments: Vec<(i32, i32)> =
                sorted.iter().enumerate().map(|(i, &level)| (level, (i as i32 + 1) * nested)).collect();
            for &p in &self.paragraphs {
                let paragraph = &mut self.arena[p];
                if paragraph.original_indent > 0 {
                    let Some(&(_, assigned)) =
                        assignments.iter().find(|(level, _)| *level == paragraph.original_indent)
                    else {
                        continue;
                    };
                    paragraph.original_indent = assigned;
                    paragraph.indent = get_indent(paragraph.original_indent);
                }
            }
        }
    }

    pub(super) fn remove_blank_paragraphs(&mut self) {
        // Remove blank lines between list items and from the end as well as around separators
        let mut i = self.paragraphs.len() as i32 - 2;
        while i >= 0 {
            let iu = i as usize;
            let paragraph = self.p(self.paragraphs[iu]);
            if paragraph.is_empty() && (!paragraph.preformatted || iu == self.paragraphs.len() - 1) {
                // Propagate blank-line intent to the next paragraph
                if paragraph.separate && iu + 1 < self.paragraphs.len() {
                    let next = self.paragraphs[iu + 1];
                    self.pm(next).separate = true;
                }
                self.paragraphs.remove(iu);
                if iu > 0 {
                    let before = self.paragraphs[iu - 1];
                    self.pm(before).next = None;
                }
            }
            i -= 1;
        }
    }

    pub(super) fn punctuate(&mut self) {
        if !self.options.add_punctuation || self.paragraphs.is_empty() {
            return;
        }
        let last = self.pm(*self.paragraphs.last().unwrap());
        if last.preformatted || last.doc || (last.hanging() && !last.continuation) || last.is_empty() {
            return;
        }

        let text = &mut last.content;
        if !starts_with_upper_case_letter(text) {
            return;
        }

        for i in (0..text.len()).rev() {
            let c = text[i];
            if c.is_whitespace() {
                continue;
            }
            if c.is_letter_or_digit() || is_close_square_bracket(c) {
                text.truncate(i + 1);
                text.push('.' as u16);
            }
            break;
        }
    }
}

pub fn contains_only(s: &[u16], chars: &[u16]) -> bool {
    s.iter().all(|c| chars.contains(c))
}

pub fn starts_with_upper_case_letter(s: &[u16]) -> bool {
    !s.is_empty() && s[0].is_upper_case() && s[0].is_letter()
}

pub fn is_close_square_bracket(c: u16) -> bool {
    c == ']' as u16
}
