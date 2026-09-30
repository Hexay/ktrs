//! Port of ktlint-rule-engine (`api/` and `internal/`).

mod ast_helpers;
pub mod code;
mod code_formatter;
mod editor_config_cache;
pub mod editor_config_defaults;
mod editor_config_generator;
pub mod editor_config_loader;
mod formatter_tags;
pub mod internal_rules;
pub(crate) mod kotlin_text;
pub mod ktlint_rule_engine;
mod ktlint_rule_engine_suppression;
mod ktlint_suppression;
mod ktlint_suppression_annotation;
mod position_in_text_locator;
mod rule_execution_context;
pub mod rule_filter;
mod suppression_locator;
pub mod visitor_provider;

pub use editor_config_cache::{THREAD_SAFE_EDITOR_CONFIG_CACHE, ThreadSafeEditorConfigCache};
pub use formatter_tags::{
    FORMATTER_TAG_OFF_ENABLED_PROPERTY, FORMATTER_TAG_ON_ENABLED_PROPERTY,
    FORMATTER_TAGS_ENABLED_PROPERTY, FormatterTags,
};
pub use ktlint_rule_engine_suppression::{
    EditorConfigPropertyRegistry, KtlintSuppression, KtlintSuppressionException,
};
pub use position_in_text_locator::PositionInTextLocator;
pub use rule_execution_context::{EmitAndApprove, execute_rules};
pub use suppression_locator::SuppressionLocator;
