//! Port of `ktlint-cli-reporter-checkstyle` (`CheckStyleReporter`).

use crate::ktlint::console::Printer;
use crate::ktlint::reporter::{KtlintCliError, ReporterV2, Status, accumulate, escape_xml_attr_value, sorted_by_file};

pub struct CheckStyleReporter {
    out: Printer,
    acc: Vec<(String, Vec<KtlintCliError>)>,
}

impl CheckStyleReporter {
    pub fn new(out: Printer) -> CheckStyleReporter {
        CheckStyleReporter { out, acc: Vec::new() }
    }
}

impl ReporterV2 for CheckStyleReporter {
    fn on_lint_error(&mut self, file: &str, ktlint_cli_error: &KtlintCliError) {
        if ktlint_cli_error.status != Status::FormatIsAutocorrected {
            accumulate(&mut self.acc, file, ktlint_cli_error);
        }
    }

    fn after_all(&mut self) {
        let out = &mut self.out;
        out.println(r#"<?xml version="1.0" encoding="utf-8"?>"#);
        out.println(r#"<checkstyle version="8.0">"#);
        for (file, err_list) in sorted_by_file(&self.acc) {
            out.println(&format!(r#"    <file name="{}">"#, escape_xml_attr_value(file)));
            for err in err_list {
                let message = escape_xml_attr_value(&err.detail);
                out.println(&format!(
                    r#"        <error line="{}" column="{}" severity="error" message="{message}" source="{}" />"#,
                    err.line, err.col, err.rule_id
                ));
            }
            out.println("    </file>");
        }
        out.println("</checkstyle>");
    }
}
