//! Port of `RedundantElementManager.kt` (lines 30-128): adds and removes elements that are not
//! strictly needed in the code, such as semicolons, unused imports and trailing commas.

use ktrs_psi::{
    KDocImpl, KtElement, KtFile, KtImportList, KtPackageDirective, KtReferenceExpression, KtVisitorVoid, PsiElement,
    PsiWhiteSpace, kt_tree_visitor_void, kt_visitor_void,
};

use super::FormatError;
use super::formatting_options::FormattingOptions;
use super::redundant_import_detector::RedundantImportDetector;
use super::redundant_semicolon_detector::RedundantSemicolonDetector;
use super::trailing_commas;

struct DropVisitor<'o> {
    options: &'o FormattingOptions,
    redundant_import_detector: RedundantImportDetector,
    redundant_semicolon_detector: RedundantSemicolonDetector,
    trailing_comma_detector: trailing_commas::Detector,
}

impl AsMut<RedundantImportDetector> for DropVisitor<'_> {
    fn as_mut(&mut self) -> &mut RedundantImportDetector {
        &mut self.redundant_import_detector
    }
}

impl KtVisitorVoid for DropVisitor<'_> {
    fn visit_element(&mut self, element: &PsiElement) {
        if let Some(kdoc) = element.cast::<KDocImpl>() {
            self.redundant_import_detector.take_kdoc(&kdoc);
            return;
        }

        self.redundant_semicolon_detector.take_element(element);
        if self.options.trailing_comma_management_strategy.remove_redundant_trailing_commas() {
            self.trailing_comma_detector.take_element(element);
        }
        kt_tree_visitor_void::visit_element(self, element);
    }

    fn visit_package_directive(&mut self, directive: &KtPackageDirective) {
        RedundantImportDetector::take_package_directive(self, directive, |v| {
            kt_visitor_void::visit_package_directive(v, directive)
        });
    }

    fn visit_import_list(&mut self, import_list: &KtImportList) {
        RedundantImportDetector::take_import_list(self, import_list, |v| kt_visitor_void::visit_import_list(v, import_list));
    }

    fn visit_reference_expression(&mut self, expression: &KtReferenceExpression) {
        self.redundant_import_detector.take_reference_expression(expression);
        kt_visitor_void::visit_reference_expression(self, expression);
    }
}

/// Remove extra semicolons and unused imports, if enabled in the [options].
pub fn drop_redundant_elements(file: &KtFile, options: &FormattingOptions) -> Result<String, FormatError> {
    let code = file.text();
    let mut visitor = DropVisitor {
        options,
        redundant_import_detector: RedundantImportDetector::new(options.remove_unused_imports),
        redundant_semicolon_detector: RedundantSemicolonDetector::default(),
        trailing_comma_detector: trailing_commas::Detector::default(),
    };

    file.accept(&mut visitor);

    let mut elements_to_remove: Vec<PsiElement> = visitor.redundant_semicolon_detector.get_redundant_semicolon_elements().to_vec();
    elements_to_remove.extend(visitor.redundant_import_detector.get_redundant_import_elements());
    elements_to_remove.extend_from_slice(visitor.trailing_comma_detector.get_trailing_comma_elements());
    if elements_to_remove.is_empty() {
        return Ok(code);
    }
    let mut result = code;

    elements_to_remove.sort_by_key(|e| std::cmp::Reverse(e.end_offset()));
    for element in elements_to_remove {
        // Don't insert extra newlines when the semicolon is already a line terminator.
        let replacement = if element.text() == ";" && !contains_newline(element.next_sibling().as_ref()) { "\n" } else { "" };
        result.replace_range(element.start_offset()..element.end_offset(), replacement);
    }

    Ok(result)
}

struct AddVisitor {
    trailing_comma_suggestor: trailing_commas::Suggestor,
}

impl KtVisitorVoid for AddVisitor {
    fn visit_element(&mut self, element: &PsiElement) {
        kt_tree_visitor_void::visit_element(self, element);
    }

    fn visit_kt_element(&mut self, element: &KtElement) {
        self.trailing_comma_suggestor.take_element(element);
        kt_tree_visitor_void::visit_element(self, element);
    }
}

pub fn add_redundant_elements(file: &KtFile, options: &FormattingOptions) -> Result<String, FormatError> {
    if !options.manage_trailing_commas() {
        return Ok(file.text());
    }

    let code = file.text();
    let mut visitor = AddVisitor { trailing_comma_suggestor: trailing_commas::Suggestor::default() };

    file.accept(&mut visitor);

    let mut suggestion_elements = visitor.trailing_comma_suggestor.get_trailing_comma_suggestions().to_vec();
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
