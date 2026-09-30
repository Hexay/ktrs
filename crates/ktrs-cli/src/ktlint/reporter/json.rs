//! Port of `ktlint-cli-reporter-json` (`JsonReporter`).

use crate::ktlint::console::Printer;
use crate::ktlint::reporter::{KtlintCliError, ReporterV2, Status, accumulate, sorted_by_file};

pub struct JsonReporter {
    out: Printer,
    acc: Vec<(String, Vec<KtlintCliError>)>,
}

impl JsonReporter {
    pub fn new(out: Printer) -> JsonReporter {
        JsonReporter { out, acc: Vec::new() }
    }
}

impl ReporterV2 for JsonReporter {
    fn on_lint_error(&mut self, file: &str, ktlint_cli_error: &KtlintCliError) {
        if ktlint_cli_error.status != Status::FormatIsAutocorrected {
            accumulate(&mut self.acc, file, ktlint_cli_error);
        }
    }

    fn after_all(&mut self) {
        let out = &mut self.out;
        out.println("[");
        let sorted = sorted_by_file(&self.acc);
        let index_last = sorted.len().wrapping_sub(1);
        for (index, (file, err_list)) in sorted.into_iter().enumerate() {
            out.println("    {");
            out.println(&format!("        \"file\": \"{}\",", escape_json_value(file)));
            out.println("        \"errors\": [");
            let err_index_last = err_list.len().wrapping_sub(1);
            for (err_index, err) in err_list.iter().enumerate() {
                out.println("            {");
                out.println(&format!("                \"line\": {},", err.line));
                out.println(&format!("                \"column\": {},", err.col));
                out.println(&format!("                \"message\": \"{}\",", escape_json_value(&err.detail)));
                out.println(&format!("                \"rule\": \"{}\"", err.rule_id));
                out.println(&format!("            }}{}", if err_index != err_index_last { "," } else { "" }));
            }
            out.println("        ]");
            out.println(&format!("    }}{}", if index != index_last { "," } else { "" }));
        }
        out.println("]");
    }
}

fn escape_json_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\u{8}', "\\b")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
