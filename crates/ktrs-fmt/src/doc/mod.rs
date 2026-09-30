//! Port of google-java-format v1.23.0's layout engine (`com.google.googlejavaformat`), limited to
//! what ktfmt uses: `OpsBuilder` -> `Op`s -> `DocBuilder` -> `Doc` -> `JavaOutput`.
//!
//! Units: positions are UTF-8 byte offsets, widths/columns UTF-16 units (see `utf16`).
//!
//! Pipeline (ktfmt `Formatter.prettyPrint`):
//! ```text
//! let mut output = JavaOutput::new("\n", &input, helper);
//! let mut builder = OpsBuilder::new(&input, &mut output);
//! /* visitor drives builder */ builder.sync(len); builder.drain();
//! let ops = builder.build()?;
//! let mut doc = DocBuilder::new().with_ops(ops).build();
//! doc.compute_breaks(output.get_comments_helper(), max_width, State::new(0, 0));
//! doc.write(&mut output);
//! output.flush();
//! let edits = output.get_format_replacements(&token_ranges);
//! JavaOutput::apply_replacements(code, &edits)
//! ```

mod blank_line_wanted;
mod comments_helper;
#[allow(clippy::module_inception)]
mod doc;
mod doc_builder;
mod doc_leaves;
mod formatting_error;
mod indent;
mod input;
mod input_output;
mod java_identifier_tables;
mod java_output;
mod java_output_replacements;
mod level;
pub mod newlines;
mod op;
mod ops_builder;
mod ops_builder_breaks;
mod ops_builder_build;
mod ops_builder_positions;
mod output;
mod range;
mod range_map;
mod replacement;
mod utf16;

#[cfg(test)]
mod tests;

pub use blank_line_wanted::BlankLineWanted;
pub use comments_helper::{CommentsHelper, reformat_parameter_comment};
pub use doc::{Doc, DocKind, FillMode, MAX_LINE_WIDTH, State};
pub use doc_builder::DocBuilder;
pub use doc_leaves::{DocBreak, DocTok, DocToken, RealOrImaginary};
pub use formatting_error::{FormatterDiagnostic, FormatterException, FormattingError};
pub use indent::Indent;
pub use input::{Input, Tok, Token};
pub use input_output::InputOutput;
pub use java_output::JavaOutput;
pub use level::Level;
pub use op::Op;
pub use ops_builder::OpsBuilder;
pub use output::{BreakTag, Output};
pub use range::{EMPTY_RANGE, Range, RangeSet};
pub use range_map::RangeMap;
pub use replacement::Replacement;
pub use utf16::utf16_len;
