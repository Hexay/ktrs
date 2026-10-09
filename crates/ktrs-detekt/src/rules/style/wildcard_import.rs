//! `WildcardImport.kt`.

use std::cell::OnceCell;
use std::sync::Arc;

use ktrs_psi::KtImportDirective;

use crate::api::{Config, Entity, Finding, Rule, RuleBase, config_property};
use crate::kotlin::{contains_ignore_case, remove_prefix, remove_suffix};

/// Wildcard imports should be replaced with imports using fully qualified class names. This helps increase
/// clarity of which classes are imported and helps prevent naming conflicts.
pub struct WildcardImport {
    base: RuleBase,
    exclude_imports: OnceCell<Vec<String>>,
}

impl WildcardImport {
    pub fn new(config: Arc<dyn Config>) -> Self {
        WildcardImport {
            base: RuleBase::new(
                config,
                "Wildcard imports should be replaced with imports using fully qualified class names. \
                 Wildcard imports can lead to naming conflicts. \
                 A library update can introduce naming clashes with your classes which \
                 results in compilation errors.",
            ),
            exclude_imports: OnceCell::new(),
        }
    }

    fn exclude_imports(&self) -> &[String] {
        self.exclude_imports.get_or_init(|| {
            let imports = config_property::list(self.base.config.as_ref(), "excludeImports", &["java.util.*"]);
            imports.iter().map(|it| remove_suffix(remove_prefix(it, "*"), "*").to_owned()).collect()
        })
    }
}

impl Rule for WildcardImport {
    crate::rule_base!(WildcardImport);
}

crate::detekt_visitor! {
    impl WildcardImport {
        fn visit_import_directive(&mut self, import_directive: &KtImportDirective) {
            let import = import_directive.import_path().map(|path| path.path_str());
            if let Some(import) = import {
                if !import.contains('*') {
                    return;
                }

                if self.exclude_imports().iter().any(|it| contains_ignore_case(&import, it)) {
                    return;
                }
                self.report(Finding::new(
                    Entity::from(import_directive),
                    format!("{import} is a wildcard import. Replace it with fully qualified imports."),
                ));
            }
        }
    }
}
