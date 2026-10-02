//! Port of `rules/MutableStateAutoboxing.kt`.

use std::collections::HashMap;

use ktrs_ast::psi::{KtCallExpression, KtConstantExpression, KtFile, KtFunction, KtReferenceExpression};
use ktrs_ast::{Ast, NodeId};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::kt_constant_expressions::{is_double, is_float, is_int, is_long};
use crate::core::util::psi_elements::find_all_children;

pub struct MutableStateAutoboxing;

impl ComposeKtVisitor for MutableStateAutoboxing {
    fn visit_file(&self, ast: &mut Ast, file: KtFile, emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {
        let all_mutable_state_of_with_constant_single_argument: Vec<(KtCallExpression, KtConstantExpression)> =
            find_mutable_state_of(ast, file.node())
                .into_iter()
                .filter_map(|it| Some((it, KtConstantExpression::cast(ast, single_argument_expression(ast, it)?)?)))
                .collect();
        let checks: [(ConstantPredicate, &str); 4] = [
            (is_int, MUTABLE_STATE_AUTOBOXING_INT),
            (is_long, MUTABLE_STATE_AUTOBOXING_LONG),
            (is_double, MUTABLE_STATE_AUTOBOXING_DOUBLE),
            (is_float, MUTABLE_STATE_AUTOBOXING_FLOAT),
        ];
        for (predicate, message) in checks {
            for &(item, constant_expression) in &all_mutable_state_of_with_constant_single_argument {
                if predicate(ast, constant_expression) {
                    emitter.report(ast, item.node(), message, false);
                }
            }
        }
    }

    fn visit_function(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {
        let parameter_names_and_types: HashMap<String, String> = function
            .value_parameters(ast)
            .into_iter()
            .filter_map(|it| Some((it, it.type_reference(ast)?.text(ast))))
            .filter(|(_, type_)| SUPPORTED_TYPES.contains(&type_.as_str()))
            .map(|(it, type_)| (it.name_as_safe_name(ast), type_))
            .collect();
        let primitive_candidates: Vec<(KtCallExpression, &str)> = find_mutable_state_of(ast, function.node())
            .into_iter()
            .filter_map(|it| {
                let argument = single_argument_expression(ast, it).filter(|&a| KtReferenceExpression::is(ast, a))?;
                Some((it, parameter_names_and_types.get(&ast.text(argument))?.as_str()))
            })
            .collect();
        for (message, types) in PARAMETER_TYPE_MESSAGES {
            for &(candidate, type_) in &primitive_candidates {
                if types.contains(&type_) {
                    emitter.report(ast, candidate.node(), message, false);
                }
            }
        }
    }
}

type ConstantPredicate = fn(&Ast, KtConstantExpression) -> bool;

fn single_argument_expression(ast: &Ast, call: KtCallExpression) -> Option<NodeId> {
    match call.value_arguments(ast).as_slice() {
        [single] => single.argument_expression(ast),
        _ => None,
    }
}

fn find_mutable_state_of(ast: &Ast, element: NodeId) -> Vec<KtCallExpression> {
    find_all_children::<KtCallExpression>(ast, element)
        .into_iter()
        .filter(|it| it.callee_expression(ast).is_some_and(|c| ast.text(c) == "mutableStateOf"))
        .filter(|it| it.value_arguments(ast).len() == 1)
        .collect()
}

/// `emitIfParameterTypeIs(message) { it == ... }`, in upstream's order.
const PARAMETER_TYPE_MESSAGES: &[(&str, &[&str])] = &[
    (MUTABLE_STATE_AUTOBOXING_INT, &["Int"]),
    (MUTABLE_STATE_AUTOBOXING_LONG, &["Long"]),
    (MUTABLE_STATE_AUTOBOXING_DOUBLE, &["Double"]),
    (MUTABLE_STATE_AUTOBOXING_FLOAT, &["Float"]),
    (MUTABLE_STATE_AUTOBOXING_INT_LIST, &["List<Int>", "PersistentList<Int>", "ImmutableList<Int>"]),
    (MUTABLE_STATE_AUTOBOXING_LONG_LIST, &["List<Long>", "PersistentList<Long>", "ImmutableList<Long>"]),
    (MUTABLE_STATE_AUTOBOXING_FLOAT_LIST, &["List<Float>", "PersistentList<Float>", "ImmutableList<Float>"]),
    (MUTABLE_STATE_AUTOBOXING_INT_SET, &["Set<Int>", "PersistentSet<Int>", "ImmutableSet<Int>"]),
    (MUTABLE_STATE_AUTOBOXING_LONG_SET, &["Set<Long>", "PersistentSet<Long>", "ImmutableSet<Long>"]),
    (MUTABLE_STATE_AUTOBOXING_FLOAT_SET, &["Set<Float>", "PersistentSet<Float>", "ImmutableSet<Float>"]),
    (MUTABLE_STATE_AUTOBOXING_INT_INT_MAP, &["Map<Int, Int>", "PersistentMap<Int, Int>", "ImmutableMap<Int, Int>"]),
    (MUTABLE_STATE_AUTOBOXING_INT_LONG_MAP, &["Map<Int, Long>", "PersistentMap<Int, Long>", "ImmutableMap<Int, Long>"]),
    (MUTABLE_STATE_AUTOBOXING_INT_FLOAT_MAP, &["Map<Int, Float>", "PersistentMap<Int, Float>", "ImmutableMap<Int, Float>"]),
    (MUTABLE_STATE_AUTOBOXING_LONG_INT_MAP, &["Map<Long, Int>", "PersistentMap<Long, Int>", "ImmutableMap<Long, Int>"]),
    (MUTABLE_STATE_AUTOBOXING_LONG_LONG_MAP, &["Map<Long, Long>", "PersistentMap<Long, Long>", "ImmutableMap<Long, Long>"]),
    (MUTABLE_STATE_AUTOBOXING_LONG_FLOAT_MAP, &["Map<Long, Float>", "PersistentMap<Long, Float>", "ImmutableMap<Long, Float>"]),
    (MUTABLE_STATE_AUTOBOXING_FLOAT_INT_MAP, &["Map<Float, Int>", "PersistentMap<Float, Int>", "ImmutableMap<Float, Int>"]),
    (MUTABLE_STATE_AUTOBOXING_FLOAT_LONG_MAP, &["Map<Float, Long>", "PersistentMap<Float, Long>", "ImmutableMap<Float, Long>"]),
    (MUTABLE_STATE_AUTOBOXING_FLOAT_FLOAT_MAP, &["Map<Float, Float>", "PersistentMap<Float, Float>", "ImmutableMap<Float, Float>"]),
];

const SUPPORTED_TYPES: &[&str] = &[
    "Int",
    "Long",
    "Float",
    "Double",
    "List<Int>",
    "PersistentList<Int>",
    "ImmutableList<Int>",
    "List<Long>",
    "PersistentList<Long>",
    "ImmutableList<Long>",
    "List<Float>",
    "PersistentList<Float>",
    "ImmutableList<Float>",
    "Set<Int>",
    "PersistentSet<Int>",
    "ImmutableSet<Int>",
    "Set<Long>",
    "PersistentSet<Long>",
    "ImmutableSet<Long>",
    "Set<Float>",
    "PersistentSet<Float>",
    "ImmutableSet<Float>",
    "Map<Int, Int>",
    "PersistentMap<Int, Int>",
    "ImmutableMap<Int, Int>",
    "Map<Int, Long>",
    "PersistentMap<Int, Long>",
    "ImmutableMap<Int, Long>",
    "Map<Int, Float>",
    "PersistentMap<Int, Float>",
    "ImmutableMap<Int, Float>",
    "Map<Long, Int>",
    "PersistentMap<Long, Int>",
    "ImmutableMap<Long, Int>",
    "Map<Long, Long>",
    "PersistentMap<Long, Long>",
    "ImmutableMap<Long, Long>",
    "Map<Long, Float>",
    "PersistentMap<Long, Float>",
    "ImmutableMap<Long, Float>",
    "Map<Float, Int>",
    "PersistentMap<Float, Int>",
    "ImmutableMap<Float, Int>",
    "Map<Float, Long>",
    "PersistentMap<Float, Long>",
    "ImmutableMap<Float, Long>",
    "Map<Float, Float>",
    "PersistentMap<Float, Float>",
    "ImmutableMap<Float, Float>",
];

macro_rules! message {
    ($($name:ident = $head:literal;)*) => {$(
        pub const $name: &str = concat!(
            $head,
            "\nSee https://mrmans0n.github.io/compose-rules/rules/#use-mutablestateof-type-specific-variants-when-possible for more information."
        );
    )*};
}

message! {
    MUTABLE_STATE_AUTOBOXING_INT = "Using mutableIntStateOf is recommended over mutableStateOf<Int>, as it uses the Int primitive directly which is more performant.";
    MUTABLE_STATE_AUTOBOXING_LONG = "Using mutableLongStateOf is recommended over mutableStateOf<Long>, as it uses the Long primitive directly which is more performant.";
    MUTABLE_STATE_AUTOBOXING_DOUBLE = "Using mutableDoubleStateOf is recommended over mutableStateOf<Double>, as it uses the Double primitive directly which is more performant.";
    MUTABLE_STATE_AUTOBOXING_FLOAT = "Using mutableFloatStateOf is recommended over mutableStateOf<Float>, as it uses the Float primitive directly which is more performant.";
    MUTABLE_STATE_AUTOBOXING_INT_LIST = "Using mutableIntListOf is recommended over mutableStateOf Immutable/Persistent/List<Int> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_LONG_LIST = "Using mutableLongListOf is recommended over mutableStateOf Immutable/Persistent/List<Long> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_FLOAT_LIST = "Using mutableFloatListOf is recommended over mutableStateOf Immutable/Persistent/List<Float> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_INT_SET = "Using mutableIntSetOf is recommended over mutableStateOf Immutable/Persistent/Set<Int> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_LONG_SET = "Using mutableLongSetOf is recommended over mutableStateOf Immutable/Persistent/Set<Long> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_FLOAT_SET = "Using mutableFloatSetOf is recommended over mutableStateOf Immutable/Persistent/Set<Float> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_INT_INT_MAP = "Using mutableIntIntMapOf is recommended over mutableStateOf Immutable/Persistent/Map<Int, Int> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_INT_LONG_MAP = "Using mutableIntLongMapOf is recommended over mutableStateOf Immutable/Persistent/Map<Int, Long> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_INT_FLOAT_MAP = "Using mutableIntFloatMapOf is recommended over mutableStateOf Immutable/Persistent/Map<Int, Float> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_LONG_INT_MAP = "Using mutableLongIntMapOf is recommended over mutableStateOf Immutable/Persistent/Map<Long, Int> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_LONG_LONG_MAP = "Using mutableLongLongMapOf is recommended over mutableStateOf Immutable/Persistent/Map<Long, Long> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_LONG_FLOAT_MAP = "Using mutableLongFloatMapOf is recommended over mutableStateOf Immutable/Persistent/Map<Long, Float> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_FLOAT_INT_MAP = "Using mutableFloatIntMapOf is recommended over mutableStateOf Immutable/Persistent/Map<Float, Int> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_FLOAT_LONG_MAP = "Using mutableFloatLongMapOf is recommended over mutableStateOf Immutable/Persistent/Map<Float, Long> due to its better performance.";
    MUTABLE_STATE_AUTOBOXING_FLOAT_FLOAT_MAP = "Using mutableFloatFloatMapOf is recommended over mutableStateOf Immutable/Persistent/Map<Float, Float> due to its better performance.";
}
