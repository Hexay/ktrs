//! The ported standard rules (`ktlint-ruleset-standard`), registered like `StandardRuleSetProvider`.

mod backing_property_naming_rule;
mod class_naming_rule;
pub mod class_signature;
mod enum_entry_name_case_rule;
mod filename_rule;
mod final_newline_rule;
mod fun_keyword_spacing_rule;
mod function_expression_body_rule;
mod function_naming_rule;
mod function_return_type_spacing_rule;
pub mod function_signature;
mod function_start_of_body_spacing_rule;
mod function_type_modifier_spacing_rule;
mod function_type_reference_spacing_rule;
mod import_ordering_rule;
mod indentation;
pub mod internal;
pub mod max_line_length_rule;
mod mixed_condition_operators_rule;
mod modifier_list_spacing_rule;
mod modifier_order_rule;
mod multi_line_if_else_rule;
mod no_blank_line_at_start_of_file_rule;
mod no_empty_file_rule;
mod no_multiple_spaces_rule;
mod no_semicolons_rule;
mod no_trailing_spaces_rule;
mod no_unit_return_rule;
mod no_unused_imports_rule;
mod no_wildcard_imports_rule;
mod nullable_type_spacing_rule;
mod package_name_rule;
mod parameter_list_spacing_rule;
mod property_naming_rule;
mod spacing_around_angle_brackets_rule;
mod spacing_around_colon_rule;
mod spacing_around_comma_rule;
mod spacing_around_curly_rule;
mod spacing_around_dot_rule;
mod spacing_around_double_colon_rule;
mod spacing_around_keyword_rule;
mod spacing_around_operators_rule;
mod spacing_around_parens_rule;
mod spacing_around_range_operator_rule;
mod spacing_around_square_brackets_rule;
mod spacing_around_unary_operator_rule;
mod spacing_between_function_name_and_opening_parenthesis_rule;
mod string_template_indent;
mod string_template_rule;
mod try_catch_finally_spacing_rule;
mod type_argument_list_spacing_rule;
mod type_parameter_list_spacing_rule;

pub use backing_property_naming_rule::BackingPropertyNamingRule;
pub use class_naming_rule::ClassNamingRule;
pub use class_signature::ClassSignatureRule;
pub use enum_entry_name_case_rule::EnumEntryNameCaseRule;
pub use filename_rule::FilenameRule;
pub use final_newline_rule::FinalNewlineRule;
pub use fun_keyword_spacing_rule::FunKeywordSpacingRule;
pub use function_expression_body_rule::FunctionExpressionBodyRule;
pub use function_naming_rule::FunctionNamingRule;
pub use function_return_type_spacing_rule::FunctionReturnTypeSpacingRule;
pub use function_signature::FunctionSignatureRule;
pub use function_start_of_body_spacing_rule::FunctionStartOfBodySpacingRule;
pub use function_type_modifier_spacing_rule::FunctionTypeModifierSpacingRule;
pub use function_type_reference_spacing_rule::FunctionTypeReferenceSpacingRule;
pub use import_ordering_rule::ImportOrderingRule;
pub use indentation::{INDENT_WHEN_ARROW_ON_NEW_LINE, IndentationRule};
pub use max_line_length_rule::MaxLineLengthRule;
pub use mixed_condition_operators_rule::MixedConditionOperatorsRule;
pub use modifier_list_spacing_rule::ModifierListSpacingRule;
pub use modifier_order_rule::ModifierOrderRule;
pub use multi_line_if_else_rule::MultiLineIfElseRule;
pub use no_blank_line_at_start_of_file_rule::NoBlankLineAtStartOfFileRule;
pub use no_empty_file_rule::NoEmptyFileRule;
pub use no_multiple_spaces_rule::NoMultipleSpacesRule;
pub use no_semicolons_rule::NoSemicolonsRule;
pub use no_trailing_spaces_rule::NoTrailingSpacesRule;
pub use no_unit_return_rule::NoUnitReturnRule;
pub use no_unused_imports_rule::NoUnusedImportsRule;
pub use no_wildcard_imports_rule::NoWildcardImportsRule;
pub use nullable_type_spacing_rule::NullableTypeSpacingRule;
pub use package_name_rule::PackageNameRule;
pub use parameter_list_spacing_rule::ParameterListSpacingRule;
pub use property_naming_rule::PropertyNamingRule;
pub use spacing_around_angle_brackets_rule::SpacingAroundAngleBracketsRule;
pub use spacing_around_colon_rule::SpacingAroundColonRule;
pub use spacing_around_comma_rule::SpacingAroundCommaRule;
pub use spacing_around_curly_rule::SpacingAroundCurlyRule;
pub use spacing_around_dot_rule::SpacingAroundDotRule;
pub use spacing_around_double_colon_rule::SpacingAroundDoubleColonRule;
pub use spacing_around_keyword_rule::SpacingAroundKeywordRule;
pub use spacing_around_operators_rule::SpacingAroundOperatorsRule;
pub use spacing_around_parens_rule::SpacingAroundParensRule;
pub use spacing_around_range_operator_rule::SpacingAroundRangeOperatorRule;
pub use spacing_around_square_brackets_rule::SpacingAroundSquareBracketsRule;
pub use spacing_around_unary_operator_rule::SpacingAroundUnaryOperatorRule;
pub use spacing_between_function_name_and_opening_parenthesis_rule::SpacingBetweenFunctionNameAndOpeningParenthesisRule;
pub use string_template_indent::StringTemplateIndentRule;
pub use string_template_rule::StringTemplateRule;
pub use try_catch_finally_spacing_rule::TryCatchFinallySpacingRule;
pub use type_argument_list_spacing_rule::TypeArgumentListSpacingRule;
pub use type_parameter_list_spacing_rule::TypeParameterListSpacingRule;

