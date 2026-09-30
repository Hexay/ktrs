//! Port of `ParagraphListBuilder.kt`, part 1: state and line accumulation. `scan` is in
//! `paragraph_list_builder_scan.rs`; `convertPrefix` onwards in `paragraph_list_builder_arrange.rs`.

use super::comment_type::{is_kdoc_comment, is_line_comment};
use super::formatting_task::FormattingTask;
use super::kstring::{KStr, KString, ks, w};
use super::options::KDocFormattingOptions;
use super::paragraph::Paragraph;
use super::utilities::{
    collapse_spaces, is_directive_marker, is_kdoc_tag, is_list_item, is_quoted, is_todo,
};

/// `(j, lineWithoutIndentation, lineWithIndentation) -> Boolean`
pub(super) type Until<'f, 'a> = &'f dyn Fn(&ParagraphListBuilder<'a>, usize, &[u16], &[u16]) -> bool;
/// `(j, paragraph) -> Unit`; the paragraph is an arena index.
pub(super) type Customize<'f, 'a> = &'f dyn Fn(&mut ParagraphListBuilder<'a>, usize, usize);

pub struct ParagraphListBuilder<'a> {
    pub(super) options: &'a KDocFormattingOptions,
    pub(super) task: &'a FormattingTask,
    pub(super) line_comment: bool,
    /// Every paragraph ever created; `paragraphs` and `paragraph` index into it.
    pub(super) arena: Vec<Paragraph>,
    pub(super) paragraphs: Vec<usize>,
    pub(super) lines: Vec<KString>,
    pub(super) paragraph: usize,
}

