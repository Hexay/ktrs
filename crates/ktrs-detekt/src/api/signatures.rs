//! `detekt-api/.../internal/Signatures.kt`: the signature of a reported element (the baseline id and SARIF
//! fingerprint are built from it).

use ktrs_psi::{KtClassOrObject, KtNamedFunction, PsiElement};
use ktrs_syntax::SyntaxKind::*;

use crate::kt_file;

pub(crate) fn build_full_signature(element: &PsiElement) -> String {
    let mut full_signature = search_signature(element);
    let mut parent_signatures: Vec<String> =
        parents(element).iter().filter(|p| p.is::<KtClassOrObject>()).map(extract_class_name).collect();
    parent_signatures.reverse();
    let parent_signatures = parent_signatures.join(".");

    if !parent_signatures.is_empty() {
        full_signature =
            if !full_signature.is_empty() { format!("{parent_signatures}${full_signature}") } else { parent_signatures };
    }

    full_signature
}

/// psiUtil `parents`: the strict ancestors up to and including the file.
fn parents(element: &PsiElement) -> Vec<PsiElement> {
    std::iter::successors(element.parent(), PsiElement::parent).collect()
}

fn extract_class_name(element: &PsiElement) -> String {
    element.get_parent_of_type::<KtClassOrObject>(false).map(|c| c.name_as_safe_name().as_string().to_owned()).unwrap_or_default()
}

fn search_signature(element: &PsiElement) -> String {
    let signature = if element.is_file() {
        file_signature(element)
    } else if let Some(function) = element.cast::<KtNamedFunction>() {
        build_function_signature(&function)
    } else if let Some(class_or_object) = element.cast::<KtClassOrObject>() {
        build_class_signature(&class_or_object)
    } else {
        match element.kind() {
            SECONDARY_CONSTRUCTOR if !element.is_leaf() => "constructor".to_owned(),
            PRIMARY_CONSTRUCTOR if !element.is_leaf() => String::new(),
            // KtFunction.name: `<anonymous>`
            FUNCTION_LITERAL if !element.is_leaf() => "<anonymous>".to_owned(),
            _ => element.text(),
        }
    };
    collapse_whitespaces(&signature.replace('\n', " "))
}

/// `replace(Regex("\\s{2,}"), " ")` with Java's `\s` (`[ \t\n\x0B\f\r]`).
fn collapse_whitespaces(text: &str) -> String {
    let is_space = |c: char| matches!(c, ' ' | '\t' | '\n' | '\u{b}' | '\u{c}' | '\r');
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if is_space(c) && chars.peek().is_some_and(|&next| is_space(next)) {
            while chars.peek().is_some_and(|&next| is_space(next)) {
                chars.next();
            }
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}

fn file_signature(file: &PsiElement) -> String {
    let context = kt_file::containing_file(file);
    format!("{}.{}", context.file().package_fq_name().as_string(), context.name())
}

fn build_class_signature(class_or_object: &KtClassOrObject) -> String {
    let mut base_name = class_or_object.name_as_safe_name().as_string().to_owned();
    let type_parameters = class_or_object.type_parameters();
    if !type_parameters.is_empty() {
        base_name += "<";
        base_name += &type_parameters.iter().map(|p| p.text()).collect::<Vec<_>>().join(", ");
        base_name += ">";
    }
    let extended_entries = class_or_object.super_type_list_entries();
    if !extended_entries.is_empty() {
        base_name += " : ";
    }
    for entry in &extended_entries {
        base_name += &entry.type_as_user_type().and_then(|t| t.referenced_name()).unwrap_or_default();
    }
    base_name
}

fn build_function_signature(element: &KtNamedFunction) -> String {
    let text = element.text_slice();
    let start_offset = element.start_offset_skipping_comments() - element.start_offset();
    let value_param_list = element.value_parameter_list();
    let end_offset = match element.type_reference() {
        Some(type_reference) => type_reference.end_offset(),
        None => value_param_list.as_ref().map_or(0, |l| l.end_offset()),
    } as isize
        - element.start_offset() as isize;

    assert!(
        (start_offset as isize) < end_offset,
        "Error building function signature with range {start_offset} - {end_offset} for element: {text}"
    );
    let end_offset = end_offset as usize;
    match value_param_list {
        None => text[start_offset..end_offset].to_owned(),
        Some(list) => {
            let (list_start, list_end) = (list.start_offset() - element.start_offset(), list.end_offset() - element.start_offset());
            format!("{}{}", &text[start_offset..list_start], &text[list_end..end_offset])
        }
    }
}
