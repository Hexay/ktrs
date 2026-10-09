//! `EmptyKotlinFile.kt`.

use ktrs_psi::KtFile;

use crate::api::{Entity, Finding, Rule};
use crate::kt_file;

empty_rule!(EmptyKotlinFile);

crate::detekt_visitor! {
    impl EmptyKotlinFile {
        fn visit_kt_file(&mut self, file: &KtFile) {
            let context = kt_file::containing_file(file);
            let mut text = context.text().to_owned();
            if let Some(package_directive) = file.package_directive() {
                text.replace_range(package_directive.start_offset()..package_directive.end_offset(), "");
            }
            if text.chars().all(char::is_whitespace) {
                let message = format!("The empty Kotlin file {} can be removed.", context.name());
                self.report(Finding::new(Entity::at_package_or_first_decl(file), message));
            }
        }
    }
}
