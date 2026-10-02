//! Port of `KtlintRule.kt`: runs a [`ComposeKtVisitor`] as a ktlint rule. Each node goes to `visitFile`,
//! `visitClassOrObject` (+ `visitClass`) or `visitFunction` (+ `visitComposable` for a `@Composable` one), and
//! reports land at the reported element's name identifier when it has one, else at its start.

use ktrs_ast::psi::{self, EmbeddedKotlin, KtClass, KtClassOrObject, KtFile, KtFunction};
use ktrs_ast::{Ast, NodeId};
use ktrs_lint::editorconfig::{KtlintVersion, PropertyRef};
use ktrs_lint::{About, AutocorrectDecision, EditorConfig, Emit, RuleId, RuleV2, RuleV2Provider};
use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::*;

use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::{Decision, Emitter};
use crate::core::util::kt_annotateds::is_composable;
use crate::core::util::psi_elements::start_offset_from_name;
use crate::ktlint::editor_config_properties::ComposeProperty;
use crate::ktlint::ktlint_compose_kt_config::KtlintComposeKtConfig;

/// The node types whose PSI is a `KtFile`, `KtClassOrObject` or `KtFunction`: the only ones the hook acts on.
const VISITED_TYPES: TokenSet = TokenSet::create(&[
    FILE,
    CLASS,
    OBJECT_DECLARATION,
    ENUM_ENTRY,
    FUN,
    FUNCTION_LITERAL,
    PRIMARY_CONSTRUCTOR,
    SECONDARY_CONSTRUCTOR,
]);

pub const ABOUT: About = About {
    maintainer: "Compose Rules",
    repository_url: "https://github.com/mrmans0n/compose-rules",
    issue_tracker_url: "https://github.com/mrmans0n/compose-rules/issues",
};

/// `abstract class KtlintRule(id, editorConfigProperties) : Rule, ComposeKtVisitor`: the `*Check` classes pass
/// their visitor (upstream: `ComposeKtVisitor by Rule()` or their own overrides).
pub struct KtlintRule {
    id: &'static str,
    editor_config_properties: Vec<ComposeProperty>,
    visitor: Box<dyn ComposeKtVisitor>,
    config: Option<KtlintComposeKtConfig>,
}

impl KtlintRule {
    pub fn new(id: &'static str, editor_config_properties: Vec<ComposeProperty>, visitor: Box<dyn ComposeKtVisitor>) -> KtlintRule {
        KtlintRule { id, editor_config_properties, visitor, config: None }
    }

    /// `RuleProvider { XCheck() }`.
    pub fn provider(create: fn() -> KtlintRule) -> RuleV2Provider {
        RuleV2Provider::new(move || Box::new(create()))
    }
}

impl RuleV2 for KtlintRule {
    fn rule_id(&self) -> RuleId {
        RuleId::new(self.id)
    }

    fn about(&self) -> About {
        ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        self.editor_config_properties.iter().map(|p| p.property_ref()).collect()
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        let embedded_kotlin = match KtlintVersion::of(editor_config) {
            KtlintVersion::V1_8 => EmbeddedKotlin::V2_2_21,
            KtlintVersion::V2_0 => EmbeddedKotlin::V2_4_10,
        };
        self.config =
            Some(KtlintComposeKtConfig::new(editor_config.clone(), self.editor_config_properties.clone(), embedded_kotlin));
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let config = self.config.as_ref().expect("UninitializedPropertyAccessException: lateinit property properties");
        let mut emitter = KtlintEmitter { emit };
        if let Some(file) = KtFile::cast(ast, node) {
            self.visitor.visit_file(ast, file, &mut emitter, config);
        } else if let Some(clazz) = KtClassOrObject::cast(ast, node) {
            self.visitor.visit_class_or_object(ast, clazz, &mut emitter, config);
            if let Some(clazz) = KtClass::cast(ast, node) {
                self.visitor.visit_class(ast, clazz, &mut emitter, config);
            }
        } else if let Some(function) = KtFunction::cast(ast, node) {
            self.visitor.visit_function(ast, function, &mut emitter, config);
            if is_composable(ast, node) {
                self.visitor.visit_composable(ast, function, &mut emitter, config);
            }
        }
    }
}

/// `emit.toEmitter()`.
struct KtlintEmitter<'a, 'b> {
    emit: &'a mut Emit<'b>,
}

impl Emitter for KtlintEmitter<'_, '_> {
    fn report(&mut self, ast: &Ast, element: NodeId, error_message: &str, can_be_auto_corrected: bool) -> Decision {
        let offset = if psi::is_name_identifier_owner(ast, element) {
            start_offset_from_name(ast, element)
        } else {
            ast.start_offset(element)
        };
        match (self.emit)(ast, offset, error_message, can_be_auto_corrected) {
            AutocorrectDecision::AllowAutocorrect => Decision::Fix,
            AutocorrectDecision::NoAutocorrect => Decision::Ignore,
        }
    }
}
