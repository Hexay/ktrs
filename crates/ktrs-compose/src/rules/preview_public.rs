//! Port of `rules/PreviewPublic.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFunction;
use ktrs_syntax::SyntaxKind::FUN_KEYWORD;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::ast_nodes::first_child_leaf_or_self;
use crate::core::util::previews::is_preview;

pub struct PreviewPublic;

impl ComposeKtVisitor for PreviewPublic {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {
        if !is_preview(ast, function.node()) {
            return;
        }
        if !function.is_public(ast) {
            return;
        }
        emitter.report(ast, function.node(), COMPOSABLES_PREVIEW_SHOULD_NOT_BE_PUBLIC, true).if_fix(|| {
            // Gotcha: upstream leaves one FUN_KEYWORD leaf with the text "private fun".
            let Some(node) = ast.find_child_by_type(function.node(), FUN_KEYWORD).map(|it| first_child_leaf_or_self(ast, it)) else {
                return;
            };
            if !ast.is_leaf_element(node) {
                return;
            }
            ast.raw_replace_with_text(node, "private fun");
        });
    }
}

pub const COMPOSABLES_PREVIEW_SHOULD_NOT_BE_PUBLIC: &str = "\
Composables annotated with @Preview that are used only for previewing the UI should not be public.
See https://mrmans0n.github.io/compose-rules/rules/#preview-composables-should-not-be-public for more information.";
