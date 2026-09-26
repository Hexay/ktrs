//! Port of `Parser.kt` (lines 34-80). ktfmt parses every input as a script (`temp.kts`), whatever
//! the file's extension, and rejects any tree with an error element.

use ktrs_parser::FileKind;
use ktrs_psi::{KtFile, PsiErrorElement};

use super::input::ParseError;

pub fn parse(code: &str) -> Result<KtFile, ParseError> {
    let parse = ktrs_parser::parse_file(code, FileKind::Script);
    let kt_file = KtFile::new(&parse);
    let descendants = kt_file.collect_descendants_of_type::<PsiErrorElement>();
    if let Some(error) = descendants.first() {
        return Err(throw_parse_error(code, &parse, error));
    }
    Ok(kt_file)
}

fn throw_parse_error(file_contents: &str, parse: &ktrs_syntax::Parse, error: &PsiErrorElement) -> ParseError {
    ParseError::at_offset(error.error_description(parse), file_contents, error.start_offset())
}
