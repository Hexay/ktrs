//! Port of `rules/CompositionLocalAllowlist.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtFile, KtProperty};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::composables::declares_composition_local;
use crate::core::util::psi_elements::find_all_children;

pub struct CompositionLocalAllowlist;

impl ComposeKtVisitor for CompositionLocalAllowlist {
    fn visit_file(&self, ast: &mut Ast, file: KtFile, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let composition_locals: Vec<KtProperty> = find_all_children::<KtProperty>(ast, file.node())
            .into_iter()
            .filter(|it| declares_composition_local(ast, *it))
            .collect();
        if composition_locals.is_empty() {
            return;
        }
        let allowed = config.get_set("allowedCompositionLocals", &[]);
        let not_allowed: Vec<KtProperty> = composition_locals
            .into_iter()
            .filter(|it| !it.name_identifier(ast).is_some_and(|n| allowed.contains(&ast.text(n))))
            .collect();
        for composition_local in not_allowed {
            emitter.report(ast, composition_local.node(), COMPOSITION_LOCAL_NOT_IN_ALLOWLIST, false);
        }
    }
}

pub const COMPOSITION_LOCAL_NOT_IN_ALLOWLIST: &str = "\
CompositionLocals are implicit dependencies and creating new ones should be avoided.
See https://mrmans0n.github.io/compose-rules/rules/#compositionlocals for more information.";
