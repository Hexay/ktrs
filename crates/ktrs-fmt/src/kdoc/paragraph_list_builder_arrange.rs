//! Port of `ParagraphListBuilder.kt`, part 3: `convertPrefix` through `sortDocTags` (the rest is in
//! `paragraph_list_builder_adjust.rs`).

use std::cmp::Ordering;

use super::kstring::{KStr, KString, ks, w};
use super::paragraph_list_builder::ParagraphListBuilder;
use super::utilities::{collapse_spaces, get_param_name, is_todo};

impl<'a> ParagraphListBuilder<'a> {
    fn convert_prefix(&mut self, text: &[u16]) -> KString {
        if self.options.convert_markup && (text.starts_with_ic(w!("<p>")) || text.starts_with_ic(w!("<p/>"))) {
            self.cur().separate = true;
            let gt = text.index_of_char('>' as u16, 0);
            text[(gt + 1) as usize..].trim().to_vec()
        } else {
            text.to_vec()
        }
    }

    fn convert_suffix(&self, trimmed_prefix: &[u16]) -> KString {
        if self.options.convert_markup
            && (trimmed_prefix.ends_with_ic(w!("<p/>")) || trimmed_prefix.ends_with_ic(w!("</p>")))
        {
            trimmed_prefix[..trimmed_prefix.len() - 4].trim_end().remove_suffix(w!("*")).trim_end().to_vec()
        } else {
            trimmed_prefix.to_vec()
        }
    }

    /// Upstream default: `brace_balance = 0`.
    pub(super) fn add_plain_text(&mut self, i: usize, text: &[u16], brace_balance: i32) -> usize {
        let prefix = self.convert_prefix(text);
        let trimmed = self.convert_suffix(&prefix);
        let s = if self.options.collapse_spaces { collapse_spaces(&trimmed) } else { trimmed };
        self.append_text(&s);
        self.append_text(w!(" "));

        if brace_balance > 0 {
            let end = s.index_of_char('}' as u16, 0);
            if end == -1 && i < self.lines.len() {
                let next = self.line_content(&self.lines[i]).trim().to_vec();
                if self.break_out_of_tag(&next) {
                    return i;
                }
                return self.add_plain_text(i + 1, &next, 1);
            }
        }

        let index = s.index_of(w!("{@"), 0);
        if index != -1 {
            // find end
            let end = s.index_of_char('}' as u16, index as usize);
            if end == -1 && i < self.lines.len() {
                let next = self.line_content(&self.lines[i]).trim().to_vec();
                if self.break_out_of_tag(&next) {
                    return i;
                }
                return self.add_plain_text(i + 1, &next, 1);
            }
        }

        i
    }

    fn break_out_of_tag(&self, next: &[u16]) -> bool {
        // Blank lines or ``` inside an open {@ tag: give up treating it as paragraph text.
        // See https://github.com/tnorbye/kdoc-formatter/issues/77
        next.is_blank() || next.starts_with(w!("```"))
    }

    fn doc_tag_rank(&self, tag: &[u16], is_priority: bool) -> i32 {
        // Canonical kdoc order -- https://kotlinlang.org/docs/kotlin-doc.html#block-tags
        // (@param and @property share a rank; they are sorted by parameter order.)
        let ranks: [(&[u16], i32); 13] = [
            (w!("@param"), 0),
            (w!("@property"), 0),
            (w!("@return"), 1),
            (w!("@constructor"), 2),
            (w!("@receiver"), 3),
            (w!("@throws"), 4),
            (w!("@exception"), 5),
            (w!("@sample"), 6),
            (w!("@see"), 7),
            (w!("@author"), 8),
            (w!("@since"), 9),
            (w!("@suppress"), 10),
            (w!("@deprecated"), 11),
        ];
        if is_priority {
            return -1;
        }
        ranks.iter().find(|(t, _)| tag.starts_with(t)).map_or(100, |&(_, r)| r) // 100: custom tags
    }

    /// Tags that are "priority" are placed before other tags, with their order unchanged, unless
    /// they come after a regular tag. See: https://github.com/facebook/ktfmt/issues/406
    fn doc_tag_is_priority(&self, tag: &[u16]) -> bool {
        tag.starts_with(w!("@sample"))
    }

