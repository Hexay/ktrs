//! Ported code mimics a Java exception with a panic, caught where the JVM tool catches the exception
//! (ktlint's rule boundary, ktfmt's per-file `catch`). The default panic hook would still print each
//! one to stderr, which the jar does not; [`silence_caught_panics`] keeps those quiet.

use std::cell::Cell;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Once;

thread_local! {
    static CATCHING: Cell<bool> = const { Cell::new(false) };
}

/// `catch_unwind`, marking the panics it catches as expected (silent once the hook is installed).
pub fn catch_quietly<R>(f: impl FnOnce() -> R) -> std::thread::Result<R> {
    let outer = CATCHING.replace(true);
    let result = panic::catch_unwind(AssertUnwindSafe(f));
    CATCHING.set(outer);
    result
}

/// Installs (once per process) a panic hook that stays quiet for panics [`catch_quietly`] catches;
/// any other panic still prints.
pub fn silence_caught_panics() {
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
