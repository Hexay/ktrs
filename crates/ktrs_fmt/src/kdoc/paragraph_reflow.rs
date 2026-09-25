//! Port of `Paragraph.kt`, part 2: `reflow` through `computeWords` (the breaking algorithms are
//! in `paragraph_breaking.rs`).

use super::comment_type::CommentType;
use super::kstring::{KChar, KStr, KString, w};
use super::paragraph::Paragraph;
use super::utilities::{
    collapse_spaces, get_indent_size, is_directive_marker, is_kdoc_tag, is_list_item, is_quoted,
    is_todo, max_of,
};

/// Java regex `\s`: `[ \t\n\x0B\f\r]`.
fn is_regex_space(c: u16) -> bool {
    matches!(c, 0x20 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D)
}

fn len(s: &[u16]) -> i32 {
    s.len() as i32
}

impl Paragraph {
    pub fn reflow(&self, first_line_max_width: i32, max_line_width: i32) -> Vec<KString> {
        let line_width = max_line_width - get_indent_size(&self.indent, &self.options);
        let hanging_indent_size = get_indent_size(&self.hanging_indent, &self.options)
            - if self.quoted > 0 { 2 * self.quoted } else { 0 };
        if len(self.text()) < first_line_max_width - hanging_indent_size {
            return vec![collapse_spaces(self.text())];
        }
        // Split text into words
        let words = self.compute_words();

        // See divide & conquer algorithm listed here: https://xxyxyz.org/line-breaking/
        if words.len() == 1 {
            return vec![words[0].clone()];
        }

        if first_line_max_width < max_line_width {
            // Ragged text: greedily fill the first line, then optimize the rest.
            let mut line = KString::new();
            let first_line_width = first_line_max_width - get_indent_size(&self.indent, &self.options);
            for i in 0..words.len() {
                let word = &words[i];
                if line.is_empty() {
                    if len(word) + self.comment_type.line_overhead() > first_line_max_width {
                        // Nothing fits on the first line: flow to full width; the caller moves the
                        // comment to the next line.
                        return self.reflow_words(&words, line_width, hanging_indent_size);
                    }
                    line.extend_from_slice(word);
                } else if len(&line) + len(word) + 1 <= first_line_width {
                    line.push(' ' as u16);
                    line.extend_from_slice(word);
                } else {
                    let remaining_words = &words[i..];
                    let reflown_remaining =
                        self.reflow_words(remaining_words, line_width, hanging_indent_size);
                    let mut result = vec![line];
                    result.extend(reflown_remaining);
                    return result;
                }
            }
            // We fit everything on the first line
            return vec![line];
        }

        self.reflow_words(&words, line_width, hanging_indent_size)
    }

    fn reflow_words(&self, words: &[KString], line_width: i32, hanging_indent_size: i32) -> Vec<KString> {
        let options = &self.options;
        if options.alternate
            || !options.optimal
            || (self.hanging() && hanging_indent_size > 0)
            // An unbreakable long word may make other lines shorter and won't look good
            || words.iter().any(|it| len(it) > line_width)
        {
            // Greedy when requested, and for hanging indents (list items, kdoc sections), where
            // optimal breaking has no separate first-line width and rarely helps short text.
            return self.reflow_greedy(line_width, options, words);
        }

        let lines = self.reflow_optimal(line_width - hanging_indent_size, words);
        if lines.len() <= 2 {
            // Just 2 lines? We prefer long+short instead of half+half.
            self.reflow_greedy(line_width, options, words)
        } else {
            // The plain algorithm over-corrects short last lines by shortening everything else.
            let max_line = |it: &KString| {
                // Ignore lines that are unbreakable
                if it.index_of_char(' ' as u16, 0) == -1 { 0 } else { len(it) }
            };
            let longest_line = max_of(&lines, max_line);
            let mut last_word = words.len() as i32 - 1;
            while last_word > 0 {
                // Cheap enough: only repeated for a single line's worth of words.
                let new_lines =
                    self.reflow_optimal(line_width - hanging_indent_size, &words[..last_word as usize]);
                if new_lines.len() < lines.len() {
                    let new_longest_line = max_of(&new_lines, max_line);
                    if new_longest_line > longest_line
                        && new_lines[..new_lines.len() - 1].iter().any(|it| len(it) > longest_line)
                    {
                        let mut result = new_lines;
                        result.extend(self.reflow_greedy(
                            line_width - hanging_indent_size,
                            options,
                            &words[last_word as usize..],
                        ));
                        return result;
                    }
                    break;
                }
                last_word -= 1;
            }

            lines
        }
    }

    /// Returns true if [word] can start a line without changing its meaning (e.g. becoming a list
    /// item, header or tag).
    fn can_break_at(&self, prev: &[u16], word: &[u16]) -> bool {
        if word.starts_with(w!("#"))
            || word.starts_with(w!("```"))
            || is_directive_marker(word)
            || word.starts_with(w!("@")) // interpreted as a tag
            || is_todo(word)
            || word.starts_with(w!(">"))
        {
            return false;
        }

        if prev == w!("@sample") {
            return false; // https://github.com/facebook/ktfmt/issues/310
        }

        if !word[0].is_letter() {
            let word_with_space = [word, w!(" ")].concat(); // for regex matching in below checks
            if (is_list_item(&word_with_space) && !word.equals_ic(w!("<li>")))
                || is_quoted(&word_with_space)
            {
                return false;
            }
        }

        true
    }

    /// Split [Self::text] into words, joining words that must not start a line with their
    /// predecessor (e.g. "the sum is 5." gives "the", "sum", "is 5." so "5." never starts a line).
    pub fn compute_words(&self) -> Vec<KString> {
        let words: Vec<KString> = self
            .text()
            .split(|&c| is_regex_space(c))
            .filter(|it| !it.is_blank())
            .map(|it| it.trim().to_vec())
            .collect();
        if words.len() == 1 {
            return words;
        }

        if self.comment_type != CommentType::Kdoc {
            // Block and line comments assign no special meaning to line-leading words.
            return words;
        }

        let mut combined: Vec<KString> = Vec::with_capacity(words.len());

        let mut from = 0;
        let end = words.len();
        while from < end {
            let start = if from == 0
                && (self.quoted > 0 || (self.hanging() && !is_kdoc_tag(self.text())))
            {
                from + 2
            } else {
                from + 1
            };
            let mut to = words.len();
            for i in start..words.len() {
                let next = &words[i];
                if next.starts_with(w!("[")) && !next.starts_with(w!("[[")) {
                    // find end
                    let j = (i..words.len()).find(|&k| words[k].contains(&(']' as u16)));
                    if let Some(j) = j {
                        // combine everything in the string; we can't break link text or @sample tags
                        if start == from + 1 && self.can_break_at(&words[start - 1], &words[start]) {
                            combined.push(words[from].clone());
                            from = start;
                        }
                        // Maybe not break; what if the next word isn't okay?
                        to = j + 1;
                        if to == words.len() || self.can_break_at(&words[to - 1], &words[to]) {
                            break;
                        }
                    } // else: unterminated [, ignore
                } else if self.can_break_at(&words[i - 1], next) {
                    to = i;
                    break;
                }
            }

            if to == from + 1 {
                combined.push(words[from].clone());
            } else if to > from {
                combined.push(words[from..to].join(&(' ' as u16)));
            }
            from = to;
        }

        combined
    }
}
