//! Port of `ktlint-cli-reporter-baseline`'s `Baseline.kt`: loading a baseline of errors to ignore.

pub mod xml;

use std::collections::HashMap;
use std::path::Path;

use ktrs_lint::editorconfig::KtlintVersion;

use crate::ktlint::console::Console;
use crate::ktlint::logger::{BASELINE, Logger};
use crate::ktlint::reporter::{KtlintCliError, Status};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaselineStatus {
    /// Not requested.
    Disabled,
    Valid,
    /// Does not exist yet: the CLI then writes it.
    NotFound,
    /// Could not be parsed (and was deleted): the CLI then rewrites it.
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Baseline {
    pub path: Option<String>,
    pub status: BaselineStatus,
    /// Errors by the (relative) file path, without their detail.
    pub lint_errors_per_file: HashMap<String, Vec<KtlintCliError>>,
}

impl Baseline {
    pub fn disabled() -> Baseline {
        Baseline { path: None, status: BaselineStatus::Disabled, lint_errors_per_file: HashMap::new() }
    }

    fn with_status(path: &str, status: BaselineStatus) -> Baseline {
        Baseline { path: Some(path.to_owned()), status, lint_errors_per_file: HashMap::new() }
    }
}

/// An exception out of `BaselineLoader.load()`: its message, and the `[Fatal Error]` line Xerces printed.
struct LoadError {
    message: String,
    fatal_error: Option<xml::XmlError>,
}

/// `loadBaseline(path, BaselineErrorHandling.LOG)`: a baseline that can't be loaded is deleted, its error
/// logged, and reported as [`BaselineStatus::Invalid`]. `path` is resolved against `working_dir`. 1.8 takes a
/// rule id without rule set as `standard:` and warns once (2.0 matches ids exactly).
pub fn load_baseline(path: &str, working_dir: &Path, logger: &Logger, console: &Console, ktlint_version: KtlintVersion) -> Baseline {
    let baseline_path = Some(working_dir.join(path)).filter(|p| p.exists());
    match load(path, baseline_path.as_deref()) {
        Ok(mut baseline) if ktlint_version.is_1_8() => {
            let rule_reference_without_rule_set_id_prefix = prefix_rule_ids_without_rule_set(&mut baseline);
            if rule_reference_without_rule_set_id_prefix > 0 {
                logger.warn(BASELINE, || {
                    format!(
                        "Baseline file '{path}' contains {rule_reference_without_rule_set_id_prefix} reference(s) to rule ids \
                         without a rule set id. For those references the rule set id 'standard' is assumed. It is advised to \
                         regenerate this baseline file."
                    )
                });
            }
            baseline
        }
        Ok(baseline) => baseline,
        Err(e) => {
            if let Some(fatal) = &e.fatal_error {
                console.println_err(&format!("[Fatal Error] :{}:{}: {}", fatal.line, fatal.col, fatal.message));
            }
            if let Some(file) = &baseline_path {
                let _ = std::fs::remove_file(file);
            }
            logger.error(BASELINE, || e.message);
            Baseline::with_status(path, BaselineStatus::Invalid)
        }
    }
}

fn load(path: &str, baseline_path: Option<&Path>) -> Result<Baseline, LoadError> {
    if path.trim().is_empty() {
        return Err(LoadError {
            message: "Path for loading baseline may not be blank or empty".to_owned(),
            fatal_error: None,
        });
    }
    let Some(baseline_file) = baseline_path else {
        return Ok(Baseline::with_status(path, BaselineStatus::NotFound));
    };
    let unable_to_parse = |fatal_error| LoadError { message: format!("Unable to parse baseline file: {path}"), fatal_error };
    let bytes = std::fs::read(baseline_file).map_err(|_| unable_to_parse(None))?;
    let document = xml::parse_document(&String::from_utf8_lossy(&bytes)).map_err(|e| unable_to_parse(Some(e)))?;
    let lint_errors_per_file = parse_baseline(&document)
        .map_err(|message| LoadError { message, fatal_error: None })?;
    Ok(Baseline { path: Some(path.to_owned()), status: BaselineStatus::Valid, lint_errors_per_file })
}

fn parse_baseline(document: &xml::Element) -> Result<HashMap<String, Vec<KtlintCliError>>, String> {
    let mut lint_errors_per_file = HashMap::new();
    let mut files = Vec::new();
    document.elements_by_tag_name("file", &mut files);
    for file in files {
        let mut errors = Vec::new();
        file.elements_by_tag_name("error", &mut errors);
        let errors = errors.into_iter().map(parse_baseline_error_element).collect::<Result<Vec<_>, _>>()?;
        lint_errors_per_file.insert(file.attribute("name").to_owned(), errors);
    }
    Ok(lint_errors_per_file)
}

fn parse_baseline_error_element(element: &xml::Element) -> Result<KtlintCliError, String> {
    Ok(KtlintCliError {
        line: to_int(element.attribute("line"))?,
        col: to_int(element.attribute("column"))?,
        rule_id: element.attribute("source").to_owned(),
        detail: String::new(),
        status: Status::BaselineIgnored,
    })
}

/// 1.8 `RuleId.prefixWithStandardRuleSetIdWhenMissing` on every `source`; the number of ids changed.
fn prefix_rule_ids_without_rule_set(baseline: &mut Baseline) -> usize {
    let mut prefixed = 0;
    for error in baseline.lint_errors_per_file.values_mut().flatten() {
        if !error.rule_id.contains(':') {
            error.rule_id = format!("standard:{}", error.rule_id);
            prefixed += 1;
        }
    }
    prefixed
}

/// Kotlin's `String.toInt()`; `Err` is the `NumberFormatException` message. Negative values (valid ints
/// upstream, never written by ktlint) are rejected too.
fn to_int(value: &str) -> Result<usize, String> {
    value
        .strip_prefix('+')
        .unwrap_or(value)
        .parse::<u32>()
        .ok()
        .filter(|&v| v <= i32::MAX as u32 && !value.starts_with("++"))
        .map(|v| v as usize)
        .ok_or_else(|| format!("For input string: \"{value}\""))
}

/// `List<KtlintCliError>.containsLintError(error)`: same line, column and rule (the baseline has no detail).
pub fn contains_lint_error(list: &[KtlintCliError], ktlint_cli_error: &KtlintCliError) -> bool {
    list.iter().any(|e| is_same_as(e, ktlint_cli_error))
}

/// `List<KtlintCliError>.doesNotContain(error)`.
pub fn does_not_contain(list: &[KtlintCliError], ktlint_cli_error: &KtlintCliError) -> bool {
    !contains_lint_error(list, ktlint_cli_error)
}

fn is_same_as(a: &KtlintCliError, b: &KtlintCliError) -> bool {
    a.col == b.col && a.line == b.line && a.rule_id == b.rule_id
}