use crate::rule::{About, RuleV2};
use crate::rule_provider::RuleV2Provider;

/// `STANDARD_RULE_ABOUT` (`StandardRule.kt`).
pub const STANDARD_RULE_ABOUT: About = About {
    maintainer: "KtLint",
    repository_url: "https://github.com/ktlint/ktlint",
    issue_tracker_url: "https://github.com/ktlint/ktlint/issues",
};

/// The ported slice of `StandardRuleSetProvider().getRuleProviders()`, in its order.
pub fn standard_rule_providers() -> Vec<RuleV2Provider> {
    vec![
        RuleV2Provider::new(|| Box::new(BackingPropertyNamingRule::new()) as Box<dyn RuleV2>),
        RuleV2Provider::new(|| Box::new(ClassNamingRule::default())),
        RuleV2Provider::new(|| Box::new(ClassSignatureRule::new())),
        RuleV2Provider::new(|| Box::new(EnumEntryNameCaseRule::default())),
        RuleV2Provider::new(|| Box::new(FilenameRule::default())),
        RuleV2Provider::new(|| Box::new(FinalNewlineRule::new())),
        RuleV2Provider::new(|| Box::new(FunKeywordSpacingRule)),
        RuleV2Provider::new(|| Box::new(FunctionExpressionBodyRule::new())),
        RuleV2Provider::new(|| Box::new(FunctionNamingRule::new())),
        RuleV2Provider::new(|| Box::new(FunctionReturnTypeSpacingRule::new())),
        RuleV2Provider::new(|| Box::new(FunctionSignatureRule::new())),
        RuleV2Provider::new(|| Box::new(FunctionStartOfBodySpacingRule)),
        RuleV2Provider::new(|| Box::new(FunctionTypeModifierSpacingRule)),
        RuleV2Provider::new(|| Box::new(FunctionTypeReferenceSpacingRule)),
        RuleV2Provider::new(|| Box::new(ImportOrderingRule::new())),
        RuleV2Provider::new(|| Box::new(IndentationRule::new())),
        RuleV2Provider::new(|| Box::new(MaxLineLengthRule::new())),
        RuleV2Provider::new(|| Box::new(MixedConditionOperatorsRule)),
        RuleV2Provider::new(|| Box::new(ModifierListSpacingRule::new())),
        RuleV2Provider::new(|| Box::new(ModifierOrderRule)),
        RuleV2Provider::new(|| Box::new(MultiLineIfElseRule::new())),
        RuleV2Provider::new(|| Box::new(NoBlankLineAtStartOfFileRule::default())),
        RuleV2Provider::new(|| Box::new(NoEmptyFileRule)),
        RuleV2Provider::new(|| Box::new(NoMultipleSpacesRule)),
        RuleV2Provider::new(|| Box::new(NoSemicolonsRule)),
        RuleV2Provider::new(|| Box::new(NoTrailingSpacesRule)),
        RuleV2Provider::new(|| Box::new(NoUnitReturnRule)),
        RuleV2Provider::new(|| Box::new(NoUnusedImportsRule::new())),
        RuleV2Provider::new(|| Box::new(NoWildcardImportsRule::new())),
        RuleV2Provider::new(|| Box::new(NullableTypeSpacingRule)),
        RuleV2Provider::new(|| Box::new(PackageNameRule)),
        RuleV2Provider::new(|| Box::new(ParameterListSpacingRule::new())),
        RuleV2Provider::new(|| Box::new(PropertyNamingRule::new())),
        RuleV2Provider::new(|| Box::new(SpacingAroundAngleBracketsRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundColonRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundCommaRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundCurlyRule::new())),
        RuleV2Provider::new(|| Box::new(SpacingAroundDotRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundDoubleColonRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundKeywordRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundOperatorsRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundParensRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundRangeOperatorRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundSquareBracketsRule)),
        RuleV2Provider::new(|| Box::new(SpacingAroundUnaryOperatorRule)),
        RuleV2Provider::new(|| Box::new(SpacingBetweenFunctionNameAndOpeningParenthesisRule)),
        RuleV2Provider::new(|| Box::new(StringTemplateIndentRule::new())),
        RuleV2Provider::new(|| Box::new(StringTemplateRule)),
        RuleV2Provider::new(|| Box::new(TryCatchFinallySpacingRule::new())),
        RuleV2Provider::new(|| Box::new(TypeArgumentListSpacingRule::new())),
        RuleV2Provider::new(|| Box::new(TypeParameterListSpacingRule::new())),
    ]
}

/// The provider of a ported standard rule by id (`standard:` prefix optional).
pub fn standard_rule_provider(id: &str) -> Option<RuleV2Provider> {
    let id = if id.contains(':') {
        id.to_owned()
    } else {
        format!("standard:{id}")
    };
    standard_rule_providers()
        .into_iter()
        .find(|p| p.rule_id().value() == id)
}
