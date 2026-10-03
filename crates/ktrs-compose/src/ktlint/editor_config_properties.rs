//! Port of `ktlint/EditorConfigProperties.kt`: every `compose_*` `.editorconfig` property, with its upstream name,
//! description, possible values, default and property mapper (all `LowerCasingPropertyType`s).

use std::sync::LazyLock;

use ktrs_editorconfig::PropertyType;
use ktrs_editorconfig::property_type::{boolean_value_parser, identity_value_parser, positive_int_value_parser};
use ktrs_lint::editorconfig::{EditorConfigProperty, PropertyRef};

/// A property of this file, typed, for [`crate::ktlint::ktlint_compose_kt_config`]'s lookups.
#[derive(Clone, Copy)]
pub enum ComposeProperty {
    String(&'static LazyLock<EditorConfigProperty<String>>),
    Boolean(&'static LazyLock<EditorConfigProperty<bool>>),
    Int(&'static LazyLock<EditorConfigProperty<i32>>),
}

impl ComposeProperty {
    pub fn name(self) -> &'static str {
        match self {
            ComposeProperty::String(p) => LazyLock::force(p).type_.name,
            ComposeProperty::Boolean(p) => LazyLock::force(p).type_.name,
            ComposeProperty::Int(p) => LazyLock::force(p).type_.name,
        }
    }

    pub fn property_ref(self) -> PropertyRef {
        match self {
            ComposeProperty::String(p) => PropertyRef::from(LazyLock::force(p)),
            ComposeProperty::Boolean(p) => PropertyRef::from(LazyLock::force(p)),
            ComposeProperty::Int(p) => PropertyRef::from(LazyLock::force(p)),
        }
    }
}

const BOOLEAN_VALUES: &[&str] = &["true", "false"];

/// A string property whose mapper turns `unset` into `$unset` and otherwise yields the parsed value.
macro_rules! string_property {
    ($type_:ident, $property:ident, $name:literal, $description:expr, $values:expr, $default:literal, $unset:literal) => {
        static $type_: PropertyType<String> = PropertyType {
            name: $name,
            description: $description,
            parser: identity_value_parser,
            possible_values: $values,
            lower_casing: true,
        };

        pub static $property: LazyLock<EditorConfigProperty<String>> = LazyLock::new(|| EditorConfigProperty {
            property_mapper: Some(|property, _| match property {
                Some(p) if p.is_unset() => Some($unset.to_owned()),
                Some(p) => p.get_value_as(&$type_),
                None => None,
            }),
            ..EditorConfigProperty::new(&$type_, $default.to_owned())
        });
    };
}

macro_rules! boolean_property {
    ($type_:ident, $property:ident, $name:literal, $description:expr) => {
        static $type_: PropertyType<bool> = PropertyType {
            name: $name,
            description: $description,
            parser: boolean_value_parser,
            possible_values: BOOLEAN_VALUES,
            lower_casing: true,
        };

        pub static $property: LazyLock<EditorConfigProperty<bool>> =
            LazyLock::new(|| EditorConfigProperty::new(&$type_, false));
    };
}

string_property!(
    CONTENT_EMITTERS_TYPE, CONTENT_EMITTERS_PROPERTY, "compose_content_emitters",
    "A comma separated list of composable functions that emit content (e.g. UI)", &[], "", ""
);
string_property!(
    CONTENT_EMITTERS_DENYLIST_TYPE, CONTENT_EMITTERS_DENYLIST, "compose_content_emitters_denylist",
    concat!(
        "A comma separated list of composable functions that we don't want to take into acccount ",
        "when assessing if something is a content emitter"
    ),
    &[], "", ""
);
string_property!(
    CHECK_MODIFIERS_FOR_VISIBILITY_TYPE, CHECK_MODIFIERS_FOR_VISIBILITY, "compose_check_modifiers_for_visibility",
    "Visibility of the composables where we want to check if a Modifier parameter is missing",
    &["only_public", "public_and_internal", "all"], "only_public", "only_public"
);
string_property!(
    COMPOSITION_LOCAL_ALLOWLIST_TYPE, COMPOSITION_LOCAL_ALLOWLIST_PROPERTY, "compose_allowed_composition_locals",
    "A comma separated list of allowed CompositionLocals", &[], "", ""
);
string_property!(
    ALLOWED_COMPOSE_NAMING_NAMES_TYPE, ALLOWED_COMPOSE_NAMING_NAMES, "compose_allowed_composable_function_names",
    "A comma separated list of regexes of allowed composable function names", &[], "", ""
);
string_property!(
    VIEW_MODEL_FACTORIES_TYPE, VIEW_MODEL_FACTORIES, "compose_view_model_factories",
    "A comma separated list of ViewModel factory methods", &[], "", ""
);
string_property!(
    ALLOWED_STATE_HOLDER_NAMES_TYPE, ALLOWED_STATE_HOLDER_NAMES, "compose_allowed_state_holder_names",
    "A comma separated list of regexes of valid state holders / ViewModel / Presenter names", &[], "", ""
);
string_property!(
    ALLOWED_FORWARDING_TYPE, ALLOWED_FORWARDING, "compose_allowed_forwarding",
    concat!(
        "A comma separated list of regexes of composable names where forwarding a ",
        "state holder / ViewModel / Presenter names is alright to do"
    ),
    &[], "", ""
);
string_property!(
    ALLOWED_FORWARDING_OF_TYPES_TYPE, ALLOWED_FORWARDING_OF_TYPES, "compose_allowed_forwarding_of_types",
    concat!(
        "A comma separated list of regexes of state holder/ViewModel names which are exempt from ",
        "the forwarding rule"
    ),
    &[], "", ""
);
string_property!(
    CUSTOM_MODIFIERS_TYPE, CUSTOM_MODIFIERS, "compose_custom_modifiers",
    "A comma separated list of custom Modifier implementations", &[], "", ""
);
string_property!(
    TREAT_AS_LAMBDA_TYPE, TREAT_AS_LAMBDA, "compose_treat_as_lambda",
    concat!(
        "A comma separated list of types that should be treated as lambdas ",
        "(e.g. typedefs of lambdas, fun interfaces)"
    ),
    &[], "", ""
);
string_property!(
    TREAT_AS_COMPOSABLE_LAMBDA_TYPE, TREAT_AS_COMPOSABLE_LAMBDA, "compose_treat_as_composable_lambda",
    concat!(
        "A comma separated list of types that should be treated as @Composable lambdas ",
        "(e.g. typedefs of lambdas, fun interfaces)"
    ),
    &[], "", ""
);
string_property!(
    ALLOWED_FROM_M2_TYPE, ALLOWED_FROM_M2, "compose_allowed_from_m2",
    "A comma separated list of Material 2 APIs that are allowed", &[], "", ""
);
boolean_property!(
    DISALLOW_MATERIAL2_TYPE, DISALLOW_MATERIAL2, "compose_disallow_material2",
    "When enabled, Compose Material 2 (M2) usages will be disallowed."
);
string_property!(
    ALLOWED_LAMBDA_PARAMETER_NAMES_TYPE, ALLOWED_LAMBDA_PARAMETER_NAMES, "compose_allowed_lambda_parameter_names",
    "A comma separated list of lambda name that are allowed", &[], "", ""
);
boolean_property!(
    DISALLOW_UNSTABLE_COLLECTIONS_TYPE, DISALLOW_UNSTABLE_COLLECTIONS, "compose_disallow_unstable_collections",
    "When enabled, unstable collections (e.g. List/Set/Map) will be disallowed."
);
string_property!(
    MODIFIER_MISSING_IGNORE_ANNOTATED_TYPE, MODIFIER_MISSING_IGNORE_ANNOTATED, "compose_modifier_missing_ignore_annotated",
    "A comma separated list of composable functions that should be exempt from the ModifierMissing check", &[], "", ""
);
boolean_property!(
    COMPOSE_PREVIEW_NAMING_ENABLED_TYPE, COMPOSE_PREVIEW_NAMING_ENABLED, "compose_preview_naming_enabled",
    "When enabled, preview composables should follow the configured naming strategy."
);
string_property!(
    COMPOSE_PREVIEW_NAMING_STRATEGY_TYPE, COMPOSE_PREVIEW_NAMING_STRATEGY, "compose_preview_naming_strategy",
    "The naming strategy for preview composables.", &[], "suffix", ""
);
boolean_property!(
    COMPOSABLE_NESTING_DEPTH_ENABLED_TYPE, COMPOSABLE_NESTING_DEPTH_ENABLED, "compose_composable_nesting_depth_enabled",
    "When enabled, @Composable functions that nest content emitters too deeply will be flagged."
);

static COMPOSABLE_NESTING_DEPTH_THRESHOLD_TYPE: PropertyType<i32> = PropertyType {
    name: "compose_composable_nesting_depth_threshold",
    description: "Maximum nesting depth allowed for content emitters inside a single @Composable function.",
    parser: positive_int_value_parser,
    possible_values: &[],
    lower_casing: true,
};

pub static COMPOSABLE_NESTING_DEPTH_THRESHOLD: LazyLock<EditorConfigProperty<i32>> =
    LazyLock::new(|| EditorConfigProperty::new(&COMPOSABLE_NESTING_DEPTH_THRESHOLD_TYPE, 3));
