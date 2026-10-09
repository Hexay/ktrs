//! `detekt-rules-empty-blocks`: rules that report empty blocks of code. `EmptyCodeProvider.kt` is [`provider`].

/// A rule that is nothing but an `EmptyRule` with the default description and message.
macro_rules! empty_rule {
    ($name:ident) => {
        pub struct $name {
            base: $crate::api::RuleBase,
        }

        impl $name {
            pub fn new(config: std::sync::Arc<dyn $crate::api::Config>) -> Self {
                $name { base: $crate::api::RuleBase::new(config, $crate::rules::emptyblocks::empty_rule::DESCRIPTION) }
            }
        }

        impl $crate::api::Rule for $name {
            $crate::rule_base!($name);
        }

        impl $crate::rules::emptyblocks::empty_rule::EmptyRule for $name {}
    };
}

mod contains_comments;
mod empty_catch_block;
mod empty_class_block;
mod empty_default_constructor;
mod empty_do_while_block;
mod empty_else_block;
mod empty_finally_block;
mod empty_for_block;
mod empty_function_block;
mod empty_if_block;
mod empty_init_block;
mod empty_kotlin_file;
mod empty_rule;
mod empty_secondary_constructor;
mod empty_try_block;
mod empty_when_block;
mod empty_while_block;

pub use empty_catch_block::EmptyCatchBlock;
pub use empty_class_block::EmptyClassBlock;
pub use empty_default_constructor::EmptyDefaultConstructor;
pub use empty_do_while_block::EmptyDoWhileBlock;
pub use empty_else_block::EmptyElseBlock;
pub use empty_finally_block::EmptyFinallyBlock;
pub use empty_for_block::EmptyForBlock;
pub use empty_function_block::EmptyFunctionBlock;
pub use empty_if_block::EmptyIfBlock;
pub use empty_init_block::EmptyInitBlock;
pub use empty_kotlin_file::EmptyKotlinFile;
pub use empty_secondary_constructor::EmptySecondaryConstructor;
pub use empty_try_block::EmptyTryBlock;
pub use empty_when_block::EmptyWhenBlock;
pub use empty_while_block::EmptyWhileBlock;

use crate::api::{RuleSet, RuleSetId, RuleSetProvider};

/// `EmptyCodeProvider`.
pub fn provider() -> RuleSetProvider {
    RuleSetProvider { rule_set_id: RuleSetId::new("empty-blocks"), instance, is_default: true }
}

fn instance() -> RuleSet {
    RuleSet::new(
        RuleSetId::new("empty-blocks"),
        crate::rule_providers![
            EmptyCatchBlock,
            EmptyClassBlock,
            EmptyDefaultConstructor,
            EmptyDoWhileBlock,
            EmptyElseBlock,
            EmptyFinallyBlock,
            EmptyForBlock,
            EmptyFunctionBlock,
            EmptyIfBlock,
            EmptyInitBlock,
            EmptyKotlinFile,
            EmptySecondaryConstructor,
            EmptyTryBlock,
            EmptyWhenBlock,
            EmptyWhileBlock,
        ],
    )
}
