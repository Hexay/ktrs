//! `TooManyFunctions.kt`.

use std::cell::OnceCell;
use std::sync::Arc;

use ktrs_psi::{KtClass, KtClassOrObject, KtFile, KtNamedFunction, KtObjectDeclaration, kt_visitor_void};

use crate::api::{Config, Entity, Finding, Rule, RuleBase, config_property};
use crate::kt_file;
use crate::psi::{has_annotation, is_internal, is_override};

pub const DEFAULT_THRESHOLD: i32 = 11;
const DEPRECATED: &str = "Deprecated";

/// This rule reports files, classes, interfaces, objects and enums which contain too many functions.
pub struct TooManyFunctions {
    base: RuleBase,
    allowed_functions_per_file: OnceCell<i32>,
    allowed_functions_per_class: OnceCell<i32>,
    allowed_functions_per_interface: OnceCell<i32>,
    allowed_functions_per_object: OnceCell<i32>,
    allowed_functions_per_enum: OnceCell<i32>,
    ignore_deprecated: OnceCell<bool>,
    ignore_private: OnceCell<bool>,
    ignore_internal: OnceCell<bool>,
    ignore_overridden: OnceCell<bool>,
    ignore_annotated_functions: OnceCell<Vec<String>>,
    amount_of_top_level_functions: i32,
}

macro_rules! threshold {
    ($name:ident, $key:literal) => {
        fn $name(&self) -> i32 {
            *self.$name.get_or_init(|| config_property::int(self.base.config.as_ref(), $key, DEFAULT_THRESHOLD))
        }
    };
}

macro_rules! flag {
    ($name:ident, $key:literal) => {
        fn $name(&self) -> bool {
            *self.$name.get_or_init(|| config_property::boolean(self.base.config.as_ref(), $key, false))
        }
    };
}

impl TooManyFunctions {
    pub fn new(config: Arc<dyn Config>) -> Self {
        TooManyFunctions {
            base: RuleBase::new(
                config,
                "Too many functions inside a/an file/class/object/interface always indicate a violation of \
                 the single responsibility principle. Maybe the file/class/object/interface wants to manage too \
                 many things at once. Extract functionality which clearly belongs together.",
            ),
            allowed_functions_per_file: OnceCell::new(),
            allowed_functions_per_class: OnceCell::new(),
            allowed_functions_per_interface: OnceCell::new(),
            allowed_functions_per_object: OnceCell::new(),
            allowed_functions_per_enum: OnceCell::new(),
            ignore_deprecated: OnceCell::new(),
            ignore_private: OnceCell::new(),
            ignore_internal: OnceCell::new(),
            ignore_overridden: OnceCell::new(),
            ignore_annotated_functions: OnceCell::new(),
            amount_of_top_level_functions: 0,
        }
    }

    threshold!(allowed_functions_per_file, "allowedFunctionsPerFile");
    threshold!(allowed_functions_per_class, "allowedFunctionsPerClass");
    threshold!(allowed_functions_per_interface, "allowedFunctionsPerInterface");
    threshold!(allowed_functions_per_object, "allowedFunctionsPerObject");
    threshold!(allowed_functions_per_enum, "allowedFunctionsPerEnum");
    flag!(ignore_deprecated, "ignoreDeprecated");
    flag!(ignore_private, "ignorePrivate");
    flag!(ignore_internal, "ignoreInternal");
    flag!(ignore_overridden, "ignoreOverridden");

    fn ignore_annotated_functions(&self) -> &[String] {
        self.ignore_annotated_functions.get_or_init(|| config_property::list(self.base.config.as_ref(), "ignoreAnnotatedFunctions", &[]))
    }

    fn calc_functions(&self, class_or_object: &KtClassOrObject) -> i32 {
        let Some(body) = class_or_object.body() else { return 0 };
        let functions = body.declarations().into_iter().filter_map(|declaration| declaration.cast::<KtNamedFunction>());
        functions.filter(|it| !self.is_ignored_function(it)).count() as i32
    }

    fn is_ignored_function(&self, function: &KtNamedFunction) -> bool {
        (self.ignore_deprecated() && has_annotation(function, &[DEPRECATED]))
            || (self.ignore_private() && function.is_private())
            || (self.ignore_internal() && is_internal(function))
            || (self.ignore_overridden() && is_override(function))
            || self.ignore_annotated_functions().iter().any(|it| has_annotation(function, &[it.as_str()]))
    }
}

impl Rule for TooManyFunctions {
    crate::rule_base!(TooManyFunctions);
}

crate::detekt_visitor! {
    impl TooManyFunctions {
        fn visit_kt_file(&mut self, file: &KtFile) {
            kt_visitor_void::visit_kt_file(self, file);
            if self.amount_of_top_level_functions > self.allowed_functions_per_file() {
                let message = format!(
                    "File '{}' with '{}' functions detected. The maximum allowed functions per file is set to '{}'",
                    kt_file::containing_file(file).name(),
                    self.amount_of_top_level_functions,
                    self.allowed_functions_per_file()
                );
                self.report(Finding::new(Entity::at_package_or_first_decl(file), message));
            }
            self.amount_of_top_level_functions = 0;
        }

        fn visit_named_function(&mut self, function: &KtNamedFunction) {
            if function.is_top_level() && !self.is_ignored_function(function) {
                self.amount_of_top_level_functions += 1;
            }
        }

        fn visit_class(&mut self, klass: &KtClass) {
            let amount = self.calc_functions(&klass.upcast());
            let name = klass.name().unwrap_or_else(|| "null".to_owned());
            if klass.is_interface() {
                if amount > self.allowed_functions_per_interface() {
                    let message = format!(
                        "Interface '{name}' with '{amount}' functions detected. The maximum allowed functions per interface is set to '{}'",
                        self.allowed_functions_per_interface()
                    );
                    self.report(Finding::new(Entity::at_name(klass), message));
                }
            } else if klass.is_enum() {
                if amount > self.allowed_functions_per_enum() {
                    let message = format!(
                        "Enum class '{name}' with '{amount}' functions detected. The maximum allowed functions per enum class is set to '{}'",
                        self.allowed_functions_per_enum()
                    );
                    self.report(Finding::new(Entity::at_name(klass), message));
                }
            } else if amount > self.allowed_functions_per_class() {
                let message = format!(
                    "Class '{name}' with '{amount}' functions detected. The maximum allowed functions per class is set to '{}'",
                    self.allowed_functions_per_class()
                );
                self.report(Finding::new(Entity::at_name(klass), message));
            }
            kt_visitor_void::visit_class(self, klass);
        }

        fn visit_object_declaration(&mut self, declaration: &KtObjectDeclaration) {
            let amount = self.calc_functions(&declaration.upcast());
            if amount > self.allowed_functions_per_object() {
                let message = format!(
                    "Object '{}' with '{amount}' functions detected. The maximum allowed functions per object is set to '{}'",
                    declaration.name().unwrap_or_else(|| "null".to_owned()),
                    self.allowed_functions_per_object()
                );
                self.report(Finding::new(Entity::at_name(declaration), message));
            }
            kt_visitor_void::visit_object_declaration(self, declaration);
        }
    }
}
