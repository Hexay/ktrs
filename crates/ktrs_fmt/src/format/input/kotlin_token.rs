//! Port of ktfmt's `KotlinToken.kt`. `toksBefore`, the tok and `toksAfter` share one allocation.

use std::fmt;
use std::rc::Rc;

use crate::doc::{Tok, Token};

pub struct KotlinToken {
    /// `toksBefore ++ [kotlinTok] ++ toksAfter`.
    toks: Box<[Rc<dyn Tok>]>,
    /// Where `kotlinTok` is in `toks`.
    tok_i: usize,
}

impl KotlinToken {
    pub fn new(toks: Box<[Rc<dyn Tok>]>, tok_i: usize) -> Self {
        assert!(tok_i < toks.len());
        KotlinToken { toks, tok_i }
    }
}

impl fmt::Debug for KotlinToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KotlinToken")
            .field("toks_before", &self.get_toks_before())
            .field("kotlin_tok", self.get_tok())
            .field("toks_after", &self.get_toks_after())
            .finish()
    }
}

impl Token for KotlinToken {
    fn get_tok(&self) -> &Rc<dyn Tok> {
        &self.toks[self.tok_i]
    }

    fn get_toks_before(&self) -> &[Rc<dyn Tok>] {
        &self.toks[..self.tok_i]
    }

    fn get_toks_after(&self) -> &[Rc<dyn Tok>] {
        &self.toks[self.tok_i + 1..]
    }
}
