//! Port of `rules/CompositionLocalNaming.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtFile, KtProperty};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::composables::declares_composition_local;
use crate::core::util::psi_elements::find_all_children;

pub struct CompositionLocalNaming;

impl ComposeKtVisitor for CompositionLocalNaming {
    fn visit_file(&self, ast: &mut Ast, file: KtFile, emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {
        let composition_locals: Vec<KtProperty> = find_all_children::<KtProperty>(ast, file.node())
            .into_iter()
            .filter(|it| declares_composition_local(ast, *it))
            .collect();
        if composition_locals.is_empty() {
            return;
        }
        let not_allowed = composition_locals
            .into_iter()
            .filter(|it| !it.name_identifier(ast).is_some_and(|n| ast.text(n).starts_with("Local")));
        for composition_local in not_allowed.collect::<Vec<_>>() {
            emitter.report(ast, composition_local.node(), COMPOSITION_LOCAL_NEEDS_LOCAL_PREFIX, false);
        }
    }
}

pub const COMPOSITION_LOCAL_NEEDS_LOCAL_PREFIX: &str = "\
CompositionLocals should be named using the `Local` prefix as an adjective, followed by a descriptive noun.
See https://mrmans0n.github.io/compose-rules/rules/#naming-compositionlocals-properly for more information.";
