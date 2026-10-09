//! `detekt-core/.../suppressors/Suppressions.kt` and `Suppressor.kt`.

use ktrs_psi::{KtAnnotated, KtAnnotationEntry, KtElement};

use crate::api::config::IGNORE_ANNOTATED_KEY;
use crate::api::{Config, RuleName, RuleSetId, config_property};

fn is_forbidden_suppress_by_id(id: &str) -> bool {
    extract_rule_name(id).is_some_and(|name| name.value() == "ForbiddenSuppress")
}

/// `extractRuleName(key)` (RuleDescriptor.kt): the part before `/`, when it is a valid rule name.
pub(crate) fn extract_rule_name(key: &str) -> Option<RuleName> {
    RuleName::try_new(key.split('/').next().unwrap_or(key))
}

/// Checks if this psi element is suppressed by @Suppress or @SuppressWarnings annotations.
/// If this element cannot have annotations, the first annotative parent is searched.
pub fn is_suppressed_by(element: &KtElement, id: &str, aliases: &[String], rule_set_id: Option<&RuleSetId>) -> bool {
    if is_forbidden_suppress_by_id(id) {
        return false;
    }
    let annotation_entries = all_annotation_entries(element);
    if annotation_entries.is_empty() {
        return false;
    }

    let mut accepted_suppression_ids = vec![id.to_owned(), "ALL".to_owned(), "all".to_owned(), "All".to_owned()];
    if let Some(rule_set_id) = rule_set_id {
        accepted_suppression_ids.extend([rule_set_id.value().to_owned(), format!("{rule_set_id}.{id}"), format!("{rule_set_id}:{id}")]);
    }
    accepted_suppression_ids.extend(aliases.iter().cloned());

    annotation_entries
        .iter()
        .filter(|entry| entry.type_reference().is_some_and(|t| matches!(t.text_slice(), "Suppress" | "SuppressWarnings")))
        .flat_map(|entry| entry.value_arguments())
        .filter_map(|argument| argument.argument_expression().map(|e| e.text()))
        .map(|text| remove_detekt_suppression_prefix(&text))
        .map(|text| text.replace('"', ""))
        .any(|text| accepted_suppression_ids.contains(&text))
}

fn all_annotation_entries(element: &KtElement) -> Vec<KtAnnotationEntry> {
    let mut entries = Vec::new();
    let mut annotated = element.get_parent_of_type::<KtAnnotated>(false);
    while let Some(current) = annotated {
        entries.extend(current.annotation_entries());
        annotated = current.get_parent_of_type::<KtAnnotated>(true);
    }
    entries
}

/// `replace("detekt[.:]".toRegex(IGNORE_CASE), "")`.
fn remove_detekt_suppression_prefix(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut index = 0;
    while index < bytes.len() {
        let is_prefix = bytes.len() - index >= 7 && bytes[index..index + 6].eq_ignore_ascii_case(b"detekt") && matches!(bytes[index + 6], b'.' | b':');
        if is_prefix {
            index += 7;
            continue;
        }
        let char_len = text[index..].chars().next().map_or(1, char::len_utf8);
        out.push_str(&text[index..index + char_len]);
        index += char_len;
    }
    out
}

/// `buildSuppressors(rule, analysisMode)`: the `ignoreAnnotated` and `ignoreFunction` suppressors.
// TODO: port AnnotationSuppressor (AnnotationExcluder, FullQualifiedNameGuesser) and FunctionSuppressor
// (FunctionMatcher); until then a config that uses either key stops the run instead of reporting too much.
pub(crate) fn assert_no_suppressors(config: &dyn Config, rule_id: &str) {
    for key in [IGNORE_ANNOTATED_KEY, "ignoreFunction"] {
        let values = config_property::list(config, key, &[]);
        assert!(values.is_empty(), "`{key}` is not supported yet (rule {rule_id})");
    }
}
