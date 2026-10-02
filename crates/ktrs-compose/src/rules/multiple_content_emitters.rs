//! Port of `rules/MultipleContentEmitters.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtFile, KtFunction, KtNamedFunction};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::composables::{create_direct_composable_to_emission_count_mapping, refine_composable_to_emission_count_mapping};
use crate::core::util::kt_annotateds::is_composable;
use crate::core::util::kt_functions::{has_any_context_arguments, has_receiver_type};
use crate::core::util::psi_elements::find_all_children;

pub struct MultipleContentEmitters;

impl ComposeKtVisitor for MultipleContentEmitters {
    fn visit_file(&self, ast: &mut Ast, file: KtFile, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let kotlin = config.embedded_kotlin();
        let composables: Vec<KtFunction> = find_all_children::<KtNamedFunction>(ast, file.node())
            .into_iter()
            .filter(|it| is_composable(ast, it.node()))
            .filter(|it| it.has_block_body(ast))
            .filter(|it| !has_any_context_arguments(ast, *it, kotlin))
            .map(|it| KtFunction::of(ast, it.node()))
            .filter(|it| !has_receiver_type(ast, *it))
            .collect();
        let composable_to_emission_count = create_direct_composable_to_emission_count_mapping(ast, &composables, config);
        let direct_emissions_reported: Vec<KtFunction> =
            composable_to_emission_count.iter().filter(|(_, count)| *count > 1).map(|&(function, _)| function).collect();
        for composable in &direct_emissions_reported {
            emitter.report(ast, composable.node(), MULTIPLE_CONTENT_EMITTERS_DETECTED, false);
        }
        let current_mapping = refine_composable_to_emission_count_mapping(ast, composable_to_emission_count, config);
        for (composable, _) in current_mapping
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .filter(|(function, _)| !direct_emissions_reported.contains(function))
        {
            emitter.report(ast, composable.node(), MULTIPLE_CONTENT_EMITTERS_DETECTED, false);
        }
    }
}

pub const MULTIPLE_CONTENT_EMITTERS_DETECTED: &str = "\
Composable functions should only be emitting content into the composition from one source at their top level.
See https://mrmans0n.github.io/compose-rules/rules/#do-not-emit-multiple-pieces-of-content for more information.";
