//! Port of ktlint-rule-engine-core `ElementType.kt`: `ElementType.X` is `SyntaxKind::X` (the compiler's
//! field name) for all but the four names below, which ktlint spells differently.

use ktrs_syntax::SyntaxKind;

pub use ktrs_syntax::SyntaxKind::*;

/// `KtNodeTypes.WHEN_CONDITION_EXPRESSION`.
pub const WHEN_CONDITION_WITH_EXPRESSION: SyntaxKind = SyntaxKind::WHEN_CONDITION_EXPRESSION;
/// `KtTokens.TYPE_ALIAS_KEYWORD`.
pub const TYPEALIAS_KEYWORD: SyntaxKind = SyntaxKind::TYPE_ALIAS_KEYWORD;
pub const DEFAULT_VISIBILITY_KEYWORD: SyntaxKind = SyntaxKind::PUBLIC_KEYWORD;
/// `KDocTokens.KDOC` (= `KtTokens.DOC_COMMENT`).
pub const KDOC: SyntaxKind = SyntaxKind::DOC_COMMENT;
