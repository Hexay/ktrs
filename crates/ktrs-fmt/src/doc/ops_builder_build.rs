//! `OpsBuilder.build()`: splices each token's comments into the op stream.

use super::blank_line_wanted::BlankLineWanted;
use super::doc::FillMode;
use super::doc_leaves::{DocBreak, DocTok};
use super::formatting_error::FormattingError;
use super::indent::Indent;
use super::input::Tok;
use super::op::Op;
use super::ops_builder::OpsBuilder;

impl<'a> OpsBuilder<'a, '_> {
    /// Build the list of `Op`s, or the first `FormattingError` Java would have thrown.
    pub fn build(mut self) -> Result<Vec<Op<'a>>, FormattingError> {
        self.mark_for_partial_format();
        if let Some(error) = self.error.take() {
            return Err(error);
        }
        let ops = std::mem::take(&mut self.ops);
        let ops_n = ops.len();
        // Rewrite the ops to insert comments.
        let mut tok_ops = TokOps::default();
        for i in 0..ops_n {
            let Op::Token(token_op) = &ops[i] else {
                continue;
            };
            // Token ops can have associated non-tokens, including comments, which we need to
            // insert. They can also cause line breaks, so we insert them before or after the
            // current level, when possible.
            let token = token_op.get_token();
            let mut j = i; // Where to insert toksBefore before.
            while 0 < j && matches!(ops[j - 1], Op::Open(_)) {
                j -= 1;
            }
            let mut k = i; // Where to insert toksAfter after.
            while k + 1 < ops_n && matches!(ops[k + 1], Op::Close) {
                k += 1;
            }
            if token_op.real_or_imaginary().is_real() {
                // Regular input token. Copy out toksBefore before token, and toksAfter after it.
                let mut newlines = 0; // Count of newlines in a row.
                let mut space = false; // Do we need an extra space after a previous "/*" comment?
                let mut last_was_comment = false; // Was the last thing we output a comment?
                let mut allow_blank_after_last_comment = false;
                for tok_before in token.get_toks_before() {
                    if tok_before.is_newline() {
                        newlines += 1;
                    } else if tok_before.is_comment() {
                        tok_ops.push(j,Op::Break(DocBreak::make(
                            if tok_before.is_slash_slash_comment() {
                                FillMode::Forced
                            } else {
                                FillMode::Unified
                            },
                            "",
                            token_op.get_plus_indent_comments_before().clone(),
                        )));
                        Self::make_comment(&mut tok_ops, j, tok_before);
                        space = tok_before.is_slash_star_comment();
                        newlines = 0;
                        last_was_comment = true;
                        if tok_before.is_javadoc_comment() {
                            tok_ops.push(j,Op::Break(DocBreak::make_forced()));
                        }
                        allow_blank_after_last_comment = tok_before.is_slash_slash_comment()
                            || (tok_before.is_slash_star_comment()
                                && !tok_before.is_javadoc_comment());
                    }
                }
                if allow_blank_after_last_comment && newlines > 1 {
                    // Force a line break after two newlines in a row following a line or block comment
                    self.output
                        .blank_line(token.get_tok().get_index(), BlankLineWanted::YES);
                }
                if last_was_comment && newlines > 0 {
                    tok_ops.push(j,Op::Break(DocBreak::make_forced()));
                } else if space {
                    tok_ops.push(j,Op::Space);
                }
                // Now we've seen the Token; output the toksAfter.
                for tok_after in token.get_toks_after() {
                    if tok_after.is_comment() {
                        let trailing_indent = token_op.break_and_indent_trailing_comment();
                        let break_after = tok_after.is_javadoc_comment()
                            || (tok_after.is_slash_star_comment() && trailing_indent.is_some());
                        if break_after {
                            let indent = trailing_indent.cloned().unwrap_or(Indent::ZERO);
                            tok_ops.push(k + 1,Op::Break(DocBreak::make(
                                FillMode::Forced,
                                "",
                                indent,
                            )));
                        } else {
                            tok_ops.push(k + 1,Op::Space);
                        }
                        Self::make_comment(&mut tok_ops, k + 1, tok_after);
                        if break_after {
                            tok_ops.push(k + 1,Op::Break(DocBreak::make(
                                FillMode::Forced,
                                "",
                                Indent::ZERO,
                            )));
                        }
                    }
                }
            } else {
                // This input token was mistakenly not generated for output. As no whitespace or
                // comments were generated (presumably), copy all input non-tokens literally, even
                // spaces and newlines.
                let mut newlines = 0;
                let mut last_was_comment = false;
                for tok_before in token.get_toks_before() {
                    if tok_before.is_newline() {
                        newlines += 1;
                    } else if tok_before.is_comment() {
                        newlines = 0;
                        last_was_comment = tok_before.is_comment();
                    }
                    if last_was_comment && newlines > 0 {
                        tok_ops.push(j,Op::Break(DocBreak::make_forced()));
                    }
                    tok_ops.push(j,Op::Tok(DocTok::make(tok_before)));
                }
                for tok_after in token.get_toks_after() {
                    tok_ops.push(k + 1,Op::Tok(DocTok::make(tok_after)));
                }
            }
        }
        // Construct new list of ops, splicing in the comments. If a comment is inserted
        // immediately before a space, suppress the space.
        let mut new_ops = Vec::with_capacity(ops_n + tok_ops.0.len());
        let mut after_forced_break = false; // Was the last Op a forced break? If so, suppress spaces.
        let mut tok_ops = tok_ops.into_sorted();
        for (i, op) in ops.into_iter().enumerate() {
            while tok_ops.as_slice().first().is_some_and(|(at, _)| *at == i) {
                let (_, tok_op) = tok_ops.next().unwrap();
                if !(after_forced_break && matches!(tok_op, Op::Space)) {
                    after_forced_break = tok_op.is_forced_break();
                    new_ops.push(tok_op);
                }
            }
            if after_forced_break
                && (matches!(op, Op::Space)
                    || matches!(&op, Op::Break(b) if b.get_plus_indent() == 0 && b.flat() == " "))
            {
                continue;
            }
            if !matches!(op, Op::Open(_)) {
                after_forced_break = op.is_forced_break();
            }
            new_ops.push(op);
        }
        for (_, tok_op) in tok_ops {
            if !(after_forced_break && matches!(tok_op, Op::Space)) {
                after_forced_break = tok_op.is_forced_break();
                new_ops.push(tok_op);
            }
        }
        Ok(new_ops)
    }

    /// `makeComment(comment)`, its ops pushed to `tok_ops` at `i` instead of returned as a list.
    fn make_comment(tok_ops: &mut TokOps<'a>, i: usize, comment: &'a Tok<'a>) {
        tok_ops.push(i, Op::Tok(DocTok::make(comment)));
        if !comment.is_slash_star_comment() {
            tok_ops.push(i, Op::Break(DocBreak::make_forced()));
        }
    }
}

/// Upstream's `tokOps` multimap (ops to insert before op `i`), kept sparse: most ops get none.
#[derive(Default)]
struct TokOps<'a>(Vec<(usize, Op<'a>)>);

impl<'a> TokOps<'a> {
    fn push(&mut self, i: usize, op: Op<'a>) {
        self.0.push((i, op));
    }

    /// Stable, so ops inserted at the same index keep their order.
    fn into_sorted(mut self) -> std::vec::IntoIter<(usize, Op<'a>)> {
        self.0.sort_by_key(|(i, _)| *i);
        self.0.into_iter()
    }
}
