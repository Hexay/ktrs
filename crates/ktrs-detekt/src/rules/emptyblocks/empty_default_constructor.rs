//! `EmptyDefaultConstructor.kt`.

use ktrs_psi::{KtClass, KtClassOrObject, KtPrimaryConstructor};
use ktrs_syntax::SyntaxKind::{self, PUBLIC_KEYWORD};

use crate::api::{Entity, Finding, Rule};

empty_rule!(EmptyDefaultConstructor);

crate::detekt_visitor! {
    impl EmptyDefaultConstructor {
        fn visit_primary_constructor(&mut self, constructor: &KtPrimaryConstructor) {
            if has_suitable_signature(constructor) && is_not_called(constructor) && !is_expected_or_actual_class(constructor) {
                self.report(Finding::new(Entity::from(constructor), "An empty default constructor can be removed."));
            }
        }
    }
}

/// Classes with the 'expect' or 'actual' keyword need the explicit default constructor - #1362 and #3929
fn is_expected_or_actual_class(constructor: &KtPrimaryConstructor) -> bool {
    match constructor.parent().and_then(|parent| parent.cast::<KtClass>()) {
        Some(parent) => parent.has_expect_modifier() || parent.has_actual_modifier(),
        None => false,
    }
}

fn has_suitable_signature(constructor: &KtPrimaryConstructor) -> bool {
    has_public_visibility(constructor.visibility_modifier_type())
        && constructor.annotation_entries().is_empty()
        && constructor.value_parameters().is_empty()
}

fn has_public_visibility(visibility: Option<SyntaxKind>) -> bool {
    visibility.is_none_or(|visibility| visibility == PUBLIC_KEYWORD)
}

fn is_not_called(constructor: &KtPrimaryConstructor) -> bool {
    let containing_class_or_object = constructor.parent().and_then(|parent| parent.cast::<KtClassOrObject>());
    let secondary_constructors = containing_class_or_object.map(|c| c.secondary_constructors()).unwrap_or_default();
    !secondary_constructors
        .iter()
        .any(|it| it.delegation_call().is_some_and(|call| call.is_call_to_this() && call.value_arguments().is_empty()))
}