impl<'a> ParagraphListBuilder<'a> {
    pub fn new(comment: &str, options: &'a KDocFormattingOptions, task: &'a FormattingTask) -> Self {
        let line_comment = is_line_comment(comment);
        let comment_prefix: &'static [u16] = if line_comment {
            w!("//")
        } else if is_kdoc_comment(comment) {
            w!("/**")
        } else {
            w!("/*")
        };
        let comment = ks(comment);
        let lines = if line_comment {
            comment.split_on('\n' as u16).iter().map(|it| it.trim_start().to_vec()).collect()
        } else if !comment.contains(&('\n' as u16)) {
            let body = comment.remove_prefix(comment_prefix).remove_suffix(w!("*/")).trim();
            vec![[w!("* "), body].concat()]
        } else {
            comment.remove_prefix(comment_prefix).remove_suffix(w!("*/")).trim().split_on('\n' as u16)
        };
        ParagraphListBuilder {
            options,
            task,
            line_comment,
            arena: vec![Paragraph::new(task)],
            paragraphs: Vec::new(),
            lines,
            paragraph: 0,
        }
    }

    pub(super) fn p(&self, id: usize) -> &Paragraph {
        &self.arena[id]
    }

    pub(super) fn pm(&mut self, id: usize) -> &mut Paragraph {
        &mut self.arena[id]
    }

    /// The paragraph currently being built (`this.paragraph` upstream).
    pub(super) fn cur(&mut self) -> &mut Paragraph {
        let id = self.paragraph;
        &mut self.arena[id]
    }

    pub(super) fn line_content(&self, line: &[u16]) -> KString {
        let trimmed = line.trim();
        if self.line_comment && trimmed.starts_with(w!("// ")) {
            trimmed[3..].to_vec()
        } else if self.line_comment && trimmed.starts_with(w!("//")) {
            trimmed[2..].to_vec()
        } else if trimmed.starts_with(w!("* ")) {
            trimmed[2..].to_vec()
        } else if trimmed.starts_with(w!("*")) {
            trimmed[1..].to_vec()
        } else {
            trimmed.to_vec()
        }
    }

    pub(super) fn close_paragraph(&mut self) -> usize {
        let text = self.cur().content.clone();
        let paragraph = self.cur();
        if paragraph.preformatted {
        } else if is_kdoc_tag(&text) {
            paragraph.doc = true;
            paragraph.set_hanging(true);
        } else if is_todo(&text) {
            paragraph.set_hanging(true);
        } else if is_list_item(&text) {
            paragraph.set_hanging(true);
        } else if is_directive_marker(&text) {
            paragraph.block = true;
            paragraph.preformatted = true;
        }
        if !paragraph.is_empty() || paragraph.allow_empty {
            self.paragraphs.push(self.paragraph);
        }
        self.paragraph
    }

    pub(super) fn new_paragraph(&mut self) -> usize {
        self.close_paragraph();
        let prev = self.paragraph;
        self.arena.push(Paragraph::new(self.task));
        self.paragraph = self.arena.len() - 1;
        let paragraph = self.paragraph;
        self.pm(prev).next = Some(paragraph);
        self.pm(paragraph).prev = Some(prev);
        paragraph
    }

    pub(super) fn append_text(&mut self, s: &[u16]) -> &mut Self {
        self.cur().content.extend_from_slice(s);
        self
    }

    /// Upstream defaults: `include_end = true`, `until = true`, no-op `customize`,
    /// `should_break = false`, `separator = " "`.
    pub(super) fn add_lines(
        &mut self,
        i: usize,
        include_end: bool,
        until: Until<'_, 'a>,
        customize: Customize<'_, 'a>,
        should_break: &dyn Fn(&[u16], &[u16]) -> bool,
        separator: &[u16],
    ) -> usize {
        let mut j = i;
        while j < self.lines.len() {
            let l = self.lines[j].clone();
            let line_with_indentation = self.line_content(&l);
            let line_without_indentation = line_with_indentation.trim();

            if !include_end && j > i && until(self, j, line_without_indentation, &line_with_indentation) {
                self.strip_trailing_blank_lines();
                return j;
            }

            if should_break(line_without_indentation, &line_with_indentation) {
                self.new_paragraph();
            }

            if is_quoted(&line_with_indentation) {
                self.append_text(&collapse_spaces(&line_without_indentation[2..]));
            } else {
                self.append_text(&collapse_spaces(line_without_indentation));
            }
            self.append_text(separator);
            let paragraph = self.paragraph;
            customize(self, j, paragraph);
            if include_end && j > i && until(self, j, line_without_indentation, &line_with_indentation) {
                self.strip_trailing_blank_lines();
                return j + 1;
            }

            j += 1;
        }

        self.strip_trailing_blank_lines();
        self.new_paragraph();

        j
    }

    /// Upstream defaults: `include_start = false`, `include_end = true`, `expect_close = false`,
    /// no-op `customize`, `until = true`.
    pub(super) fn add_preformatted(
        &mut self,
        i: usize,
        include_start: bool,
        include_end: bool,
        expect_close: bool,
        customize: Customize<'_, 'a>,
        until: &dyn Fn(&[u16]) -> bool,
    ) -> usize {
        self.new_paragraph();
        let mut j = i;
        let mut found_close = false;
        let mut allow_customize = true;
        while j < self.lines.len() {
            let line_with_indentation = self.line_content(&self.lines[j]);
            if line_with_indentation.contains_seq(w!("```"))
                && line_with_indentation.trim_start().starts_with(w!("```"))
            {
                // Don't convert <pre> tags if we already have nested ``` content
                allow_customize = false;
            }
            let done = (include_start || j > i) && until(&line_with_indentation);
            if !include_end && done {
                found_close = true;
                break;
            }
            j += 1;
            if include_end && done {
                found_close = true;
                break;
            }
        }

        // Unterminated block (likely a doc error): treat just one line as preformatted and the
        // rest normally, rather than preformatting everything that follows.
        if !found_close && expect_close {
            allow_customize = false;
            j = self.lines.len();
        }

        for index in i..j {
            let line_with_indentation = self.line_content(&self.lines[index]);
            self.append_text(&line_with_indentation);
            self.cur().preformatted = true;
            self.cur().allow_empty = true;
            if allow_customize {
                let paragraph = self.paragraph;
                customize(self, index, paragraph);
            }
            self.new_paragraph();
        }
        self.strip_trailing_blank_lines();
        self.new_paragraph();

        j
    }

    pub(super) fn strip_trailing_blank_lines(&mut self) {
        while let Some(&last) = self.paragraphs.last() {
            if !self.p(last).is_empty() {
                break;
            }
            self.paragraphs.pop();
        }
    }
}
