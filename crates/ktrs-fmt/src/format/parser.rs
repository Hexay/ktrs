//! Port of `Parser.kt` (lines 34-80). ktfmt parses every input as a script (`temp.kts`), whatever
//! the file's extension, and rejects any tree with an error element.

use ktrs_parser::{ChameleonCache, FileKind};
use ktrs_psi::{KtFile, PsiErrorElement};
use ktrs_syntax::{SyntaxKind, Tree};

use super::input::ParseError;

/// `cache` carries expanded blocks and lambdas between the parses of one `format` call.
pub fn parse(code: &str, cache: &mut ChameleonCache) -> Result<KtFile, ParseError> {
    let parse = ktrs_parser::parse_file_cached(code, FileKind::Script, cache);
    let kt_file = KtFile::with_text(&parse, code);
    // A cheap pre-check: `collectDescendantsOfType` visits every element.
    if !parse.tree.has_descendant_of_kind(Tree::ROOT, SyntaxKind::ERROR_ELEMENT) {
        return Ok(kt_file);
    }
    let descendants = kt_file.collect_descendants_of_type::<PsiErrorElement>();
    if let Some(error) = descendants.first() {
        return Err(throw_parse_error(code, &parse, error));
    }
    Ok(kt_file)
}

fn throw_parse_error(file_contents: &str, parse: &ktrs_syntax::Parse, error: &PsiErrorElement) -> ParseError {
    ParseError::at_offset(error.error_description(parse), file_contents, error.start_offset())
}
