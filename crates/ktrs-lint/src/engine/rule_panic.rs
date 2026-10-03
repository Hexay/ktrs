//! Rule exceptions are panics that the engine catches and reports as `KtLintRuleException`, as ktlint does with
//! Java exceptions; see `ktrs_syntax::caught_panic`.

pub(crate) use ktrs_syntax::caught_panic::catch_quietly as catch_rule_panic;
pub use ktrs_syntax::caught_panic::silence_caught_panics as silence_caught_rule_panics;