    /// Make a pass over the paragraphs and make sure that we (for example) place blank lines around
    /// preformatted text.
    pub(super) fn arrange(&mut self) {
        if self.paragraphs.is_empty() {
            return;
        }

        self.sort_doc_tags();
        self.adjust_paragraph_separators();
        self.adjust_indentation();
        self.remove_blank_paragraphs();
        self.strip_trailing_blank_lines();
    }

    fn sort_doc_tags(&mut self) {
        if !(self.options.order_doc_tags && self.paragraphs.iter().any(|&p| self.p(p).doc)) {
            return;
        }
        // order[p] = position of arena paragraph p in `paragraphs`.
        let mut order = vec![usize::MAX; self.arena.len()];
        for (index, &p) in self.paragraphs.iter().enumerate() {
            order[p] = index;
        }
        let first_non_priority_doc_tag = self
            .paragraphs
            .iter()
            .position(|&p| self.p(p).doc && !self.doc_tag_is_priority(self.p(p).text()))
            .map_or(-1, |i| i as i64);
        let ordered_parameter_names: Vec<KString> =
            self.task.ordered_parameter_names.iter().map(|n| ks(n)).collect();
        let parameter_rank = |p: usize| -> i32 {
            if let Some(name) = get_param_name(self.p(p).text()) {
                if let Some(index) = ordered_parameter_names.iter().position(|n| n == name) {
                    return index as i32;
                }
            }
            1000
        };
        let compare = |l1: &Vec<usize>, l2: &Vec<usize>| -> Ordering {
            let (p1, p2) = (l1[0], l2[0]);
            let (a, b) = (self.p(p1), self.p(p2));
            let (o1, o2) = (order[p1], order[p2]);
            let is_priority1 = a.doc && self.doc_tag_is_priority(a.text()) && (o1 as i64) < first_non_priority_doc_tag;
            let is_priority2 = b.doc && self.doc_tag_is_priority(b.text()) && (o2 as i64) < first_non_priority_doc_tag;

            // Sort TODOs to the end
            if is_todo(a.text()) != is_todo(b.text()) {
                return if is_todo(a.text()) { Ordering::Greater } else { Ordering::Less };
            }

            if a.doc == b.doc {
                if a.doc {
                    // Sort @return after @param etc
                    let r1 = self.doc_tag_rank(a.text(), is_priority1);
                    let r2 = self.doc_tag_rank(b.text(), is_priority2);
                    if r1 != r2 {
                        return r1.cmp(&r2);
                    }
                    // Identical tags keep their order, except params follow signature order.
                    if !ordered_parameter_names.is_empty() {
                        let i1 = parameter_rank(p1);
                        let i2 = parameter_rank(p2);
                        // If the parameter names are not matching, ignore.
                        if i1 != i2 {
                            return i1.cmp(&i2);
                        }
                    }
                }
                return o1.cmp(&o2);
            }
            if a.doc { Ordering::Greater } else { Ordering::Less }
        };

        // Sort units instead of paragraphs: a KDoc tag carries all following paragraphs (until the
        // next tag) with it. Units are ordered by their first paragraph. The comparator is a total
        // order, so Rust's stable sort matches Java's TimSort.
        let mut units: Vec<Vec<usize>> = Vec::new();
        let mut tag: Option<usize> = None; // index into `units`
        for &paragraph in &self.paragraphs {
            if self.p(paragraph).doc {
                units.push(Vec::new());
                tag = Some(units.len() - 1);
            }
            match tag {
                Some(t) if !is_todo(self.p(paragraph).text()) => units[t].push(paragraph),
                _ => units.push(vec![paragraph]),
            }
        }
        units.sort_by(compare);

        let mut prev: Option<usize> = None;
        self.paragraphs.clear();
        for paragraph in units.into_iter().flatten() {
            self.paragraphs.push(paragraph);
            if let Some(prev) = prev {
                self.pm(prev).next = Some(paragraph);
            }
            self.pm(paragraph).prev = prev;
            prev = Some(paragraph);
        }
    }
}
