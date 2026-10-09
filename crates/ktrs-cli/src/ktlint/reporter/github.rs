//! ktrs-only (`ktrs lint --reporter github`, not a ktlint reporter): each error that was not autocorrected as a
//! GitHub annotation titled with its rule id (`crate::github_annotations`).

use crate::github_annotations::Annotations;
use crate::ktlint::console::Printer;
use crate::ktlint::reporter::{KtlintCliError, ReporterV2, Status, caused_by};

pub struct GithubReporter {
    out: Printer,
    annotations: Annotations,
}

impl GithubReporter {
    pub fn new(out: Printer, annotations: Annotations) -> GithubReporter {
        GithubReporter { out, annotations }
    }
}

impl ReporterV2 for GithubReporter {
    fn on_lint_error(&mut self, file: &str, ktlint_cli_error: &KtlintCliError) {
        if ktlint_cli_error.status != Status::FormatIsAutocorrected {
            let (line, col) = (ktlint_cli_error.line, Some(ktlint_cli_error.col));
            let annotation = self.annotations.error(file, line, col, &caused_by(ktlint_cli_error), &ktlint_cli_error.detail);
            self.out.print(&format!("{annotation}\n"));
        }
    }

    fn after_all(&mut self) {
        self.out.close();
    }
}
