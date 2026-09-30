//! Ports of `Op.java`, `OpenOp.java` and `CloseOp.java`. Java's `Op` implementors become variants:
//! `OpenOp.make(i)` is `Op::Open(i)`, `CloseOp.make()` is `Op::Close`, and the `Doc` leaves that
//! implement `Op` are carried by value.

use super::doc::{Doc, DocKind};
use super::doc_builder::DocBuilder;
use super::doc_leaves::{DocBreak, DocTok, DocToken};
use super::indent::Indent;

#[derive(Debug)]
pub enum Op<'a> {
    Open(Indent),
    Close,
    Token(DocToken<'a>),
    Space,
    Break(DocBreak),
    Tok(DocTok<'a>),
    /// ktfmt's `FenceCommentsOp`: adds nothing, but keeps `OpsBuilder.build` from hoisting
    /// comments past it into parent levels.
    FenceComments,
}

impl<'a> Op<'a> {
    pub fn add(self, builder: &mut DocBuilder<'a>) {
        match self {
            Op::Open(plus_indent) => builder.open(plus_indent),
            Op::Close => builder.close(),
            Op::Token(token) => builder.add(Doc::new(DocKind::Token(token))),
            Op::Space => builder.add(Doc::new(DocKind::Space)),
            Op::Break(b) => builder.break_doc(Doc::new(DocKind::Break(b))),
            Op::Tok(tok) => builder.add(Doc::new(DocKind::Tok(tok))),
            Op::FenceComments => {}
        }
    }

    pub(crate) fn is_forced_break(&self) -> bool {
        matches!(self, Op::Break(b) if b.is_forced())
    }
}
