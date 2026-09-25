//! Port of `Paragraph.kt`, part 3: the line breaking algorithms (`reflowOptimal`,
//! `reflowGreedy`) and `toString`.

use std::fmt;

use super::kstring::{KString, to_string};
use super::options::KDocFormattingOptions;
use super::paragraph::Paragraph;
use super::utilities::get_indent_size;

fn len(s: &[u16]) -> i32 {
    s.len() as i32
}

impl Paragraph {
    pub(super) fn reflow_optimal(&self, max_line_width: i32, words: &[KString]) -> Vec<KString> {
        let count = words.len();
        let mut lines = Vec::new();

        let mut offsets = vec![0i32];
        for word in words {
            offsets.push(offsets.last().unwrap() + len(word).min(max_line_width));
        }

        let big = 10 << 20;
        let mut minimum = vec![big; count + 1];
        let mut breaks = vec![0usize; count + 1];
        minimum[0] = 0;

        let cost = |minimum: &[i32], i: usize, j: usize| -> i32 {
            let width = offsets[j] - offsets[i] + j as i32 - i as i32 - 1;
            if width <= max_line_width {
                let squared = (max_line_width - width) * (max_line_width - width);
                minimum[i] + squared
            } else {
                big
            }
        };

        let search = |minimum: &mut Vec<i32>, breaks: &mut Vec<usize>, pi0, pj0, pi1, pj1| {
            let mut stack: Vec<(usize, usize, usize, usize)> = vec![(pi0, pj0, pi1, pj1)];
            while let Some((i0, j0, i1, j1)) = stack.pop() {
                if j0 < j1 {
                    let j = (j0 + j1) / 2;
                    for i in i0..i1 {
                        let c = cost(minimum, i, j);
                        if c <= minimum[j] {
                            minimum[j] = c;
                            breaks[j] = i;
                        }
                    }
                    stack.push((breaks[j], j + 1, i1, j1));
                    stack.push((i0, j0, breaks[j] + 1, j));
                }
            }
        };

        let mut n = count + 1;
        let mut i = 0u32;
        let mut offset = 0;

        loop {
            let r = n.min(1 << (i + 1));
            let edge = (1 << i) + offset;
            search(&mut minimum, &mut breaks, offset, edge, edge, r + offset);
            let x = minimum[r - 1 + offset];
            let mut flag = true;
            for j in (1 << i)..(r.saturating_sub(1)) {
                let y = cost(&minimum, j + offset, r - 1 + offset);
                if y <= x {
                    n -= j;
                    i = 0;
                    offset += j;
                    flag = false;
                    break;
                }
            }
            if flag {
                if r == n {
                    break;
                }
                i += 1;
            }
        }

        let mut j = count;
        while j > 0 {
            let i = breaks[j];
            lines.push(words[i..j].join(&(' ' as u16)));
            j = i;
        }

        lines.reverse();
        lines
    }

    pub(super) fn reflow_greedy(
        &self,
        line_width: i32,
        options: &KDocFormattingOptions,
        words: &[KString],
    ) -> Vec<KString> {
        let mut width = line_width;
        if options.hanging_indent > 0 && self.hanging() && self.continuation {
            width -= get_indent_size(&self.hanging_indent, options);
        }

        let mut lines = Vec::new();
        let mut column = 0;
        let mut sb = KString::new();
        for word in words {
            if sb.is_empty() {
                sb.extend_from_slice(word);
                column += len(word);
            } else if column + len(word) + 1 <= width {
                sb.push(' ' as u16);
                sb.extend_from_slice(word);
                column += len(word) + 1;
            } else {
                width = line_width;
                if options.hanging_indent > 0 && self.hanging() {
                    width -= get_indent_size(&self.hanging_indent, options);
                }
                lines.push(std::mem::take(&mut sb));
                sb.extend_from_slice(word);
                column = len(&sb);
            }
        }
        if !sb.is_empty() {
            lines.push(sb);
        }
        lines
    }
}

impl fmt::Display for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, separate={}, block={}, hanging={}, preformatted={}, quoted={}, continuation={}, allowempty={}, separator={}",
            to_string(&self.content),
            self.separate,
            self.block,
            self.hanging(),
            self.preformatted,
            self.quoted,
            self.continuation,
            self.allow_empty,
            self.separator
        )
    }
}
