//! `AllowedExceptionNamePattern.kt`.

use ktrs_psi::KtCatchClause;

use crate::kotlin::Regex;

/// `KtCatchClause.isAllowedExceptionName(regex)`.
pub fn is_allowed_exception_name(catch_clause: &KtCatchClause, regex: &Regex) -> bool {
    catch_clause.catch_parameter().and_then(|parameter| parameter.name()).is_some_and(|name| regex.matches(&name))
}
