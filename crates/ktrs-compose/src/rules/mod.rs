//! Port of compose-rules `rules/common/.../rules`: one module per rule (the unported ones are empty until their
//! port lands, so porting a rule never edits this file).

pub mod composable_annotation_naming;
pub mod composable_nesting_depth;
pub mod composition_local_allowlist;
pub mod composition_local_naming;
pub mod content_emitter_returning_values;
pub mod content_slot_reused;
pub mod content_trailing_lambda;
pub mod defaults_visibility;
pub mod lambda_parameter_event_trailing;
pub mod lambda_parameter_in_restartable_effect;
pub mod material2;
pub mod modifier_clickable_order;
pub mod modifier_composed;
pub mod modifier_missing;
pub mod modifier_naming;
pub mod modifier_not_used_at_root;
pub mod modifier_reused;
pub mod modifier_without_default;
pub mod multiple_content_emitters;
pub mod mutable_parameters;
pub mod mutable_state_autoboxing;
pub mod mutable_state_parameter;
pub mod naming;
pub mod parameter_naming;
pub mod parameter_order;
pub mod preview_annotation_naming;
pub mod preview_naming;
pub mod preview_public;
pub mod remember_content_missing;
pub mod remember_state_missing;
pub mod state_parameter;
pub mod unstable_collections;
pub mod view_model_forwarding;
pub mod view_model_injection;
