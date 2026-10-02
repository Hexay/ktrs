//! Port of `core/Emitter.kt`.

use ktrs_ast::{Ast, NodeId};

/// `fun interface Emitter`. The extension `report(element, errorMessage)` is `report(ast, element, message, false)`.
pub trait Emitter {
    fn report(&mut self, ast: &Ast, element: NodeId, error_message: &str, can_be_auto_corrected: bool) -> Decision;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Fix,
    Ignore,
}

impl Decision {
    /// `ifFix { }`: runs `block` when the engine allows the fix.
    pub fn if_fix(self, block: impl FnOnce()) {
        if self == Decision::Fix {
            block();
        }
    }
}
