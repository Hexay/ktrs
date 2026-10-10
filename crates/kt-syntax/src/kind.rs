use std::fmt;

use ktrs_syntax::SyntaxKind as Raw;

use crate::generated::kinds::{BY_NAME, NAMES};

/// The kind of a node or token: one of the Kotlin compiler's PSI element types.
///
/// Kinds are identified by **name**, which is the compiler's field name (`KtNodeTypes.FUN` is
/// [`SyntaxKind::FUN`], `KtTokens.FUN_KEYWORD` is [`SyntaxKind::FUN_KEYWORD`]; KDoc tokens carry a `KDOC_`
/// prefix). Every kind is an associated constant, so it can be used in patterns:
///
/// ```
/// use kt_syntax::SyntaxKind;
///
/// fn is_declaration(kind: SyntaxKind) -> bool {
///     matches!(kind, SyntaxKind::FUN | SyntaxKind::PROPERTY | SyntaxKind::CLASS)
/// }
/// assert!(is_declaration(SyntaxKind::from_name("FUN").unwrap()));
/// ```
///
/// The type is opaque on purpose: a release that follows a newer Kotlin can add kinds, so a `match` always
/// needs a wildcard arm, and no number is exposed. Store and exchange [`name`](SyntaxKind::name)s.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SyntaxKind(pub(crate) Raw);

impl SyntaxKind {
    /// The stable name, e.g. `"FUN"`, `"FUN_KEYWORD"`, `"KDOC_TAG"`.
    pub fn name(self) -> &'static str {
        NAMES[self.0 as usize]
    }

    /// The kind called `name`, if this version has one.
    pub fn from_name(name: &str) -> Option<SyntaxKind> {
        BY_NAME.binary_search_by(|(n, _)| (*n).cmp(name)).ok().map(|i| BY_NAME[i].1)
    }

    /// Every kind of this version, sorted by name.
    pub fn all() -> impl Iterator<Item = SyntaxKind> {
        BY_NAME.iter().map(|&(_, kind)| kind)
    }

    /// What the compiler's `DebugUtil.psiToString` prints for this kind (`fun` for `FUN_KEYWORD`, `KDoc` for
    /// `DOC_COMMENT`); the spelling used by [`SourceFile::dump`](crate::SourceFile::dump).
    pub fn dump_name(self) -> &'static str {
        self.0.debug_name()
    }

    /// The source text of a keyword kind (`"fun"` for `FUN_KEYWORD`), for hard, soft and modifier keywords.
    pub fn keyword_text(self) -> Option<&'static str> {
        self.0.keyword_text()
    }

    /// Whether this is a keyword kind.
    pub fn is_keyword(self) -> bool {
        self.0.keyword_text().is_some()
    }

    /// White space or a comment (KDoc included).
    pub fn is_trivia(self) -> bool {
        self.0.is_trivia()
    }

    /// `EOL_COMMENT`, `BLOCK_COMMENT`, `SHEBANG_COMMENT` or `DOC_COMMENT` (KDoc).
    pub fn is_comment(self) -> bool {
        self.0.is_trivia() && self.0 != Raw::WHITE_SPACE
    }
}

impl fmt::Debug for SyntaxKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl fmt::Display for SyntaxKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip_for_every_kind() {
        let mut count = 0;
        for kind in SyntaxKind::all() {
            assert_eq!(SyntaxKind::from_name(kind.name()), Some(kind));
            assert_eq!(format!("{kind}"), kind.name());
            assert_eq!(format!("{kind:?}"), format!("{:?}", kind.0));
            count += 1;
        }
        assert_eq!(count, NAMES.len());
        assert_eq!(SyntaxKind::from_name("NOT_A_KIND"), None);
        assert_eq!(SyntaxKind::from_name("fun"), None);
    }

    #[test]
    fn names_are_the_compiler_field_names() {
        assert_eq!(SyntaxKind::FUN.name(), "FUN");
        assert_eq!(SyntaxKind::FUN_KEYWORD.name(), "FUN_KEYWORD");
        assert_eq!(SyntaxKind::FUN_KEYWORD.dump_name(), "fun");
        assert_eq!(SyntaxKind::DOC_COMMENT.dump_name(), "KDoc");
        assert_eq!(SyntaxKind::FUN_KEYWORD.keyword_text(), Some("fun"));
        assert!(SyntaxKind::FUN_KEYWORD.is_keyword());
        assert!(!SyntaxKind::IDENTIFIER.is_keyword());
    }

    #[test]
    fn trivia_and_comments() {
        assert!(SyntaxKind::WHITE_SPACE.is_trivia());
        assert!(!SyntaxKind::WHITE_SPACE.is_comment());
        for kind in [SyntaxKind::EOL_COMMENT, SyntaxKind::BLOCK_COMMENT, SyntaxKind::SHEBANG_COMMENT, SyntaxKind::DOC_COMMENT] {
            assert!(kind.is_trivia() && kind.is_comment(), "{kind}");
        }
        assert!(!SyntaxKind::IDENTIFIER.is_trivia());
    }

    #[test]
    fn kinds_work_as_patterns() {
        let describe = |kind: SyntaxKind| match kind {
            SyntaxKind::FUN => "function",
            SyntaxKind::CLASS => "class",
            _ => "other",
        };
        assert_eq!(describe(SyntaxKind::FUN), "function");
        assert_eq!(describe(SyntaxKind::PROPERTY), "other");
    }
}
