//! `MethodSignature.kt`.

use ktrs_psi::KtFunction;

use super::kt_modifier_list::is_override;

// TODO: isEqualsFunction, isJvmFinalizeFunction, hasCorrectEqualsParameter, isMainFunction (rules not ported yet)

/// `KtFunction.isHashCodeFunction()`.
pub fn is_hash_code_function(function: &KtFunction) -> bool {
    function.name().as_deref() == Some("hashCode")
        && is_override(function)
        && function.value_parameters().is_empty()
        && function.receiver_type_reference().is_none()
}
