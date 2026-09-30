//! ktfmt's input layer (`KotlinInput.kt`, `Tokenizer.kt`, `WhitespaceTombstones.kt`): the Kotlin
//! tree as a gjf `Input`. `KotlinTok.kt`/`KotlinToken.kt` are `doc::Tok`/`doc::Token` (see `doc::input`).

mod kotlin_input;
mod parse_error;
mod string_util;
mod tokenizer;
pub mod whitespace_tombstones;

#[cfg(test)]
mod tests;

pub use crate::doc::{Tok as KotlinTok, Token as KotlinToken};
pub use kotlin_input::KotlinInput;
pub use parse_error::ParseError;
pub use string_util::offset_to_line_column;
pub use tokenizer::Tokenizer;
