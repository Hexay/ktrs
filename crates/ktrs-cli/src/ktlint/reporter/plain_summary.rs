//! Port of `ktlint-cli-reporter-plain-summary` (`PlainSummaryReporter`).

use crate::ktlint::console::Printer;
use crate::ktlint::reporter::{KtlintCliError, ReporterV2, Status, caused_by, increment, print_summary};

/// Reports a count per rule of the autocorrected errors and of those not autocorrected.
pub struct PlainSummaryReporter {
    out: Printer,
    rule_violation_count_autocorrected: Vec<(String, u64)>,
    rule_violation_count_no_autocorrection: Vec<(String, u64)>,
}

impl PlainSummaryReporter {
    pub fn new(out: Printer) -> PlainSummaryReporter {
        PlainSummaryReporter {
            out,
            rule_violation_count_autocorrected: Vec::new(),
            rule_violation_count_no_autocorrection: Vec::new(),
        }
    }
}

impl ReporterV2 for PlainSummaryReporter {
    fn on_lint_error(&mut self, _file: &str, ktlint_cli_error: &KtlintCliError) {
        if ktlint_cli_error.status == Status::FormatIsAutocorrected {
            increment(&mut self.rule_violation_count_autocorrected, &ktlint_cli_error.rule_id);
        } else {
            increment(&mut self.rule_violation_count_no_autocorrection, &caused_by(ktlint_cli_error));
        }
    }

    fn after_all(&mut self) {
        if !self.rule_violation_count_autocorrected.is_empty() {
            print_summary(
                &mut self.out,
                "Count (descending) of autocorrected errors by rule:",
                &self.rule_violation_count_autocorrected,
            );
        }
        if !self.rule_violation_count_no_autocorrection.is_empty() {
            if !self.rule_violation_count_autocorrected.is_empty() {
                self.out.println("");
            }
            print_summary(
                &mut self.out,
                "Count (descending) of errors not autocorrected by rule:",
                &self.rule_violation_count_no_autocorrection,
            );
        }
    }
}
