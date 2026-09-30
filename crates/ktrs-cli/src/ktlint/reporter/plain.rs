//! Port of `ktlint-cli-reporter-plain` (`PlainReporter`, `PlainReporterProvider`).

use crate::ktlint::console::Printer;
use crate::ktlint::reporter::{
    Color, KtlintCliError, ReporterOptions, ReporterV2, Status, accumulate, caused_by, color_file_name, empty_or_true,
    increment, print_summary,
};

/// Reports the errors which have not been autocorrected, then a count per rule.
pub struct PlainReporter {
    out: Printer,
    group_by_file: bool,
    should_color_output: bool,
    output_color: Color,
    pad: bool,
    acc: Vec<(String, Vec<KtlintCliError>)>,
    rule_violation_count: Vec<(String, u64)>,
}

impl PlainReporter {
    pub fn new(out: Printer, group_by_file: bool, should_color_output: bool, output_color: Color, pad: bool) -> PlainReporter {
        PlainReporter {
            out,
            group_by_file,
            should_color_output,
            output_color,
            pad,
            acc: Vec::new(),
            rule_violation_count: Vec::new(),
        }
    }

    /// `PlainReporter(out)` with the defaults.
    pub fn with_defaults(out: Printer) -> PlainReporter {
        PlainReporter::new(out, false, false, Color::DarkGray, false)
    }

    fn colored(&self, text: &str) -> String {
        if self.should_color_output { self.output_color.paint(text) } else { text.to_owned() }
    }

    fn color_file_name(&self, file_name: &str) -> String {
        color_file_name(file_name, |t| self.colored(t))
    }
}

impl ReporterV2 for PlainReporter {
    fn on_lint_error(&mut self, file: &str, ktlint_cli_error: &KtlintCliError) {
        if ktlint_cli_error.status != Status::FormatIsAutocorrected {
            if self.group_by_file {
                accumulate(&mut self.acc, file, ktlint_cli_error);
            } else {
                let column = if self.pad {
                    format!("{:<4}", ktlint_cli_error.col)
                } else {
                    ktlint_cli_error.col.to_string()
                };
                let line = format!(
                    "{}{}{}{} {} {}",
                    self.color_file_name(file),
                    self.colored(":"),
                    ktlint_cli_error.line,
                    self.colored(&format!(":{column}:")),
                    ktlint_cli_error.detail,
                    self.colored(&format!("({})", ktlint_cli_error.rule_id)),
                );
                self.out.println(&line);
            }
            increment(&mut self.rule_violation_count, &caused_by(ktlint_cli_error));
        }
    }

    fn after(&mut self, file: &str) {
        if self.group_by_file {
            let Some(index) = self.acc.iter().position(|(f, _)| f == file) else { return };
            let header = self.color_file_name(file);
            self.out.println(&header);
            let errors = self.acc[index].1.clone();
            for err in errors {
                let column = if self.pad { format!("{:<3}", err.col) } else { err.col.to_string() };
                let line = format!(
                    "  {}{} {} {}",
                    err.line,
                    self.colored(&format!(":{column}")),
                    err.detail,
                    self.colored(&format!("({})", err.rule_id)),
                );
                self.out.println(&line);
            }
        }
    }

    fn after_all(&mut self) {
        if !self.rule_violation_count.is_empty() {
            self.out.println("");
            print_summary(&mut self.out, "Summary error count (descending) by rule:", &self.rule_violation_count);
        }
    }
}

pub struct PlainReporterProvider;

impl PlainReporterProvider {
    pub const ID: &'static str = "plain";

    pub fn get(out: Printer, opt: &ReporterOptions) -> Result<PlainReporter, String> {
        let group_by_file = empty_or_true(opt.get("group_by_file"));
        let should_color_output = empty_or_true(opt.get("color"));
        let output_color = Color::from_option(opt.get("color_name"))?;
        let pad = empty_or_true(opt.get("pad"));
        Ok(PlainReporter::new(out, group_by_file, should_color_output, output_color, pad))
    }
}
