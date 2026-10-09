//! `MagicNumber.kt`.

use std::cell::OnceCell;
use std::sync::Arc;

use ktrs_psi::{
    KtAnnotationEntry, KtBinaryExpression, KtCallElement, KtConstantExpression, KtDotQualifiedExpression, KtEnumEntry, KtFunction,
    KtNamedFunction, KtObjectDeclaration, KtOperationReferenceExpression, KtParameter, KtPrefixExpression, KtPrimaryConstructor,
    KtProperty, KtReturnExpression, KtSecondaryConstructor, KtValueArgument, PsiElement,
};
use ktrs_syntax::SyntaxKind::{FLOAT_CONSTANT, INTEGER_CONSTANT, MINUS, RANGE};

use crate::api::{Config, Entity, Finding, Rule, RuleBase, config_property};
use crate::psi::{is_constant, is_hash_code_function, is_part_of};

const HEX_RADIX: u32 = 16;
const BINARY_RADIX: u32 = 2;
const RANGE_OPERATORS: [&str; 3] = ["downTo", "until", "step"];

/// This rule detects and reports usages of magic numbers in the code. Prefer defining constants with clear names
/// describing what the magic number means.
pub struct MagicNumber {
    base: RuleBase,
    ignore_numbers: OnceCell<Vec<f64>>,
    ignore_hash_code_function: OnceCell<bool>,
    ignore_property_declaration: OnceCell<bool>,
    ignore_local_variable_declaration: OnceCell<bool>,
    ignore_constant_declaration: OnceCell<bool>,
    ignore_companion_object_property_declaration: OnceCell<bool>,
    ignore_annotation: OnceCell<bool>,
    ignore_named_argument: OnceCell<bool>,
    ignore_enums: OnceCell<bool>,
    ignore_ranges: OnceCell<bool>,
    ignore_extension_functions: OnceCell<bool>,
}

macro_rules! flag {
    ($name:ident, $key:literal, $default:literal) => {
        fn $name(&self) -> bool {
            *self.$name.get_or_init(|| config_property::boolean(self.base.config.as_ref(), $key, $default))
        }
    };
}

impl MagicNumber {
    pub fn new(config: Arc<dyn Config>) -> Self {
        MagicNumber {
            base: RuleBase::new(
                config,
                "Report magic numbers. Magic number is a numeric literal that is not defined as a constant \
                 and hence it's unclear what the purpose of this number is. \
                 It's better to declare such numbers as constants and give them a proper name. \
                 By default, -1, 0, 1, and 2 are not considered to be magic numbers.",
            ),
            ignore_numbers: OnceCell::new(),
            ignore_hash_code_function: OnceCell::new(),
            ignore_property_declaration: OnceCell::new(),
            ignore_local_variable_declaration: OnceCell::new(),
            ignore_constant_declaration: OnceCell::new(),
            ignore_companion_object_property_declaration: OnceCell::new(),
            ignore_annotation: OnceCell::new(),
            ignore_named_argument: OnceCell::new(),
            ignore_enums: OnceCell::new(),
            ignore_ranges: OnceCell::new(),
            ignore_extension_functions: OnceCell::new(),
        }
    }

    fn ignore_numbers(&self) -> &[f64] {
        self.ignore_numbers.get_or_init(|| {
            let numbers = config_property::list(self.base.config.as_ref(), "ignoreNumbers", &["-1", "0", "1", "2"]);
            let mut numbers: Vec<f64> =
                numbers.iter().map(|it| parse_as_double(it).unwrap_or_else(|| panic!("java.lang.NumberFormatException: For input string: \"{it}\""))).collect();
            numbers.sort_by(f64::total_cmp);
            numbers
        })
    }

