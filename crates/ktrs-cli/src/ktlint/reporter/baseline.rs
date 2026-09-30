//! Port of `ktlint-cli-reporter-baseline`'s `BaselineReporter`.

use std::path::{Path, PathBuf};

use crate::ktlint::console::Printer;
use crate::ktlint::reporter::{
    KtlintCliError, ReporterV2, Status, accumulate, escape_xml_attr_value, relative_to_or_self, sorted_by_file,
};

pub struct BaselineReporter {
    out: Printer,
    root_dir_path: PathBuf,
    acc: Vec<(String, Vec<KtlintCliError>)>,
}

impl BaselineReporter {
    /// `root_dir_path`: the working directory (`Paths.get("").toAbsolutePath()`).
    pub fn new(out: Printer, root_dir_path: PathBuf) -> BaselineReporter {
        BaselineReporter { out, root_dir_path, acc: Vec::new() }
    }
}

impl ReporterV2 for BaselineReporter {
    fn on_lint_error(&mut self, file: &str, ktlint_cli_error: &KtlintCliError) {
        if ktlint_cli_error.status != Status::FormatIsAutocorrected {
            accumulate(&mut self.acc, file, ktlint_cli_error);
        }
    }

    fn after_all(&mut self) {
        let out = &mut self.out;
        out.println(r#"<?xml version="1.0" encoding="utf-8"?>"#);
        out.println(r#"<baseline version="1.0">"#);
        for (file, err_list) in sorted_by_file(&self.acc) {
            // Relative, so a baseline checked into a repository applies wherever it's checked out.
            let relative_file = relative_to_or_self(Path::new(file), &self.root_dir_path)
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/");
            out.println(&format!(r#"    <file name="{}">"#, escape_xml_attr_value(&relative_file)));
            for err in err_list {
                out.println(&format!(r#"        <error line="{}" column="{}" source="{}" />"#, err.line, err.col, err.rule_id));
            }
            out.println("    </file>");
        }
        out.println("</baseline>");
    }
}
