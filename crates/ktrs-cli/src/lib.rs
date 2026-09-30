//! Command-line front ends.
//!
//! - `ktfmt` — a drop-in for ktfmt's CLI: same flags, messages, exit codes and stdin/stdout.
//! - `ktrs`  — the native command (`ktrs fmt`), a thin layer over the same engine, and `ktrs serve`
//!   for build tools.

pub mod ktfmt;
pub mod ktrs;
pub mod serve;
