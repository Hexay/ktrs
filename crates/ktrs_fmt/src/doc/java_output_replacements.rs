//! `JavaOutput.getFormatReplacements` and friends: turning the formatted lines into input edits.

use super::java_output::JavaOutput;
use super::newlines;
use super::range::{EMPTY_RANGE, Range, RangeSet};
use super::replacement::Replacement;

impl JavaOutput<'_> {
    /// Replacements converting input to output, sorted by start index, without overlaps.
    pub fn get_format_replacements(&self, i_range_set0: &RangeSet) -> Vec<Replacement> {
        let mut result = Vec::new();
        let k_to_j = super::input_output::InputOutput::make_k_to_ij(&self.io);
        let text = self.java_input.get_text();

        // Expand the token ranges to align with re-formattable boundaries.
        let mut breakable_ranges = RangeSet::create();
        let i_range_set = i_range_set0.sub_range_set_closed(0, self.java_input.get_kn());
        for i_range in i_range_set.as_canonical_ranges() {
            let range = self.expand_to_breakable_regions(i_range);
            if range == EMPTY_RANGE {
                // the range contains only whitespace
                continue;
            }
            breakable_ranges.add(range);
        }

        // Construct replacements for each reformatted region.
        for range in breakable_ranges.as_canonical_ranges() {
            let start_token = self
                .java_input
                .get_token(range.lower_endpoint())
                .expect("no token for start index");
            let end_token = self
                .java_input
                .get_token(range.upper_endpoint() - 1)
                .expect("no token for end index");
            let start_tok = Self::start_tok(&**start_token);
            let end_tok = Self::end_tok(&**end_token);

            // Add all output lines in the given token range to the replacement.
            let mut replacement = String::new();

            let mut replace_from = start_tok.get_position() as usize;
            // Replace leading whitespace in the input with the whitespace from the formatted file
            while replace_from > 0 {
                let previous = text[..replace_from].chars().next_back().unwrap();
                if !previous.is_whitespace() {
                    break;
                }
                replace_from -= previous.len_utf8();
            }

            let k_range = |k: i32| k_to_j[k as usize].expect("no output range for tok");
            let mut i = k_range(start_tok.get_index()).lower_endpoint();
            // Include leading blank lines from the formatted output, unless the formatted range
            // starts at the beginning of the file.
            while i > 0 && self.io.get_line(i - 1).is_empty() {
                i -= 1;
            }
            // Write out the formatted range.
            while i < k_range(end_tok.get_index()).upper_endpoint() {
                // It's possible to run out of output lines (e.g. if the input ended with multiple
                // trailing newlines).
                if i < self.io.get_line_count() {
                    if i > 0 {
                        replacement.push_str(&self.line_separator);
                    }
                    replacement.push_str(self.io.get_line(i));
                }
                i += 1;
            }

            let end_position = end_tok.get_position() as usize + end_tok.get_original_text().len();
            let mut replace_to = end_position.min(text.len());
            // If the formatted range ended in the trailing trivia of the last token before EOF,
            // format all the way up to EOF to deal with trailing whitespace correctly.
            if end_tok.get_index() == self.java_input.get_kn() - 1 {
                replace_to = text.len();
            }
            // Replace trailing whitespace in the input with the whitespace from the formatted
            // file. If it includes line breaks, preserve the whitespace after the last newline to
            // avoid re-indenting the line following the formatted line.
            let mut newline: Option<usize> = None;
            while let Some(next) = text[replace_to..].chars().next() {
                if !next.is_whitespace() {
                    break;
                }
                let newline_length = newlines::has_newline_at(text, replace_to);
                if newline_length != -1 {
                    newline = Some(replace_to);
                    // Skip over the entire newline; don't count the second character of \r\n.
                    replace_to += newline_length as usize;
                } else {
                    replace_to += next.len_utf8();
                }
            }
            if let Some(newline) = newline {
                replace_to = newline;
            }

            if newline.is_none() {
                // There wasn't an existing trailing newline; add one.
                replacement.push_str(&self.line_separator);
            }
            while i < self.io.get_line_count() {
                let after = self.io.get_line(i);
                match after.find(|c: char| !c.is_whitespace()) {
                    // Write out trailing empty lines from the formatted output.
                    None => replacement.push_str(&self.line_separator),
                    Some(idx) => {
                        if newline.is_none() {
                            // If there wasn't a trailing newline in the input, indent the next line.
                            replacement.push_str(&after[..idx]);
                        }
                        break;
                    }
                }
                i += 1;
            }

            result.push(Replacement::create(
                replace_from as i32,
                replace_to as i32,
                replacement,
            ));
        }
        result
    }

    /// Expand a token range to start and end on acceptable boundaries for re-formatting.
    fn expand_to_breakable_regions(&self, i_range: Range) -> Range {
        // The original line range.
        let lo_tok = i_range.lower_endpoint();
        let hi_tok = i_range.upper_endpoint() - 1;

        // Expand the token indices to formattable boundaries (e.g. edges of statements).
        let (Some((lo, _)), Some((_, hi))) = (
            self.partial_format_ranges.range_containing(lo_tok),
            self.partial_format_ranges.range_containing(hi_tok),
        ) else {
            return EMPTY_RANGE;
        };
        Range::closed_open(lo, hi + 1)
    }

    pub fn apply_replacements(input: &str, replacements: &[Replacement]) -> String {
        let mut replacements: Vec<&Replacement> = replacements.iter().collect();
        replacements.sort_by_key(|r| std::cmp::Reverse(r.get_replace_range().lower_endpoint()));
        let mut writer = input.to_string();
        for replacement in replacements {
            let range = replacement.get_replace_range();
            writer.replace_range(
                range.lower_endpoint() as usize..range.upper_endpoint() as usize,
                replacement.get_replacement_string(),
            );
        }
        writer
    }
}
