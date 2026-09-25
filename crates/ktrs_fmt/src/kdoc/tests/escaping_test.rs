//! Port of `EscapingTest.kt`.

use crate::kdoc::escape_kdoc;

#[test]
fn escaping_kdoc() {
    assert_eq!(escape_kdoc("/** foo */"), "/** foo */");
    assert_eq!(escape_kdoc("/*** foo */"), "/*** foo */");
    assert_eq!(escape_kdoc("/** * foo */"), "/** * foo */");
    assert_eq!(escape_kdoc("/** /* foo */"), "/** \u{0004}\u{0005} foo */");
    assert_eq!(escape_kdoc("/** /* foo */ */"), "/** \u{0004}\u{0005} foo \u{0005}\u{0004} */");

    assert_eq!(escape_kdoc("/* /* foo */ */"), "/* \u{0004}\u{0005} foo \u{0005}\u{0004} */");
}
