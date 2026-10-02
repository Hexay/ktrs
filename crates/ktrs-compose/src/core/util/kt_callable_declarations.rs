//! Port of `core/util/KtCallableDeclarations.kt`.

use std::sync::LazyLock;

use ktrs_ast::psi::{KtCallableDeclaration, KtParameter};
use ktrs_ast::{Ast, NodeId};
use ktrs_lint::rules::internal::KotlinRegex;

use crate::core::util::kotlin_utils::matches_any_of;
use crate::core::util::lambdas::is_composable_ui_emitter_lambda;

fn type_reference_text(ast: &Ast, callable: NodeId) -> Option<String> {
    KtCallableDeclaration::of(ast, callable).type_reference(ast).map(|t| t.text(ast))
}

/// `KtCallableDeclaration.isTypeMutable`.
pub fn is_type_mutable(ast: &Ast, callable: NodeId) -> bool {
    matches_any_of(type_reference_text(ast, callable).as_deref(), &KNOWN_MUTABLE_COMMON_TYPES_REGEX)
}

static KNOWN_MUTABLE_COMMON_TYPES_REGEX: LazyLock<Vec<KotlinRegex>> = LazyLock::new(|| {
    [
        "MutableSet<.*>\\??",
        "ArraySet<.*>\\??",
        "HashSet<.*>\\??",
        "MutableList<.*>\\??",
        "ArrayList<.*>\\??",
        "SparseArray<.*>\\??",
        "SparseArrayCompat<.*>\\??",
        "LongSparseArray<.*>\\??",
        "SparseBooleanArray\\??",
        "SparseIntArray\\??",
        "MutableMap<.*>\\??",
        "HashMap<.*>\\??",
        "Hashtable<.*>\\??",
        "MutableStateFlow<.*>\\??",
        "MutableSharedFlow<.*>\\??",
        "PublishSubject<.*>\\??",
        "BehaviorSubject<.*>\\??",
        "ReplaySubject<.*>\\??",
        "PublishRelay<.*>\\??",
        "BehaviorRelay<.*>\\??",
        "ReplayRelay<.*>\\??",
    ]
    .into_iter()
    .map(KotlinRegex::new)
    .collect()
});

/// `KtCallableDeclaration.isTypeUnstableCollection`.
pub fn is_type_unstable_collection(ast: &Ast, callable: NodeId) -> bool {
    matches_any_of(type_reference_text(ast, callable).as_deref(), &KNOWN_UNSTABLE_COLLECTION_TYPES_REGEX)
}

pub static KNOWN_UNSTABLE_COLLECTION_TYPES_REGEX: LazyLock<Vec<KotlinRegex>> =
    LazyLock::new(|| ["Set<.*>\\??", "List<.*>\\??", "Map<.*>\\??"].into_iter().map(KotlinRegex::new).collect());

/// `KtCallableDeclaration.contentSlots(treatAsLambdaTypes, treatAsComposableLambdaTypes)`.
pub fn content_slots(
    ast: &Ast,
    callable: NodeId,
    treat_as_lambda_types: &[String],
    treat_as_composable_lambda_types: &[String],
) -> Vec<KtParameter> {
    KtCallableDeclaration::of(ast, callable)
        .value_parameters(ast)
        .into_iter()
        .filter(|parameter| {
            parameter.type_reference(ast).is_some_and(|t| {
                is_composable_ui_emitter_lambda(ast, t, treat_as_lambda_types, treat_as_composable_lambda_types)
            })
        })
        .collect()
}
