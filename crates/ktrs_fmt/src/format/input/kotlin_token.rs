//! Port of ktfmt's `KotlinToken.kt`.

use std::rc::Rc;

use crate::doc::{Tok, Token};

#[derive(Debug)]
pub struct KotlinToken {
    toks_before: Vec<Rc<dyn Tok>>,
    kotlin_tok: Rc<dyn Tok>,
    toks_after: Vec<Rc<dyn Tok>>,
}

impl KotlinToken {
    pub fn new(
        toks_before: Vec<Rc<dyn Tok>>,
        kotlin_tok: Rc<dyn Tok>,
        toks_after: Vec<Rc<dyn Tok>>,
    ) -> Self {
        KotlinToken {
            toks_before,
            kotlin_tok,
            toks_after,
        }
    }
}

impl Token for KotlinToken {
    fn get_tok(&self) -> &Rc<dyn Tok> {
        &self.kotlin_tok
    }

    fn get_toks_before(&self) -> &[Rc<dyn Tok>] {
        &self.toks_before
    }

    fn get_toks_after(&self) -> &[Rc<dyn Tok>] {
        &self.toks_after
    }
}
