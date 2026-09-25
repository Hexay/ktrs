//! Port of `Indent.java`: `Indent.Const` and `Indent.If`.

use std::rc::Rc;

use super::output::BreakTag;

#[derive(Clone, Debug)]
pub enum Indent {
    Const(i32),
    If(Rc<IndentIf>),
}

#[derive(Debug)]
pub struct IndentIf {
    condition: BreakTag,
    then_indent: Indent,
    else_indent: Indent,
}

impl Indent {
    /// `Indent.Const.ZERO`.
    pub const ZERO: Indent = Indent::Const(0);

    /// `Indent.Const.make(n, indentMultiplier)`.
    pub fn make_const(n: i32, indent_multiplier: i32) -> Indent {
        Indent::Const(n * indent_multiplier)
    }

    /// `Indent.If.make(condition, thenIndent, elseIndent)`.
    pub fn make_if(condition: &BreakTag, then_indent: Indent, else_indent: Indent) -> Indent {
        Indent::If(Rc::new(IndentIf {
            condition: condition.clone(),
            then_indent,
            else_indent,
        }))
    }

    pub fn eval(&self) -> i32 {
        match self {
            Indent::Const(n) => *n,
            Indent::If(i) => {
                if i.condition.was_break_taken() {
                    i.then_indent.eval()
                } else {
                    i.else_indent.eval()
                }
            }
        }
    }
}
