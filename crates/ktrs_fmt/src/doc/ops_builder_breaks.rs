//! `OpsBuilder`'s `space`/`breakOp`/`breakToFill`/`forcedBreak` overloads. Java overloads get
//! suffixed names; the three-argument `breakOp(FillMode, String, Indent)` is plain `break_op`.

use super::doc::FillMode;
use super::doc_leaves::DocBreak;
use super::indent::Indent;
use super::op::Op;
use super::ops_builder::OpsBuilder;
use super::output::BreakTag;

impl OpsBuilder<'_> {
    pub fn space(&mut self) {
        self.add(Op::Space);
    }

    /// `breakOp()`.
    pub fn break_op_default(&mut self) {
        self.break_op(FillMode::Unified, "", Indent::ZERO);
    }

    /// `breakOp(Indent plusIndent)`.
    pub fn break_op_indent(&mut self, plus_indent: Indent) {
        self.break_op(FillMode::Unified, "", plus_indent);
    }

    /// `breakToFill()`.
    pub fn break_to_fill(&mut self) {
        self.break_op(FillMode::Independent, "", Indent::ZERO);
    }

    /// `forcedBreak()`.
    pub fn forced_break(&mut self) {
        self.break_op(FillMode::Forced, "", Indent::ZERO);
    }

    /// `forcedBreak(Indent plusIndent)`.
    pub fn forced_break_indent(&mut self, plus_indent: Indent) {
        self.break_op(FillMode::Forced, "", plus_indent);
    }

    /// `breakOp(String flat)`.
    pub fn break_op_flat(&mut self, flat: &str) {
        self.break_op(FillMode::Unified, flat, Indent::ZERO);
    }

    /// `breakToFill(String flat)`.
    pub fn break_to_fill_flat(&mut self, flat: &str) {
        self.break_op(FillMode::Independent, flat, Indent::ZERO);
    }

    /// `breakOp(FillMode fillMode, String flat, Indent plusIndent)`.
    pub fn break_op(&mut self, fill_mode: FillMode, flat: &str, plus_indent: Indent) {
        self.break_op_tagged(fill_mode, flat, plus_indent, None);
    }

    /// `breakOp(FillMode, String, Indent, Optional<BreakTag> optionalTag)`.
    pub fn break_op_tagged(
        &mut self,
        fill_mode: FillMode,
        flat: &str,
        plus_indent: Indent,
        optional_tag: Option<BreakTag>,
    ) {
        self.add(Op::Break(DocBreak::make_tagged(
            fill_mode,
            flat,
            plus_indent,
            optional_tag,
        )));
    }
}
