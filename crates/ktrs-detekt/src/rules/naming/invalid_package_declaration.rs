//! `InvalidPackageDeclaration.kt`.

use std::cell::OnceCell;
use std::path::Component;
use std::sync::Arc;

use ktrs_psi::{FqName, KtPackageDirective, kt_visitor_void};

use crate::api::{Config, Entity, Finding, Rule, RuleBase, config_property};
use crate::kt_file;

/// Reports when the file location does not match the declared package.
pub struct InvalidPackageDeclaration {
    base: RuleBase,
    root_package: OnceCell<String>,
    require_root_in_declaration: OnceCell<bool>,
}

impl InvalidPackageDeclaration {
    pub fn new(config: Arc<dyn Config>) -> Self {
        InvalidPackageDeclaration {
            base: RuleBase::new(config, "Kotlin source files should be stored in the directory corresponding to its package statement."),
            root_package: OnceCell::new(),
            require_root_in_declaration: OnceCell::new(),
        }
    }

    fn root_package(&self) -> &str {
        self.root_package.get_or_init(|| config_property::string(self.base.config.as_ref(), "rootPackage", ""))
    }

    fn require_root_in_declaration(&self) -> bool {
        *self.require_root_in_declaration.get_or_init(|| config_property::boolean(self.base.config.as_ref(), "requireRootInDeclaration", false))
    }

    fn report_invalid_package_declaration(&mut self, element: &KtPackageDirective, message: &str) {
        self.report(Finding::new(Entity::from(element), message));
    }
}

impl Rule for InvalidPackageDeclaration {
    crate::rule_base!(InvalidPackageDeclaration);
}

crate::detekt_visitor! {
    impl InvalidPackageDeclaration {
        fn visit_package_directive(&mut self, directive: &KtPackageDirective) {
            kt_visitor_void::visit_package_directive(self, directive);
            let package_name = directive.fq_name();
            if !package_name.is_root() {
                let root_package_name = FqName::new(self.root_package());
                if self.require_root_in_declaration() && !package_name.starts_with(&root_package_name) {
                    self.report_invalid_package_declaration(directive, "The package declaration is missing the root package");
                    return;
                }

                let context = kt_file::containing_file(directive);
                let parent = context.absolute_path().parent();
                let directories = parent.into_iter().flat_map(|parent| parent.components()).filter_map(|component| match component {
                    Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
                    _ => None,
                });
                let normalized_file_path = to_normalized_form(directories);
                let expected_path = to_normalized_form(without_prefix(&package_name, &root_package_name).into_iter());

                let is_in_root_package = expected_path.is_empty();
                if !is_in_root_package && !normalized_file_path.ends_with(&format!("|{expected_path}")) {
                    self.report_invalid_package_declaration(directive, "The package declaration does not match the actual file location.");
                }
            }
        }
    }
}

fn to_normalized_form(elements: impl Iterator<Item = String>) -> String {
    elements.collect::<Vec<_>>().join("|")
}

fn without_prefix(name: &FqName, prefix: &FqName) -> Vec<String> {
    let drop_count = if name.starts_with(prefix) { prefix.path_segments().len() } else { 0 };
    name.path_segments().into_iter().skip(drop_count).collect()
}
