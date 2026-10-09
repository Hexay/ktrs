//! Port of the ktlint 2.0.0-ALPHA-4 rule engine (`ktlint-rule-engine`, `ktlint-rule-engine-core`) and of
//! standard rules, over the mutable [`ktrs_ast::Ast`]. Scope and status: research/15-ktlint-spike.md.
//!
//! # Porting conventions (Kotlin -> Rust)
//! - A rule is a struct implementing [`RuleV2`]; its body is the upstream one line for line. `node.foo`
//!   ASTNode members and `ASTNodeExtension.kt` helpers are methods on the `Ast` (`ast.prev_leaf(node)`,
//!   via `use crate::ast_node_extension::*` and [`AstNodeEdit`]); `?.` chains become `Option` combinators.
//!   `ElementType.X` -> [`element_type`], `TokenSets` -> [`token_sets`], `node.psi as KtX` -> `ktrs_ast::psi`.
//!   Upstream symbol -> Rust name, with status: research/16-ktlint-api-coverage.md.
//! - `emit(offset, message, canBeAutoCorrected)` -> `emit(ast, offset, message, can_be_auto_corrected)`
//!   with the UTF-8 offset of the current tree; the engine converts it to UTF-16 and maps it through the
//!   line table of the *original* text, as ktlint does (stale after earlier edits by design).
//! - A rule reads `.editorconfig` values as `editor_config.get(&INDENT_SIZE_PROPERTY)` in
//!   `before_first_node`, for the properties it lists in `uses_editor_config_properties`
//!   ([`editorconfig`]); marker interfaces (`Experimental`, ...) are `is_*` methods of [`RuleV2`].
//! - A Kotlin exception (NPE on `!!`, a failed cast) is a panic whose message starts with the exception
//!   name; the engine turns a rule's panic into a [`KtLintRuleException`] (see `engine/rule_panic.rs`).

pub mod ast_node_edit;
pub mod ast_node_extension;
pub mod editorconfig;
pub mod element_type;
pub mod engine;
pub mod indent_config;
pub mod rule;
pub mod rule_provider;
pub mod rules;
pub mod token_sets;

pub use ast_node_edit::AstNodeEdit;
pub use ast_node_extension::{AstNodeExtension, AstNodeLines, AstNodeQueries};
pub use engine::code::{
    Code, KtLintException, KtLintParseException, KtLintRuleException, LintError,
};
pub use engine::editor_config_defaults::{EditorConfigDefaults, EditorConfigOverride};
pub use engine::ktlint_rule_engine::KtLintRuleEngine;
pub use rule::{
    About, AutocorrectDecision, EditorConfig, Emit, RuleId, RuleSetId, RuleV2, RunAfterRuleMode, TraversalState,
    VisitorModifier,
};
pub use rule_provider::RuleV2Provider;
