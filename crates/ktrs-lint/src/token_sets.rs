//! Port of ktlint-rule-engine-core `TokenSets.kt`. `KtTokens` sets are in `ktrs_parser::kt_tokens`,
//! `KtTokenSets` ones in `ktrs_ast::psi`.

use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::*;

pub use ktrs_parser::kt_tokens::COMMENTS;

/// A subset of `KotlinExpressionParsing.EXPRESSION_FIRST`.
pub const CONTROL_FLOW_KEYWORDS: TokenSet =
    TokenSet::create(&[IF_KEYWORD, WHEN_KEYWORD, TRY_KEYWORD, OBJECT_KEYWORD, FOR_KEYWORD, WHILE_KEYWORD, DO_KEYWORD]);
