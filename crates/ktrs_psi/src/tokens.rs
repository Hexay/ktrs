//! Token classes and token sets owned by psi-api (`KtTokens` value tables, `KtTokenSets`, private sets of
//! PSI classes). Parser-owned sets are re-exported from `ktrs_parser::kt_tokens`.

use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::{self, *};

pub use ktrs_parser::kt_tokens::{ALL_ASSIGNMENTS, OPERATIONS, VAL_VAR};

/// `KtTokenSets.TYPE_ELEMENT_TYPES`.
pub const TYPE_ELEMENT_TYPES: TokenSet =
    TokenSet::create(&[USER_TYPE, NULLABLE_TYPE, FUNCTION_TYPE, DYNAMIC_TYPE, INTERSECTION_TYPE]);
/// `KtTokenSets.SUPER_TYPE_LIST_ENTRIES`.
pub const SUPER_TYPE_LIST_ENTRIES: TokenSet =
    TokenSet::create(&[DELEGATED_SUPER_TYPE_ENTRY, SUPER_TYPE_CALL_ENTRY, SUPER_TYPE_ENTRY]);
/// `KtTokenSets.INSIDE_DIRECTIVE_EXPRESSIONS`.
pub const INSIDE_DIRECTIVE_EXPRESSIONS: TokenSet = TokenSet::create(&[REFERENCE_EXPRESSION, DOT_QUALIFIED_EXPRESSION]);
/// `KtNameReferenceExpression.NAME_REFERENCE_EXPRESSIONS`.
pub(crate) const NAME_REFERENCE_EXPRESSIONS: TokenSet = TokenSet::create(&[IDENTIFIER, THIS_KEYWORD, SUPER_KEYWORD]);
/// `KtClassOrObject.classInterfaceObjectTokenSet`.
pub(crate) const CLASS_INTERFACE_OBJECT: TokenSet = TokenSet::create(&[CLASS_KEYWORD, INTERFACE_KEYWORD, OBJECT_KEYWORD]);
/// `KtDestructuringDeclaration.OPENING_BRACES` / `CLOSING_BRACES`.
pub(crate) const OPENING_BRACES: TokenSet = TokenSet::create(&[LPAR, LBRACKET]);
pub(crate) const CLOSING_BRACES: TokenSet = TokenSet::create(&[RPAR, RBRACKET]);

/// `KtOperationReferenceExpression.OPERATION_TOKENS`: postfix + prefix operations + every
/// `BinaryOperationPrecedence` token.
pub(crate) const OPERATION_TOKENS: TokenSet = TokenSet::create(&[
    // KtTokenSets.POSTFIX_OPERATIONS
    PLUSPLUS, MINUSMINUS, EXCLEXCL, DOT, SAFE_ACCESS,
    // KtTokenSets.PREFIX_OPERATIONS
    MINUS, PLUS, EXCL,
    // BinaryOperationPrecedence
    AS_KEYWORD, AS_SAFE, MUL, DIV, PERC, RANGE, RANGE_UNTIL, IDENTIFIER, ELVIS, IN_KEYWORD, NOT_IN, IS_KEYWORD,
    NOT_IS, LT, GT, LTEQ, GTEQ, EQEQ, EXCLEQ, EQEQEQ, EXCLEQEQEQ, ANDAND, OROR, EQ, PLUSEQ, MINUSEQ, MULTEQ, DIVEQ,
    PERCEQ,
]);

/// Kotlin `elementType is KtModifierKeywordToken`.
pub fn is_modifier_keyword_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        FUN_KEYWORD
            | IN_KEYWORD
            | ABSTRACT_KEYWORD
            | ENUM_KEYWORD
            | CONTRACT_KEYWORD
            | OPEN_KEYWORD
            | INNER_KEYWORD
            | OVERRIDE_KEYWORD
            | PRIVATE_KEYWORD
            | PUBLIC_KEYWORD
            | INTERNAL_KEYWORD
            | PROTECTED_KEYWORD
            | OUT_KEYWORD
            | VARARG_KEYWORD
            | REIFIED_KEYWORD
            | COMPANION_KEYWORD
            | SEALED_KEYWORD
            | FINAL_KEYWORD
            | LATEINIT_KEYWORD
            | DATA_KEYWORD
            | VALUE_KEYWORD
            | INLINE_KEYWORD
            | NOINLINE_KEYWORD
            | TAILREC_KEYWORD
            | EXTERNAL_KEYWORD
            | ANNOTATION_KEYWORD
            | CROSSINLINE_KEYWORD
            | OPERATOR_KEYWORD
            | INFIX_KEYWORD
            | CONST_KEYWORD
            | SUSPEND_KEYWORD
            | EXPECT_KEYWORD
            | ACTUAL_KEYWORD
    )
}

/// An element type typed as `KtSingleValueToken` (e.g. `KtQualifiedExpression.operationSign`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct KtSingleValueToken(pub SyntaxKind);

impl KtSingleValueToken {
    /// `getValue()`, e.g. `"?."` for `SAFE_ACCESS`.
    pub fn value(self) -> &'static str {
        single_value(self.0).unwrap_or_else(|| panic!("{:?} is not a KtSingleValueToken", self.0))
    }
}

/// `KtSingleValueToken.getValue()` (keywords are single-value tokens too); `None` for other tokens.
pub fn single_value(kind: SyntaxKind) -> Option<&'static str> {
    let value = match kind {
        LBRACKET => "[",
        RBRACKET => "]",
        LBRACE => "{",
        RBRACE => "}",
        LPAR => "(",
        RPAR => ")",
        DOT => ".",
        PLUSPLUS => "++",
        MINUSMINUS => "--",
        MUL => "*",
        PLUS => "+",
        MINUS => "-",
        EXCL => "!",
        DIV => "/",
        PERC => "%",
        LT => "<",
        GT => ">",
        LTEQ => "<=",
        GTEQ => ">=",
        EQEQEQ => "===",
        ARROW => "->",
        DOUBLE_ARROW => "=>",
        EXCLEQEQEQ => "!==",
        EQEQ => "==",
        EXCLEQ => "!=",
        EXCLEXCL => "!!",
        ANDAND => "&&",
        AND => "&",
        OROR => "||",
        SAFE_ACCESS => "?.",
        ELVIS => "?:",
        QUEST => "?",
        COLONCOLON => "::",
        COLON => ":",
        SEMICOLON => ";",
        DOUBLE_SEMICOLON => ";;",
        RANGE => "..",
        RANGE_UNTIL => "..<",
        EQ => "=",
        MULTEQ => "*=",
        DIVEQ => "/=",
        PERCEQ => "%=",
        PLUSEQ => "+=",
        MINUSEQ => "-=",
        HASH => "#",
        AT => "@",
        COMMA => ",",
        _ => return kind.keyword_text(),
    };
    Some(value)
}
