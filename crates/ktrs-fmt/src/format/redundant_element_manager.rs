//! Port of `RedundantElementManager.kt` (lines 30-128): adds and removes elements that are not
//! strictly needed in the code, such as semicolons, unused imports and trailing commas.

use ktrs_psi::{KDocImpl, KtElement, KtFile, KtReferenceExpression, PsiElement, PsiWhiteSpace};

use ktrs_syntax::SyntaxKind;

use super::FormatError;
use super::formatting_options::FormattingOptions;
use super::redundant_import_detector::RedundantImportDetector;
use super::redundant_semicolon_detector::RedundantSemicolonDetector;
use super::trailing_commas;

/// Remove extra semicolons and unused imports, if enabled in the [options].
pub fn drop_redundant_elements(file: &KtFile, options: &FormattingOptions) -> Result<String, FormatError> {
    let code = file.text();
    let mut import_detector = RedundantImportDetector::new(options.remove_unused_imports);
    let mut semicolon_detector = RedundantSemicolonDetector::default();
    let mut trailing_comma_detector = trailing_commas::Detector::default();
    let removes_trailing_commas = options.trailing_comma_management_strategy.remove_redundant_trailing_commas();
    let tree = file.tree();

    // Upstream's `KtTreeVisitorVoid`, as a preorder scan: the same elements in the same order,
    // with each override's work done where the visitor would do it (see `visitor/dispatch.rs`).
    // Only the kinds some override or detector acts on are visited (the detectors: `;` and `,` leaves).
    let (mut package_end, mut import_list_end, mut kdoc_end) = (None, None, file.id());
    let comma = if removes_trailing_commas { SyntaxKind::COMMA } else { SyntaxKind::SEMICOLON };
    let visited = tree.find_kinds(
        file.id(),
        [
            SyntaxKind::PACKAGE_DIRECTIVE,
            SyntaxKind::IMPORT_LIST,
            SyntaxKind::REFERENCE_EXPRESSION,
            SyntaxKind::ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION,
            SyntaxKind::OPERATION_REFERENCE,
            SyntaxKind::LABEL,
            SyntaxKind::CALL_EXPRESSION,
            SyntaxKind::ARRAY_ACCESS_EXPRESSION,
            SyntaxKind::DOC_COMMENT,
        ],
        [SyntaxKind::SEMICOLON, comma],
    );
    for id in visited {
        if id < kdoc_end {
            continue;
        }
        if package_end.is_some_and(|e| id >= e) {
            import_detector.leave_package_directive();
            package_end = None;
        }
        if import_list_end.is_some_and(|e| id >= e) {
            import_detector.leave_import_list();
            import_list_end = None;
        }
        let kind = tree.kind(id);
        if tree.is_token(id) {
            match kind {
                SyntaxKind::SEMICOLON => semicolon_detector.take_element(&file.at(id)),
                _ => trailing_comma_detector.take_element(&file.at(id)),
            }
            continue;
        }
        match kind {
            SyntaxKind::PACKAGE_DIRECTIVE => {
                import_detector.enter_package_directive(&file.at(id).cast().expect("package directive"));
                package_end = Some(tree.subtree_end(id));
            }
            SyntaxKind::IMPORT_LIST => {
                import_detector.enter_import_list(&file.at(id).cast().expect("import list"));
                import_list_end = Some(tree.subtree_end(id));
            }
            // The kinds `accept` routes to `visitReferenceExpression`.
            SyntaxKind::REFERENCE_EXPRESSION
            | SyntaxKind::ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION
            | SyntaxKind::OPERATION_REFERENCE
            | SyntaxKind::LABEL
            | SyntaxKind::CALL_EXPRESSION
            | SyntaxKind::ARRAY_ACCESS_EXPRESSION => {
                let reference = file.at(id).cast::<KtReferenceExpression>().expect("reference expression kind");
                import_detector.take_reference_expression(&reference);
            }
            // `visitElement`: KDoc is read for references and not descended into.
            SyntaxKind::DOC_COMMENT => {
                import_detector.take_kdoc(&file.at(id).cast::<KDocImpl>().expect("KDoc"));
                kdoc_end = tree.subtree_end(id);
            }
            _ => {}
        }
    }

    let mut elements_to_remove: Vec<PsiElement> = semicolon_detector.get_redundant_semicolon_elements().to_vec();
    elements_to_remove.extend(import_detector.get_redundant_import_elements());
    elements_to_remove.extend_from_slice(trailing_comma_detector.get_trailing_comma_elements());
    if elements_to_remove.is_empty() {
        return Ok(code);
    }
    let mut result = code;

    elements_to_remove.sort_by_key(|e| std::cmp::Reverse(e.end_offset()));
    for element in elements_to_remove {
        // Don't insert extra newlines when the semicolon is already a line terminator.
        let replacement = if element.text() == ";" && !contains_newline(element.next_sibling().as_ref()) { "\n" } else { "" };
        string_builder_replace(&mut result, element.start_offset(), element.end_offset(), replacement)?;
    }

    Ok(result)
}

/// `StringBuilder.replace`: `end` is clamped to the length, a `start` past it throws. Reached when a
/// removed element contains another one, e.g. an import directive that swallowed a trailing comment
/// before its own `;` (`import a; /* x */`).
fn string_builder_replace(result: &mut String, start: usize, end: usize, replacement: &str) -> Result<(), FormatError> {
    let end = end.min(result.len());
    if start > end {
        return Err(FormatError::Runtime(format!(
            "java.lang.StringIndexOutOfBoundsException: Range [{start}, {end}) out of bounds for length {}",
            result.len()
        )));
    }
    result.replace_range(start..end, replacement);
    Ok(())
}

pub fn add_redundant_elements(file: &KtFile, options: &FormattingOptions) -> Result<String, FormatError> {
    if !options.manage_trailing_commas() {
        return Ok(file.text());
    }

    let code = file.text();
    let mut suggestor = trailing_commas::Suggestor::default();

    // Upstream visits every KtElement (a `KtTreeVisitorVoid`) and offers each to the suggestor, which
    // acts only on list-like ones: a preorder scan for those visits the same elements in the same order
    // without dispatching at every node.
    let tree = file.tree();
    for id in tree.find_kinds(file.id(), trailing_commas::Suggestor::LIST_KINDS, []) {
        if let Some(kt_element) = file.at(id).cast::<KtElement>() {
            suggestor.take_element(&kt_element);
        }
    }

    let mut suggestion_elements = suggestor.get_trailing_comma_suggestions().to_vec();
    if suggestion_elements.is_empty() {
        return Ok(code);
    }
    let mut result = code;

    suggestion_elements.sort_by_key(|e| std::cmp::Reverse(e.end_offset()));
    for element in suggestion_elements {
        result.insert(element.end_offset(), ',');
    }

    Ok(result)
}

fn contains_newline(element: Option<&PsiElement>) -> bool {
    element.is_some_and(|e| e.is::<PsiWhiteSpace>() && e.text_contains('\n'))
}
