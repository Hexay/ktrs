//! `detekt-metrics`: the measures rules share with the report processors. Ported so far: what the rules call.

pub mod cyclomatic_complexity;
mod lines_of_code;

pub use cyclomatic_complexity::CyclomaticComplexity;
pub use lines_of_code::lines_of_code;
