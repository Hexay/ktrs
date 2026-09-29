//! `OpsBuilder.actualSize` / `actualStartColumn` (first in `OpsBuilder.java`; unused by ktfmt).

use super::ops_builder::OpsBuilder;

impl OpsBuilder<'_, '_> {
    /// The size of the AST node at `position` including comments, in bytes.
    pub fn actual_size(&self, position: i32, length: i32) -> i32 {
        let map = self.input.get_position_token_map();
        let start_token = &self.tokens[*map.get(position).expect("no token at position")];
        let mut start = start_token.get_tok().get_position();
        for tok in start_token.get_toks_before() {
            if tok.is_comment() {
                start = start.min(tok.get_position());
            }
        }
        let end_token = &self.tokens[*map
            .get(position + length - 1)
            .expect("no token at position")];
        let end_tok = end_token.get_tok();
        let mut end = end_tok.get_position() + end_tok.get_original_text().len() as i32;
        for tok in end_token.get_toks_after() {
            if tok.is_comment() {
                end = end.max(tok.get_position() + tok.get_original_text().len() as i32);
            }
        }
        end - start
    }

    /// The start position of the token at `position`, including leading comments on its line.
    pub fn actual_start_column(&self, position: i32) -> i32 {
        let start_token = &self.tokens[*self
            .input
            .get_position_token_map()
            .get(position)
            .expect("no token at position")];
        let mut start = start_token.get_tok().get_position();
        let line0 = self.input.get_line_number(start);
        for tok in start_token.get_toks_before() {
            if line0 != self.input.get_line_number(tok.get_position()) {
                return start;
            }
            if tok.is_comment() {
                start = start.min(tok.get_position());
            }
        }
        start
    }
}
