//! Port of `rules/DefaultsVisibility.kt`.

use ktrs_ast::psi::{KtClassOrObject, KtFunction, KtModifierListOwner, KtReferenceExpression, containing_kt_file};
use ktrs_ast::{Ast, NodeId};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::kt_annotateds::is_composable;
use crate::core::util::kt_functions::{is_internal, is_private, is_protected};
use crate::core::util::psi_elements::find_all_children;

pub struct DefaultsVisibility;

impl ComposeKtVisitor for DefaultsVisibility {
    fn visit_class_or_object(&self, ast: &mut Ast, clazz: KtClassOrObject, emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {
        let Some(default_object_name) = clazz.name(ast) else { return };
        let Some(composable_name) = default_object_name.strip_suffix("Defaults") else { return };
        if composable_name.is_empty() {
            return;
        }
        let file = containing_kt_file(ast, clazz.node()).expect("containingKtFile");
        let references_defaults = |ast: &Ast, root: NodeId| {
            find_all_children::<KtReferenceExpression>(ast, root).into_iter().any(|it| it.text(ast) == default_object_name)
        };
        let most_visible_composable = find_all_children::<KtFunction>(ast, file.node())
            .into_iter()
            .filter(|it| is_composable(ast, it.node()))
            .filter(|it| it.name(ast).as_deref() == Some(composable_name))
            .filter(|composable| {
                let has_reference_in_parameters = composable
                    .value_parameters(ast)
                    .into_iter()
                    .filter_map(|it| it.default_value(ast))
                    .any(|it| references_defaults(ast, it));
                if has_reference_in_parameters {
                    return true;
                }
                composable.body_block_expression(ast).is_some_and(|body| references_defaults(ast, body.node()))
            })
            .fold(None::<KtFunction>, |best, it| match best {
                Some(b) if visibility_int(ast, b.node()) >= visibility_int(ast, it.node()) => Some(b),
                _ => Some(it),
            });
        let Some(most_visible_composable) = most_visible_composable else { return };
        if visibility_int(ast, clazz.node()) < visibility_int(ast, most_visible_composable.node()) {
            let message = create_message(
                visibility_string(ast, most_visible_composable.node()),
                &default_object_name,
                visibility_string(ast, clazz.node()),
            );
            emitter.report(ast, clazz.node(), &message, false);
        }
    }
}

fn visibility_string(ast: &Ast, owner: NodeId) -> &'static str {
    if KtModifierListOwner::of(ast, owner).is_public(ast) {
        "public"
    } else if is_protected(ast, owner) {
        "protected"
    } else if is_internal(ast, owner) {
        "internal"
    } else if is_private(ast, owner) {
        "private"
    } else {
        "not supported"
    }
}

fn visibility_int(ast: &Ast, owner: NodeId) -> i32 {
    if KtModifierListOwner::of(ast, owner).is_public(ast) {
        4
    } else if is_internal(ast, owner) {
        3
    } else if is_protected(ast, owner) {
        2
    } else if is_private(ast, owner) {
        1
    } else {
        0
    }
}

pub fn create_message(composable_visibility: &str, default_object_name: &str, default_object_visibility: &str) -> String {
    format!(
        "`Defaults` objects should match visibility of the composables they serve. `{default_object_name}` is \
         {default_object_visibility} but it should be {composable_visibility}.\n\
         See https://mrmans0n.github.io/compose-rules/rules/#componentdefaults-object-should-match-the-composable-visibility for more information."
    )
}
