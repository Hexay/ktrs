//! Port of `InputOutput.java`: the line/range bookkeeping shared by inputs and outputs.

use std::collections::HashMap;
use std::rc::Rc;

use super::input::Tok;
use super::newlines;
use super::range::{EMPTY_RANGE, Range};

#[derive(Clone, Debug, Default)]
pub struct InputOutput {
    lines: Vec<String>,
    pub(crate) ranges: Vec<Range>,
}

impl InputOutput {
    pub fn set_lines(&mut self, lines: Vec<String>) {
        self.lines = lines;
    }

    pub fn get_line_count(&self) -> i32 {
        self.lines.len() as i32
    }

    pub fn get_line(&self, line_i: i32) -> &str {
        &self.lines[line_i as usize]
    }

    fn add_to_ranges(ranges: &mut Vec<Range>, i: usize, k: i32) {
        while ranges.len() <= i {
            ranges.push(EMPTY_RANGE);
        }
        let old_value = ranges[i];
        ranges[i] = Range::closed_open(
            if old_value.is_empty() {
                k
            } else {
                old_value.lower_endpoint()
            },
            k + 1,
        );
    }

    pub fn compute_ranges<T: Tok + ?Sized>(&mut self, toks: &[Rc<T>]) {
        let mut line_i = 0usize;
        for tok in toks {
            let txt = tok.get_original_text();
            let line_i0 = line_i;
            line_i += newlines::count(txt) as usize;
            let k = tok.get_index();
            if k >= 0 {
                for i in line_i0..=line_i {
                    Self::add_to_ranges(&mut self.ranges, i, k);
                }
            }
        }
    }

    pub fn make_k_to_ij(put: &InputOutput) -> HashMap<i32, Range> {
        let mut map: HashMap<i32, Range> = HashMap::new();
        let ij_n = put.get_line_count();
        for ij in 0..=ij_n {
            let range = put.get_ranges(ij);
            for k in range.lower_endpoint()..range.upper_endpoint() {
                let lower = map.get(&k).map_or(ij, |r| r.lower_endpoint());
                map.insert(k, Range::closed_open(lower, ij + 1));
            }
        }
        map
    }

    pub fn get_ranges(&self, line_i: i32) -> Range {
        if 0 <= line_i && (line_i as usize) < self.ranges.len() {
            self.ranges[line_i as usize]
        } else {
            EMPTY_RANGE
        }
    }
}
