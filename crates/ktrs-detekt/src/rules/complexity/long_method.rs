//! `LongMethod.kt`.

use std::cell::OnceCell;
use std::collections::BTreeMap;
use std::sync::Arc;

use ktrs_psi::{KtFile, KtNamedFunction, PsiElement, kt_visitor_void};
use ktrs_syntax::ElementId;

use crate::api::{Config, Entity, Finding, Rule, RuleBase, config_property};
use crate::metrics::lines_of_code;

/// Methods should have one responsibility. Long methods can indicate that a method handles too many cases at
/// once. Prefer smaller methods with clear names that describe their functionality clearly.
pub struct LongMethod {
    base: RuleBase,
    allowed_lines: OnceCell<i32>,
    // Keyed by element id: upstream's maps hash the PSI elements by identity, so its report order within a file
    // is arbitrary; here it is document order. `nestedFunctionTracking` is only ever written upstream: left out.
    function_to_lines_cache: BTreeMap<ElementId, i32>,
    function_to_body_lines_cache: BTreeMap<ElementId, i32>,
}

impl LongMethod {
    pub fn new(config: Arc<dyn Config>) -> Self {
        LongMethod {
            base: RuleBase::new(
                config,
                "One method should have one responsibility. Long methods tend to handle many things at once. \
                 Prefer smaller methods to make them easier to understand.",
            ),
            allowed_lines: OnceCell::new(),
            function_to_lines_cache: BTreeMap::new(),
            function_to_body_lines_cache: BTreeMap::new(),
        }
    }

    fn allowed_lines(&self) -> i32 {
        *self.allowed_lines.get_or_init(|| config_property::int(self.base.config.as_ref(), "allowedLines", 60))
    }
}

impl Rule for LongMethod {
    crate::rule_base!(LongMethod);

    fn pre_visit(&mut self, _root: &KtFile) {
        self.function_to_lines_cache.clear();
        self.function_to_body_lines_cache.clear();
    }

    fn post_visit(&mut self, root: &KtFile) {
        let mut function_to_lines = BTreeMap::new();
        for (&function, &lines) in &self.function_to_lines_cache {
            let is_nested = root.at(function).get_parent_of_type::<KtNamedFunction>(true).is_some();
            if is_nested {
                function_to_lines.insert(function, self.function_to_body_lines_cache.get(&function).copied().unwrap_or(0));
            } else {
                function_to_lines.insert(function, lines);
            }
        }
        for (function, lines) in function_to_lines {
            if lines > self.allowed_lines() {
                let function = root.at(function).upcast::<KtNamedFunction>();
                let message =
                    format!("The function {} is too long ({lines}). The maximum length is {}.", function.name_as_safe_name(), self.allowed_lines());
                self.report(Finding::new(Entity::at_name(&function), message));
            }
        }
    }
}

crate::detekt_visitor! {
    impl LongMethod {
        fn visit_named_function(&mut self, function: &KtNamedFunction) {
            let parent_methods = function.get_parent_of_type::<KtNamedFunction>(true);
            let body_entity = function.body_block_expression().map(PsiElement::from).or_else(|| function.body_expression().map(PsiElement::from));
            let lines = match (&parent_methods, &body_entity) {
                (Some(_), _) => lines_of_code(function) as i32,
                (None, Some(body_entity)) => lines_of_code(body_entity) as i32,
                (None, None) => 0,
            };
            self.function_to_lines_cache.insert(function.id(), lines);
            self.function_to_body_lines_cache.insert(function.id(), body_entity.as_ref().map_or(0, |body| lines_of_code(body) as i32));
            kt_visitor_void::visit_named_function(self, function);

            let nested_lines: i32 = function
                .find_children_of_type::<KtNamedFunction>()
                .iter()
                .map(|next| self.function_to_lines_cache.get(&next.id()).copied().unwrap_or(0))
                .sum();
            if nested_lines > 0 {
                self.function_to_lines_cache.insert(function.id(), lines - nested_lines);
            }
        }
    }
}
