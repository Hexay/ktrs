//! `EmptyClassBlock.kt`.

use ktrs_psi::{KtClassOrObject, kt_visitor_void};

use super::contains_comments::class_has_comment_inside;
use crate::api::{Entity, Finding, Rule};

empty_rule!(EmptyClassBlock);

crate::detekt_visitor! {
    impl EmptyClassBlock {
        fn visit_class_or_object(&mut self, class_or_object: &KtClassOrObject) {
            kt_visitor_void::visit_class_or_object(self, class_or_object);
            if class_or_object.is_object_literal() {
                return;
            }
            if class_has_comment_inside(class_or_object) {
                return;
            }

            if let Some(body) = class_or_object.body()
                && body.declarations().is_empty()
            {
                let name = class_or_object.name().unwrap_or_else(|| "null".to_owned());
                self.report(Finding::new(Entity::from(&body), format!("The class or object {name} is empty.")));
            }
        }
    }
}
