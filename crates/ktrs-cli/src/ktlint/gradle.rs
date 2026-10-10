//! Hidden, ktrs-only options for the ktlint-gradle drop-in (`io.github.hexay.ktrs.ktlint`, research/29). That plugin
//! runs ktlint's engine in-process with its own reporting; these options make a CLI run behave the same way:
//!
//! | option | effect |
//! |---|---|
//! | `--ktrs-gradle-events=<file>` | ktlint-gradle mode, and every reported error to `<file>` with its status ([`GradleEventsReporter`]). `--format` then formats like the engine's `format(code, callback: (LintError, Boolean))`: everything autocorrectable is fixed (baseline errors too), and the reporters get every error found, fixed ones included, with lint statuses and the plain detail |
//! | `--ktrs-relative-to=<dir>` | with `--relative`, report paths relative to `<dir>` (ktlint-gradle: the root project) instead of the working directory; the baseline stays keyed by the working directory |
//!
//! In ktlint-gradle mode reporters also get paths with the platform's separators ([`native_separators`]).
//! | `--ktrs-editorconfig-override=<name>=<value>` | repeatable: an `EditorConfigOverride` entry, which wins over every `.editorconfig` (`ktlint { additionalEditorconfig }`), resolved like `EditorConfigPropertyRegistry.find` |
//!
//! A run handed to the ktlint jar ([`crate::ktlint::ktlint_jar`]) drops them, and writes the events file with the
//! `json` reporter instead (`hand_off_args`).

use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::Ordering;

use ktrs_lint::{AutocorrectDecision, Code};

use ktrs_lint::editorconfig::{
    CODE_STYLE_PROPERTY, END_OF_LINE_PROPERTY, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, INSERT_FINAL_NEWLINE_PROPERTY,
    MAX_LINE_LENGTH_PROPERTY, PropertyRef, RuleExecution, create_rule_execution_editor_config_property,
    create_rule_set_execution_editor_config_property,
};
use ktrs_lint::rule_provider::RuleV2Provider;

use crate::ktlint::baseline::does_not_contain;
use crate::ktlint::command_line::Exit;
use crate::ktlint::process::Processor;
use crate::ktlint::reporter::{KtlintCliError, ReporterV2, Status};

pub const EVENTS_OPTION: &str = "--ktrs-gradle-events";
pub const RELATIVE_TO_OPTION: &str = "--ktrs-relative-to";
pub const EDITOR_CONFIG_OVERRIDE_OPTION: &str = "--ktrs-editorconfig-override";

/// First line of an events file.
pub(crate) const EVENTS_HEADER: &str = "# ktrs-gradle-events 1";

/// A `file\t<file>` line per linted file (ktlint-gradle's `LintErrorResult`s), then one per error:
/// `error\t<file>\t<line>\t<col>\t<rule id>\t<status>\t<corrected>\t<detail>`, the status as `KtlintCliError.Status`
/// names it; `\`, tab, line feed and carriage return in the detail escaped as `\\`, `\t`, `\n`, `\r`. Files come
/// in the order the run reports them (its arguments' order).
pub struct GradleEventsReporter {
    out: BufWriter<File>,
}

impl GradleEventsReporter {
    pub fn new(file: File) -> GradleEventsReporter {
        GradleEventsReporter { out: BufWriter::new(file) }
    }
}

impl ReporterV2 for GradleEventsReporter {
    fn before_all(&mut self) {
        let _ = writeln!(self.out, "{EVENTS_HEADER}");
    }

    fn before(&mut self, file: &str) {
        let _ = writeln!(self.out, "file\t{file}");
    }

    fn on_lint_error(&mut self, file: &str, e: &KtlintCliError) {
        let _ = writeln!(self.out, "{}", error_event(file, e));
    }

    fn after_all(&mut self) {
        let _ = self.out.flush();
    }
}

/// The `error` line of an events file ([`GradleEventsReporter`]).
pub(crate) fn error_event(file: &str, e: &KtlintCliError) -> String {
    let status = match e.status {
        Status::BaselineIgnored => "BASELINE_IGNORED",
        Status::LintCanNotBeAutocorrected => "LINT_CAN_NOT_BE_AUTOCORRECTED",
        Status::LintCanBeAutocorrected => "LINT_CAN_BE_AUTOCORRECTED",
        Status::FormatIsAutocorrected => "FORMAT_IS_AUTOCORRECTED",
        Status::KotlinParseException => "KOTLIN_PARSE_EXCEPTION",
        Status::KtlintRuleEngineException => "KTLINT_RULE_ENGINE_EXCEPTION",
    };
    let detail = e.detail.replace('\\', "\\\\").replace('\t', "\\t").replace('\n', "\\n").replace('\r', "\\r");
    format!("error\t{file}\t{}\t{}\t{}\t{status}\t{}\t{detail}", e.line, e.col, e.rule_id, e.corrected)
}

