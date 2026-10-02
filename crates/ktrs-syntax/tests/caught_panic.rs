//! The hook from `silence_caught_panics` passes on only the panics `catch_quietly` doesn't catch.

use std::panic;
use std::sync::atomic::{AtomicUsize, Ordering};

use ktrs_syntax::caught_panic::{catch_quietly, silence_caught_panics};

static REPORTED: AtomicUsize = AtomicUsize::new(0);

#[test]
fn caught_panics_are_silent_and_others_still_reach_the_previous_hook() {
    panic::set_hook(Box::new(|_| {
        REPORTED.fetch_add(1, Ordering::Relaxed);
    }));
    silence_caught_panics();

    assert!(catch_quietly(|| panic!("java.lang.IllegalStateException")).is_err());
    assert_eq!(catch_quietly(|| 7).ok(), Some(7));
    assert_eq!(REPORTED.load(Ordering::Relaxed), 0);

    assert!(panic::catch_unwind(|| panic!("a bug")).is_err());
    assert_eq!(REPORTED.load(Ordering::Relaxed), 1);
}
