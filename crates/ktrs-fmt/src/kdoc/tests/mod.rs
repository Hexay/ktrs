//! Ports of ktfmt's `com.facebook.ktfmt.kdoc` test suites.
//!
//! `KDocFormatterTest.kt`'s `checkFormatter` cases live in `data/<upstream test name>.txt`, dumped
//! from the upstream test class on the JVM so every option, indent and expectation is exact (see
//! `harness.rs` for the format). Its extra non-`checkFormatter` assertions are in `formatter_extra.rs`.

mod escaping_test;
mod formatter_cases;
mod formatter_extra;
mod harness;
mod utilities_test;
