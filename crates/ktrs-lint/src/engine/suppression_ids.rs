//! The rule ids an `@Suppress`/`@SuppressWarnings` value stands for (`SuppressionLocator.findRuleSuppressionIds`
//! and `SUPPRESS_ANNOTATION_RULE_MAP`).

use crate::editorconfig::KtlintVersion;
use crate::rule::RuleSetId;

pub(crate) const ALL_KTLINT_RULES_SUPPRESSION_ID: &str = "ktlint:suppress-all-rules";

/// 1.8 prefixes an id without rule set with `standard:` (backwards compatibility); 2.0 takes it as is.
pub(crate) fn find_rule_suppression_ids(value: &str, ktlint_version: KtlintVersion) -> Vec<String> {
    if value == "ktlint" {
        vec![ALL_KTLINT_RULES_SUPPRESSION_ID.to_owned()]
    } else if let Some(rule_id) = value.strip_prefix("ktlint:") {
        if ktlint_version.is_1_8() {
            vec![prefix_with_standard_rule_set_id_when_missing(rule_id)]
        } else {
            vec![rule_id.to_owned()]
        }
    } else {
        suppress_annotation_rule_map(value)
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

/// 1.8 `RuleId.prefixWithStandardRuleSetIdWhenMissing(id)`.
pub(crate) fn prefix_with_standard_rule_set_id_when_missing(id: &str) -> String {
    if id.contains(':') {
        id.to_owned()
    } else {
        format!("{}:{id}", RuleSetId::STANDARD.value())
    }
}

/// Non-ktlint suppressions that also suppress the matching ktlint rules.
fn suppress_annotation_rule_map(annotation_value: &str) -> &'static [&'static str] {
    match annotation_value {
        "EnumEntryName" => &["standard:enum-entry-name-case"],
        "RemoveCurlyBracesFromTemplate" => &["standard:string-template"],
        "ClassName" => &["standard:class-naming"],
        "FunctionName" => &["standard:function-naming"],
        "LocalVariableName" => &["standard:backing-property-naming"],
        "PackageName" => &["standard:package-name"],
        "PropertyName" | "ObjectPropertyName" => &[
            "standard:property-naming",
            "standard:backing-property-naming",
        ],
        "ConstPropertyName" | "PrivatePropertyName" => &["standard:property-naming"],
        "UnusedImport" => &["standard:no-unused-imports"],
        _ => &[],
    }
}

/// Kotlin `removeSurrounding(delimiter)`: only when both ends have it and they don't overlap.
pub(crate) fn remove_surrounding<'a>(s: &'a str, delimiter: &str) -> &'a str {
    if s.len() >= 2 * delimiter.len() && s.starts_with(delimiter) && s.ends_with(delimiter) {
        &s[delimiter.len()..s.len() - delimiter.len()]
    } else {
        s
    }
}
