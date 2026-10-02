//! Rule exceptions are panics that the engine catches and reports as `KtLintRuleException`, as ktlint does with
//! Java exceptions. The default panic hook would still print each one to stderr, which the jar does not.

use std::cell::Cell;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Once;

thread_local! {
    static CATCHING: Cell<bool> = const { Cell::new(false) };
}

pub(crate) fn catch_rule_panic<R>(f: impl FnOnce() -> R) -> std::thread::Result<R> {
    let outer = CATCHING.replace(true);
    let result = panic::catch_unwind(AssertUnwindSafe(f));
    CATCHING.set(outer);
    result
}

/// Installs a panic hook that stays quiet for panics the engine catches; any other panic still prints.
pub fn silence_caught_rule_panics() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if !CATCHING.get() {
                previous(info);
            }
        }));
    });
}
