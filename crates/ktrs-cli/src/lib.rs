//! Command-line front ends.
//!
//! - `ktfmt` — a drop-in for ktfmt's CLI: same flags, messages, exit codes and stdin/stdout.
//! - `ktlint` — a drop-in for ktlint's CLI (2.0.0-ALPHA-4): same flags, reporters, messages and exit codes.
//! - `ktrs`  — the native command (`ktrs fmt`, `ktrs lint`), a thin layer over the same engines, and
//!   `ktrs serve` for build tools.

pub mod ktfmt;
pub mod ktlint;
pub mod ktrs;
pub mod ktrs_lint;
pub mod serve;
