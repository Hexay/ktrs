//! Port of `core/ComposeKtVisitor.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtClass, KtClassOrObject, KtFile, KtFunction};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::emitter::Emitter;

/// `ComposeKtVisitor`: every hook defaults to doing nothing. The tree is `&mut` because a fix edits it inside
/// `Decision::if_fix`; read-only code passes it on as `&Ast`.
pub trait ComposeKtVisitor {
    fn is_opt_in(&self) -> bool {
        false
    }

    fn visit_function(&self, _ast: &mut Ast, _function: KtFunction, _emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {}

    fn visit_composable(&self, _ast: &mut Ast, _function: KtFunction, _emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {}

    fn visit_class(&self, _ast: &mut Ast, _clazz: KtClass, _emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {}

    fn visit_class_or_object(
        &self,
        _ast: &mut Ast,
        _clazz: KtClassOrObject,
        _emitter: &mut dyn Emitter,
        _config: &dyn ComposeKtConfig,
    ) {
    }

    fn visit_file(&self, _ast: &mut Ast, _file: KtFile, _emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {}
}