    flag!(ignore_hash_code_function, "ignoreHashCodeFunction", true);
    flag!(ignore_property_declaration, "ignorePropertyDeclaration", true);
    flag!(ignore_local_variable_declaration, "ignoreLocalVariableDeclaration", true);
    flag!(ignore_constant_declaration, "ignoreConstantDeclaration", true);
    flag!(ignore_companion_object_property_declaration, "ignoreCompanionObjectPropertyDeclaration", true);
    flag!(ignore_annotation, "ignoreAnnotation", false);
    flag!(ignore_named_argument, "ignoreNamedArgument", true);
    flag!(ignore_enums, "ignoreEnums", false);
    flag!(ignore_ranges, "ignoreRanges", false);
    flag!(ignore_extension_functions, "ignoreExtensionFunctions", true);

    fn is_ignored_by_config(&self, expression: &KtConstantExpression) -> bool {
        (self.ignore_property_declaration() && is_property(expression))
            || (self.ignore_local_variable_declaration() && is_local_property(expression))
            || (self.ignore_constant_declaration() && is_constant_property(expression))
            || (self.ignore_companion_object_property_declaration() && is_companion_object_property(expression))
            || (self.ignore_annotation() && is_part_of::<KtAnnotationEntry>(expression))
            || (self.ignore_hash_code_function() && is_part_of_hash_code(expression))
            || (self.ignore_enums() && is_part_of::<KtEnumEntry>(expression))
            || (self.ignore_named_argument() && is_named_argument(expression))
            || (self.ignore_ranges() && is_part_of_range(expression))
            || (self.ignore_extension_functions() && is_subject_of_extension_function(expression))
    }
}

impl Rule for MagicNumber {
    crate::rule_base!(MagicNumber);
}

crate::detekt_visitor! {
    impl MagicNumber {
        fn visit_constant_expression(&mut self, expression: &KtConstantExpression) {
            let element_type = expression.element_type();
            if element_type != INTEGER_CONSTANT && element_type != FLOAT_CONSTANT {
                return;
            }

            if self.is_ignored_by_config(expression)
                || is_part_of_function_return_constant(expression)
                || is_part_of_constructor_or_function_constant(expression)
            {
                return;
            }

            let parent = expression.parent();
            let raw_number = match &parent {
                Some(parent) if has_unary_minus_prefix(parent) => parent.text(),
                _ => expression.text(),
            };

            let number = parse_as_double(&raw_number);
            if number.is_some_and(|number| !self.ignore_numbers().iter().any(|ignored| double_equals(*ignored, number))) {
                self.report(Finding::new(
                    Entity::from(expression),
                    "This expression contains a magic number. Consider defining it to a well named constant.",
                ));
            }
        }
    }
}

/// `java.lang.Double.equals`: by bits, so `0.0` is not `-0.0` and NaN equals NaN.
fn double_equals(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
}

/// `parseAsDoubleOrNull` / `parseAsDouble`: `None` is the `NumberFormatException`.
fn parse_as_double(raw_number: &str) -> Option<f64> {
    let normalized_text = normalize_for_parsing_as_double(raw_number);
    if let Some(hex) = normalized_text.strip_prefix("0x") {
        i64::from_str_radix(hex, HEX_RADIX).ok().map(|n| n as f64)
    } else if let Some(binary) = normalized_text.strip_prefix("0b") {
        i64::from_str_radix(binary, BINARY_RADIX).ok().map(|n| n as f64)
    } else {
        to_double(&normalized_text)
    }
}

/// Kotlin `String.toDouble()` for what a normalized literal can be (digits, `.`, exponent, sign).
fn to_double(text: &str) -> Option<f64> {
    let text = text.trim_matches(|c: char| c <= ' ');
    let is_literal = text.bytes().all(|b| b.is_ascii_digit() || matches!(b, b'.' | b'e' | b'-' | b'+'));
    if is_literal { text.parse().ok() } else { None }
}

// Gotcha: the suffixes go one after the other, so a hex literal ending in `d` or `f` loses that digit too.
fn normalize_for_parsing_as_double(text: &str) -> String {
    let text = text.trim().to_lowercase().replace('_', "");
    let mut text = text.as_str();
    for suffix in ["ul", "l", "d", "f", "u"] {
        text = text.strip_suffix(suffix).unwrap_or(text);
    }
    text.to_owned()
}

