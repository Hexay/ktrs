//! Port of `Parser.kt` (lines 43-89). ktfmt parses the code as `temp.kt` or `temp.kts` by its file
//! type and rejects any tree with an error element.

use ktrs_parser::{ChameleonCache, FileKind};
use ktrs_psi::{KtFile, PsiErrorElement};
use ktrs_syntax::{SyntaxKind, Tree};

use super::FormatError;
use super::input::ParseError;
use super::kotlin_code::{FileType, KotlinCode};

/// `cache` carries expanded blocks and lambdas between the parses of one `format` call.
pub fn parse(code: &KotlinCode, cache: &mut ChameleonCache) -> Result<KtFile, FormatError> {
    let file_kind = match code.file_type {
        FileType::Regular => FileKind::Source,
        FileType::Script => FileKind::Script,
    };
    let parse = ktrs_parser::parse_file_cached(&code.code, file_kind, cache);
    // Before any parse error: see `ktrs_syntax::MissedTokens`.
    if let Some(missed) = parse.first_missed_tokens() {
        return Err(FormatError::MissedTokens(missed.clone()));
    }
    let kt_file = KtFile::with_text(&parse, &code.code);
    // A cheap pre-check: `collectDescendantsOfType` visits every element.
    if !parse.tree.has_descendant_of_kind(Tree::ROOT, SyntaxKind::ERROR_ELEMENT) {
        return Ok(kt_file);
    }
    let descendants = kt_file.collect_descendants_of_type::<PsiErrorElement>();
    if let Some(error) = descendants.first() {
        return Err(throw_parse_error(&code.code, &parse, error).into());
    }
    Ok(kt_file)
}

fn throw_parse_error(file_contents: &str, parse: &ktrs_syntax::Parse, error: &PsiErrorElement) -> ParseError {
    ParseError::at_offset(error.error_description(parse), file_contents, error.start_offset())
}
