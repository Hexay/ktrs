//! Port of the ktlint 2.0.0-ALPHA-4 rule engine (`ktlint-rule-engine`, `ktlint-rule-engine-core`) and of
//! standard rules, over the mutable [`ktrs_ast::Ast`]. Scope and status: research/15-ktlint-spike.md.
//!
//! # Porting conventions (Kotlin -> Rust)
//! - A rule is a struct implementing [`RuleV2`]; its body is the upstream one line for line. `node.foo`
//!   ASTNode members and `ASTNodeExtension.kt` helpers are methods on the `Ast` (`ast.prev_leaf(node)`,
//!   via [`AstNodeExtension`]); `?.` chains become `Option` combinators.
//! - `emit(offset, message, canBeAutoCorrected)` -> `emit(ast, offset, message, can_be_auto_corrected)`
//!   with the UTF-8 offset of the current tree; the engine converts it to UTF-16 and maps it through the
//!   line table of the *original* text, as ktlint does (stale after earlier edits by design).
//! - A Kotlin exception (NPE on `!!`, a failed cast) is a panic whose message starts with the exception name.

pub mod ast_node_edit;
pub mod ast_node_extension;
pub mod engine;
pub mod indent_config;
pub mod rule;
pub mod rules;

pub use ast_node_edit::AstNodeEdit;
pub use ast_node_extension::AstNodeExtension;
pub use engine::ktlint_rule_engine::{Code, KtLintParseException, KtLintRuleEngine, LintError};
pub use rule::{AutocorrectDecision, EditorConfig, Emit, RuleId, RuleV2, RuleV2Provider};