fn is_named_argument(expression: &KtConstantExpression) -> bool {
    let value_argument = expression.get_parent_of_type::<KtValueArgument>(false);
    value_argument.is_some_and(|argument| argument.is_named()) && is_part_of::<KtCallElement>(expression)
}

fn is_part_of_function_return_constant(expression: &KtConstantExpression) -> bool {
    let Some(parent) = expression.parent() else { return false };
    parent.is::<KtNamedFunction>() || (parent.is::<KtReturnExpression>() && parent.parent().is_some_and(|p| p.children().len() == 1))
}

fn is_part_of_constructor_or_function_constant(expression: &KtConstantExpression) -> bool {
    let Some(parent) = expression.parent().filter(|parent| parent.is::<KtParameter>()) else { return false };
    let owner = parent.parent().and_then(|list| list.parent());
    owner.is_some_and(|owner| owner.is::<KtNamedFunction>() || owner.is::<KtPrimaryConstructor>() || owner.is::<KtSecondaryConstructor>())
}

fn is_part_of_range(expression: &KtConstantExpression) -> bool {
    let Some(the_parent) = expression.parent() else { return false };

    // Case 1: Direct range expression
    if let Some(the_parent) = the_parent.cast::<KtBinaryExpression>() {
        return is_range_operation(&the_parent);
    }

    // Case 2: Negative number part of a range
    if the_parent.is::<KtPrefixExpression>() {
        if !has_unary_minus_prefix(&the_parent) {
            return false;
        }
        let Some(grand_parent) = the_parent.parent().and_then(|p| p.cast::<KtBinaryExpression>()) else { return false };
        return is_range_operation(&grand_parent);
    }

    false
}

/// `operationToken == KtTokens.RANGE || operationReference.getReferencedName() in RANGE_OPERATORS`.
fn is_range_operation(expression: &KtBinaryExpression) -> bool {
    expression.operation_token() == Some(RANGE)
        || expression.operation_reference().is_some_and(|reference| RANGE_OPERATORS.contains(&reference.referenced_name().as_str()))
}

fn is_subject_of_extension_function(expression: &KtConstantExpression) -> bool {
    expression.parent().is_some_and(|parent| parent.is::<KtDotQualifiedExpression>())
}

fn is_part_of_hash_code(expression: &KtConstantExpression) -> bool {
    let containing_function = expression.get_parent_of_type::<KtNamedFunction>(false);
    containing_function.is_some_and(|function| is_hash_code_function(&function.upcast::<KtFunction>()))
}

fn is_local_property(expression: &KtConstantExpression) -> bool {
    expression.get_parent_of_type::<KtProperty>(false).is_some_and(|property| property.is_local())
}

fn is_property(expression: &KtConstantExpression) -> bool {
    expression.get_parent_of_type::<KtProperty>(false).is_some_and(|property| !property.is_local())
}

fn is_companion_object_property(expression: &KtConstantExpression) -> bool {
    is_property(expression) && is_in_companion_object(expression)
}

fn is_in_companion_object(expression: &KtConstantExpression) -> bool {
    expression.get_parent_of_type::<KtObjectDeclaration>(false).is_some_and(|object| object.is_companion())
}

fn is_constant_property(expression: &KtConstantExpression) -> bool {
    is_property(expression) && expression.get_parent_of_type::<KtProperty>(false).is_some_and(|property| is_constant(&property))
}

fn has_unary_minus_prefix(element: &PsiElement) -> bool {
    element.is::<KtPrefixExpression>()
        && element
            .first_child()
            .and_then(|first_child| first_child.cast::<KtOperationReferenceExpression>())
            .and_then(|reference| reference.operation_sign_token_type())
            .is_some_and(|token| token.0 == MINUS)
}
