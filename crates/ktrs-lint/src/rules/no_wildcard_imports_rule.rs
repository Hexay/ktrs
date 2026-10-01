//! Port of ktlint-ruleset-standard `NoWildcardImportsRule.kt`.

use std::sync::LazyLock;

use ktrs_ast::psi::KtImportDirective;
use ktrs_ast::{Ast, NodeId};
use ktrs_editorconfig::{PropertyType, PropertyValue};
use ktrs_syntax::SyntaxKind::IMPORT_DIRECTIVE;

use crate::editorconfig::{EditorConfigProperty, PropertyRef};
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::internal::importordering::PatternEntry;

const VISITED_TYPES: TokenSet = TokenSet::create(&[IMPORT_DIRECTIVE]);

pub struct NoWildcardImportsRule {
    allowed_wildcard_imports: Vec<PatternEntry>,
}

impl NoWildcardImportsRule {
    pub fn new() -> NoWildcardImportsRule {
        NoWildcardImportsRule { allowed_wildcard_imports: Vec::new() }
    }
}

impl Default for NoWildcardImportsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for NoWildcardImportsRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-wildcard-imports")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*IJ_KOTLIN_PACKAGES_TO_USE_IMPORT_ON_DEMAND)]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.allowed_wildcard_imports = editor_config.get(&IJ_KOTLIN_PACKAGES_TO_USE_IMPORT_ON_DEMAND);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == IMPORT_DIRECTIVE {
            let import_directive = KtImportDirective::of(ast, node);
            let Some(path) = import_directive.import_path(ast) else { return };
            if !path.is_all_under {
                return;
            }
            if !self.allowed_wildcard_imports.iter().any(|it| it.matches(&path)) {
                emit(ast, ast.start_offset(node), "Wildcard import", false);
            }
        }
    }
}

const WILDCARD_WITHOUT_SUBPACKAGES: &str = "*";
const WILDCARD_WITH_SUBPACKAGES: &str = "**";

// Gotcha: upstream's `onEach { it.trim() }` discards the trimmed value, so the entries keep their spaces.
fn parse_allowed_wildcard_imports(allowed_wildcard_imports: &str) -> Vec<PatternEntry> {
    allowed_wildcard_imports
        .split(',')
        .map(|import| match import.strip_suffix(WILDCARD_WITH_SUBPACKAGES) {
            // java.**
            Some(package) => PatternEntry::new(&format!("{package}{WILDCARD_WITHOUT_SUBPACKAGES}"), true, false),
            None => PatternEntry::new(import, false, false),
        })
        .collect()
}

fn packages_to_use_on_demand_import_property_parser(_name: &str, value: Option<&str>) -> PropertyValue<Vec<PatternEntry>> {
    PropertyValue::valid(value, Some(value.map(parse_allowed_wildcard_imports).unwrap_or_default()))
}

static IJ_KOTLIN_PACKAGES_TO_USE_IMPORT_ON_DEMAND_TYPE: PropertyType<Vec<PatternEntry>> = PropertyType {
    name: "ij_kotlin_packages_to_use_import_on_demand",
    description: "Defines allowed wildcard imports",
    parser: packages_to_use_on_demand_import_property_parser,
    possible_values: &[],
    lower_casing: false,
};

/// Default IntelliJ IDEA style: wildcard imports for `java.util` and `kotlinx.android.synthetic` (with its
/// subpackages); none for `ktlint_official`.
pub static IJ_KOTLIN_PACKAGES_TO_USE_IMPORT_ON_DEMAND: LazyLock<EditorConfigProperty<Vec<PatternEntry>>> =
    LazyLock::new(|| EditorConfigProperty {
        property_writer: |it| {
            if it.is_empty() {
                "unset".to_owned()
            } else {
                it.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
            }
        },
        ktlint_official_code_style_default_value: Vec::new(),
        ..EditorConfigProperty::new(
            &IJ_KOTLIN_PACKAGES_TO_USE_IMPORT_ON_DEMAND_TYPE,
            parse_allowed_wildcard_imports("java.util.*,kotlinx.android.synthetic.**"),
        )
    });
