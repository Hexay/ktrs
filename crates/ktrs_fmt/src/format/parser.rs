//! Port of `Parser.kt` (lines 34-80). ktfmt parses every input as a script (`temp.kts`), whatever
//! the file's extension, and rejects any tree with an error element.

use ktrs_parser::FileKind;
use ktrs_psi::{KtFile, PsiErrorElement};
use ktrs_syntax::SyntaxKind;

use super::input::ParseError;

pub fn parse(code: &str) -> Result<KtFile, ParseError> {
    let parse = ktrs_parser::parse_file(code, FileKind::Script);
    let kt_file = KtFile::with_text(&parse, code);
    // A cheap pre-check: the cursor walk of `collectDescendantsOfType` allocates per node.
    if !has_descendant_of_kind(&parse.green, SyntaxKind::ERROR_ELEMENT) {
        return Ok(kt_file);
    }
    let descendants = kt_file.collect_descendants_of_type::<PsiErrorElement>();
    if let Some(error) = descendants.first() {
        return Err(throw_parse_error(code, &parse, error));
    }
    Ok(kt_file)
}

/// Whether any node or token under `node` has `kind`, scanning the green tree (no cursor allocations).
pub fn has_descendant_of_kind(node: &rowan::GreenNodeData, kind: SyntaxKind) -> bool {
    node.children().any(|child| {
        child.kind().0 == kind as u16 || child.as_node().is_some_and(|n| has_descendant_of_kind(n, kind))
    })
}

fn throw_parse_error(file_contents: &str, parse: &ktrs_syntax::Parse, error: &PsiErrorElement) -> ParseError {
    ParseError::at_offset(error.error_description(parse), file_contents, error.start_offset())
}
