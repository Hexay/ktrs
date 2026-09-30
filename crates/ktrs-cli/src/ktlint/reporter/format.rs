//! Port of `ktlint-cli-reporter-format` (`FormatReporter`, `FormatReporterProvider`).

use std::collections::HashMap;

use crate::ktlint::console::Printer;
use crate::ktlint::reporter::{Color, KtlintCliError, ReporterOptions, ReporterV2, Status, color_file_name, empty_or_true};

/// Prints one line per file: whether formatting is (or was) needed and whether it is complete.
pub struct FormatReporter {
    out: Printer,
    format: bool,
    should_color_output: bool,
    output_color: Color,
    count_auto_correct_possible_or_done: HashMap<String, usize>,
    count_can_not_be_auto_corrected: HashMap<String, usize>,
}

impl FormatReporter {
    pub fn new(out: Printer, format: bool, should_color_output: bool, output_color: Color) -> FormatReporter {
        FormatReporter {
            out,
            format,
            should_color_output,
            output_color,
            count_auto_correct_possible_or_done: HashMap::new(),
            count_can_not_be_auto_corrected: HashMap::new(),
        }
    }

    fn colored(&self, text: &str) -> String {
        if self.should_color_output { self.output_color.paint(text) } else { text.to_owned() }
    }
}

impl ReporterV2 for FormatReporter {
    fn on_lint_error(&mut self, file: &str, ktlint_cli_error: &KtlintCliError) {
        let counts = match ktlint_cli_error.status {
            Status::LintCanBeAutocorrected | Status::FormatIsAutocorrected => &mut self.count_auto_correct_possible_or_done,
            _ => &mut self.count_can_not_be_auto_corrected,
        };
        *counts.entry(file.to_owned()).or_insert(0) += 1;
    }

    fn after(&mut self, file: &str) {
        let can_not_be_autocorrected = self.count_can_not_be_auto_corrected.get(file).copied().unwrap_or(0);
        let result = match can_not_be_autocorrected {
            1 if self.format => "Format not completed (1 violation needs manual fixing)".to_owned(),
            1 => "Format required (1 violation needs manual fixing)".to_owned(),
            n if n > 1 && self.format => format!("Format not completed ({n} violations need manual fixing)"),
            n if n > 1 => format!("Format required ({n} violations need manual fixing)"),
            _ if self.count_auto_correct_possible_or_done.get(file).copied().unwrap_or(0) > 0 => {
                if self.format {
                    "Format completed (all violations have been fixed)".to_owned()
                } else {
                    "Format required (all violations can be autocorrected)".to_owned()
                }
            }
            _ => "Format not needed (no violations found)".to_owned(),
        };
        let line = format!("{}{} {result}", color_file_name(file, |t| self.colored(t)), self.colored(":"));
        self.out.println(&line);
    }
}

pub struct FormatReporterProvider;

impl FormatReporterProvider {
    pub const ID: &'static str = "format";

    pub fn get(out: Printer, opt: &ReporterOptions) -> Result<FormatReporter, String> {
        let format = match opt.get("format") {
            None => {
                return Err("java.lang.IllegalArgumentException: Format is not specified in config options".to_owned());
            }
            Some("true") => true,
            Some("false") => false,
            Some(other) => {
                return Err(format!(
                    "java.lang.IllegalArgumentException: The string doesn't represent a boolean value: {other}"
                ));
            }
        };
        let should_color_output = empty_or_true(opt.get("color"));
        let output_color = Color::from_option(opt.get("color_name"))?;
        Ok(FormatReporter::new(out, format, should_color_output, output_color))
    }
}
