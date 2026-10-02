//! Port of `rules/ContentEmitterReturningValues.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtFile, KtFunction};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::composables::{create_direct_composable_to_emission_count_mapping, refine_composable_to_emission_count_mapping};
use crate::core::util::kt_annotateds::is_composable;
use crate::core::util::kt_functions::{has_receiver_type, returns_value};
use crate::core::util::psi_elements::find_all_children;

pub struct ContentEmitterReturningValues;

impl ComposeKtVisitor for ContentEmitterReturningValues {
    fn visit_file(&self, ast: &mut Ast, file: KtFile, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let composables: Vec<KtFunction> = find_all_children::<KtFunction>(ast, file.node())
            .into_iter()
            .filter(|it| is_composable(ast, it.node()))
            .filter(|it| it.has_block_body(ast))
            .filter(|it| !has_receiver_type(ast, *it))
            .collect();
        let composable_to_emission_count = create_direct_composable_to_emission_count_mapping(ast, &composables, config);
        let mapping = refine_composable_to_emission_count_mapping(ast, composable_to_emission_count, config);
        for (composable, _) in mapping.into_iter().filter(|(_, count)| *count > 0).filter(|(it, _)| returns_value(ast, *it)) {
            emitter.report(ast, composable.node(), CONTENT_EMITTER_RETURNING_VALUES_TOO, false);
        }
    }
}

pub const CONTENT_EMITTER_RETURNING_VALUES_TOO: &str = "\
Composable functions should either emit content into the composition or return a value, but not both.
If a composable should offer additional control surfaces to its caller, those control surfaces or callbacks should be provided as parameters to the composable function by the caller.
See https://mrmans0n.github.io/compose-rules/rules/#do-not-emit-content-and-return-a-result for more information.";
