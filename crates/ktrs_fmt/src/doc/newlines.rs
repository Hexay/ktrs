//! Port of `Newlines.java`. Offsets are byte offsets (see `utf16`).

/// `BREAKS`, in `ImmutableSet` iteration (insertion) order.
const BREAKS: [&str; 3] = ["\r\n", "\n", "\r"];

pub fn count(input: &str) -> i32 {
    line_offset_iterator(input).count() as i32 - 1
}

pub fn first_break(input: &str) -> i32 {
    let mut it = line_offset_iterator(input);
    it.next();
    it.next().map_or(-1, |i| i as i32)
}

pub fn is_newline(input: &str) -> bool {
    BREAKS.contains(&input)
}

pub fn has_newline_at(input: &str, idx: usize) -> i32 {
    for b in BREAKS {
        if input.get(idx..).is_some_and(|rest| rest.starts_with(b)) {
            return b.len() as i32;
        }
    }
    -1
}

pub fn get_line_ending(input: &str) -> Option<&'static str> {
    BREAKS.into_iter().find(|&b| input.ends_with(b))
}

pub fn guess_line_separator(text: &str) -> &'static str {
    let bytes = text.as_bytes();
    for (i, &c) in bytes.iter().enumerate() {
        match c {
            b'\r' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    return "\r\n";
                }
                return "\r";
            }
            b'\n' => return "\n",
            _ => {}
        }
    }
    "\n"
}

pub fn contains_breaks(text: &str) -> bool {
    text.bytes().any(|c| c == b'\n' || c == b'\r')
}

/// Yields `0`, then the offset just past each line break.
pub fn line_offset_iterator(input: &str) -> LineOffsetIterator<'_> {
    LineOffsetIterator {
        curr: Some(0),
        idx: 0,
        input: input.as_bytes(),
    }
}

/// Yields each line including its terminator; a trailing unterminated line is yielded too.
pub fn line_iterator(input: &str) -> LineIterator<'_> {
    let mut indices = line_offset_iterator(input);
    let idx = indices.next().unwrap_or(0);
    LineIterator {
        idx,
        input,
        indices,
    }
}

pub struct LineOffsetIterator<'a> {
    curr: Option<usize>,
    idx: usize,
    input: &'a [u8],
}

impl LineOffsetIterator<'_> {
    fn advance(&mut self) {
        while self.idx < self.input.len() {
            match self.input[self.idx] {
                b'\r' | b'\n' => {
                    if self.input[self.idx] == b'\r'
                        && self.idx + 1 < self.input.len()
                        && self.input[self.idx + 1] == b'\n'
                    {
                        self.idx += 1;
                    }
                    self.idx += 1;
                    self.curr = Some(self.idx);
                    return;
                }
                _ => self.idx += 1,
            }
        }
        self.curr = None;
    }
}

impl Iterator for LineOffsetIterator<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let result = self.curr?;
        self.advance();
        Some(result)
    }
}

pub struct LineIterator<'a> {
    idx: usize,
    input: &'a str,
    indices: LineOffsetIterator<'a>,
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.idx >= self.input.len() {
            return None;
        }
        let last = self.idx;
        self.idx = self.indices.next().unwrap_or(self.input.len());
        Some(&self.input[last..self.idx])
    }
}