impl Processor<'_> {
    /// ktlint-gradle's `KtLintInvocation100.invokeFormat` + its reports: the engine's `format(code, callback:
    /// (LintError, Boolean))` fixes all it can; every distinct error is reported with its lint status and plain
    /// detail, fixed ones flagged [`KtlintCliError::corrected`]; errors in the baseline are not reported.
    pub(crate) fn gradle_format(&self, code: &Code, baseline_lint_errors: &[KtlintCliError]) -> Result<Vec<KtlintCliError>, Exit> {
        let mut ktlint_cli_errors: Vec<KtlintCliError> = Vec::new();
        let result = self.engine.format_reporting(code, &mut |_| AutocorrectDecision::AllowAutocorrect, &mut |e, corrected| {
            let status = if e.can_be_auto_corrected { Status::LintCanBeAutocorrected } else { Status::LintCanNotBeAutocorrected };
            let mut ktlint_cli_error = KtlintCliError::new(e.line, e.col, e.rule_id.value(), &e.detail, status);
            ktlint_cli_error.corrected = corrected;
            if does_not_contain(baseline_lint_errors, &ktlint_cli_error) {
                if !corrected {
                    self.contains_unfixed_lint_errors.store(true, Ordering::SeqCst);
                }
                ktlint_cli_errors.push(ktlint_cli_error);
            }
        });
        match result {
            Ok(formatted_file_content) => {
                if code.content != formatted_file_content
                    && let Some(path) = &code.file_path
                    && let Err(e) = std::fs::write(path, &formatted_file_content)
                {
                    return Err(Exit::Crash(format!("java.io.FileNotFoundException: {} ({e})", path.display())));
                }
            }
            Err(e) => {
                ktlint_cli_errors.push(self.to_ktlint_cli_error(&e, code)?);
                self.contains_unfixed_lint_errors.store(true, Ordering::SeqCst);
            }
        }
        Ok(ktlint_cli_errors)
    }
}

/// `EditorConfigOverride.from(entries.mapKeys { EditorConfigPropertyRegistry(providers).find(it.key) })`; `Err` is the
/// `EditorConfigPropertyNotFoundException` message.
pub fn editor_config_override(entries: &[String], providers: &[RuleV2Provider]) -> Result<Vec<(PropertyRef, Option<String>)>, String> {
    let core: [PropertyRef; 6] = [
        (&*CODE_STYLE_PROPERTY).into(),
        (&*END_OF_LINE_PROPERTY).into(),
        (&*INDENT_STYLE_PROPERTY).into(),
        (&*INDENT_SIZE_PROPERTY).into(),
        (&*INSERT_FINAL_NEWLINE_PROPERTY).into(),
        (&*MAX_LINE_LENGTH_PROPERTY).into(),
    ];
    let known: Vec<&PropertyRef> = providers.iter().flat_map(|p| p.uses_editor_config_properties()).chain(&core).collect();
    entries
        .iter()
        .map(|entry| {
            let (name, value) = entry.split_once('=').ok_or_else(|| format!("{EDITOR_CONFIG_OVERRIDE_OPTION} needs name=value, got '{entry}'"))?;
            let property = match known.iter().find(|p| p.name() == name) {
                Some(property) => (*property).clone(),
                None => rule_execution_property(name).ok_or_else(|| not_found(name, &known))?,
            };
            Ok((property, Some(value.to_owned())))
        })
        .collect()
}

/// `toRuleExecutionPropertyOrNull`: `ktlint_<rule set>_<rule>` / `ktlint_<rule set>` with `_` read as `:`.
fn rule_execution_property(name: &str) -> Option<PropertyRef> {
    let id = name.strip_prefix("ktlint_")?.replace('_', ":");
    let valid = |part: &str| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    match id.split_once(':') {
        Some((rule_set, rule)) if valid(rule_set) && valid(rule) => {
            Some(create_rule_execution_editor_config_property(&id, RuleExecution::Enabled).into())
        }
        None if valid(&id) => Some(create_rule_set_execution_editor_config_property(&id, RuleExecution::Enabled).into()),
        _ => None,
    }
}

fn not_found(name: &str, known: &[&PropertyRef]) -> String {
    let mut names: Vec<&str> = known.iter().map(|p| p.name()).collect();
    names.sort_unstable();
    names.dedup();
    let list: String = names.iter().map(|n| format!("\n\t- {n}")).collect();
    format!(
        "EditorConfigPropertyNotFoundException: Property with name '{name}' is not found in any of given rules. Available \
         properties:{list}\nNext to properties above, the properties to enable or disable ktlint rules are allowed as well."
    )
}

/// ktlint-gradle hands reporters `File.absolutePath` / `File.toRelativeString`: the platform's separators (which
/// also decide the order of the `html` report, kept in a `ConcurrentHashMap` by path).
pub fn native_separators(route: &str) -> String {
    if cfg!(windows) { route.replace('/', "\\") } else { route.to_owned() }
}
